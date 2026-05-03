//! Session registry — tracks active sessions and their WebSocket connections.

use dashmap::DashMap;
use tokio::sync::mpsc;

/// Information about a connected client.
pub struct ConnectedClient {
    /// Client's session ID.
    pub session_id: String,
    /// Client's display name.
    pub display_name: String,
    /// Channel to send messages to this client.
    pub sender: mpsc::Sender<Vec<u8>>,
    /// Invite code this client belongs to (for session routing).
    pub invite_code: Option<String>,
}

/// Thread-safe session registry using DashMap for lock-free concurrent access.
pub struct SessionRegistry {
    /// Map of session_id → ConnectedClient.
    clients: DashMap<String, ConnectedClient>,
    /// Map of invite_code → Vec<session_id> (session membership).
    sessions: DashMap<String, Vec<String>>,
}

impl SessionRegistry {
    /// Create a new empty registry.
    pub fn new() -> Self {
        Self {
            clients: DashMap::new(),
            sessions: DashMap::new(),
        }
    }

    /// Register a new client connection.
    pub fn register(
        &self,
        session_id: String,
        display_name: String,
        sender: mpsc::Sender<Vec<u8>>,
    ) {
        self.clients.insert(
            session_id.clone(),
            ConnectedClient {
                session_id,
                display_name,
                sender,
                invite_code: None,
            },
        );
        tracing::info!("Client registered, total: {}", self.clients.len());
    }

    /// Unregister a client.
    pub fn unregister(&self, session_id: &str) {
        // Remove from session group if applicable
        if let Some(client) = self.clients.get(session_id) {
            if let Some(ref code) = client.invite_code {
                if let Some(mut members) = self.sessions.get_mut(code) {
                    members.retain(|id| id != session_id);
                    if members.is_empty() {
                        drop(members);
                        self.sessions.remove(code);
                    }
                }
            }
        }
        self.clients.remove(session_id);
        tracing::info!("Client unregistered, total: {}", self.clients.len());
    }

    /// Join a session by invite code.
    pub fn join_session(&self, session_id: &str, invite_code: &str) {
        // Update client's invite code
        if let Some(mut client) = self.clients.get_mut(session_id) {
            client.invite_code = Some(invite_code.to_string());
        }

        // Add to session group
        self.sessions
            .entry(invite_code.to_string())
            .or_insert_with(Vec::new)
            .push(session_id.to_string());
    }

    /// Get all peers in the same session (excluding the given session_id).
    pub fn get_peers(&self, session_id: &str, invite_code: &str) -> Vec<String> {
        self.sessions
            .get(invite_code)
            .map(|members| {
                members
                    .iter()
                    .filter(|id| id.as_str() != session_id)
                    .cloned()
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Send data to a specific client by session ID.
    pub async fn send_to(&self, session_id: &str, data: Vec<u8>) -> bool {
        if let Some(client) = self.clients.get(session_id) {
            client.sender.send(data).await.is_ok()
        } else {
            false
        }
    }

    /// Broadcast data to all peers in a session (excluding sender).
    pub async fn broadcast(
        &self,
        sender_id: &str,
        invite_code: &str,
        data: Vec<u8>,
    ) {
        let peers = self.get_peers(sender_id, invite_code);
        for peer_id in peers {
            if let Some(client) = self.clients.get(&peer_id) {
                let _ = client.sender.send(data.clone()).await;
            }
        }
    }

    /// Get the number of connected clients.
    pub fn client_count(&self) -> usize {
        self.clients.len()
    }

    /// Get the number of active sessions.
    pub fn session_count(&self) -> usize {
        self.sessions.len()
    }

    /// Broadcast data to all peers in the sender's session (looks up invite code internally).
    pub async fn broadcast_from(&self, sender_id: &str, data: Vec<u8>) {
        let invite_code = self
            .clients
            .get(sender_id)
            .and_then(|c| c.invite_code.clone());

        if let Some(code) = invite_code {
            self.broadcast(sender_id, &code, data).await;
        }
    }

    /// Check if an invite code exists.
    pub fn session_exists(&self, invite_code: &str) -> bool {
        self.sessions.contains_key(invite_code)
    }
}

impl Default for SessionRegistry {
    fn default() -> Self {
        Self::new()
    }
}
