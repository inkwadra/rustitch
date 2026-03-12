//! Shared Helix transport helpers for typed JSON endpoints.

use crate::client::{HelixClient, HelixRequestAuth, HelixResponse};
use crate::error::HelixError;
use reqwest::{Method, Url, header::HeaderMap};
use rustitch_core::{Cursor, PageInfo, RateLimitMetadata};
use serde::Deserialize;
use serde::de::DeserializeOwned;
use time::OffsetDateTime;

pub(crate) async fn execute_get<T>(
    client: &HelixClient,
    path: &str,
    auth: HelixRequestAuth,
    query: &[(String, String)],
) -> Result<HelixResponse<T>, HelixError>
where
    T: DeserializeOwned,
{
    execute_json(client, Method::GET, path, auth, query).await
}

async fn execute_json<T>(
    client: &HelixClient,
    method: Method,
    path: &str,
    auth: HelixRequestAuth,
    query: &[(String, String)],
) -> Result<HelixResponse<T>, HelixError>
where
    T: DeserializeOwned,
{
    let url = build_url(client.config().base_url.as_str(), path, query)?;
    let bearer_token = resolve_bearer_token(client, auth).await?;

    let response = client
        .http_client()
        .request(method, url)
        .header("Authorization", format!("Bearer {bearer_token}"))
        .header("Client-Id", client.config().client_id.as_str())
        .send()
        .await
        .map_err(|error| HelixError::request(format!("failed to send helix request: {error}")))?;

    let status = response.status();
    let rate_limit = parse_rate_limit(response.headers());
    let body = response.bytes().await.map_err(|error| {
        HelixError::request(format!("failed to read helix response body: {error}"))
    })?;

    if !status.is_success() {
        return Err(parse_api_error(status.as_u16(), &body));
    }

    let payload = serde_json::from_slice::<HelixPayload<T>>(&body).map_err(|error| {
        HelixError::decode(format!("failed to decode helix response body: {error}"))
    })?;

    Ok(HelixResponse::new(
        payload.data,
        payload.pagination.and_then(ResponsePagination::into_page_info),
        rate_limit,
    ))
}

async fn resolve_bearer_token(
    client: &HelixClient,
    auth: HelixRequestAuth,
) -> Result<String, HelixError> {
    match auth {
        HelixRequestAuth::App => {
            Ok(client.token_manager().app_token().await?.access_token().expose_secret().to_owned())
        }
        HelixRequestAuth::User { user_id } => Ok(client
            .token_manager()
            .user_token(&user_id)
            .await?
            .access_token()
            .expose_secret()
            .to_owned()),
    }
}

fn build_url(base_url: &str, path: &str, query: &[(String, String)]) -> Result<Url, HelixError> {
    let mut url =
        Url::parse(&format!("{}/{}", base_url.trim_end_matches('/'), path.trim_start_matches('/')))
            .map_err(|error| {
                HelixError::request(format!("failed to construct helix URL: {error}"))
            })?;

    {
        let mut query_pairs = url.query_pairs_mut();
        for (key, value) in query {
            query_pairs.append_pair(key, value);
        }
    }

    Ok(url)
}

fn parse_api_error(status: u16, body: &[u8]) -> HelixError {
    match serde_json::from_slice::<ApiErrorPayload>(body) {
        Ok(payload) => HelixError::api(status, payload.error, payload.message),
        Err(_) => {
            let body_text = String::from_utf8_lossy(body).trim().to_owned();
            let message = if body_text.is_empty() { None } else { Some(body_text) };
            HelixError::api(status, None, message)
        }
    }
}

fn parse_rate_limit(headers: &HeaderMap) -> Option<RateLimitMetadata> {
    let limit = header_u32(headers, "Ratelimit-Limit")?;
    let remaining = header_u32(headers, "Ratelimit-Remaining")?;
    let reset_at =
        OffsetDateTime::from_unix_timestamp(i64::from(header_u32(headers, "Ratelimit-Reset")?))
            .ok()?;

    Some(RateLimitMetadata { limit, remaining, reset_at })
}

fn header_u32(headers: &HeaderMap, name: &str) -> Option<u32> {
    headers.get(name)?.to_str().ok()?.parse().ok()
}

#[derive(Debug, Deserialize)]
struct HelixPayload<T> {
    data: T,
    #[serde(default)]
    pagination: Option<ResponsePagination>,
}

#[derive(Debug, Deserialize)]
struct ResponsePagination {
    #[serde(default)]
    cursor: Option<String>,
}

impl ResponsePagination {
    fn into_page_info(self) -> Option<PageInfo> {
        self.cursor.map(|cursor| PageInfo { next: Some(Cursor::new(cursor)) })
    }
}

#[derive(Debug, Deserialize)]
struct ApiErrorPayload {
    #[serde(default)]
    error: Option<String>,
    #[serde(default)]
    message: Option<String>,
}
