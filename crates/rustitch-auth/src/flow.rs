//! OAuth flow descriptions and request scaffolding.

use crate::error::AuthError;
use reqwest::Url;
use rustitch_core::ClientId;
use std::fmt;
use std::time::Duration;

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

/// PKCE challenge metadata attached to an authorization request.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PkceChallenge {
    /// Challenge method, typically `S256`.
    pub method: String,
    /// Encoded PKCE code challenge.
    pub challenge: String,
}

impl PkceChallenge {
    /// Creates a new PKCE challenge descriptor.
    #[must_use]
    pub fn new(method: impl Into<String>, challenge: impl Into<String>) -> Self {
        Self { method: method.into(), challenge: challenge.into() }
    }
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
    /// Explicit state parameter, when set.
    pub state: Option<String>,
    /// Whether Twitch should force the user to re-authorize.
    pub force_verify: bool,
    /// PKCE challenge, when one is used.
    pub pkce_challenge: Option<PkceChallenge>,
    pub(crate) authorization_endpoint: String,
}

impl AuthorizationRequest {
    /// Sets the state parameter on the request.
    #[must_use]
    pub fn state(mut self, state: impl Into<String>) -> Self {
        self.state = Some(state.into());
        self
    }

    /// Sets the force-verify flag on the request.
    #[must_use]
    pub fn force_verify(mut self, force_verify: bool) -> Self {
        self.force_verify = force_verify;
        self
    }

    /// Sets the PKCE challenge on the request.
    #[must_use]
    pub fn pkce_challenge(mut self, pkce_challenge: PkceChallenge) -> Self {
        self.pkce_challenge = Some(pkce_challenge);
        self
    }

    /// Renders the fully encoded Twitch authorization URL.
    pub fn authorization_url(&self) -> Result<String, AuthError> {
        let redirect_uri = self.redirect_uri.as_ref().ok_or_else(|| {
            AuthError::configuration("authorization requests require a redirect_uri")
        })?;

        let mut url = Url::parse(&self.authorization_endpoint).map_err(|error| {
            AuthError::configuration(format!(
                "invalid authorization endpoint {}: {error}",
                self.authorization_endpoint
            ))
        })?;

        {
            let mut query = url.query_pairs_mut();
            query.append_pair("response_type", "code");
            query.append_pair("client_id", self.client_id.as_str());
            query.append_pair("redirect_uri", redirect_uri);
            query.append_pair("scope", &self.scopes.join(" "));

            if self.force_verify {
                query.append_pair("force_verify", "true");
            }

            if let Some(state) = self.state.as_deref() {
                query.append_pair("state", state);
            }

            if let Some(pkce) = self.pkce_challenge.as_ref() {
                query.append_pair("code_challenge", &pkce.challenge);
                query.append_pair("code_challenge_method", &pkce.method);
            }
        }

        Ok(url.into())
    }
}

/// Device authorization details returned by Twitch.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeviceAuthorization {
    /// Device code used for later polling.
    pub device_code: String,
    /// User code shown to the user.
    pub user_code: String,
    /// Verification URI the user should visit.
    pub verification_uri: String,
    /// Optional complete verification URI.
    pub verification_uri_complete: Option<String>,
    /// Time until the device code expires.
    pub expires_in: Duration,
    /// Minimum poll interval returned by Twitch.
    pub interval: Duration,
    /// Scopes attached to the authorization request.
    pub scopes: Vec<String>,
}

/// Result of polling a device authorization.
#[derive(Clone, Debug)]
pub enum DeviceTokenPoll {
    /// The user has not completed the device authorization yet.
    Pending {
        /// Interval after which another poll should be attempted.
        interval: Duration,
    },
    /// Twitch requested slower polling.
    SlowDown {
        /// New recommended poll interval.
        interval: Duration,
    },
    /// The user denied the device authorization.
    Denied,
    /// The device code expired.
    Expired,
    /// Polling completed and returned an access token.
    Authorized(crate::token::UserAccessToken),
}

impl fmt::Display for DeviceTokenPoll {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Pending { .. } => formatter.write_str("authorization pending"),
            Self::SlowDown { .. } => formatter.write_str("authorization pending; slow down"),
            Self::Denied => formatter.write_str("authorization denied"),
            Self::Expired => formatter.write_str("device code expired"),
            Self::Authorized(_) => formatter.write_str("authorization completed"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{AuthorizationRequest, PkceChallenge};
    use rustitch_core::ClientId;

    #[test]
    fn authorization_request_renders_encoded_url() {
        let url = AuthorizationRequest {
            client_id: ClientId::new("client-1"),
            redirect_uri: Some(String::from("http://localhost/callback")),
            scopes: vec![String::from("chat:read"), String::from("chat:edit")],
            state: Some(String::from("csrf-token")),
            force_verify: true,
            pkce_challenge: Some(PkceChallenge::new("S256", "challenge-value")),
            authorization_endpoint: String::from("https://id.twitch.tv/oauth2/authorize"),
        }
        .authorization_url()
        .expect("authorization url should render");

        assert!(url.contains("response_type=code"));
        assert!(url.contains("client_id=client-1"));
        assert!(url.contains("state=csrf-token"));
        assert!(url.contains("force_verify=true"));
        assert!(url.contains("code_challenge=challenge-value"));
        assert!(url.contains("code_challenge_method=S256"));
        assert!(url.contains("scope=chat%3Aread+chat%3Aedit"));
    }
}
