#![warn(missing_docs)]
#![warn(unreachable_pub)]
//! Shared domain primitives for the `rustitch` workspace.
//!
//! `rustitch-core` contains transport-agnostic identifiers, configuration,
//! secret wrappers, pagination primitives, rate-limit metadata, and the common
//! error taxonomy used by the higher-level crates.

pub mod config;
pub mod error;
pub mod id;
pub mod pagination;
pub mod rate_limit;
pub mod token;

pub use config::{AuthServiceConfig, HelixConfig, TwitchConfig};
pub use error::CoreError;
pub use id::{BroadcasterId, ChannelId, ClientId, MessageId, SessionId, SubscriptionId, UserId};
pub use pagination::{Cursor, PageInfo, PageRequest};
pub use rate_limit::RateLimitMetadata;
pub use token::{AccessToken, ClientSecret, RefreshToken, WebhookSecret};
