//! Helix client errors.

use rustitch_auth::AuthError;
use rustitch_core::CoreError;
use thiserror::Error;

/// Errors produced by the Helix client layer.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum HelixError {
    /// Helix client configuration is invalid.
    #[error("helix configuration error: {0}")]
    Configuration(String),

    /// Request construction or transport failed before an API payload was decoded.
    #[error("helix request error: {0}")]
    Request(String),

    /// Twitch returned an API error response.
    #[error("{}", render_api_error(*status, error.as_deref(), message.as_deref()))]
    Api {
        /// HTTP status code returned by Twitch.
        status: u16,
        /// Twitch `error` field, when one was present.
        error: Option<String>,
        /// Twitch `message` field, when one was present.
        message: Option<String>,
    },

    /// A success response could not be decoded into typed models.
    #[error("helix decode error: {0}")]
    Decode(String),

    /// Authentication or token acquisition failed.
    #[error(transparent)]
    Auth(#[from] AuthError),

    /// A shared core invariant failed.
    #[error(transparent)]
    Core(#[from] CoreError),
}

impl HelixError {
    /// Creates a request error with the provided message.
    #[must_use]
    pub fn request(message: impl Into<String>) -> Self {
        Self::Request(message.into())
    }

    /// Creates a decode error with the provided message.
    #[must_use]
    pub fn decode(message: impl Into<String>) -> Self {
        Self::Decode(message.into())
    }

    /// Creates an API error with the provided fields.
    #[must_use]
    pub fn api(status: u16, error: Option<String>, message: Option<String>) -> Self {
        Self::Api { status, error, message }
    }
}

fn render_api_error(status: u16, error: Option<&str>, message: Option<&str>) -> String {
    match (error, message) {
        (Some(error), Some(message)) => format!("helix API error ({status} {error}): {message}"),
        (None, Some(message)) => format!("helix API error ({status}): {message}"),
        (Some(error), None) => format!("helix API error ({status}): {error}"),
        (None, None) => format!("helix API error ({status})"),
    }
}
