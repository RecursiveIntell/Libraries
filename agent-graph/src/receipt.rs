//! Receipt types for graph execution auditability.
//!
//! Receipts capture the full state of graph execution steps for replay,
//! debugging, and compliance auditing.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// Reference to semantic-memory generation/candidate provenance that influenced graph context.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GraphMemoryGenerationRefV1 {
    pub memory_backend: String,
    pub candidate_backend: Option<String>,
    pub generation_id: Option<String>,
    pub embedding_snapshot_digest: Option<String>,
    pub manifest_digest: Option<String>,
    pub exact_rerank: bool,
    pub fallback: Option<String>,
}

/// Digests the full state of a graph execution step for auditability.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepExecutionReceiptV1 {
    pub step_index: usize,
    pub agent_id: String,
    pub started_at: DateTime<Utc>,
    pub finished_at: DateTime<Utc>,
    pub input_digest: String,
    pub output_digest: String,
    pub tool_calls: Vec<ToolCallReceipt>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallReceipt {
    pub tool_name: String,
    pub arguments_digest: String,
    pub result_digest: String,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphExecutionReceiptV1 {
    pub graph_id: String,
    pub execution_id: String,
    pub started_at: DateTime<Utc>,
    pub finished_at: DateTime<Utc>,
    pub steps: Vec<StepExecutionReceiptV1>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub memory_generations: Vec<GraphMemoryGenerationRefV1>,
    /// Last in-memory state when a recoverable partial run could not persist.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recovery_state: Option<HashMap<String, Value>>,
    pub outcome: ExecutionOutcome,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExecutionOutcome {
    Completed,
    Partial { failed_step: usize },
    Cancelled,
    InternalError { message: String },
}

/// A deterministic state transition recorded for one executed node.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StepStateDeltaV1 {
    pub step_index: usize,
    pub node_name: String,
    pub state_before: HashMap<String, Value>,
    pub state_after: HashMap<String, Value>,
    pub input_digest: String,
    pub output_digest: String,
}

/// Recorded request and response replacing a tool dependency during replay.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ToolCallEnvelopeV1 {
    pub step_index: usize,
    pub tool_name: String,
    pub request: Value,
    pub response: Value,
    pub request_digest: String,
    pub response_digest: String,
}

/// Recorded query and result replacing a memory dependency during replay.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MemoryReadEnvelopeV1 {
    pub step_index: usize,
    pub backend: String,
    pub query: Value,
    pub result: Value,
    pub query_digest: String,
    pub result_digest: String,
}

/// Complete offline replay artifact for one graph execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunBundleV1 {
    pub graph_spec: crate::graph::GraphSpecV1,
    pub input_state: HashMap<String, Value>,
    pub step_state_deltas: Vec<StepStateDeltaV1>,
    pub tool_call_envelopes: Vec<ToolCallEnvelopeV1>,
    pub memory_read_envelopes: Vec<MemoryReadEnvelopeV1>,
    pub terminal_receipt: GraphExecutionReceiptV1,
}

/// Successful replay verification summary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplayVerification {
    pub steps_verified: usize,
    pub final_state_digest: String,
}

/// A localized deterministic replay failure.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum ReplayError {
    #[error("replay graph digest mismatch: expected {expected}, got {actual}")]
    GraphMismatch { expected: String, actual: String },
    #[error("replay diverged at step {step_index} ({node_name}): {reason}")]
    StepDivergence {
        step_index: usize,
        node_name: String,
        reason: String,
    },
    #[error("replay envelope for step {step_index} diverged: {reason}")]
    EnvelopeDivergence { step_index: usize, reason: String },
    #[error("terminal receipt diverged: {reason}")]
    TerminalDivergence { reason: String },
}

pub(crate) fn digest_value(value: &Value) -> String {
    fn canonicalize(value: &Value) -> Value {
        match value {
            Value::Object(map) => {
                let mut keys: Vec<_> = map.keys().collect();
                keys.sort();
                let mut result = serde_json::Map::new();
                for key in keys {
                    result.insert(key.clone(), canonicalize(&map[key]));
                }
                Value::Object(result)
            }
            Value::Array(values) => Value::Array(values.iter().map(canonicalize).collect()),
            other => other.clone(),
        }
    }

    let bytes = serde_json::to_vec(&canonicalize(value)).expect("JSON values serialize");
    format!("blake3:{}", blake3::hash(&bytes).to_hex())
}

pub(crate) fn digest_state(state: &HashMap<String, Value>) -> String {
    digest_value(&serde_json::to_value(state).expect("state values serialize"))
}
