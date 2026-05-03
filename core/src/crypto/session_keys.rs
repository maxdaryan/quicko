//! Per-session key management with automatic rotation.

use std::time::{Duration, Instant};

use crate::crypto::{kdf, KeyPair};
use crate::error::Result;

/// Maximum messages before key rotation.
const ROTATION_MESSAGE_THRESHOLD: u64 = 100;

/// Maximum time before key rotation.
const ROTATION_TIME_THRESHOLD: Duration = Duration::from_secs(3600); // 1 hour

/// Manages encryption keys for a single peer session.
///
/// Handles key derivation from ECDH shared secrets and automatic
/// key rotation based on message count and time thresholds.
pub struct SessionKeys {
    /// Current encryption key (derived via HKDF).
    encryption_key: [u8; 32],
    /// Current authentication key (derived via HKDF).
    auth_key: [u8; 32],
    /// Our keypair for this session.
    local_keypair: KeyPair,
    /// Peer's public key.
    peer_public_key: [u8; 32],
    /// Messages encrypted with the current key.
    message_count: u64,
    /// When the current key was derived.
    key_created_at: Instant,
    /// Session-specific salt for HKDF.
    salt: Vec<u8>,
    /// Key generation counter (increments on each rotation).
    generation: u32,
}

impl SessionKeys {
    /// Create session keys from an ECDH key exchange.
    ///
    /// Performs X25519 DH and derives encryption + auth keys via HKDF.
    pub fn from_key_exchange(
        local_keypair: KeyPair,
        peer_public_key: [u8; 32],
        session_salt: &[u8],
    ) -> Result<Self> {
        let shared_secret = local_keypair.diffie_hellman(&peer_public_key);
        let (enc_key, auth_key) = kdf::derive_session_keys(&shared_secret, Some(session_salt))?;

        Ok(Self {
            encryption_key: enc_key,
            auth_key,
            local_keypair,
            peer_public_key,
            message_count: 0,
            key_created_at: Instant::now(),
            salt: session_salt.to_vec(),
            generation: 0,
        })
    }

    /// Get the current encryption key.
    pub fn encryption_key(&self) -> &[u8; 32] {
        &self.encryption_key
    }

    /// Get the current authentication key.
    pub fn auth_key(&self) -> &[u8; 32] {
        &self.auth_key
    }

    /// Get our public key bytes.
    pub fn local_public_key(&self) -> [u8; 32] {
        self.local_keypair.public_key_bytes()
    }

    /// Get the current key generation number.
    pub fn generation(&self) -> u32 {
        self.generation
    }

    /// Check if key rotation is needed.
    pub fn needs_rotation(&self) -> bool {
        self.message_count >= ROTATION_MESSAGE_THRESHOLD
            || self.key_created_at.elapsed() >= ROTATION_TIME_THRESHOLD
    }

    /// Rotate keys by generating a new keypair and re-deriving.
    ///
    /// Returns the new public key that must be sent to the peer.
    pub fn rotate(&mut self) -> Result<[u8; 32]> {
        // Generate new keypair
        let new_keypair = KeyPair::generate();
        let new_public = new_keypair.public_key_bytes();

        // Derive new shared secret with peer's existing public key
        let shared_secret = new_keypair.diffie_hellman(&self.peer_public_key);

        // Increment generation and use it in the salt for domain separation
        self.generation += 1;
        let mut rotation_salt = self.salt.clone();
        rotation_salt.extend_from_slice(&self.generation.to_be_bytes());

        let (enc_key, auth_key) =
            kdf::derive_session_keys(&shared_secret, Some(&rotation_salt))?;

        // Update state
        self.encryption_key = enc_key;
        self.auth_key = auth_key;
        self.local_keypair = new_keypair;
        self.message_count = 0;
        self.key_created_at = Instant::now();

        Ok(new_public)
    }

    /// Update the peer's public key (after they rotate).
    pub fn update_peer_key(&mut self, new_peer_public: [u8; 32]) -> Result<()> {
        self.peer_public_key = new_peer_public;

        // Re-derive keys with the new peer key
        let shared_secret = self.local_keypair.diffie_hellman(&self.peer_public_key);

        self.generation += 1;
        let mut rotation_salt = self.salt.clone();
        rotation_salt.extend_from_slice(&self.generation.to_be_bytes());

        let (enc_key, auth_key) =
            kdf::derive_session_keys(&shared_secret, Some(&rotation_salt))?;

        self.encryption_key = enc_key;
        self.auth_key = auth_key;
        self.message_count = 0;
        self.key_created_at = Instant::now();

        Ok(())
    }

    /// Record that a message was encrypted with the current key.
    pub fn increment_message_count(&mut self) {
        self.message_count += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_session_keys() -> (SessionKeys, KeyPair) {
        let alice = KeyPair::generate();
        let bob = KeyPair::generate();
        let salt = b"test-session-salt";

        let alice_keys = SessionKeys::from_key_exchange(
            alice,
            bob.public_key_bytes(),
            salt,
        )
        .unwrap();

        (alice_keys, bob)
    }

    #[test]
    fn test_session_keys_creation() {
        let (keys, _) = setup_session_keys();
        assert_ne!(*keys.encryption_key(), [0u8; 32]);
        assert_ne!(*keys.auth_key(), [0u8; 32]);
        assert_ne!(keys.encryption_key(), keys.auth_key());
        assert_eq!(keys.generation(), 0);
    }

    #[test]
    fn test_rotation_threshold() {
        let (mut keys, _) = setup_session_keys();

        assert!(!keys.needs_rotation());

        // Simulate 100 messages
        for _ in 0..100 {
            keys.increment_message_count();
        }

        assert!(keys.needs_rotation());
    }

    #[test]
    fn test_key_rotation() {
        let (mut keys, _) = setup_session_keys();

        let old_enc = *keys.encryption_key();
        let old_pub = keys.local_public_key();

        let new_pub = keys.rotate().unwrap();

        // New public key is different
        assert_ne!(new_pub, old_pub);
        // Encryption key changed
        assert_ne!(*keys.encryption_key(), old_enc);
        // Generation incremented
        assert_eq!(keys.generation(), 1);
    }
}
