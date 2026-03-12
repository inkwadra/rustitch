//! OAuth lifecycle scaffolding for `rustitch`.
//!
//! `rustitch-auth` owns OAuth-oriented contracts and orchestration:
//! authorization flow descriptions, token storage and retrieval boundaries,
//! validation policy, and token manager scaffolding.

pub mod client;
pub mod error;
pub mod flow;
pub mod manager;
pub mod token;

pub use client::{AuthClient, AuthClientBuilder};
pub use error::AuthError;
pub use flow::{AuthorizationRequest, OAuthFlow};
pub use manager::{TokenManager, ValidationPolicy};
pub use token::{
    AppAccessToken, BoxFuture, StoredToken, TokenKey, TokenKind, TokenProvider, TokenStore,
    UserAccessToken,
};
