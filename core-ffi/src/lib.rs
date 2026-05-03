//! PyO3 bindings for Quicko2 core.
//!
//! Exposes the Rust core as a Python module (`quicko2_core`) for use
//! by the macOS PyQt6 UI.

use pyo3::prelude::*;
use pyo3::exceptions::PyRuntimeError;
use tokio::runtime::Runtime;

use ::quicko2_core::{Client, ClientConfig};

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

/// The main Quicko2 client handle exposed to Python.
///
/// Usage:
/// ```python
/// import quicko2_core
/// client = quicko2_core.QuickoClient("ws://localhost:9900")
/// session = client.create_session()
/// print(f"Your name: {session.display_name}")
/// print(f"Invite code: {session.invite_code}")
/// ```
#[pyclass]
pub struct QuickoClient {
    client: Client,
    runtime: Runtime,
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
        })
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
}

/// Initialize the quicko2_core Python module.
#[pymodule]
fn quicko2_core(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<QuickoClient>()?;
    m.add_class::<PyMessage>()?;
    m.add_class::<PySessionInfo>()?;
    Ok(())
}
