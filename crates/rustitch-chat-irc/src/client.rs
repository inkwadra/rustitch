//! IRC transport scaffolding.

use crate::config::IrcTransportConfig;

/// Thin IRC transport adapter scaffold.
#[derive(Clone, Debug)]
pub struct IrcTransport {
    config: IrcTransportConfig,
}

impl IrcTransport {
    /// Creates a new IRC transport scaffold.
    #[must_use]
    pub fn new(config: IrcTransportConfig) -> Self {
        Self { config }
    }

    /// Returns the transport configuration.
    #[must_use]
    pub fn config(&self) -> &IrcTransportConfig {
        &self.config
    }

    /// Returns the Twitch IRC endpoint hostname.
    #[must_use]
    pub fn server_host(&self) -> &'static str {
        "irc.chat.twitch.tv"
    }
}
