//! Server-side key directory — an in-memory phone book.
//!
//! Maps QuickoKey → public key + display name + online status.
//! This allows users to look up each other by QuickoKey (like a phone number)
//! and initiate calls. The server never stores messages — only key mappings.

use dashmap::DashMap;
use std::time::Instant;

/// A directory entry for a registered QuickoKey.
#[derive(Debug, Clone)]
pub struct DirectoryEntry {
    /// Formatted QuickoKey (e.g., "QK-1A2B-...")
    pub quicko_key: String,
    /// X25519 public key (32 bytes)
    pub public_key: Vec<u8>,
    /// Human-readable display name
    pub display_name: String,
    /// Current WebSocket session ID (if online)
    pub session_id: Option<String>,
    /// When this key was registered
    pub registered_at: Instant,
    /// Last time this key holder was seen
    pub last_seen: Instant,
    /// Whether the key holder is currently connected
    pub is_online: bool,
}

/// In-memory key directory using DashMap for lock-free concurrent access.
///
/// Acts as a "phone book" — stores QuickoKey → connection info mappings.
/// Cleared on server restart (ephemeral by design).
pub struct KeyDirectory {
    entries: DashMap<String, DirectoryEntry>,
}

impl KeyDirectory {
    /// Create a new empty directory.
    pub fn new() -> Self {
        Self {
            entries: DashMap::new(),
        }
    }

    /// Register or update a QuickoKey in the directory.
    pub fn register(
        &self,
        quicko_key: String,
        public_key: Vec<u8>,
        display_name: String,
        session_id: String,
    ) {
        let now = Instant::now();
        self.entries.insert(
            quicko_key.clone(),
            DirectoryEntry {
                quicko_key,
                public_key,
                display_name,
                session_id: Some(session_id),
                registered_at: now,
                last_seen: now,
                is_online: true,
            },
        );
        tracing::info!("Key directory: registered, total: {}", self.entries.len());
    }

    /// Look up a QuickoKey in the directory.
    pub fn lookup(&self, quicko_key: &str) -> Option<DirectoryEntry> {
        self.entries.get(quicko_key).map(|e| e.clone())
    }

    /// Mark a key holder as online with a specific session.
    pub fn set_online(&self, quicko_key: &str, session_id: String) {
        if let Some(mut entry) = self.entries.get_mut(quicko_key) {
            entry.session_id = Some(session_id);
            entry.is_online = true;
            entry.last_seen = Instant::now();
        }
    }

    /// Mark a key holder as offline.
    pub fn set_offline(&self, quicko_key: &str) {
        if let Some(mut entry) = self.entries.get_mut(quicko_key) {
            entry.session_id = None;
            entry.is_online = false;
        }
    }

    /// Set offline by session ID (used when a WebSocket disconnects).
    pub fn set_offline_by_session(&self, session_id: &str) {
        for mut entry in self.entries.iter_mut() {
            if entry.session_id.as_deref() == Some(session_id) {
                entry.session_id = None;
                entry.is_online = false;
            }
        }
    }

    /// Remove a QuickoKey from the directory.
    pub fn unregister(&self, quicko_key: &str) {
        self.entries.remove(quicko_key);
        tracing::info!("Key directory: unregistered, total: {}", self.entries.len());
    }

    /// Get the session ID for a QuickoKey (if online).
    pub fn get_session_id(&self, quicko_key: &str) -> Option<String> {
        self.entries
            .get(quicko_key)
            .and_then(|e| e.session_id.clone())
    }

    /// Get the total number of registered keys.
    pub fn count(&self) -> usize {
        self.entries.len()
    }
}

impl Default for KeyDirectory {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_and_lookup() {
        let dir = KeyDirectory::new();
        dir.register(
            "QK-1A2B-3C4D-5E6F-7890-ABCD-EF12-3456-7890".to_string(),
            vec![1u8; 32],
            "Swift Falcon".to_string(),
            "session-123".to_string(),
        );

        let entry = dir.lookup("QK-1A2B-3C4D-5E6F-7890-ABCD-EF12-3456-7890");
        assert!(entry.is_some());
        let entry = entry.unwrap();
        assert!(entry.is_online);
        assert_eq!(entry.display_name, "Swift Falcon");
    }

    #[test]
    fn test_lookup_missing() {
        let dir = KeyDirectory::new();
        assert!(dir.lookup("QK-FFFF-FFFF-FFFF-FFFF-FFFF-FFFF-FFFF-FFFF").is_none());
    }

    #[test]
    fn test_online_offline() {
        let dir = KeyDirectory::new();
        let key = "QK-TEST-TEST-TEST-TEST-TEST-TEST-TEST-TEST".to_string();
        dir.register(key.clone(), vec![1u8; 32], "Test".to_string(), "s1".to_string());

        assert!(dir.lookup(&key).unwrap().is_online);

        dir.set_offline(&key);
        assert!(!dir.lookup(&key).unwrap().is_online);

        dir.set_online(&key, "s2".to_string());
        assert!(dir.lookup(&key).unwrap().is_online);
    }

    #[test]
    fn test_unregister() {
        let dir = KeyDirectory::new();
        let key = "QK-DEAD-BEEF-0000-0000-0000-0000-0000-0000".to_string();
        dir.register(key.clone(), vec![1u8; 32], "Test".to_string(), "s1".to_string());
        assert_eq!(dir.count(), 1);

        dir.unregister(&key);
        assert_eq!(dir.count(), 0);
        assert!(dir.lookup(&key).is_none());
    }

    #[test]
    fn test_set_offline_by_session() {
        let dir = KeyDirectory::new();
        dir.register("QK-A".to_string(), vec![1u8; 32], "A".to_string(), "s1".to_string());
        dir.register("QK-B".to_string(), vec![2u8; 32], "B".to_string(), "s2".to_string());

        dir.set_offline_by_session("s1");

        assert!(!dir.lookup("QK-A").unwrap().is_online);
        assert!(dir.lookup("QK-B").unwrap().is_online);
    }
}
