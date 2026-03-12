//! OAuth flow descriptions and request scaffolding.

use rustitch_core::ClientId;

/// OAuth flows supported by the `rustitch` architecture.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OAuthFlow {
    /// Authorization code flow.
    AuthorizationCode,
    /// Client credentials flow.
    ClientCredentials,
    /// Device code flow.
    DeviceCode,
    /// Refresh flow for an existing token.
    Refresh,
    /// `/validate` flow for startup and periodic validation.
    Validate,
}

/// Authorization request parameters derived from the configured client.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AuthorizationRequest {
    /// Twitch application client identifier.
    pub client_id: ClientId,
    /// Redirect URI for the authorization response.
    pub redirect_uri: Option<String>,
    /// Scopes requested from the user.
    pub scopes: Vec<String>,
}
