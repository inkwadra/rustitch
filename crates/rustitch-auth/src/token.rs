//! Token types, storage contracts, and provider contracts.

use crate::error::AuthError;
use rustitch_core::{AccessToken, ClientId, RefreshToken, UserId};
use std::future::Future;
use std::pin::Pin;
use time::OffsetDateTime;

/// Pinned boxed future used by async token contracts.
pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// Identifies a stored token by owner and kind.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum TokenKey {
    /// Application token identified by client ID.
    App(ClientId),
    /// User token identified by user ID.
    User(UserId),
}

/// Token kind recognized by the auth layer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TokenKind {
    /// App access token.
    App,
    /// User access token.
    User,
}

/// Token persisted through the [`TokenStore`] boundary.
#[derive(Clone, Debug)]
pub struct StoredToken {
    /// Kind of token that was issued.
    pub kind: TokenKind,
    /// Granted OAuth scopes.
    pub scopes: Vec<String>,
    /// Access token secret.
    pub access_token: AccessToken,
    /// Refresh token secret, when refresh is supported.
    pub refresh_token: Option<RefreshToken>,
    /// Expiry timestamp, when one is known.
    pub expires_at: Option<OffsetDateTime>,
    /// Subject that owns the token, when the token is user-bound.
    pub subject: Option<UserId>,
    /// Client identifier used when issuing the token.
    pub client_id: ClientId,
}

/// App access token returned by a provider.
#[derive(Clone, Debug)]
pub struct AppAccessToken {
    /// Access token secret.
    pub access_token: AccessToken,
    /// Expiry timestamp, when one is known.
    pub expires_at: Option<OffsetDateTime>,
    /// Client identifier that owns the token.
    pub client_id: ClientId,
}

/// User access token returned by a provider.
#[derive(Clone, Debug)]
pub struct UserAccessToken {
    /// Access token secret.
    pub access_token: AccessToken,
    /// Refresh token secret, when one exists.
    pub refresh_token: Option<RefreshToken>,
    /// Expiry timestamp, when one is known.
    pub expires_at: Option<OffsetDateTime>,
    /// User that owns the token.
    pub user_id: UserId,
    /// Granted scopes.
    pub scopes: Vec<String>,
}

/// Persistence boundary for managed tokens.
pub trait TokenStore: Send + Sync {
    /// Retrieves a token by key, if it exists.
    fn get<'a>(
        &'a self,
        key: &'a TokenKey,
    ) -> BoxFuture<'a, Result<Option<StoredToken>, AuthError>>;

    /// Stores a token under the provided key.
    fn put<'a>(&'a self, key: TokenKey, token: StoredToken)
    -> BoxFuture<'a, Result<(), AuthError>>;

    /// Removes a token from the store.
    fn remove<'a>(&'a self, key: &'a TokenKey) -> BoxFuture<'a, Result<(), AuthError>>;
}

/// Retrieval and refresh boundary for app and user tokens.
pub trait TokenProvider: Send + Sync {
    /// Returns an app token for the provided client.
    fn app_token<'a>(
        &'a self,
        client_id: &'a ClientId,
    ) -> BoxFuture<'a, Result<AppAccessToken, AuthError>>;

    /// Returns a user token for the provided user.
    fn user_token<'a>(
        &'a self,
        user_id: &'a UserId,
    ) -> BoxFuture<'a, Result<UserAccessToken, AuthError>>;
}
