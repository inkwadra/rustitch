//! Public facade crate for `rustitch`.
//!
//! `rustitch` is the main public entry point for the workspace. The default
//! feature set is a convenience preset for the full stack; minimal consumers
//! should disable default features and opt into the required surfaces.

pub use rustitch_core as core;
pub use rustitch_core::ClientSecret;
pub use rustitch_core::{
    AuthServiceConfig, BroadcasterId, ChannelId, ClientId, HelixConfig, MessageId, SessionId,
    SubscriptionId, TwitchConfig, UserId,
};

#[cfg(feature = "auth")]
pub use rustitch_auth as auth;

#[cfg(feature = "helix")]
pub use rustitch_helix as helix;

#[cfg(feature = "eventsub")]
pub use rustitch_eventsub as eventsub;

#[cfg(feature = "chat")]
pub use rustitch_chat as chat;

#[cfg(feature = "chat-irc")]
pub use rustitch_chat_irc as chat_irc;
