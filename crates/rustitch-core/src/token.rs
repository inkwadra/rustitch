//! Secret wrapper types shared across the workspace.

use crate::error::CoreError;
use secrecy::{ExposeSecret, SecretString};
use std::fmt;

macro_rules! define_secret_wrapper {
    ($name:ident, $doc:literal) => {
        #[doc = $doc]
        #[derive(Clone)]
        pub struct $name(SecretString);

        impl $name {
            /// Wraps the provided secret value.
            #[must_use]
            pub fn new(value: impl Into<String>) -> Self {
                Self(SecretString::from(value.into()))
            }

            /// Reveals the secret value for explicit use at an I/O boundary.
            #[must_use]
            pub fn expose_secret(&self) -> &str {
                self.0.expose_secret()
            }
        }

        impl fmt::Debug for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str(concat!(stringify!($name), "(**redacted**)"))
            }
        }
    };
}

define_secret_wrapper!(AccessToken, "Wrapped OAuth access token.");
define_secret_wrapper!(RefreshToken, "Wrapped OAuth refresh token.");
define_secret_wrapper!(ClientSecret, "Wrapped Twitch application client secret.");

/// Wrapped EventSub webhook secret.
#[derive(Clone)]
pub struct WebhookSecret(SecretString);

impl WebhookSecret {
    /// Validates and wraps an EventSub webhook secret.
    ///
    /// Twitch requires an ASCII secret whose length is between 10 and 100
    /// characters.
    pub fn new(value: impl Into<String>) -> Result<Self, CoreError> {
        let secret = value.into();

        if !secret.is_ascii() {
            return Err(CoreError::InvalidSecret(String::from(
                "webhook secret must contain only ASCII characters",
            )));
        }

        if !(10..=100).contains(&secret.len()) {
            return Err(CoreError::InvalidSecret(String::from(
                "webhook secret length must be between 10 and 100 characters",
            )));
        }

        Ok(Self(SecretString::from(secret)))
    }

    /// Reveals the secret value for explicit HMAC operations.
    #[must_use]
    pub fn expose_secret(&self) -> &str {
        self.0.expose_secret()
    }
}

impl fmt::Debug for WebhookSecret {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("WebhookSecret(**redacted**)")
    }
}

#[cfg(test)]
mod tests {
    use super::{AccessToken, ClientSecret, RefreshToken, WebhookSecret};
    use crate::CoreError;

    #[test]
    fn debug_redacts_all_secret_wrappers() {
        let access_token = AccessToken::new("access-token");
        let refresh_token = RefreshToken::new("refresh-token");
        let client_secret = ClientSecret::new("client-secret");
        let webhook_secret = WebhookSecret::new("0123456789").expect("secret should be valid");

        assert_eq!(format!("{access_token:?}"), "AccessToken(**redacted**)");
        assert_eq!(format!("{refresh_token:?}"), "RefreshToken(**redacted**)");
        assert_eq!(format!("{client_secret:?}"), "ClientSecret(**redacted**)");
        assert_eq!(format!("{webhook_secret:?}"), "WebhookSecret(**redacted**)");
    }

    #[test]
    fn webhook_secret_accepts_boundary_lengths() {
        let min = WebhookSecret::new("0123456789").expect("minimum length secret should be valid");
        let max =
            WebhookSecret::new("a".repeat(100)).expect("maximum length secret should be valid");

        assert_eq!(min.expose_secret(), "0123456789");
        assert_eq!(max.expose_secret(), "a".repeat(100));
    }

    #[test]
    fn webhook_secret_rejects_non_ascii_content() {
        let error = WebhookSecret::new("secrêt-value").expect_err("non-ascii secret should fail");

        assert_eq!(
            error,
            CoreError::InvalidSecret(String::from(
                "webhook secret must contain only ASCII characters",
            ))
        );
    }

    #[test]
    fn webhook_secret_rejects_values_outside_length_bounds() {
        let too_short = WebhookSecret::new("short").expect_err("short secret should fail");
        let too_long = WebhookSecret::new("a".repeat(101)).expect_err("long secret should fail");

        assert_eq!(
            too_short,
            CoreError::InvalidSecret(String::from(
                "webhook secret length must be between 10 and 100 characters",
            ))
        );
        assert_eq!(
            too_long,
            CoreError::InvalidSecret(String::from(
                "webhook secret length must be between 10 and 100 characters",
            ))
        );
    }
}
