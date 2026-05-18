//! Client-side directory operation types.
//!
//! These are the request/response payloads sent over the WebSocket
//! connection for key directory operations (register, lookup, call).
//! The server stores only the mapping QuickoKey → public key + presence,
//! never any message content.

use serde::{Deserialize, Serialize};

/// Register a QuickoKey on the server directory.
#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterKeyPayload {
    /// Formatted QuickoKey (e.g., "QK-1A2B-...")
    pub quicko_key: String,
    /// X25519 public key (32 bytes)
    pub public_key: Vec<u8>,
    /// Human-readable display name
    pub display_name: String,
}

/// Server acknowledgment of key registration.
#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterKeyAckPayload {
    /// Whether the registration was accepted.
    pub accepted: bool,
    /// Error message if rejected.
    pub error: Option<String>,
}

/// Look up a QuickoKey in the server directory.
#[derive(Debug, Serialize, Deserialize)]
pub struct LookupKeyPayload {
    /// The QuickoKey to look up.
    pub quicko_key: String,
}

/// Server response to a key lookup.
#[derive(Debug, Serialize, Deserialize)]
pub struct LookupResponsePayload {
    /// The key that was looked up.
    pub quicko_key: String,
    /// Whether the key was found.
    pub found: bool,
    /// Holder's X25519 public key (if found).
    pub public_key: Option<Vec<u8>>,
    /// Holder's display name (if found).
    pub display_name: Option<String>,
    /// Whether the key holder is currently online.
    pub is_online: bool,
}

/// Remove a QuickoKey from the server directory.
#[derive(Debug, Serialize, Deserialize)]
pub struct UnregisterKeyPayload {
    /// The QuickoKey to remove.
    pub quicko_key: String,
}

/// Initiate a "call" (connection request) to another QuickoKey.
#[derive(Debug, Serialize, Deserialize)]
pub struct CallPeerPayload {
    /// Caller's formatted QuickoKey.
    pub caller_key: String,
    /// Callee's formatted QuickoKey.
    pub callee_key: String,
    /// Caller's X25519 public key (32 bytes).
    pub caller_public_key: Vec<u8>,
    /// Caller's display name.
    pub caller_display_name: String,
}

/// Response to a call request.
#[derive(Debug, Serialize, Deserialize)]
pub struct CallResponsePayload {
    /// Caller's QuickoKey.
    pub caller_key: String,
    /// Callee's QuickoKey.
    pub callee_key: String,
    /// Whether the call was accepted.
    pub accepted: bool,
    /// Responder's X25519 public key (if accepted).
    pub responder_public_key: Option<Vec<u8>>,
    /// Responder's display name.
    pub responder_display_name: Option<String>,
}
