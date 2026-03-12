//! High-level authentication client scaffolding.

use crate::error::AuthError;
use crate::flow::AuthorizationRequest;
use crate::manager::{TokenManager, ValidationPolicy};
use crate::token::{TokenProvider, TokenStore};
use rustitch_core::ClientId;
use std::sync::Arc;

/// High-level OAuth client configuration.
#[derive(Clone, Debug)]
pub struct AuthClient {
    client_id: ClientId,
    redirect_uri: Option<String>,
    default_scopes: Vec<String>,
    validation_policy: ValidationPolicy,
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

    /// Returns the token validation policy.
    #[must_use]
    pub fn validation_policy(&self) -> ValidationPolicy {
        self.validation_policy
    }

    /// Creates an authorization request stub using the configured defaults.
    #[must_use]
    pub fn authorization_request(&self) -> AuthorizationRequest {
        AuthorizationRequest {
            client_id: self.client_id.clone(),
            redirect_uri: self.redirect_uri.clone(),
            scopes: self.default_scopes.clone(),
        }
    }

    /// Creates a token manager using the configured validation policy.
    #[must_use]
    pub fn token_manager(
        &self,
        store: Arc<dyn TokenStore>,
        provider: Arc<dyn TokenProvider>,
    ) -> TokenManager {
        TokenManager::new(store, provider, self.validation_policy)
    }
}

/// Builder for [`AuthClient`].
#[derive(Clone, Debug, Default)]
pub struct AuthClientBuilder {
    client_id: Option<ClientId>,
    redirect_uri: Option<String>,
    default_scopes: Vec<String>,
    validation_policy: ValidationPolicy,
}

impl AuthClientBuilder {
    /// Sets the Twitch application client identifier.
    #[must_use]
    pub fn client_id(mut self, client_id: ClientId) -> Self {
        self.client_id = Some(client_id);
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

    /// Builds the authentication client.
    pub fn build(self) -> Result<AuthClient, AuthError> {
        let client_id = self.client_id.ok_or_else(|| {
            AuthError::Configuration(String::from("auth client requires a client_id"))
        })?;

        Ok(AuthClient {
            client_id,
            redirect_uri: self.redirect_uri,
            default_scopes: self.default_scopes,
            validation_policy: self.validation_policy,
        })
    }
}
