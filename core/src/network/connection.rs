//! WebSocket connection manager with exponential backoff reconnection.

use std::time::Duration;
use tokio::sync::mpsc;
use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::{connect_async, tungstenite::Message as WsMessage};
use tracing::{debug, error, info, warn};

use super::transport::TransportEvent;

/// Reconnection configuration.
#[derive(Debug, Clone)]
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
            initial_delay: Duration::from_millis(500),
            max_delay: Duration::from_secs(30),
            multiplier: 1.5,
            max_attempts: 0, // infinite
        }
    }
}

/// Manages the WebSocket connection lifecycle.
pub struct ConnectionManager {
    server_url: String,
    reconnect_config: ReconnectConfig,
}

impl ConnectionManager {
    /// Create a new connection manager.
    pub fn new(server_url: String) -> Self {
        Self {
            server_url,
            reconnect_config: ReconnectConfig::default(),
        }
    }

    /// Create with custom reconnection config.
    pub fn with_reconnect_config(server_url: String, config: ReconnectConfig) -> Self {
        Self {
            server_url,
            reconnect_config: config,
        }
    }

    /// Start the connection supervisor.
    ///
    /// Returns channels for sending/receiving transport events.
    /// The supervisor runs in a background task and automatically reconnects.
    pub fn start(
        &self,
    ) -> (
        mpsc::Sender<Vec<u8>>,
        mpsc::Receiver<TransportEvent>,
    ) {
        let (outbound_tx, outbound_rx) = mpsc::channel::<Vec<u8>>(256);
        let (event_tx, event_rx) = mpsc::channel::<TransportEvent>(256);

        let server_url = self.server_url.clone();
        let config = self.reconnect_config.clone();

        tokio::spawn(async move {
            let mut outbound_rx = outbound_rx;
            let mut attempt = 0;

            loop {
                info!("Connecting to relay server at {} (attempt {})...", server_url, attempt + 1);
                
                match connect_async(&server_url).await {
                    Ok((ws_stream, _)) => {
                        info!("Connected to relay server");
                        attempt = 0;
                        let _ = event_tx.send(TransportEvent::Connected).await;

                        let (mut ws_write, mut ws_read) = ws_stream.split();
                        let (internal_tx, mut internal_rx) = mpsc::channel::<Vec<u8>>(256);

                        // Bridge outbound_rx to ws_write, but allow it to survive disconnects
                        // We need to carefully handle the ownership here.
                        // Instead of giving outbound_rx to the writer task, we'll use a bridge.
                        
                        let event_tx_inner = event_tx.clone();
                        
                        // Reader task
                        let reader_event_tx = event_tx.clone();
                        let mut reader_handle = tokio::spawn(async move {
                            while let Some(msg_result) = ws_read.next().await {
                                match msg_result {
                                    Ok(WsMessage::Binary(data)) => {
                                        let _ = reader_event_tx.send(TransportEvent::MessageReceived(data.to_vec())).await;
                                    }
                                    Ok(WsMessage::Close(_)) => {
                                        warn!("Server closed connection");
                                        break;
                                    }
                                    Ok(WsMessage::Ping(data)) => {
                                        let _ = reader_event_tx.send(TransportEvent::PingReceived(data.to_vec())).await;
                                    }
                                    Err(e) => {
                                        error!("WebSocket error: {}", e);
                                        break;
                                    }
                                    _ => {}
                                }
                            }
                        });

                        // Writer task bridge
                        let mut writer_handle = tokio::spawn(async move {
                            while let Some(data) = internal_rx.recv().await {
                                if ws_write.send(WsMessage::Binary(data.into())).await.is_err() {
                                    break;
                                }
                            }
                        });

                        // Supervisor loop for this specific connection
                        loop {
                            tokio::select! {
                                Some(data) = outbound_rx.recv() => {
                                    if internal_tx.send(data).await.is_err() {
                                        break;
                                    }
                                }
                                _ = &mut reader_handle => {
                                    debug!("Reader task finished");
                                    break;
                                }
                                _ = &mut writer_handle => {
                                    debug!("Writer task finished");
                                    break;
                                }
                            }
                        }

                        let _ = event_tx_inner.send(TransportEvent::Disconnected("Connection lost".into())).await;
                    }
                    Err(e) => {
                        error!("Failed to connect: {}", e);
                    }
                }

                attempt += 1;
                if config.max_attempts > 0 && attempt >= config.max_attempts {
                    error!("Max reconnection attempts reached ({})", config.max_attempts);
                    let _ = event_tx.send(TransportEvent::ReconnectionFailed).await;
                    break;
                }

                let delay = backoff_delay(&config, attempt);
                warn!("Reconnecting in {:?}...", delay);
                let _ = event_tx.send(TransportEvent::Reconnecting(attempt)).await;
                tokio::time::sleep(delay).await;
            }
        });

        (outbound_tx, event_rx)
    }

    /// Get the server URL.
    pub fn server_url(&self) -> &str {
        &self.server_url
    }
}

/// Calculate backoff delay for a given attempt number.
fn backoff_delay(config: &ReconnectConfig, attempt: u32) -> Duration {
    let delay_ms = config.initial_delay.as_millis() as f64
        * config.multiplier.powi(attempt as i32);
    let capped_ms = delay_ms.min(config.max_delay.as_millis() as f64);
    Duration::from_millis(capped_ms as u64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backoff_delay() {
        let config = ReconnectConfig {
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(30),
            multiplier: 2.0,
            max_attempts: 0,
        };

        assert_eq!(backoff_delay(&config, 0), Duration::from_millis(100));
        assert_eq!(backoff_delay(&config, 1), Duration::from_millis(200));
        assert_eq!(backoff_delay(&config, 2), Duration::from_millis(400));
    }

    #[test]
    fn test_backoff_capped() {
        let config = ReconnectConfig {
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(1),
            multiplier: 2.0,
            max_attempts: 0,
        };

        assert_eq!(backoff_delay(&config, 10), Duration::from_secs(1));
    }
}
