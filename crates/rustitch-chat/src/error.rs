//! Chat domain errors.

use rustitch_auth::AuthError;
use rustitch_eventsub::EventSubError;
use rustitch_helix::HelixError;
use thiserror::Error;

/// Errors produced by the transport-agnostic chat layer.
#[derive(Debug, Error)]
pub enum ChatError {
    /// Chat client configuration is invalid.
    #[error("chat configuration error: {0}")]
    Configuration(String),

    /// A send request violates Twitch chat invariants.
    #[error("invalid chat message: {0}")]
    InvalidMessage(String),

    /// Message delivery failed.
    #[error("chat send failed: {0}")]
    SendFailed(String),

    /// Underlying transport failed.
    #[error("chat transport error: {0}")]
    Transport(String),

    /// Authentication failed.
    #[error(transparent)]
    Auth(#[from] AuthError),

    /// Helix API interaction failed.
    #[error(transparent)]
    Helix(#[from] HelixError),

    /// EventSub interaction failed.
    #[error(transparent)]
    EventSub(#[from] EventSubError),
}
