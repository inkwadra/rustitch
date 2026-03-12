//! EventSub runtime scaffolding for `rustitch`.
//!
//! This crate models the runtime-facing EventSub surface: typed notifications,
//! replay protection, broadcast dispatch, WebSocket runtime configuration,
//! optional webhook verification, and optional Helix-backed management.

pub mod client;
pub mod dispatch;
pub mod error;
#[cfg(feature = "eventsub-manage")]
pub mod manage;
pub mod model;
pub mod replay;
#[cfg(feature = "eventsub-webhook")]
pub mod webhook;
pub mod websocket;

pub use client::{EventSubClient, EventSubClientBuilder, RuntimeTransport};
pub use dispatch::EventDispatcher;
pub use error::EventSubError;
#[cfg(feature = "eventsub-manage")]
pub use manage::{EventSubManagementClient, SubscriptionManagementRequest};
pub use model::{
    EventSubMessageType, EventSubNotification, EventSubTransport, NotificationMetadata,
    Subscription, SubscriptionStatus,
};
pub use replay::{InMemoryReplayStore, ReplayStore, ReplayStoreConfig};
#[cfg(feature = "eventsub-webhook")]
pub use webhook::{VerifiedWebhookMessage, WebhookHeaders, WebhookMessageType, WebhookVerifier};
pub use websocket::{EventSubWebSocketClient, EventSubWebSocketConfig, WebSocketSessionState};
