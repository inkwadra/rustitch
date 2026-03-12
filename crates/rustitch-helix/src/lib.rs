//! Typed Helix client scaffolding for `rustitch`.
//!
//! `rustitch-helix` owns typed endpoint groups, request construction
//! boundaries, response decoding boundaries, and shared client configuration
//! for the Helix API.

pub mod client;
pub mod endpoints;
pub mod error;
mod transport;

pub use client::{
    HelixClient, HelixClientBuilder, HelixClientConfig, HelixRequestAuth, HelixResponse,
};
pub use endpoints::users::{
    GetUsersRequest, GetUsersResponse, HelixBroadcasterType, HelixUser, HelixUserType,
};
pub use error::HelixError;
