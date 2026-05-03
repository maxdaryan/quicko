//! AES-256-GCM authenticated encryption with Associated Authenticated Data (AAD).
//!
//! All messages are encrypted with AES-256-GCM using:
//! - A 256-bit key derived via HKDF (never raw DH output)
//! - A random 96-bit nonce per message
//! - AAD binding sender_id, recipient_id, and timestamp to prevent replay attacks

use aes_gcm::{
    aead::{Aead, KeyInit, Payload},
    Aes256Gcm, Nonce,
};
use rand::{rngs::OsRng, RngCore};

use crate::error::{QuickoError, Result};

/// Nonce size in bytes (96 bits for AES-GCM).
const NONCE_SIZE: usize = 12;

/// Authentication tag size in bytes (128 bits for AES-GCM).
const TAG_SIZE: usize = 16;

/// Encrypt plaintext with AES-256-GCM.
///
/// # Arguments
/// * `key` — 256-bit encryption key (from HKDF)
/// * `plaintext` — Data to encrypt
/// * `aad` — Associated Authenticated Data (e.g., sender_id || recipient_id || timestamp)
///
/// # Returns
/// `nonce (12 bytes) || ciphertext || tag (16 bytes)`
pub fn encrypt(key: &[u8; 32], plaintext: &[u8], aad: &[u8]) -> Result<Vec<u8>> {
    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|_| QuickoError::EncryptionFailed)?;

    // Generate random nonce
    let mut nonce_bytes = [0u8; NONCE_SIZE];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    // Encrypt with AAD
    let payload = Payload {
        msg: plaintext,
        aad,
    };
    let ciphertext = cipher
        .encrypt(nonce, payload)
        .map_err(|_| QuickoError::EncryptionFailed)?;

    // Output: nonce || ciphertext (includes tag)
    let mut output = Vec::with_capacity(NONCE_SIZE + ciphertext.len());
    output.extend_from_slice(&nonce_bytes);
    output.extend_from_slice(&ciphertext);
    Ok(output)
}

/// Decrypt ciphertext with AES-256-GCM.
///
/// # Arguments
/// * `key` — 256-bit encryption key (from HKDF)
/// * `encrypted` — Output of `encrypt()`: `nonce || ciphertext || tag`
/// * `aad` — Must match the AAD used during encryption
///
/// # Returns
/// The original plaintext, or `DecryptionFailed` if tampered/replayed.
pub fn decrypt(key: &[u8; 32], encrypted: &[u8], aad: &[u8]) -> Result<Vec<u8>> {
    if encrypted.len() < NONCE_SIZE + TAG_SIZE {
        return Err(QuickoError::DecryptionFailed);
    }

    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|_| QuickoError::DecryptionFailed)?;

    let nonce = Nonce::from_slice(&encrypted[..NONCE_SIZE]);
    let ciphertext = &encrypted[NONCE_SIZE..];

    let payload = Payload {
        msg: ciphertext,
        aad,
    };
    cipher
        .decrypt(nonce, payload)
        .map_err(|_| QuickoError::DecryptionFailed)
}

/// Build AAD bytes from message metadata.
///
/// AAD = sender_id || ":" || recipient_id || ":" || timestamp
/// This binds the ciphertext to its context, preventing replay attacks.
pub fn build_aad(sender_id: &str, recipient_id: &str, timestamp: i64) -> Vec<u8> {
    format!("{}:{}:{}", sender_id, recipient_id, timestamp).into_bytes()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let key = [42u8; 32];
        let plaintext = b"Hello, Quicko2!";
        let aad = build_aad("alice", "bob", 1234567890);

        let encrypted = encrypt(&key, plaintext, &aad).unwrap();
        let decrypted = decrypt(&key, &encrypted, &aad).unwrap();

        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_wrong_key_fails() {
        let key = [42u8; 32];
        let wrong_key = [43u8; 32];
        let plaintext = b"Secret message";
        let aad = build_aad("alice", "bob", 1234567890);

        let encrypted = encrypt(&key, plaintext, &aad).unwrap();
        let result = decrypt(&wrong_key, &encrypted, &aad);

        assert!(result.is_err());
    }

    #[test]
    fn test_wrong_aad_fails() {
        let key = [42u8; 32];
        let plaintext = b"Secret message";
        let aad = build_aad("alice", "bob", 1234567890);
        let wrong_aad = build_aad("alice", "charlie", 1234567890);

        let encrypted = encrypt(&key, plaintext, &aad).unwrap();
        let result = decrypt(&key, &encrypted, &wrong_aad);

        assert!(result.is_err());
    }

    #[test]
    fn test_tampered_ciphertext_fails() {
        let key = [42u8; 32];
        let plaintext = b"Secret message";
        let aad = build_aad("alice", "bob", 1234567890);

        let mut encrypted = encrypt(&key, plaintext, &aad).unwrap();
        // Flip a bit in the ciphertext
        if let Some(byte) = encrypted.get_mut(15) {
            *byte ^= 0x01;
        }
        let result = decrypt(&key, &encrypted, &aad);

        assert!(result.is_err());
    }

    #[test]
    fn test_unique_nonces() {
        let key = [42u8; 32];
        let plaintext = b"Same message";
        let aad = build_aad("alice", "bob", 1234567890);

        let enc1 = encrypt(&key, plaintext, &aad).unwrap();
        let enc2 = encrypt(&key, plaintext, &aad).unwrap();

        // Same plaintext, same key, but different nonces → different ciphertext
        assert_ne!(enc1, enc2);
    }

    #[test]
    fn test_empty_plaintext() {
        let key = [42u8; 32];
        let plaintext = b"";
        let aad = build_aad("alice", "bob", 1234567890);

        let encrypted = encrypt(&key, plaintext, &aad).unwrap();
        let decrypted = decrypt(&key, &encrypted, &aad).unwrap();

        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_short_ciphertext_rejected() {
        let key = [42u8; 32];
        let too_short = vec![0u8; 10]; // Less than NONCE_SIZE + TAG_SIZE
        let aad = build_aad("alice", "bob", 1234567890);

        let result = decrypt(&key, &too_short, &aad);
        assert!(result.is_err());
    }
}
