//! `streams` endpoint group scaffolding.

use crate::client::HelixClient;
use rustitch_core::UserId;

/// Typed access to the `streams` Helix endpoints.
#[derive(Clone, Copy, Debug)]
pub struct StreamsApi<'a> {
    client: &'a HelixClient,
}

impl<'a> StreamsApi<'a> {
    pub(crate) fn new(client: &'a HelixClient) -> Self {
        Self { client }
    }

    /// Returns the owning Helix client.
    #[must_use]
    pub fn client(&self) -> &HelixClient {
        self.client
    }
}

/// Request scaffold for `Get Streams`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GetStreamsRequest {
    /// User identifiers whose streams should be inspected.
    pub user_ids: Vec<UserId>,
}
