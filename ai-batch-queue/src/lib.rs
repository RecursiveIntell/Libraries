//! # AI Batch Queue
//!
//! Model-aware batch processing queue with ETA estimation for Tauri applications.
//!
//! ## Key Features
//!
//! - **Resource-aware reordering** — automatically groups jobs by resource key
//!   (e.g. model name) to minimize expensive swaps
//! - **Size-bucketed ETA estimation** — tracks processing durations by
//!   (resource, operation, size) for accurate time predictions
//! - **Item-level status tracking** — each item has its own lifecycle
//! - **Overwrite policies** — skip items that already have results
//! - **Progressive completion with retry** — failed items can be retried
//!   without re-processing successful ones
//! - **Trace context propagation** — optional `stack-ids` `TraceCtx`,
//!   `AttemptId`, and `TrialId` on each batch item for cross-crate
//!   observability and retry lineage tracking
//!
//! ## Quick Start
//!
//! 1. Define your item data type
//! 2. Implement [`BatchItemHandler`] for your processing logic
//! 3. Create a [`BatchQueue`] and register it in Tauri state
//! 4. Call [`executor::spawn()`] to start the background processor

pub mod eta;
pub mod executor;
pub mod queue;
pub mod types;

pub use queue::BatchQueue;
pub use types::{
    BatchCompletionSummary, BatchItem, BatchItemStatus, BatchJob, BatchJobStatus, EtaConfidence,
    EtaEstimate, ItemResult, OverwritePolicy, SchedulingConfig, SizeBucket,
};

/// Trait for processing individual items in a batch.
///
/// Implement this for your application to define:
/// - How to process each item (`process`)
/// - Whether an item should be skipped (`should_skip`)
///
/// # Type Parameter
///
/// `D` is the per-item data type (e.g. a file path, image reference, document ID).
///
/// # Example
///
/// ```ignore
/// use ai_batch_queue::*;
///
/// struct MyProcessor;
///
/// impl BatchItemHandler<String> for MyProcessor {
///     async fn process(
///         &self,
///         data: &String,
///         resource_key: &str,
///         operation: &str,
///     ) -> anyhow::Result<ItemResult> {
///         println!("Processing {} with {}", data, resource_key);
///         Ok(ItemResult::success())
///     }
///
///     fn should_skip(&self, data: &String, operation: &str) -> bool {
///         false // never skip
///     }
/// }
/// ```
pub trait BatchItemHandler<D>: Send + Sync + 'static
where
    D: Clone + Send + Sync + serde::Serialize,
{
    /// Process a single item.
    ///
    /// # Arguments
    /// * `data` — the item's user-defined data payload
    /// * `resource_key` — the resource this batch uses (e.g. model name)
    /// * `operation` — the operation label (e.g. "tag", "caption")
    fn process(
        &self,
        data: &D,
        resource_key: &str,
        operation: &str,
    ) -> impl std::future::Future<Output = anyhow::Result<ItemResult>> + Send;

    /// Check if this item should be skipped when the overwrite policy is `Skip`.
    ///
    /// Return `true` to skip (item already has results).
    /// Default implementation never skips.
    fn should_skip(&self, _data: &D, _operation: &str) -> bool {
        false
    }
}

/// Trait for persisting batch queue state.
///
/// The default [`BatchQueue`] is in-memory only. Implement this trait to
/// persist jobs across restarts (e.g., to SQLite or a file).
pub trait BatchStore<D>: Send + Sync
where
    D: Clone + Send + Sync + serde::Serialize + serde::de::DeserializeOwned + 'static,
{
    /// Save or update a batch job.
    fn save_job(&self, job: &BatchJob<D>) -> anyhow::Result<()>;

    /// Load all jobs (for startup recovery).
    fn load_all(&self) -> anyhow::Result<Vec<BatchJob<D>>>;

    /// Delete a completed/cancelled job.
    fn delete_job(&self, job_id: &str) -> anyhow::Result<()>;
}

/// Helper to build a [`BatchJob`] from a list of items.
///
/// Trace fields (`trace_ctx`, `attempt_id`, `trial_id`) are initialized to
/// `None`. Callers that need trace propagation should set them after
/// construction or use [`build_job_traced`].
///
/// # Example
///
/// ```
/// use ai_batch_queue::*;
///
/// let job = build_job(
///     "llava:13b",
///     "tag",
///     OverwritePolicy::Skip,
///     vec![
///         ("img-1".to_string(), "path/to/1.png".to_string(), SizeBucket::Medium),
///         ("img-2".to_string(), "path/to/2.png".to_string(), SizeBucket::Large),
///     ],
/// );
///
/// assert_eq!(job.items.len(), 2);
/// assert_eq!(job.resource_key, "llava:13b");
/// ```
pub fn build_job<D>(
    resource_key: &str,
    operation: &str,
    overwrite_policy: OverwritePolicy,
    items: Vec<(String, D, SizeBucket)>,
) -> BatchJob<D>
where
    D: Clone + Send + Sync + serde::Serialize,
{
    let batch_items = items
        .into_iter()
        .map(|(id, data, bucket)| BatchItem {
            id,
            data,
            status: BatchItemStatus::Pending,
            error: None,
            duration_ms: None,
            size_bucket: bucket,
            trace_ctx: None,
            attempt_id: None,
            trial_id: None,
        })
        .collect();

    BatchJob {
        id: String::new(),
        resource_key: resource_key.to_string(),
        operation: operation.to_string(),
        overwrite_policy,
        items: batch_items,
        status: BatchJobStatus::Queued,
        created_at: String::new(),
        started_at: None,
        completed_at: None,
        reordered: false,
        reorder_note: None,
        claim_generation: 0,
        claimed_by: None,
    }
}

/// Helper to build a [`BatchJob`] with trace context propagation.
///
/// Each item receives the provided `trace_ctx` and a freshly generated
/// `AttemptId` (one per item, since each item is its own retry-owner
/// boundary within the batch). `TrialId` is left `None` — it will be
/// stamped by the executor on each concrete execution.
pub fn build_job_traced<D>(
    resource_key: &str,
    operation: &str,
    overwrite_policy: OverwritePolicy,
    items: Vec<(String, D, SizeBucket)>,
    trace_ctx: stack_ids::TraceCtx,
) -> BatchJob<D>
where
    D: Clone + Send + Sync + serde::Serialize,
{
    let batch_items = items
        .into_iter()
        .map(|(id, data, bucket)| BatchItem {
            id,
            data,
            status: BatchItemStatus::Pending,
            error: None,
            duration_ms: None,
            size_bucket: bucket,
            trace_ctx: Some(trace_ctx.clone()),
            attempt_id: Some(stack_ids::AttemptId::generate()),
            trial_id: None,
        })
        .collect();

    BatchJob {
        id: String::new(),
        resource_key: resource_key.to_string(),
        operation: operation.to_string(),
        overwrite_policy,
        items: batch_items,
        status: BatchJobStatus::Queued,
        created_at: String::new(),
        started_at: None,
        completed_at: None,
        reordered: false,
        reorder_note: None,
        claim_generation: 0,
        claimed_by: None,
    }
}
