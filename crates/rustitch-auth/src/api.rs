//! Internal Twitch OAuth transport helpers.

use crate::error::AuthError;
use crate::flow::{DeviceAuthorization, DeviceTokenPoll};
use crate::token::{AppAccessToken, TokenValidation, TokenValidationStatus, UserAccessToken};
use reqwest::StatusCode;
use rustitch_core::{AccessToken, ClientId, ClientSecret, RefreshToken, UserId};
use serde::Deserialize;
use serde::de::Deserializer;
use std::time::Duration;
use time::OffsetDateTime;

#[derive(Clone, Debug)]
pub(crate) struct TwitchAuthApi {
    client_id: ClientId,
    client_secret: Option<ClientSecret>,
    oauth_base_url: String,
    http_client: reqwest::Client,
}

impl TwitchAuthApi {
    pub(crate) fn new(
        client_id: ClientId,
        client_secret: Option<ClientSecret>,
        oauth_base_url: String,
        http_client: reqwest::Client,
    ) -> Self {
        Self { client_id, client_secret, oauth_base_url, http_client }
    }

    pub(crate) fn client_id(&self) -> &ClientId {
        &self.client_id
    }

    pub(crate) async fn exchange_client_credentials(
        &self,
        scopes: &[String],
    ) -> Result<AppAccessToken, AuthError> {
        let client_secret = self.require_client_secret("client_credentials")?;
        let mut params = vec![
            ("client_id", self.client_id.as_str().to_owned()),
            ("client_secret", client_secret.expose_secret().to_owned()),
            ("grant_type", String::from("client_credentials")),
        ];

        if !scopes.is_empty() {
            params.push(("scope", scopes.join(" ")));
        }

        let response = self.post_form("client_credentials", "/token", &params).await?;
        let token = response.parse_json::<TokenResponseBody>("client_credentials").await?;
        let expires_at = token.expires_at();
        Ok(AppAccessToken::new(
            AccessToken::new(token.access_token),
            self.client_id.clone(),
            expires_at,
            token.scope,
        ))
    }

    pub(crate) async fn exchange_authorization_code(
        &self,
        code: &str,
        redirect_uri: &str,
        pkce_verifier: Option<&str>,
    ) -> Result<UserAccessToken, AuthError> {
        let client_secret = self.require_client_secret("authorization_code")?;
        let mut params = vec![
            ("client_id", self.client_id.as_str().to_owned()),
            ("client_secret", client_secret.expose_secret().to_owned()),
            ("code", code.to_owned()),
            ("grant_type", String::from("authorization_code")),
            ("redirect_uri", redirect_uri.to_owned()),
        ];

        if let Some(pkce_verifier) = pkce_verifier {
            params.push(("code_verifier", pkce_verifier.to_owned()));
        }

        let response = self.post_form("authorization_code", "/token", &params).await?;
        let token = response.parse_json::<TokenResponseBody>("authorization_code").await?;
        self.user_token_from_response("authorization_code", token).await
    }

    pub(crate) async fn exchange_refresh_token(
        &self,
        refresh_token: &RefreshToken,
        scopes: &[String],
    ) -> Result<UserAccessToken, AuthError> {
        let client_secret = self.require_client_secret("refresh_token")?;
        let mut params = vec![
            ("client_id", self.client_id.as_str().to_owned()),
            ("client_secret", client_secret.expose_secret().to_owned()),
            ("refresh_token", refresh_token.expose_secret().to_owned()),
            ("grant_type", String::from("refresh_token")),
        ];

        if !scopes.is_empty() {
            params.push(("scope", scopes.join(" ")));
        }

        let response = self.post_form("refresh_token", "/token", &params).await?;
        let token = response.parse_json::<TokenResponseBody>("refresh_token").await?;
        self.user_token_from_response("refresh_token", token).await
    }

    pub(crate) async fn validate_access_token(
        &self,
        access_token: &AccessToken,
    ) -> Result<TokenValidationStatus, AuthError> {
        let url = self.endpoint("/validate");
        let response = self
            .http_client
            .get(url)
            .header("Authorization", format!("OAuth {}", access_token.expose_secret()))
            .send()
            .await
            .map_err(|error| AuthError::oauth("validate", error.to_string()))?;

        match response.status() {
            StatusCode::OK => {
                let validation = response
                    .json::<ValidateResponseBody>()
                    .await
                    .map_err(|error| AuthError::oauth("validate", error.to_string()))?;
                Ok(TokenValidationStatus::Valid(TokenValidation {
                    client_id: ClientId::new(validation.client_id),
                    user_id: validation.user_id.map(UserId::new),
                    login: validation.login,
                    scopes: validation.scopes.unwrap_or_default(),
                    expires_in: Duration::from_secs(validation.expires_in),
                }))
            }
            StatusCode::UNAUTHORIZED => Ok(TokenValidationStatus::Invalid),
            _ => {
                let status = response.status();
                let message = match response.json::<ErrorBody>().await {
                    Ok(error) => error.render_with_status(status),
                    Err(_) => format!("unexpected HTTP status {}", status),
                };
                Err(AuthError::oauth("validate", message))
            }
        }
    }

    pub(crate) async fn start_device_authorization(
        &self,
        scopes: &[String],
    ) -> Result<DeviceAuthorization, AuthError> {
        if scopes.is_empty() {
            return Err(AuthError::configuration(
                "device authorization requires at least one scope",
            ));
        }

        let params =
            vec![("client_id", self.client_id.as_str().to_owned()), ("scopes", scopes.join(" "))];
        let response = self.post_form("device_authorization", "/device", &params).await?;
        let device = response.parse_json::<DeviceAuthorizationBody>("device_authorization").await?;

        Ok(DeviceAuthorization {
            device_code: device.device_code,
            user_code: device.user_code,
            verification_uri: device.verification_uri,
            verification_uri_complete: device.verification_uri_complete,
            expires_in: Duration::from_secs(device.expires_in),
            interval: Duration::from_secs(device.interval.unwrap_or(5)),
            scopes: scopes.to_vec(),
        })
    }

    pub(crate) async fn poll_device_token(
        &self,
        authorization: &DeviceAuthorization,
    ) -> Result<DeviceTokenPoll, AuthError> {
        let mut params = vec![
            ("client_id", self.client_id.as_str().to_owned()),
            ("device_code", authorization.device_code.clone()),
            ("grant_type", String::from("urn:ietf:params:oauth:grant-type:device_code")),
        ];

        if !authorization.scopes.is_empty() {
            params.push(("scopes", authorization.scopes.join(" ")));
        }

        let response = self.post_form("device_poll", "/token", &params).await?;

        match response.status {
            StatusCode::OK => {
                let token = response.parse_json::<TokenResponseBody>("device_poll").await?;
                self.user_token_from_response("device_poll", token)
                    .await
                    .map(DeviceTokenPoll::Authorized)
            }
            StatusCode::BAD_REQUEST => {
                let error = response.parse_json::<ErrorBody>("device_poll").await?;
                match error.message.as_deref() {
                    Some("authorization_pending") => {
                        Ok(DeviceTokenPoll::Pending { interval: authorization.interval })
                    }
                    Some("slow_down") => Ok(DeviceTokenPoll::SlowDown {
                        interval: authorization.interval + Duration::from_secs(5),
                    }),
                    Some("access_denied") => Ok(DeviceTokenPoll::Denied),
                    Some("expired_token") | Some("invalid device code") => {
                        Ok(DeviceTokenPoll::Expired)
                    }
                    _ => Err(AuthError::oauth(
                        "device_poll",
                        error.render_with_status(StatusCode::BAD_REQUEST),
                    )),
                }
            }
            _ => Err(AuthError::oauth("device_poll", response.error_message().await)),
        }
    }

    fn require_client_secret(&self, operation: &'static str) -> Result<&ClientSecret, AuthError> {
        self.client_secret
            .as_ref()
            .ok_or_else(|| AuthError::oauth(operation, "client_secret is required"))
    }

    fn endpoint(&self, path: &str) -> String {
        format!("{}/{}", self.oauth_base_url.trim_end_matches('/'), path.trim_start_matches('/'))
    }

    async fn post_form(
        &self,
        operation: &'static str,
        path: &str,
        params: &[(&str, String)],
    ) -> Result<HttpResponse, AuthError> {
        let response = self
            .http_client
            .post(self.endpoint(path))
            .form(params)
            .send()
            .await
            .map_err(|error| AuthError::oauth(operation, error.to_string()))?;

        Ok(HttpResponse { status: response.status(), response })
    }

    async fn user_token_from_response(
        &self,
        operation: &'static str,
        token: TokenResponseBody,
    ) -> Result<UserAccessToken, AuthError> {
        let expires_at = token.expires_at();
        let access_token = AccessToken::new(token.access_token);
        let refresh_token = token.refresh_token.map(RefreshToken::new);

        let validation = match self.validate_access_token(&access_token).await? {
            TokenValidationStatus::Valid(validation) => validation,
            TokenValidationStatus::Invalid => {
                return Err(AuthError::oauth(
                    operation,
                    "newly issued user token failed validation",
                ));
            }
        };

        let user_id = validation.user_id.clone().ok_or_else(|| {
            AuthError::oauth(operation, "validated user token did not include a user_id")
        })?;

        let scopes = if token.scope.is_empty() { validation.scopes.clone() } else { token.scope };

        Ok(UserAccessToken::new(
            access_token,
            refresh_token,
            expires_at,
            user_id,
            self.client_id.clone(),
            scopes,
            validation.login,
        ))
    }
}

#[derive(Debug)]
struct HttpResponse {
    status: StatusCode,
    response: reqwest::Response,
}

impl HttpResponse {
    async fn parse_json<T>(self, operation: &'static str) -> Result<T, AuthError>
    where
        T: for<'de> Deserialize<'de>,
    {
        self.response
            .json::<T>()
            .await
            .map_err(|error| AuthError::oauth(operation, error.to_string()))
    }

    async fn error_message(self) -> String {
        match self.response.json::<ErrorBody>().await {
            Ok(error) => error.render_with_status(self.status),
            Err(_) => format!("unexpected HTTP status {}", self.status),
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
struct TokenResponseBody {
    access_token: String,
    #[serde(default)]
    expires_in: Option<u64>,
    #[serde(default)]
    refresh_token: Option<String>,
    #[serde(default, deserialize_with = "deserialize_scopes")]
    scope: Vec<String>,
}

impl TokenResponseBody {
    fn expires_at(&self) -> Option<OffsetDateTime> {
        self.expires_in.map(|expires_in| {
            OffsetDateTime::now_utc() + time::Duration::seconds(expires_in as i64)
        })
    }
}

#[derive(Clone, Debug, Deserialize)]
struct ValidateResponseBody {
    client_id: String,
    #[serde(default)]
    login: Option<String>,
    #[serde(default)]
    scopes: Option<Vec<String>>,
    #[serde(default)]
    user_id: Option<String>,
    expires_in: u64,
}

#[derive(Clone, Debug, Deserialize)]
struct DeviceAuthorizationBody {
    device_code: String,
    user_code: String,
    verification_uri: String,
    #[serde(default)]
    verification_uri_complete: Option<String>,
    expires_in: u64,
    #[serde(default)]
    interval: Option<u64>,
}

#[derive(Clone, Debug, Deserialize)]
struct ErrorBody {
    #[serde(default)]
    error: Option<String>,
    #[serde(default)]
    message: Option<String>,
    #[serde(default)]
    status: Option<u16>,
}

impl ErrorBody {
    fn render_with_status(&self, status: StatusCode) -> String {
        let status = self.status.unwrap_or(status.as_u16());
        match (self.error.as_deref(), self.message.as_deref()) {
            (Some(error), Some(message)) => format!("{status} {error}: {message}"),
            (_, Some(message)) => format!("{status}: {message}"),
            (Some(error), None) => format!("{status}: {error}"),
            (None, None) => format!("unexpected HTTP status {status}"),
        }
    }
}

fn deserialize_scopes<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum ScopeRepr {
        List(Vec<String>),
        Text(String),
        Empty(()),
    }

    match ScopeRepr::deserialize(deserializer)? {
        ScopeRepr::List(scopes) => Ok(scopes),
        ScopeRepr::Text(scopes) => Ok(scopes
            .split_whitespace()
            .filter(|scope| !scope.is_empty())
            .map(String::from)
            .collect()),
        ScopeRepr::Empty(()) => Ok(Vec::new()),
    }
}

#[cfg(test)]
mod tests {
    use super::deserialize_scopes;
    use serde::Deserialize;

    #[derive(Debug, Deserialize)]
    struct ScopeHolder {
        #[serde(deserialize_with = "deserialize_scopes")]
        scope: Vec<String>,
    }

    #[test]
    fn deserialize_scopes_accepts_string_and_arrays() {
        let from_string: ScopeHolder = serde_json::from_str(r#"{"scope":"chat:read chat:edit"}"#)
            .expect("string scope should deserialize");
        let from_array: ScopeHolder =
            serde_json::from_str(r#"{"scope":["chat:read","chat:edit"]}"#)
                .expect("array scope should deserialize");

        assert_eq!(from_string.scope, vec!["chat:read", "chat:edit"]);
        assert_eq!(from_array.scope, vec!["chat:read", "chat:edit"]);
    }
}
