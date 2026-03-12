//! `moderation` endpoint group scaffolding.

use crate::client::HelixClient;
use rustitch_core::{BroadcasterId, UserId};

/// Typed access to the `moderation` Helix endpoints.
#[derive(Clone, Copy, Debug)]
pub struct ModerationApi<'a> {
    client: &'a HelixClient,
}

impl<'a> ModerationApi<'a> {
    pub(crate) fn new(client: &'a HelixClient) -> Self {
        Self { client }
    }

    /// Returns the owning Helix client.
    #[must_use]
    pub fn client(&self) -> &HelixClient {
        self.client
    }
}

/// Request scaffold for a moderation action such as ban or timeout.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModerateUserRequest {
    /// Broadcaster in whose chat room the action applies.
    pub broadcaster_id: BroadcasterId,
    /// Moderator performing the action.
    pub moderator_id: UserId,
    /// User being moderated.
    pub user_id: UserId,
    /// Optional textual reason.
    pub reason: Option<String>,
}
