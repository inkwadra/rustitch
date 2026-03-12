//! Feature-gated EventSub subscription management scaffolding.

use rustitch_helix::HelixClient;

/// Transport-agnostic request scaffold for managing EventSub subscriptions.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SubscriptionManagementRequest {
    /// EventSub type string, such as `channel.chat.message`.
    pub subscription_type: String,
    /// EventSub version string.
    pub version: String,
}

/// Helix-backed EventSub management entry point.
#[derive(Clone, Debug)]
pub struct EventSubManagementClient {
    helix: HelixClient,
}

impl EventSubManagementClient {
    /// Creates a new management client over a typed Helix client.
    #[must_use]
    pub fn new(helix: HelixClient) -> Self {
        Self { helix }
    }

    /// Returns the underlying Helix client.
    #[must_use]
    pub fn helix(&self) -> &HelixClient {
        &self.helix
    }
}
