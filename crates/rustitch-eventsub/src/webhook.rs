//! Webhook verification on exact raw request bytes.

use crate::error::EventSubError;
use crate::replay::{InMemoryReplayStore, ReplayStore};
use axum::http::HeaderMap;
use bytes::Bytes;
use hmac::{Hmac, Mac};
use rustitch_core::{MessageId, WebhookSecret};
use sha2::Sha256;
use std::sync::Arc;
use std::time::Duration;
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

type HmacSha256 = Hmac<Sha256>;

/// EventSub webhook message type header.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WebhookMessageType {
    /// Webhook callback verification challenge.
    Challenge,
    /// Event notification delivery.
    Notification,
    /// Subscription revocation.
    Revocation,
}

/// Raw webhook headers required for signature verification.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WebhookHeaders {
    /// EventSub message identifier.
    pub message_id: MessageId,
    /// Raw RFC3339 message timestamp header.
    pub message_timestamp: String,
    /// EventSub message type header.
    pub message_type: WebhookMessageType,
    /// Raw Twitch signature header.
    pub signature: String,
}

impl TryFrom<&HeaderMap> for WebhookHeaders {
    type Error = EventSubError;

    fn try_from(headers: &HeaderMap) -> Result<Self, Self::Error> {
        fn header(headers: &HeaderMap, name: &str) -> Result<String, EventSubError> {
            let value = headers.get(name).ok_or_else(|| {
                EventSubError::Webhook(format!("missing required webhook header `{name}`"))
            })?;

            let value = value.to_str().map_err(|_| {
                EventSubError::Webhook(format!("invalid UTF-8 in webhook header `{name}`"))
            })?;

            Ok(String::from(value))
        }

        let message_type = match header(headers, "Twitch-Eventsub-Message-Type")?.as_str() {
            "webhook_callback_verification" => WebhookMessageType::Challenge,
            "notification" => WebhookMessageType::Notification,
            "revocation" => WebhookMessageType::Revocation,
            other => {
                return Err(EventSubError::Webhook(format!(
                    "unsupported webhook message type `{other}`",
                )));
            }
        };

        Ok(Self {
            message_id: MessageId::new(header(headers, "Twitch-Eventsub-Message-Id")?),
            message_timestamp: header(headers, "Twitch-Eventsub-Message-Timestamp")?,
            message_type,
            signature: header(headers, "Twitch-Eventsub-Message-Signature")?,
        })
    }
}

/// Result of successful webhook verification.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VerifiedWebhookMessage {
    /// EventSub message identifier.
    pub message_id: MessageId,
    /// Parsed RFC3339 timestamp.
    pub message_timestamp: OffsetDateTime,
    /// EventSub webhook message type.
    pub message_type: WebhookMessageType,
    /// Exact raw request body.
    pub raw_body: Bytes,
}

/// Verifies webhook signatures, timestamp freshness, and replay constraints.
#[derive(Clone)]
pub struct WebhookVerifier {
    secret: WebhookSecret,
    replay_store: Arc<dyn ReplayStore>,
    replay_window: Duration,
    message_id_ttl: Duration,
}

impl WebhookVerifier {
    /// Creates a webhook verifier with default replay behavior.
    #[must_use]
    pub fn new(secret: WebhookSecret) -> Self {
        Self {
            secret,
            replay_store: Arc::new(InMemoryReplayStore::default()),
            replay_window: Duration::from_secs(10 * 60),
            message_id_ttl: Duration::from_secs(15 * 60),
        }
    }

    /// Injects a custom replay store.
    #[must_use]
    pub fn replay_store(mut self, replay_store: Arc<dyn ReplayStore>) -> Self {
        self.replay_store = replay_store;
        self
    }

    /// Overrides the permitted replay window.
    #[must_use]
    pub fn replay_window(mut self, replay_window: Duration) -> Self {
        self.replay_window = replay_window;
        self
    }

    /// Overrides message identifier retention for duplicate suppression.
    #[must_use]
    pub fn message_id_ttl(mut self, message_id_ttl: Duration) -> Self {
        self.message_id_ttl = message_id_ttl;
        self
    }

    /// Verifies the webhook request on the exact raw body bytes.
    pub async fn verify(
        &self,
        headers: &WebhookHeaders,
        raw_body: Bytes,
    ) -> Result<VerifiedWebhookMessage, EventSubError> {
        let parsed_timestamp = OffsetDateTime::parse(&headers.message_timestamp, &Rfc3339)
            .map_err(|_| {
                EventSubError::Webhook(String::from("webhook timestamp is not valid RFC3339"))
            })?;

        let now = OffsetDateTime::now_utc();
        if parsed_timestamp < now - Self::time_duration(self.replay_window)? {
            return Err(EventSubError::StaleTimestamp);
        }

        self.verify_signature(headers, &raw_body)?;

        if self
            .replay_store
            .seen_or_insert(&headers.message_id, parsed_timestamp, self.message_id_ttl)
            .await?
        {
            return Err(EventSubError::DuplicateMessage(headers.message_id.to_string()));
        }

        Ok(VerifiedWebhookMessage {
            message_id: headers.message_id.clone(),
            message_timestamp: parsed_timestamp,
            message_type: headers.message_type,
            raw_body,
        })
    }

    fn verify_signature(
        &self,
        headers: &WebhookHeaders,
        raw_body: &Bytes,
    ) -> Result<(), EventSubError> {
        let mut mac = HmacSha256::new_from_slice(self.secret.expose_secret().as_bytes())
            .map_err(|_| EventSubError::Webhook(String::from("invalid webhook secret")))?;
        mac.update(headers.message_id.as_str().as_bytes());
        mac.update(headers.message_timestamp.as_bytes());
        mac.update(raw_body.as_ref());

        let provided_signature =
            headers.signature.strip_prefix("sha256=").unwrap_or(headers.signature.as_str());
        let provided_bytes = decode_hex(provided_signature)?;

        mac.verify_slice(&provided_bytes).map_err(|_| EventSubError::InvalidSignature)
    }

    fn time_duration(duration: Duration) -> Result<time::Duration, EventSubError> {
        let seconds = i64::try_from(duration.as_secs()).map_err(|_| {
            EventSubError::Webhook(String::from("duration seconds exceed supported range"))
        })?;
        let nanoseconds = i32::try_from(duration.subsec_nanos()).map_err(|_| {
            EventSubError::Webhook(String::from("duration nanos exceed supported range"))
        })?;

        Ok(time::Duration::new(seconds, nanoseconds))
    }
}

fn decode_hex(input: &str) -> Result<Vec<u8>, EventSubError> {
    if input.len() % 2 != 0 {
        return Err(EventSubError::InvalidSignature);
    }

    let mut output = Vec::with_capacity(input.len() / 2);
    for pair in input.as_bytes().chunks_exact(2) {
        let high = decode_hex_nibble(pair[0])?;
        let low = decode_hex_nibble(pair[1])?;
        output.push((high << 4) | low);
    }

    Ok(output)
}

fn decode_hex_nibble(value: u8) -> Result<u8, EventSubError> {
    match value {
        b'0'..=b'9' => Ok(value - b'0'),
        b'a'..=b'f' => Ok(value - b'a' + 10),
        b'A'..=b'F' => Ok(value - b'A' + 10),
        _ => Err(EventSubError::InvalidSignature),
    }
}
