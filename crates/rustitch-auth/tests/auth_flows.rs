//! Integration coverage for `rustitch-auth` OAuth flows and provider behavior.

use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use rustitch_auth::{
    AuthClient, DeviceTokenPoll, InMemoryTokenStore, TokenProvider, TokenValidationStatus,
};
use rustitch_core::{ClientId, ClientSecret};
use serde_json::{Value, json};
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::Mutex;

#[derive(Clone, Debug)]
struct MockReply {
    status: StatusCode,
    body: Value,
}

#[derive(Debug, Default)]
struct MockState {
    device_replies: Mutex<VecDeque<MockReply>>,
    token_replies: Mutex<VecDeque<MockReply>>,
    validate_replies: Mutex<VecDeque<MockReply>>,
    bodies: Mutex<Vec<(String, String)>>,
    auth_headers: Mutex<Vec<String>>,
}

#[derive(Clone)]
struct MockServer {
    base_url: String,
    state: Arc<MockState>,
}

impl MockServer {
    async fn start() -> Self {
        let state = Arc::new(MockState::default());
        let router = Router::new()
            .route("/oauth2/device", post(device_handler))
            .route("/oauth2/token", post(token_handler))
            .route("/oauth2/validate", get(validate_handler))
            .with_state(state.clone());

        let listener = TcpListener::bind("127.0.0.1:0").await.expect("listener should bind");
        let address = listener.local_addr().expect("listener should expose local address");

        tokio::spawn(async move {
            axum::serve(listener, router).await.expect("mock oauth server should run");
        });

        Self { base_url: format!("http://{address}/oauth2"), state }
    }

    async fn push_device_reply(&self, status: StatusCode, body: Value) {
        self.state.device_replies.lock().await.push_back(MockReply { status, body });
    }

    async fn push_token_reply(&self, status: StatusCode, body: Value) {
        self.state.token_replies.lock().await.push_back(MockReply { status, body });
    }

    async fn push_validate_reply(&self, status: StatusCode, body: Value) {
        self.state.validate_replies.lock().await.push_back(MockReply { status, body });
    }

    async fn bodies(&self) -> Vec<(String, String)> {
        self.state.bodies.lock().await.clone()
    }

    async fn auth_headers(&self) -> Vec<String> {
        self.state.auth_headers.lock().await.clone()
    }
}

async fn device_handler(State(state): State<Arc<MockState>>, body: String) -> Response {
    state.bodies.lock().await.push((String::from("/oauth2/device"), body));
    pop_reply(&state.device_replies).await
}

async fn token_handler(State(state): State<Arc<MockState>>, body: String) -> Response {
    state.bodies.lock().await.push((String::from("/oauth2/token"), body));
    pop_reply(&state.token_replies).await
}

async fn validate_handler(State(state): State<Arc<MockState>>, headers: HeaderMap) -> Response {
    let header = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .to_owned();
    state.auth_headers.lock().await.push(header);
    pop_reply(&state.validate_replies).await
}

async fn pop_reply(queue: &Mutex<VecDeque<MockReply>>) -> Response {
    let reply = queue.lock().await.pop_front().unwrap_or(MockReply {
        status: StatusCode::INTERNAL_SERVER_ERROR,
        body: json!({"message":"no mock reply queued"}),
    });
    (reply.status, Json(reply.body)).into_response()
}

fn build_auth_client(base_url: &str) -> AuthClient {
    AuthClient::builder()
        .client_id(ClientId::new("client-1"))
        .client_secret(ClientSecret::new("secret-1"))
        .oauth_base_url(base_url)
        .redirect_uri("http://localhost/callback")
        .scope("chat:read")
        .build()
        .expect("auth client should build")
}

#[tokio::test]
async fn exchanges_authorization_code_and_validates_new_token() {
    let server = MockServer::start().await;
    server
        .push_token_reply(
            StatusCode::OK,
            json!({
                "access_token": "user-token",
                "refresh_token": "refresh-token",
                "expires_in": 3600,
                "scope": ["chat:read"],
                "token_type": "bearer"
            }),
        )
        .await;
    server
        .push_validate_reply(
            StatusCode::OK,
            json!({
                "client_id": "client-1",
                "login": "user_login",
                "scopes": ["chat:read"],
                "user_id": "user-1",
                "expires_in": 1800
            }),
        )
        .await;

    let auth = build_auth_client(&server.base_url);
    let token = auth
        .exchange_authorization_code("code-123", None)
        .await
        .expect("authorization code exchange should succeed");

    assert_eq!(token.user_id().as_str(), "user-1");
    assert_eq!(token.access_token().expose_secret(), "user-token");
    assert_eq!(
        token.refresh_token().expect("refresh token should exist").expose_secret(),
        "refresh-token"
    );
    assert_eq!(token.login(), Some("user_login"));

    let bodies = server.bodies().await;
    assert!(bodies.iter().any(|(path, body)| path == "/oauth2/token"
        && body.contains("grant_type=authorization_code")
        && body.contains("code=code-123")));
    assert_eq!(server.auth_headers().await, vec![String::from("OAuth user-token")]);
}

#[tokio::test]
async fn device_flow_reports_pending_then_authorized() {
    let server = MockServer::start().await;
    server
        .push_device_reply(
            StatusCode::OK,
            json!({
                "device_code": "device-code-1",
                "user_code": "ABCDEFGH",
                "verification_uri": "https://www.twitch.tv/activate",
                "expires_in": 1800,
                "interval": 5
            }),
        )
        .await;
    server
        .push_token_reply(
            StatusCode::BAD_REQUEST,
            json!({
                "status": 400,
                "message": "authorization_pending"
            }),
        )
        .await;
    server
        .push_token_reply(
            StatusCode::OK,
            json!({
                "access_token": "device-user-token",
                "refresh_token": "device-refresh-token",
                "expires_in": 3600,
                "scope": ["chat:read"],
                "token_type": "bearer"
            }),
        )
        .await;
    server
        .push_validate_reply(
            StatusCode::OK,
            json!({
                "client_id": "client-1",
                "login": "device_user",
                "scopes": ["chat:read"],
                "user_id": "user-22",
                "expires_in": 1800
            }),
        )
        .await;

    let auth = build_auth_client(&server.base_url);
    let device =
        auth.start_device_authorization().await.expect("device authorization should start");

    let first_poll = auth.poll_device_token(&device).await.expect("first poll should succeed");
    assert!(matches!(first_poll, DeviceTokenPoll::Pending { .. }));

    let second_poll = auth.poll_device_token(&device).await.expect("second poll should succeed");
    match second_poll {
        DeviceTokenPoll::Authorized(token) => {
            assert_eq!(token.user_id().as_str(), "user-22");
            assert_eq!(token.access_token().expose_secret(), "device-user-token");
        }
        other => panic!("expected authorized token, got {other}"),
    }
}

#[tokio::test]
async fn twitch_provider_handles_client_credentials_validation_and_refresh() {
    let server = MockServer::start().await;
    server
        .push_token_reply(
            StatusCode::OK,
            json!({
                "access_token": "app-token",
                "expires_in": 3600,
                "scope": ["chat:read"],
                "token_type": "bearer"
            }),
        )
        .await;
    server
        .push_validate_reply(
            StatusCode::OK,
            json!({
                "client_id": "client-1",
                "scopes": ["chat:read"],
                "expires_in": 1200
            }),
        )
        .await;
    server
        .push_token_reply(
            StatusCode::OK,
            json!({
                "access_token": "refreshed-user-token",
                "refresh_token": "refreshed-refresh-token",
                "expires_in": 3600,
                "scope": ["chat:read"],
                "token_type": "bearer"
            }),
        )
        .await;
    server
        .push_validate_reply(
            StatusCode::OK,
            json!({
                "client_id": "client-1",
                "login": "refresh_user",
                "scopes": ["chat:read"],
                "user_id": "user-1",
                "expires_in": 1200
            }),
        )
        .await;

    let auth = build_auth_client(&server.base_url);
    let provider = auth.twitch_provider();

    let app_token = provider
        .app_token(&ClientId::new("client-1"))
        .await
        .expect("client credentials should succeed");
    assert_eq!(app_token.access_token().expose_secret(), "app-token");

    let stored = app_token.clone().into_stored_token();
    let validation = provider.validate_token(&stored).await.expect("validate should succeed");
    assert!(matches!(validation, TokenValidationStatus::Valid(_)));

    let refreshed = provider
        .refresh_user_token(
            &ClientId::new("client-1"),
            &rustitch_core::UserId::new("user-1"),
            &rustitch_core::RefreshToken::new("refresh-token"),
            &[String::from("chat:read")],
        )
        .await
        .expect("refresh should succeed");

    assert_eq!(
        refreshed.refresh_token().expect("refresh token should exist").expose_secret(),
        "refreshed-refresh-token"
    );
    assert_eq!(refreshed.user_id().as_str(), "user-1");
}

#[tokio::test]
async fn builder_allows_injected_http_client() {
    let http_client = reqwest::ClientBuilder::new()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .expect("http client should build");
    let auth = AuthClient::builder()
        .client_id(ClientId::new("client-1"))
        .http_client(http_client)
        .build()
        .expect("auth client should build");

    assert_eq!(auth.client_id().as_str(), "client-1");
    assert_eq!(auth.oauth_base_url(), "https://id.twitch.tv/oauth2");

    let _ = Arc::new(InMemoryTokenStore::new());
}
