//! Receipt types for graph execution auditability.
//!
//! Receipts capture the full state of graph execution steps for replay,
//! debugging, and compliance auditing.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

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
    pub outcome: ExecutionOutcome,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExecutionOutcome {
    Completed,
    Partial { failed_step: usize },
    Cancelled,
    InternalError { message: String },
}
