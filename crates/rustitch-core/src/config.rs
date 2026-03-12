//! Shared configuration types for Twitch API access.

use crate::id::ClientId;
use crate::token::ClientSecret;

/// Shared configuration for Twitch OAuth and token validation surfaces.
#[derive(Clone, Debug)]
pub struct AuthServiceConfig {
    /// OAuth application client identifier.
    pub client_id: ClientId,
    /// OAuth application client secret.
    pub client_secret: ClientSecret,
    /// Base URL for the OAuth and validation surface.
    pub oauth_base_url: String,
}

impl AuthServiceConfig {
    /// Returns the canonical Twitch production auth configuration.
    #[must_use]
    pub fn production(client_id: ClientId, client_secret: ClientSecret) -> Self {
        Self {
            client_id,
            client_secret,
            oauth_base_url: String::from("https://id.twitch.tv/oauth2"),
        }
    }
}

/// Shared configuration for Twitch Helix API access.
#[derive(Clone, Debug)]
pub struct HelixConfig {
    /// Twitch application client identifier used for `Client-Id` headers.
    pub client_id: ClientId,
    /// Base URL for the Helix API surface.
    pub api_base_url: String,
}

impl HelixConfig {
    /// Returns the canonical Twitch production Helix configuration.
    #[must_use]
    pub fn production(client_id: ClientId) -> Self {
        Self { client_id, api_base_url: String::from("https://api.twitch.tv/helix") }
    }
}

/// Base configuration required to interact with Twitch services.
#[derive(Clone, Debug)]
pub struct TwitchConfig {
    /// Shared auth and token validation configuration.
    pub auth: AuthServiceConfig,
    /// Shared Helix configuration.
    pub helix: HelixConfig,
}

impl TwitchConfig {
    /// Returns the canonical Twitch production configuration.
    #[must_use]
    pub fn production(client_id: ClientId, client_secret: ClientSecret) -> Self {
        Self {
            auth: AuthServiceConfig::production(client_id.clone(), client_secret),
            helix: HelixConfig::production(client_id),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{AuthServiceConfig, HelixConfig, TwitchConfig};
    use crate::{ClientId, ClientSecret};

    #[test]
    fn auth_service_config_uses_production_defaults() {
        let config =
            AuthServiceConfig::production(ClientId::new("client-id"), ClientSecret::new("secret"));

        assert_eq!(config.client_id.as_str(), "client-id");
        assert_eq!(config.client_secret.expose_secret(), "secret");
        assert_eq!(config.oauth_base_url, "https://id.twitch.tv/oauth2");
    }

    #[test]
    fn helix_config_uses_production_defaults() {
        let config = HelixConfig::production(ClientId::new("client-id"));

        assert_eq!(config.client_id.as_str(), "client-id");
        assert_eq!(config.api_base_url, "https://api.twitch.tv/helix");
    }

    #[test]
    fn twitch_config_production_builds_service_specific_configs() {
        let config =
            TwitchConfig::production(ClientId::new("client-id"), ClientSecret::new("secret"));

        assert_eq!(config.auth.client_id.as_str(), "client-id");
        assert_eq!(config.auth.client_secret.expose_secret(), "secret");
        assert_eq!(config.auth.oauth_base_url, "https://id.twitch.tv/oauth2");
        assert_eq!(config.helix.client_id.as_str(), "client-id");
        assert_eq!(config.helix.api_base_url, "https://api.twitch.tv/helix");
    }
}
