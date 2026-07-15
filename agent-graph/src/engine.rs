#![allow(deprecated)] // Constructs GraphEvent with legacy trace_id/attempt fields during migration

use crate::checkpoint::Checkpoint;
use crate::checkpoint_store::{CheckpointStore, RunStatus, RunSummary};
use crate::command::{Navigation, NodeOutput};
use crate::config::GraphConfig;
use crate::edge::EdgeType;
use crate::error::{AgentGraphError, Result};
use crate::event_sink::{EventSink, GraphEvent, NodeOutcomeKind};
use crate::graph::{AgentGraph, CheckpointPolicy, END, START};
use crate::interrupt::{ExecutionResult, InterruptCheckpoint};
use crate::payload::PayloadContext;
use crate::receipt::{
    digest_state, digest_value, ExecutionOutcome, GraphExecutionReceiptV1, ReplayError,
    ReplayVerification, RunBundleV1, StepExecutionReceiptV1, StepStateDeltaV1,
};
use crate::retry::RetryPolicy;
use crate::router::RouterOutput;
use crate::state::AgentState;
use crate::stream::StreamEvent;
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::future::Future;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};

#[derive(Clone)]
struct ScheduledNode {
    name: String,
    state: Option<HashMap<String, Value>>,
}

impl ScheduledNode {
    fn plain(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            state: None,
        }
    }
}

fn payload_context(
    event_sink: Arc<dyn EventSink>,
    run_id: String,
    node_id: String,
    legacy_trace_id: String,
    trace_ctx: stack_ids::TraceCtx,
) -> PayloadContext {
    let token_run_id = run_id.clone();
    let token_node_id = node_id.clone();
    PayloadContext {
        on_token: Some(Arc::new(move |token| {
            event_sink.emit(GraphEvent::Token {
                run_id: token_run_id.clone(),
                trace_id: legacy_trace_id.clone(),
                trace_ctx: Some(trace_ctx.clone()),
                node_id: token_node_id.clone(),
                token: token.to_string(),
            });
        })),
        run_id,
        node_id,
    }
}

/// Execution-related methods on AgentGraph.
impl AgentGraph {
    /// Execute the graph starting from a specific node.
    pub async fn execute(&self, start_node: &str, state: AgentState) -> Result<AgentState> {
        self.execute_with_config(start_node, state, GraphConfig::default())
            .await
    }

    /// Execute the graph and return a structured run summary alongside the result.
    pub async fn execute_with_summary(
        &self,
        start_node: &str,
        state: AgentState,
        config: GraphConfig,
    ) -> (Result<AgentState>, RunSummary) {
        self.register_reducers_on_state(&state).await;
        let event_sink = self.resolve_event_sink(None);
        let cancel = Arc::new(AtomicBool::new(false));
        let run_id = self.create_run_id(&event_sink).await;
        let trace_ctx = config.resolve_trace_ctx();

        let executor = GraphExecutor {
            graph: self,
            state,
            config,
            iteration: 0,
            event_sink,
            run_id,
            trace_ctx,
            cancel_flag: cancel,
            started_at: chrono::Utc::now(),
            total_attempts: 0,
            failed_attempts: 0,
            executed_nodes: HashSet::new(),
            recording: None,
            receipt_recording: None,
        };

        executor.execute(start_node).await
    }

    /// Execute the graph with a runtime config.
    pub async fn execute_with_config(
        &self,
        start_node: &str,
        state: AgentState,
        config: GraphConfig,
    ) -> Result<AgentState> {
        let (result, _summary) = self.execute_with_summary(start_node, state, config).await;
        result
    }

    /// Execute the graph and emit one receipt per completed node transition.
    pub async fn execute_with_receipt(
        &self,
        start_node: &str,
        state: AgentState,
        config: GraphConfig,
    ) -> (Result<AgentState>, GraphExecutionReceiptV1) {
        self.register_reducers_on_state(&state).await;
        let started_at = chrono::Utc::now();
        let recovery_state = state.clone();
        let recording = Arc::new(Mutex::new(Vec::new()));
        let event_sink = self.resolve_event_sink(None);
        let run_id = self.create_run_id(&event_sink).await;
        let trace_ctx = config.resolve_trace_ctx();
        let executor = GraphExecutor {
            graph: self,
            state,
            config,
            iteration: 0,
            event_sink,
            run_id,
            trace_ctx,
            cancel_flag: Arc::new(AtomicBool::new(false)),
            started_at,
            total_attempts: 0,
            failed_attempts: 0,
            executed_nodes: HashSet::new(),
            recording: None,
            receipt_recording: Some(recording.clone()),
        };
        let (result, summary) = executor.execute(start_node).await;
        let deltas = recording.lock().await.clone();
        let finished_at = summary.finished_at.unwrap_or_else(chrono::Utc::now);

        let outcome = match (&result, &summary.status) {
            (Err(AgentGraphError::CheckpointError(_)), _) => ExecutionOutcome::Partial {
                failed_step: summary.total_nodes_executed.saturating_sub(1),
            },
            (_, crate::checkpoint_store::RunStatus::Completed) => ExecutionOutcome::Completed,
            (_, crate::checkpoint_store::RunStatus::Cancelled) => ExecutionOutcome::Cancelled,
            (_, crate::checkpoint_store::RunStatus::Interrupted) => {
                // Interruptions produce a Partial outcome; the failed step
                // index is the count of nodes executed so far (best
                // available signal without per-step receipts).
                ExecutionOutcome::Partial {
                    failed_step: summary.total_nodes_executed,
                }
            }
            (_, crate::checkpoint_store::RunStatus::Failed) => ExecutionOutcome::InternalError {
                message: format!(
                    "graph execution failed (run_id={}, attempts={}, failed_attempts={})",
                    summary.run_id, summary.total_attempts, summary.failed_attempts
                ),
            },
            // Any other status (Pending, Running, etc.) is treated as
            // an internal error since execute_with_summary only returns
            // terminal summaries.
            (_, other) => ExecutionOutcome::InternalError {
                message: format!("unexpected non-terminal run status: {other:?}"),
            },
        };

        let terminal_error = result.as_ref().err().map(ToString::to_string);
        let last_index = deltas.len().checked_sub(1);
        let steps = deltas
            .iter()
            .map(|delta| StepExecutionReceiptV1 {
                step_index: delta.step_index,
                agent_id: delta.node_name.clone(),
                started_at,
                finished_at,
                input_digest: delta.input_digest.clone(),
                output_digest: delta.output_digest.clone(),
                tool_calls: Vec::new(),
                error: (Some(delta.step_index) == last_index)
                    .then(|| terminal_error.clone())
                    .flatten(),
            })
            .collect();

        let receipt = GraphExecutionReceiptV1 {
            graph_id: self.compute_graph_hash(),
            execution_id: summary.run_id.clone(),
            started_at,
            finished_at,
            steps,
            memory_generations: Vec::new(),
            recovery_state: if matches!(&result, Err(AgentGraphError::CheckpointError(_))) {
                Some(recovery_state.export().await)
            } else {
                None
            },
            outcome,
        };

        (result, receipt)
    }

    /// Execute the graph with interrupt support.
    pub async fn execute_with_interrupt(
        &self,
        start_node: &str,
        state: AgentState,
        config: GraphConfig,
    ) -> ExecutionResult {
        self.register_reducers_on_state(&state).await;
        let state_clone = state.clone();
        let event_sink = self.resolve_event_sink(None);
        let cancel = Arc::new(AtomicBool::new(false));
        let run_id = self.create_run_id(&event_sink).await;
        let trace_ctx = config.resolve_trace_ctx();

        let executor = GraphExecutor {
            graph: self,
            state,
            config,
            iteration: 0,
            event_sink,
            run_id,
            trace_ctx,
            cancel_flag: cancel,
            started_at: chrono::Utc::now(),
            total_attempts: 0,
            failed_attempts: 0,
            executed_nodes: HashSet::new(),
            recording: None,
            receipt_recording: None,
        };

        let (result, _summary) = executor.execute(start_node).await;
        match result {
            Ok(final_state) => ExecutionResult::Complete(final_state),
            Err(AgentGraphError::InterruptError {
                ref node,
                ref value,
            }) => ExecutionResult::Interrupted {
                state: state_clone,
                node: node.clone(),
                interrupt_value: value.clone(),
                checkpoint_data: Some(InterruptCheckpoint {
                    resume_node: node.clone(),
                    resume_before: false,
                    iteration: 0,
                    active_nodes: Vec::new(),
                    graph_hash: Some(self.compute_graph_hash()),
                }),
            },
            Err(ref e) => ExecutionResult::Failed {
                state: state_clone,
                error: e.to_string(),
            },
        }
    }

    /// Execute with cancellation support using an Arc-wrapped graph.
    pub fn execute_cancellable(
        self: Arc<Self>,
        start_node: &str,
        state: AgentState,
        config: GraphConfig,
    ) -> (tokio::task::JoinHandle<Result<AgentState>>, Arc<AtomicBool>) {
        let cancel = Arc::new(AtomicBool::new(false));
        let cancel_clone = cancel.clone();
        let start = start_node.to_string();
        let graph = self;

        let handle = tokio::spawn(async move {
            graph.register_reducers_on_state(&state).await;
            let event_sink = graph.resolve_event_sink(None);
            let run_id = graph.create_run_id(&event_sink).await;
            let trace_ctx = config.resolve_trace_ctx();

            let executor = GraphExecutor {
                graph: &graph,
                state,
                config,
                iteration: 0,
                event_sink,
                run_id,
                trace_ctx,
                cancel_flag: cancel_clone,
                started_at: chrono::Utc::now(),
                total_attempts: 0,
                failed_attempts: 0,
                executed_nodes: HashSet::new(),
                recording: None,
                receipt_recording: None,
            };

            let (result, _summary) = executor.execute(&start).await;
            result
        });

        (handle, cancel)
    }

    /// Execute with streaming using Arc-wrapped graph.
    /// Returns a join handle for the result and a receiver for events.
    pub fn stream(
        self: Arc<Self>,
        start_node: &str,
        state: AgentState,
        config: GraphConfig,
    ) -> (
        tokio::task::JoinHandle<Result<AgentState>>,
        mpsc::Receiver<StreamEvent>,
    ) {
        let (tx, rx) = mpsc::channel(256);
        let start = start_node.to_string();
        let graph = self;

        let handle = tokio::spawn(async move {
            graph.register_reducers_on_state(&state).await;
            let event_sink = graph.resolve_event_sink(Some(tx));
            let cancel = Arc::new(AtomicBool::new(false));
            let run_id = graph.create_run_id(&event_sink).await;
            let trace_ctx = config.resolve_trace_ctx();

            let executor = GraphExecutor {
                graph: &graph,
                state,
                config,
                iteration: 0,
                event_sink,
                run_id,
                trace_ctx,
                cancel_flag: cancel,
                started_at: chrono::Utc::now(),
                total_attempts: 0,
                failed_attempts: 0,
                executed_nodes: HashSet::new(),
                recording: None,
                receipt_recording: None,
            };

            let (result, _summary) = executor.execute(&start).await;
            result
        });

        (handle, rx)
    }

    /// Execute once while recording an offline state-transition integrity bundle.
    pub async fn record_run_bundle(
        &self,
        start_node: &str,
        state: AgentState,
        config: GraphConfig,
    ) -> Result<RunBundleV1> {
        self.register_reducers_on_state(&state).await;
        let input_state = state.export().await;
        let recording = Arc::new(Mutex::new(Vec::new()));
        let event_sink = self.resolve_event_sink(None);
        let run_id = self.create_run_id(&event_sink).await;
        let trace_ctx = config.resolve_trace_ctx();
        let started_at = chrono::Utc::now();
        let executor = GraphExecutor {
            graph: self,
            state,
            config,
            iteration: 0,
            event_sink,
            run_id,
            trace_ctx,
            cancel_flag: Arc::new(AtomicBool::new(false)),
            started_at,
            total_attempts: 0,
            failed_attempts: 0,
            executed_nodes: HashSet::new(),
            recording: Some(recording.clone()),
            receipt_recording: None,
        };
        let (result, summary) = executor.execute(start_node).await;
        let deltas = recording.lock().await.clone();
        let finished_at = summary.finished_at.unwrap_or_else(chrono::Utc::now);
        let steps = deltas
            .iter()
            .map(|delta| StepExecutionReceiptV1 {
                step_index: delta.step_index,
                agent_id: delta.node_name.clone(),
                started_at,
                finished_at,
                input_digest: delta.input_digest.clone(),
                output_digest: delta.output_digest.clone(),
                tool_calls: Vec::new(),
                error: None,
            })
            .collect();
        let outcome = match &result {
            Ok(_) => ExecutionOutcome::Completed,
            Err(AgentGraphError::CheckpointError(_)) => ExecutionOutcome::Partial {
                failed_step: deltas.len().saturating_sub(1),
            },
            Err(error) => ExecutionOutcome::InternalError {
                message: error.to_string(),
            },
        };
        let terminal_receipt = GraphExecutionReceiptV1 {
            graph_id: self.compute_graph_hash(),
            execution_id: summary.run_id,
            started_at,
            finished_at,
            steps,
            memory_generations: Vec::new(),
            recovery_state: None,
            outcome,
        };
        result?;
        // Replay bundles never enumerate the process environment. Callers
        // may add explicitly constructed, non-secret descriptors after
        // recording when their artifact policy permits it.
        let environment = Default::default();
        Ok(RunBundleV1 {
            graph_spec: self.graph_spec_v1(),
            input_state,
            step_state_deltas: deltas,
            model_call_envelopes: Vec::new(),
            tool_call_envelopes: Vec::new(),
            memory_read_envelopes: Vec::new(),
            environment,
            terminal_receipt,
        })
    }

    /// Verify bundle integrity without re-executing nodes or external effects.
    pub fn verify_replay(
        &self,
        bundle: &RunBundleV1,
    ) -> std::result::Result<ReplayVerification, ReplayError> {
        let expected = self.compute_graph_hash();
        let actual = bundle.graph_spec.canonical_digest();
        if expected != actual {
            return Err(ReplayError::GraphMismatch { expected, actual });
        }

        let mut state = bundle.input_state.clone();
        for (position, delta) in bundle.step_state_deltas.iter().enumerate() {
            if delta.step_index != position || delta.state_before != state {
                return Err(ReplayError::StepDivergence {
                    step_index: position,
                    node_name: delta.node_name.clone(),
                    reason: "input state does not match the preceding step".into(),
                });
            }
            if digest_state(&delta.state_before) != delta.input_digest {
                return Err(ReplayError::StepDivergence {
                    step_index: position,
                    node_name: delta.node_name.clone(),
                    reason: "input digest mismatch".into(),
                });
            }
            if digest_state(&delta.state_after) != delta.output_digest {
                return Err(ReplayError::StepDivergence {
                    step_index: position,
                    node_name: delta.node_name.clone(),
                    reason: "output digest mismatch".into(),
                });
            }
            let receipt_step = bundle.terminal_receipt.steps.get(position).ok_or_else(|| {
                ReplayError::TerminalDivergence {
                    reason: format!("missing receipt step {position}"),
                }
            })?;
            if receipt_step.input_digest != delta.input_digest
                || receipt_step.output_digest != delta.output_digest
            {
                return Err(ReplayError::StepDivergence {
                    step_index: position,
                    node_name: delta.node_name.clone(),
                    reason: "terminal receipt digest mismatch".into(),
                });
            }
            state = delta.state_after.clone();
        }

        for envelope in &bundle.tool_call_envelopes {
            if envelope.step_index >= bundle.step_state_deltas.len()
                || digest_value(&envelope.request) != envelope.request_digest
                || digest_value(&envelope.response) != envelope.response_digest
            {
                return Err(ReplayError::EnvelopeDivergence {
                    step_index: envelope.step_index,
                    reason: format!("tool envelope '{}' digest mismatch", envelope.tool_name),
                });
            }
        }
        for envelope in &bundle.model_call_envelopes {
            if envelope.step_index >= bundle.step_state_deltas.len()
                || digest_value(&envelope.request) != envelope.request_digest
                || digest_value(&envelope.response) != envelope.response_digest
            {
                return Err(ReplayError::EnvelopeDivergence {
                    step_index: envelope.step_index,
                    reason: format!("model envelope '{}' digest mismatch", envelope.model_name),
                });
            }
        }
        for envelope in &bundle.memory_read_envelopes {
            if envelope.step_index >= bundle.step_state_deltas.len()
                || digest_value(&envelope.query) != envelope.query_digest
                || digest_value(&envelope.result) != envelope.result_digest
            {
                return Err(ReplayError::EnvelopeDivergence {
                    step_index: envelope.step_index,
                    reason: format!("memory envelope '{}' digest mismatch", envelope.backend),
                });
            }
        }
        if bundle.terminal_receipt.graph_id != self.compute_graph_hash()
            || bundle.terminal_receipt.steps.len() != bundle.step_state_deltas.len()
            || !matches!(bundle.terminal_receipt.outcome, ExecutionOutcome::Completed)
        {
            return Err(ReplayError::TerminalDivergence {
                reason: "terminal graph, step count, or outcome mismatch".into(),
            });
        }

        Ok(ReplayVerification {
            steps_verified: bundle.step_state_deltas.len(),
            final_state_digest: digest_state(&state),
        })
    }
}

/// Internal executor that runs the graph using superstep-based execution.
struct GraphExecutor<'a> {
    graph: &'a AgentGraph,
    state: AgentState,
    config: GraphConfig,
    iteration: usize,
    event_sink: Arc<dyn EventSink>,
    run_id: String,
    /// Canonical trace context resolved from config at execution start.
    /// The legacy `trace_id: String` is derived on demand via
    /// `self.trace_ctx.to_legacy_trace_id().to_string()` at event emission sites.
    trace_ctx: stack_ids::TraceCtx,
    cancel_flag: Arc<AtomicBool>,
    started_at: chrono::DateTime<chrono::Utc>,
    total_attempts: usize,
    failed_attempts: usize,
    executed_nodes: HashSet<String>,
    recording: Option<Arc<Mutex<Vec<StepStateDeltaV1>>>>,
    receipt_recording: Option<Arc<Mutex<Vec<StepStateDeltaV1>>>>,
}

impl<'a> GraphExecutor<'a> {
    /// Derive the legacy trace_id string from the canonical TraceCtx.
    fn legacy_trace_id(&self) -> String {
        self.trace_ctx.to_legacy_trace_id().to_string()
    }

    async fn persist<T, E: std::fmt::Display>(
        &self,
        context: &str,
        result: std::result::Result<T, E>,
    ) -> Result<Option<T>> {
        match result {
            Ok(value) => Ok(Some(value)),
            Err(error) => match self.graph.checkpoint_policy {
                CheckpointPolicy::Required => Err(AgentGraphError::CheckpointError(format!(
                    "{context}: {error}"
                ))),
                CheckpointPolicy::BestEffort => {
                    tracing::warn!(checkpoint_context = context, error = %error, "checkpoint persistence failed");
                    Ok(None)
                }
                CheckpointPolicy::Disabled => Ok(None),
            },
        }
    }

    async fn execute(mut self, start_node: &str) -> (Result<AgentState>, RunSummary) {
        // Derive legacy trace_id once at the run-level compatibility boundary.
        let run_legacy_tid = self.legacy_trace_id();

        // Emit run start
        self.event_sink.emit(GraphEvent::RunStart {
            run_id: self.run_id.clone(),
            trace_id: run_legacy_tid.clone(),
            trace_ctx: Some(self.trace_ctx.clone()),
            graph_name: self.graph.graph_name.clone(),
        });

        let mut result = self.execute_inner(start_node).await;

        // Record final status in checkpoint store
        if self.graph.checkpoint_policy != CheckpointPolicy::Disabled {
            if let Some(ref store) = self.graph.checkpoint_store {
                let persistence = match &result {
                    Ok(_) => {
                        self.persist("complete run", store.complete_run(&self.run_id).await)
                            .await
                    }
                    Err(AgentGraphError::Cancelled) => {
                        self.persist(
                            "cancel run",
                            store.fail_run(&self.run_id, "cancelled").await,
                        )
                        .await
                    }
                    Err(error) => {
                        self.persist(
                            "fail run",
                            store.fail_run(&self.run_id, &error.to_string()).await,
                        )
                        .await
                    }
                };
                if let Err(error) = persistence {
                    result = Err(error);
                }
            }
        }

        let status = match &result {
            Ok(_) => RunStatus::Completed,
            Err(AgentGraphError::Cancelled) => RunStatus::Cancelled,
            Err(AgentGraphError::InterruptError { .. }) => RunStatus::Interrupted,
            Err(_) => RunStatus::Failed,
        };

        let summary = self.build_run_summary(status, run_legacy_tid.clone());

        // Emit run end
        self.event_sink.emit(GraphEvent::RunEnd {
            run_id: self.run_id.clone(),
            trace_id: run_legacy_tid,
            trace_ctx: Some(self.trace_ctx.clone()),
        });

        (result, summary)
    }

    async fn execute_inner(&mut self, start_node: &str) -> Result<AgentState> {
        let mut current_superstep = if start_node == START {
            self.get_edge_targets(START).await?
        } else {
            vec![ScheduledNode::plain(start_node)]
        };

        let mut step_number: usize = 0;
        let max_iter = self.config.recursion_limit.min(self.graph.max_iterations);

        // Derive legacy trace_id once for all superstep-level events in this execution.
        let loop_legacy_tid = self.legacy_trace_id();

        loop {
            current_superstep.retain(|node| node.name != END);

            if current_superstep.is_empty() {
                break;
            }

            // Check cancellation
            if self.cancel_flag.load(Ordering::Relaxed) {
                return Err(AgentGraphError::Cancelled);
            }

            // Check max iterations
            if self.iteration >= max_iter {
                return Err(AgentGraphError::MaxIterationsExceeded {
                    current: self.iteration,
                    max: max_iter,
                });
            }

            // Cycle detection
            if self.graph.enable_cycle_detection && step_number > max_iter * 2 {
                return Err(AgentGraphError::CycleDetected {
                    path: current_superstep
                        .iter()
                        .map(|node| node.name.clone())
                        .collect(),
                });
            }

            // Emit superstep start
            self.event_sink.emit(GraphEvent::SuperstepStart {
                run_id: self.run_id.clone(),
                trace_id: loop_legacy_tid.clone(),
                trace_ctx: Some(self.trace_ctx.clone()),
                step: step_number,
                nodes: current_superstep
                    .iter()
                    .map(|node| node.name.clone())
                    .collect(),
            });

            // Check interrupt_before
            if let Some(ref interrupt_cfg) = self.graph.interrupt_config {
                for scheduled in &current_superstep {
                    let node_name = &scheduled.name;
                    if interrupt_cfg.should_interrupt_before(node_name) {
                        self.event_sink.emit(GraphEvent::InterruptRaised {
                            run_id: self.run_id.clone(),
                            trace_id: loop_legacy_tid.clone(),
                            trace_ctx: Some(self.trace_ctx.clone()),
                            node_id: node_name.clone(),
                            kind: "before".to_string(),
                            payload: Value::Null,
                        });

                        // Save legacy checkpoint
                        if self.graph.checkpoint_policy != CheckpointPolicy::Disabled {
                            if let Some(ref checkpointer) = self.graph.checkpointer {
                                if let Some(ref thread_id) = self.config.thread_id {
                                    let cp = Checkpoint {
                                        execution_id: thread_id.clone(),
                                        timestamp: chrono::Utc::now(),
                                        current_node: node_name.clone(),
                                        iteration: self.iteration,
                                        state: self.state.snapshot().await,
                                        step_number,
                                        active_nodes: current_superstep
                                            .iter()
                                            .map(|node| node.name.clone())
                                            .collect(),
                                    };
                                    self.persist(
                                        "interrupt-before checkpoint",
                                        checkpointer.save(&cp).await,
                                    )
                                    .await?;
                                }
                            }
                        }

                        // Save state to checkpoint store
                        if self.graph.checkpoint_policy != CheckpointPolicy::Disabled {
                            if let Some(ref store) = self.graph.checkpoint_store {
                                let state_data = self.state.export().await;
                                self.persist(
                                    "interrupt-before state snapshot",
                                    store.save_state_snapshot(&self.run_id, &state_data).await,
                                )
                                .await?;
                            }
                        }

                        return Err(AgentGraphError::InterruptError {
                            node: node_name.clone(),
                            value: None,
                        });
                    }
                }
            }

            // Execute the superstep
            let mut next_nodes = Vec::new();

            if current_superstep.len() == 1 {
                let scheduled = &current_superstep[0];
                if let Some(branch_state) = &scheduled.state {
                    for (key, value) in branch_state {
                        self.state.set_raw(key, value.clone()).await?;
                    }
                }
                let node_name = &scheduled.name;
                let output = self.execute_single_node(node_name).await?;
                let targets = self.resolve_output(node_name, output).await?;
                next_nodes.extend(targets);
            } else {
                // Parallel execution
                let snapshot_data = self.state.export().await;
                let mut join_set = tokio::task::JoinSet::new();
                let max_parallelism = self.config.max_parallelism.max(1);
                let mut pending = current_superstep.iter().enumerate();

                for _ in 0..max_parallelism {
                    let Some((invocation_index, scheduled)) = pending.next() else {
                        break;
                    };
                    self.spawn_parallel_branch(&mut join_set, invocation_index, scheduled)
                        .await?;
                }

                let mut branch_results = Vec::new();
                while let Some(result) = join_set.join_next().await {
                    let inner = result.map_err(|e| AgentGraphError::ExecutionError(e.to_string()));
                    let branch = match inner {
                        Ok(Ok(branch)) => branch,
                        Ok(Err(error)) => {
                            self.failed_attempts += 1;
                            return Err(error);
                        }
                        Err(error) => {
                            self.failed_attempts += 1;
                            return Err(error);
                        }
                    };
                    branch_results.push(branch);

                    if let Some((invocation_index, scheduled)) = pending.next() {
                        self.spawn_parallel_branch(&mut join_set, invocation_index, scheduled)
                            .await?;
                    }
                }

                branch_results.sort_by_key(|(invocation_index, _, _, _)| *invocation_index);

                if let Some(recording) = &self.receipt_recording {
                    let mut recording = recording.lock().await;
                    for (invocation_index, name, branch_state, _) in &branch_results {
                        let mut before = snapshot_data.clone();
                        if let Some(state) = &current_superstep[*invocation_index].state {
                            before.extend(state.clone());
                        }
                        let after = branch_state.export().await;
                        let step_index = recording.len();
                        recording.push(StepStateDeltaV1 {
                            step_index,
                            node_name: name.clone(),
                            input_digest: digest_state(&before),
                            output_digest: digest_state(&after),
                            state_before: before,
                            state_after: after,
                        });
                    }
                }

                self.merge_parallel_states(&snapshot_data, &branch_results)
                    .await?;

                if let Some(recording) = &self.recording {
                    let final_merged_state = self.state.export().await;
                    let mut replay_state = snapshot_data.clone();
                    let branch_count = branch_results.len();
                    let mut recording = recording.lock().await;
                    for (branch_index, (_, name, branch_state, _)) in
                        branch_results.iter().enumerate()
                    {
                        let before = replay_state.clone();
                        let branch_data = branch_state.export().await;
                        for (key, value) in branch_data {
                            if snapshot_data.get(&key) != Some(&value) {
                                replay_state.insert(key, value);
                            }
                        }
                        if branch_index + 1 == branch_count {
                            replay_state = final_merged_state.clone();
                        }
                        let step_index = recording.len();
                        recording.push(StepStateDeltaV1 {
                            step_index,
                            node_name: name.clone(),
                            input_digest: digest_state(&before),
                            output_digest: digest_state(&replay_state),
                            state_before: before,
                            state_after: replay_state.clone(),
                        });
                    }
                }

                for (_, name, _, output) in branch_results {
                    let targets = self.resolve_output(&name, output).await?;
                    next_nodes.extend(targets);
                }
            }

            // Check interrupt_after
            if let Some(ref interrupt_cfg) = self.graph.interrupt_config {
                for scheduled in &current_superstep {
                    let node_name = &scheduled.name;
                    if interrupt_cfg.should_interrupt_after(node_name) {
                        if self.graph.checkpoint_policy != CheckpointPolicy::Disabled {
                            if let Some(ref checkpointer) = self.graph.checkpointer {
                                if let Some(ref thread_id) = self.config.thread_id {
                                    let cp = Checkpoint {
                                        execution_id: thread_id.clone(),
                                        timestamp: chrono::Utc::now(),
                                        current_node: node_name.clone(),
                                        iteration: self.iteration,
                                        state: self.state.snapshot().await,
                                        step_number,
                                        active_nodes: next_nodes
                                            .iter()
                                            .map(|node| node.name.clone())
                                            .collect(),
                                    };
                                    self.persist(
                                        "interrupt-after checkpoint",
                                        checkpointer.save(&cp).await,
                                    )
                                    .await?;
                                }
                            }
                        }

                        if self.graph.checkpoint_policy != CheckpointPolicy::Disabled {
                            if let Some(ref store) = self.graph.checkpoint_store {
                                let state_data = self.state.export().await;
                                self.persist(
                                    "interrupt-after state snapshot",
                                    store.save_state_snapshot(&self.run_id, &state_data).await,
                                )
                                .await?;
                            }
                        }

                        return Err(AgentGraphError::InterruptError {
                            node: node_name.clone(),
                            value: None,
                        });
                    }
                }
            }

            // Save checkpoint after superstep
            if self.graph.checkpoint_policy != CheckpointPolicy::Disabled {
                if let Some(ref checkpointer) = self.graph.checkpointer {
                    if let Some(ref thread_id) = self.config.thread_id {
                        let current = current_superstep
                            .first()
                            .map(|node| node.name.clone())
                            .unwrap_or_default();
                        let cp = Checkpoint {
                            execution_id: thread_id.clone(),
                            timestamp: chrono::Utc::now(),
                            current_node: current,
                            iteration: self.iteration,
                            state: self.state.snapshot().await,
                            step_number,
                            active_nodes: next_nodes.iter().map(|node| node.name.clone()).collect(),
                        };
                        self.persist("superstep checkpoint", checkpointer.save(&cp).await)
                            .await?;
                    }
                }
            }

            // Save state snapshot to new checkpoint store
            if self.graph.checkpoint_policy != CheckpointPolicy::Disabled {
                if let Some(ref store) = self.graph.checkpoint_store {
                    let state_data = self.state.export().await;
                    self.persist(
                        "superstep state snapshot",
                        store.save_state_snapshot(&self.run_id, &state_data).await,
                    )
                    .await?;
                }
            }

            // Emit superstep end
            self.event_sink.emit(GraphEvent::SuperstepEnd {
                run_id: self.run_id.clone(),
                trace_id: loop_legacy_tid.clone(),
                trace_ctx: Some(self.trace_ctx.clone()),
                step: step_number,
            });

            // Deduplicate next nodes
            let mut seen = std::collections::HashSet::new();
            next_nodes.retain(|node| node.state.is_some() || seen.insert(node.name.clone()));

            self.iteration += 1;
            step_number += 1;
            current_superstep = next_nodes;
        }

        Ok(self.state.clone())
    }

    /// Execute a single node (sequential path).
    async fn execute_single_node(&mut self, name: &str) -> Result<NodeOutput> {
        self.total_attempts += 1;
        self.executed_nodes.insert(name.to_string());
        let node = self
            .graph
            .nodes
            .get(name)
            .cloned()
            .ok_or_else(|| AgentGraphError::NodeNotFound(name.to_string()))?;

        // AttemptId is stable across the whole retry family (I031).
        let canonical_attempt_id = stack_ids::AttemptId::generate();
        let family_attempt = self.total_attempts as u32;

        // Derive legacy trace_id once at the compatibility boundary for all
        // events emitted during this node execution.
        let legacy_tid = self.legacy_trace_id();
        let trace_ctx = Some(self.trace_ctx.clone());
        let node_name = name.to_string();
        let retry = self.graph.retry_policies.get(name).cloned();

        let before = self.state.export().await;
        let outcome = if let Some(ref executor) = self.graph.executor {
            let executor = executor.clone();
            let state = self.state.clone();
            let config = self.config.clone();
            execute_node_attempt_family(
                move || {
                    let executor = executor.clone();
                    let node = node.clone();
                    let state = state.clone();
                    let config = config.clone();
                    async move { executor.execute_node(node, state, config).await }
                },
                retry,
                self.state.clone(),
                self.event_sink.clone(),
                self.graph.checkpoint_store.clone(),
                self.graph.checkpoint_policy,
                self.run_id.clone(),
                node_name.clone(),
                legacy_tid.clone(),
                trace_ctx.clone(),
                family_attempt,
                canonical_attempt_id.clone(),
            )
            .await
        } else {
            let state = self.state.clone();
            let config = self.config.clone();
            let context = payload_context(
                self.event_sink.clone(),
                self.run_id.clone(),
                node_name.clone(),
                legacy_tid.clone(),
                self.trace_ctx.clone(),
            );
            execute_node_attempt_family(
                move || {
                    let node = node.clone();
                    let state = state.clone();
                    let config = config.clone();
                    let context = context.clone();
                    async move { node.execute_with_context(&state, &config, &context).await }
                },
                retry,
                self.state.clone(),
                self.event_sink.clone(),
                self.graph.checkpoint_store.clone(),
                self.graph.checkpoint_policy,
                self.run_id.clone(),
                node_name.clone(),
                legacy_tid.clone(),
                trace_ctx.clone(),
                family_attempt,
                canonical_attempt_id.clone(),
            )
            .await
        };

        match outcome {
            Ok(outcome) => {
                // Track state updates for events
                let after = self.state.export().await;
                for recording in [self.recording.as_ref(), self.receipt_recording.as_ref()]
                    .into_iter()
                    .flatten()
                {
                    let mut recording = recording.lock().await;
                    let step_index = recording.len();
                    recording.push(StepStateDeltaV1 {
                        step_index,
                        node_name: name.to_string(),
                        input_digest: digest_state(&before),
                        output_digest: digest_state(&after),
                        state_before: before.clone(),
                        state_after: after.clone(),
                    });
                }
                let mut updates = HashMap::new();
                for (key, val) in &after {
                    match before.get(key) {
                        Some(old_val) if old_val != val => {
                            updates.insert(key.clone(), val.clone());
                        }
                        None => {
                            updates.insert(key.clone(), val.clone());
                        }
                        _ => {}
                    }
                }
                if !updates.is_empty() {
                    self.event_sink.emit(GraphEvent::StateUpdate {
                        run_id: self.run_id.clone(),
                        trace_id: legacy_tid.clone(),
                        trace_ctx: trace_ctx.clone(),
                        node_id: node_name.clone(),
                        updates,
                    });
                }

                self.event_sink.emit(GraphEvent::NodeEnd {
                    run_id: self.run_id.clone(),
                    trace_id: legacy_tid.clone(),
                    trace_ctx,
                    node_id: node_name,
                    outcome: NodeOutcomeKind::Success,
                    attempt_id: Some(canonical_attempt_id),
                    trial_id: Some(outcome.trial_id),
                });
                Ok(outcome.output)
            }
            Err(failure) => {
                self.failed_attempts += 1;
                if let Some(recording) = &self.receipt_recording {
                    let after = self.state.export().await;
                    let mut recording = recording.lock().await;
                    let step_index = recording.len();
                    recording.push(StepStateDeltaV1 {
                        step_index,
                        node_name: name.to_string(),
                        input_digest: digest_state(&before),
                        output_digest: digest_state(&after),
                        state_before: before,
                        state_after: after,
                    });
                }
                self.event_sink.emit(GraphEvent::NodeEnd {
                    run_id: self.run_id.clone(),
                    trace_id: legacy_tid,
                    trace_ctx,
                    node_id: node_name,
                    outcome: failure.outcome,
                    attempt_id: Some(canonical_attempt_id),
                    trial_id: Some(failure.trial_id),
                });
                Err(failure.error)
            }
        }
    }

    /// Resolve the output of a node to a list of next node names.
    async fn resolve_output(
        &self,
        node_name: &str,
        output: NodeOutput,
    ) -> Result<Vec<ScheduledNode>> {
        match output {
            NodeOutput::Done => self.get_edge_targets(node_name).await,
            NodeOutput::Command(cmd) => match cmd.goto {
                Navigation::Default => self.get_edge_targets(node_name).await,
                Navigation::Node(n) => Ok(vec![ScheduledNode::plain(n)]),
                Navigation::Nodes(ns) => Ok(ns.into_iter().map(ScheduledNode::plain).collect()),
                Navigation::End => Ok(vec![ScheduledNode::plain(END)]),
                Navigation::Send(ops) => Ok(ops
                    .into_iter()
                    .map(|op| ScheduledNode {
                        name: op.node,
                        state: Some(op.state),
                    })
                    .collect()),
            },
        }
    }

    /// Get all edge targets from a node.
    async fn get_edge_targets(&self, node_name: &str) -> Result<Vec<ScheduledNode>> {
        let mut targets = Vec::new();
        if let Some(edge_list) = self.graph.edges.get(node_name) {
            for edge in edge_list {
                match edge {
                    EdgeType::Normal(to) => targets.push(ScheduledNode::plain(to)),
                    EdgeType::Conditional(router) => {
                        match router.route(&self.state, &self.config).await? {
                            RouterOutput::Next(Some(n)) => targets.push(ScheduledNode::plain(n)),
                            RouterOutput::Next(None) => {}
                            RouterOutput::FanOut(ns) => {
                                targets.extend(ns.into_iter().map(ScheduledNode::plain))
                            }
                        }
                    }
                }
            }
        }
        Ok(targets)
    }

    /// Merge state changes from parallel branches back into the main state.
    async fn merge_parallel_states(
        &self,
        snapshot: &HashMap<String, Value>,
        branches: &[(usize, String, AgentState, NodeOutput)],
    ) -> Result<()> {
        let mut changes: HashMap<String, Vec<Value>> = HashMap::new();

        for (_, _, branch_state, _) in branches {
            let branch_data = branch_state.export().await;
            for (key, new_value) in &branch_data {
                let changed = match snapshot.get(key) {
                    Some(old_val) => old_val != new_value,
                    None => true,
                };
                if changed {
                    changes
                        .entry(key.clone())
                        .or_default()
                        .push(new_value.clone());
                }
            }
        }

        for (key, values) in changes {
            let base = snapshot.get(&key).cloned().unwrap_or(Value::Null);
            let mut current = base;
            for value in values {
                current = self.state.apply_reducer(&key, &current, &value).await?;
            }
            self.state.set_raw(&key, current).await?;
        }

        Ok(())
    }

    async fn spawn_parallel_branch(
        &mut self,
        join_set: &mut tokio::task::JoinSet<Result<(usize, String, AgentState, NodeOutput)>>,
        invocation_index: usize,
        scheduled: &ScheduledNode,
    ) -> Result<()> {
        let node_name = &scheduled.name;
        self.total_attempts += 1;
        self.executed_nodes.insert(node_name.to_string());

        let forked_state = self.state.fork().await;
        if let Some(branch_state) = &scheduled.state {
            for (key, value) in branch_state {
                forked_state.set_raw(key, value.clone()).await?;
            }
        }
        let node = self
            .graph
            .nodes
            .get(node_name)
            .cloned()
            .ok_or_else(|| AgentGraphError::NodeNotFound(node_name.to_string()))?;
        let config = self.config.clone();
        let name = node_name.to_string();
        let retry_policy = self.graph.retry_policies.get(node_name).cloned();
        let event_sink = self.event_sink.clone();
        let run_id = self.run_id.clone();
        let trace_id = self.legacy_trace_id();
        let trace_ctx = Some(self.trace_ctx.clone());
        let checkpoint_store = self.graph.checkpoint_store.clone();
        let checkpoint_policy = self.graph.checkpoint_policy;
        let node_attempt_count = self.total_attempts as u32;

        if let Some(ref executor) = self.graph.executor {
            let exec = executor.clone();
            join_set.spawn(async move {
                // AttemptId is stable across the retry family (I031).
                let canonical_attempt_id = stack_ids::AttemptId::generate();

                let before = forked_state.export().await;
                let execution_state = forked_state.clone();
                let outcome = execute_node_attempt_family(
                    move || {
                        let exec = exec.clone();
                        let node = node.clone();
                        let forked_state = execution_state.clone();
                        let config = config.clone();
                        async move { exec.execute_node(node, forked_state, config).await }
                    },
                    retry_policy,
                    forked_state.clone(),
                    event_sink.clone(),
                    checkpoint_store.clone(),
                    checkpoint_policy,
                    run_id.clone(),
                    name.clone(),
                    trace_id.clone(),
                    trace_ctx.clone(),
                    node_attempt_count,
                    canonical_attempt_id.clone(),
                )
                .await;

                match outcome {
                    Ok(outcome) => {
                        let after = forked_state.export().await;
                        let mut updates = HashMap::new();
                        for (key, val) in &after {
                            match before.get(key) {
                                Some(old_val) if old_val != val => {
                                    updates.insert(key.clone(), val.clone());
                                }
                                None => {
                                    updates.insert(key.clone(), val.clone());
                                }
                                _ => {}
                            }
                        }
                        if !updates.is_empty() {
                            event_sink.emit(GraphEvent::StateUpdate {
                                run_id: run_id.clone(),
                                trace_id: trace_id.clone(),
                                trace_ctx: trace_ctx.clone(),
                                node_id: name.clone(),
                                updates,
                            });
                        }

                        event_sink.emit(GraphEvent::NodeEnd {
                            run_id: run_id.clone(),
                            trace_id: trace_id.clone(),
                            trace_ctx: trace_ctx.clone(),
                            node_id: name.clone(),
                            outcome: NodeOutcomeKind::Success,
                            attempt_id: Some(canonical_attempt_id),
                            trial_id: Some(outcome.trial_id),
                        });

                        Ok::<_, AgentGraphError>((
                            invocation_index,
                            name,
                            forked_state,
                            outcome.output,
                        ))
                    }
                    Err(failure) => {
                        event_sink.emit(GraphEvent::NodeEnd {
                            run_id: run_id.clone(),
                            trace_id: trace_id.clone(),
                            trace_ctx: trace_ctx.clone(),
                            node_id: name.clone(),
                            outcome: failure.outcome,
                            attempt_id: Some(canonical_attempt_id),
                            trial_id: Some(failure.trial_id),
                        });
                        Err(failure.error)
                    }
                }
            });
        } else {
            let context = payload_context(
                event_sink.clone(),
                run_id.clone(),
                name.clone(),
                trace_id.clone(),
                self.trace_ctx.clone(),
            );
            join_set.spawn(async move {
                // AttemptId is stable across the retry family (I031).
                let canonical_attempt_id = stack_ids::AttemptId::generate();

                let before = forked_state.export().await;
                let execution_state = forked_state.clone();
                let outcome = execute_node_attempt_family(
                    move || {
                        let node = node.clone();
                        let forked_state = execution_state.clone();
                        let config = config.clone();
                        let context = context.clone();
                        async move {
                            node.execute_with_context(&forked_state, &config, &context)
                                .await
                        }
                    },
                    retry_policy,
                    forked_state.clone(),
                    event_sink.clone(),
                    checkpoint_store.clone(),
                    checkpoint_policy,
                    run_id.clone(),
                    name.clone(),
                    trace_id.clone(),
                    trace_ctx.clone(),
                    node_attempt_count,
                    canonical_attempt_id.clone(),
                )
                .await;

                match outcome {
                    Ok(outcome) => {
                        let after = forked_state.export().await;
                        let mut updates = HashMap::new();
                        for (key, val) in &after {
                            match before.get(key) {
                                Some(old_val) if old_val != val => {
                                    updates.insert(key.clone(), val.clone());
                                }
                                None => {
                                    updates.insert(key.clone(), val.clone());
                                }
                                _ => {}
                            }
                        }
                        if !updates.is_empty() {
                            event_sink.emit(GraphEvent::StateUpdate {
                                run_id: run_id.clone(),
                                trace_id: trace_id.clone(),
                                trace_ctx: trace_ctx.clone(),
                                node_id: name.clone(),
                                updates,
                            });
                        }

                        event_sink.emit(GraphEvent::NodeEnd {
                            run_id: run_id.clone(),
                            trace_id: trace_id.clone(),
                            trace_ctx: trace_ctx.clone(),
                            node_id: name.clone(),
                            outcome: NodeOutcomeKind::Success,
                            attempt_id: Some(canonical_attempt_id),
                            trial_id: Some(outcome.trial_id),
                        });

                        Ok::<_, AgentGraphError>((
                            invocation_index,
                            name,
                            forked_state,
                            outcome.output,
                        ))
                    }
                    Err(failure) => {
                        event_sink.emit(GraphEvent::NodeEnd {
                            run_id: run_id.clone(),
                            trace_id: trace_id.clone(),
                            trace_ctx: trace_ctx.clone(),
                            node_id: name.clone(),
                            outcome: failure.outcome,
                            attempt_id: Some(canonical_attempt_id),
                            trial_id: Some(failure.trial_id),
                        });
                        Err(failure.error)
                    }
                }
            });
        }

        Ok(())
    }

    /// Build the run summary. `legacy_tid` is the pre-derived legacy trace ID
    /// from the run-level compatibility boundary.
    fn build_run_summary(&self, status: RunStatus, legacy_tid: String) -> RunSummary {
        RunSummary {
            run_id: self.run_id.clone(),
            graph_name: self
                .graph
                .graph_name
                .clone()
                .unwrap_or_else(|| "unnamed".to_string()),
            status,
            total_nodes_executed: self.executed_nodes.len(),
            total_attempts: self.total_attempts,
            failed_attempts: self.failed_attempts,
            trace_id: Some(legacy_tid),
            trace_ctx: Some(self.trace_ctx.clone()),
            started_at: self.started_at,
            finished_at: Some(chrono::Utc::now()),
        }
    }
}

/// Successful execution outcome for a retry family.
struct AttemptFamilySuccess {
    output: NodeOutput,
    trial_id: stack_ids::TrialId,
}

/// Final failure for a retry family after the last concrete trial ends.
struct AttemptFamilyFailure {
    error: AgentGraphError,
    outcome: NodeOutcomeKind,
    trial_id: stack_ids::TrialId,
}

#[allow(clippy::too_many_arguments)]
async fn execute_node_attempt_family<ExecOnce, ExecFut>(
    mut exec_once: ExecOnce,
    retry: Option<RetryPolicy>,
    state: AgentState,
    event_sink: Arc<dyn EventSink>,
    checkpoint_store: Option<Arc<dyn CheckpointStore>>,
    checkpoint_policy: CheckpointPolicy,
    run_id: String,
    node_id: String,
    legacy_trace_id: String,
    trace_ctx: Option<stack_ids::TraceCtx>,
    family_attempt: u32,
    canonical_attempt_id: stack_ids::AttemptId,
) -> std::result::Result<AttemptFamilySuccess, AttemptFamilyFailure>
where
    ExecOnce: FnMut() -> ExecFut,
    ExecFut: Future<Output = Result<NodeOutput>>,
{
    let max_attempts = retry
        .as_ref()
        .map_or(1, |policy| policy.max_attempts.max(1));

    for attempt_index in 0..max_attempts {
        let trial_id = stack_ids::TrialId::generate();
        event_sink.emit(GraphEvent::NodeStart {
            run_id: run_id.clone(),
            trace_id: legacy_trace_id.clone(),
            trace_ctx: trace_ctx.clone(),
            node_id: node_id.clone(),
            attempt: family_attempt,
            attempt_id: Some(canonical_attempt_id.clone()),
            trial_id: Some(trial_id.clone()),
        });

        let checkpoint_attempt_id = if checkpoint_policy == CheckpointPolicy::Disabled {
            None
        } else if let Some(ref store) = checkpoint_store {
            let state_val = serde_json::to_value(&state.export().await).unwrap_or(Value::Null);
            match store
                .record_attempt(&run_id, &node_id, attempt_index as u32, &state_val)
                .await
            {
                Ok(id) => Some(id),
                Err(error) if checkpoint_policy == CheckpointPolicy::Required => {
                    return Err(AttemptFamilyFailure {
                        error: AgentGraphError::CheckpointError(format!(
                            "record node attempt: {error}"
                        )),
                        outcome: NodeOutcomeKind::Failed,
                        trial_id,
                    });
                }
                Err(error) => {
                    tracing::warn!(error = %error, node_id, "checkpoint attempt recording failed");
                    None
                }
            }
        } else {
            None
        };

        match exec_once().await {
            Ok(output) => {
                if let NodeOutput::Command(ref cmd) = output {
                    if let Some(ref updates) = cmd.update {
                        for (key, value) in updates {
                            if let Err(error) = state.set(key, value.clone()).await {
                                if let Some(ref store) = checkpoint_store {
                                    if let Some(ref attempt_id) = checkpoint_attempt_id {
                                        if let Err(checkpoint_error) =
                                            store.fail_attempt(attempt_id, &error.to_string()).await
                                        {
                                            if checkpoint_policy == CheckpointPolicy::Required {
                                                return Err(AttemptFamilyFailure {
                                                    error: AgentGraphError::CheckpointError(format!("record failed node attempt: {checkpoint_error}")),
                                                    outcome: NodeOutcomeKind::Failed,
                                                    trial_id: trial_id.clone(),
                                                });
                                            }
                                            tracing::warn!(error = %checkpoint_error, node_id, "checkpoint failure recording failed");
                                        }
                                    }
                                }
                                return Err(AttemptFamilyFailure {
                                    error,
                                    outcome: NodeOutcomeKind::Failed,
                                    trial_id: trial_id.clone(),
                                });
                            }
                        }
                    }
                }

                if let Some(ref store) = checkpoint_store {
                    if let Some(ref attempt_id) = checkpoint_attempt_id {
                        let mut meta = HashMap::new();
                        meta.insert(
                            "trace_id".to_string(),
                            Value::String(legacy_trace_id.clone()),
                        );
                        let state_val =
                            serde_json::to_value(&state.export().await).unwrap_or(Value::Null);
                        if let Err(error) =
                            store.complete_attempt(attempt_id, &state_val, &meta).await
                        {
                            if checkpoint_policy == CheckpointPolicy::Required {
                                return Err(AttemptFamilyFailure {
                                    error: AgentGraphError::CheckpointError(format!(
                                        "complete node attempt: {error}"
                                    )),
                                    outcome: NodeOutcomeKind::Failed,
                                    trial_id,
                                });
                            }
                            tracing::warn!(error = %error, node_id, "checkpoint attempt completion failed");
                        }
                    }
                }

                return Ok(AttemptFamilySuccess { output, trial_id });
            }
            Err(error) => {
                if let Some(ref store) = checkpoint_store {
                    if let Some(ref attempt_id) = checkpoint_attempt_id {
                        if let Err(checkpoint_error) =
                            store.fail_attempt(attempt_id, &error.to_string()).await
                        {
                            if checkpoint_policy == CheckpointPolicy::Required {
                                return Err(AttemptFamilyFailure {
                                    error: AgentGraphError::CheckpointError(format!(
                                        "record failed node attempt: {checkpoint_error}"
                                    )),
                                    outcome: NodeOutcomeKind::Failed,
                                    trial_id,
                                });
                            }
                            tracing::warn!(error = %checkpoint_error, node_id, "checkpoint failure recording failed");
                        }
                    }
                }

                let outcome = if matches!(&error, AgentGraphError::InterruptError { .. }) {
                    NodeOutcomeKind::Interrupted
                } else {
                    NodeOutcomeKind::Failed
                };

                let should_retry = retry.as_ref().is_some_and(|policy| {
                    attempt_index + 1 < max_attempts && policy.should_retry(&error)
                });

                if should_retry {
                    event_sink.emit(GraphEvent::NodeEnd {
                        run_id: run_id.clone(),
                        trace_id: legacy_trace_id.clone(),
                        trace_ctx: trace_ctx.clone(),
                        node_id: node_id.clone(),
                        outcome: outcome.clone(),
                        attempt_id: Some(canonical_attempt_id.clone()),
                        trial_id: Some(trial_id.clone()),
                    });
                    let delay = retry
                        .as_ref()
                        .expect("retry policy must exist when retrying")
                        .delay_for_attempt(attempt_index);
                    tokio::time::sleep(delay).await;
                } else {
                    return Err(AttemptFamilyFailure {
                        error,
                        outcome,
                        trial_id,
                    });
                }
            }
        }
    }

    Err(AttemptFamilyFailure {
        error: AgentGraphError::ExecutionError("Retry exhausted with no error".to_string()),
        outcome: NodeOutcomeKind::Failed,
        trial_id: stack_ids::TrialId::generate(),
    })
}
