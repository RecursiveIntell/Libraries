//! Event bridge: implements `llm-pipeline::EventHandler` to capture payload lifecycle events.

use crate::models::MonitoredEvent;
use crate::store::ActivityStore;

use llm_pipeline::events::{Event, EventHandler};
use stack_observation::{LifecycleStatus, ObservationEnvelope, ObservationKind};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

/// An `EventHandler` that writes `llm-pipeline` events to the ActivityStore.
///
/// Attach this to an `ExecCtx` to capture all LLM pipeline activity:
///
/// ```rust
/// use std::sync::Arc;
/// use stack_monitor::LlmPipelineEventHandler;
/// use llm_pipeline::ExecCtx;
///
/// let store = stack_monitor::ActivityStore::open("activity.db").unwrap();
/// let handler = LlmPipelineEventHandler::new(&store);
///
/// let ctx = ExecCtx::builder("http://localhost:11434")
///     .event_handler(Arc::new(handler).into_arc())
///     .build();
/// ```
///
/// This captures:
/// - `PayloadStart` — an LLM call or payload is starting
/// - `Token` — streaming token received
/// - `PayloadEnd` — payload finished (success/failure)
/// - `RetryStart` / `RetryEnd` — semantic or transport retries
/// - `PartialParse` — streaming JSON parse progress
/// - `TransportRetry` — HTTP-level retry
/// - `CostUpdate` — estimated cost after a call
pub struct LlmPipelineEventHandler {
    store: Arc<ActivityStore>,
}

impl LlmPipelineEventHandler {
    /// Create a new event handler backed by the given store.
    #[deprecated(
        note = "use LlmPipelineObservationHandler with a MonitorClient for non-blocking capture"
    )]
    pub fn new(store: &ActivityStore) -> Self {
        Self {
            store: Arc::new(store.clone()),
        }
    }

    /// Convert to an `Arc<dyn EventHandler>` for use with `ExecCtx`.
    pub fn into_arc(self: Arc<Self>) -> Arc<dyn EventHandler> {
        self
    }
}

#[allow(deprecated)]
impl EventHandler for LlmPipelineEventHandler {
    fn on_event(&self, event: Event) {
        match event {
            Event::PayloadStart { name, kind } => {
                let monitored = MonitoredEvent::new(
                    "llm-pipeline",
                    "payload_start",
                    format!("▶ payload '{}' [{}]", name, kind),
                )
                .with_tag("payload")
                .with_tag(kind);
                let _ = self.store.record(&monitored);
            }

            Event::Token { name, chunk } => {
                // Don't record every single token — that would flood the store.
                // Sample: keep a cheap atomic counter and only record every
                // Nth token cluster plus always record a token when the chunk
                // looks like a natural sentence boundary (ends with punctuation).
                use std::sync::atomic::{AtomicU64, Ordering};
                static TOKEN_SEQ: AtomicU64 = AtomicU64::new(0);
                let seq = TOKEN_SEQ.fetch_add(1, Ordering::Relaxed);
                let is_boundary = chunk.trim_end().ends_with(['.', '!', '?', '\n']);
                const SAMPLE_EVERY: u64 = 25;
                if is_boundary || seq % SAMPLE_EVERY == 0 {
                    let preview = if is_boundary {
                        truncate(&chunk, 120)
                    } else {
                        format!("…({} tokens)…", seq + 1)
                    };
                    let monitored = MonitoredEvent::new(
                        "llm-pipeline",
                        "token",
                        format!("  {} {}", name, preview),
                    )
                    .with_tag("streaming")
                    .with_tag("token");
                    let _ = self.store.record(&monitored);
                }
            }

            Event::PayloadEnd { name, ok } => {
                let status = if ok { "✓" } else { "✗" };
                let monitored = MonitoredEvent::new(
                    "llm-pipeline",
                    "payload_end",
                    format!("{} payload '{}' finished ok={}", status, name, ok),
                )
                .with_tag("payload")
                .with_tag(if ok { "success" } else { "failure" });
                let _ = self.store.record(&monitored);
            }

            Event::RetryStart {
                name,
                attempt,
                reason,
                attempt_id,
                trial_id,
            } => {
                let monitored = MonitoredEvent::new(
                    "llm-pipeline",
                    "retry_start",
                    format!("↻ retry {} for '{}' (attempt {})", name, reason, attempt),
                )
                .with_attempt(attempt_id.to_string())
                .with_trial(trial_id.to_string())
                .with_tag("retry")
                .with_tag("semantic");
                let _ = self.store.record(&monitored);
            }

            Event::RetryEnd {
                name,
                attempts,
                success,
                attempt_id,
            } => {
                let monitored = MonitoredEvent::new(
                    "llm-pipeline",
                    "retry_end",
                    format!(
                        "↻ retry '{}' done: {} attempts, success={}",
                        name, attempts, success
                    ),
                )
                .with_attempt(attempt_id.to_string())
                .with_tag("retry")
                .with_tag(if success { "success" } else { "failure" });
                let _ = self.store.record(&monitored);
            }

            Event::PartialParse {
                name,
                value,
                complete,
            } => {
                // Only record meaningful parse milestones
                if complete || value.as_object().map(|o| o.len() > 2).unwrap_or(false) {
                    let status = if complete {
                        "✓ complete"
                    } else {
                        "… partial"
                    };
                    let monitored = MonitoredEvent::new(
                        "llm-pipeline",
                        "partial_parse",
                        format!("{} partial parse for '{}'", status, name),
                    )
                    .with_tag("parse");
                    let _ = self.store.record(&monitored);
                }
            }

            Event::TransportRetry {
                name,
                attempt,
                delay_ms,
                reason,
            } => {
                let monitored = MonitoredEvent::new(
                    "llm-pipeline",
                    "transport_retry",
                    format!(
                        "↺ transport retry {} for '{}' ({}ms): {}",
                        attempt, name, delay_ms, reason
                    ),
                )
                .with_tag("retry")
                .with_tag("transport");
                let _ = self.store.record(&monitored);
            }

            Event::CostUpdate {
                name,
                estimated_cost,
                currency,
                token_usage,
            } => {
                let monitored = MonitoredEvent::new(
                    "llm-pipeline",
                    "cost_update",
                    format!("💰 {} cost: {}{:.4}", name, currency, estimated_cost),
                )
                .with_tag("cost")
                .with_detail(format!(
                    "{{\"prompt_tokens\":{},\"completion_tokens\":{},\"total_tokens\":{}}}",
                    token_usage.prompt_tokens,
                    token_usage.completion_tokens,
                    token_usage.total_tokens,
                ));
                let _ = self.store.record(&monitored);
            }
        }
    }
}

/// Truncate a string for display in event summaries.
fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let shortened: String = s.chars().take(max.saturating_sub(1)).collect();
        format!("{}…", shortened)
    }
}

/// Non-blocking, metadata-only `llm-pipeline` adapter for the collector path.
pub struct LlmPipelineObservationHandler {
    client: crate::MonitorClient,
    producer_id: String,
    sequence: AtomicU64,
}

impl LlmPipelineObservationHandler {
    /// Create a handler that never performs storage I/O from `on_event`.
    pub fn new(client: crate::MonitorClient, producer_id: impl Into<String>) -> Self {
        Self {
            client,
            producer_id: producer_id.into(),
            sequence: AtomicU64::new(0),
        }
    }

    fn emit_metadata(&self, kind: ObservationKind, status: LifecycleStatus, summary: String) {
        let sequence = self.sequence.fetch_add(1, Ordering::Relaxed);
        let event = ObservationEnvelope::metadata(
            self.producer_id.clone(),
            "llm-pipeline",
            "llm-pipeline-event-handler",
            sequence,
            kind,
            status,
            summary,
        );
        let _ = self.client.try_emit(event);
    }
}

impl EventHandler for LlmPipelineObservationHandler {
    fn on_event(&self, event: Event) {
        match event {
            Event::PayloadStart { name, .. } => self.emit_metadata(
                ObservationKind::LlmCall,
                LifecycleStatus::Started,
                format!("payload '{}' started", name),
            ),
            Event::Token { name, .. } => self.emit_metadata(
                ObservationKind::TokenProgress,
                LifecycleStatus::Streaming,
                format!("payload '{}' streaming", name),
            ),
            Event::PayloadEnd { name, ok } => self.emit_metadata(
                ObservationKind::LlmCall,
                if ok {
                    LifecycleStatus::Completed
                } else {
                    LifecycleStatus::Failed
                },
                format!("payload '{}' ended", name),
            ),
            Event::RetryStart { name, .. } => self.emit_metadata(
                ObservationKind::Retry,
                LifecycleStatus::Retried,
                format!("retry '{}' started", name),
            ),
            Event::RetryEnd { name, success, .. } => self.emit_metadata(
                ObservationKind::Retry,
                if success {
                    LifecycleStatus::Completed
                } else {
                    LifecycleStatus::Failed
                },
                format!("retry '{}' ended", name),
            ),
            Event::PartialParse { name, complete, .. } => self.emit_metadata(
                ObservationKind::Parse,
                if complete {
                    LifecycleStatus::Completed
                } else {
                    LifecycleStatus::Streaming
                },
                format!("partial parse for '{}'", name),
            ),
            Event::TransportRetry { name, .. } => self.emit_metadata(
                ObservationKind::Transport,
                LifecycleStatus::Retried,
                format!("transport retry for '{}'", name),
            ),
            Event::CostUpdate { name, .. } => self.emit_metadata(
                ObservationKind::Cost,
                LifecycleStatus::Completed,
                format!("cost update for '{}'", name),
            ),
        }
    }
}
