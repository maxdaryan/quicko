//! Server configuration.

/// Server configuration.
pub struct ServerConfig {
    /// Address to bind to.
    pub bind_address: String,
    /// Maximum concurrent connections.
    pub max_connections: usize,
    /// Session buffer TTL (seconds) — how long to hold messages for offline peers.
    pub buffer_ttl_secs: u64,
    /// Rate limit: messages per second per session.
    pub rate_limit_per_second: u32,
    /// Maximum payload size in bytes.
    pub max_payload_size: usize,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            bind_address: "0.0.0.0:9900".to_string(),
            max_connections: 10_000,
            buffer_ttl_secs: 60,
            rate_limit_per_second: 30,
            max_payload_size: 1_048_576, // 1 MB
        }
    }
}
