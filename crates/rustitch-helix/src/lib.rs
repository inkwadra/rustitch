//! Typed Helix client scaffolding for `rustitch`.
//!
//! `rustitch-helix` owns typed endpoint groups, request construction
//! boundaries, response decoding boundaries, and shared client configuration
//! for the Helix API.

pub mod client;
pub mod endpoints;
pub mod error;

pub use client::{HelixClient, HelixClientBuilder, HelixClientConfig};
pub use error::HelixError;
