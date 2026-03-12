//! IRC transport configuration.

use rustitch_core::{AccessToken, ChannelId, UserId};

/// Twitch IRC capabilities requested during the handshake.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IrcCapability {
    /// IRCv3 tags capability.
    Tags,
    /// Twitch commands capability.
    Commands,
    /// Twitch membership capability.
    Membership,
}

/// IRC transport configuration for Twitch chat compatibility.
#[derive(Clone, Debug)]
pub struct IrcTransportConfig {
    /// User identity used for `NICK`.
    pub user_id: UserId,
    /// OAuth access token used for `PASS`.
    pub access_token: AccessToken,
    /// Channels to join after connecting.
    pub channels: Vec<ChannelId>,
    /// Requested Twitch IRC capabilities.
    pub capabilities: Vec<IrcCapability>,
}
