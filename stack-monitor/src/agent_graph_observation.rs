//! Non-blocking Agent Graph adapter for the collector path.

use agent_graph::event_sink::{EventSink, GraphEvent, NodeOutcomeKind};
use stack_observation::{
    Correlation, LifecycleStatus, ObservationEnvelope, ObservationKind, Provenance,
};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

/// Collector-backed Agent Graph sink. It never performs storage I/O in `emit`.
pub struct AgentGraphObservationSink {
    client: Arc<dyn crate::ObservationEmitter>,
    producer_id: String,
    sequence: AtomicU64,
}

impl AgentGraphObservationSink {
    /// Create a collector-backed graph sink.
    pub fn new(client: crate::MonitorClient, producer_id: impl Into<String>) -> Self {
        Self::new_with_emitter(Arc::new(client), producer_id)
    }

    /// Create a sink from any non-blocking observation emitter, including Unix IPC.
    pub fn new_with_emitter(
        client: Arc<dyn crate::ObservationEmitter>,
        producer_id: impl Into<String>,
    ) -> Self {
        Self {
            client,
            producer_id: producer_id.into(),
            sequence: AtomicU64::new(0),
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn envelope(
        &self,
        kind: ObservationKind,
        status: LifecycleStatus,
        summary: String,
        run_id: Option<String>,
        node_id: Option<String>,
        trace_ctx: Option<stack_ids::TraceCtx>,
        attempt_id: Option<String>,
        trial_id: Option<String>,
    ) -> ObservationEnvelope {
        let sequence = self.sequence.fetch_add(1, Ordering::Relaxed);
        let correlation = Correlation {
            run_id,
            node_id,
            trace_id: trace_ctx.as_ref().map(|ctx| ctx.trace_id.clone()),
            parent_span_id: trace_ctx.and_then(|ctx| ctx.parent_id),
            attempt_id,
            trial_id,
            ..Correlation::default()
        };
        let mut event = ObservationEnvelope::metadata(
            self.producer_id.clone(),
            "agent-graph",
            "agent-graph-observation-sink",
            sequence,
            kind,
            status,
            summary,
        );
        event.provenance = Provenance::Adapted;
        event.correlation = correlation;
        event
    }

    fn submit(&self, event: ObservationEnvelope) {
        let _ = self.client.emit_observation(event);
    }
}

impl EventSink for AgentGraphObservationSink {
    fn emit(&self, event: GraphEvent) {
        match event {
            GraphEvent::RunStart {
                run_id,
                trace_ctx,
                graph_name,
                ..
            } => self.submit(self.envelope(
                ObservationKind::GraphRun,
                LifecycleStatus::Started,
                format!(
                    "graph run started ({})",
                    graph_name.unwrap_or_else(|| "unnamed".into())
                ),
                Some(run_id),
                None,
                trace_ctx,
                None,
                None,
            )),
            GraphEvent::RunEnd {
                run_id, trace_ctx, ..
            } => self.submit(self.envelope(
                ObservationKind::GraphRun,
                LifecycleStatus::Completed,
                "graph run ended".into(),
                Some(run_id),
                None,
                trace_ctx,
                None,
                None,
            )),
            GraphEvent::NodeStart {
                run_id,
                node_id,
                trace_ctx,
                attempt_id,
                trial_id,
                ..
            } => self.submit(self.envelope(
                ObservationKind::GraphNode,
                LifecycleStatus::Started,
                format!("node '{}' started", node_id),
                Some(run_id),
                Some(node_id),
                trace_ctx,
                attempt_id.map(|id| id.to_string()),
                trial_id.map(|id| id.to_string()),
            )),
            GraphEvent::NodeEnd {
                run_id,
                node_id,
                outcome,
                trace_ctx,
                attempt_id,
                trial_id,
                ..
            } => {
                let status = match outcome {
                    NodeOutcomeKind::Success => LifecycleStatus::Completed,
                    NodeOutcomeKind::Failed => LifecycleStatus::Failed,
                    NodeOutcomeKind::Interrupted => LifecycleStatus::Cancelled,
                };
                self.submit(self.envelope(
                    ObservationKind::GraphNode,
                    status,
                    format!("node '{}' ended", node_id),
                    Some(run_id),
                    Some(node_id),
                    trace_ctx,
                    attempt_id.map(|id| id.to_string()),
                    trial_id.map(|id| id.to_string()),
                ));
            }
            GraphEvent::Token {
                run_id,
                node_id,
                trace_ctx,
                ..
            } => self.submit(self.envelope(
                ObservationKind::TokenProgress,
                LifecycleStatus::Streaming,
                format!("node '{}' streaming", node_id),
                Some(run_id),
                Some(node_id),
                trace_ctx,
                None,
                None,
            )),
            GraphEvent::CheckpointWritten {
                run_id,
                trace_ctx,
                checkpoint_attempt_id,
                ..
            } => self.submit(self.envelope(
                ObservationKind::Receipt,
                LifecycleStatus::Health,
                "graph checkpoint written".into(),
                Some(run_id),
                None,
                trace_ctx,
                Some(checkpoint_attempt_id),
                None,
            )),
            GraphEvent::InterruptRaised {
                run_id,
                node_id,
                kind,
                trace_ctx,
                ..
            } => self.submit(self.envelope(
                ObservationKind::GraphNode,
                LifecycleStatus::Cancelled,
                format!("interrupt '{}' raised", kind),
                Some(run_id),
                Some(node_id),
                trace_ctx,
                None,
                None,
            )),
            GraphEvent::StateUpdate {
                run_id,
                node_id,
                updates,
                trace_ctx,
                ..
            } => self.submit(self.envelope(
                ObservationKind::GraphNode,
                LifecycleStatus::Streaming,
                format!("node '{}' updated {} state keys", node_id, updates.len()),
                Some(run_id),
                Some(node_id),
                trace_ctx,
                None,
                None,
            )),
            GraphEvent::SuperstepStart {
                run_id,
                step,
                nodes,
                trace_ctx,
                ..
            } => self.submit(self.envelope(
                ObservationKind::GraphRun,
                LifecycleStatus::Started,
                format!("superstep {} started ({} nodes)", step, nodes.len()),
                Some(run_id),
                None,
                trace_ctx,
                None,
                None,
            )),
            GraphEvent::SuperstepEnd {
                run_id,
                step,
                trace_ctx,
                ..
            } => self.submit(self.envelope(
                ObservationKind::GraphRun,
                LifecycleStatus::Completed,
                format!("superstep {} ended", step),
                Some(run_id),
                None,
                trace_ctx,
                None,
                None,
            )),
            GraphEvent::ParallelCancellation {
                run_id,
                trace_ctx,
                external_effects_may_have_escaped,
                ..
            } => self.submit(self.envelope(
                ObservationKind::GraphRun,
                LifecycleStatus::Cancelled,
                format!(
                    "parallel branches cancelled; external effects escaped={}",
                    external_effects_may_have_escaped
                ),
                Some(run_id),
                None,
                trace_ctx,
                None,
                None,
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{start_collector, ActivityStore, EmitStatus};

    #[test]
    fn graph_events_use_non_blocking_observation_path() {
        let store = ActivityStore::open(":memory:").unwrap();
        let (client, collector) = start_collector(store.clone(), 8);
        let sink = AgentGraphObservationSink::new(client.clone(), "graph-test");
        sink.emit(GraphEvent::RunStart {
            run_id: "run-1".into(),
            trace_id: "legacy".into(),
            trace_ctx: None,
            graph_name: Some("demo".into()),
        });
        assert_eq!(client.stats().accepted, 1);
        assert_eq!(client.stats().dropped, 0);
        assert_eq!(
            store.observation_count_for_producer("graph-test").unwrap(),
            0
        );
        assert_eq!(collector.shutdown().persisted, 1);
        let _ = EmitStatus::Accepted;
    }
}
