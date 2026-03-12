//! `eventsub` endpoint group scaffolding.

use crate::client::HelixClient;
use rustitch_core::{SessionId, SubscriptionId};

/// Typed access to the `eventsub` Helix endpoints.
#[derive(Clone, Copy, Debug)]
pub struct EventSubApi<'a> {
    client: &'a HelixClient,
}

impl<'a> EventSubApi<'a> {
    pub(crate) fn new(client: &'a HelixClient) -> Self {
        Self { client }
    }

    /// Returns the owning Helix client.
    #[must_use]
    pub fn client(&self) -> &HelixClient {
        self.client
    }
}

/// Transport configuration for EventSub subscription management.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SubscriptionTransportConfig {
    /// WebSocket subscription transport.
    WebSocket {
        /// Session identifier of the active WebSocket connection.
        session_id: SessionId,
    },
    /// Webhook subscription transport.
    Webhook {
        /// Callback URL that Twitch will invoke.
        callback: String,
        /// Whether a webhook secret has been configured.
        secret_present: bool,
    },
}

/// Request scaffold for creating a Helix-managed EventSub subscription.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ManageSubscriptionRequest {
    /// EventSub type string, such as `channel.chat.message`.
    pub subscription_type: String,
    /// EventSub version string.
    pub version: String,
    /// Target transport configuration.
    pub transport: SubscriptionTransportConfig,
}

/// Response scaffold for EventSub management operations.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ManageSubscriptionResponse {
    /// Managed subscription identifier.
    pub subscription_id: SubscriptionId,
}
