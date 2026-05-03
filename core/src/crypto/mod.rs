//! Cryptographic primitives for Quicko2.
//!
//! Provides:
//! - X25519 Diffie-Hellman key exchange
//! - HKDF-SHA256 key derivation
//! - AES-256-GCM authenticated encryption with AAD

pub mod encrypt;
pub mod kdf;
pub mod keys;
pub mod session_keys;

pub use encrypt::{decrypt, encrypt};
pub use kdf::derive_key;
pub use keys::KeyPair;
pub use session_keys::SessionKeys;
