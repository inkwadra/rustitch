//! Token provider implementations.

use crate::api::TwitchAuthApi;
use crate::error::AuthError;
use crate::token::{
    AppAccessToken, BoxFuture, StoredToken, TokenKey, TokenProvider, TokenValidationStatus,
    UserAccessToken,
};
use rustitch_core::{ClientId, RefreshToken, UserId};
use std::collections::{HashMap, VecDeque};
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};

type AppTokenResponses = HashMap<ClientId, Result<AppAccessToken, AuthError>>;
type UserTokenResponses = HashMap<(ClientId, UserId), Result<UserAccessToken, AuthError>>;
type ValidationResponses = HashMap<TokenKey, VecDeque<Result<TokenValidationStatus, AuthError>>>;
type RefreshResponses = HashMap<(ClientId, UserId), VecDeque<Result<UserAccessToken, AuthError>>>;

/// Twitch-backed provider implementation for client credentials, validation, and refresh.
#[derive(Clone, Debug)]
pub struct TwitchTokenProvider {
    api: TwitchAuthApi,
    default_scopes: Vec<String>,
}

impl TwitchTokenProvider {
    /// Creates a new Twitch-backed token provider.
    #[must_use]
    pub(crate) fn new(api: TwitchAuthApi, default_scopes: Vec<String>) -> Self {
        Self { api, default_scopes }
    }
}

impl TokenProvider for TwitchTokenProvider {
    fn app_token<'a>(
        &'a self,
        client_id: &'a ClientId,
    ) -> BoxFuture<'a, Result<AppAccessToken, AuthError>> {
        Box::pin(async move {
            if client_id != self.api.client_id() {
                return Err(AuthError::provider(
                    "app_token",
                    "twitch provider is scoped to a different client_id",
                ));
            }

            self.api.exchange_client_credentials(&self.default_scopes).await
        })
    }

    fn user_token<'a>(
        &'a self,
        client_id: &'a ClientId,
        _user_id: &'a UserId,
    ) -> BoxFuture<'a, Result<UserAccessToken, AuthError>> {
        Box::pin(async move {
            if client_id != self.api.client_id() {
                return Err(AuthError::provider(
                    "user_token",
                    "twitch provider is scoped to a different client_id",
                ));
            }

            Err(AuthError::provider(
                "user_token",
                "twitch provider cannot mint user tokens without an authorization-code or device-code exchange",
            ))
        })
    }

    fn validate_token<'a>(
        &'a self,
        token: &'a StoredToken,
    ) -> BoxFuture<'a, Result<TokenValidationStatus, AuthError>> {
        Box::pin(async move { self.api.validate_access_token(token.access_token()).await })
    }

    fn refresh_user_token<'a>(
        &'a self,
        client_id: &'a ClientId,
        _user_id: &'a UserId,
        refresh_token: &'a RefreshToken,
        scopes: &'a [String],
    ) -> BoxFuture<'a, Result<UserAccessToken, AuthError>> {
        Box::pin(async move {
            if client_id != self.api.client_id() {
                return Err(AuthError::provider(
                    "refresh_user_token",
                    "twitch provider is scoped to a different client_id",
                ));
            }

            self.api.exchange_refresh_token(refresh_token, scopes).await
        })
    }
}

/// Static provider for tests and examples.
#[derive(Debug, Default)]
pub struct StaticTokenProvider {
    app_tokens: Mutex<AppTokenResponses>,
    user_tokens: Mutex<UserTokenResponses>,
    validations: Mutex<ValidationResponses>,
    refreshes: Mutex<RefreshResponses>,
    app_token_requests: AtomicUsize,
    user_token_requests: AtomicUsize,
    validation_requests: AtomicUsize,
    refresh_requests: AtomicUsize,
}

impl StaticTokenProvider {
    /// Creates an empty static provider.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the app-token response for a client.
    pub fn set_app_token_response(
        &self,
        client_id: ClientId,
        response: Result<AppAccessToken, AuthError>,
    ) {
        let mut app_tokens = self.app_tokens.lock().expect("static provider mutex poisoned");
        app_tokens.insert(client_id, response);
    }

    /// Sets the user-token response for a `(client_id, user_id)` pair.
    pub fn set_user_token_response(
        &self,
        client_id: ClientId,
        user_id: UserId,
        response: Result<UserAccessToken, AuthError>,
    ) {
        let mut user_tokens = self.user_tokens.lock().expect("static provider mutex poisoned");
        user_tokens.insert((client_id, user_id), response);
    }

    /// Queues a validation response for a token key.
    pub fn push_validation_response(
        &self,
        key: TokenKey,
        response: Result<TokenValidationStatus, AuthError>,
    ) {
        let mut validations = self.validations.lock().expect("static provider mutex poisoned");
        validations.entry(key).or_default().push_back(response);
    }

    /// Queues a refresh response for a user token.
    pub fn push_refresh_response(
        &self,
        client_id: ClientId,
        user_id: UserId,
        response: Result<UserAccessToken, AuthError>,
    ) {
        let mut refreshes = self.refreshes.lock().expect("static provider mutex poisoned");
        refreshes.entry((client_id, user_id)).or_default().push_back(response);
    }

    /// Returns the number of app-token requests that were made.
    #[must_use]
    pub fn app_token_requests(&self) -> usize {
        self.app_token_requests.load(Ordering::SeqCst)
    }

    /// Returns the number of user-token requests that were made.
    #[must_use]
    pub fn user_token_requests(&self) -> usize {
        self.user_token_requests.load(Ordering::SeqCst)
    }

    /// Returns the number of validation requests that were made.
    #[must_use]
    pub fn validation_requests(&self) -> usize {
        self.validation_requests.load(Ordering::SeqCst)
    }

    /// Returns the number of refresh requests that were made.
    #[must_use]
    pub fn refresh_requests(&self) -> usize {
        self.refresh_requests.load(Ordering::SeqCst)
    }
}

impl TokenProvider for StaticTokenProvider {
    fn app_token<'a>(
        &'a self,
        client_id: &'a ClientId,
    ) -> BoxFuture<'a, Result<AppAccessToken, AuthError>> {
        Box::pin(async move {
            self.app_token_requests.fetch_add(1, Ordering::SeqCst);
            let app_tokens = self.app_tokens.lock().expect("static provider mutex poisoned");
            app_tokens.get(client_id).cloned().unwrap_or_else(|| {
                Err(AuthError::provider("app_token", "no app-token response configured"))
            })
        })
    }

    fn user_token<'a>(
        &'a self,
        client_id: &'a ClientId,
        user_id: &'a UserId,
    ) -> BoxFuture<'a, Result<UserAccessToken, AuthError>> {
        Box::pin(async move {
            self.user_token_requests.fetch_add(1, Ordering::SeqCst);
            let user_tokens = self.user_tokens.lock().expect("static provider mutex poisoned");
            user_tokens.get(&(client_id.clone(), user_id.clone())).cloned().unwrap_or_else(|| {
                Err(AuthError::provider("user_token", "no user-token response configured"))
            })
        })
    }

    fn validate_token<'a>(
        &'a self,
        token: &'a StoredToken,
    ) -> BoxFuture<'a, Result<TokenValidationStatus, AuthError>> {
        Box::pin(async move {
            self.validation_requests.fetch_add(1, Ordering::SeqCst);
            let key = token.key()?;
            let mut validations = self.validations.lock().expect("static provider mutex poisoned");
            validations.get_mut(&key).and_then(VecDeque::pop_front).unwrap_or_else(|| {
                Err(AuthError::provider(
                    "validate_token",
                    format!("no validation response configured for {key:?}"),
                ))
            })
        })
    }

    fn refresh_user_token<'a>(
        &'a self,
        client_id: &'a ClientId,
        user_id: &'a UserId,
        _refresh_token: &'a RefreshToken,
        _scopes: &'a [String],
    ) -> BoxFuture<'a, Result<UserAccessToken, AuthError>> {
        Box::pin(async move {
            self.refresh_requests.fetch_add(1, Ordering::SeqCst);
            let mut refreshes = self.refreshes.lock().expect("static provider mutex poisoned");
            refreshes
                .get_mut(&(client_id.clone(), user_id.clone()))
                .and_then(VecDeque::pop_front)
                .unwrap_or_else(|| {
                    Err(AuthError::provider("refresh_user_token", "no refresh response configured"))
                })
        })
    }
}
