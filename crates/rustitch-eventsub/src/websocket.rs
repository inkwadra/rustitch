//! WebSocket runtime scaffolding for EventSub.

use rustitch_core::SessionId;

/// Runtime limits for EventSub WebSocket sessions.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EventSubWebSocketConfig {
    /// Maximum enabled connections for a `(client_id, user_id)` pair.
    pub max_connections_per_identity: u8,
    /// Maximum enabled subscriptions per connection.
    pub max_enabled_subscriptions: u16,
    /// Maximum total cost per connection.
    pub max_total_cost: u8,
}

impl Default for EventSubWebSocketConfig {
    fn default() -> Self {
        Self { max_connections_per_identity: 3, max_enabled_subscriptions: 300, max_total_cost: 10 }
    }
}

/// Session lifecycle state for an EventSub WebSocket transport.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WebSocketSessionState {
    /// No session is established yet.
    Idle,
    /// Session is connected and healthy.
    Connected(SessionId),
    /// Session is reconnecting to a new endpoint.
    Reconnecting(SessionId),
    /// Session was revoked by Twitch.
    Revoked(SessionId),
}

/// WebSocket transport scaffold.
#[derive(Clone, Debug)]
pub struct EventSubWebSocketClient {
    config: EventSubWebSocketConfig,
    state: WebSocketSessionState,
}

impl EventSubWebSocketClient {
    /// Creates a WebSocket transport scaffold with the provided limits.
    #[must_use]
    pub fn new(config: EventSubWebSocketConfig) -> Self {
        Self { config, state: WebSocketSessionState::Idle }
    }

    /// Returns the configured runtime limits.
    #[must_use]
    pub fn config(&self) -> EventSubWebSocketConfig {
        self.config
    }

    /// Returns the current session state.
    #[must_use]
    pub fn state(&self) -> &WebSocketSessionState {
        &self.state
    }
}
