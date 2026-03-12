//! `channels` endpoint group scaffolding.

use crate::client::HelixClient;
use rustitch_core::BroadcasterId;

/// Typed access to the `channels` Helix endpoints.
#[derive(Clone, Copy, Debug)]
pub struct ChannelsApi<'a> {
    client: &'a HelixClient,
}

impl<'a> ChannelsApi<'a> {
    pub(crate) fn new(client: &'a HelixClient) -> Self {
        Self { client }
    }

    /// Returns the owning Helix client.
    #[must_use]
    pub fn client(&self) -> &HelixClient {
        self.client
    }
}

/// Request scaffold for `Get Channel Information`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GetChannelInformationRequest {
    /// Broadcaster whose channel information should be fetched.
    pub broadcaster_id: BroadcasterId,
}
