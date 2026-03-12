//! Pagination primitives shared by the Twitch API surfaces.

use serde::{Deserialize, Serialize};

/// Opaque pagination cursor returned by Twitch APIs.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Cursor(String);

impl Cursor {
    /// Creates a new pagination cursor.
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Returns the raw cursor string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Cursor-based page request parameters.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PageRequest {
    /// Cursor after which the next page starts.
    pub after: Option<Cursor>,
    /// Maximum number of items requested.
    pub first: Option<u16>,
}

/// Cursor metadata returned with a page response.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PageInfo {
    /// Cursor to request the next page, if one exists.
    pub next: Option<Cursor>,
}
