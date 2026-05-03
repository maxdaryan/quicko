//! HKDF-SHA256 key derivation.
//!
//! Derives encryption keys from raw Diffie-Hellman shared secrets.
//! This is critical — raw DH output must NEVER be used directly as a key.

use hkdf::Hkdf;
use sha2::Sha256;

use crate::error::{QuickoError, Result};

/// Domain separation context for message encryption keys.
pub const CONTEXT_MESSAGE_KEY: &[u8] = b"quicko2-msg-v1";

/// Domain separation context for session authentication keys.
pub const CONTEXT_AUTH_KEY: &[u8] = b"quicko2-auth-v1";

/// Derive a 256-bit key from a shared secret using HKDF-SHA256.
///
/// # Arguments
/// * `shared_secret` — Raw X25519 DH output (32 bytes)
/// * `salt` — Optional salt for domain separation (can be session-specific)
/// * `info` — Context string for domain separation (e.g., `CONTEXT_MESSAGE_KEY`)
///
/// # Returns
/// A 32-byte derived key suitable for use with AES-256-GCM.
pub fn derive_key(shared_secret: &[u8; 32], salt: Option<&[u8]>, info: &[u8]) -> Result<[u8; 32]> {
    let hk = Hkdf::<Sha256>::new(salt, shared_secret);
    let mut key = [0u8; 32];
    hk.expand(info, &mut key)
        .map_err(|_| QuickoError::KeyDerivationFailed)?;
    Ok(key)
}

/// Derive both a message encryption key and an authentication key from a single
/// shared secret. Uses different domain separation contexts for each.
pub fn derive_session_keys(
    shared_secret: &[u8; 32],
    salt: Option<&[u8]>,
) -> Result<([u8; 32], [u8; 32])> {
    let enc_key = derive_key(shared_secret, salt, CONTEXT_MESSAGE_KEY)?;
    let auth_key = derive_key(shared_secret, salt, CONTEXT_AUTH_KEY)?;
    Ok((enc_key, auth_key))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_derivation() {
        let shared_secret = [42u8; 32];
        let key = derive_key(&shared_secret, None, CONTEXT_MESSAGE_KEY).unwrap();

        // Key should be 32 bytes and not all zeros
        assert_eq!(key.len(), 32);
        assert_ne!(key, [0u8; 32]);
    }

    #[test]
    fn test_deterministic_derivation() {
        let shared_secret = [42u8; 32];
        let key1 = derive_key(&shared_secret, None, CONTEXT_MESSAGE_KEY).unwrap();
        let key2 = derive_key(&shared_secret, None, CONTEXT_MESSAGE_KEY).unwrap();

        // Same input produces same output
        assert_eq!(key1, key2);
    }

    #[test]
    fn test_different_contexts_different_keys() {
        let shared_secret = [42u8; 32];
        let msg_key = derive_key(&shared_secret, None, CONTEXT_MESSAGE_KEY).unwrap();
        let auth_key = derive_key(&shared_secret, None, CONTEXT_AUTH_KEY).unwrap();

        // Different contexts produce different keys
        assert_ne!(msg_key, auth_key);
    }

    #[test]
    fn test_different_salts_different_keys() {
        let shared_secret = [42u8; 32];
        let key1 = derive_key(&shared_secret, Some(b"session-1"), CONTEXT_MESSAGE_KEY).unwrap();
        let key2 = derive_key(&shared_secret, Some(b"session-2"), CONTEXT_MESSAGE_KEY).unwrap();

        assert_ne!(key1, key2);
    }

    #[test]
    fn test_derive_session_keys() {
        let shared_secret = [42u8; 32];
        let (enc_key, auth_key) = derive_session_keys(&shared_secret, None).unwrap();

        assert_ne!(enc_key, auth_key);
        assert_ne!(enc_key, [0u8; 32]);
        assert_ne!(auth_key, [0u8; 32]);
    }
}
