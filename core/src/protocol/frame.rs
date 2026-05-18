//! Wire frame format for Quicko2.
//!
//! Frame layout:
//! ```text
//! ┌─────────┬──────────┬─────────┬──────────────────────┐
//! │ Version │  Type    │ Length  │       Payload        │
//! │ 1 byte  │ 1 byte  │ 4 bytes │  Variable (msgpack)  │
//! └─────────┴──────────┴─────────┴──────────────────────┘
//! ```

use serde::{Deserialize, Serialize};

use crate::error::{QuickoError, Result};

/// Current protocol version.
pub const PROTOCOL_VERSION: u8 = 1;

/// Frame header size: version(1) + type(1) + length(4)
pub const HEADER_SIZE: usize = 6;

/// Maximum payload size (1 MB).
pub const MAX_PAYLOAD_SIZE: usize = 1_048_576;

/// Message types in the Quicko2 protocol.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum MessageType {
    /// Initial handshake (client → server).
    Hello = 0x01,
    /// Handshake acknowledgment (server → client).
    HelloAck = 0x02,
    /// Encrypted chat message (client ↔ client via server).
    Message = 0x10,
    /// Delivery acknowledgment.
    Ack = 0x11,
    /// Heartbeat ping.
    Ping = 0x20,
    /// Heartbeat pong.
    Pong = 0x21,
    /// Join session by invite code.
    Join = 0x30,
    /// Leave session.
    Leave = 0x31,
    /// Notification: peer joined.
    PeerJoined = 0x32,
    /// Notification: peer left.
    PeerLeft = 0x33,
    /// X25519 key exchange.
    KeyExchange = 0x40,
    /// Register QuickoKey on server directory.
    RegisterKey = 0x50,
    /// Look up a QuickoKey in the directory.
    LookupKey = 0x51,
    /// Server response to a key lookup.
    LookupResponse = 0x52,
    /// Remove QuickoKey from directory.
    UnregisterKey = 0x53,
    /// Initiate a "call" (connection request) to a QuickoKey.
    CallPeer = 0x54,
    /// Accept/reject a call.
    CallResponse = 0x55,
    /// Error from server.
    Error = 0xFF,
}

impl MessageType {
    /// Parse a message type from a byte.
    pub fn from_byte(b: u8) -> Result<Self> {
        match b {
            0x01 => Ok(Self::Hello),
            0x02 => Ok(Self::HelloAck),
            0x10 => Ok(Self::Message),
            0x11 => Ok(Self::Ack),
            0x20 => Ok(Self::Ping),
            0x21 => Ok(Self::Pong),
            0x30 => Ok(Self::Join),
            0x31 => Ok(Self::Leave),
            0x32 => Ok(Self::PeerJoined),
            0x33 => Ok(Self::PeerLeft),
            0x40 => Ok(Self::KeyExchange),
            0x50 => Ok(Self::RegisterKey),
            0x51 => Ok(Self::LookupKey),
            0x52 => Ok(Self::LookupResponse),
            0x53 => Ok(Self::UnregisterKey),
            0x54 => Ok(Self::CallPeer),
            0x55 => Ok(Self::CallResponse),
            0xFF => Ok(Self::Error),
            _ => Err(QuickoError::UnknownMessageType(b)),
        }
    }

    /// Convert to byte.
    pub fn as_byte(&self) -> u8 {
        *self as u8
    }
}

/// A protocol frame consisting of metadata and a payload.
#[derive(Debug, Clone)]
pub struct Frame {
    /// Protocol version.
    pub version: u8,
    /// Message type.
    pub msg_type: MessageType,
    /// Payload data (serialized with MessagePack).
    pub payload: Vec<u8>,
}

impl Frame {
    /// Create a new frame with the current protocol version.
    pub fn new(msg_type: MessageType, payload: Vec<u8>) -> Result<Self> {
        if payload.len() > MAX_PAYLOAD_SIZE {
            return Err(QuickoError::InvalidFrame(format!(
                "Payload too large: {} bytes (max {})",
                payload.len(),
                MAX_PAYLOAD_SIZE
            )));
        }
        Ok(Self {
            version: PROTOCOL_VERSION,
            msg_type,
            payload,
        })
    }

    /// Create a ping frame (empty payload).
    pub fn ping() -> Self {
        Self {
            version: PROTOCOL_VERSION,
            msg_type: MessageType::Ping,
            payload: Vec::new(),
        }
    }

    /// Create a pong frame (empty payload).
    pub fn pong() -> Self {
        Self {
            version: PROTOCOL_VERSION,
            msg_type: MessageType::Pong,
            payload: Vec::new(),
        }
    }
}

/// Payload structures for each message type.
pub mod payloads {
    use serde::{Deserialize, Serialize};

    /// Hello handshake payload.
    #[derive(Debug, Serialize, Deserialize)]
    pub struct HelloPayload {
        pub session_id: String,
        pub display_name: String,
        pub public_key: Vec<u8>,
    }

    /// HelloAck payload from server.
    #[derive(Debug, Serialize, Deserialize)]
    pub struct HelloAckPayload {
        pub accepted: bool,
        pub server_time: i64,
    }

    /// Encrypted message payload.
    #[derive(Debug, Serialize, Deserialize)]
    pub struct MessagePayload {
        pub message_id: String,
        pub sender_id: String,
        pub recipient_id: String,
        pub timestamp: i64,
        pub encrypted_content: Vec<u8>,
        pub key_generation: u32,
    }

    /// Delivery acknowledgment payload.
    #[derive(Debug, Serialize, Deserialize)]
    pub struct AckPayload {
        pub message_id: String,
        pub status: String, // "delivered" | "read"
    }

    /// Join session payload.
    #[derive(Debug, Serialize, Deserialize)]
    pub struct JoinPayload {
        pub invite_code: String,
        pub session_id: String,
        pub display_name: String,
        pub public_key: Vec<u8>,
    }

    /// Peer joined/left notification payload.
    #[derive(Debug, Serialize, Deserialize)]
    pub struct PeerEventPayload {
        pub session_id: String,
        pub display_name: String,
        pub public_key: Option<Vec<u8>>,
    }

    /// Key exchange payload.
    #[derive(Debug, Serialize, Deserialize)]
    pub struct KeyExchangePayload {
        pub sender_id: String,
        pub public_key: Vec<u8>,
        pub generation: u32,
    }

    /// Error payload.
    #[derive(Debug, Serialize, Deserialize)]
    pub struct ErrorPayload {
        pub code: u32,
        pub message: String,
    }

    // --- QuickoKey directory payloads ---

    /// Register a QuickoKey on the server directory.
    #[derive(Debug, Serialize, Deserialize)]
    pub struct RegisterKeyPayload {
        pub quicko_key: String,
        pub public_key: Vec<u8>,
        pub display_name: String,
    }

    /// Register acknowledgment.
    #[derive(Debug, Serialize, Deserialize)]
    pub struct RegisterKeyAckPayload {
        pub accepted: bool,
        pub error: Option<String>,
    }

    /// Look up a QuickoKey.
    #[derive(Debug, Serialize, Deserialize)]
    pub struct LookupKeyPayload {
        pub quicko_key: String,
    }

    /// Lookup response from server.
    #[derive(Debug, Serialize, Deserialize)]
    pub struct LookupResponsePayload {
        pub quicko_key: String,
        pub found: bool,
        pub public_key: Option<Vec<u8>>,
        pub display_name: Option<String>,
        pub is_online: bool,
    }

    /// Unregister a QuickoKey.
    #[derive(Debug, Serialize, Deserialize)]
    pub struct UnregisterKeyPayload {
        pub quicko_key: String,
    }

    /// Initiate a call to another QuickoKey.
    #[derive(Debug, Serialize, Deserialize)]
    pub struct CallPeerPayload {
        pub caller_key: String,
        pub callee_key: String,
        pub caller_public_key: Vec<u8>,
        pub caller_display_name: String,
    }

    /// Response to a call request.
    #[derive(Debug, Serialize, Deserialize)]
    pub struct CallResponsePayload {
        pub caller_key: String,
        pub callee_key: String,
        pub accepted: bool,
        pub responder_public_key: Option<Vec<u8>>,
        pub responder_display_name: Option<String>,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_type_roundtrip() {
        let types = [
            MessageType::Hello,
            MessageType::HelloAck,
            MessageType::Message,
            MessageType::Ack,
            MessageType::Ping,
            MessageType::Pong,
            MessageType::Join,
            MessageType::Leave,
            MessageType::PeerJoined,
            MessageType::PeerLeft,
            MessageType::KeyExchange,
            MessageType::RegisterKey,
            MessageType::LookupKey,
            MessageType::LookupResponse,
            MessageType::UnregisterKey,
            MessageType::CallPeer,
            MessageType::CallResponse,
            MessageType::Error,
        ];

        for mt in types {
            let byte = mt.as_byte();
            let parsed = MessageType::from_byte(byte).unwrap();
            assert_eq!(parsed, mt);
        }
    }

    #[test]
    fn test_unknown_message_type() {
        let result = MessageType::from_byte(0x99);
        assert!(result.is_err());
    }

    #[test]
    fn test_frame_creation() {
        let payload = vec![1, 2, 3, 4];
        let frame = Frame::new(MessageType::Message, payload.clone()).unwrap();

        assert_eq!(frame.version, PROTOCOL_VERSION);
        assert_eq!(frame.msg_type, MessageType::Message);
        assert_eq!(frame.payload, payload);
    }

    #[test]
    fn test_payload_too_large() {
        let payload = vec![0u8; MAX_PAYLOAD_SIZE + 1];
        let result = Frame::new(MessageType::Message, payload);
        assert!(result.is_err());
    }
}
