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

    /// Consumes the cursor and returns the raw cursor string.
    #[must_use]
    pub fn into_inner(self) -> String {
        self.0
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

impl PageRequest {
    /// Creates an empty page request.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the cursor after which the next page starts.
    #[must_use]
    pub fn with_after(mut self, after: Cursor) -> Self {
        self.after = Some(after);
        self
    }

    /// Sets the maximum number of items requested.
    #[must_use]
    pub fn with_first(mut self, first: u16) -> Self {
        self.first = Some(first);
        self
    }
}

/// Cursor metadata returned with a page response.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PageInfo {
    /// Cursor to request the next page, if one exists.
    pub next: Option<Cursor>,
}

impl PageInfo {
    /// Returns whether the response indicates another page is available.
    #[must_use]
    pub fn has_next_page(&self) -> bool {
        self.next.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::{Cursor, PageInfo, PageRequest};
    use serde_json::{from_str, to_string};

    #[test]
    fn cursor_exposes_and_consumes_raw_value() {
        let cursor = Cursor::new("opaque-cursor");

        assert_eq!(cursor.as_str(), "opaque-cursor");
        assert_eq!(cursor.clone().into_inner(), "opaque-cursor");
    }

    #[test]
    fn page_request_helpers_build_expected_shape() {
        let request = PageRequest::new().with_after(Cursor::new("next-cursor")).with_first(25);

        assert_eq!(request.after.as_ref().map(Cursor::as_str), Some("next-cursor"));
        assert_eq!(request.first, Some(25));
    }

    #[test]
    fn page_info_reports_when_next_page_exists() {
        let empty = PageInfo::default();
        let next = PageInfo { next: Some(Cursor::new("cursor")) };

        assert!(!empty.has_next_page());
        assert!(next.has_next_page());
    }

    #[test]
    fn pagination_types_round_trip_through_serde() {
        let request = PageRequest::new().with_after(Cursor::new("cursor-1")).with_first(50);
        let info = PageInfo { next: Some(Cursor::new("cursor-2")) };

        let request_json = to_string(&request).expect("page request should serialize");
        let info_json = to_string(&info).expect("page info should serialize");

        assert_eq!(
            from_str::<PageRequest>(&request_json).expect("page request should deserialize"),
            request
        );
        assert_eq!(from_str::<PageInfo>(&info_json).expect("page info should deserialize"), info);
    }
}
