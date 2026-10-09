//! Offline licensing.
//!
//! Free is the default. Pro is unlocked by an Ed25519-signed key that is bound
//! to this installation *and* this machine, verified locally — the app never
//! contacts a licence server, and never has.

mod seal;
mod ticket;

pub use ticket::{Entitlements, PRO_FEATURES};

use rusqlite::{params, Connection};
use serde::Serialize;

use ledgerly_license::{binding_for, RequestCode, FORMAT_VERSION};

use crate::error::{AppError, AppResult};

/// The product this build licenses.
pub const SKU: &str = "ledgerly";
pub const DEFAULT_TRIAL_DAYS: i64 = 14;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LicenseStatus {
    pub licensed: bool,
    /// `free`, `license` or `trial`.
    pub source: String,
    pub plan: Option<String>,
    pub key_id: Option<String>,
    pub expires_at: Option<String>,
    pub days_remaining: Option<i64>,
    pub features: Vec<String>,
    /// Identifiers, in short forms safe to show or read out.
    pub serial: String,
    pub machine_fingerprint: String,
    pub binding: String,
    pub trial_active: bool,
    pub trial_ends_at: Option<String>,
    pub trial_available: bool,
    /// Why a stored licence is not currently valid, if one is stored.
    pub problem: Option<String>,
}

impl LicenseStatus {
    pub fn entitlements(&self) -> Entitlements {
        Entitlements {
            licensed: self.licensed,
            source: self.source.clone(),
            plan: self.plan.clone(),
            features: self.features.clone(),
            expires_at: self.expires_at.clone(),
        }
    }
}

fn today(conn: &Connection) -> rusqlite::Result<String> {
    conn.query_row("SELECT date('now', 'localtime')", [], |row| row.get(0))
}

fn short(value: &str) -> String {
    value.chars().take(16).collect()
}

fn grouped(value: &str) -> String {
    value
        .as_bytes()
        .chunks(4)
        .map(|chunk| std::str::from_utf8(chunk).unwrap_or_default())
        .collect::<Vec<_>>()
        .join("-")
}

/// A salted hash of this machine's stable identifier. The raw identifier is
/// never stored and never leaves the machine.
pub fn machine_fingerprint() -> String {
    let raw = machine_uid::get().unwrap_or_else(|_| "unknown-machine".to_string());
    ledgerly_license::sha256_hex(format!("ledgerly-v1|{raw}").as_bytes())
        .chars()
        .take(32)
        .collect()
}

/// The per-install serial, generated once and kept sealed.
pub fn installation_serial(conn: &Connection) -> AppResult<String> {
    installation_serial_with_key(conn, &seal::keychain_key()?)
}

pub fn installation_serial_with_key(conn: &Connection, key: &[u8; 32]) -> AppResult<String> {
    let existing: Option<String> = conn.query_row(
        "SELECT serial_sealed FROM installation WHERE id = 'default'",
        [],
        |row| row.get(0),
    )?;

    if let Some(sealed) = existing {
        if let Ok(serial) = seal::open(key, &sealed) {
            return Ok(serial);
        }
    }

    // First run (or the sealing key was lost): make a fresh 128-bit serial.
    let mut bytes = [0u8; 16];
    getrandom::fill(&mut bytes).map_err(|error| AppError::Validation(error.to_string()))?;
    let serial = ledgerly_license::hex(&bytes);

    conn.execute(
        "UPDATE installation SET serial_sealed = ?1, machine_fp = ?2 WHERE id = 'default'",
        params![seal::seal(key, &serial)?, machine_fingerprint()],
    )?;

    Ok(serial)
}

/// What a licence for this installation must be bound to.
pub fn binding_for_here(serial: &str) -> String {
    binding_for(serial, &machine_fingerprint(), SKU)
}

fn trial_state(conn: &Connection) -> rusqlite::Result<(bool, Option<String>, bool)> {
    let (started_at, ends_at): (Option<String>, Option<String>) = conn.query_row(
        "SELECT started_at, ends_at FROM trial WHERE id = 'default'",
        [],
        |row| Ok((row.get(0)?, row.get(1)?)),
    )?;

    let today = today(conn)?;
    let active = matches!(&ends_at, Some(ends) if ends.as_str() >= today.as_str());

    Ok((active, ends_at, started_at.is_none()))
}

fn build_status(
    conn: &Connection,
    serial: Option<String>,
    key_problem: Option<String>,
) -> AppResult<LicenseStatus> {
    let fingerprint = machine_fingerprint();
    let binding = serial.as_ref().map(|value| binding_for_here(value));
    let today = today(conn)?;
    let (trial_active, trial_ends_at, trial_available) = trial_state(conn)?;

    let stored: Option<String> = conn
        .query_row("SELECT file_json FROM license LIMIT 1", [], |row| {
            row.get(0)
        })
        .ok();

    let mut problem = key_problem;
    let mut licensed = false;
    let mut source = "free".to_string();
    let mut plan = None;
    let mut key_id = None;
    let mut expires_at = None;
    let mut days_remaining = None;
    let mut features: Vec<String> = Vec::new();

    if let Some(file_json) = stored {
        match (|| -> AppResult<ledgerly_license::License> {
            let file = ticket::parse_file(&file_json)?;
            let binding = binding.as_deref().ok_or_else(|| {
                AppError::Validation("The installation serial is unavailable.".to_string())
            })?;
            ticket::verify(&file, binding, &today)
        })() {
            Ok(license) => {
                licensed = true;
                source = "license".to_string();
                plan = Some(license.plan.clone());
                key_id = Some(license.key_id.clone());
                expires_at = license.expires_at.clone();
                days_remaining = remaining_days(conn, license.expires_at.as_deref())?;
                features = license.features.clone();

                conn.execute("UPDATE license SET last_verified_at = datetime('now')", [])?;
            }
            Err(error) => problem = Some(error.to_string()),
        }
    }

    // A trial grants everything, but only while it lasts and nothing better is active.
    if !licensed && trial_active {
        licensed = true;
        source = "trial".to_string();
        plan = Some("trial".to_string());
        expires_at = trial_ends_at.clone();
        days_remaining = remaining_days(conn, trial_ends_at.as_deref())?;
        features = PRO_FEATURES
            .iter()
            .map(|name| (*name).to_string())
            .collect();
    }

    Ok(LicenseStatus {
        licensed,
        source,
        plan,
        key_id,
        expires_at,
        days_remaining,
        features,
        serial: serial.as_deref().map(grouped).unwrap_or_default(),
        machine_fingerprint: short(&fingerprint),
        binding: binding.as_deref().map(short).unwrap_or_default(),
        trial_active,
        trial_ends_at,
        trial_available,
        problem,
    })
}

fn remaining_days(conn: &Connection, until: Option<&str>) -> rusqlite::Result<Option<i64>> {
    let Some(until) = until else {
        return Ok(None);
    };
    conn.query_row(
        "SELECT CAST(julianday(?1) - julianday(date('now', 'localtime')) AS INTEGER)",
        [until],
        |row| row.get(0),
    )
    .map(Some)
}

/// The current licence state. A keychain failure degrades to Free rather than
/// breaking the app, and is reported in `problem`.
pub fn status(conn: &Connection) -> AppResult<LicenseStatus> {
    match installation_serial(conn) {
        Ok(serial) => build_status(conn, Some(serial), None),
        Err(error) => build_status(conn, None, Some(error.to_string())),
    }
}

/// Same as `status`, but with a caller-supplied sealing key so tests never need
/// the OS keychain.
#[cfg(test)]
pub fn status_with_key(conn: &Connection, key: &[u8; 32]) -> AppResult<LicenseStatus> {
    let serial = installation_serial_with_key(conn, key)?;
    build_status(conn, Some(serial), None)
}

/// The features this installation may use right now.
pub fn entitlements(conn: &Connection) -> AppResult<Entitlements> {
    Ok(status(conn)?.entitlements())
}

/// The request code as the user copies or saves it.
pub fn request_code_text(conn: &Connection) -> AppResult<String> {
    Ok(ledgerly_license::encode_request(&request_code(conn)?))
}

pub fn request_code(conn: &Connection) -> AppResult<RequestCode> {
    Ok(RequestCode {
        version: FORMAT_VERSION,
        sku: SKU.to_string(),
        app_serial: installation_serial(conn)?,
        machine_fp: machine_fingerprint(),
        app_version: env!("CARGO_PKG_VERSION").to_string(),
        os: std::env::consts::OS.to_string(),
    })
}

/// Import an upgrade key. Verification is local; a bad key is rejected outright.
pub fn activate(conn: &Connection, text: &str) -> AppResult<LicenseStatus> {
    let file = ticket::parse_file(text)?;
    let binding = binding_for_here(&installation_serial(conn)?);
    let license = ticket::verify(&file, &binding, &today(conn)?)?;

    conn.execute("DELETE FROM license", [])?;
    conn.execute(
        "INSERT INTO license (
            id, key_id, sku, plan, features_json, issued_at, expires_at, binding,
            file_json, last_verified_at
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, datetime('now'))",
        params![
            uuid::Uuid::now_v7().to_string(),
            license.key_id,
            license.sku,
            license.plan,
            serde_json::to_string(&license.features)
                .map_err(|error| AppError::Validation(error.to_string()))?,
            license.issued_at,
            license.expires_at,
            license.binding,
            text,
        ],
    )?;

    status(conn)
}

/// Forget the licence on this machine. No server is involved.
pub fn deactivate(conn: &Connection) -> AppResult<LicenseStatus> {
    conn.execute("DELETE FROM license", [])?;
    status(conn)
}

pub fn start_trial(conn: &Connection, days: i64) -> AppResult<LicenseStatus> {
    let (_, _, available) = trial_state(conn)?;
    if !available {
        return Err(AppError::Validation(
            "The trial has already been used on this installation.".to_string(),
        ));
    }

    conn.execute(
        "UPDATE trial
         SET started_at = date('now', 'localtime'),
             ends_at = date('now', 'localtime', ?1)
         WHERE id = 'default'",
        params![format!("+{days} days")],
    )?;

    status(conn)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::tests_support::temp_db;
    use ed25519_dalek::{Signer, SigningKey};
    use ledgerly_license::LicenseFile;

    const KEY: [u8; 32] = [5u8; 32];

    fn signed(binding: &str, expires_at: Option<&str>) -> String {
        // A licence signed by a throwaway key; only used to exercise loading.
        let license = ledgerly_license::License {
            version: FORMAT_VERSION,
            key_id: "test".to_string(),
            sku: SKU.to_string(),
            plan: "perpetual".to_string(),
            features: vec!["reports".to_string()],
            issued_at: "2026-01-01".to_string(),
            expires_at: expires_at.map(str::to_string),
            binding: binding.to_string(),
        };
        let signature = SigningKey::from_bytes(&[77u8; 32]).sign(&license.signing_bytes());
        serde_json::to_string(&LicenseFile {
            license,
            signature: ledgerly_license::hex(&signature.to_bytes()),
        })
        .unwrap()
    }

    #[test]
    fn a_fresh_install_has_a_serial_and_is_free() {
        let t = temp_db();
        let status = status_with_key(&t.conn, &KEY).unwrap();

        assert!(!status.licensed);
        assert_eq!(status.source, "free");
        assert!(status.trial_available);
        assert!(status.features.is_empty());
        assert!(!status.serial.is_empty());
        assert_eq!(status.serial.chars().filter(|c| *c == '-').count(), 7);

        let again = status_with_key(&t.conn, &KEY).unwrap();
        assert_eq!(again.serial, status.serial, "the serial must be stable");
    }

    #[test]
    fn the_serial_is_not_stored_in_the_clear() {
        let t = temp_db();
        let status = status_with_key(&t.conn, &KEY).unwrap();
        let sealed: String = t
            .conn
            .query_row(
                "SELECT serial_sealed FROM installation WHERE id = 'default'",
                [],
                |row| row.get(0),
            )
            .unwrap();

        let plain = status.serial.replace('-', "");
        assert!(
            !sealed.contains(&plain),
            "the raw serial must not be readable"
        );
        assert_ne!(sealed, plain);
    }

    #[test]
    fn a_trial_grants_pro_features_once() {
        let t = temp_db();

        let trialing = start_trial(&t.conn, DEFAULT_TRIAL_DAYS).unwrap();
        assert!(trialing.licensed);
        assert_eq!(trialing.source, "trial");
        assert!(trialing.trial_active);
        assert!(!trialing.trial_available);
        assert!(trialing.features.iter().any(|f| f == "reports"));
        assert!(trialing.days_remaining.unwrap_or(0) > 0);

        assert!(
            start_trial(&t.conn, DEFAULT_TRIAL_DAYS).is_err(),
            "a trial must not be restartable"
        );
    }

    #[test]
    fn an_expired_trial_does_not_grant_anything() {
        let t = temp_db();
        t.conn
            .execute(
                "UPDATE trial SET started_at = '2020-01-01', ends_at = '2020-01-15' WHERE id = 'default'",
                [],
            )
            .unwrap();

        let status = status_with_key(&t.conn, &KEY).unwrap();
        assert!(!status.licensed);
        assert!(!status.trial_active);
        assert!(!status.trial_available, "the trial was already used");
        assert!(!status.entitlements().allows("reports"));
    }

    // Signing/rejection details are covered in `ticket`; these check the storage
    // and reporting behaviour around a licence that does not verify.
    #[test]
    fn a_stored_licence_that_does_not_verify_is_reported_as_a_problem() {
        let t = temp_db();
        let foreign = signed("not-this-machine", None);
        t.conn
            .execute(
                "INSERT INTO license (id, key_id, sku, plan, features_json, issued_at, expires_at, binding, file_json)
                 VALUES ('x', 'test', 'ledgerly', 'perpetual', '[\"reports\"]', '2026-01-01', NULL, 'not-this-machine', ?1)",
                [&foreign],
            )
            .unwrap();

        let status = status_with_key(&t.conn, &KEY).unwrap();
        assert!(!status.licensed);
        assert!(
            status.features.is_empty(),
            "an invalid licence grants nothing"
        );
        assert!(status.problem.is_some(), "the reason should be reported");
    }

    #[test]
    fn an_unverifiable_key_is_refused_and_stores_nothing() {
        let t = temp_db();
        assert!(activate(&t.conn, &signed("not-this-machine", None)).is_err());

        let count: i64 = t
            .conn
            .query_row("SELECT COUNT(*) FROM license", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 0, "a rejected key must not be stored");
    }

    #[test]
    fn deactivating_with_nothing_else_returns_to_free() {
        let t = temp_db();

        let after = deactivate(&t.conn).unwrap();
        assert!(!after.licensed);
        assert_eq!(after.source, "free");
        assert!(after.features.is_empty());
    }

    #[test]
    fn a_running_trial_still_applies_after_a_licence_is_removed() {
        let t = temp_db();
        assert!(start_trial(&t.conn, DEFAULT_TRIAL_DAYS).unwrap().licensed);

        let after = deactivate(&t.conn).unwrap();
        assert!(after.licensed, "the trial outlives the licence");
        assert_eq!(after.source, "trial");
    }

    /// Touches the real OS keychain, so it stays out of the default run.
    #[test]
    #[ignore = "needs a real OS keychain"]
    fn the_keychain_holds_a_usable_sealing_key() {
        let t = temp_db();
        let serial = installation_serial(&t.conn).expect("keychain unavailable");
        assert_eq!(serial.len(), 32);
        assert_eq!(
            installation_serial(&t.conn).unwrap(),
            serial,
            "the serial must survive a round trip through the keychain"
        );
    }

    /// The real thing: sign a licence the way `tools/licensectl` does, then make
    /// the app accept it. Needs the keychain and the dev vendor key.
    #[test]
    #[ignore = "needs a real OS keychain and the vendor dev key"]
    fn a_licence_issued_for_this_installation_unlocks_pro() {
        use ed25519_dalek::{Signer, SigningKey};

        // The dev signing key lives outside the repo; skip when it is absent.
        let Ok(key_text) = std::fs::read_to_string("../.vendor/licensectl-key.txt") else {
            eprintln!("skipping: no vendor key at .vendor/licensectl-key.txt");
            return;
        };
        let secret: [u8; 32] = ledgerly_license::unhex(&key_text)
            .unwrap()
            .try_into()
            .unwrap();
        let signing = SigningKey::from_bytes(&secret);

        let t = temp_db();
        let binding = binding_for_here(&installation_serial(&t.conn).unwrap());

        let license = ledgerly_license::License {
            version: FORMAT_VERSION,
            key_id: "ledgerly-dev".to_string(),
            sku: SKU.to_string(),
            plan: "perpetual".to_string(),
            features: vec!["reports".to_string(), "branding".to_string()],
            issued_at: "2026-01-01".to_string(),
            expires_at: None,
            binding,
        };
        let signature = signing.sign(&license.signing_bytes());
        let file = serde_json::to_string(&ledgerly_license::LicenseFile {
            license,
            signature: ledgerly_license::hex(&signature.to_bytes()),
        })
        .unwrap();

        let activated = activate(&t.conn, &file).unwrap();

        assert!(activated.licensed);
        assert_eq!(activated.source, "license");
        assert_eq!(activated.key_id.as_deref(), Some("ledgerly-dev"));
        assert!(activated.problem.is_none());

        let entitlements = activated.entitlements();
        assert!(entitlements.allows("reports"));
        assert!(entitlements.allows("branding"));
        assert!(
            !entitlements.allows("batch_export"),
            "only the granted features are unlocked"
        );

        // And it is remembered (read back through the keychain, as on relaunch).
        let reloaded = status(&t.conn).unwrap();
        assert!(reloaded.licensed, "the licence must survive a restart");
        assert!(reloaded.problem.is_none());
    }

    #[test]
    fn garbage_is_refused_as_a_key() {
        let t = temp_db();
        assert!(activate(&t.conn, "not a licence at all").is_err());
        assert!(activate(&t.conn, "{}").is_err());
    }
}
