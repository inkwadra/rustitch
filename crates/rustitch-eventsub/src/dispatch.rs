//! Local event dispatch primitives for EventSub runtime notifications.

use crate::error::EventSubError;
use tokio::sync::broadcast;

/// Broadcast-based typed event dispatcher.
#[derive(Clone)]
pub struct EventDispatcher<E>
where
    E: Clone,
{
    sender: broadcast::Sender<E>,
}

impl<E> EventDispatcher<E>
where
    E: Clone,
{
    /// Creates a dispatcher with the provided bounded capacity.
    #[must_use]
    pub fn new(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);
        Self { sender }
    }

    /// Dispatches an event to local subscribers.
    pub fn dispatch(&self, event: E) -> Result<usize, EventSubError> {
        self.sender
            .send(event)
            .map_err(|_| EventSubError::Dispatch(String::from("no active subscribers")))
    }

    /// Subscribes to future events.
    #[must_use]
    pub fn subscribe(&self) -> broadcast::Receiver<E> {
        self.sender.subscribe()
    }
}
