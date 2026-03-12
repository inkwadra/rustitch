//! Authentication and token management errors.

use crate::token::{TokenKey, TokenKind};
use rustitch_core::CoreError;
use thiserror::Error;

/// Errors produced by the authentication layer.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum AuthError {
    /// Authentication client or manager configuration is invalid.
    #[error("authentication configuration error: {message}")]
    Configuration {
        /// Human-readable failure detail.
        message: String,
    },

    /// An operation against a token provider failed.
    #[error("token provider error during {operation}: {message}")]
    Provider {
        /// Provider operation that failed.
        operation: &'static str,
        /// Human-readable failure detail.
        message: String,
    },

    /// An operation against a token store failed.
    #[error("token store error during {operation}: {message}")]
    Store {
        /// Store operation that failed.
        operation: &'static str,
        /// Human-readable failure detail.
        message: String,
    },

    /// A validation request failed for a token.
    #[error("token validation failed for {key:?}: {message}")]
    Validation {
        /// Token key associated with the validation failure.
        key: TokenKey,
        /// Human-readable failure detail.
        message: String,
    },

    /// A refresh request failed for a token.
    #[error("token refresh failed for {key:?}: {message}")]
    Refresh {
        /// Token key associated with the refresh failure.
        key: TokenKey,
        /// Human-readable failure detail.
        message: String,
    },

    /// A token did not match the expected token kind.
    #[error("token kind mismatch: expected {expected:?}, got {actual:?}")]
    TokenKindMismatch {
        /// Expected token kind.
        expected: TokenKind,
        /// Actual token kind.
        actual: TokenKind,
    },

    /// A stored token violated the auth-layer invariants.
    #[error("invalid stored token for {key:?}: {message}")]
    InvalidStoredToken {
        /// Token key associated with the invalid state.
        key: TokenKey,
        /// Human-readable failure detail.
        message: String,
    },

    /// An OAuth request or response could not be translated into crate-owned types.
    #[error("oauth error during {operation}: {message}")]
    OAuth {
        /// OAuth operation that failed.
        operation: &'static str,
        /// Human-readable failure detail.
        message: String,
    },

    /// A shared core invariant failed.
    #[error(transparent)]
    Core(#[from] CoreError),
}

impl AuthError {
    /// Creates a configuration error with the provided message.
    #[must_use]
    pub fn configuration(message: impl Into<String>) -> Self {
        Self::Configuration { message: message.into() }
    }

    /// Creates a provider error with the provided operation and message.
    #[must_use]
    pub fn provider(operation: &'static str, message: impl Into<String>) -> Self {
        Self::Provider { operation, message: message.into() }
    }

    /// Creates a store error with the provided operation and message.
    #[must_use]
    pub fn store(operation: &'static str, message: impl Into<String>) -> Self {
        Self::Store { operation, message: message.into() }
    }

    /// Creates a validation error tied to a token key.
    #[must_use]
    pub fn validation(key: TokenKey, message: impl Into<String>) -> Self {
        Self::Validation { key, message: message.into() }
    }

    /// Creates a refresh error tied to a token key.
    #[must_use]
    pub fn refresh(key: TokenKey, message: impl Into<String>) -> Self {
        Self::Refresh { key, message: message.into() }
    }

    /// Creates an invalid stored token error tied to a token key.
    #[must_use]
    pub fn invalid_stored_token(key: TokenKey, message: impl Into<String>) -> Self {
        Self::InvalidStoredToken { key, message: message.into() }
    }

    /// Creates an OAuth error with the provided operation and message.
    #[must_use]
    pub fn oauth(operation: &'static str, message: impl Into<String>) -> Self {
        Self::OAuth { operation, message: message.into() }
    }
}

impl Clone for AuthError {
    fn clone(&self) -> Self {
        match self {
            Self::Configuration { message } => Self::Configuration { message: message.clone() },
            Self::Provider { operation, message } => {
                Self::Provider { operation, message: message.clone() }
            }
            Self::Store { operation, message } => {
                Self::Store { operation, message: message.clone() }
            }
            Self::Validation { key, message } => {
                Self::Validation { key: key.clone(), message: message.clone() }
            }
            Self::Refresh { key, message } => {
                Self::Refresh { key: key.clone(), message: message.clone() }
            }
            Self::TokenKindMismatch { expected, actual } => {
                Self::TokenKindMismatch { expected: *expected, actual: *actual }
            }
            Self::InvalidStoredToken { key, message } => {
                Self::InvalidStoredToken { key: key.clone(), message: message.clone() }
            }
            Self::OAuth { operation, message } => {
                Self::OAuth { operation, message: message.clone() }
            }
            Self::Core(error) => Self::Core(match error {
                CoreError::Configuration(message) => CoreError::Configuration(message.clone()),
                CoreError::InvalidIdentifier(message) => {
                    CoreError::InvalidIdentifier(message.clone())
                }
                CoreError::InvalidSecret(message) => CoreError::InvalidSecret(message.clone()),
            }),
        }
    }
}
