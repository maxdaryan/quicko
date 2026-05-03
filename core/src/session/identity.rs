//! Ephemeral identity generation.
//!
//! Each session creates a random, non-persistent identity:
//! - 128-bit random session ID
//! - Human-readable display name ("Swift Falcon #7A3F")
//! - Short invite code for peer discovery ("KBQW-E3TU")

use rand::{rngs::OsRng, Rng, RngCore};
use std::time::{Duration, Instant};

use crate::crypto::KeyPair;

/// An ephemeral session identity — exists only in memory.
pub struct SessionIdentity {
    /// Unique session identifier (hex-encoded 128-bit random).
    session_id: String,
    /// Human-readable display name.
    display_name: String,
    /// Short invite code for sharing.
    invite_code: String,
    /// X25519 keypair for this session.
    keypair: KeyPair,
    /// When this session was created.
    created_at: Instant,
    /// Session time-to-live.
    ttl: Duration,
}

// Animal names for display name generation
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

impl SessionIdentity {
    /// Generate a new random session identity.
    pub fn generate(ttl: Duration) -> Self {
        let mut rng = OsRng;

        // Generate 128-bit random session ID
        let mut id_bytes = [0u8; 16];
        rng.fill_bytes(&mut id_bytes);
        let session_id = hex::encode(id_bytes);

        // Generate display name: "Adjective Animal #XXXX"
        let adj = ADJECTIVES[rng.gen_range(0..ADJECTIVES.len())];
        let animal = ANIMALS[rng.gen_range(0..ANIMALS.len())];
        let suffix: u16 = rng.gen();
        let display_name = format!("{} {} #{:04X}", adj, animal, suffix);

        // Generate invite code: 8 chars of base32
        let mut invite_bytes = [0u8; 5];
        rng.fill_bytes(&mut invite_bytes);
        let raw_code = base32::encode(base32::Alphabet::Crockford, &invite_bytes);
        let invite_code = format!("{}-{}", &raw_code[..4], &raw_code[4..8]);

        // Generate keypair
        let keypair = KeyPair::generate();

        Self {
            session_id,
            display_name,
            invite_code,
            keypair,
            created_at: Instant::now(),
            ttl,
        }
    }

    /// Get the session ID (hex string).
    pub fn session_id(&self) -> &str {
        &self.session_id
    }

    /// Get the human-readable display name.
    pub fn display_name(&self) -> &str {
        &self.display_name
    }

    /// Get the invite code for sharing.
    pub fn invite_code(&self) -> &str {
        &self.invite_code
    }

    /// Get a reference to the keypair.
    pub fn keypair(&self) -> &KeyPair {
        &self.keypair
    }

    /// Get the public key bytes.
    pub fn public_key_bytes(&self) -> [u8; 32] {
        self.keypair.public_key_bytes()
    }

    /// Check if this session has expired.
    pub fn is_expired(&self) -> bool {
        self.created_at.elapsed() >= self.ttl
    }

    /// Get remaining time before expiry.
    pub fn remaining_ttl(&self) -> Duration {
        self.ttl.saturating_sub(self.created_at.elapsed())
    }

    /// Get the TTL duration.
    pub fn ttl(&self) -> Duration {
        self.ttl
    }
}

// We need hex encoding — add a minimal inline implementation
// to avoid an extra dependency
mod hex {
    const HEX_CHARS: &[u8; 16] = b"0123456789abcdef";

    pub fn encode(bytes: &[u8]) -> String {
        let mut s = String::with_capacity(bytes.len() * 2);
        for &b in bytes {
            s.push(HEX_CHARS[(b >> 4) as usize] as char);
            s.push(HEX_CHARS[(b & 0x0f) as usize] as char);
        }
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identity_generation() {
        let identity = SessionIdentity::generate(Duration::from_secs(3600));

        // Session ID is 32 hex chars (128 bits)
        assert_eq!(identity.session_id().len(), 32);

        // Display name matches pattern
        assert!(identity.display_name().contains('#'));

        // Invite code is formatted as XXXX-XXXX
        assert_eq!(identity.invite_code().len(), 9);
        assert_eq!(identity.invite_code().chars().nth(4), Some('-'));
    }

    #[test]
    fn test_unique_identities() {
        let id1 = SessionIdentity::generate(Duration::from_secs(3600));
        let id2 = SessionIdentity::generate(Duration::from_secs(3600));

        assert_ne!(id1.session_id(), id2.session_id());
        assert_ne!(id1.public_key_bytes(), id2.public_key_bytes());
    }

    #[test]
    fn test_ttl() {
        let identity = SessionIdentity::generate(Duration::from_secs(3600));
        assert!(!identity.is_expired());
        assert!(identity.remaining_ttl() > Duration::from_secs(3500));
    }

    #[test]
    fn test_expired_session() {
        let identity = SessionIdentity::generate(Duration::from_secs(0));
        // With 0 TTL, it should expire immediately
        std::thread::sleep(Duration::from_millis(10));
        assert!(identity.is_expired());
    }
}
