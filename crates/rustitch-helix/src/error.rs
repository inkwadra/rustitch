//! Helix client errors.

use rustitch_auth::AuthError;
use rustitch_core::CoreError;
use thiserror::Error;

/// Errors produced by the Helix client layer.
#[derive(Debug, Error)]
pub enum HelixError {
    /// Helix client configuration is invalid.
    #[error("helix configuration error: {0}")]
    Configuration(String),

    /// Request construction or transport failed.
    #[error("helix request error: {0}")]
    Request(String),

    /// Response decoding failed.
    #[error("helix response error: {0}")]
    Response(String),

    /// Authentication or token acquisition failed.
    #[error(transparent)]
    Auth(#[from] AuthError),

    /// A shared core invariant failed.
    #[error(transparent)]
    Core(#[from] CoreError),
}
