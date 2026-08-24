//! Bridge from `agent-graph`'s structured `EventSink` to the activity store.
//!
//! This makes monitoring any agent graph a **one-liner** in the host app:
//!
//! ```rust,no_run
//! # #[cfg(feature = "agent-graph-bridge")]
//! # {
//! use stack_monitor::agent_graph_bridge::AgentGraphMonitorSink;
//! use agent_graph::AgentGraph;
//! use std::sync::Arc;
//!
//! let store = stack_monitor::ActivityStore::open("activity.db").unwrap();
//! let monitor = Arc::new(AgentGraphMonitorSink::new(&store));
//!
//! let _graph = AgentGraph::builder()
//!     .with_event_sink(monitor)        // <- monitoring attached here
//!     .build()
//!     .unwrap();
//! # }
//! ```
//!
//! The sink implements `agent_graph::EventSink` as a best-effort prototype
//! and translates every [`agent_graph::event_sink::GraphEvent`] into a [`crate::MonitoredEvent`]
//! tagged `agent-graph`, carrying `trace_ctx` / `attempt_id` / `trial_id` for
//! cross-crate correlation in the monitor GUI. It composes transparently with
//! other sinks via `agent_graph::CompositeEventSink`.

use crate::models::MonitoredEvent;
use crate::store::ActivityStore;
use agent_graph::event_sink::{EventSink, GraphEvent};
use std::sync::Arc;

/// An `agent_graph::EventSink` that records graph execution events into the
/// activity store. Drop the returned `Arc` into `AgentGraph::builder().with_event_sink(...)`.
///
/// Writes are synchronous and best-effort in this prototype. Production use
/// must route events through a bounded non-blocking channel before storage.
pub struct AgentGraphMonitorSink {
    store: Arc<ActivityStore>,
}

impl AgentGraphMonitorSink {
    /// Create a new monitor sink backed by the given store.
    #[deprecated(
        note = "use AgentGraphObservationSink with a MonitorClient for non-blocking capture"
    )]
    pub fn new(store: &ActivityStore) -> Self {
        Self {
            store: Arc::new(store.clone()),
        }
    }

    /// Convert to `Arc<dyn EventSink>` for `with_event_sink`.
    pub fn into_arc(self) -> Arc<dyn EventSink> {
        Arc::new(self)
    }
}

#[allow(deprecated)]
impl EventSink for AgentGraphMonitorSink {
    fn emit(&self, event: GraphEvent) {
        let monitored = match event {
            GraphEvent::RunStart {
                run_id,
                trace_ctx,
                graph_name,
                ..
            } => MonitoredEvent::new(
                "agent-graph",
                "graph_run_start",
                format!(
                    "▶ run {} ({})",
                    run_id,
                    graph_name.unwrap_or_else(|| "unnamed".into())
                ),
            )
            .with_trace_opt(trace_ctx)
            .with_tag("run"),

            GraphEvent::RunEnd {
                run_id, trace_ctx, ..
            } => MonitoredEvent::new(
                "agent-graph",
                "graph_run_end",
                format!("■ run {} ended", run_id),
            )
            .with_trace_opt(trace_ctx)
            .with_tag("run"),

            GraphEvent::NodeStart {
                run_id,
                node_id,
                trace_ctx,
                attempt_id,
                trial_id,
                ..
            } => MonitoredEvent::new(
                "agent-graph",
                "graph_node_start",
                format!("▸ node '{}' started (run {})", node_id, run_id),
            )
            .with_trace_opt(trace_ctx)
            .with_attempt_opt(attempt_id.map(|a| a.to_string()))
            .with_trial_opt(trial_id.map(|t| t.to_string()))
            .with_tag("graph_node"),

            GraphEvent::NodeEnd {
                run_id,
                node_id,
                outcome,
                trace_ctx,
                attempt_id,
                trial_id,
                ..
            } => {
                let (mark, ok) = match outcome {
                    agent_graph::event_sink::NodeOutcomeKind::Success => ("✓", true),
                    agent_graph::event_sink::NodeOutcomeKind::Failed => ("✗", false),
                    agent_graph::event_sink::NodeOutcomeKind::Interrupted => ("⏸", false),
                };
                MonitoredEvent::new(
                    "agent-graph",
                    "graph_node_end",
                    format!("{} node '{}' (run {})", mark, node_id, run_id),
                )
                .with_trace_opt(trace_ctx)
                .with_attempt_opt(attempt_id.map(|a| a.to_string()))
                .with_trial_opt(trial_id.map(|t| t.to_string()))
                .with_tag("graph_node")
                .with_tag(if ok { "success" } else { "failure" })
            }

            GraphEvent::Token {
                run_id,
                node_id,
                token,
                trace_ctx,
                ..
            } => MonitoredEvent::new(
                "agent-graph",
                "graph_token",
                format!("  {}…", truncate(&token, 80)),
            )
            .with_trace_opt(trace_ctx)
            .with_tag("graph_node")
            .with_tag("token")
            .with_detail(format!(
                "{{\"run_id\":\"{}\",\"node\":\"{}\"}}",
                run_id, node_id
            )),

            GraphEvent::CheckpointWritten {
                run_id,
                trace_ctx,
                checkpoint_attempt_id,
                ..
            } => MonitoredEvent::new(
                "agent-graph",
                "graph_checkpoint",
                format!(
                    "💾 checkpoint for run {} (attempt {})",
                    run_id, checkpoint_attempt_id
                ),
            )
            .with_trace_opt(trace_ctx)
            .with_tag("checkpoint"),

            GraphEvent::InterruptRaised {
                run_id,
                node_id,
                kind,
                payload,
                trace_ctx,
                ..
            } => MonitoredEvent::new(
                "agent-graph",
                "graph_interrupt",
                format!(
                    "⏸ interrupt '{}' at node '{}' (run {})",
                    kind, node_id, run_id
                ),
            )
            .with_trace_opt(trace_ctx)
            .with_tag("interrupt")
            .with_detail(serde_json::to_string(&payload).unwrap_or_default()),

            GraphEvent::StateUpdate {
                run_id,
                node_id,
                updates,
                trace_ctx,
                ..
            } => MonitoredEvent::new(
                "agent-graph",
                "graph_state_update",
                format!(
                    "⟳ state updated by '{}' (run {}): {} keys",
                    node_id,
                    run_id,
                    updates.len()
                ),
            )
            .with_trace_opt(trace_ctx)
            .with_tag("state"),

            GraphEvent::SuperstepStart {
                run_id,
                step,
                nodes,
                trace_ctx,
                ..
            } => MonitoredEvent::new(
                "agent-graph",
                "graph_superstep_start",
                format!(
                    "⊞ superstep {} ({} nodes) (run {})",
                    step,
                    nodes.len(),
                    run_id
                ),
            )
            .with_trace_opt(trace_ctx)
            .with_tag("superstep"),

            GraphEvent::SuperstepEnd {
                run_id,
                step,
                trace_ctx,
                ..
            } => MonitoredEvent::new(
                "agent-graph",
                "graph_superstep_end",
                format!("⊟ superstep {} done (run {})", step, run_id),
            )
            .with_trace_opt(trace_ctx)
            .with_tag("superstep"),

            GraphEvent::ParallelCancellation {
                run_id,
                trace_ctx,
                external_effects_may_have_escaped,
                ..
            } => MonitoredEvent::new(
                "agent-graph",
                "graph_parallel_cancel",
                format!(
                    "⊘ parallel branches cancelled (run {}); external_effects_escaped={}",
                    run_id, external_effects_may_have_escaped
                ),
            )
            .with_trace_opt(trace_ctx)
            .with_tag("cancel")
            .with_tag("failure"),
        };

        if let Err(e) = self.store.record(&monitored) {
            eprintln!("stack-monitor: failed to record agent-graph event: {e}");
        }
    }
}

/// Truncate a string to a max length, adding "..." if truncated.
fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max.saturating_sub(3)])
    }
}

#[cfg(test)]
#[allow(deprecated)]
mod tests {
    use super::*;
    use agent_graph::event_sink::{GraphEvent, NodeOutcomeKind};
    use stack_ids::{AttemptId, TraceCtx, TrialId};

    #[test]
    fn test_record_run_start() {
        let store = ActivityStore::open(":memory:").unwrap();
        let sink = AgentGraphMonitorSink::new(&store);
        sink.emit(GraphEvent::RunStart {
            run_id: "r1".into(),
            trace_id: "legacy".into(),
            trace_ctx: Some(TraceCtx::generate()),
            graph_name: Some("demo".into()),
        });
        let events = store.get_recent(10).unwrap();
        assert!(events
            .iter()
            .any(|e| e.event_type == "graph_run_start" && e.crate_name == "agent-graph"));
    }

    #[test]
    fn test_record_node_end_carries_ids() {
        let store = ActivityStore::open(":memory:").unwrap();
        let sink = AgentGraphMonitorSink::new(&store);
        let attempt_id = AttemptId::generate();
        let trial_id = TrialId::generate();
        sink.emit(GraphEvent::NodeEnd {
            run_id: "r1".into(),
            trace_id: "legacy".into(),
            trace_ctx: Some(TraceCtx::generate()),
            node_id: "llm_node".into(),
            outcome: NodeOutcomeKind::Success,
            attempt_id: Some(attempt_id.clone()),
            trial_id: Some(trial_id.clone()),
        });
        let events = store.get_recent(10).unwrap();
        let e = events
            .iter()
            .find(|e| e.event_type == "graph_node_end")
            .expect("node end event recorded");
        assert_eq!(
            e.attempt_id.as_deref(),
            Some(attempt_id.to_string().as_str())
        );
        assert_eq!(e.trial_id.as_deref(), Some(trial_id.to_string().as_str()));
        assert!(e.tags.contains(&"success".to_string()));
    }

    #[test]
    fn test_record_interrupt_and_cancel() {
        let store = ActivityStore::open(":memory:").unwrap();
        let sink = AgentGraphMonitorSink::new(&store);
        sink.emit(GraphEvent::InterruptRaised {
            run_id: "r1".into(),
            trace_id: "legacy".into(),
            trace_ctx: None,
            node_id: "n".into(),
            kind: "human".into(),
            payload: serde_json::json!({"x": 1}),
        });
        sink.emit(GraphEvent::ParallelCancellation {
            run_id: "r1".into(),
            trace_id: "legacy".into(),
            trace_ctx: None,
            external_effects_may_have_escaped: false,
        });
        let events = store.get_recent(10).unwrap();
        assert!(events.iter().any(|e| e.event_type == "graph_interrupt"));
        assert!(events
            .iter()
            .any(|e| e.event_type == "graph_parallel_cancel"));
    }
}
