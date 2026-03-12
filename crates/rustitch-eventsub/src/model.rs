//! Typed EventSub models.

use rustitch_core::{MessageId, SessionId, SubscriptionId};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

/// EventSub transport model.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventSubTransport {
    /// WebSocket transport.
    WebSocket {
        /// Active WebSocket session identifier.
        session_id: SessionId,
    },
    /// Webhook transport.
    Webhook {
        /// Callback URL.
        callback: String,
    },
}

/// Subscription status reported by Twitch.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SubscriptionStatus {
    /// The subscription is enabled.
    Enabled,
    /// Webhook callback verification is pending.
    WebhookCallbackVerificationPending,
    /// Webhook callback verification failed.
    WebhookCallbackVerificationFailed,
    /// Notification delivery failures disabled the subscription.
    NotificationFailuresExceeded,
    /// Authorization for the subscription was revoked.
    AuthorizationRevoked,
    /// The referenced user was removed.
    UserRemoved,
}

/// Typed EventSub subscription descriptor.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Subscription {
    /// Subscription identifier.
    pub id: SubscriptionId,
    /// EventSub type string.
    pub subscription_type: String,
    /// EventSub version string.
    pub version: String,
    /// Current subscription status.
    pub status: SubscriptionStatus,
    /// Subscription transport descriptor.
    pub transport: EventSubTransport,
}

/// Kind of EventSub message being delivered.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventSubMessageType {
    /// Verification challenge message.
    Challenge,
    /// Event notification message.
    Notification,
    /// Subscription revocation message.
    Revocation,
}

/// Metadata attached to an EventSub delivery.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NotificationMetadata {
    /// Unique EventSub message identifier.
    pub message_id: MessageId,
    /// Delivery timestamp.
    pub message_timestamp: OffsetDateTime,
    /// Kind of EventSub message.
    pub message_type: EventSubMessageType,
}

/// Typed EventSub notification scaffold.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EventSubNotification {
    /// EventSub delivery metadata.
    pub metadata: NotificationMetadata,
    /// Subscription that originated the event.
    pub subscription: Subscription,
    /// Raw JSON payload body.
    pub payload: String,
}
