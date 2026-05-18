//! Quicko2 Core — High-performance ephemeral messaging engine.
//!
//! This crate provides the core functionality for Quicko2:
//! - End-to-end encryption (X25519 + HKDF + AES-256-GCM)
//! - Ephemeral session management
//! - Wire protocol encoding/decoding
//! - In-memory message storage with TTL eviction
//! - WebSocket networking with auto-reconnect
//! - Message queuing and delivery receipts

pub mod crypto;
pub mod error;
pub mod messaging;
pub mod network;
pub mod protocol;
pub mod quickokey;
pub mod session;
pub mod store;

pub use error::{QuickoError, Result};
pub use quickokey::QuickoKey;
pub use session::identity::SessionIdentity;

use tokio::sync::mpsc;
use crate::network::connection::ConnectionManager;
use crate::network::transport::TransportEvent;

/// Core client that ties all modules together.
///
/// This is the primary entry point for the Quicko2 messaging engine.
/// It manages the session lifecycle, networking, encryption, and message flow.
pub struct Client {
    quicko_key: Option<quickokey::QuickoKey>,
    session: Option<session::identity::SessionIdentity>,
    store: store::EphemeralStore,
    config: ClientConfig,
    outbound_tx: Option<mpsc::Sender<Vec<u8>>>,
}

/// Configuration for the Quicko2 client.
#[derive(Debug, Clone)]
pub struct ClientConfig {
    /// Relay server URL (e.g., "wss://relay.quicko.dev")
    pub server_url: String,
    /// Session time-to-live in seconds (default: 3600)
    pub session_ttl_secs: u64,
    /// Maximum messages to keep in ephemeral store (default: 1000)
    pub max_messages: usize,
    /// Heartbeat interval in seconds (default: 15)
    pub heartbeat_interval_secs: u64,
    /// Auto-reconnect on connection loss (default: true)
    pub auto_reconnect: bool,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            server_url: "ws://127.0.0.1:9900".to_string(),
            session_ttl_secs: 3600,
            max_messages: 1000,
            heartbeat_interval_secs: 15,
            auto_reconnect: true,
        }
    }
}

impl Client {
    /// Create a new Quicko2 client with the given configuration.
    pub fn new(config: ClientConfig) -> Self {
        Self {
            quicko_key: None,
            session: None,
            store: store::EphemeralStore::new(
                config.max_messages,
                std::time::Duration::from_secs(config.session_ttl_secs),
            ),
            config,
            outbound_tx: None,
        }
    }

    /// Create a new client from an existing QuickoKey.
    ///
    /// The session identity and crypto keys are derived deterministically
    /// from the QuickoKey.
    pub fn from_quickokey(key: quickokey::QuickoKey, config: ClientConfig) -> Result<Self> {
        let session = session::identity::SessionIdentity::from_quickokey(
            &key,
            std::time::Duration::from_secs(config.session_ttl_secs),
        )?;
        Ok(Self {
            quicko_key: Some(key),
            session: Some(session),
            store: store::EphemeralStore::new(
                config.max_messages,
                std::time::Duration::from_secs(config.session_ttl_secs),
            ),
            config,
            outbound_tx: None,
        })
    }

    /// Connect to the relay server.
    ///
    /// Returns a receiver for transport events (connected, disconnected, messages, etc).
    pub async fn connect(&mut self) -> Result<mpsc::Receiver<TransportEvent>> {
        let cm = ConnectionManager::new(self.config.server_url.clone());
        let (tx, rx) = cm.start();
        self.outbound_tx = Some(tx);
        Ok(rx)
    }

    /// Disconnect from the relay server.
    pub fn disconnect(&mut self) {
        self.outbound_tx = None;
    }

    /// Send a raw binary message to the relay server.
    pub async fn send_raw(&self, data: Vec<u8>) -> Result<()> {
        if let Some(tx) = &self.outbound_tx {
            tx.send(data).await.map_err(|_| QuickoError::SendFailed("Connection closed".into()))
        } else {
            Err(QuickoError::ConnectionFailed("Not connected".into()))
        }
    }

    /// Generate a new random QuickoKey and seed phrase.
    ///
    /// Returns `(formatted_key, seed_phrase_words, client)` — the user
    /// should save the seed phrase to recover their key later.
    pub fn generate_key(config: ClientConfig) -> Result<(String, Vec<String>, Self)> {
        let (words, key) = quickokey::generate_seed_phrase();
        let formatted = key.format();
        let client = Self::from_quickokey(key, config)?;
        Ok((formatted, words, client))
    }

    /// Create a new ephemeral session (legacy, random identity).
    pub fn create_session(&mut self) -> &SessionIdentity {
        let identity = session::identity::SessionIdentity::generate(
            std::time::Duration::from_secs(self.config.session_ttl_secs),
        );
        self.session = Some(identity);
        self.session.as_ref().unwrap()
    }

    /// Get the current session identity, if any.
    pub fn session(&self) -> Option<&SessionIdentity> {
        self.session.as_ref()
    }

    /// Get the current QuickoKey, if any.
    pub fn quicko_key(&self) -> Option<&quickokey::QuickoKey> {
        self.quicko_key.as_ref()
    }

    /// Get the formatted QuickoKey string, if set.
    pub fn quicko_key_formatted(&self) -> Option<String> {
        self.quicko_key.as_ref().map(|k| k.format())
    }

    /// Destroy the current session and zeroize all data.
    pub fn destroy_session(&mut self) {
        self.session = None;
        self.quicko_key = None;
        self.store.clear();
        tracing::info!("Session destroyed, all data zeroized");
    }

    /// Get the server URL.
    pub fn server_url(&self) -> &str {
        &self.config.server_url
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let config = ClientConfig::default();
        let client = Client::new(config);
        assert!(client.session().is_none());
    }

    #[test]
    fn test_session_lifecycle() {
        let config = ClientConfig::default();
        let mut client = Client::new(config);

        // Create session
        let session = client.create_session();
        assert!(!session.session_id().is_empty());
        assert!(!session.display_name().is_empty());
        assert!(!session.invite_code().is_empty());

        // Session exists
        assert!(client.session().is_some());

        // Destroy session
        client.destroy_session();
        assert!(client.session().is_none());
    }
}
