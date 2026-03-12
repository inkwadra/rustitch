//! Helix client scaffolding.

use crate::endpoints::{
    channels::ChannelsApi, chat::ChatApi, eventsub::EventSubApi, moderation::ModerationApi,
    streams::StreamsApi, users::UsersApi,
};
use crate::error::HelixError;
use reqwest::Client as HttpClient;
use rustitch_core::ClientId;

/// Configuration for the typed Helix client.
#[derive(Clone, Debug)]
pub struct HelixClientConfig {
    /// Twitch application client identifier used for `Client-Id` headers.
    pub client_id: ClientId,
    /// Base URL for the Helix API.
    pub base_url: String,
}

impl HelixClientConfig {
    /// Returns the standard Twitch production Helix configuration.
    #[must_use]
    pub fn production(client_id: ClientId) -> Self {
        Self { client_id, base_url: String::from("https://api.twitch.tv/helix") }
    }
}

/// Typed Helix client.
#[derive(Clone, Debug)]
pub struct HelixClient {
    config: HelixClientConfig,
    http_client: HttpClient,
}

impl HelixClient {
    /// Starts building a Helix client.
    #[must_use]
    pub fn builder() -> HelixClientBuilder {
        HelixClientBuilder::default()
    }

    /// Returns the immutable client configuration.
    #[must_use]
    pub fn config(&self) -> &HelixClientConfig {
        &self.config
    }

    /// Returns the shared HTTP client.
    #[must_use]
    pub fn http_client(&self) -> &HttpClient {
        &self.http_client
    }

    /// Returns the `users` endpoint group.
    #[must_use]
    pub fn users(&self) -> UsersApi<'_> {
        UsersApi::new(self)
    }

    /// Returns the `channels` endpoint group.
    #[must_use]
    pub fn channels(&self) -> ChannelsApi<'_> {
        ChannelsApi::new(self)
    }

    /// Returns the `streams` endpoint group.
    #[must_use]
    pub fn streams(&self) -> StreamsApi<'_> {
        StreamsApi::new(self)
    }

    /// Returns the `moderation` endpoint group.
    #[must_use]
    pub fn moderation(&self) -> ModerationApi<'_> {
        ModerationApi::new(self)
    }

    /// Returns the `chat` endpoint group.
    #[must_use]
    pub fn chat(&self) -> ChatApi<'_> {
        ChatApi::new(self)
    }

    /// Returns the `eventsub` endpoint group.
    #[must_use]
    pub fn eventsub(&self) -> EventSubApi<'_> {
        EventSubApi::new(self)
    }
}

/// Builder for [`HelixClient`].
#[derive(Clone, Debug, Default)]
pub struct HelixClientBuilder {
    client_id: Option<ClientId>,
    base_url: Option<String>,
    http_client: Option<HttpClient>,
}

impl HelixClientBuilder {
    /// Sets the Twitch application client identifier.
    #[must_use]
    pub fn client_id(mut self, client_id: ClientId) -> Self {
        self.client_id = Some(client_id);
        self
    }

    /// Overrides the Helix base URL.
    #[must_use]
    pub fn base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = Some(base_url.into());
        self
    }

    /// Injects a preconfigured HTTP client.
    #[must_use]
    pub fn http_client(mut self, http_client: HttpClient) -> Self {
        self.http_client = Some(http_client);
        self
    }

    /// Builds the Helix client.
    pub fn build(self) -> Result<HelixClient, HelixError> {
        let client_id = self.client_id.ok_or_else(|| {
            HelixError::Configuration(String::from("helix client requires a client_id"))
        })?;

        let config = HelixClientConfig {
            client_id,
            base_url: self.base_url.unwrap_or_else(|| String::from("https://api.twitch.tv/helix")),
        };

        Ok(HelixClient { config, http_client: self.http_client.unwrap_or_default() })
    }
}
