//! WebSocket connection manager with exponential backoff reconnection.

use std::time::Duration;
use tokio::sync::mpsc;
use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::{connect_async, tungstenite::Message as WsMessage};

use crate::error::{QuickoError, Result};
use super::transport::TransportEvent;

/// Reconnection configuration.
pub struct ReconnectConfig {
    /// Initial delay between reconnect attempts.
    pub initial_delay: Duration,
    /// Maximum delay between reconnect attempts.
    pub max_delay: Duration,
    /// Backoff multiplier.
    pub multiplier: f64,
    /// Maximum number of reconnect attempts (0 = infinite).
    pub max_attempts: u32,
}

impl Default for ReconnectConfig {
    fn default() -> Self {
        Self {
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(30),
            multiplier: 2.0,
            max_attempts: 0, // infinite
        }
    }
}

/// Manages the WebSocket connection lifecycle.
pub struct ConnectionManager {
    server_url: String,
    reconnect_config: ReconnectConfig,
    connected: bool,
}

impl ConnectionManager {
    /// Create a new connection manager.
    pub fn new(server_url: String) -> Self {
        Self {
            server_url,
            reconnect_config: ReconnectConfig::default(),
            connected: false,
        }
    }

    /// Create with custom reconnection config.
    pub fn with_reconnect_config(server_url: String, config: ReconnectConfig) -> Self {
        Self {
            server_url,
            reconnect_config: config,
            connected: false,
        }
    }

    /// Connect to the relay server.
    ///
    /// Returns channels for sending/receiving transport events.
    pub async fn connect(
        &mut self,
    ) -> Result<(
        mpsc::Sender<Vec<u8>>,
        mpsc::Receiver<TransportEvent>,
    )> {
        let (ws_stream, _) = connect_async(&self.server_url)
            .await
            .map_err(|e| QuickoError::ConnectionFailed(e.to_string()))?;

        let (mut ws_write, mut ws_read) = ws_stream.split();

        // Channel for outbound messages (UI → WebSocket)
        let (outbound_tx, mut outbound_rx) = mpsc::channel::<Vec<u8>>(256);

        // Channel for transport events (WebSocket → UI)
        let (event_tx, event_rx) = mpsc::channel::<TransportEvent>(256);

        self.connected = true;
        let event_tx_clone = event_tx.clone();

        // Spawn writer task
        tokio::spawn(async move {
            while let Some(data) = outbound_rx.recv().await {
                if ws_write
                    .send(WsMessage::Binary(data.into()))
                    .await
                    .is_err()
                {
                    break;
                }
            }
        });

        // Spawn reader task
        tokio::spawn(async move {
            while let Some(msg_result) = ws_read.next().await {
                match msg_result {
                    Ok(WsMessage::Binary(data)) => {
                        let _ = event_tx_clone
                            .send(TransportEvent::MessageReceived(data.to_vec()))
                            .await;
                    }
                    Ok(WsMessage::Close(_)) => {
                        let _ = event_tx_clone
                            .send(TransportEvent::Disconnected("Server closed connection".into()))
                            .await;
                        break;
                    }
                    Ok(WsMessage::Ping(data)) => {
                        // Pong is handled automatically by tungstenite
                        let _ = event_tx_clone
                            .send(TransportEvent::PingReceived(data.to_vec()))
                            .await;
                    }
                    Err(e) => {
                        let _ = event_tx_clone
                            .send(TransportEvent::Disconnected(e.to_string()))
                            .await;
                        break;
                    }
                    _ => {} // Ignore text, pong, etc.
                }
            }
        });

        // Send connected event
        let _ = event_tx.send(TransportEvent::Connected).await;

        Ok((outbound_tx, event_rx))
    }

    /// Calculate backoff delay for a given attempt number.
    pub fn backoff_delay(&self, attempt: u32) -> Duration {
        let delay_ms = self.reconnect_config.initial_delay.as_millis() as f64
            * self.reconnect_config.multiplier.powi(attempt as i32);
        let capped_ms = delay_ms.min(self.reconnect_config.max_delay.as_millis() as f64);
        Duration::from_millis(capped_ms as u64)
    }

    /// Check if connected.
    pub fn is_connected(&self) -> bool {
        self.connected
    }

    /// Get the server URL.
    pub fn server_url(&self) -> &str {
        &self.server_url
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backoff_delay() {
        let cm = ConnectionManager::new("ws://localhost:9900".into());

        let d0 = cm.backoff_delay(0);
        let d1 = cm.backoff_delay(1);
        let d2 = cm.backoff_delay(2);

        assert_eq!(d0, Duration::from_millis(100));
        assert_eq!(d1, Duration::from_millis(200));
        assert_eq!(d2, Duration::from_millis(400));
    }

    #[test]
    fn test_backoff_capped() {
        let cm = ConnectionManager::new("ws://localhost:9900".into());

        let d20 = cm.backoff_delay(20);
        assert!(d20 <= Duration::from_secs(30));
    }
}
