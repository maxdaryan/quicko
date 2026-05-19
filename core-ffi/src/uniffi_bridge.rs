//! UniFFI bridge — exposes Quicko2 core to Kotlin/Swift via proc-macros.
//!
//! This module uses the modern `uniffi::export` proc-macro approach
//! instead of UDL files, avoiding the parser issues in uniffi 0.28.3.

use std::sync::{Arc, Mutex};
use tokio::runtime::Runtime;
use tokio::sync::mpsc;
use ::quicko2_core::{Client, ClientConfig, network::transport::TransportEvent};

// ── Error type ──────────────────────────────────────────────────────────

#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum QuickoError {
    #[error("Runtime error: {0}")]
    RuntimeError(String),
    #[error("Connect failed: {0}")]
    ConnectFailed(String),
    #[error("Send failed: {0}")]
    SendFailed(String),
    #[error("Key initialization failed: {0}")]
    KeyInitFailed(String),
    #[error("Recovery failed: {0}")]
    RecoveryFailed(String),
}

// ── Data records ────────────────────────────────────────────────────────

#[derive(uniffi::Record)]
pub struct QuickoMessage {
    pub id: String,
    pub sender_id: String,
    pub recipient_id: String,
    pub content: String,
    pub timestamp: i64,
    pub status: String,
}

#[derive(uniffi::Record)]
pub struct QuickoSessionInfo {
    pub session_id: String,
    pub display_name: String,
    pub invite_code: String,
}

#[derive(uniffi::Record)]
pub struct QuickoKeyInfo {
    pub formatted_key: String,
    pub display_name: String,
    pub seed_phrase: Vec<String>,
}

#[derive(uniffi::Record)]
pub struct QuickoTransportEvent {
    pub event_type: String,
    pub data: Option<Vec<u8>>,
    pub message: Option<String>,
    pub attempt: Option<u32>,
}

// ── Main client object ──────────────────────────────────────────────────

#[derive(uniffi::Object)]
pub struct QuickoClient {
    client: Arc<Mutex<Client>>,
    runtime: Runtime,
    event_rx: Arc<Mutex<Option<mpsc::Receiver<TransportEvent>>>>,
}

#[uniffi::export]
impl QuickoClient {
    /// Create a new QuickoClient with the given server URL and configuration.
    #[uniffi::constructor]
    pub fn new(server_url: String, session_ttl_secs: u64, max_messages: u32) -> Result<Self, QuickoError> {
        let runtime = Runtime::new()
            .map_err(|e| QuickoError::RuntimeError(e.to_string()))?;

        let config = ClientConfig {
            server_url,
            session_ttl_secs,
            max_messages: max_messages as usize,
            ..ClientConfig::default()
        };

        Ok(Self {
            client: Arc::new(Mutex::new(Client::new(config))),
            runtime,
            event_rx: Arc::new(Mutex::new(None)),
        })
    }

    /// Connect to the relay server.
    pub fn connect(&self) -> Result<(), QuickoError> {
        let mut client = self.client.lock().unwrap();
        let rx = self.runtime.block_on(async {
            client.connect().await
        }).map_err(|e| QuickoError::ConnectFailed(e.to_string()))?;

        let mut event_rx = self.event_rx.lock().unwrap();
        *event_rx = Some(rx);
        Ok(())
    }

    /// Disconnect from the relay server.
    pub fn disconnect(&self) {
        let mut client = self.client.lock().unwrap();
        client.disconnect();
        let mut event_rx = self.event_rx.lock().unwrap();
        *event_rx = None;
    }

    /// Poll for the next transport event (non-blocking).
    pub fn poll_event(&self) -> Option<QuickoTransportEvent> {
        let mut event_rx = self.event_rx.lock().unwrap();
        let rx = event_rx.as_mut()?;

        match rx.try_recv() {
            Ok(event) => {
                let (event_type, data, message, attempt) = match event {
                    TransportEvent::Connected => ("Connected", None, None, None),
                    TransportEvent::Disconnected(msg) => ("Disconnected", None, Some(msg), None),
                    TransportEvent::MessageReceived(data) => ("MessageReceived", Some(data), None, None),
                    TransportEvent::PingReceived(data) => ("PingReceived", Some(data), None, None),
                    TransportEvent::Reconnecting(n) => ("Reconnecting", None, None, Some(n)),
                    TransportEvent::ReconnectionFailed => ("ReconnectionFailed", None, None, None),
                };

                Some(QuickoTransportEvent {
                    event_type: event_type.to_string(),
                    data,
                    message,
                    attempt,
                })
            }
            Err(_) => None,
        }
    }

    /// Send raw binary data to the relay server.
    pub fn send_raw(&self, data: Vec<u8>) -> Result<(), QuickoError> {
        let client = self.client.lock().unwrap();
        self.runtime.block_on(async {
            client.send_raw(data).await
        }).map_err(|e| QuickoError::SendFailed(e.to_string()))
    }

    /// Create a new ephemeral session with a random identity.
    pub fn create_session(&self) -> QuickoSessionInfo {
        let mut client = self.client.lock().unwrap();
        let identity = client.create_session();
        QuickoSessionInfo {
            session_id: identity.session_id().to_string(),
            display_name: identity.display_name().to_string(),
            invite_code: identity.invite_code().to_string(),
        }
    }

    /// Get the current session info, if a session is active.
    pub fn session_info(&self) -> Option<QuickoSessionInfo> {
        let client = self.client.lock().unwrap();
        client.session().map(|s| QuickoSessionInfo {
            session_id: s.session_id().to_string(),
            display_name: s.display_name().to_string(),
            invite_code: s.invite_code().to_string(),
        })
    }

    /// Check whether a session is currently active.
    pub fn has_session(&self) -> bool {
        let client = self.client.lock().unwrap();
        client.session().is_some()
    }

    /// Destroy the current session and zeroize all sensitive data.
    pub fn destroy_session(&self) {
        let mut client = self.client.lock().unwrap();
        client.destroy_session();
    }

    /// Get the relay server URL.
    pub fn server_url(&self) -> String {
        let client = self.client.lock().unwrap();
        client.server_url().to_string()
    }

    /// Get the formatted QuickoKey string, if set.
    pub fn quicko_key(&self) -> Option<String> {
        let client = self.client.lock().unwrap();
        client.quicko_key_formatted()
    }

    /// Generate a new QuickoKey with a random seed phrase.
    pub fn generate_quickokey(&self) -> Result<QuickoKeyInfo, QuickoError> {
        let (words, key) = ::quicko2_core::quickokey::generate_seed_phrase();
        let formatted = key.format();
        let display_name = key.derive_display_name();

        let mut client = self.client.lock().unwrap();
        let config = ClientConfig {
            server_url: client.server_url().to_string(),
            ..ClientConfig::default()
        };
        *client = Client::from_quickokey(key, config)
            .map_err(|e| QuickoError::KeyInitFailed(e.to_string()))?;

        Ok(QuickoKeyInfo {
            formatted_key: formatted,
            display_name,
            seed_phrase: words,
        })
    }

    /// Recover a QuickoKey from a seed phrase (space-separated words).
    pub fn recover_quickokey(&self, seed_phrase: String) -> Result<QuickoKeyInfo, QuickoError> {
        let words: Vec<&str> = seed_phrase.split_whitespace().collect();
        let key = ::quicko2_core::quickokey::seed_phrase_to_key(&words)
            .map_err(|e| QuickoError::RecoveryFailed(e.to_string()))?;

        let formatted = key.format();
        let display_name = key.derive_display_name();

        let mut client = self.client.lock().unwrap();
        let config = ClientConfig {
            server_url: client.server_url().to_string(),
            ..ClientConfig::default()
        };
        *client = Client::from_quickokey(key, config)
            .map_err(|e| QuickoError::KeyInitFailed(e.to_string()))?;

        Ok(QuickoKeyInfo {
            formatted_key: formatted,
            display_name,
            seed_phrase: words.into_iter().map(|s| s.to_string()).collect(),
        })
    }
}
