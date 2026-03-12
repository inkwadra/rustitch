//! Typed `users` Helix endpoint support.

use crate::client::{HelixClient, HelixRequestAuth, HelixResponse};
use crate::error::HelixError;
use rustitch_core::UserId;
use serde::{Deserialize, Deserializer};
use time::OffsetDateTime;

const MAX_GET_USERS_QUERY_ITEMS: usize = 100;

/// Typed access to the `users` Helix endpoints.
#[derive(Clone, Copy, Debug)]
pub struct UsersApi<'a> {
    client: &'a HelixClient,
}

impl<'a> UsersApi<'a> {
    pub(crate) fn new(client: &'a HelixClient) -> Self {
        Self { client }
    }

    /// Returns the owning Helix client.
    #[must_use]
    pub fn client(&self) -> &HelixClient {
        self.client
    }

    /// Executes the Helix `Get Users` endpoint.
    pub async fn get_users(
        &self,
        request: &GetUsersRequest,
        auth: HelixRequestAuth,
    ) -> Result<GetUsersResponse, HelixError> {
        request.validate_for_auth(&auth)?;
        self.client.execute_get("users", auth, &request.query_pairs()).await
    }
}

/// Typed response for the Helix `Get Users` endpoint.
pub type GetUsersResponse = HelixResponse<Vec<HelixUser>>;

/// Typed request for the Helix `Get Users` endpoint.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GetUsersRequest {
    user_ids: Vec<UserId>,
    logins: Vec<String>,
}

impl GetUsersRequest {
    /// Creates an empty request.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a user identifier to the request.
    pub fn with_user_id(mut self, user_id: UserId) -> Result<Self, HelixError> {
        self.push_user_id(user_id)?;
        Ok(self)
    }

    /// Adds a login name to the request.
    pub fn with_login(mut self, login: impl AsRef<str>) -> Result<Self, HelixError> {
        self.push_login(login)?;
        Ok(self)
    }

    /// Appends a user identifier to the request.
    pub fn push_user_id(&mut self, user_id: UserId) -> Result<(), HelixError> {
        self.ensure_capacity_for(1)?;
        self.user_ids.push(user_id);
        Ok(())
    }

    /// Appends a login name to the request.
    pub fn push_login(&mut self, login: impl AsRef<str>) -> Result<(), HelixError> {
        self.ensure_capacity_for(1)?;

        let login = login.as_ref().trim();
        if login.is_empty() {
            return Err(HelixError::request(
                "get users request login values must not be empty or whitespace",
            ));
        }

        self.logins.push(String::from(login));
        Ok(())
    }

    /// Returns the ordered user IDs in the request.
    #[must_use]
    pub fn user_ids(&self) -> &[UserId] {
        &self.user_ids
    }

    /// Returns the ordered login names in the request.
    #[must_use]
    pub fn logins(&self) -> &[String] {
        &self.logins
    }

    /// Returns whether the request contains no filters.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.user_ids.is_empty() && self.logins.is_empty()
    }

    fn ensure_capacity_for(&self, additional: usize) -> Result<(), HelixError> {
        let next_len = self.user_ids.len() + self.logins.len() + additional;
        if next_len > MAX_GET_USERS_QUERY_ITEMS {
            return Err(HelixError::request(format!(
                "get users request supports at most {MAX_GET_USERS_QUERY_ITEMS} combined id and login filters"
            )));
        }

        Ok(())
    }

    fn validate_for_auth(&self, auth: &HelixRequestAuth) -> Result<(), HelixError> {
        if self.is_empty() && matches!(auth, HelixRequestAuth::App) {
            return Err(HelixError::request(
                "get users request requires at least one id or login when using app authorization",
            ));
        }

        Ok(())
    }

    fn query_pairs(&self) -> Vec<(String, String)> {
        let mut pairs = Vec::with_capacity(self.user_ids.len() + self.logins.len());
        pairs.extend(self.user_ids.iter().map(|user_id| (String::from("id"), user_id.to_string())));
        pairs.extend(self.logins.iter().cloned().map(|login| (String::from("login"), login)));
        pairs
    }
}

/// Typed Helix user record returned by the `Get Users` endpoint.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct HelixUser {
    id: UserId,
    login: String,
    display_name: String,
    #[serde(rename = "type")]
    user_type: HelixUserType,
    broadcaster_type: HelixBroadcasterType,
    description: String,
    profile_image_url: String,
    offline_image_url: String,
    #[serde(default, deserialize_with = "deserialize_optional_string")]
    email: Option<String>,
    #[serde(with = "time::serde::rfc3339")]
    created_at: OffsetDateTime,
}

impl HelixUser {
    /// Returns the Twitch user ID.
    #[must_use]
    pub fn id(&self) -> &UserId {
        &self.id
    }

    /// Returns the Twitch login name.
    #[must_use]
    pub fn login(&self) -> &str {
        &self.login
    }

    /// Returns the display name.
    #[must_use]
    pub fn display_name(&self) -> &str {
        &self.display_name
    }

    /// Returns the user type classification.
    #[must_use]
    pub fn user_type(&self) -> &HelixUserType {
        &self.user_type
    }

    /// Returns the broadcaster type classification.
    #[must_use]
    pub fn broadcaster_type(&self) -> &HelixBroadcasterType {
        &self.broadcaster_type
    }

    /// Returns the channel description.
    #[must_use]
    pub fn description(&self) -> &str {
        &self.description
    }

    /// Returns the profile image URL.
    #[must_use]
    pub fn profile_image_url(&self) -> &str {
        &self.profile_image_url
    }

    /// Returns the offline image URL.
    #[must_use]
    pub fn offline_image_url(&self) -> &str {
        &self.offline_image_url
    }

    /// Returns the verified email address when Twitch included it.
    #[must_use]
    pub fn email(&self) -> Option<&str> {
        self.email.as_deref()
    }

    /// Returns the account creation timestamp.
    #[must_use]
    pub fn created_at(&self) -> OffsetDateTime {
        self.created_at
    }
}

/// Helix `type` field for user records.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HelixUserType {
    /// A normal Twitch user.
    Normal,
    /// A Twitch administrator.
    Admin,
    /// A Twitch global moderator.
    GlobalMod,
    /// A Twitch staff member.
    Staff,
    /// Any value not recognized by the current crate version.
    Other(String),
}

impl<'de> Deserialize<'de> for HelixUserType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = String::deserialize(deserializer)?;
        Ok(match raw.as_str() {
            "" => Self::Normal,
            "admin" => Self::Admin,
            "global_mod" => Self::GlobalMod,
            "staff" => Self::Staff,
            _ => Self::Other(raw),
        })
    }
}

/// Helix `broadcaster_type` field for user records.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HelixBroadcasterType {
    /// A normal broadcaster account.
    Normal,
    /// An affiliate broadcaster.
    Affiliate,
    /// A partner broadcaster.
    Partner,
    /// Any value not recognized by the current crate version.
    Other(String),
}

impl<'de> Deserialize<'de> for HelixBroadcasterType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = String::deserialize(deserializer)?;
        Ok(match raw.as_str() {
            "" => Self::Normal,
            "affiliate" => Self::Affiliate,
            "partner" => Self::Partner,
            _ => Self::Other(raw),
        })
    }
}

fn deserialize_optional_string<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Option::<String>::deserialize(deserializer)?;
    Ok(value.and_then(|value| {
        let trimmed = value.trim();
        if trimmed.is_empty() { None } else { Some(String::from(trimmed)) }
    }))
}

#[cfg(test)]
mod tests {
    use super::GetUsersRequest;
    use crate::HelixRequestAuth;
    use rustitch_core::UserId;

    #[test]
    fn request_preserves_query_order_by_field_type() {
        let request = GetUsersRequest::new()
            .with_user_id(UserId::new("1"))
            .and_then(|request| request.with_user_id(UserId::new("2")))
            .and_then(|request| request.with_login("foo"))
            .and_then(|request| request.with_login("bar"))
            .expect("request should be valid");

        assert_eq!(
            request.query_pairs(),
            vec![
                (String::from("id"), String::from("1")),
                (String::from("id"), String::from("2")),
                (String::from("login"), String::from("foo")),
                (String::from("login"), String::from("bar")),
            ]
        );
    }

    #[test]
    fn request_rejects_empty_login_values() {
        let error = GetUsersRequest::new()
            .with_login("   ")
            .expect_err("whitespace-only logins must be rejected");

        assert!(matches!(error, crate::HelixError::Request(_)));
    }

    #[test]
    fn request_rejects_app_auth_without_filters() {
        let error = GetUsersRequest::new()
            .validate_for_auth(&HelixRequestAuth::App)
            .expect_err("empty app-auth request must be rejected");

        assert!(matches!(error, crate::HelixError::Request(_)));
    }
}
