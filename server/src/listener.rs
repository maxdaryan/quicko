//! WebSocket listener and HTTP router.

use std::sync::Arc;
use axum::{
    extract::{
        ws::{Message as AxumWsMessage, WebSocket, WebSocketUpgrade},
        State,
    },
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use futures_util::{SinkExt, StreamExt};
use tokio::sync::mpsc;

use crate::AppState;
use crate::relay;

/// Create the axum router with WebSocket and health endpoints.
pub fn create_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/ws", get(ws_handler))
        .route("/health", get(health_handler))
        .route("/", get(root_handler))
        .with_state(state)
}

/// Root handler — basic info page.
async fn root_handler() -> Html<&'static str> {
    Html("<h1>Quicko2 Relay Server</h1><p>WebSocket endpoint: /ws</p>")
}

/// Health check endpoint.
async fn health_handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let clients = state.registry.client_count();
    let sessions = state.registry.session_count();
    format!(
        "{{\"status\":\"ok\",\"clients\":{},\"sessions\":{}}}",
        clients, sessions
    )
}

/// WebSocket upgrade handler.
async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

/// Handle an individual WebSocket connection.
async fn handle_socket(socket: WebSocket, state: Arc<AppState>) {
    let (mut ws_sender, mut ws_receiver) = socket.split();

    // Create a channel for sending messages to this client
    let (tx, mut rx) = mpsc::channel::<Vec<u8>>(256);

    // Spawn writer task: forwards from channel to WebSocket
    let write_task = tokio::spawn(async move {
        while let Some(data) = rx.recv().await {
            if ws_sender
                .send(AxumWsMessage::Binary(data.into()))
                .await
                .is_err()
            {
                break;
            }
        }
    });

    // Read messages from WebSocket and relay them
    let mut session_id: Option<String> = None;

    while let Some(msg_result) = ws_receiver.next().await {
        match msg_result {
            Ok(AxumWsMessage::Binary(data)) => {
                let result = relay::handle_message(
                    &data,
                    &tx,
                    &state,
                    &mut session_id,
                )
                .await;

                if let Err(e) = result {
                    tracing::warn!("Message handling error: {}", e);
                }
            }
            Ok(AxumWsMessage::Close(_)) => {
                tracing::info!("Client disconnected (close frame)");
                break;
            }
            Err(e) => {
                tracing::warn!("WebSocket error: {}", e);
                break;
            }
            _ => {} // Ignore text, ping/pong (handled by axum)
        }
    }

    // Cleanup: unregister the client
    if let Some(ref sid) = session_id {
        state.registry.unregister(sid);
    }

    // Abort the writer task
    write_task.abort();
}
