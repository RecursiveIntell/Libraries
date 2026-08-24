//! Bounded, non-blocking producer transport and collector-owned ingestion.

use stack_observation::{
    install_global_sink, GlobalSinkGuard, ObservationEnvelope, ObservationError, ObservationSink,
    PrivacyPolicy,
};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::mpsc::{sync_channel, Receiver, SyncSender, TrySendError};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;

use crate::store::ActivityStore;
use crate::LiveHub;

/// Result of attempting to enqueue an observation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmitStatus {
    /// The collector queue accepted the event.
    Accepted,
    /// The bounded queue was full; the host workload was not blocked.
    Dropped,
    /// The collector is unavailable or has shut down.
    CollectorUnavailable,
}

/// Common non-blocking observation submission boundary.
pub trait ObservationEmitter: Send + Sync {
    fn emit_observation(&self, event: ObservationEnvelope) -> Result<EmitStatus, ObservationError>;
}

impl ObservationEmitter for MonitorClient {
    fn emit_observation(&self, event: ObservationEnvelope) -> Result<EmitStatus, ObservationError> {
        self.try_emit(event)
    }
}

impl ObservationSink for MonitorClient {
    fn submit(&self, event: ObservationEnvelope) -> bool {
        matches!(self.try_emit(event), Ok(EmitStatus::Accepted))
    }
}

/// Atomically maintained producer/collector counters.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct TransportStats {
    pub attempted: u64,
    pub accepted: u64,
    pub dropped: u64,
    pub rejected: u64,
    pub persisted: u64,
    pub storage_failures: u64,
}

#[derive(Default)]
struct Counters {
    attempted: AtomicU64,
    accepted: AtomicU64,
    dropped: AtomicU64,
    rejected: AtomicU64,
    persisted: AtomicU64,
    storage_failures: AtomicU64,
}

impl Counters {
    fn snapshot(&self) -> TransportStats {
        TransportStats {
            attempted: self.attempted.load(Ordering::Relaxed),
            accepted: self.accepted.load(Ordering::Relaxed),
            dropped: self.dropped.load(Ordering::Relaxed),
            rejected: self.rejected.load(Ordering::Relaxed),
            persisted: self.persisted.load(Ordering::Relaxed),
            storage_failures: self.storage_failures.load(Ordering::Relaxed),
        }
    }
}

/// Cheap cloneable producer handle. `try_emit` never waits for storage.
#[derive(Clone)]
pub struct MonitorClient {
    tx: SyncSender<ObservationEnvelope>,
    counters: Arc<Counters>,
}

impl MonitorClient {
    /// Attempt to enqueue an event without waiting for the collector.
    pub fn try_emit(&self, event: ObservationEnvelope) -> Result<EmitStatus, ObservationError> {
        self.counters.attempted.fetch_add(1, Ordering::Relaxed);
        event.validate()?;
        match self.tx.try_send(event) {
            Ok(()) => {
                self.counters.accepted.fetch_add(1, Ordering::Relaxed);
                Ok(EmitStatus::Accepted)
            }
            Err(TrySendError::Full(_)) => {
                self.counters.dropped.fetch_add(1, Ordering::Relaxed);
                Ok(EmitStatus::Dropped)
            }
            Err(TrySendError::Disconnected(_)) => {
                self.counters.dropped.fetch_add(1, Ordering::Relaxed);
                Ok(EmitStatus::CollectorUnavailable)
            }
        }
    }

    /// Snapshot producer and collector counters.
    pub fn stats(&self) -> TransportStats {
        self.counters.snapshot()
    }
}

/// Join handle for the collector worker.
pub struct CollectorHandle {
    stop: Arc<AtomicBool>,
    join: Option<JoinHandle<()>>,
    counters: Arc<Counters>,
    global_sink: Option<GlobalSinkGuard>,
}

impl CollectorHandle {
    /// Request shutdown and wait for the collector thread to exit.
    pub fn shutdown(mut self) -> TransportStats {
        self.stop.store(true, Ordering::Release);
        if let Some(join) = self.join.take() {
            let _ = join.join();
        }
        let _ = self.global_sink.take();
        self.counters.snapshot()
    }

    /// Snapshot counters without stopping the collector.
    pub fn stats(&self) -> TransportStats {
        self.counters.snapshot()
    }
}

impl Drop for CollectorHandle {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
    }
}

/// Start a collector worker and return its bounded producer client.
pub fn start_collector(store: ActivityStore, capacity: usize) -> (MonitorClient, CollectorHandle) {
    start_collector_with_live(store, capacity, None)
}

/// Start a collector worker with an optional bounded live-event hub.
pub fn start_collector_with_live(
    store: ActivityStore,
    capacity: usize,
    live: Option<Arc<LiveHub>>,
) -> (MonitorClient, CollectorHandle) {
    let capacity = capacity.max(1);
    let (tx, rx) = sync_channel(capacity);
    let counters = Arc::new(Counters::default());
    let worker_counters = Arc::clone(&counters);
    let stop = Arc::new(AtomicBool::new(false));
    let worker_stop = Arc::clone(&stop);

    let join = match thread::Builder::new()
        .name("stack-monitor-collector".to_string())
        .spawn(move || collector_loop(store, rx, worker_stop, worker_counters, live))
    {
        Ok(join) => join,
        Err(error) => panic!("collector thread must start: {error}"),
    };

    let client = MonitorClient {
        tx,
        counters: Arc::clone(&counters),
    };
    let global_sink = Some(install_global_sink(Arc::new(client.clone())));
    (
        client,
        CollectorHandle {
            stop,
            join: Some(join),
            counters,
            global_sink,
        },
    )
}

fn collector_loop(
    store: ActivityStore,
    rx: Receiver<ObservationEnvelope>,
    stop: Arc<AtomicBool>,
    counters: Arc<Counters>,
    live: Option<Arc<LiveHub>>,
) {
    while !stop.load(Ordering::Acquire) {
        match rx.recv_timeout(Duration::from_millis(25)) {
            Ok(event) => persist_event(&store, event, &counters, live.as_ref()),
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {}
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => break,
        };
    }

    // Drain events already accepted before shutdown. This is bounded by the queue.
    while let Ok(event) = rx.try_recv() {
        persist_event(&store, event, &counters, live.as_ref());
    }
}

fn persist_event(
    store: &ActivityStore,
    mut event: ObservationEnvelope,
    counters: &Counters,
    live: Option<&Arc<LiveHub>>,
) {
    event.apply_privacy_policy(&PrivacyPolicy::default());
    match store.record_observation(&event) {
        Ok(true) => {
            counters.persisted.fetch_add(1, Ordering::Relaxed);
            if let Some(live) = live {
                live.publish(event);
            }
        }
        Ok(false) => {}
        Err(_error) => {
            counters.storage_failures.fetch_add(1, Ordering::Relaxed);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use stack_observation::{LifecycleStatus, ObservationKind};
    fn event(sequence: u64) -> ObservationEnvelope {
        ObservationEnvelope::metadata(
            "producer-test",
            "llm-pipeline",
            "transport-test",
            sequence,
            ObservationKind::LlmCall,
            LifecycleStatus::Started,
            "started",
        )
    }

    #[test]
    fn accepted_events_are_persisted_by_collector() {
        let _guard = crate::test_support::global_sink_guard();
        let store = ActivityStore::open(":memory:").unwrap();
        let (client, collector) = start_collector(store.clone(), 8);
        assert_eq!(client.try_emit(event(1)).unwrap(), EmitStatus::Accepted);
        let stats = collector.shutdown();
        assert_eq!(stats.persisted, 1);
        assert_eq!(
            store
                .observation_count_for_producer("producer-test")
                .unwrap(),
            1
        );
    }

    #[test]
    fn start_collector_installs_global_observation_sink() {
        let _guard = crate::test_support::global_sink_guard();
        let store = ActivityStore::open(":memory:").unwrap();
        let (_client, collector) = start_collector(store.clone(), 8);
        let event = stack_observation::ObservationEnvelope::metadata(
            "global-test",
            "llm-pipeline",
            "global-test",
            stack_observation::next_global_sequence(),
            stack_observation::ObservationKind::Health,
            stack_observation::LifecycleStatus::Health,
            "global event",
        );
        assert!(stack_observation::emit_global(event));
        assert_eq!(collector.shutdown().persisted, 1);
    }

    #[test]
    fn duplicate_event_ids_are_idempotent() {
        let _guard = crate::test_support::global_sink_guard();
        let store = ActivityStore::open(":memory:").unwrap();
        let (client, collector) = start_collector(store.clone(), 8);
        let first = event(1);
        let event_id = first.event_id.to_string();
        assert_eq!(
            client.try_emit(first.clone()).unwrap(),
            EmitStatus::Accepted
        );
        assert_eq!(client.try_emit(first).unwrap(), EmitStatus::Accepted);
        let stats = collector.shutdown();
        assert_eq!(stats.persisted, 1);
        assert_eq!(store.observation_count_for_event_id(&event_id).unwrap(), 1);
    }

    #[test]
    fn collector_publishes_durable_events_to_live_subscribers() {
        let _guard = crate::test_support::global_sink_guard();
        let store = ActivityStore::open(":memory:").unwrap();
        let hub = Arc::new(LiveHub::new(4));
        let mut subscription = hub.subscribe();
        let (client, collector) = start_collector_with_live(store, 8, Some(hub));
        assert_eq!(client.try_emit(event(1)).unwrap(), EmitStatus::Accepted);
        assert_eq!(collector.shutdown().persisted, 1);
        let live = subscription.try_recv().unwrap();
        assert_eq!(live.cursor, 1);
        assert_eq!(live.observation.source_crate, "llm-pipeline");
    }

    #[test]
    fn full_queue_drops_without_blocking() {
        let _guard = crate::test_support::global_sink_guard();
        let store = ActivityStore::open(":memory:").unwrap();
        let (client, collector) = start_collector(store, 1);
        let _ = client.try_emit(event(1));
        let _ = client.try_emit(event(2));
        let stats = client.stats();
        assert!(stats.dropped <= 1);
        assert_eq!(stats.attempted, 2);
        collector.shutdown();
    }

    #[test]
    fn invalid_event_is_rejected_before_transport() {
        let _guard = crate::test_support::global_sink_guard();
        let store = ActivityStore::open(":memory:").unwrap();
        let (client, collector) = start_collector(store, 2);
        let mut invalid = event(1);
        invalid.producer_id.clear();
        assert!(client.try_emit(invalid).is_err());
        assert_eq!(client.stats().accepted, 0);
        collector.shutdown();
    }
}
