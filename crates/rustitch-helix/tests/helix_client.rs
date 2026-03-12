//! Protocol coverage for the Phase 3 Helix client surface.

use axum::extract::State;
use axum::http::{HeaderMap, StatusCode, Uri};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use rustitch_auth::{
    AppAccessToken, InMemoryTokenStore, StaticTokenProvider, TokenManager, TokenProvider,
    UserAccessToken, ValidationPolicy,
};
use rustitch_core::{AccessToken, ClientId, UserId};
use rustitch_helix::{
    GetUsersRequest, HelixBroadcasterType, HelixClient, HelixError, HelixRequestAuth, HelixUserType,
};
use serde_json::{Value, json};
use std::collections::VecDeque;
use std::sync::Arc;
use time::OffsetDateTime;
use tokio::net::TcpListener;
use tokio::sync::Mutex;

#[derive(Clone, Debug)]
struct MockReply {
    status: StatusCode,
    body: Value,
    headers: Vec<(String, String)>,
}

#[derive(Debug, Default)]
struct MockState {
    replies: Mutex<VecDeque<MockReply>>,
    requests: Mutex<Vec<RecordedRequest>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct RecordedRequest {
    query: String,
    authorization: String,
    client_id: String,
}

#[derive(Clone)]
struct MockServer {
    base_url: String,
    state: Arc<MockState>,
}

impl MockServer {
    async fn start() -> Self {
        let state = Arc::new(MockState::default());
        let router =
            Router::new().route("/helix/users", get(users_handler)).with_state(state.clone());

        let listener = TcpListener::bind("127.0.0.1:0").await.expect("listener should bind");
        let address = listener.local_addr().expect("listener should expose local address");

        tokio::spawn(async move {
            axum::serve(listener, router).await.expect("mock helix server should run");
        });

        Self { base_url: format!("http://{address}/helix"), state }
    }

    async fn push_reply(&self, reply: MockReply) {
        self.state.replies.lock().await.push_back(reply);
    }

    async fn requests(&self) -> Vec<RecordedRequest> {
        self.state.requests.lock().await.clone()
    }
}

async fn users_handler(
    State(state): State<Arc<MockState>>,
    headers: HeaderMap,
    uri: Uri,
) -> Response {
    let authorization = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .to_owned();
    let client_id = headers
        .get("client-id")
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .to_owned();

    state.requests.lock().await.push(RecordedRequest {
        query: uri.query().unwrap_or_default().to_owned(),
        authorization,
        client_id,
    });

    let reply = state.replies.lock().await.pop_front().unwrap_or(MockReply {
        status: StatusCode::INTERNAL_SERVER_ERROR,
        body: json!({"message":"no mock reply queued"}),
        headers: Vec::new(),
    });

    let mut response = (reply.status, Json(reply.body)).into_response();
    for (name, value) in reply.headers {
        response.headers_mut().insert(
            axum::http::HeaderName::from_bytes(name.as_bytes())
                .expect("header name should be valid"),
            axum::http::HeaderValue::from_str(&value).expect("header value should be valid"),
        );
    }
    response
}

fn build_token_manager(
    client_id: &ClientId,
    provider: Arc<dyn TokenProvider>,
) -> Arc<TokenManager> {
    Arc::new(TokenManager::new(
        client_id.clone(),
        Arc::new(InMemoryTokenStore::new()),
        provider,
        ValidationPolicy::default(),
    ))
}

fn app_reply() -> AppAccessToken {
    AppAccessToken::new(AccessToken::new("app-token"), ClientId::new("client-1"), None, Vec::new())
}

fn user_reply(user_id: &UserId) -> UserAccessToken {
    UserAccessToken::new(
        AccessToken::new("user-token"),
        None,
        None,
        user_id.clone(),
        ClientId::new("client-1"),
        vec![String::from("user:read:email")],
        Some(String::from("user-login")),
    )
}

fn build_client(server: &MockServer, token_manager: Arc<TokenManager>) -> HelixClient {
    HelixClient::builder()
        .base_url(server.base_url.clone())
        .token_manager(token_manager)
        .build()
        .expect("helix client should build")
}

#[tokio::test]
async fn get_users_injects_headers_query_parameters_and_rate_limit_metadata() {
    let server = MockServer::start().await;
    server
        .push_reply(MockReply {
            status: StatusCode::OK,
            body: json!({
                "data": [
                    {
                        "id": "1",
                        "login": "foo",
                        "display_name": "Foo",
                        "type": "",
                        "broadcaster_type": "affiliate",
                        "description": "desc",
                        "profile_image_url": "https://example.com/profile.png",
                        "offline_image_url": "https://example.com/offline.png",
                        "email": "foo@example.com",
                        "created_at": "2024-01-02T03:04:05Z"
                    }
                ]
            }),
            headers: vec![
                (String::from("Ratelimit-Limit"), String::from("800")),
                (String::from("Ratelimit-Remaining"), String::from("799")),
                (String::from("Ratelimit-Reset"), String::from("1710000000")),
            ],
        })
        .await;

    let client_id = ClientId::new("client-1");
    let provider = Arc::new(StaticTokenProvider::new());
    provider.set_app_token_response(client_id.clone(), Ok(app_reply()));
    let token_manager = build_token_manager(&client_id, provider.clone());
    let helix = build_client(&server, token_manager);

    let request = GetUsersRequest::new()
        .with_user_id(UserId::new("1"))
        .and_then(|request| request.with_user_id(UserId::new("2")))
        .and_then(|request| request.with_login("foo"))
        .and_then(|request| request.with_login("bar"))
        .expect("request should be valid");
    let response = helix
        .users()
        .get_users(&request, HelixRequestAuth::App)
        .await
        .expect("get users should succeed");

    let requests = server.requests().await;
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].query, "id=1&id=2&login=foo&login=bar");
    assert_eq!(requests[0].authorization, "Bearer app-token");
    assert_eq!(requests[0].client_id, "client-1");

    let user = &response.data()[0];
    assert_eq!(user.id().as_str(), "1");
    assert_eq!(user.login(), "foo");
    assert_eq!(user.display_name(), "Foo");
    assert_eq!(user.user_type(), &HelixUserType::Normal);
    assert_eq!(user.broadcaster_type(), &HelixBroadcasterType::Affiliate);
    assert_eq!(user.email(), Some("foo@example.com"));
    assert_eq!(
        user.created_at(),
        OffsetDateTime::parse(
            "2024-01-02T03:04:05Z",
            &time::format_description::well_known::Rfc3339
        )
        .expect("timestamp should parse")
    );

    let rate_limit = response.rate_limit().expect("rate limit metadata should exist");
    assert_eq!(rate_limit.limit, 800);
    assert_eq!(rate_limit.remaining, 799);
    assert_eq!(rate_limit.reset_at.unix_timestamp(), 1_710_000_000);
    assert_eq!(provider.app_token_requests(), 1);
}

#[tokio::test]
async fn get_users_allows_empty_request_for_user_auth_and_uses_user_token() {
    let server = MockServer::start().await;
    server
        .push_reply(MockReply {
            status: StatusCode::OK,
            body: json!({
                "data": [
                    {
                        "id": "user-1",
                        "login": "user-login",
                        "display_name": "User",
                        "type": "",
                        "broadcaster_type": "",
                        "description": "",
                        "profile_image_url": "https://example.com/profile.png",
                        "offline_image_url": "https://example.com/offline.png",
                        "email": "",
                        "created_at": "2024-01-02T03:04:05Z"
                    }
                ]
            }),
            headers: vec![],
        })
        .await;

    let client_id = ClientId::new("client-1");
    let user_id = UserId::new("user-1");
    let provider = Arc::new(StaticTokenProvider::new());
    provider.set_user_token_response(client_id.clone(), user_id.clone(), Ok(user_reply(&user_id)));
    let provider_trait: Arc<dyn TokenProvider> = provider.clone();
    let helix = build_client(&server, build_token_manager(&client_id, provider_trait));

    let response = helix
        .users()
        .get_users(&GetUsersRequest::new(), HelixRequestAuth::User { user_id: user_id.clone() })
        .await
        .expect("empty user-auth request should succeed");

    let requests = server.requests().await;
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].query, "");
    assert_eq!(requests[0].authorization, "Bearer user-token");
    assert_eq!(provider.user_token_requests(), 1);
    assert_eq!(response.data()[0].email(), None);
}

#[tokio::test]
async fn get_users_rejects_invalid_local_requests_without_http_calls() {
    let server = MockServer::start().await;
    let client_id = ClientId::new("client-1");
    let provider = Arc::new(StaticTokenProvider::new());
    provider.set_app_token_response(client_id.clone(), Ok(app_reply()));
    let helix = build_client(&server, build_token_manager(&client_id, provider));

    let empty_error = helix
        .users()
        .get_users(&GetUsersRequest::new(), HelixRequestAuth::App)
        .await
        .expect_err("empty app-auth request should fail");
    assert!(matches!(empty_error, HelixError::Request(_)));

    let mut request = GetUsersRequest::new();
    for index in 0..100 {
        request
            .push_user_id(UserId::new(index.to_string()))
            .expect("request should accept up to 100 filters");
    }
    let limit_error = request
        .push_login("overflow")
        .expect_err("request should reject more than 100 combined filters");
    assert!(matches!(limit_error, HelixError::Request(_)));

    assert!(server.requests().await.is_empty());
}

#[tokio::test]
async fn get_users_maps_twitch_api_errors() {
    let server = MockServer::start().await;
    for status in [StatusCode::BAD_REQUEST, StatusCode::UNAUTHORIZED, StatusCode::TOO_MANY_REQUESTS]
    {
        server
            .push_reply(MockReply {
                status,
                body: json!({
                    "error": status.canonical_reason().unwrap_or("unknown"),
                    "message": format!("status {}", status.as_u16())
                }),
                headers: vec![],
            })
            .await;
    }

    let client_id = ClientId::new("client-1");
    let provider = Arc::new(StaticTokenProvider::new());
    provider.set_app_token_response(client_id.clone(), Ok(app_reply()));
    let helix = build_client(&server, build_token_manager(&client_id, provider));
    let request =
        GetUsersRequest::new().with_user_id(UserId::new("1")).expect("request should be valid");

    for expected_status in [400_u16, 401_u16, 429_u16] {
        let error = helix
            .users()
            .get_users(&request, HelixRequestAuth::App)
            .await
            .expect_err("request should map API error");

        match error {
            HelixError::Api { status, error, message } => {
                assert_eq!(status, expected_status);
                assert!(error.is_some());
                assert_eq!(message, Some(format!("status {expected_status}")));
            }
            other => panic!("expected API error, got {other:?}"),
        }
    }
}

#[tokio::test]
async fn get_users_decodes_unknown_enum_values_and_ignores_invalid_rate_limit_headers() {
    let server = MockServer::start().await;
    server
        .push_reply(MockReply {
            status: StatusCode::OK,
            body: json!({
                "data": [
                    {
                        "id": "1",
                        "login": "mystery",
                        "display_name": "Mystery",
                        "type": "future_type",
                        "broadcaster_type": "future_broadcaster",
                        "description": "",
                        "profile_image_url": "https://example.com/profile.png",
                        "offline_image_url": "https://example.com/offline.png",
                        "created_at": "2024-01-02T03:04:05Z"
                    }
                ]
            }),
            headers: vec![
                (String::from("Ratelimit-Limit"), String::from("oops")),
                (String::from("Ratelimit-Remaining"), String::from("799")),
                (String::from("Ratelimit-Reset"), String::from("1710000000")),
            ],
        })
        .await;

    let client_id = ClientId::new("client-1");
    let provider = Arc::new(StaticTokenProvider::new());
    provider.set_app_token_response(client_id.clone(), Ok(app_reply()));
    let helix = build_client(&server, build_token_manager(&client_id, provider));

    let request = GetUsersRequest::new().with_login("mystery").expect("request should be valid");
    let response = helix
        .users()
        .get_users(&request, HelixRequestAuth::App)
        .await
        .expect("request should decode successfully");

    assert_eq!(response.data()[0].user_type(), &HelixUserType::Other(String::from("future_type")));
    assert_eq!(
        response.data()[0].broadcaster_type(),
        &HelixBroadcasterType::Other(String::from("future_broadcaster"))
    );
    assert!(response.rate_limit().is_none());
}

#[tokio::test]
async fn get_users_maps_success_decode_failures() {
    let server = MockServer::start().await;
    server
        .push_reply(MockReply {
            status: StatusCode::OK,
            body: json!({
                "data": [
                    {
                        "id": "1",
                        "login": "broken",
                        "display_name": "Broken",
                        "type": "",
                        "broadcaster_type": "",
                        "description": "",
                        "profile_image_url": "https://example.com/profile.png",
                        "offline_image_url": "https://example.com/offline.png",
                        "created_at": "not-a-timestamp"
                    }
                ]
            }),
            headers: vec![],
        })
        .await;

    let client_id = ClientId::new("client-1");
    let provider = Arc::new(StaticTokenProvider::new());
    provider.set_app_token_response(client_id.clone(), Ok(app_reply()));
    let helix = build_client(&server, build_token_manager(&client_id, provider));
    let request =
        GetUsersRequest::new().with_user_id(UserId::new("1")).expect("request should be valid");

    let error = helix
        .users()
        .get_users(&request, HelixRequestAuth::App)
        .await
        .expect_err("malformed success payload should fail");
    assert!(matches!(error, HelixError::Decode(_)));
}

#[tokio::test]
async fn get_users_surfaces_token_provider_failures_as_auth_errors() {
    let server = MockServer::start().await;
    let client_id = ClientId::new("client-1");
    let provider = Arc::new(StaticTokenProvider::new());
    let helix = build_client(&server, build_token_manager(&client_id, provider));
    let request =
        GetUsersRequest::new().with_user_id(UserId::new("1")).expect("request should be valid");

    let error = helix
        .users()
        .get_users(&request, HelixRequestAuth::App)
        .await
        .expect_err("token acquisition failures should bubble up");

    assert!(matches!(error, HelixError::Auth(_)));
    assert!(server.requests().await.is_empty());
}
