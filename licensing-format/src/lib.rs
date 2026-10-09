//! The licence wire format.
//!
//! Shared by the app (which only ever *verifies*) and the vendor signing tool
//! (which *signs*). Keeping one definition means the two can never disagree
//! about what a licence or a request code looks like.

use base32::Alphabet;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Bumped only if the wire format changes incompatibly.
pub const FORMAT_VERSION: u32 = 1;

/// The signed payload. Field order matters: it defines the signed bytes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct License {
    pub version: u32,
    /// Which signing key made this, so keys can be rotated.
    pub key_id: String,
    pub sku: String,
    /// `perpetual`, `term` or `version_locked`.
    pub plan: String,
    pub features: Vec<String>,
    pub issued_at: String,
    /// Absent for a perpetual licence.
    pub expires_at: Option<String>,
    /// `binding_for(...)` of the install this licence belongs to.
    pub binding: String,
}

impl License {
    /// The exact bytes the vendor signs and the app verifies.
    pub fn signing_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).expect("a licence always serialises")
    }
}

/// What gets written to a `.lic` file.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LicenseFile {
    pub license: License,
    /// Hex-encoded Ed25519 signature over `license.signing_bytes()`.
    pub signature: String,
}

/// What the app shows the user to request a licence.
///
/// Contains no personal data: a random install serial and a salted device hash.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RequestCode {
    pub version: u32,
    pub sku: String,
    pub app_serial: String,
    pub machine_fp: String,
    pub app_version: String,
    pub os: String,
}

/// Binds a licence to one installation on one machine.
pub fn binding_for(app_serial: &str, machine_fingerprint: &str, sku: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(app_serial.as_bytes());
    hasher.update([0u8]);
    hasher.update(machine_fingerprint.as_bytes());
    hasher.update([0u8]);
    hasher.update(sku.as_bytes());
    hex(&hasher.finalize())
}

pub fn sha256_hex(value: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(value);
    hex(&hasher.finalize())
}

pub fn hex(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push_str(&format!("{byte:02x}"));
    }
    out
}

pub fn unhex(text: &str) -> Option<Vec<u8>> {
    let cleaned = text.trim();
    if cleaned.len() % 2 != 0 {
        return None;
    }
    (0..cleaned.len())
        .step_by(2)
        .map(|index| u8::from_str_radix(cleaned.get(index..index + 2)?, 16).ok())
        .collect()
}

/// Encode bytes as grouped Crockford base32 — readable and hard to mistype.
pub fn encode_code(bytes: &[u8]) -> String {
    let encoded = base32::encode(Alphabet::Crockford, bytes);
    encoded
        .as_bytes()
        .chunks(5)
        .map(|chunk| std::str::from_utf8(chunk).unwrap_or_default())
        .collect::<Vec<_>>()
        .join("-")
}

/// Decode a code, ignoring grouping, dashes and case.
pub fn decode_code(text: &str) -> Option<Vec<u8>> {
    let cleaned: String = text
        .chars()
        .filter(|character| character.is_ascii_alphanumeric())
        .collect();
    base32::decode(Alphabet::Crockford, &cleaned.to_ascii_uppercase())
}

pub fn encode_request(request: &RequestCode) -> String {
    encode_code(&serde_json::to_vec(request).expect("a request always serialises"))
}

pub fn decode_request(text: &str) -> Option<RequestCode> {
    serde_json::from_slice(&decode_code(text)?).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request() -> RequestCode {
        RequestCode {
            version: FORMAT_VERSION,
            sku: "ledgerly".to_string(),
            app_serial: "018f2c1e-0000-7000-8000-000000000001".to_string(),
            machine_fp: "a1b2c3d4e5f60718293a4b5c6d7e8f90".to_string(),
            app_version: "0.1.0".to_string(),
            os: "windows".to_string(),
        }
    }

    #[test]
    fn request_codes_round_trip_through_text() {
        let original = request();
        let code = encode_request(&original);
        assert!(code.contains('-'), "codes are grouped for reading: {code}");

        let decoded = decode_request(&code).unwrap();
        assert_eq!(decoded, original);
    }

    #[test]
    fn decoding_ignores_grouping_and_case() {
        let code = encode_request(&request()).to_lowercase().replace('-', " ");
        assert_eq!(decode_request(&code).unwrap(), request());
    }

    #[test]
    fn binding_depends_on_all_three_inputs() {
        let base = binding_for("serial", "fingerprint", "ledgerly");

        assert_ne!(base, binding_for("other", "fingerprint", "ledgerly"));
        assert_ne!(base, binding_for("serial", "other", "ledgerly"));
        assert_ne!(base, binding_for("serial", "fingerprint", "other"));
        assert_eq!(base, binding_for("serial", "fingerprint", "ledgerly"));
        assert_eq!(base.len(), 64);
    }

    #[test]
    fn signing_bytes_are_stable_for_the_same_licence() {
        let license = License {
            version: FORMAT_VERSION,
            key_id: "ledgerly-dev".to_string(),
            sku: "ledgerly".to_string(),
            plan: "perpetual".to_string(),
            features: vec!["reports".to_string(), "pdf_templates".to_string()],
            issued_at: "2026-10-09".to_string(),
            expires_at: None,
            binding: "abc".to_string(),
        };

        assert_eq!(license.signing_bytes(), license.clone().signing_bytes());
        // Field order is part of the contract: a regression here breaks every
        // licence ever issued.
        let text = String::from_utf8(license.signing_bytes()).unwrap();
        assert!(text.starts_with("{\"version\":"), "unexpected field order: {text}");
    }

    #[test]
    fn hex_round_trips() {
        let bytes = vec![0u8, 1, 127, 128, 255];
        assert_eq!(unhex(&hex(&bytes)).unwrap(), bytes);
        assert!(unhex("xyz").is_none());
        assert!(unhex("abc").is_none(), "odd length is rejected");
    }
}
