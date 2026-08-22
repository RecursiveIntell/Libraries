//! Read-side projections for desktop/UI consumers.

use crate::store::StoreError;
use crate::{ActivityStore, LiveHub, ObservationFilter, TransportStats};
use serde::{Deserialize, Serialize};
use stack_observation::ObservationEnvelope;
use std::sync::Arc;

/// Historical timeline projection. It is a snapshot, not a live stream.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineProjection {
    pub events: Vec<ObservationEnvelope>,
    pub live_cursor: u64,
    pub history_complete: bool,
}

/// Collector health projection suitable for a status bar or diagnostics panel.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HealthProjection {
    pub live_cursor: u64,
    pub attempted: u64,
    pub accepted: u64,
    pub dropped: u64,
    pub rejected: u64,
    pub persisted: u64,
    pub storage_failures: u64,
    pub incomplete_history: bool,
}

/// Read-side service that keeps UI consumers away from SQLite internals.
pub struct ProjectionService {
    store: ActivityStore,
    live: Arc<LiveHub>,
}

impl ProjectionService {
    /// Create projections backed by the store and live hub used by a collector.
    pub fn new(store: ActivityStore, live: Arc<LiveHub>) -> Self {
        Self { store, live }
    }

    /// Build a typed historical timeline snapshot.
    pub fn timeline(&self, filter: &ObservationFilter) -> Result<TimelineProjection, StoreError> {
        Ok(TimelineProjection {
            events: self.store.query_observations(filter)?,
            live_cursor: self.live.current_cursor(),
            // A bounded non-blocking collector cannot prove global completeness.
            history_complete: false,
        })
    }

    /// Build a privacy-aware JSONL export for the desktop download action.
    pub fn export_jsonl(&self) -> Result<String, StoreError> {
        let mut output = Vec::new();
        self.store.export_observations_jsonl_to(&mut output)?;
        String::from_utf8(output).map_err(|error| {
            StoreError::Io(std::io::Error::new(std::io::ErrorKind::InvalidData, error))
        })
    }

    /// Build a health projection from collector counters.
    pub fn health(&self, stats: TransportStats) -> HealthProjection {
        HealthProjection {
            live_cursor: self.live.current_cursor(),
            attempted: stats.attempted,
            accepted: stats.accepted,
            dropped: stats.dropped,
            rejected: stats.rejected,
            persisted: stats.persisted,
            storage_failures: stats.storage_failures,
            incomplete_history: stats.dropped > 0 || stats.storage_failures > 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::start_collector_with_live;
    use stack_observation::{LifecycleStatus, ObservationEnvelope, ObservationKind};

    #[test]
    fn timeline_projection_exposes_cursor_and_incomplete_state() {
        let store = ActivityStore::open(":memory:").unwrap();
        let live = Arc::new(LiveHub::new(4));
        let (client, collector) =
            start_collector_with_live(store.clone(), 8, Some(Arc::clone(&live)));
        let event = ObservationEnvelope::metadata(
            "projection-test",
            "llm-pipeline",
            "projection-adapter",
            1,
            ObservationKind::LlmCall,
            LifecycleStatus::Completed,
            "projection",
        );
        client.try_emit(event).unwrap();
        let stats = collector.shutdown();
        let service = ProjectionService::new(store, live);
        let projection = service.timeline(&ObservationFilter::default()).unwrap();
        assert_eq!(projection.events.len(), 1);
        assert_eq!(projection.live_cursor, 1);
        assert!(!projection.history_complete);
        assert_eq!(service.health(stats).persisted, 1);
    }
}
