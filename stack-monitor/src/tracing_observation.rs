//! Non-blocking tracing fallback adapter for collector-backed observations.

use stack_observation::{LifecycleStatus, ObservationEnvelope, ObservationKind, Provenance};
use std::sync::atomic::{AtomicU64, Ordering};
use tracing::field::{Field, Visit};
use tracing::Subscriber;
use tracing_subscriber::layer::Context;
use tracing_subscriber::Layer;

/// A tracing layer that emits bounded, explicitly inferred observations.
pub struct TracingObservationLayer {
    client: crate::MonitorClient,
    producer_id: String,
    prefixes: Vec<String>,
    sequence: AtomicU64,
}

impl TracingObservationLayer {
    /// Create a fallback layer for the supplied target prefixes.
    pub fn new(
        client: crate::MonitorClient,
        producer_id: impl Into<String>,
        prefixes: Vec<String>,
    ) -> Self {
        Self {
            client,
            producer_id: producer_id.into(),
            prefixes,
            sequence: AtomicU64::new(0),
        }
    }

    fn accepts(&self, target: &str) -> bool {
        self.prefixes.is_empty()
            || self
                .prefixes
                .iter()
                .any(|prefix| target.starts_with(prefix))
    }
}

struct SummaryVisitor {
    summary: String,
}

impl Visit for SummaryVisitor {
    fn record_str(&mut self, field: &Field, value: &str) {
        if self.summary.is_empty() && (field.name() == "message" || field.name() == "summary") {
            self.summary = value.chars().take(256).collect();
        }
    }

    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        if self.summary.is_empty() && (field.name() == "message" || field.name() == "summary") {
            self.summary = format!("{value:?}").chars().take(256).collect();
        }
    }
}

impl<S> Layer<S> for TracingObservationLayer
where
    S: Subscriber,
{
    fn on_event(&self, event: &tracing::Event<'_>, _ctx: Context<'_, S>) {
        let target = event.metadata().target();
        if !self.accepts(target) {
            return;
        }
        let mut visitor = SummaryVisitor {
            summary: String::new(),
        };
        event.record(&mut visitor);
        let summary = if visitor.summary.is_empty() {
            event.metadata().name().to_string()
        } else {
            visitor.summary
        };
        let sequence = self.sequence.fetch_add(1, Ordering::Relaxed);
        let mut observation = ObservationEnvelope::metadata(
            self.producer_id.clone(),
            target,
            "tracing-fallback",
            sequence,
            ObservationKind::TracingFallback,
            LifecycleStatus::Health,
            summary,
        );
        observation.provenance = Provenance::Inferred;
        let _ = self.client.try_emit(observation);
    }
}
