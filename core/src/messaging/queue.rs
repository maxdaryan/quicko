//! Outbound message queue for offline buffering.

use std::collections::VecDeque;

use super::message::Message;

/// Maximum number of messages to buffer while offline.
const MAX_QUEUE_SIZE: usize = 500;

/// Outbound message queue that buffers messages during disconnection.
pub struct MessageQueue {
    queue: VecDeque<Message>,
    max_size: usize,
}

impl MessageQueue {
    /// Create a new message queue with default capacity.
    pub fn new() -> Self {
        Self {
            queue: VecDeque::with_capacity(64),
            max_size: MAX_QUEUE_SIZE,
        }
    }

    /// Create a message queue with a custom maximum size.
    pub fn with_capacity(max_size: usize) -> Self {
        Self {
            queue: VecDeque::with_capacity(max_size.min(64)),
            max_size,
        }
    }

    /// Enqueue a message for sending.
    ///
    /// If the queue is full, the oldest message is dropped.
    pub fn enqueue(&mut self, message: Message) {
        if self.queue.len() >= self.max_size {
            self.queue.pop_front();
            tracing::warn!("Message queue full, dropping oldest message");
        }
        self.queue.push_back(message);
    }

    /// Dequeue the next message to send.
    pub fn dequeue(&mut self) -> Option<Message> {
        self.queue.pop_front()
    }

    /// Peek at the next message without removing it.
    pub fn peek(&self) -> Option<&Message> {
        self.queue.front()
    }

    /// Drain all messages from the queue.
    pub fn drain_all(&mut self) -> Vec<Message> {
        self.queue.drain(..).collect()
    }

    /// Number of messages in the queue.
    pub fn len(&self) -> usize {
        self.queue.len()
    }

    /// Check if the queue is empty.
    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    /// Clear all messages from the queue.
    pub fn clear(&mut self) {
        self.queue.clear();
    }
}

impl Default for MessageQueue {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_msg(content: &str) -> Message {
        Message::new("alice".into(), "bob".into(), content.into())
    }

    #[test]
    fn test_enqueue_dequeue() {
        let mut q = MessageQueue::new();
        q.enqueue(make_msg("hello"));
        q.enqueue(make_msg("world"));

        assert_eq!(q.len(), 2);
        assert_eq!(q.dequeue().unwrap().content, "hello");
        assert_eq!(q.dequeue().unwrap().content, "world");
        assert!(q.is_empty());
    }

    #[test]
    fn test_overflow_drops_oldest() {
        let mut q = MessageQueue::with_capacity(2);
        q.enqueue(make_msg("first"));
        q.enqueue(make_msg("second"));
        q.enqueue(make_msg("third"));

        assert_eq!(q.len(), 2);
        assert_eq!(q.dequeue().unwrap().content, "second");
    }

    #[test]
    fn test_drain_all() {
        let mut q = MessageQueue::new();
        q.enqueue(make_msg("a"));
        q.enqueue(make_msg("b"));
        q.enqueue(make_msg("c"));

        let msgs = q.drain_all();
        assert_eq!(msgs.len(), 3);
        assert!(q.is_empty());
    }
}
