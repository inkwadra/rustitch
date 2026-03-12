//! Authentication and token management errors.

use rustitch_core::CoreError;
use thiserror::Error;

/// Errors produced by the authentication layer.
#[derive(Debug, Error)]
pub enum AuthError {
    /// Authentication client configuration is invalid.
    #[error("authentication configuration error: {0}")]
    Configuration(String),

    /// Token validation failed.
    #[error("token validation failed: {0}")]
    ValidationFailed(String),

    /// Token refresh failed.
    #[error("token refresh failed: {0}")]
    RefreshFailed(String),

    /// Token storage operation failed.
    #[error("token store error: {0}")]
    Store(String),

    /// A requested token could not be found.
    #[error("token not found")]
    NotFound,

    /// A shared core invariant failed.
    #[error(transparent)]
    Core(#[from] CoreError),
}
