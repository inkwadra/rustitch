//! High-level authentication client configuration and OAuth flow entry points.

use crate::api::TwitchAuthApi;
use crate::error::AuthError;
use crate::flow::{AuthorizationRequest, DeviceAuthorization, DeviceTokenPoll};
use crate::manager::{TokenManager, ValidationPolicy};
use crate::provider::TwitchTokenProvider;
use crate::token::{TokenProvider, TokenStore, UserAccessToken};
use rustitch_core::{AuthServiceConfig, ClientId, ClientSecret};
use std::sync::Arc;

/// High-level OAuth client configuration.
#[derive(Clone, Debug)]
pub struct AuthClient {
    client_id: ClientId,
    client_secret: Option<ClientSecret>,
    oauth_base_url: String,
    redirect_uri: Option<String>,
    default_scopes: Vec<String>,
    validation_policy: ValidationPolicy,
    http_client: reqwest::Client,
}

impl AuthClient {
    /// Starts building a new authentication client.
    #[must_use]
    pub fn builder() -> AuthClientBuilder {
        AuthClientBuilder::default()
    }

    /// Returns the configured Twitch application client identifier.
    #[must_use]
    pub fn client_id(&self) -> &ClientId {
        &self.client_id
    }

    /// Returns the redirect URI configured for authorization code flow.
    #[must_use]
    pub fn redirect_uri(&self) -> Option<&str> {
        self.redirect_uri.as_deref()
    }

    /// Returns the default scopes requested by this client.
    #[must_use]
    pub fn default_scopes(&self) -> &[String] {
        &self.default_scopes
    }

    /// Returns the OAuth base URL.
    #[must_use]
    pub fn oauth_base_url(&self) -> &str {
        &self.oauth_base_url
    }

    /// Returns the token validation policy.
    #[must_use]
    pub fn validation_policy(&self) -> ValidationPolicy {
        self.validation_policy
    }

    /// Returns the shared HTTP client used for OAuth requests.
    #[must_use]
    pub fn http_client(&self) -> &reqwest::Client {
        &self.http_client
    }

    /// Creates an authorization request stub using the configured defaults.
    #[must_use]
    pub fn authorization_request(&self) -> AuthorizationRequest {
        AuthorizationRequest {
            client_id: self.client_id.clone(),
            redirect_uri: self.redirect_uri.clone(),
            scopes: self.default_scopes.clone(),
            state: None,
            force_verify: false,
            pkce_challenge: None,
            authorization_endpoint: format!(
                "{}/authorize",
                self.oauth_base_url.trim_end_matches('/')
            ),
        }
    }

    /// Creates a Twitch-backed token provider using the configured OAuth settings.
    #[must_use]
    pub fn twitch_provider(&self) -> TwitchTokenProvider {
        TwitchTokenProvider::new(self.api(), self.default_scopes.clone())
    }

    /// Creates a token manager using the configured validation policy and Twitch provider.
    #[must_use]
    pub fn token_manager(&self, store: Arc<dyn TokenStore>) -> TokenManager {
        let provider: Arc<dyn TokenProvider> = Arc::new(self.twitch_provider());
        TokenManager::new(self.client_id.clone(), store, provider, self.validation_policy)
    }

    /// Creates a token manager using a custom token provider.
    #[must_use]
    pub fn token_manager_with_provider(
        &self,
        store: Arc<dyn TokenStore>,
        provider: Arc<dyn TokenProvider>,
    ) -> TokenManager {
        TokenManager::new(self.client_id.clone(), store, provider, self.validation_policy)
    }

    /// Exchanges an authorization code for a user token.
    pub async fn exchange_authorization_code(
        &self,
        code: impl AsRef<str>,
        pkce_verifier: Option<&str>,
    ) -> Result<UserAccessToken, AuthError> {
        let redirect_uri = self.redirect_uri.as_deref().ok_or_else(|| {
            AuthError::configuration("authorization code exchange requires a redirect_uri")
        })?;

        self.api().exchange_authorization_code(code.as_ref(), redirect_uri, pkce_verifier).await
    }

    /// Starts the Twitch device authorization flow using the configured default scopes.
    pub async fn start_device_authorization(&self) -> Result<DeviceAuthorization, AuthError> {
        self.start_device_authorization_with_scopes(&self.default_scopes).await
    }

    /// Starts the Twitch device authorization flow with explicit scopes.
    pub async fn start_device_authorization_with_scopes(
        &self,
        scopes: &[String],
    ) -> Result<DeviceAuthorization, AuthError> {
        self.api().start_device_authorization(scopes).await
    }

    /// Polls the device authorization flow once.
    pub async fn poll_device_token(
        &self,
        authorization: &DeviceAuthorization,
    ) -> Result<DeviceTokenPoll, AuthError> {
        self.api().poll_device_token(authorization).await
    }

    fn api(&self) -> TwitchAuthApi {
        TwitchAuthApi::new(
            self.client_id.clone(),
            self.client_secret.clone(),
            self.oauth_base_url.clone(),
            self.http_client.clone(),
        )
    }
}

/// Builder for [`AuthClient`].
#[derive(Clone, Debug, Default)]
pub struct AuthClientBuilder {
    client_id: Option<ClientId>,
    client_secret: Option<ClientSecret>,
    oauth_base_url: Option<String>,
    redirect_uri: Option<String>,
    default_scopes: Vec<String>,
    validation_policy: ValidationPolicy,
    http_client: Option<reqwest::Client>,
}

impl AuthClientBuilder {
    /// Seeds the builder from a shared auth service configuration.
    #[must_use]
    pub fn service_config(mut self, service_config: AuthServiceConfig) -> Self {
        self.client_id = Some(service_config.client_id);
        self.client_secret = Some(service_config.client_secret);
        self.oauth_base_url = Some(service_config.oauth_base_url);
        self
    }

    /// Sets the Twitch application client identifier.
    #[must_use]
    pub fn client_id(mut self, client_id: ClientId) -> Self {
        self.client_id = Some(client_id);
        self
    }

    /// Sets the Twitch application client secret.
    #[must_use]
    pub fn client_secret(mut self, client_secret: ClientSecret) -> Self {
        self.client_secret = Some(client_secret);
        self
    }

    /// Overrides the OAuth base URL.
    #[must_use]
    pub fn oauth_base_url(mut self, oauth_base_url: impl Into<String>) -> Self {
        self.oauth_base_url = Some(oauth_base_url.into());
        self
    }

    /// Sets the redirect URI for authorization code flow.
    #[must_use]
    pub fn redirect_uri(mut self, redirect_uri: impl Into<String>) -> Self {
        self.redirect_uri = Some(redirect_uri.into());
        self
    }

    /// Adds a default OAuth scope.
    #[must_use]
    pub fn scope(mut self, scope: impl Into<String>) -> Self {
        self.default_scopes.push(scope.into());
        self
    }

    /// Overrides the default token validation policy.
    #[must_use]
    pub fn validation_policy(mut self, validation_policy: ValidationPolicy) -> Self {
        self.validation_policy = validation_policy;
        self
    }

    /// Injects a preconfigured HTTP client.
    #[must_use]
    pub fn http_client(mut self, http_client: reqwest::Client) -> Self {
        self.http_client = Some(http_client);
        self
    }

    /// Builds the authentication client.
    pub fn build(self) -> Result<AuthClient, AuthError> {
        let client_id = self
            .client_id
            .ok_or_else(|| AuthError::configuration("auth client requires a client_id"))?;

        let http_client = match self.http_client {
            Some(http_client) => http_client,
            None => reqwest::ClientBuilder::new()
                .redirect(reqwest::redirect::Policy::none())
                .build()
                .map_err(|error| {
                    AuthError::configuration(format!("failed to build OAuth HTTP client: {error}"))
                })?,
        };

        Ok(AuthClient {
            client_id,
            client_secret: self.client_secret,
            oauth_base_url: self
                .oauth_base_url
                .unwrap_or_else(|| String::from("https://id.twitch.tv/oauth2")),
            redirect_uri: self.redirect_uri,
            default_scopes: self.default_scopes,
            validation_policy: self.validation_policy,
            http_client,
        })
    }
}
