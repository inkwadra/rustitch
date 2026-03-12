//! Reference token-store implementations.

use crate::error::AuthError;
use crate::token::{BoxFuture, StoredToken, TokenKey, TokenStore};
use rustitch_core::ClientId;
use std::collections::HashMap;
use std::sync::RwLock;

/// In-memory token store for tests, examples, and single-process usage.
#[derive(Debug, Default)]
pub struct InMemoryTokenStore {
    tokens: RwLock<HashMap<TokenKey, StoredToken>>,
}

impl InMemoryTokenStore {
    /// Creates an empty in-memory token store.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

impl TokenStore for InMemoryTokenStore {
    fn get<'a>(
        &'a self,
        key: &'a TokenKey,
    ) -> BoxFuture<'a, Result<Option<StoredToken>, AuthError>> {
        Box::pin(async move {
            let tokens = self
                .tokens
                .read()
                .map_err(|_| AuthError::store("get", "in-memory token store read lock poisoned"))?;
            Ok(tokens.get(key).cloned())
        })
    }

    fn put(&self, key: TokenKey, token: StoredToken) -> BoxFuture<'_, Result<(), AuthError>> {
        Box::pin(async move {
            let mut tokens = self.tokens.write().map_err(|_| {
                AuthError::store("put", "in-memory token store write lock poisoned")
            })?;
            tokens.insert(key, token);
            Ok(())
        })
    }

    fn remove<'a>(&'a self, key: &'a TokenKey) -> BoxFuture<'a, Result<(), AuthError>> {
        Box::pin(async move {
            let mut tokens = self.tokens.write().map_err(|_| {
                AuthError::store("remove", "in-memory token store write lock poisoned")
            })?;
            tokens.remove(key);
            Ok(())
        })
    }

    fn list_for_client<'a>(
        &'a self,
        client_id: &'a ClientId,
    ) -> BoxFuture<'a, Result<Vec<(TokenKey, StoredToken)>, AuthError>> {
        Box::pin(async move {
            let tokens = self.tokens.read().map_err(|_| {
                AuthError::store("list_for_client", "in-memory token store read lock poisoned")
            })?;

            Ok(tokens
                .iter()
                .filter(|(key, _)| key.client_id() == client_id)
                .map(|(key, token)| (key.clone(), token.clone()))
                .collect())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::InMemoryTokenStore;
    use crate::token::{StoredToken, TokenKey, TokenStore};
    use rustitch_core::{AccessToken, ClientId, UserId};

    #[tokio::test]
    async fn list_for_client_filters_tokens_by_client_scope() {
        let store = InMemoryTokenStore::new();
        let client_a = ClientId::new("client-a");
        let client_b = ClientId::new("client-b");

        store
            .put(
                TokenKey::app(client_a.clone()),
                StoredToken::app(AccessToken::new("app-a"), client_a.clone(), None, Vec::new()),
            )
            .await
            .expect("put should succeed");
        store
            .put(
                TokenKey::user(client_b.clone(), UserId::new("user-b")),
                StoredToken::user(
                    AccessToken::new("user-b"),
                    None,
                    client_b.clone(),
                    UserId::new("user-b"),
                    None,
                    Vec::new(),
                ),
            )
            .await
            .expect("put should succeed");

        let tokens = store.list_for_client(&client_a).await.expect("listing should succeed");

        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].0, TokenKey::app(client_a));
    }
}
