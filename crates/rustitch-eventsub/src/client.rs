//! High-level EventSub client scaffolding.

use crate::dispatch::EventDispatcher;
use crate::model::EventSubNotification;
use crate::replay::{InMemoryReplayStore, ReplayStore};
use std::sync::Arc;
use tokio::sync::broadcast;
use tokio_util::sync::CancellationToken;

/// Primary EventSub runtime transport.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeTransport {
    /// WebSocket transport for installed and local clients.
    #[default]
    WebSocket,
    /// Webhook transport for server-side integrations.
    Webhook,
}

/// High-level EventSub runtime entry point.
#[derive(Clone)]
pub struct EventSubClient {
    transport: RuntimeTransport,
    replay_store: Arc<dyn ReplayStore>,
    dispatcher: EventDispatcher<EventSubNotification>,
    shutdown: CancellationToken,
}

impl EventSubClient {
    /// Starts building an EventSub client.
    #[must_use]
    pub fn builder() -> EventSubClientBuilder {
        EventSubClientBuilder::default()
    }

    /// Returns the configured runtime transport.
    #[must_use]
    pub fn transport(&self) -> RuntimeTransport {
        self.transport
    }

    /// Returns the configured replay store.
    #[must_use]
    pub fn replay_store(&self) -> Arc<dyn ReplayStore> {
        Arc::clone(&self.replay_store)
    }

    /// Subscribes to runtime notifications from the local dispatcher.
    #[must_use]
    pub fn subscribe(&self) -> broadcast::Receiver<EventSubNotification> {
        self.dispatcher.subscribe()
    }

    /// Returns the graceful shutdown token.
    #[must_use]
    pub fn shutdown_token(&self) -> CancellationToken {
        self.shutdown.clone()
    }
}

/// Builder for [`EventSubClient`].
#[derive(Clone, Default)]
pub struct EventSubClientBuilder {
    transport: RuntimeTransport,
    replay_store: Option<Arc<dyn ReplayStore>>,
    dispatcher_capacity: Option<usize>,
}

impl EventSubClientBuilder {
    /// Selects the primary runtime transport.
    #[must_use]
    pub fn transport(mut self, transport: RuntimeTransport) -> Self {
        self.transport = transport;
        self
    }

    /// Injects a custom replay store implementation.
    #[must_use]
    pub fn replay_store(mut self, replay_store: Arc<dyn ReplayStore>) -> Self {
        self.replay_store = Some(replay_store);
        self
    }

    /// Sets the local broadcast capacity used for typed notifications.
    #[must_use]
    pub fn dispatcher_capacity(mut self, dispatcher_capacity: usize) -> Self {
        self.dispatcher_capacity = Some(dispatcher_capacity);
        self
    }

    /// Builds the EventSub client.
    #[must_use]
    pub fn build(self) -> EventSubClient {
        EventSubClient {
            transport: self.transport,
            replay_store: self
                .replay_store
                .unwrap_or_else(|| Arc::new(InMemoryReplayStore::default())),
            dispatcher: EventDispatcher::new(self.dispatcher_capacity.unwrap_or(64)),
            shutdown: CancellationToken::new(),
        }
    }
}
