//! Token orchestration for startup validation, retrieval, refresh, and persistence.

use crate::error::AuthError;
use crate::token::{
    AppAccessToken, StoredToken, TokenKey, TokenLifecycleEvent, TokenProvider, TokenStore,
    TokenValidation, TokenValidationStatus, UserAccessToken,
};
use rustitch_core::{ClientId, UserId};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use time::OffsetDateTime;
use tokio::sync::{Mutex, broadcast};
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

const DEFAULT_NOTIFICATION_CAPACITY: usize = 32;

/// Policy that controls token validation cadence.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ValidationPolicy {
    /// Whether active sessions should be validated at startup.
    pub validate_on_startup: bool,
    /// Interval for periodic token validation.
    pub revalidate_every: Duration,
}

impl Default for ValidationPolicy {
    fn default() -> Self {
        Self { validate_on_startup: true, revalidate_every: Duration::from_secs(60 * 60) }
    }
}

trait Clock: Send + Sync {
    fn now(&self) -> OffsetDateTime;
}

#[derive(Debug)]
struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> OffsetDateTime {
        OffsetDateTime::now_utc()
    }
}

/// Orchestrates token lifecycle over storage, retrieval, validation, and refresh.
#[derive(Clone)]
pub struct TokenManager {
    client_id: ClientId,
    store: Arc<dyn TokenStore>,
    provider: Arc<dyn TokenProvider>,
    validation_policy: ValidationPolicy,
    notifications: broadcast::Sender<TokenLifecycleEvent>,
    key_locks: Arc<Mutex<HashMap<TokenKey, Arc<Mutex<()>>>>>,
    clock: Arc<dyn Clock>,
}

impl TokenManager {
    /// Creates a token manager with the provided store, provider, and policy.
    #[must_use]
    pub fn new(
        client_id: ClientId,
        store: Arc<dyn TokenStore>,
        provider: Arc<dyn TokenProvider>,
        validation_policy: ValidationPolicy,
    ) -> Self {
        Self::new_with_clock(client_id, store, provider, validation_policy, Arc::new(SystemClock))
    }

    fn new_with_clock(
        client_id: ClientId,
        store: Arc<dyn TokenStore>,
        provider: Arc<dyn TokenProvider>,
        validation_policy: ValidationPolicy,
        clock: Arc<dyn Clock>,
    ) -> Self {
        let (notifications, _) = broadcast::channel(DEFAULT_NOTIFICATION_CAPACITY);
        Self {
            client_id,
            store,
            provider,
            validation_policy,
            notifications,
            key_locks: Arc::new(Mutex::new(HashMap::new())),
            clock,
        }
    }

    /// Returns the client ID owned by this manager.
    #[must_use]
    pub fn client_id(&self) -> &ClientId {
        &self.client_id
    }

    /// Returns the configured token store boundary.
    #[must_use]
    pub fn store(&self) -> Arc<dyn TokenStore> {
        Arc::clone(&self.store)
    }

    /// Returns the configured token provider boundary.
    #[must_use]
    pub fn provider(&self) -> Arc<dyn TokenProvider> {
        Arc::clone(&self.provider)
    }

    /// Returns the validation cadence policy.
    #[must_use]
    pub fn validation_policy(&self) -> ValidationPolicy {
        self.validation_policy
    }

    /// Subscribes to lifecycle notifications emitted by this manager.
    #[must_use]
    pub fn subscribe(&self) -> broadcast::Receiver<TokenLifecycleEvent> {
        self.notifications.subscribe()
    }

    /// Returns the current app access token, acquiring or rotating it when necessary.
    pub async fn app_token(&self) -> Result<AppAccessToken, AuthError> {
        let key = TokenKey::app(self.client_id.clone());
        let _guard = self.key_guard(&key).await;
        self.load_or_acquire_app_token(&key).await
    }

    /// Returns the current user access token for the provided user.
    pub async fn user_token(&self, user_id: &UserId) -> Result<UserAccessToken, AuthError> {
        let key = TokenKey::user(self.client_id.clone(), user_id.clone());
        let _guard = self.key_guard(&key).await;
        self.load_or_acquire_user_token(&key).await
    }

    /// Validates and repairs a token within this manager's client scope.
    pub async fn validate(&self, key: &TokenKey) -> Result<Option<TokenValidation>, AuthError> {
        self.ensure_client_scope(key)?;

        let _guard = self.key_guard(key).await;
        let stored = match self.store.get(key).await? {
            Some(stored) => stored,
            None => return Ok(None),
        };
        stored.validate_for_key(key)?;

        self.validate_stored_token(key, stored).await
    }

    /// Starts the background validation loop after performing startup validation.
    pub async fn start_validation_task(
        &self,
        cancellation_token: CancellationToken,
    ) -> Result<JoinHandle<()>, AuthError> {
        if self.validation_policy.validate_on_startup {
            self.validate_all_for_client().await?;
        }

        let manager = self.clone();
        let revalidate_every = self.validation_policy.revalidate_every;

        Ok(tokio::spawn(async move {
            let mut interval = tokio::time::interval(revalidate_every);
            interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);

            loop {
                tokio::select! {
                    _ = cancellation_token.cancelled() => break,
                    _ = interval.tick() => {
                        if let Err(error) = manager.validate_all_for_client().await {
                            tracing::debug!("background token validation failed: {error}");
                        }
                    }
                }
            }
        }))
    }

    async fn validate_all_for_client(&self) -> Result<(), AuthError> {
        let keys = self
            .store
            .list_for_client(&self.client_id)
            .await?
            .into_iter()
            .map(|(key, _)| key)
            .collect::<Vec<_>>();

        for key in keys {
            self.validate(&key).await?;
        }

        Ok(())
    }

    async fn load_or_acquire_app_token(&self, key: &TokenKey) -> Result<AppAccessToken, AuthError> {
        let now = self.clock.now();
        match self.store.get(key).await? {
            Some(stored) => {
                stored.validate_for_key(key)?;
                if stored.is_expired_at(now) {
                    let reacquired = self.provider.app_token(&self.client_id).await?;
                    let stored_token = reacquired.clone().into_stored_token();
                    self.store.put(key.clone(), stored_token).await?;
                    self.send(TokenLifecycleEvent::Reacquired { key: key.clone() });
                    Ok(reacquired)
                } else {
                    AppAccessToken::try_from(stored)
                }
            }
            None => {
                let acquired = self.provider.app_token(&self.client_id).await?;
                let stored_token = acquired.clone().into_stored_token();
                self.store.put(key.clone(), stored_token).await?;
                self.send(TokenLifecycleEvent::Reacquired { key: key.clone() });
                Ok(acquired)
            }
        }
    }

    async fn load_or_acquire_user_token(
        &self,
        key: &TokenKey,
    ) -> Result<UserAccessToken, AuthError> {
        let now = self.clock.now();
        let user_id = match key {
            TokenKey::User { user_id, .. } => user_id,
            TokenKey::App { .. } => {
                return Err(AuthError::TokenKindMismatch {
                    expected: crate::token::TokenKind::User,
                    actual: crate::token::TokenKind::App,
                });
            }
        };

        match self.store.get(key).await? {
            Some(stored) => {
                stored.validate_for_key(key)?;
                if stored.is_expired_at(now) {
                    let refreshed = self.refresh_stored_user_token(key, stored).await?;
                    Ok(UserAccessToken::try_from(refreshed)?)
                } else {
                    UserAccessToken::try_from(stored)
                }
            }
            None => {
                let acquired = self.provider.user_token(&self.client_id, user_id).await?;
                let stored_token = acquired.clone().into_stored_token();
                self.store.put(key.clone(), stored_token).await?;
                self.send(TokenLifecycleEvent::Reacquired { key: key.clone() });
                Ok(acquired)
            }
        }
    }

    async fn validate_stored_token(
        &self,
        key: &TokenKey,
        stored: StoredToken,
    ) -> Result<Option<TokenValidation>, AuthError> {
        match self.provider.validate_token(&stored).await {
            Ok(TokenValidationStatus::Valid(validation)) => {
                let updated = stored.with_validation(&validation, self.clock.now())?;
                self.store.put(key.clone(), updated).await?;
                self.send(TokenLifecycleEvent::Validated {
                    key: key.clone(),
                    validation: validation.clone(),
                });
                Ok(Some(validation))
            }
            Ok(TokenValidationStatus::Invalid) => {
                self.send(TokenLifecycleEvent::Invalidated { key: key.clone() });
                self.repair_invalid_token(key, stored).await
            }
            Err(error) => {
                self.send(TokenLifecycleEvent::ValidationFailed {
                    key: key.clone(),
                    error: error.clone(),
                });
                Err(error)
            }
        }
    }

    async fn repair_invalid_token(
        &self,
        key: &TokenKey,
        stored: StoredToken,
    ) -> Result<Option<TokenValidation>, AuthError> {
        match key {
            TokenKey::App { .. } => {
                let reacquired = self.provider.app_token(&self.client_id).await?;
                let stored_token = reacquired.into_stored_token();
                self.store.put(key.clone(), stored_token).await?;
                self.send(TokenLifecycleEvent::Reacquired { key: key.clone() });
                Ok(None)
            }
            TokenKey::User { user_id, .. } => {
                if stored.refresh_token().is_some() {
                    let refreshed = self.refresh_stored_user_token(key, stored).await?;
                    if let Ok(TokenValidationStatus::Valid(validation)) =
                        self.provider.validate_token(&refreshed).await
                    {
                        let updated = refreshed.with_validation(&validation, self.clock.now())?;
                        self.store.put(key.clone(), updated).await?;
                        self.send(TokenLifecycleEvent::Validated {
                            key: key.clone(),
                            validation: validation.clone(),
                        });
                        return Ok(Some(validation));
                    }

                    Ok(None)
                } else {
                    self.store.remove(key).await?;
                    Err(AuthError::validation(
                        TokenKey::user(self.client_id.clone(), user_id.clone()),
                        "user token became invalid and cannot be refreshed",
                    ))
                }
            }
        }
    }

    async fn refresh_stored_user_token(
        &self,
        key: &TokenKey,
        stored: StoredToken,
    ) -> Result<StoredToken, AuthError> {
        let user_id = stored.subject().cloned().ok_or_else(|| {
            AuthError::invalid_stored_token(key.clone(), "user token is missing a user subject")
        })?;
        let refresh_token = stored.refresh_token().cloned().ok_or_else(|| {
            AuthError::refresh(key.clone(), "user token is missing a refresh token")
        })?;

        match self
            .provider
            .refresh_user_token(&self.client_id, &user_id, &refresh_token, stored.scopes())
            .await
        {
            Ok(refreshed) => {
                let stored_token = refreshed.into_stored_token();
                self.store.put(key.clone(), stored_token.clone()).await?;
                self.send(TokenLifecycleEvent::Refreshed { key: key.clone() });
                Ok(stored_token)
            }
            Err(error) => {
                self.send(TokenLifecycleEvent::RefreshFailed {
                    key: key.clone(),
                    error: error.clone(),
                });
                self.store.remove(key).await?;
                self.send(TokenLifecycleEvent::Invalidated { key: key.clone() });
                Err(error)
            }
        }
    }

    fn ensure_client_scope(&self, key: &TokenKey) -> Result<(), AuthError> {
        if key.client_id() != &self.client_id {
            return Err(AuthError::configuration(format!(
                "token manager for client {} cannot manage token {:?}",
                self.client_id, key
            )));
        }

        Ok(())
    }

    fn send(&self, event: TokenLifecycleEvent) {
        let _ = self.notifications.send(event);
    }

    async fn key_guard(&self, key: &TokenKey) -> tokio::sync::OwnedMutexGuard<()> {
        let lock = {
            let mut locks = self.key_locks.lock().await;
            locks.entry(key.clone()).or_insert_with(|| Arc::new(Mutex::new(()))).clone()
        };

        lock.lock_owned().await
    }
}

#[cfg(test)]
mod tests {
    use super::{Clock, TokenManager, ValidationPolicy};
    use crate::error::AuthError;
    use crate::provider::StaticTokenProvider;
    use crate::store::InMemoryTokenStore;
    use crate::token::{
        AppAccessToken, StoredToken, TokenKey, TokenLifecycleEvent, TokenProvider, TokenStore,
        TokenValidation, TokenValidationStatus, UserAccessToken,
    };
    use rustitch_core::{AccessToken, ClientId, RefreshToken, UserId};
    use std::sync::Arc;
    use std::time::Duration;
    use time::OffsetDateTime;
    use tokio_util::sync::CancellationToken;

    #[derive(Debug)]
    struct FixedClock {
        now: OffsetDateTime,
    }

    impl Clock for FixedClock {
        fn now(&self) -> OffsetDateTime {
            self.now
        }
    }

    fn manager_with_clock(
        client_id: ClientId,
        store: Arc<dyn TokenStore>,
        provider: Arc<dyn TokenProvider>,
        now: OffsetDateTime,
        validation_policy: ValidationPolicy,
    ) -> TokenManager {
        TokenManager::new_with_clock(
            client_id,
            store,
            provider,
            validation_policy,
            Arc::new(FixedClock { now }),
        )
    }

    #[tokio::test]
    async fn app_token_is_acquired_and_persisted_when_missing() {
        let client_id = ClientId::new("client-1");
        let store = Arc::new(InMemoryTokenStore::new());
        let provider = Arc::new(StaticTokenProvider::new());
        provider.set_app_token_response(
            client_id.clone(),
            Ok(AppAccessToken::new(
                AccessToken::new("app-token"),
                client_id.clone(),
                None,
                Vec::new(),
            )),
        );

        let manager = manager_with_clock(
            client_id.clone(),
            store.clone(),
            provider.clone(),
            OffsetDateTime::now_utc(),
            ValidationPolicy::default(),
        );

        let token = manager.app_token().await.expect("app token should resolve");
        assert_eq!(token.access_token().expose_secret(), "app-token");
        assert_eq!(provider.app_token_requests(), 1);
        assert!(
            store.get(&TokenKey::app(client_id)).await.expect("store get should succeed").is_some()
        );
    }

    #[tokio::test]
    async fn expired_user_token_refreshes_and_rotates_refresh_token() {
        let client_id = ClientId::new("client-1");
        let user_id = UserId::new("user-1");
        let store = Arc::new(InMemoryTokenStore::new());
        let provider = Arc::new(StaticTokenProvider::new());
        let now = OffsetDateTime::now_utc();

        store
            .put(
                TokenKey::user(client_id.clone(), user_id.clone()),
                StoredToken::user(
                    AccessToken::new("old-token"),
                    Some(RefreshToken::new("old-refresh")),
                    client_id.clone(),
                    user_id.clone(),
                    Some(now - time::Duration::minutes(1)),
                    vec![String::from("chat:read")],
                ),
            )
            .await
            .expect("seed token should store");
        provider.push_refresh_response(
            client_id.clone(),
            user_id.clone(),
            Ok(UserAccessToken::new(
                AccessToken::new("new-token"),
                Some(RefreshToken::new("new-refresh")),
                Some(now + time::Duration::hours(1)),
                user_id.clone(),
                client_id.clone(),
                vec![String::from("chat:read")],
                Some(String::from("user_login")),
            )),
        );

        let manager = manager_with_clock(
            client_id.clone(),
            store.clone(),
            provider.clone(),
            now,
            ValidationPolicy::default(),
        );

        let token = manager.user_token(&user_id).await.expect("user token should refresh");

        assert_eq!(token.access_token().expose_secret(), "new-token");
        assert_eq!(
            token.refresh_token().expect("refresh token should exist").expose_secret(),
            "new-refresh"
        );
        assert_eq!(provider.refresh_requests(), 1);
    }

    #[tokio::test]
    async fn startup_validation_only_processes_current_client_tokens() {
        let client_id = ClientId::new("client-1");
        let other_client = ClientId::new("client-2");
        let store = Arc::new(InMemoryTokenStore::new());
        let provider = Arc::new(StaticTokenProvider::new());
        let now = OffsetDateTime::now_utc();
        let key = TokenKey::app(client_id.clone());

        store
            .put(
                key.clone(),
                StoredToken::app(
                    AccessToken::new("app-1"),
                    client_id.clone(),
                    Some(now + time::Duration::minutes(30)),
                    Vec::new(),
                ),
            )
            .await
            .expect("seed token should store");
        store
            .put(
                TokenKey::app(other_client.clone()),
                StoredToken::app(
                    AccessToken::new("app-2"),
                    other_client,
                    Some(now + time::Duration::minutes(30)),
                    Vec::new(),
                ),
            )
            .await
            .expect("seed token should store");
        provider.push_validation_response(
            key.clone(),
            Ok(TokenValidationStatus::Valid(TokenValidation {
                client_id: client_id.clone(),
                user_id: None,
                login: None,
                scopes: Vec::new(),
                expires_in: Duration::from_secs(3600),
            })),
        );

        let manager = manager_with_clock(
            client_id,
            store,
            provider.clone(),
            now,
            ValidationPolicy {
                validate_on_startup: true,
                revalidate_every: Duration::from_secs(3600),
            },
        );

        let handle = manager
            .start_validation_task(CancellationToken::new())
            .await
            .expect("startup validation should succeed");
        handle.abort();

        assert_eq!(provider.validation_requests(), 1);
    }

    #[tokio::test]
    async fn validation_failures_keep_tokens_and_emit_events() {
        let client_id = ClientId::new("client-1");
        let store = Arc::new(InMemoryTokenStore::new());
        let provider = Arc::new(StaticTokenProvider::new());
        let now = OffsetDateTime::now_utc();
        let key = TokenKey::app(client_id.clone());

        store
            .put(
                key.clone(),
                StoredToken::app(
                    AccessToken::new("app-token"),
                    client_id.clone(),
                    Some(now + time::Duration::minutes(30)),
                    Vec::new(),
                ),
            )
            .await
            .expect("seed token should store");
        provider.push_validation_response(
            key.clone(),
            Err(AuthError::validation(key.clone(), "temporary validate outage")),
        );

        let manager = manager_with_clock(
            client_id.clone(),
            store.clone(),
            provider,
            now,
            ValidationPolicy::default(),
        );
        let mut events = manager.subscribe();

        let error = manager.validate(&key).await.expect_err("validation should fail");

        assert!(matches!(error, AuthError::Validation { .. }));
        assert!(
            store.get(&TokenKey::app(client_id)).await.expect("store get should succeed").is_some()
        );

        let event = events.recv().await.expect("event should be emitted");
        assert!(matches!(event, TokenLifecycleEvent::ValidationFailed { .. }));
    }
}
