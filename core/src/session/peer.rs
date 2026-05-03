//! Peer tracking and presence.

use std::time::Instant;

/// Information about a connected peer.
#[derive(Debug, Clone)]
pub struct PeerInfo {
    /// Peer's session ID.
    pub session_id: String,
    /// Peer's display name.
    pub display_name: String,
    /// Peer's X25519 public key.
    pub public_key: [u8; 32],
    /// When the peer joined.
    pub joined_at: Instant,
    /// Whether the peer is currently online.
    pub is_online: bool,
    /// Last activity timestamp.
    pub last_seen: Instant,
}

impl PeerInfo {
    /// Create a new peer info entry.
    pub fn new(session_id: String, display_name: String, public_key: [u8; 32]) -> Self {
        let now = Instant::now();
        Self {
            session_id,
            display_name,
            public_key,
            joined_at: now,
            is_online: true,
            last_seen: now,
        }
    }

    /// Mark this peer as seen (update last_seen).
    pub fn touch(&mut self) {
        self.last_seen = Instant::now();
        self.is_online = true;
    }

    /// Mark this peer as offline.
    pub fn mark_offline(&mut self) {
        self.is_online = false;
    }

    /// Get how long since last activity.
    pub fn idle_duration(&self) -> std::time::Duration {
        self.last_seen.elapsed()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_peer_creation() {
        let peer = PeerInfo::new(
            "abc123".to_string(),
            "Swift Falcon #1234".to_string(),
            [1u8; 32],
        );
        assert!(peer.is_online);
        assert_eq!(peer.session_id, "abc123");
    }

    #[test]
    fn test_peer_offline() {
        let mut peer = PeerInfo::new(
            "abc123".to_string(),
            "Swift Falcon #1234".to_string(),
            [1u8; 32],
        );
        peer.mark_offline();
        assert!(!peer.is_online);
    }
}
