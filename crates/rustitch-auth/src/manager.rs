//! Token orchestration scaffolding.

use crate::token::{TokenProvider, TokenStore};
use std::sync::Arc;
use std::time::Duration;

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

/// Orchestrates token lifecycle over storage, retrieval, validation, and refresh.
#[derive(Clone)]
pub struct TokenManager {
    store: Arc<dyn TokenStore>,
    provider: Arc<dyn TokenProvider>,
    validation_policy: ValidationPolicy,
}

impl TokenManager {
    /// Creates a token manager with the provided store, provider, and policy.
    #[must_use]
    pub fn new(
        store: Arc<dyn TokenStore>,
        provider: Arc<dyn TokenProvider>,
        validation_policy: ValidationPolicy,
    ) -> Self {
        Self { store, provider, validation_policy }
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
}
