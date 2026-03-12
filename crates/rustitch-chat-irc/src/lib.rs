//! IRC transport adapter scaffolding for `rustitch-chat`.
//!
//! This crate remains intentionally narrow: Twitch IRC transport setup,
//! Twitch-specific capability and login configuration, and reconnect-oriented
//! adapter scaffolding. It does not expose `irc` crate types through its
//! public API.

pub mod client;
pub mod config;
pub mod error;

pub use client::IrcTransport;
pub use config::{IrcCapability, IrcTransportConfig};
pub use error::IrcError;
