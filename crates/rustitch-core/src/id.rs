//! Strongly typed Twitch identifiers.
//!
//! `BroadcasterId` and `UserId` intentionally remain distinct even though
//! Twitch uses the same identifier class underneath. The separation prevents
//! accidental API misuse at compile time.

use serde::{Deserialize, Serialize};
use std::fmt;

macro_rules! define_id {
    ($(#[$meta:meta])* $name:ident) => {
        $(#[$meta])*
        #[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
        pub struct $name(String);

        impl $name {
            /// Creates a new identifier from an owned or borrowed string.
            #[must_use]
            pub fn new(value: impl Into<String>) -> Self {
                Self(value.into())
            }

            /// Returns the raw identifier string.
            #[must_use]
            pub fn as_str(&self) -> &str {
                &self.0
            }

            /// Consumes the wrapper and returns the raw identifier string.
            #[must_use]
            pub fn into_inner(self) -> String {
                self.0
            }
        }

        impl AsRef<str> for $name {
            fn as_ref(&self) -> &str {
                self.as_str()
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str(&self.0)
            }
        }
    };
}

define_id!(
    /// Twitch user identifier.
    UserId
);

define_id!(
    /// Twitch broadcaster identifier.
    BroadcasterId
);

define_id!(
    /// Twitch application client identifier.
    ClientId
);

define_id!(
    /// EventSub subscription identifier.
    SubscriptionId
);

define_id!(
    /// EventSub message identifier.
    MessageId
);

define_id!(
    /// Twitch chat channel identifier.
    ChannelId
);

define_id!(
    /// EventSub WebSocket session identifier.
    SessionId
);

impl From<UserId> for BroadcasterId {
    fn from(value: UserId) -> Self {
        Self(value.into_inner())
    }
}

impl From<BroadcasterId> for UserId {
    fn from(value: BroadcasterId) -> Self {
        Self(value.into_inner())
    }
}
