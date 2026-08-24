//! Collector-backed `llm-tool-runtime` receipt adapter.

use async_trait::async_trait;
use llm_tool_runtime::{ToolError, ToolReceipt, ToolReceiptSink};
use stack_observation::{
    Correlation, LifecycleStatus, ObservationEnvelope, ObservationKind, Provenance,
};
use std::sync::atomic::{AtomicU64, Ordering};

/// Converts canonical tool receipts into metadata-only observations.
pub struct ToolObservationSink {
    client: crate::MonitorClient,
    producer_id: String,
    sequence: AtomicU64,
}

impl ToolObservationSink {
    /// Create a collector-backed tool receipt sink.
    pub fn new(client: crate::MonitorClient, producer_id: impl Into<String>) -> Self {
        Self {
            client,
            producer_id: producer_id.into(),
            sequence: AtomicU64::new(0),
        }
    }
}

#[async_trait]
impl ToolReceiptSink for ToolObservationSink {
    async fn persist(&self, receipt: &ToolReceipt) -> Result<(), ToolError> {
        let sequence = self.sequence.fetch_add(1, Ordering::Relaxed);
        let mut observation = ObservationEnvelope::metadata(
            self.producer_id.clone(),
            "llm-tool-runtime",
            "tool-receipt-sink",
            sequence,
            ObservationKind::Tool,
            if receipt.error_class.is_some() {
                LifecycleStatus::Failed
            } else {
                LifecycleStatus::Completed
            },
            format!("tool '{}' completed", receipt.tool_name),
        );
        observation.provenance = Provenance::Canonical;
        observation.correlation = Correlation {
            run_id: Some(receipt.tool_run_id.clone()),
            trace_id: Some(receipt.trace_ctx.trace_id.clone()),
            parent_span_id: receipt.trace_ctx.parent_id.clone(),
            attempt_id: Some(receipt.attempt_id.to_string()),
            trial_id: Some(receipt.trial_id.to_string()),
            request_id: receipt.provider_call_id.clone(),
            ..Correlation::default()
        };
        let _ = self.client.try_emit(observation);
        Ok(())
    }
}
