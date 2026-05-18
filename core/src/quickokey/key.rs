//! The core QuickoKey type — a 128-bit unified identity key.

use rand::rngs::OsRng;
use rand::RngCore;

use crate::crypto::kdf;
use crate::error::{QuickoError, Result};

const QUICKOKEY_PREFIX: &str = "QK";
const GROUP_COUNT: usize = 8;
const GROUP_SIZE: usize = 4;

/// A 128-bit unified identity key for Quicko2.
///
/// 128 bits = 2^128 ≈ 3.4e38 possible keys.
/// Format: `QK-1A2B-3C4D-5E6F-7890-ABCD-EF12-3456-7890`
#[derive(Clone)]
pub struct QuickoKey {
    raw: [u8; 16],
}

impl QuickoKey {
    /// Generate a new random QuickoKey using OS randomness.
    pub fn generate() -> Self {
        let mut raw = [0u8; 16];
        OsRng.fill_bytes(&mut raw);
        Self { raw }
    }

    /// Create a QuickoKey from raw 16-byte array.
    pub fn from_bytes(raw: [u8; 16]) -> Self {
        Self { raw }
    }

    /// Get the raw 128-bit key bytes.
    pub fn as_bytes(&self) -> &[u8; 16] {
        &self.raw
    }

    /// Format the key as `QK-1A2B-3C4D-5E6F-7890-ABCD-EF12-3456-7890`.
    pub fn format(&self) -> String {
        let hex = hex_encode(&self.raw);
        let groups: Vec<&str> = (0..GROUP_COUNT)
            .map(|i| &hex[i * GROUP_SIZE..(i + 1) * GROUP_SIZE])
            .collect();
        format!("{}-{}", QUICKOKEY_PREFIX, groups.join("-"))
    }

    /// Parse a formatted QuickoKey string.
    pub fn parse(s: &str) -> Result<Self> {
        let s = s.trim().to_uppercase();
        let hex_part = if s.starts_with("QK-") {
            s[3..].replace('-', "")
        } else if s.starts_with("QK") {
            s[2..].replace('-', "")
        } else {
            return Err(QuickoError::InvalidQuickoKey(
                "Must start with 'QK-' or 'QK'".to_string(),
            ));
        };
        if hex_part.len() != 32 {
            return Err(QuickoError::InvalidQuickoKey(format!(
                "Expected 32 hex chars, got {}", hex_part.len()
            )));
        }
        let bytes = hex_decode(&hex_part)
            .map_err(|e| QuickoError::InvalidQuickoKey(format!("Invalid hex: {}", e)))?;
        let mut raw = [0u8; 16];
        raw.copy_from_slice(&bytes);
        Ok(Self { raw })
    }

    /// Derive a deterministic X25519 static secret (32 bytes) from this key.
    pub fn derive_x25519_secret(&self) -> Result<[u8; 32]> {
        kdf::derive_x25519_from_quickokey(&self.raw)
    }

    /// Derive a session salt from this key.
    pub fn derive_session_salt(&self) -> Result<[u8; 32]> {
        kdf::derive_session_salt_from_quickokey(&self.raw)
    }

    /// Derive a deterministic display name: "Adjective Animal #XXXX".
    pub fn derive_display_name(&self) -> String {
        const ADJECTIVES: &[&str] = &[
            "Swift", "Silent", "Bright", "Bold", "Calm", "Crisp", "Dark", "Deft",
            "Fair", "Fast", "Keen", "Kind", "Lush", "Neat", "Pure", "Rare",
            "Sharp", "Sleek", "Slim", "Soft", "Sure", "Tame", "True", "Vast",
            "Warm", "Wild", "Wise", "Zen", "Cool", "Free", "Glow", "Haze",
        ];
        const ANIMALS: &[&str] = &[
            "Falcon", "Otter", "Panda", "Tiger", "Eagle", "Lynx", "Raven", "Wolf",
            "Heron", "Viper", "Crane", "Shark", "Koala", "Gecko", "Bison", "Whale",
            "Cobra", "Finch", "Hawk", "Moose", "Puma", "Swan", "Fox", "Owl",
            "Bear", "Dove", "Elk", "Jay", "Ram", "Yak", "Ibis", "Wren",
        ];
        let adj_idx = (self.raw[0] as usize) % ADJECTIVES.len();
        let animal_idx = (self.raw[1] as usize) % ANIMALS.len();
        let suffix = u16::from_be_bytes([self.raw[14], self.raw[15]]);
        format!("{} {} #{:04X}", ADJECTIVES[adj_idx], ANIMALS[animal_idx], suffix)
    }

    /// Short identifier for logging (first 8 hex chars).
    pub fn short_id(&self) -> String {
        hex_encode(&self.raw[..4])
    }
}

impl std::fmt::Debug for QuickoKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "QuickoKey({}...)", self.short_id())
    }
}

impl std::fmt::Display for QuickoKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.format())
    }
}

impl PartialEq for QuickoKey {
    fn eq(&self, other: &Self) -> bool {
        // Constant-time comparison to prevent timing attacks
        let mut result = 0u8;
        for (a, b) in self.raw.iter().zip(other.raw.iter()) {
            result |= a ^ b;
        }
        result == 0
    }
}

impl Eq for QuickoKey {}

impl std::hash::Hash for QuickoKey {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.raw.hash(state);
    }
}

// --- Hex helpers ---

const HEX_CHARS_UPPER: &[u8; 16] = b"0123456789ABCDEF";

fn hex_encode(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        s.push(HEX_CHARS_UPPER[(b >> 4) as usize] as char);
        s.push(HEX_CHARS_UPPER[(b & 0x0f) as usize] as char);
    }
    s
}

fn hex_decode(hex: &str) -> std::result::Result<Vec<u8>, String> {
    if hex.len() % 2 != 0 {
        return Err("Odd-length hex string".to_string());
    }
    let mut bytes = Vec::with_capacity(hex.len() / 2);
    let chars: Vec<u8> = hex.bytes().collect();
    for i in (0..chars.len()).step_by(2) {
        let high = hex_nibble(chars[i])?;
        let low = hex_nibble(chars[i + 1])?;
        bytes.push((high << 4) | low);
    }
    Ok(bytes)
}

fn hex_nibble(c: u8) -> std::result::Result<u8, String> {
    match c {
        b'0'..=b'9' => Ok(c - b'0'),
        b'a'..=b'f' => Ok(c - b'a' + 10),
        b'A'..=b'F' => Ok(c - b'A' + 10),
        _ => Err(format!("Invalid hex char: {}", c as char)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_is_16_bytes() {
        let key = QuickoKey::generate();
        assert_eq!(key.as_bytes().len(), 16);
        assert_ne!(*key.as_bytes(), [0u8; 16]);
    }

    #[test]
    fn test_unique_keys() {
        let k1 = QuickoKey::generate();
        let k2 = QuickoKey::generate();
        assert_ne!(k1, k2);
    }

    #[test]
    fn test_format_structure() {
        let key = QuickoKey::from_bytes([
            0x1A, 0x2B, 0x3C, 0x4D, 0x5E, 0x6F, 0x78, 0x90,
            0xAB, 0xCD, 0xEF, 0x12, 0x34, 0x56, 0x78, 0x90,
        ]);
        assert_eq!(key.format(), "QK-1A2B-3C4D-5E6F-7890-ABCD-EF12-3456-7890");
    }

    #[test]
    fn test_format_parse_roundtrip() {
        let original = QuickoKey::generate();
        let parsed = QuickoKey::parse(&original.format()).unwrap();
        assert_eq!(original, parsed);
    }

    #[test]
    fn test_parse_case_insensitive() {
        let key = QuickoKey::parse("qk-1a2b-3c4d-5e6f-7890-abcd-ef12-3456-7890").unwrap();
        assert_eq!(key.as_bytes()[0], 0x1A);
    }

    #[test]
    fn test_parse_invalid_prefix() {
        assert!(QuickoKey::parse("XX-1A2B-3C4D-5E6F-7890-ABCD-EF12-3456-7890").is_err());
    }

    #[test]
    fn test_derive_x25519_deterministic() {
        let key = QuickoKey::from_bytes([42u8; 16]);
        let s1 = key.derive_x25519_secret().unwrap();
        let s2 = key.derive_x25519_secret().unwrap();
        assert_eq!(s1, s2);
        assert_ne!(s1, [0u8; 32]);
    }

    #[test]
    fn test_derived_values_independent() {
        let key = QuickoKey::from_bytes([42u8; 16]);
        let x25519 = key.derive_x25519_secret().unwrap();
        let salt = key.derive_session_salt().unwrap();
        assert_ne!(x25519, salt);
    }

    #[test]
    fn test_deterministic_display_name() {
        let key = QuickoKey::from_bytes([0x1A; 16]);
        assert_eq!(key.derive_display_name(), key.derive_display_name());
        assert!(key.derive_display_name().contains('#'));
    }

    #[test]
    fn test_display_trait() {
        let key = QuickoKey::from_bytes([
            0x1A, 0x2B, 0x3C, 0x4D, 0x5E, 0x6F, 0x78, 0x90,
            0xAB, 0xCD, 0xEF, 0x12, 0x34, 0x56, 0x78, 0x90,
        ]);
        assert_eq!(format!("{}", key), "QK-1A2B-3C4D-5E6F-7890-ABCD-EF12-3456-7890");
    }
}
