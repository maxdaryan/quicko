//! Message delivery status tracking.

use serde::{Deserialize, Serialize};

/// Delivery status for a message.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeliveryStatus {
    /// Message created, not yet sent.
    Pending,
    /// Message sent to relay server.
    Sent,
    /// Message delivered to recipient.
    Delivered,
    /// Message read by recipient.
    Read,
    /// Message delivery failed.
    Failed,
}

impl DeliveryStatus {
    /// Check if the message has been successfully delivered.
    pub fn is_delivered(&self) -> bool {
        matches!(self, Self::Delivered | Self::Read)
    }

    /// Check if the message is still in-flight.
    pub fn is_pending(&self) -> bool {
        matches!(self, Self::Pending | Self::Sent)
    }
}

impl std::fmt::Display for DeliveryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pending => write!(f, "pending"),
            Self::Sent => write!(f, "sent"),
            Self::Delivered => write!(f, "delivered"),
            Self::Read => write!(f, "read"),
            Self::Failed => write!(f, "failed"),
        }
    }
}
