//! Rate-limit metadata extracted from Twitch API responses.

use time::OffsetDateTime;

/// Rate-limit information returned by the Helix API.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RateLimitMetadata {
    /// Maximum number of requests allowed in the current window.
    pub limit: u32,
    /// Number of requests remaining in the current window.
    pub remaining: u32,
    /// Timestamp when the rate-limit window resets.
    pub reset_at: OffsetDateTime,
}
