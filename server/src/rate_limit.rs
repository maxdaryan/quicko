//! Token-bucket rate limiter.

use dashmap::DashMap;
use std::time::Instant;

/// Per-session rate limiter using the token bucket algorithm.
pub struct RateLimiter {
    /// Map of session_id → bucket state.
    buckets: DashMap<String, TokenBucket>,
    /// Maximum tokens (burst capacity).
    max_tokens: u32,
    /// Token refill rate (tokens per second).
    refill_rate: f64,
}

/// A single token bucket for one session.
struct TokenBucket {
    tokens: f64,
    last_refill: Instant,
}

impl RateLimiter {
    /// Create a new rate limiter.
    ///
    /// # Arguments
    /// * `max_tokens` — Burst capacity.
    /// * `refill_rate` — Tokens added per second.
    pub fn new(max_tokens: u32, refill_rate: f64) -> Self {
        Self {
            buckets: DashMap::new(),
            max_tokens,
            refill_rate,
        }
    }

    /// Check if a session is allowed to send a message.
    ///
    /// Returns `true` if allowed (consumes a token), `false` if rate limited.
    pub fn check(&self, session_id: &str) -> bool {
        let mut bucket = self.buckets.entry(session_id.to_string()).or_insert_with(|| {
            TokenBucket {
                tokens: self.max_tokens as f64,
                last_refill: Instant::now(),
            }
        });

        // Refill tokens based on elapsed time
        let now = Instant::now();
        let elapsed = now.duration_since(bucket.last_refill).as_secs_f64();
        bucket.tokens = (bucket.tokens + elapsed * self.refill_rate)
            .min(self.max_tokens as f64);
        bucket.last_refill = now;

        // Try to consume a token
        if bucket.tokens >= 1.0 {
            bucket.tokens -= 1.0;
            true
        } else {
            false
        }
    }

    /// Remove a session's bucket (on disconnect).
    pub fn remove(&self, session_id: &str) {
        self.buckets.remove(session_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limiter_allows() {
        let limiter = RateLimiter::new(5, 1.0);
        for _ in 0..5 {
            assert!(limiter.check("test-session"));
        }
    }

    #[test]
    fn test_rate_limiter_blocks() {
        let limiter = RateLimiter::new(2, 1.0);
        assert!(limiter.check("test-session"));
        assert!(limiter.check("test-session"));
        assert!(!limiter.check("test-session")); // Should be blocked
    }

    #[test]
    fn test_rate_limiter_refills() {
        let limiter = RateLimiter::new(1, 100.0); // Fast refill
        assert!(limiter.check("test-session"));
        std::thread::sleep(std::time::Duration::from_millis(20));
        assert!(limiter.check("test-session")); // Should be refilled
    }
}
