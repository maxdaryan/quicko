//! Frame codec — encode/decode frames to/from bytes.

use crate::error::{QuickoError, Result};
use crate::protocol::frame::{Frame, MessageType, HEADER_SIZE, MAX_PAYLOAD_SIZE, PROTOCOL_VERSION};

/// Codec for encoding and decoding protocol frames.
pub struct FrameCodec;

impl FrameCodec {
    /// Encode a frame into bytes.
    ///
    /// Layout: version(1) + type(1) + length(4, big-endian) + payload(N)
    pub fn encode(frame: &Frame) -> Vec<u8> {
        let len = frame.payload.len() as u32;
        let mut buf = Vec::with_capacity(HEADER_SIZE + frame.payload.len());

        buf.push(frame.version);
        buf.push(frame.msg_type.as_byte());
        buf.extend_from_slice(&len.to_be_bytes());
        buf.extend_from_slice(&frame.payload);

        buf
    }

    /// Decode a frame from bytes.
    ///
    /// Returns the frame and the number of bytes consumed.
    pub fn decode(data: &[u8]) -> Result<(Frame, usize)> {
        if data.len() < HEADER_SIZE {
            return Err(QuickoError::InvalidFrame(
                "Not enough data for header".to_string(),
            ));
        }

        let version = data[0];
        if version != PROTOCOL_VERSION {
            return Err(QuickoError::VersionMismatch {
                expected: PROTOCOL_VERSION,
                got: version,
            });
        }

        let msg_type = MessageType::from_byte(data[1])?;

        let payload_len =
            u32::from_be_bytes([data[2], data[3], data[4], data[5]]) as usize;

        if payload_len > MAX_PAYLOAD_SIZE {
            return Err(QuickoError::InvalidFrame(format!(
                "Payload length {} exceeds maximum {}",
                payload_len, MAX_PAYLOAD_SIZE
            )));
        }

        let total_len = HEADER_SIZE + payload_len;
        if data.len() < total_len {
            return Err(QuickoError::InvalidFrame(format!(
                "Expected {} bytes, got {}",
                total_len,
                data.len()
            )));
        }

        let payload = data[HEADER_SIZE..total_len].to_vec();

        let frame = Frame {
            version,
            msg_type,
            payload,
        };

        Ok((frame, total_len))
    }

    /// Serialize a payload struct to MessagePack bytes.
    pub fn serialize_payload<T: serde::Serialize>(payload: &T) -> Result<Vec<u8>> {
        rmp_serde::to_vec(payload).map_err(|e| QuickoError::SerializationError(e.to_string()))
    }

    /// Deserialize a payload from MessagePack bytes.
    pub fn deserialize_payload<'a, T: serde::Deserialize<'a>>(data: &'a [u8]) -> Result<T> {
        rmp_serde::from_slice(data)
            .map_err(|e| QuickoError::DeserializationError(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::frame::payloads::HelloPayload;

    #[test]
    fn test_encode_decode_roundtrip() {
        let original = Frame::new(MessageType::Message, vec![1, 2, 3, 4, 5]).unwrap();
        let encoded = FrameCodec::encode(&original);
        let (decoded, consumed) = FrameCodec::decode(&encoded).unwrap();

        assert_eq!(consumed, encoded.len());
        assert_eq!(decoded.version, original.version);
        assert_eq!(decoded.msg_type, original.msg_type);
        assert_eq!(decoded.payload, original.payload);
    }

    #[test]
    fn test_encode_decode_empty_payload() {
        let frame = Frame::ping();
        let encoded = FrameCodec::encode(&frame);
        let (decoded, consumed) = FrameCodec::decode(&encoded).unwrap();

        assert_eq!(consumed, HEADER_SIZE);
        assert_eq!(decoded.msg_type, MessageType::Ping);
        assert!(decoded.payload.is_empty());
    }

    #[test]
    fn test_too_short_data() {
        let result = FrameCodec::decode(&[0x01, 0x10]);
        assert!(result.is_err());
    }

    #[test]
    fn test_wrong_version() {
        let mut encoded = FrameCodec::encode(&Frame::ping());
        encoded[0] = 99; // Wrong version
        let result = FrameCodec::decode(&encoded);
        assert!(matches!(
            result,
            Err(QuickoError::VersionMismatch { expected: 1, got: 99 })
        ));
    }

    #[test]
    fn test_payload_serialization() {
        let payload = HelloPayload {
            session_id: "abc123".to_string(),
            display_name: "Swift Falcon #1234".to_string(),
            public_key: vec![1u8; 32],
        };

        let serialized = FrameCodec::serialize_payload(&payload).unwrap();
        let deserialized: HelloPayload =
            FrameCodec::deserialize_payload(&serialized).unwrap();

        assert_eq!(deserialized.session_id, "abc123");
        assert_eq!(deserialized.display_name, "Swift Falcon #1234");
        assert_eq!(deserialized.public_key.len(), 32);
    }
}
