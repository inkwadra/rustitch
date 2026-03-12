//! Rate-limit metadata extracted from Twitch API responses.

use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

/// Rate-limit information returned by the Helix API.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RateLimitMetadata {
    /// Maximum number of requests allowed in the current window.
    pub limit: u32,
    /// Number of requests remaining in the current window.
    pub remaining: u32,
    /// Timestamp when the rate-limit window resets.
    pub reset_at: OffsetDateTime,
}

#[cfg(test)]
mod tests {
    use super::RateLimitMetadata;
    use serde_json::{from_str, to_string};
    use time::OffsetDateTime;

    #[test]
    fn rate_limit_metadata_round_trips_through_serde() {
        let metadata =
            RateLimitMetadata { limit: 800, remaining: 799, reset_at: OffsetDateTime::UNIX_EPOCH };
        let json = to_string(&metadata).expect("rate limit metadata should serialize");

        assert_eq!(
            from_str::<RateLimitMetadata>(&json).expect("rate limit metadata should deserialize"),
            metadata
        );
    }
}
