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
