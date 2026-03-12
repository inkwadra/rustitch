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

impl UserId {
    /// Converts this user identifier into a broadcaster identifier explicitly.
    #[must_use]
    pub fn to_broadcaster_id(&self) -> BroadcasterId {
        BroadcasterId::new(self.0.clone())
    }
}

impl BroadcasterId {
    /// Converts this broadcaster identifier into a user identifier explicitly.
    #[must_use]
    pub fn to_user_id(&self) -> UserId {
        UserId::new(self.0.clone())
    }
}

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

#[cfg(test)]
mod tests {
    use super::{BroadcasterId, ClientId, UserId};
    use serde_json::{from_str, to_string};

    #[test]
    fn user_and_broadcaster_ids_convert_explicitly_without_mixing_types() {
        let user_id = UserId::new("1234");
        let broadcaster_id = user_id.to_broadcaster_id();

        assert_eq!(broadcaster_id.as_str(), "1234");
        assert_eq!(broadcaster_id.to_user_id(), user_id);
    }

    #[test]
    fn id_wrappers_preserve_display_and_owned_values() {
        let client_id = ClientId::new("client-123");

        assert_eq!(client_id.as_str(), "client-123");
        assert_eq!(client_id.to_string(), "client-123");
        assert_eq!(client_id.clone().into_inner(), "client-123");
    }

    #[test]
    fn broadcaster_and_user_ids_round_trip_through_from_impls() {
        let broadcaster_id = BroadcasterId::new("42");
        let user_id = UserId::from(broadcaster_id.clone());

        assert_eq!(user_id.as_str(), "42");
        assert_eq!(BroadcasterId::from(user_id), broadcaster_id);
    }

    #[test]
    fn ids_serialize_and_deserialize_as_plain_strings() {
        let user_id = UserId::new("5678");
        let json = to_string(&user_id).expect("user id should serialize");

        assert_eq!(json, "\"5678\"");
        assert_eq!(from_str::<UserId>(&json).expect("user id should deserialize"), user_id);
    }
}
