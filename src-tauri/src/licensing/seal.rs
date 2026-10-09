//! Sealing for the installation serial.
//!
//! The key is random and lives in the OS keychain (Credential Manager on
//! Windows, Keychain on macOS, Secret Service on Linux). Nothing here needs the
//! network, and the sealing key never leaves the machine.

use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Nonce};

use crate::error::{AppError, AppResult};

const SERVICE: &str = "com.ledgerly.desktop";
const ACCOUNT: &str = "installation-key";
const NONCE_LEN: usize = 12;

fn invalid(message: impl Into<String>) -> AppError {
    AppError::Validation(message.into())
}

/// Encrypt with a caller-supplied key; returns hex of `nonce || ciphertext`.
pub fn seal(key: &[u8; 32], plaintext: &str) -> AppResult<String> {
    let cipher =
        Aes256Gcm::new_from_slice(key).map_err(|error| invalid(format!("bad sealing key: {error}")))?;

    let mut nonce = [0u8; NONCE_LEN];
    getrandom::fill(&mut nonce).map_err(|error| invalid(format!("no randomness: {error}")))?;

    let ciphertext = cipher
        .encrypt(&Nonce::from(nonce), plaintext.as_bytes())
        .map_err(|_| invalid("could not seal the value"))?;

    let mut out = nonce.to_vec();
    out.extend_from_slice(&ciphertext);
    Ok(ledgerly_license::hex(&out))
}

pub fn open(key: &[u8; 32], sealed: &str) -> AppResult<String> {
    let bytes = ledgerly_license::unhex(sealed).ok_or_else(|| invalid("sealed value is not hex"))?;
    if bytes.len() <= NONCE_LEN {
        return Err(invalid("sealed value is too short"));
    }

    let (nonce, body) = bytes.split_at(NONCE_LEN);
    let cipher =
        Aes256Gcm::new_from_slice(key).map_err(|error| invalid(format!("bad sealing key: {error}")))?;

    let nonce = Nonce::try_from(nonce).map_err(|_| invalid("sealed value is too short"))?;
    let plaintext = cipher
        .decrypt(&nonce, body)
        .map_err(|_| invalid("could not unseal the value"))?;

    String::from_utf8(plaintext).map_err(|_| invalid("unsealed value is not text"))
}

/// The per-install sealing key, created in the keychain on first use.
pub fn keychain_key() -> AppResult<[u8; 32]> {
    let entry = keyring::Entry::new(SERVICE, ACCOUNT)
        .map_err(|error| invalid(format!("no keychain available: {error}")))?;

    if let Ok(stored) = entry.get_password() {
        if let Some(bytes) = ledgerly_license::unhex(&stored) {
            if let Ok(key) = <[u8; 32]>::try_from(bytes) {
                return Ok(key);
            }
        }
    }

    let mut key = [0u8; 32];
    getrandom::fill(&mut key).map_err(|error| invalid(format!("no randomness: {error}")))?;

    entry
        .set_password(&ledgerly_license::hex(&key))
        .map_err(|error| invalid(format!("could not store the sealing key: {error}")))?;

    Ok(key)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sealing_round_trips() {
        let key = [7u8; 32];
        let sealed = seal(&key, "018f2c1e-0000-7000-8000-000000000001").unwrap();

        assert_ne!(sealed, "018f2c1e-0000-7000-8000-000000000001");
        assert_eq!(
            open(&key, &sealed).unwrap(),
            "018f2c1e-0000-7000-8000-000000000001"
        );
    }

    #[test]
    fn sealing_is_randomised() {
        let key = [1u8; 32];
        assert_ne!(seal(&key, "same").unwrap(), seal(&key, "same").unwrap());
    }

    #[test]
    fn a_wrong_key_cannot_open_it() {
        let sealed = seal(&[1u8; 32], "secret").unwrap();
        assert!(open(&[2u8; 32], &sealed).is_err());
    }

    #[test]
    fn tampering_is_detected() {
        let key = [3u8; 32];
        let mut sealed = seal(&key, "secret").unwrap();
        // Flip the last hex digit.
        let last = if sealed.ends_with('0') { '1' } else { '0' };
        sealed.pop();
        sealed.push(last);

        assert!(open(&key, &sealed).is_err());
    }
}
