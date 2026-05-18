//! Quicko2 Relay Server
//!
//! A minimal, zero-knowledge relay server that routes encrypted messages
//! between clients. It never decrypts, never stores — just relays.

mod config;
mod directory;
mod listener;
mod rate_limit;
mod registry;
mod relay;

use std::sync::Arc;
use tracing_subscriber::EnvFilter;

use crate::config::ServerConfig;
use crate::directory::KeyDirectory;
use crate::registry::SessionRegistry;

/// Shared application state.
pub struct AppState {
    pub registry: SessionRegistry,
    pub directory: KeyDirectory,
    pub config: ServerConfig,
}

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("quicko2_server=info")),
        )
        .init();

    let config = ServerConfig::default();
    let bind_addr = config.bind_address.clone();

    let state = Arc::new(AppState {
        registry: SessionRegistry::new(),
        directory: KeyDirectory::new(),
        config,
    });

    tracing::info!("🚀 Quicko2 relay server starting on {}", bind_addr);

    let app = listener::create_router(state);

    let listener = tokio::net::TcpListener::bind(&bind_addr)
        .await
        .expect("Failed to bind to address");

    tracing::info!("✅ Server listening on {}", bind_addr);

    axum::serve(listener, app)
        .await
        .expect("Server failed");
}
