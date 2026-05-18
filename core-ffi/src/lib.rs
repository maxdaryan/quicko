//! PyO3 bindings for Quicko2 core.
//!
//! Exposes the Rust core as a Python module (`quicko2_core`) for use
//! by the macOS PyQt6 UI.

use pyo3::prelude::*;
use pyo3::exceptions::PyRuntimeError;
use tokio::runtime::Runtime;
use tokio::sync::mpsc;
use ::quicko2_core::{Client, ClientConfig, network::transport::TransportEvent};

/// A message returned to Python.
#[pyclass]
#[derive(Clone)]
pub struct PyMessage {
    #[pyo3(get)]
    pub id: String,
    #[pyo3(get)]
    pub sender_id: String,
    #[pyo3(get)]
    pub recipient_id: String,
    #[pyo3(get)]
    pub content: String,
    #[pyo3(get)]
    pub timestamp: i64,
    #[pyo3(get)]
    pub status: String,
}

/// Session identity information returned to Python.
#[pyclass]
#[derive(Clone)]
pub struct PySessionInfo {
    #[pyo3(get)]
    pub session_id: String,
    #[pyo3(get)]
    pub display_name: String,
    #[pyo3(get)]
    pub invite_code: String,
}

/// QuickoKey information returned to Python.
#[pyclass]
#[derive(Clone)]
pub struct PyQuickoKeyInfo {
    #[pyo3(get)]
    pub formatted_key: String,
    #[pyo3(get)]
    pub display_name: String,
    #[pyo3(get)]
    pub seed_phrase: Vec<String>,
}

/// Networking events returned to Python.
#[pyclass]
pub struct PyTransportEvent {
    #[pyo3(get)]
    pub event_type: String,
    #[pyo3(get)]
    pub data: Option<Vec<u8>>,
    #[pyo3(get)]
    pub message: Option<String>,
    #[pyo3(get)]
    pub attempt: Option<u32>,
}

/// The main Quicko2 client handle exposed to Python.
///
/// Usage:
/// ```python
/// import quicko2_core
/// client = quicko2_core.QuickoClient("ws://localhost:9900")
/// client.connect()
/// # In an event loop:
/// event = client.poll_event()
/// if event and event.event_type == "Connected":
///     print("Connected!")
/// ```
#[pyclass]
pub struct QuickoClient {
    client: Client,
    runtime: Runtime,
    event_rx: Option<mpsc::Receiver<TransportEvent>>,
}

#[pymethods]
impl QuickoClient {
    /// Create a new Quicko2 client.
    ///
    /// Args:
    ///     server_url: WebSocket URL of the relay server (e.g., "ws://localhost:9900")
    ///     session_ttl_secs: Session time-to-live in seconds (default: 3600)
    ///     max_messages: Maximum messages in ephemeral store (default: 1000)
    #[new]
    #[pyo3(signature = (server_url, session_ttl_secs=3600, max_messages=1000))]
    fn new(server_url: &str, session_ttl_secs: u64, max_messages: usize) -> PyResult<Self> {
        let runtime = Runtime::new()
            .map_err(|e| PyRuntimeError::new_err(format!("Failed to create runtime: {}", e)))?;

        let config = ClientConfig {
            server_url: server_url.to_string(),
            session_ttl_secs,
            max_messages,
            ..ClientConfig::default()
        };

        Ok(Self {
            client: Client::new(config),
            runtime,
            event_rx: None,
        })
    }

    /// Connect to the relay server.
    fn connect(&mut self) -> PyResult<()> {
        let rx = self.runtime.block_on(async {
            self.client.connect().await
        }).map_err(|e| PyRuntimeError::new_err(format!("Connect failed: {}", e)))?;
        
        self.event_rx = Some(rx);
        Ok(())
    }

    /// Disconnect from the relay server.
    fn disconnect(&mut self) {
        self.client.disconnect();
        self.event_rx = None;
    }

    /// Poll for the next transport event.
    ///
    /// Returns a PyTransportEvent or None if no event is available.
    fn poll_event(&mut self) -> Option<PyTransportEvent> {
        let rx = self.event_rx.as_mut()?;
        
        // Try to receive without blocking
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
                
                Some(PyTransportEvent {
                    event_type: event_type.to_string(),
                    data,
                    message,
                    attempt,
                })
            }
            Err(_) => None,
        }
    }

    /// Send a raw binary message.
    fn send_raw(&mut self, data: Vec<u8>) -> PyResult<()> {
        self.runtime.block_on(async {
            self.client.send_raw(data).await
        }).map_err(|e| PyRuntimeError::new_err(format!("Send failed: {}", e)))
    }

    /// Create a new ephemeral session.
    ///
    /// Returns session info with session_id, display_name, and invite_code.
    fn create_session(&mut self) -> PyResult<PySessionInfo> {
        let identity = self.client.create_session();
        Ok(PySessionInfo {
            session_id: identity.session_id().to_string(),
            display_name: identity.display_name().to_string(),
            invite_code: identity.invite_code().to_string(),
        })
    }

    /// Get current session info, if any.
    fn session_info(&self) -> Option<PySessionInfo> {
        self.client.session().map(|s| PySessionInfo {
            session_id: s.session_id().to_string(),
            display_name: s.display_name().to_string(),
            invite_code: s.invite_code().to_string(),
        })
    }

    /// Check if a session is active.
    fn has_session(&self) -> bool {
        self.client.session().is_some()
    }

    /// Destroy the current session and zeroize all data.
    fn destroy_session(&mut self) {
        self.client.destroy_session();
    }

    /// Get the server URL.
    fn server_url(&self) -> String {
        self.client.server_url().to_string()
    }

    /// Get the current QuickoKey formatted string, if set.
    fn quicko_key(&self) -> Option<String> {
        self.client.quicko_key_formatted()
    }

    /// Generate a new QuickoKey and initialize the client with it.
    ///
    /// Returns a PyQuickoKeyInfo with the formatted key, display name,
    /// and seed phrase (12 words) for backup.
    fn generate_quickokey(&mut self) -> PyResult<PyQuickoKeyInfo> {
        let (words, key) = ::quicko2_core::quickokey::generate_seed_phrase();
        let formatted = key.format();
        let display_name = key.derive_display_name();

        let config = ClientConfig {
            server_url: self.client.server_url().to_string(),
            ..ClientConfig::default()
        };
        self.client = Client::from_quickokey(key, config)
            .map_err(|e| PyRuntimeError::new_err(format!("Key init failed: {}", e)))?;

        Ok(PyQuickoKeyInfo {
            formatted_key: formatted,
            display_name,
            seed_phrase: words,
        })
    }

    /// Recover a QuickoKey from a 12-word seed phrase.
    ///
    /// Args:
    ///     seed_phrase: Space-separated 12 words
    fn recover_quickokey(&mut self, seed_phrase: &str) -> PyResult<PyQuickoKeyInfo> {
        let words: Vec<&str> = seed_phrase.split_whitespace().collect();
        let key = ::quicko2_core::quickokey::seed_phrase_to_key(&words)
            .map_err(|e| PyRuntimeError::new_err(format!("Recovery failed: {}", e)))?;

        let formatted = key.format();
        let display_name = key.derive_display_name();

        let config = ClientConfig {
            server_url: self.client.server_url().to_string(),
            ..ClientConfig::default()
        };
        self.client = Client::from_quickokey(key, config)
            .map_err(|e| PyRuntimeError::new_err(format!("Key init failed: {}", e)))?;

        Ok(PyQuickoKeyInfo {
            formatted_key: formatted,
            display_name,
            seed_phrase: words.into_iter().map(|s| s.to_string()).collect(),
        })
    }
}

/// Initialize the quicko2_core Python module.
#[pymodule]
fn quicko2_core(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<QuickoClient>()?;
    m.add_class::<PyMessage>()?;
    m.add_class::<PySessionInfo>()?;
    m.add_class::<PyQuickoKeyInfo>()?;
    m.add_class::<PyTransportEvent>()?;
    Ok(())
}
