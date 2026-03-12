//! Token types, storage contracts, provider contracts, and lifecycle models.

use crate::error::AuthError;
use rustitch_core::{AccessToken, ClientId, RefreshToken, UserId};
use std::future::Future;
use std::pin::Pin;
use std::time::Duration;
use time::OffsetDateTime;

/// Pinned boxed future used by async token contracts.
pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// Identifies a stored token by owning client and kind.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum TokenKey {
    /// Application token identified by client ID.
    App {
        /// Client that owns the token.
        client_id: ClientId,
    },
    /// User token identified by client ID and user ID.
    User {
        /// Client that owns the token.
        client_id: ClientId,
        /// User that owns the token.
        user_id: UserId,
    },
}

impl TokenKey {
    /// Creates an app-token key.
    #[must_use]
    pub fn app(client_id: ClientId) -> Self {
        Self::App { client_id }
    }

    /// Creates a user-token key.
    #[must_use]
    pub fn user(client_id: ClientId, user_id: UserId) -> Self {
        Self::User { client_id, user_id }
    }

    /// Returns the client that owns this token key.
    #[must_use]
    pub fn client_id(&self) -> &ClientId {
        match self {
            Self::App { client_id } | Self::User { client_id, .. } => client_id,
        }
    }

    /// Returns the user identity for user tokens.
    #[must_use]
    pub fn user_id(&self) -> Option<&UserId> {
        match self {
            Self::App { .. } => None,
            Self::User { user_id, .. } => Some(user_id),
        }
    }

    /// Returns the token kind represented by this key.
    #[must_use]
    pub fn kind(&self) -> TokenKind {
        match self {
            Self::App { .. } => TokenKind::App,
            Self::User { .. } => TokenKind::User,
        }
    }
}

/// Token kind recognized by the auth layer.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TokenKind {
    /// App access token.
    App,
    /// User access token.
    User,
}

/// Token persisted through the [`TokenStore`] boundary.
#[derive(Clone, Debug)]
pub struct StoredToken {
    kind: TokenKind,
    scopes: Vec<String>,
    access_token: AccessToken,
    refresh_token: Option<RefreshToken>,
    expires_at: Option<OffsetDateTime>,
    subject: Option<UserId>,
    client_id: ClientId,
}

impl StoredToken {
    /// Creates a stored app token.
    #[must_use]
    pub fn app(
        access_token: AccessToken,
        client_id: ClientId,
        expires_at: Option<OffsetDateTime>,
        scopes: Vec<String>,
    ) -> Self {
        Self {
            kind: TokenKind::App,
            scopes,
            access_token,
            refresh_token: None,
            expires_at,
            subject: None,
            client_id,
        }
    }

    /// Creates a stored user token.
    #[must_use]
    pub fn user(
        access_token: AccessToken,
        refresh_token: Option<RefreshToken>,
        client_id: ClientId,
        user_id: UserId,
        expires_at: Option<OffsetDateTime>,
        scopes: Vec<String>,
    ) -> Self {
        Self {
            kind: TokenKind::User,
            scopes,
            access_token,
            refresh_token,
            expires_at,
            subject: Some(user_id),
            client_id,
        }
    }

    /// Returns the token kind.
    #[must_use]
    pub fn kind(&self) -> TokenKind {
        self.kind
    }

    /// Returns the granted scopes.
    #[must_use]
    pub fn scopes(&self) -> &[String] {
        &self.scopes
    }

    /// Returns the access token secret.
    #[must_use]
    pub fn access_token(&self) -> &AccessToken {
        &self.access_token
    }

    /// Returns the refresh token secret when one exists.
    #[must_use]
    pub fn refresh_token(&self) -> Option<&RefreshToken> {
        self.refresh_token.as_ref()
    }

    /// Returns the token expiry timestamp, if known.
    #[must_use]
    pub fn expires_at(&self) -> Option<OffsetDateTime> {
        self.expires_at
    }

    /// Returns the user subject when the token is user-bound.
    #[must_use]
    pub fn subject(&self) -> Option<&UserId> {
        self.subject.as_ref()
    }

    /// Returns the client that owns the token.
    #[must_use]
    pub fn client_id(&self) -> &ClientId {
        &self.client_id
    }

    /// Returns the canonical storage key for this token.
    pub fn key(&self) -> Result<TokenKey, AuthError> {
        match (self.kind, self.subject.as_ref()) {
            (TokenKind::App, None) => Ok(TokenKey::app(self.client_id.clone())),
            (TokenKind::User, Some(user_id)) => {
                Ok(TokenKey::user(self.client_id.clone(), user_id.clone()))
            }
            (TokenKind::App, Some(_)) => Err(AuthError::invalid_stored_token(
                TokenKey::app(self.client_id.clone()),
                "app tokens must not contain a user subject",
            )),
            (TokenKind::User, None) => Err(AuthError::invalid_stored_token(
                TokenKey::app(self.client_id.clone()),
                "user tokens must contain a user subject",
            )),
        }
    }

    /// Returns whether the token is expired at the provided time.
    #[must_use]
    pub fn is_expired_at(&self, now: OffsetDateTime) -> bool {
        self.expires_at.is_some_and(|expires_at| expires_at <= now)
    }

    /// Validates the token invariants against the provided key.
    pub fn validate_for_key(&self, key: &TokenKey) -> Result<(), AuthError> {
        if self.kind != key.kind() {
            return Err(AuthError::TokenKindMismatch { expected: key.kind(), actual: self.kind });
        }

        if self.client_id != *key.client_id() {
            return Err(AuthError::invalid_stored_token(
                key.clone(),
                "stored token client_id does not match the token key",
            ));
        }

        match (self.kind, self.subject.as_ref(), key.user_id()) {
            (TokenKind::App, None, None) => Ok(()),
            (TokenKind::User, Some(subject), Some(user_id)) if subject == user_id => Ok(()),
            (TokenKind::App, Some(_), None) => Err(AuthError::invalid_stored_token(
                key.clone(),
                "app tokens must not include a user subject",
            )),
            (TokenKind::User, None, Some(_)) => Err(AuthError::invalid_stored_token(
                key.clone(),
                "user tokens must include a user subject",
            )),
            (TokenKind::User, Some(_), None) | (TokenKind::App, _, Some(_)) => {
                Err(AuthError::invalid_stored_token(
                    key.clone(),
                    "stored token key and token kind disagree about user ownership",
                ))
            }
            (TokenKind::User, None, None) => Err(AuthError::invalid_stored_token(
                key.clone(),
                "user tokens must include a user subject",
            )),
            (TokenKind::User, Some(_), Some(_)) => Err(AuthError::invalid_stored_token(
                key.clone(),
                "stored token subject does not match the token key user_id",
            )),
        }
    }

    /// Returns a copy of the token updated from a validation result.
    pub fn with_validation(
        &self,
        validation: &TokenValidation,
        now: OffsetDateTime,
    ) -> Result<Self, AuthError> {
        if self.client_id != validation.client_id {
            return Err(AuthError::invalid_stored_token(
                self.key().unwrap_or_else(|_| TokenKey::app(self.client_id.clone())),
                "validation client_id does not match the stored token client_id",
            ));
        }

        if self.kind == TokenKind::App && validation.user_id.is_some() {
            return Err(AuthError::invalid_stored_token(
                self.key().unwrap_or_else(|_| TokenKey::app(self.client_id.clone())),
                "app-token validation unexpectedly carried a user identity",
            ));
        }

        if self.kind == TokenKind::User && self.subject != validation.user_id {
            return Err(AuthError::invalid_stored_token(
                self.key().unwrap_or_else(|_| TokenKey::app(self.client_id.clone())),
                "validation user_id does not match the stored token subject",
            ));
        }

        let mut updated = self.clone();
        updated.scopes = validation.scopes.clone();
        updated.expires_at =
            Some(now + time::Duration::seconds(validation.expires_in.as_secs() as i64));
        Ok(updated)
    }
}

/// App access token returned by a provider.
#[derive(Clone, Debug)]
pub struct AppAccessToken {
    access_token: AccessToken,
    expires_at: Option<OffsetDateTime>,
    client_id: ClientId,
    scopes: Vec<String>,
}

impl AppAccessToken {
    /// Creates a new app access token.
    #[must_use]
    pub fn new(
        access_token: AccessToken,
        client_id: ClientId,
        expires_at: Option<OffsetDateTime>,
        scopes: Vec<String>,
    ) -> Self {
        Self { access_token, expires_at, client_id, scopes }
    }

    /// Returns the access token secret.
    #[must_use]
    pub fn access_token(&self) -> &AccessToken {
        &self.access_token
    }

    /// Returns the expiry timestamp, if known.
    #[must_use]
    pub fn expires_at(&self) -> Option<OffsetDateTime> {
        self.expires_at
    }

    /// Returns the owning client ID.
    #[must_use]
    pub fn client_id(&self) -> &ClientId {
        &self.client_id
    }

    /// Returns the granted scopes.
    #[must_use]
    pub fn scopes(&self) -> &[String] {
        &self.scopes
    }

    /// Converts the app token into a storable token.
    #[must_use]
    pub fn into_stored_token(self) -> StoredToken {
        StoredToken::app(self.access_token, self.client_id, self.expires_at, self.scopes)
    }
}

/// User access token returned by a provider or an auth flow.
#[derive(Clone, Debug)]
pub struct UserAccessToken {
    access_token: AccessToken,
    refresh_token: Option<RefreshToken>,
    expires_at: Option<OffsetDateTime>,
    user_id: UserId,
    client_id: ClientId,
    scopes: Vec<String>,
    login: Option<String>,
}

impl UserAccessToken {
    /// Creates a new user access token.
    #[must_use]
    pub fn new(
        access_token: AccessToken,
        refresh_token: Option<RefreshToken>,
        expires_at: Option<OffsetDateTime>,
        user_id: UserId,
        client_id: ClientId,
        scopes: Vec<String>,
        login: Option<String>,
    ) -> Self {
        Self { access_token, refresh_token, expires_at, user_id, client_id, scopes, login }
    }

    /// Returns the access token secret.
    #[must_use]
    pub fn access_token(&self) -> &AccessToken {
        &self.access_token
    }

    /// Returns the refresh token secret, when one exists.
    #[must_use]
    pub fn refresh_token(&self) -> Option<&RefreshToken> {
        self.refresh_token.as_ref()
    }

    /// Returns the expiry timestamp, if known.
    #[must_use]
    pub fn expires_at(&self) -> Option<OffsetDateTime> {
        self.expires_at
    }

    /// Returns the user that owns the token.
    #[must_use]
    pub fn user_id(&self) -> &UserId {
        &self.user_id
    }

    /// Returns the client that owns the token.
    #[must_use]
    pub fn client_id(&self) -> &ClientId {
        &self.client_id
    }

    /// Returns the granted scopes.
    #[must_use]
    pub fn scopes(&self) -> &[String] {
        &self.scopes
    }

    /// Returns the validated user login when it is known.
    #[must_use]
    pub fn login(&self) -> Option<&str> {
        self.login.as_deref()
    }

    /// Converts the user token into a storable token.
    #[must_use]
    pub fn into_stored_token(self) -> StoredToken {
        StoredToken::user(
            self.access_token,
            self.refresh_token,
            self.client_id,
            self.user_id,
            self.expires_at,
            self.scopes,
        )
    }
}

/// Result of a successful `/validate` call.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TokenValidation {
    /// Client that owns the token.
    pub client_id: ClientId,
    /// User identity for a user token, when one exists.
    pub user_id: Option<UserId>,
    /// Login associated with the user token, when one exists.
    pub login: Option<String>,
    /// Scopes granted to the token.
    pub scopes: Vec<String>,
    /// Remaining lifetime reported by Twitch.
    pub expires_in: Duration,
}

/// Result of validating a token against Twitch.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TokenValidationStatus {
    /// The token is valid and includes validation metadata.
    Valid(TokenValidation),
    /// The token is no longer valid.
    Invalid,
}

/// Typed lifecycle notifications emitted by [`crate::manager::TokenManager`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TokenLifecycleEvent {
    /// A token validated successfully.
    Validated {
        /// Token key that was validated.
        key: TokenKey,
        /// Validation metadata returned by Twitch.
        validation: TokenValidation,
    },
    /// An app token was reacquired after being missing, expired, or invalid.
    Reacquired {
        /// Token key that was reacquired.
        key: TokenKey,
    },
    /// A user token was refreshed successfully.
    Refreshed {
        /// Token key that was refreshed.
        key: TokenKey,
    },
    /// A token became invalid and was removed or replaced.
    Invalidated {
        /// Token key that became invalid.
        key: TokenKey,
    },
    /// A validation request failed due to transport or server issues.
    ValidationFailed {
        /// Token key associated with the failure.
        key: TokenKey,
        /// Error detail.
        error: AuthError,
    },
    /// A refresh request failed.
    RefreshFailed {
        /// Token key associated with the failure.
        key: TokenKey,
        /// Error detail.
        error: AuthError,
    },
}

/// Persistence boundary for managed tokens.
pub trait TokenStore: Send + Sync {
    /// Retrieves a token by key, if it exists.
    fn get<'a>(
        &'a self,
        key: &'a TokenKey,
    ) -> BoxFuture<'a, Result<Option<StoredToken>, AuthError>>;

    /// Stores a token under the provided key.
    fn put(&self, key: TokenKey, token: StoredToken) -> BoxFuture<'_, Result<(), AuthError>>;

    /// Removes a token from the store.
    fn remove<'a>(&'a self, key: &'a TokenKey) -> BoxFuture<'a, Result<(), AuthError>>;

    /// Lists tokens that belong to a single client scope.
    fn list_for_client<'a>(
        &'a self,
        client_id: &'a ClientId,
    ) -> BoxFuture<'a, Result<Vec<(TokenKey, StoredToken)>, AuthError>>;
}

/// Retrieval, validation, and refresh boundary for app and user tokens.
pub trait TokenProvider: Send + Sync {
    /// Returns an app token for the provided client.
    fn app_token<'a>(
        &'a self,
        client_id: &'a ClientId,
    ) -> BoxFuture<'a, Result<AppAccessToken, AuthError>>;

    /// Returns a user token for the provided client and user.
    fn user_token<'a>(
        &'a self,
        client_id: &'a ClientId,
        user_id: &'a UserId,
    ) -> BoxFuture<'a, Result<UserAccessToken, AuthError>>;

    /// Validates a token against Twitch.
    fn validate_token<'a>(
        &'a self,
        token: &'a StoredToken,
    ) -> BoxFuture<'a, Result<TokenValidationStatus, AuthError>>;

    /// Refreshes a user token using its refresh token.
    fn refresh_user_token<'a>(
        &'a self,
        client_id: &'a ClientId,
        user_id: &'a UserId,
        refresh_token: &'a RefreshToken,
        scopes: &'a [String],
    ) -> BoxFuture<'a, Result<UserAccessToken, AuthError>>;
}

impl TryFrom<StoredToken> for AppAccessToken {
    type Error = AuthError;

    fn try_from(value: StoredToken) -> Result<Self, Self::Error> {
        if value.kind != TokenKind::App {
            return Err(AuthError::TokenKindMismatch {
                expected: TokenKind::App,
                actual: value.kind,
            });
        }

        if value.subject.is_some() || value.refresh_token.is_some() {
            return Err(AuthError::invalid_stored_token(
                TokenKey::app(value.client_id.clone()),
                "app tokens must not contain a subject or refresh token",
            ));
        }

        Ok(Self::new(value.access_token, value.client_id, value.expires_at, value.scopes))
    }
}

impl TryFrom<StoredToken> for UserAccessToken {
    type Error = AuthError;

    fn try_from(value: StoredToken) -> Result<Self, Self::Error> {
        if value.kind != TokenKind::User {
            return Err(AuthError::TokenKindMismatch {
                expected: TokenKind::User,
                actual: value.kind,
            });
        }

        let user_id = value.subject.ok_or_else(|| {
            AuthError::invalid_stored_token(
                TokenKey::app(value.client_id.clone()),
                "user tokens must contain a user subject",
            )
        })?;

        Ok(Self::new(
            value.access_token,
            value.refresh_token,
            value.expires_at,
            user_id,
            value.client_id,
            value.scopes,
            None,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::{StoredToken, TokenKey, TokenKind};
    use rustitch_core::{AccessToken, ClientId, RefreshToken, UserId};

    #[test]
    fn stored_app_token_exposes_expected_key() {
        let token = StoredToken::app(
            AccessToken::new("app-token"),
            ClientId::new("client-1"),
            None,
            vec![String::from("scope:a")],
        );

        assert_eq!(
            token.key().expect("app token should produce a key"),
            TokenKey::app(ClientId::new("client-1"))
        );
    }

    #[test]
    fn stored_user_token_exposes_expected_key() {
        let token = StoredToken::user(
            AccessToken::new("user-token"),
            Some(RefreshToken::new("refresh-token")),
            ClientId::new("client-1"),
            UserId::new("user-7"),
            None,
            vec![String::from("scope:a")],
        );

        assert_eq!(
            token.key().expect("user token should produce a key"),
            TokenKey::user(ClientId::new("client-1"), UserId::new("user-7"))
        );
    }

    #[test]
    fn stored_token_detects_key_mismatch() {
        let token = StoredToken::user(
            AccessToken::new("user-token"),
            Some(RefreshToken::new("refresh-token")),
            ClientId::new("client-1"),
            UserId::new("user-7"),
            None,
            vec![String::from("scope:a")],
        );

        let error = token
            .validate_for_key(&TokenKey::user(ClientId::new("client-1"), UserId::new("user-8")))
            .expect_err("mismatched user should be rejected");

        assert!(matches!(error, crate::error::AuthError::InvalidStoredToken { .. }));
    }

    #[test]
    fn token_kind_matches_key_kind() {
        assert_eq!(TokenKey::app(ClientId::new("client-1")).kind(), TokenKind::App);
        assert_eq!(
            TokenKey::user(ClientId::new("client-1"), UserId::new("user-1")).kind(),
            TokenKind::User
        );
    }
}
