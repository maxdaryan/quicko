//! Unified error types for Quicko2 core.

use thiserror::Error;

/// Result type alias using QuickoError.
pub type Result<T> = std::result::Result<T, QuickoError>;

/// All possible errors in the Quicko2 core.
#[derive(Debug, Error)]
pub enum QuickoError {
    // -- Crypto errors --
    #[error("Key exchange failed")]
    KeyExchangeFailed,

    #[error("Encryption failed")]
    EncryptionFailed,

    #[error("Decryption failed: message may be tampered or replayed")]
    DecryptionFailed,

    #[error("Key derivation failed")]
    KeyDerivationFailed,

    // -- Session errors --
    #[error("No active session")]
    NoActiveSession,

    #[error("Session expired")]
    SessionExpired,

    #[error("Invalid invite code")]
    InvalidInviteCode,

    #[error("Invalid QuickoKey: {0}")]
    InvalidQuickoKey(String),

    // -- Network errors --
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    #[error("Connection lost")]
    ConnectionLost,

    #[error("Send failed: {0}")]
    SendFailed(String),

    #[error("Handshake failed")]
    HandshakeFailed,

    // -- Protocol errors --
    #[error("Invalid frame: {0}")]
    InvalidFrame(String),

    #[error("Protocol version mismatch: expected {expected}, got {got}")]
    VersionMismatch { expected: u8, got: u8 },

    #[error("Unknown message type: 0x{0:02x}")]
    UnknownMessageType(u8),

    // -- Store errors --
    #[error("Message not found")]
    MessageNotFound,

    #[error("Store capacity exceeded")]
    StoreCapacityExceeded,

    // -- Serialization errors --
    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Deserialization error: {0}")]
    DeserializationError(String),

    // -- General --
    #[error("Internal error: {0}")]
    Internal(String),
}

impl From<rmp_serde::encode::Error> for QuickoError {
    fn from(e: rmp_serde::encode::Error) -> Self {
        QuickoError::SerializationError(e.to_string())
    }
}

impl From<rmp_serde::decode::Error> for QuickoError {
    fn from(e: rmp_serde::decode::Error) -> Self {
        QuickoError::DeserializationError(e.to_string())
    }
}
