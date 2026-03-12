//! `users` endpoint group scaffolding.

use crate::client::HelixClient;
use rustitch_core::UserId;

/// Typed access to the `users` Helix endpoints.
#[derive(Clone, Copy, Debug)]
pub struct UsersApi<'a> {
    client: &'a HelixClient,
}

impl<'a> UsersApi<'a> {
    pub(crate) fn new(client: &'a HelixClient) -> Self {
        Self { client }
    }

    /// Returns the owning Helix client.
    #[must_use]
    pub fn client(&self) -> &HelixClient {
        self.client
    }
}

/// Request scaffold for `Get Users`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GetUsersRequest {
    /// User identifiers to resolve.
    pub user_ids: Vec<UserId>,
}
