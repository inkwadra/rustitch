//! Replay protection and duplicate suppression.

use crate::error::EventSubError;
use rustitch_auth::BoxFuture;
use rustitch_core::MessageId;
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Duration;
use time::OffsetDateTime;

/// Replay protection contract.
pub trait ReplayStore: Send + Sync {
    /// Returns `true` when the message was already seen, otherwise inserts it.
    fn seen_or_insert<'a>(
        &'a self,
        message_id: &'a MessageId,
        seen_at: OffsetDateTime,
        ttl: Duration,
    ) -> BoxFuture<'a, Result<bool, EventSubError>>;
}

/// Configuration for the built-in in-memory replay store.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReplayStoreConfig {
    /// Maximum accepted message age.
    pub replay_window: Duration,
    /// Retention for message identifiers in duplicate suppression.
    pub message_id_ttl: Duration,
    /// Maximum number of tracked message identifiers.
    pub max_entries: usize,
}

impl Default for ReplayStoreConfig {
    fn default() -> Self {
        Self {
            replay_window: Duration::from_secs(10 * 60),
            message_id_ttl: Duration::from_secs(15 * 60),
            max_entries: 4_096,
        }
    }
}

/// In-memory replay store used in `v1`.
pub struct InMemoryReplayStore {
    config: ReplayStoreConfig,
    entries: Mutex<HashMap<MessageId, OffsetDateTime>>,
}

impl InMemoryReplayStore {
    /// Creates a replay store with the provided configuration.
    #[must_use]
    pub fn new(config: ReplayStoreConfig) -> Self {
        Self { config, entries: Mutex::new(HashMap::new()) }
    }

    fn evict_expired(
        entries: &mut HashMap<MessageId, OffsetDateTime>,
        oldest_allowed: OffsetDateTime,
    ) {
        entries.retain(|_, seen_at| *seen_at >= oldest_allowed);
    }

    fn enforce_bound(entries: &mut HashMap<MessageId, OffsetDateTime>, max_entries: usize) {
        if entries.len() <= max_entries {
            return;
        }

        let mut ordered_entries: Vec<(MessageId, OffsetDateTime)> =
            entries.iter().map(|(id, seen_at)| (id.clone(), *seen_at)).collect();
        ordered_entries.sort_by_key(|(_, seen_at)| *seen_at);

        let overflow = ordered_entries.len() - max_entries;
        for (message_id, _) in ordered_entries.into_iter().take(overflow) {
            entries.remove(&message_id);
        }
    }

    fn time_duration(duration: Duration) -> Result<time::Duration, EventSubError> {
        let seconds = i64::try_from(duration.as_secs()).map_err(|_| {
            EventSubError::Replay(String::from("duration seconds exceed supported range"))
        })?;
        let nanoseconds = i32::try_from(duration.subsec_nanos()).map_err(|_| {
            EventSubError::Replay(String::from("duration nanos exceed supported range"))
        })?;

        Ok(time::Duration::new(seconds, nanoseconds))
    }

    /// Returns the built-in replay store configuration.
    #[must_use]
    pub fn config(&self) -> ReplayStoreConfig {
        self.config
    }
}

impl Default for InMemoryReplayStore {
    fn default() -> Self {
        Self::new(ReplayStoreConfig::default())
    }
}

impl ReplayStore for InMemoryReplayStore {
    fn seen_or_insert<'a>(
        &'a self,
        message_id: &'a MessageId,
        seen_at: OffsetDateTime,
        ttl: Duration,
    ) -> BoxFuture<'a, Result<bool, EventSubError>> {
        Box::pin(async move {
            let ttl = Self::time_duration(ttl)?;
            let oldest_allowed = seen_at - ttl;

            let mut entries = self.entries.lock().map_err(|_| {
                EventSubError::Replay(String::from("replay store lock was poisoned"))
            })?;

            Self::evict_expired(&mut entries, oldest_allowed);

            if entries.contains_key(message_id) {
                return Ok(true);
            }

            entries.insert(message_id.clone(), seen_at);
            Self::enforce_bound(&mut entries, self.config.max_entries);

            Ok(false)
        })
    }
}
