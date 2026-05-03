//! Message types and creation.

use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A chat message in the Quicko2 system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// Unique message ID (UUID v4).
    pub id: String,
    /// Sender's session ID.
    pub sender_id: String,
    /// Recipient's session ID.
    pub recipient_id: String,
    /// Message content (plaintext, before encryption).
    pub content: String,
    /// UTC timestamp (milliseconds since epoch).
    pub timestamp: i64,
    /// Delivery status.
    pub status: super::DeliveryStatus,
}

impl Message {
    /// Create a new outbound message.
    pub fn new(sender_id: String, recipient_id: String, content: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            sender_id,
            recipient_id,
            content,
            timestamp: Utc::now().timestamp_millis(),
            status: super::DeliveryStatus::Pending,
        }
    }

    /// Create from received data (already has ID and timestamp).
    pub fn from_received(
        id: String,
        sender_id: String,
        recipient_id: String,
        content: String,
        timestamp: i64,
    ) -> Self {
        Self {
            id,
            sender_id,
            recipient_id,
            content,
            timestamp,
            status: super::DeliveryStatus::Delivered,
        }
    }

    /// Get the message age in milliseconds.
    pub fn age_ms(&self) -> i64 {
        Utc::now().timestamp_millis() - self.timestamp
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_creation() {
        let msg = Message::new(
            "alice".to_string(),
            "bob".to_string(),
            "Hello!".to_string(),
        );

        assert!(!msg.id.is_empty());
        assert_eq!(msg.sender_id, "alice");
        assert_eq!(msg.recipient_id, "bob");
        assert_eq!(msg.content, "Hello!");
        assert!(msg.timestamp > 0);
        assert_eq!(msg.status, super::super::DeliveryStatus::Pending);
    }

    #[test]
    fn test_unique_message_ids() {
        let msg1 = Message::new("a".into(), "b".into(), "1".into());
        let msg2 = Message::new("a".into(), "b".into(), "2".into());
        assert_ne!(msg1.id, msg2.id);
    }
}
