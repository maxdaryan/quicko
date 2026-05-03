//! Transport events for the networking layer.

/// Events emitted by the transport layer.
#[derive(Debug, Clone)]
pub enum TransportEvent {
    /// Successfully connected to the relay server.
    Connected,
    /// Disconnected from the relay server.
    Disconnected(String),
    /// Received a binary message from the server.
    MessageReceived(Vec<u8>),
    /// Received a ping from the server.
    PingReceived(Vec<u8>),
    /// Reconnecting (attempt number).
    Reconnecting(u32),
    /// Reconnection failed after max attempts.
    ReconnectionFailed,
}
