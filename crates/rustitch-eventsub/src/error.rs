//! EventSub errors.

use rustitch_auth::AuthError;
use rustitch_core::CoreError;
use thiserror::Error;

/// Errors produced by the EventSub runtime and optional transports.
#[derive(Debug, Error)]
pub enum EventSubError {
    /// EventSub client configuration is invalid.
    #[error("eventsub configuration error: {0}")]
    Configuration(String),

    /// A duplicate message was detected by replay protection.
    #[error("duplicate message: {0}")]
    DuplicateMessage(String),

    /// The incoming message timestamp is outside the permitted replay window.
    #[error("stale message timestamp")]
    StaleTimestamp,

    /// Webhook signature verification failed.
    #[error("invalid webhook signature")]
    InvalidSignature,

    /// The replay store failed.
    #[error("replay store error: {0}")]
    Replay(String),

    /// Local dispatch failed.
    #[error("event dispatch error: {0}")]
    Dispatch(String),

    /// WebSocket runtime error.
    #[error("websocket error: {0}")]
    WebSocket(String),

    /// Webhook verification or extraction error.
    #[error("webhook error: {0}")]
    Webhook(String),

    /// Helix-backed management error.
    #[error("subscription management error: {0}")]
    Management(String),

    /// Authentication or token acquisition failed.
    #[error(transparent)]
    Auth(#[from] AuthError),

    /// A shared core invariant failed.
    #[error(transparent)]
    Core(#[from] CoreError),
}
