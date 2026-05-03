//! WebSocket networking with auto-reconnect.

pub mod connection;
pub mod heartbeat;
pub mod transport;

pub use connection::ConnectionManager;
pub use transport::TransportEvent;
