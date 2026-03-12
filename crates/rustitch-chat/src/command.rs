//! Transport-agnostic chat commands.

use crate::error::ChatError;
use rustitch_core::{BroadcasterId, MessageId, UserId};

/// Explicit token and delivery semantics for `Send Chat Message`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SendChatSemantics {
    /// User-token send semantics.
    UserToken {
        /// User sending the message.
        sender_id: UserId,
    },
    /// App-token send semantics.
    AppToken {
        /// Bot or application-associated sender identity.
        sender_id: UserId,
        /// App-token-only `for_source_only` behavior.
        for_source_only: bool,
    },
}

/// Transport-agnostic chat send request.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SendChatMessage {
    /// Channel receiving the message.
    pub broadcaster_id: BroadcasterId,
    /// Message body.
    pub message: String,
    /// Reply target when replying to an earlier message.
    pub reply_parent_message_id: Option<MessageId>,
    /// Token semantics for the send operation.
    pub semantics: SendChatSemantics,
}

impl SendChatMessage {
    /// Creates and validates a send request.
    pub fn new(
        broadcaster_id: BroadcasterId,
        message: impl Into<String>,
        reply_parent_message_id: Option<MessageId>,
        semantics: SendChatSemantics,
    ) -> Result<Self, ChatError> {
        let message = message.into();

        if message.is_empty() {
            return Err(ChatError::InvalidMessage(String::from("chat messages must not be empty")));
        }

        if message.chars().count() > 500 {
            return Err(ChatError::InvalidMessage(String::from(
                "chat messages must be 500 characters or fewer",
            )));
        }

        Ok(Self { broadcaster_id, message, reply_parent_message_id, semantics })
    }
}
