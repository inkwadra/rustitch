//! OAuth lifecycle scaffolding for `rustitch`.
//!
//! `rustitch-auth` owns OAuth-oriented contracts and orchestration:
//! authorization flow descriptions, token storage and retrieval boundaries,
//! validation policy, and token manager scaffolding.

mod api;
pub mod client;
pub mod error;
pub mod flow;
pub mod manager;
pub mod provider;
pub mod store;
pub mod token;

pub use client::{AuthClient, AuthClientBuilder};
pub use error::AuthError;
pub use flow::{
    AuthorizationRequest, DeviceAuthorization, DeviceTokenPoll, OAuthFlow, PkceChallenge,
};
pub use manager::{TokenManager, ValidationPolicy};
pub use provider::{StaticTokenProvider, TwitchTokenProvider};
pub use store::InMemoryTokenStore;
pub use token::{
    AppAccessToken, BoxFuture, StoredToken, TokenKey, TokenKind, TokenLifecycleEvent,
    TokenProvider, TokenStore, TokenValidation, TokenValidationStatus, UserAccessToken,
};
