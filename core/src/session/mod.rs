//! Ephemeral session management.

pub mod identity;
pub mod lifecycle;
pub mod peer;

pub use identity::SessionIdentity;
pub use lifecycle::SessionState;
pub use peer::PeerInfo;
