//! Heartbeat / keep-alive mechanism.

use std::time::{Duration, Instant};

/// Heartbeat tracker for connection health monitoring.
pub struct HeartbeatTracker {
    /// Interval between pings.
    interval: Duration,
    /// Timeout before considering connection dead.
    timeout: Duration,
    /// Last time we sent a ping.
    last_ping_sent: Option<Instant>,
    /// Last time we received a pong.
    last_pong_received: Option<Instant>,
    /// Measured round-trip latency.
    latency: Option<Duration>,
}

impl HeartbeatTracker {
    /// Create a new heartbeat tracker.
    pub fn new(interval: Duration, timeout: Duration) -> Self {
        Self {
            interval,
            timeout,
            last_ping_sent: None,
            last_pong_received: None,
            latency: None,
        }
    }

    /// Create with default settings (15s interval, 5s timeout).
    pub fn default_config() -> Self {
        Self::new(Duration::from_secs(15), Duration::from_secs(5))
    }

    /// Check if it's time to send a ping.
    pub fn should_ping(&self) -> bool {
        match self.last_ping_sent {
            Some(last) => last.elapsed() >= self.interval,
            None => true, // Never pinged
        }
    }

    /// Record that a ping was sent.
    pub fn ping_sent(&mut self) {
        self.last_ping_sent = Some(Instant::now());
    }

    /// Record that a pong was received. Updates latency measurement.
    pub fn pong_received(&mut self) {
        let now = Instant::now();
        if let Some(ping_time) = self.last_ping_sent {
            self.latency = Some(now.duration_since(ping_time));
        }
        self.last_pong_received = Some(now);
    }

    /// Check if the connection should be considered dead.
    pub fn is_timed_out(&self) -> bool {
        match self.last_ping_sent {
            Some(ping_time) => {
                // If we sent a ping and haven't received a pong within timeout
                match self.last_pong_received {
                    Some(pong_time) => {
                        // Pong received before ping? Shouldn't happen, but check
                        if pong_time < ping_time {
                            ping_time.elapsed() >= self.timeout
                        } else {
                            false
                        }
                    }
                    None => ping_time.elapsed() >= self.timeout,
                }
            }
            None => false,
        }
    }

    /// Get the last measured latency.
    pub fn latency(&self) -> Option<Duration> {
        self.latency
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_should_ping() {
        let hb = HeartbeatTracker::default_config();
        assert!(hb.should_ping());
    }

    #[test]
    fn test_latency_measurement() {
        let mut hb = HeartbeatTracker::default_config();
        hb.ping_sent();
        std::thread::sleep(Duration::from_millis(10));
        hb.pong_received();

        let latency = hb.latency().unwrap();
        assert!(latency >= Duration::from_millis(10));
    }

    #[test]
    fn test_not_timed_out_after_pong() {
        let mut hb = HeartbeatTracker::default_config();
        hb.ping_sent();
        hb.pong_received();
        assert!(!hb.is_timed_out());
    }
}
