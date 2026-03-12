//! Transport-normalized chat events.

use rustitch_core::{BroadcasterId, MessageId, UserId};

/// Normalized chat event.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ChatEvent {
    /// Chat message event.
    Message(ChatMessage),
    /// Reply message event.
    Reply(ReplyMessage),
    /// Room state update.
    RoomState(RoomState),
    /// Server notice.
    Notice(Notice),
    /// Join event.
    Join(Join),
    /// Part event.
    Part(Part),
    /// Reconnect event.
    Reconnect(Reconnect),
    /// Message deletion event.
    Delete(Delete),
    /// Clear chat event.
    Clear(Clear),
}

/// Chat message received from a transport.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChatMessage {
    /// Unique message identifier.
    pub id: MessageId,
    /// Channel that received the message.
    pub broadcaster_id: BroadcasterId,
    /// User who sent the message.
    pub user: ChatUser,
    /// Message text content.
    pub text: String,
    /// Badges attached to the user in this room.
    pub badges: Vec<ChatBadge>,
    /// Emotes referenced in the message.
    pub emotes: Vec<ChatEmote>,
}

/// Reply message that references an earlier message.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReplyMessage {
    /// Reply message itself.
    pub message: ChatMessage,
    /// Parent message identifier.
    pub parent_message_id: MessageId,
}

/// Room-state change event.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RoomState {
    /// Channel whose room state changed.
    pub broadcaster_id: BroadcasterId,
}

/// Server-generated notice.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Notice {
    /// Notice message text.
    pub message: String,
}

/// User joined a channel.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Join {
    /// Joined channel.
    pub broadcaster_id: BroadcasterId,
    /// User who joined.
    pub user_id: UserId,
}

/// User parted from a channel.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Part {
    /// Channel that was parted.
    pub broadcaster_id: BroadcasterId,
    /// User who parted.
    pub user_id: UserId,
}

/// Server requested a reconnect.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Reconnect;

/// Message deletion event.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Delete {
    /// Deleted message identifier.
    pub message_id: MessageId,
}

/// Chat clear event.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Clear {
    /// Channel that was cleared.
    pub broadcaster_id: BroadcasterId,
}

/// Normalized chat user descriptor.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChatUser {
    /// User identifier.
    pub id: UserId,
    /// Display name.
    pub display_name: String,
}

/// Normalized chat badge.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChatBadge {
    /// Badge set identifier.
    pub set_id: String,
    /// Badge version identifier.
    pub version: String,
}

/// Normalized chat emote span.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChatEmote {
    /// Emote identifier.
    pub id: String,
    /// Start byte offset within the message.
    pub start: usize,
    /// End byte offset within the message.
    pub end: usize,
}
