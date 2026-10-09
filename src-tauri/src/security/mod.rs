//! Optional app lock.
//!
//! The passphrase is never stored — only an Argon2id verifier, with its own
//! salt. Nothing here needs the network, and unlocking is entirely local.

use argon2::password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString};
use argon2::Argon2;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

use crate::error::{AppError, AppResult};

pub const MIN_PASSPHRASE_LEN: usize = 8;
pub const DEFAULT_AUTO_LOCK_MINUTES: i64 = 15;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LockSettings {
    pub enabled: bool,
    pub auto_lock_minutes: i64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LockSettingsInput {
    pub passphrase: String,
    #[serde(default = "default_auto_lock")]
    pub auto_lock_minutes: i64,
}

fn default_auto_lock() -> i64 {
    DEFAULT_AUTO_LOCK_MINUTES
}

fn hash(passphrase: &str) -> AppResult<String> {
    let mut salt_bytes = [0u8; 16];
    getrandom::fill(&mut salt_bytes).map_err(|error| AppError::Validation(error.to_string()))?;
    let salt = SaltString::encode_b64(&salt_bytes)
        .map_err(|error| AppError::Validation(error.to_string()))?;

    Ok(Argon2::default()
        .hash_password(passphrase.as_bytes(), &salt)
        .map_err(|error| AppError::Validation(error.to_string()))?
        .to_string())
}

fn verifier(conn: &Connection) -> rusqlite::Result<Option<String>> {
    conn.query_row(
        "SELECT verifier FROM app_lock WHERE id = 'default'",
        [],
        |row| row.get(0),
    )
}

pub fn settings(conn: &Connection) -> rusqlite::Result<LockSettings> {
    conn.query_row(
        "SELECT enabled, auto_lock_minutes FROM app_lock WHERE id = 'default'",
        [],
        |row| {
            Ok(LockSettings {
                enabled: row.get::<_, i64>(0)? != 0,
                auto_lock_minutes: row.get(1)?,
            })
        },
    )
}

pub fn is_enabled(conn: &Connection) -> rusqlite::Result<bool> {
    Ok(settings(conn)?.enabled)
}

/// Does this passphrase open the lock?
pub fn verify(conn: &Connection, passphrase: &str) -> AppResult<bool> {
    let Some(stored) = verifier(conn)? else {
        return Ok(false);
    };

    let parsed = PasswordHash::new(&stored).map_err(|_| {
        AppError::Validation("The stored lock verifier is unreadable.".to_string())
    })?;

    Ok(Argon2::default()
        .verify_password(passphrase.as_bytes(), &parsed)
        .is_ok())
}

pub fn enable(
    conn: &Connection,
    passphrase: &str,
    auto_lock_minutes: i64,
) -> AppResult<LockSettings> {
    if passphrase.chars().count() < MIN_PASSPHRASE_LEN {
        return Err(AppError::Validation(format!(
            "Use at least {MIN_PASSPHRASE_LEN} characters for the passphrase."
        )));
    }

    conn.execute(
        "UPDATE app_lock
         SET verifier = ?1, enabled = 1, auto_lock_minutes = ?2, updated_at = datetime('now')
         WHERE id = 'default'",
        params![hash(passphrase)?, auto_lock_minutes.clamp(1, 240)],
    )?;

    Ok(settings(conn)?)
}

pub fn set_auto_lock(conn: &Connection, minutes: i64) -> AppResult<LockSettings> {
    conn.execute(
        "UPDATE app_lock SET auto_lock_minutes = ?1, updated_at = datetime('now')
         WHERE id = 'default'",
        params![minutes.clamp(1, 240)],
    )?;

    Ok(settings(conn)?)
}

/// Turn the lock off. The current passphrase is required.
pub fn disable(conn: &Connection, passphrase: &str) -> AppResult<LockSettings> {
    if !verify(conn, passphrase)? {
        return Err(AppError::Validation(
            "That passphrase is not correct.".to_string(),
        ));
    }

    conn.execute(
        "UPDATE app_lock SET verifier = NULL, enabled = 0, updated_at = datetime('now')
         WHERE id = 'default'",
        [],
    )?;

    Ok(settings(conn)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::tests_support::temp_db;

    #[test]
    fn a_fresh_database_has_no_lock() {
        let t = temp_db();
        let settings = settings(&t.conn).unwrap();

        assert!(!settings.enabled);
        assert_eq!(settings.auto_lock_minutes, DEFAULT_AUTO_LOCK_MINUTES);
        assert!(!verify(&t.conn, "").unwrap());
    }

    #[test]
    fn enabling_stores_only_a_verifier() {
        let t = temp_db();
        enable(&t.conn, "correct horse battery", 20).unwrap();

        assert!(settings(&t.conn).unwrap().enabled);
        assert!(verify(&t.conn, "correct horse battery").unwrap());
        assert!(!verify(&t.conn, "wrong passphrase").unwrap());

        // The passphrase itself must not be anywhere in the row.
        let stored: Option<String> = verifier(&t.conn).unwrap();
        let stored = stored.expect("a verifier should be stored");
        assert!(!stored.contains("correct horse battery"));
        assert!(stored.starts_with("$argon2"), "expected a PHC string, got {stored}");
    }

    #[test]
    fn the_same_passphrase_hashes_differently_each_time() {
        let t = temp_db();
        enable(&t.conn, "same passphrase here", 15).unwrap();
        let first = verifier(&t.conn).unwrap().unwrap();

        enable(&t.conn, "same passphrase here", 15).unwrap();
        let second = verifier(&t.conn).unwrap().unwrap();

        assert_ne!(first, second, "each hash must get its own salt");
        assert!(verify(&t.conn, "same passphrase here").unwrap());
    }

    #[test]
    fn a_short_passphrase_is_refused() {
        let t = temp_db();
        assert!(enable(&t.conn, "short", 15).is_err());
        assert!(!settings(&t.conn).unwrap().enabled);
    }

    #[test]
    fn disabling_needs_the_passphrase() {
        let t = temp_db();
        enable(&t.conn, "a decent passphrase", 15).unwrap();

        assert!(disable(&t.conn, "not it").is_err());
        assert!(settings(&t.conn).unwrap().enabled, "still locked");

        disable(&t.conn, "a decent passphrase").unwrap();
        assert!(!settings(&t.conn).unwrap().enabled);
        assert!(!verify(&t.conn, "a decent passphrase").unwrap());
    }

    #[test]
    fn the_auto_lock_delay_is_clamped() {
        let t = temp_db();
        assert_eq!(set_auto_lock(&t.conn, 0).unwrap().auto_lock_minutes, 1);
        assert_eq!(set_auto_lock(&t.conn, 100_000).unwrap().auto_lock_minutes, 240);
        assert_eq!(set_auto_lock(&t.conn, 30).unwrap().auto_lock_minutes, 30);
    }
}
