//! `chat` endpoint group scaffolding.

use crate::client::HelixClient;
use rustitch_core::{BroadcasterId, MessageId, UserId};

/// Typed access to the `chat` Helix endpoints.
#[derive(Clone, Copy, Debug)]
pub struct ChatApi<'a> {
    client: &'a HelixClient,
}

impl<'a> ChatApi<'a> {
    pub(crate) fn new(client: &'a HelixClient) -> Self {
        Self { client }
    }

    /// Returns the owning Helix client.
    #[must_use]
    pub fn client(&self) -> &HelixClient {
        self.client
    }
}

/// Request scaffold for `Send Chat Message`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SendChatMessageRequest {
    /// Channel whose chat room receives the message.
    pub broadcaster_id: BroadcasterId,
    /// User identity associated with the send operation.
    pub sender_id: UserId,
    /// Message body to send.
    pub message: String,
    /// Parent message identifier when the send is a reply.
    pub reply_parent_message_id: Option<MessageId>,
    /// App-token-only `for_source_only` semantic.
    pub for_source_only: Option<bool>,
}

/// Response scaffold for `Send Chat Message`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SendChatMessageResponse {
    /// Identifier of the accepted message when Twitch returns one.
    pub message_id: Option<MessageId>,
    /// Drop reason when the message was rejected by Twitch.
    pub dropped_reason: Option<String>,
}
