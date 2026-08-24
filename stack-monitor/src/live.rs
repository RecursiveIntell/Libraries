//! Bounded live observation subscriptions with explicit lag reporting.

use serde::{Deserialize, Serialize};
use stack_observation::ObservationEnvelope;
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::sync::broadcast;

/// A live event with a process-local monotonic cursor.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiveEvent {
    pub cursor: u64,
    pub observation: ObservationEnvelope,
}

/// Receive result for a live subscription.
#[derive(Debug, PartialEq, Eq)]
pub enum LiveReceive {
    /// No event is currently available.
    Empty,
    /// A bounded subscriber fell behind and missed this many events.
    Lagged(u64),
    /// The hub has been closed.
    Closed,
}

/// Bounded fan-out hub. Publishing never waits for subscribers.
pub struct LiveHub {
    sender: broadcast::Sender<LiveEvent>,
    cursor: AtomicU64,
}

impl LiveHub {
    /// Create a hub with a finite replay window.
    pub fn new(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity.max(1));
        Self {
            sender,
            cursor: AtomicU64::new(0),
        }
    }

    /// Subscribe from the current cursor. Events before this point are historical.
    pub fn subscribe(&self) -> LiveSubscription {
        LiveSubscription {
            receiver: self.sender.subscribe(),
            starting_cursor: self.cursor.load(Ordering::Acquire),
        }
    }

    /// Publish without waiting for any subscriber.
    pub fn publish(&self, observation: ObservationEnvelope) -> u64 {
        let cursor = self.cursor.fetch_add(1, Ordering::AcqRel) + 1;
        let _ = self.sender.send(LiveEvent {
            cursor,
            observation,
        });
        cursor
    }

    /// Latest cursor assigned by this hub.
    pub fn current_cursor(&self) -> u64 {
        self.cursor.load(Ordering::Acquire)
    }
}

/// A bounded live subscription.
pub struct LiveSubscription {
    receiver: broadcast::Receiver<LiveEvent>,
    starting_cursor: u64,
}

impl LiveSubscription {
    /// Cursor at the time this subscription was created.
    pub fn starting_cursor(&self) -> u64 {
        self.starting_cursor
    }

    /// Try to receive without blocking.
    pub fn try_recv(&mut self) -> Result<LiveEvent, LiveReceive> {
        match self.receiver.try_recv() {
            Ok(event) => Ok(event),
            Err(broadcast::error::TryRecvError::Empty) => Err(LiveReceive::Empty),
            Err(broadcast::error::TryRecvError::Lagged(count)) => Err(LiveReceive::Lagged(count)),
            Err(broadcast::error::TryRecvError::Closed) => Err(LiveReceive::Closed),
        }
    }

    /// Receive with a bounded timeout.
    pub async fn recv(&mut self) -> Result<LiveEvent, LiveReceive> {
        match self.receiver.recv().await {
            Ok(event) => Ok(event),
            Err(broadcast::error::RecvError::Lagged(count)) => Err(LiveReceive::Lagged(count)),
            Err(broadcast::error::RecvError::Closed) => Err(LiveReceive::Closed),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use stack_observation::{LifecycleStatus, ObservationKind};

    fn event() -> ObservationEnvelope {
        ObservationEnvelope::metadata(
            "live-test",
            "llm-pipeline",
            "live-adapter",
            1,
            ObservationKind::LlmCall,
            LifecycleStatus::Completed,
            "live",
        )
    }

    #[test]
    fn subscription_reports_cursor_and_events() {
        let hub = LiveHub::new(4);
        let mut subscription = hub.subscribe();
        assert_eq!(subscription.starting_cursor(), 0);
        hub.publish(event());
        let received = subscription.try_recv().unwrap();
        assert_eq!(received.cursor, 1);
        assert_eq!(hub.current_cursor(), 1);
    }

    #[test]
    fn slow_subscription_reports_lag_instead_of_blocking() {
        let hub = LiveHub::new(1);
        let mut subscription = hub.subscribe();
        hub.publish(event());
        hub.publish(event());
        assert!(matches!(
            subscription.try_recv(),
            Err(LiveReceive::Lagged(1))
        ));
    }
}
