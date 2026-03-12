//! Transport-agnostic chat scaffolding for `rustitch`.
//!
//! `rustitch-chat` models normalized chat events and high-level message-send
//! semantics. Reading is centered on EventSub, writing is centered on Helix,
//! and the IRC adapter remains feature-gated compatibility infrastructure.

pub mod client;
pub mod command;
pub mod error;
pub mod event;

pub use client::{ChatClient, ChatClientBuilder, ChatReadTransport, ChatWriteTransport};
pub use command::{SendChatMessage, SendChatSemantics};
pub use error::ChatError;
pub use event::{
    ChatBadge, ChatEmote, ChatEvent, ChatMessage, ChatUser, Clear, Delete, Join, Notice, Part,
    Reconnect, ReplyMessage, RoomState,
};
