//! Ephemeral in-memory message store with TTL eviction.
//!
//! Uses a ring buffer to cap memory usage. Messages are automatically
//! evicted when they exceed the TTL or when the buffer is full.
//! All data is lost when the process exits — by design.

use std::collections::VecDeque;
use std::time::{Duration, Instant};

use crate::messaging::Message;

/// An entry in the ephemeral store with a creation timestamp.
struct StoreEntry {
    message: Message,
    stored_at: Instant,
}

/// In-memory message store with bounded capacity and TTL eviction.
///
/// This store is intentionally ephemeral — nothing is persisted to disk.
/// When the process exits, all messages are gone forever.
pub struct EphemeralStore {
    entries: VecDeque<StoreEntry>,
    max_capacity: usize,
    ttl: Duration,
}

impl EphemeralStore {
    /// Create a new ephemeral store.
    ///
    /// # Arguments
    /// * `max_capacity` — Maximum number of messages to keep.
    /// * `ttl` — Time-to-live for each message.
    pub fn new(max_capacity: usize, ttl: Duration) -> Self {
        Self {
            entries: VecDeque::with_capacity(max_capacity.min(1024)),
            max_capacity,
            ttl,
        }
    }

    /// Store a message. Evicts expired entries and drops the oldest
    /// if at capacity.
    pub fn store(&mut self, message: Message) {
        // First, evict expired entries
        self.evict_expired();

        // If still at capacity, drop the oldest
        if self.entries.len() >= self.max_capacity {
            self.entries.pop_front();
        }

        self.entries.push_back(StoreEntry {
            message,
            stored_at: Instant::now(),
        });
    }

    /// Get the N most recent messages (non-expired).
    pub fn recent(&mut self, limit: usize) -> Vec<&Message> {
        self.evict_expired();
        self.entries
            .iter()
            .rev()
            .take(limit)
            .map(|e| &e.message)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect()
    }

    /// Get all messages for a specific peer (non-expired).
    pub fn messages_with_peer(&mut self, peer_id: &str) -> Vec<&Message> {
        self.evict_expired();
        self.entries
            .iter()
            .filter(|e| e.message.sender_id == peer_id || e.message.recipient_id == peer_id)
            .map(|e| &e.message)
            .collect()
    }

    /// Find a message by ID.
    pub fn find_by_id(&self, message_id: &str) -> Option<&Message> {
        self.entries
            .iter()
            .find(|e| e.message.id == message_id)
            .map(|e| &e.message)
    }

    /// Number of stored messages (including potentially expired ones).
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if the store is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Clear all messages immediately.
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// Evict all expired entries.
    fn evict_expired(&mut self) {
        let now = Instant::now();
        while let Some(front) = self.entries.front() {
            if now.duration_since(front.stored_at) >= self.ttl {
                self.entries.pop_front();
            } else {
                break; // Entries are ordered by time, so we can stop
            }
        }
    }

    /// Get the number of non-expired entries.
    pub fn active_count(&mut self) -> usize {
        self.evict_expired();
        self.entries.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_msg(content: &str) -> Message {
        Message::new("alice".into(), "bob".into(), content.into())
    }

    #[test]
    fn test_store_and_retrieve() {
        let mut store = EphemeralStore::new(100, Duration::from_secs(3600));

        store.store(make_msg("hello"));
        store.store(make_msg("world"));

        let msgs = store.recent(10);
        assert_eq!(msgs.len(), 2);
        assert_eq!(msgs[0].content, "hello");
        assert_eq!(msgs[1].content, "world");
    }

    #[test]
    fn test_capacity_limit() {
        let mut store = EphemeralStore::new(2, Duration::from_secs(3600));

        store.store(make_msg("first"));
        store.store(make_msg("second"));
        store.store(make_msg("third"));

        let msgs = store.recent(10);
        assert_eq!(msgs.len(), 2);
        assert_eq!(msgs[0].content, "second");
        assert_eq!(msgs[1].content, "third");
    }

    #[test]
    fn test_ttl_eviction() {
        let mut store = EphemeralStore::new(100, Duration::from_millis(50));

        store.store(make_msg("old"));
        std::thread::sleep(Duration::from_millis(60));
        store.store(make_msg("new"));

        let msgs = store.recent(10);
        assert_eq!(msgs.len(), 1);
        assert_eq!(msgs[0].content, "new");
    }

    #[test]
    fn test_clear() {
        let mut store = EphemeralStore::new(100, Duration::from_secs(3600));
        store.store(make_msg("hello"));
        store.store(make_msg("world"));

        store.clear();
        assert!(store.is_empty());
    }

    #[test]
    fn test_find_by_id() {
        let mut store = EphemeralStore::new(100, Duration::from_secs(3600));
        let msg = make_msg("hello");
        let id = msg.id.clone();
        store.store(msg);

        assert!(store.find_by_id(&id).is_some());
        assert!(store.find_by_id("nonexistent").is_none());
    }

    #[test]
    fn test_messages_with_peer() {
        let mut store = EphemeralStore::new(100, Duration::from_secs(3600));

        store.store(Message::new("alice".into(), "bob".into(), "hi bob".into()));
        store.store(Message::new(
            "alice".into(),
            "charlie".into(),
            "hi charlie".into(),
        ));
        store.store(Message::new("bob".into(), "alice".into(), "hi alice".into()));

        let bob_msgs = store.messages_with_peer("bob");
        assert_eq!(bob_msgs.len(), 2); // alice→bob and bob→alice
    }
}
