//! Shared configuration types for Twitch API access.

use crate::id::ClientId;
use crate::token::ClientSecret;

/// Base configuration required to interact with Twitch services.
#[derive(Clone, Debug)]
pub struct TwitchConfig {
    /// OAuth application client identifier.
    pub client_id: ClientId,
    /// OAuth application client secret.
    pub client_secret: ClientSecret,
    /// Base URL for the Helix API surface.
    pub api_base_url: String,
    /// Base URL for the OAuth and validation surface.
    pub oauth_base_url: String,
}

impl TwitchConfig {
    /// Returns the canonical Twitch production configuration.
    #[must_use]
    pub fn production(client_id: ClientId, client_secret: ClientSecret) -> Self {
        Self {
            client_id,
            client_secret,
            api_base_url: String::from("https://api.twitch.tv/helix"),
            oauth_base_url: String::from("https://id.twitch.tv/oauth2"),
        }
    }
}
