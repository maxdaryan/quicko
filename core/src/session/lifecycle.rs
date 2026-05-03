//! Session lifecycle state machine.

use std::time::Instant;

/// Possible states for a session.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionState {
    /// Session created but not yet connected.
    Created,
    /// Connecting to relay server.
    Connecting,
    /// Connected and ready for messaging.
    Connected,
    /// Actively chatting with peers.
    Chatting,
    /// Connection lost, attempting to reconnect.
    Reconnecting,
    /// Session expired (TTL reached).
    Expired,
    /// Session destroyed by user.
    Destroyed,
}

/// Tracks the lifecycle of a session with state transitions.
pub struct SessionLifecycle {
    state: SessionState,
    last_transition: Instant,
    reconnect_attempts: u32,
}

impl SessionLifecycle {
    /// Create a new session lifecycle in the Created state.
    pub fn new() -> Self {
        Self {
            state: SessionState::Created,
            last_transition: Instant::now(),
            reconnect_attempts: 0,
        }
    }

    /// Get the current state.
    pub fn state(&self) -> SessionState {
        self.state
    }

    /// Get time since last state transition.
    pub fn time_in_state(&self) -> std::time::Duration {
        self.last_transition.elapsed()
    }

    /// Get reconnect attempt count.
    pub fn reconnect_attempts(&self) -> u32 {
        self.reconnect_attempts
    }

    /// Transition to a new state, if the transition is valid.
    ///
    /// Returns `true` if the transition was accepted.
    pub fn transition(&mut self, new_state: SessionState) -> bool {
        let valid = matches!(
            (self.state, new_state),
            (SessionState::Created, SessionState::Connecting)
                | (SessionState::Connecting, SessionState::Connected)
                | (SessionState::Connecting, SessionState::Destroyed) // failed to connect
                | (SessionState::Connected, SessionState::Chatting)
                | (SessionState::Connected, SessionState::Reconnecting)
                | (SessionState::Connected, SessionState::Destroyed)
                | (SessionState::Chatting, SessionState::Connected) // peer left
                | (SessionState::Chatting, SessionState::Reconnecting)
                | (SessionState::Chatting, SessionState::Destroyed)
                | (SessionState::Reconnecting, SessionState::Connected)
                | (SessionState::Reconnecting, SessionState::Expired)
                | (SessionState::Reconnecting, SessionState::Destroyed)
                // Any state can be destroyed
                | (_, SessionState::Destroyed)
                | (_, SessionState::Expired)
        );

        if valid {
            if new_state == SessionState::Reconnecting {
                self.reconnect_attempts += 1;
            } else if new_state == SessionState::Connected {
                self.reconnect_attempts = 0;
            }
            self.state = new_state;
            self.last_transition = Instant::now();
        }

        valid
    }

    /// Check if the session is in an active state (can send/receive).
    pub fn is_active(&self) -> bool {
        matches!(
            self.state,
            SessionState::Connected | SessionState::Chatting
        )
    }
}

// Adding the missing Destroying variant for the transition table
impl SessionState {
    /// Check if this is a terminal state.
    pub fn is_terminal(&self) -> bool {
        matches!(self, SessionState::Expired | SessionState::Destroyed)
    }
}


impl Default for SessionLifecycle {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_state() {
        let lifecycle = SessionLifecycle::new();
        assert_eq!(lifecycle.state(), SessionState::Created);
        assert!(!lifecycle.is_active());
    }

    #[test]
    fn test_valid_transitions() {
        let mut lc = SessionLifecycle::new();

        assert!(lc.transition(SessionState::Connecting));
        assert_eq!(lc.state(), SessionState::Connecting);

        assert!(lc.transition(SessionState::Connected));
        assert_eq!(lc.state(), SessionState::Connected);
        assert!(lc.is_active());

        assert!(lc.transition(SessionState::Chatting));
        assert_eq!(lc.state(), SessionState::Chatting);
        assert!(lc.is_active());
    }

    #[test]
    fn test_reconnection_tracking() {
        let mut lc = SessionLifecycle::new();
        lc.transition(SessionState::Connecting);
        lc.transition(SessionState::Connected);

        // Disconnect
        lc.transition(SessionState::Reconnecting);
        assert_eq!(lc.reconnect_attempts(), 1);

        // Reconnect
        lc.transition(SessionState::Connected);
        assert_eq!(lc.reconnect_attempts(), 0);
    }

    #[test]
    fn test_destroy_from_any_state() {
        let mut lc = SessionLifecycle::new();
        assert!(lc.transition(SessionState::Destroyed));
        assert!(lc.state().is_terminal());
    }
}
