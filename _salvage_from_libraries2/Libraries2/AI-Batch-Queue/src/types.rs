use serde::{Deserialize, Serialize};
use stack_ids::{AttemptId, TraceCtx, TrialId};
use std::time::Duration;

/// Scheduling configuration for batch queue fairness.
///
/// Controls how consecutive same-resource jobs are limited to prevent
/// starvation of other resources.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulingConfig {
    /// Maximum consecutive jobs for the same resource key before yielding
    /// to a different resource. Default: 3.
    pub max_consecutive_same_key: usize,
    /// Cooldown duration between resource switches. Default: 0 (none).
    #[serde(with = "duration_millis")]
    pub resource_switch_cooldown: Duration,
    /// Whether to enable model-aware reordering. Default: true.
    pub enable_reordering: bool,
}

impl Default for SchedulingConfig {
    fn default() -> Self {
        Self {
            max_consecutive_same_key: 3,
            resource_switch_cooldown: Duration::ZERO,
            enable_reordering: true,
        }
    }
}

/// Confidence level of an ETA estimate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum EtaConfidence {
    /// No historical data; estimate is a rough guess.
    Low,
    /// Some data points but not enough for high confidence.
    Medium,
    /// Sufficient data points (>= 10) for a reliable estimate.
    High,
}

/// Structured ETA estimate with confidence indication.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EtaEstimate {
    /// Estimated remaining time in milliseconds.
    pub remaining_ms: u64,
    /// Number of items remaining.
    pub items_remaining: usize,
    /// Average per-item duration in milliseconds (based on history).
    pub avg_item_ms: u64,
    /// Confidence level of the estimate.
    pub confidence: EtaConfidence,
    /// Number of historical data points used for the estimate.
    pub sample_count: u64,
}

/// Serde helper for Duration as milliseconds.
mod duration_millis {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::time::Duration;

    pub fn serialize<S: Serializer>(d: &Duration, s: S) -> Result<S::Ok, S::Error> {
        d.as_millis().serialize(s)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Duration, D::Error> {
        let ms = u64::deserialize(d)?;
        Ok(Duration::from_millis(ms))
    }
}

/// Per-item status within a batch job.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum BatchItemStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Skipped,
    Cancelled,
}

/// Overall batch job status.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum BatchJobStatus {
    Queued,
    Running,
    Completed,
    CompletedWithErrors,
    Cancelled,
}

/// Overwrite policy for batch operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum OverwritePolicy {
    /// Skip items that already have results.
    Skip,
    /// Overwrite existing results.
    Overwrite,
}

/// Size bucket for ETA estimation — groups items by processing complexity.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SizeBucket {
    Small,
    Medium,
    Large,
    Unknown,
}

impl SizeBucket {
    /// Classify by pixel count. Thresholds: <500K = Small, <2M = Medium, else Large.
    pub fn from_pixel_count(pixels: u64) -> Self {
        if pixels < 500_000 {
            Self::Small
        } else if pixels < 2_000_000 {
            Self::Medium
        } else {
            Self::Large
        }
    }

    /// Classify from optional width/height dimensions.
    pub fn from_dimensions(width: Option<u32>, height: Option<u32>) -> Self {
        match (width, height) {
            (Some(w), Some(h)) => Self::from_pixel_count(w as u64 * h as u64),
            _ => Self::Unknown,
        }
    }
}

/// A single item within a batch job.
///
/// The `data` field carries user-defined per-item payload (e.g. file path,
/// image ID, document reference). It must be serializable for event emission.
///
/// ## Trace context
///
/// Optional `trace_ctx` propagates cross-crate observability context from
/// `stack-ids`. The `attempt_id` identifies the logical retry family for this
/// batch-item (distinct from any outer job-level retry). The `trial_id` tracks
/// each concrete execution attempt within that family.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchItem<D>
where
    D: Clone + Send + Sync + Serialize,
{
    /// Unique identifier for this item within the batch.
    pub id: String,
    /// User-defined data payload.
    pub data: D,
    /// Current processing status.
    pub status: BatchItemStatus,
    /// Error message if status is Failed.
    pub error: Option<String>,
    /// Processing duration in milliseconds (set after completion).
    pub duration_ms: Option<u64>,
    /// Size bucket for ETA estimation.
    pub size_bucket: SizeBucket,
    /// Cross-crate trace context for observability propagation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trace_ctx: Option<TraceCtx>,
    /// Logical retry family for this batch-item.
    ///
    /// One `AttemptId` per retry-owner boundary. Batch-item retries (via
    /// `retry_failed`) produce new `TrialId`s under the same `AttemptId`,
    /// keeping them distinguishable from outer job-level retries.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attempt_id: Option<AttemptId>,
    /// Concrete execution identifier within the current retry family.
    ///
    /// Minted fresh on each processing attempt. Allows correlating logs,
    /// metrics, and errors to a specific execution without ambiguity.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trial_id: Option<TrialId>,
}

/// A batch job containing multiple items processed with the same resource.
///
/// The "resource key" (e.g. model name) is used for intelligent reordering:
/// jobs sharing a resource are grouped together to minimize expensive
/// resource swaps (GPU model loads, connection setup, etc.).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchJob<D>
where
    D: Clone + Send + Sync + Serialize,
{
    /// Unique job identifier.
    pub id: String,
    /// The resource key (e.g. model name) used for grouping.
    pub resource_key: String,
    /// Human-readable operation label (e.g. "tag", "caption", "embed").
    pub operation: String,
    /// Overwrite policy for items that already have results.
    pub overwrite_policy: OverwritePolicy,
    /// The items in this batch.
    pub items: Vec<BatchItem<D>>,
    /// Current job status.
    pub status: BatchJobStatus,
    /// ISO 8601 timestamp when the job was created.
    pub created_at: String,
    /// ISO 8601 timestamp when processing started.
    pub started_at: Option<String>,
    /// ISO 8601 timestamp when processing finished.
    pub completed_at: Option<String>,
    /// Whether the job was reordered for resource optimization.
    pub reordered: bool,
    /// Human-readable note explaining the reorder.
    pub reorder_note: Option<String>,
}

/// Summary of a completed batch job.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchCompletionSummary {
    pub job_id: String,
    pub operation: String,
    pub resource_key: String,
    pub total: usize,
    pub succeeded: usize,
    pub failed: usize,
    pub skipped: usize,
    pub total_duration_ms: u64,
    pub avg_duration_ms: u64,
}

/// Result of processing a single batch item.
#[derive(Debug, Clone)]
pub struct ItemResult {
    /// Whether the item was processed successfully.
    pub success: bool,
    /// Optional output data (e.g. generated tags, captions).
    pub output: Option<String>,
    /// Error message if processing failed.
    pub error: Option<String>,
}

impl ItemResult {
    pub fn success() -> Self {
        Self {
            success: true,
            output: None,
            error: None,
        }
    }

    pub fn success_with_output(output: String) -> Self {
        Self {
            success: true,
            output: Some(output),
            error: None,
        }
    }

    pub fn failure(error: String) -> Self {
        Self {
            success: false,
            output: None,
            error: Some(error),
        }
    }
}
