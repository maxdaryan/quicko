//! X25519 key pair generation and Diffie-Hellman shared secret computation.

use rand::rngs::OsRng;
use x25519_dalek::{PublicKey, StaticSecret};

/// An X25519 key pair for ephemeral key exchange.
///
/// Each session generates a fresh keypair. Private keys never leave
/// the process and are zeroized when the session is destroyed.
pub struct KeyPair {
    secret: StaticSecret,
    public: PublicKey,
}

impl KeyPair {
    /// Generate a new random X25519 key pair using OS randomness.
    pub fn generate() -> Self {
        let secret = StaticSecret::random_from_rng(OsRng);
        let public = PublicKey::from(&secret);
        Self { secret, public }
    }

    /// Get the public key bytes (32 bytes).
    pub fn public_key_bytes(&self) -> [u8; 32] {
        *self.public.as_bytes()
    }

    /// Get a reference to the public key.
    pub fn public_key(&self) -> &PublicKey {
        &self.public
    }

    /// Perform X25519 Diffie-Hellman to compute a shared secret.
    ///
    /// Returns the raw 32-byte shared secret. This MUST be passed through
    /// HKDF before use as an encryption key — never use raw DH output directly.
    pub fn diffie_hellman(&self, peer_public: &[u8; 32]) -> [u8; 32] {
        let peer_key = PublicKey::from(*peer_public);
        let shared = self.secret.diffie_hellman(&peer_key);
        *shared.as_bytes()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keypair_generation() {
        let kp = KeyPair::generate();
        let pub_bytes = kp.public_key_bytes();
        // Public key should not be all zeros
        assert_ne!(pub_bytes, [0u8; 32]);
    }

    #[test]
    fn test_diffie_hellman_shared_secret() {
        let alice = KeyPair::generate();
        let bob = KeyPair::generate();

        let secret_a = alice.diffie_hellman(&bob.public_key_bytes());
        let secret_b = bob.diffie_hellman(&alice.public_key_bytes());

        // Both sides derive the same shared secret
        assert_eq!(secret_a, secret_b);
    }

    #[test]
    fn test_different_keypairs_different_secrets() {
        let alice = KeyPair::generate();
        let bob = KeyPair::generate();
        let charlie = KeyPair::generate();

        let secret_ab = alice.diffie_hellman(&bob.public_key_bytes());
        let secret_ac = alice.diffie_hellman(&charlie.public_key_bytes());

        // Different peers produce different shared secrets
        assert_ne!(secret_ab, secret_ac);
    }

    #[test]
    fn test_unique_keypairs() {
        let kp1 = KeyPair::generate();
        let kp2 = KeyPair::generate();
        assert_ne!(kp1.public_key_bytes(), kp2.public_key_bytes());
    }
}
