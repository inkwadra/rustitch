//! Helix client configuration, auth selection, and endpoint accessors.

use crate::endpoints::{
    channels::ChannelsApi, chat::ChatApi, eventsub::EventSubApi, moderation::ModerationApi,
    streams::StreamsApi, users::UsersApi,
};
use crate::error::HelixError;
use crate::transport;
use reqwest::Client as HttpClient;
use rustitch_auth::TokenManager;
use rustitch_core::{ClientId, PageInfo, RateLimitMetadata, UserId};
use serde::de::DeserializeOwned;
use std::fmt;
use std::sync::Arc;

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

/// Selects which token path a Helix request should use.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HelixRequestAuth {
    /// Resolve and inject the app access token for the configured client.
    App,
    /// Resolve and inject the user access token for the provided user.
    User {
        /// User whose token should authorize the request.
        user_id: UserId,
    },
}

impl HelixRequestAuth {
    /// Returns the user identity required by the request, when one exists.
    #[must_use]
    pub fn user_id(&self) -> Option<&UserId> {
        match self {
            Self::App => None,
            Self::User { user_id } => Some(user_id),
        }
    }
}

/// Typed Helix response envelope with shared transport metadata.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HelixResponse<T> {
    data: T,
    pagination: Option<PageInfo>,
    rate_limit: Option<RateLimitMetadata>,
}

impl<T> HelixResponse<T> {
    pub(crate) fn new(
        data: T,
        pagination: Option<PageInfo>,
        rate_limit: Option<RateLimitMetadata>,
    ) -> Self {
        Self { data, pagination, rate_limit }
    }

    /// Returns the typed response data.
    #[must_use]
    pub fn data(&self) -> &T {
        &self.data
    }

    /// Consumes the response envelope and returns the typed response data.
    #[must_use]
    pub fn into_data(self) -> T {
        self.data
    }

    /// Returns the pagination metadata when the endpoint provided it.
    #[must_use]
    pub fn pagination(&self) -> Option<&PageInfo> {
        self.pagination.as_ref()
    }

    /// Returns the extracted rate-limit metadata when all required headers were present.
    #[must_use]
    pub fn rate_limit(&self) -> Option<&RateLimitMetadata> {
        self.rate_limit.as_ref()
    }
}

/// Typed Helix client.
#[derive(Clone)]
pub struct HelixClient {
    config: HelixClientConfig,
    http_client: HttpClient,
    token_manager: Arc<TokenManager>,
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

    /// Returns the token manager used to resolve request authorization.
    #[must_use]
    pub fn token_manager(&self) -> &TokenManager {
        self.token_manager.as_ref()
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

    pub(crate) async fn execute_get<T>(
        &self,
        path: &str,
        auth: HelixRequestAuth,
        query: &[(String, String)],
    ) -> Result<HelixResponse<T>, HelixError>
    where
        T: DeserializeOwned,
    {
        transport::execute_get(self, path, auth, query).await
    }
}

impl fmt::Debug for HelixClient {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("HelixClient")
            .field("config", &self.config)
            .field("http_client", &"reqwest::Client")
            .field("token_manager", &"TokenManager")
            .finish()
    }
}

/// Builder for [`HelixClient`].
#[derive(Clone, Default)]
pub struct HelixClientBuilder {
    client_id: Option<ClientId>,
    base_url: Option<String>,
    http_client: Option<HttpClient>,
    token_manager: Option<Arc<TokenManager>>,
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

    /// Injects the token manager used to resolve request authorization.
    #[must_use]
    pub fn token_manager(mut self, token_manager: Arc<TokenManager>) -> Self {
        self.token_manager = Some(token_manager);
        self
    }

    /// Builds the Helix client.
    pub fn build(self) -> Result<HelixClient, HelixError> {
        let token_manager = self.token_manager.ok_or_else(|| {
            HelixError::Configuration(String::from("helix client requires a token_manager"))
        })?;

        let derived_client_id = token_manager.client_id().clone();
        let client_id = match self.client_id {
            Some(client_id) if client_id == derived_client_id => client_id,
            Some(client_id) => {
                return Err(HelixError::Configuration(format!(
                    "helix client client_id {} does not match token manager client_id {}",
                    client_id, derived_client_id
                )));
            }
            None => derived_client_id,
        };

        let base_url = self.base_url.unwrap_or_else(|| String::from("https://api.twitch.tv/helix"));
        reqwest::Url::parse(&base_url).map_err(|error| {
            HelixError::Configuration(format!("helix client base_url is invalid: {error}"))
        })?;

        let config = HelixClientConfig { client_id, base_url };
        Ok(HelixClient { config, http_client: self.http_client.unwrap_or_default(), token_manager })
    }
}

impl fmt::Debug for HelixClientBuilder {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("HelixClientBuilder")
            .field("client_id", &self.client_id)
            .field("base_url", &self.base_url)
            .field("http_client", &self.http_client.as_ref().map(|_| "reqwest::Client"))
            .field("token_manager", &self.token_manager.as_ref().map(|_| "TokenManager"))
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::HelixClient;
    use crate::HelixError;
    use rustitch_auth::{InMemoryTokenStore, StaticTokenProvider, TokenManager, ValidationPolicy};
    use rustitch_core::ClientId;
    use std::sync::Arc;

    fn token_manager(client_id: ClientId) -> Arc<TokenManager> {
        Arc::new(TokenManager::new(
            client_id,
            Arc::new(InMemoryTokenStore::new()),
            Arc::new(StaticTokenProvider::new()),
            ValidationPolicy::default(),
        ))
    }

    #[test]
    fn builder_derives_client_id_from_token_manager() {
        let helix = HelixClient::builder()
            .token_manager(token_manager(ClientId::new("client-1")))
            .build()
            .expect("helix client should build");

        assert_eq!(helix.config().client_id.as_str(), "client-1");
    }

    #[test]
    fn builder_rejects_client_id_mismatch_with_token_manager() {
        let error = HelixClient::builder()
            .client_id(ClientId::new("client-2"))
            .token_manager(token_manager(ClientId::new("client-1")))
            .build()
            .expect_err("client_id mismatch must fail");

        assert!(matches!(error, HelixError::Configuration(_)));
    }
}
