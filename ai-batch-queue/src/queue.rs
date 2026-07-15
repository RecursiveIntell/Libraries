use std::sync::Mutex;
use std::time::Instant;

use stack_ids::TrialId;

use crate::eta::EtaTracker;
use crate::types::*;

/// Mutable scheduling state that tracks resource-switch heuristics.
///
/// These three fields are combined into a single struct so they can be
/// protected by one `Mutex`, preventing the split-lock inconsistency
/// that occurred when each was locked independently (ABQ-2).
struct SchedulingState {
    last_resource_key: Option<String>,
    consecutive_same_key: usize,
    last_resource_switch: Option<Instant>,
}

/// In-memory batch queue with model-aware reordering and ETA estimation.
///
/// The queue automatically groups jobs by `resource_key` to minimize expensive
/// resource swaps (e.g. GPU model loads). It also tracks per-item processing
/// durations bucketed by size for accurate ETA predictions.
pub struct BatchQueue<D>
where
    D: Clone + Send + Sync + serde::Serialize + 'static,
{
    jobs: Mutex<Vec<BatchJob<D>>>,
    pub(crate) eta: EtaTracker,
    scheduling: SchedulingConfig,
    scheduling_state: Mutex<SchedulingState>,
}

impl<D> Default for BatchQueue<D>
where
    D: Clone + Send + Sync + serde::Serialize + 'static,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<D> BatchQueue<D>
where
    D: Clone + Send + Sync + serde::Serialize + 'static,
{
    /// Create a new empty batch queue.
    pub fn new() -> Self {
        Self::with_scheduling(SchedulingConfig::default())
    }

    /// Create a new queue with explicit scheduling controls.
    pub fn with_scheduling(scheduling: SchedulingConfig) -> Self {
        Self {
            jobs: Mutex::new(Vec::new()),
            eta: EtaTracker::new(),
            scheduling,
            scheduling_state: Mutex::new(SchedulingState {
                last_resource_key: None,
                consecutive_same_key: 0,
                last_resource_switch: None,
            }),
        }
    }

    /// Add a new batch job and perform resource-aware reordering.
    /// Returns the assigned job ID.
    pub fn enqueue(&self, mut job: BatchJob<D>) -> anyhow::Result<String> {
        let mut jobs = self.jobs.lock().map_err(|e| anyhow::anyhow!("{}", e))?;

        if job.id.is_empty() {
            // TODO(ID-002): migrate this queue-local generator to stack-ids::random.
            job.id = uuid::Uuid::new_v4().to_string();
        }
        job.status = BatchJobStatus::Queued;
        job.created_at = chrono::Utc::now().to_rfc3339();

        let job_id = job.id.clone();
        jobs.push(job);

        Self::reorder_queued_jobs(&mut jobs, &self.scheduling);
        Ok(job_id)
    }

    /// Reorder only queued jobs to group by resource_key (minimizes resource swaps).
    ///
    /// For example, if you queue jobs for models A, B, A, this reorders to A, A, B
    /// so the GPU only loads each model once instead of switching back and forth.
    fn reorder_queued_jobs(jobs: &mut [BatchJob<D>], scheduling: &SchedulingConfig) {
        if !scheduling.enable_reordering {
            return;
        }

        let queued_indices: Vec<usize> = jobs
            .iter()
            .enumerate()
            .filter(|(_, j)| j.status == BatchJobStatus::Queued)
            .map(|(i, _)| i)
            .collect();

        if queued_indices.len() < 2 {
            return;
        }

        let mut queued_jobs: Vec<BatchJob<D>> =
            queued_indices.iter().map(|&i| jobs[i].clone()).collect();

        let original_order: Vec<String> = queued_jobs.iter().map(|j| j.id.clone()).collect();
        queued_jobs.sort_by(|a, b| a.resource_key.cmp(&b.resource_key));
        let new_order: Vec<String> = queued_jobs.iter().map(|j| j.id.clone()).collect();

        if original_order != new_order {
            for job in &mut queued_jobs {
                job.reordered = true;
                job.reorder_note =
                    Some("Reordered: grouping by resource to minimize swaps".to_string());
            }
            for (slot_idx, job) in queued_indices.iter().zip(queued_jobs) {
                jobs[*slot_idx] = job;
            }
        }
    }

    /// Get the next queued job (without removing it).
    pub fn next_queued(&self) -> Option<BatchJob<D>> {
        let jobs = self.jobs.lock().ok()?;
        let queued: Vec<&BatchJob<D>> = jobs
            .iter()
            .filter(|j| j.status == BatchJobStatus::Queued)
            .collect();
        if queued.is_empty() {
            return None;
        }

        let sched = self
            .scheduling_state
            .lock()
            .unwrap_or_else(|e| e.into_inner());

        if let Some(ref last_key) = sched.last_resource_key {
            let same_resource = queued
                .iter()
                .find(|job| job.resource_key == *last_key)
                .copied();
            let different_resource = queued
                .iter()
                .find(|job| job.resource_key != *last_key)
                .copied();

            if let Some(different_resource) = different_resource {
                let cooldown_active = sched
                    .last_resource_switch
                    .map(|ts| ts.elapsed() < self.scheduling.resource_switch_cooldown)
                    .unwrap_or(false);

                if sched.consecutive_same_key >= self.scheduling.max_consecutive_same_key
                    && !cooldown_active
                {
                    return Some(different_resource.clone());
                }
            }

            if let Some(same_resource) = same_resource {
                return Some(same_resource.clone());
            }
        }

        queued.first().cloned().cloned()
    }

    /// Atomically claim the next queued job for `worker`.
    /// The generation is required for safe completion by the claiming worker.
    pub fn claim_next(
        &self,
        worker: impl Into<String>,
    ) -> anyhow::Result<Option<(BatchJob<D>, u64)>> {
        let worker = worker.into();
        let mut jobs = self
            .jobs
            .lock()
            .map_err(|_| QueueTransitionError::Poisoned)?;
        let Some(job) = jobs.iter_mut().find(|j| j.status == BatchJobStatus::Queued) else {
            return Ok(None);
        };
        job.status = BatchJobStatus::Running;
        job.started_at = Some(chrono::Utc::now().to_rfc3339());
        job.claim_generation = job.claim_generation.saturating_add(1);
        job.claimed_by = Some(worker);
        Ok(Some((job.clone(), job.claim_generation)))
    }

    /// Complete a job only when the worker still owns the exact claim.
    pub fn mark_completed_claimed(
        &self,
        job_id: &str,
        worker: &str,
        generation: u64,
    ) -> anyhow::Result<Option<BatchCompletionSummary>> {
        let jobs = self
            .jobs
            .lock()
            .map_err(|_| QueueTransitionError::Poisoned)?;
        let job = jobs
            .iter()
            .find(|j| j.id == job_id)
            .ok_or_else(|| QueueTransitionError::Missing(job_id.to_string()))?;
        if job.status != BatchJobStatus::Running
            || job.claimed_by.as_deref() != Some(worker)
            || job.claim_generation != generation
        {
            return Err(QueueTransitionError::StaleClaim {
                job: job_id.to_string(),
            }
            .into());
        }
        drop(jobs);
        self.mark_completed(job_id)
    }

    /// Mark a job as running and set its started_at timestamp.
    ///
    /// Both the job status and scheduling metadata are updated while
    /// holding the jobs lock, ensuring they cannot become inconsistent
    /// (ABQ-2 fix: previously the jobs lock was dropped before the
    /// scheduling locks were acquired).
    pub fn mark_running(&self, job_id: &str) -> anyhow::Result<()> {
        let mut jobs = self.jobs.lock().map_err(|e| anyhow::anyhow!("{}", e))?;
        let job = jobs
            .iter_mut()
            .find(|j| j.id == job_id)
            .ok_or_else(|| anyhow::anyhow!("job {} not found", job_id))?;
        if job.status != BatchJobStatus::Queued {
            anyhow::bail!("job {} must be queued before it can run", job_id);
        }

        let resource_key = job.resource_key.clone();
        job.status = BatchJobStatus::Running;
        job.started_at = Some(chrono::Utc::now().to_rfc3339());

        // Update scheduling state while still holding the jobs lock so
        // that job status and scheduling metadata stay consistent.
        let mut sched = self
            .scheduling_state
            .lock()
            .unwrap_or_else(|e| e.into_inner());

        match sched.last_resource_key.as_ref() {
            Some(current) if *current == resource_key => {
                sched.consecutive_same_key += 1;
            }
            _ => {
                sched.last_resource_key = Some(resource_key);
                sched.consecutive_same_key = 1;
                sched.last_resource_switch = Some(Instant::now());
            }
        }
        Ok(())
    }

    /// Update a single item's status within a job.
    ///
    /// If the item completed successfully and `duration_ms` is provided,
    /// the ETA tracker is automatically updated with the new data point.
    pub fn update_item(
        &self,
        job_id: &str,
        item_id: &str,
        status: BatchItemStatus,
        error: Option<String>,
        duration_ms: Option<u64>,
    ) -> anyhow::Result<()> {
        let mut jobs = self.jobs.lock().map_err(|e| anyhow::anyhow!("{}", e))?;
        let job = jobs
            .iter_mut()
            .find(|j| j.id == job_id)
            .ok_or_else(|| anyhow::anyhow!("job {} not found", job_id))?;
        if matches!(
            job.status,
            BatchJobStatus::Completed
                | BatchJobStatus::CompletedWithErrors
                | BatchJobStatus::Cancelled
        ) {
            anyhow::bail!("job {} is no longer mutable", job_id);
        }
        let item = job
            .items
            .iter_mut()
            .find(|i| i.id == item_id)
            .ok_or_else(|| anyhow::anyhow!("item {} not found in job {}", item_id, job_id))?;

        if item.status == BatchItemStatus::Cancelled && status != BatchItemStatus::Cancelled {
            return Ok(());
        }

        match status {
            BatchItemStatus::Running if item.status != BatchItemStatus::Pending => {
                anyhow::bail!("item {} must be pending before running", item_id);
            }
            BatchItemStatus::Cancelled
                if item.status != BatchItemStatus::Pending
                    && item.status != BatchItemStatus::Running =>
            {
                anyhow::bail!(
                    "item {} cannot be cancelled from {:?}",
                    item_id,
                    item.status
                );
            }
            BatchItemStatus::Completed | BatchItemStatus::Failed | BatchItemStatus::Skipped
                if item.status != BatchItemStatus::Running
                    && item.status != BatchItemStatus::Pending =>
            {
                anyhow::bail!(
                    "item {} cannot transition from {:?} to {:?}",
                    item_id,
                    item.status,
                    status
                );
            }
            BatchItemStatus::Pending => {
                anyhow::bail!("item {} cannot be moved back to pending directly", item_id);
            }
            _ => {}
        }

        let should_record = status == BatchItemStatus::Completed && duration_ms.is_some();
        let resource_key = job.resource_key.clone();
        let operation = job.operation.clone();
        let bucket = item.size_bucket;

        item.status = status;
        item.error = error;
        item.duration_ms = duration_ms;

        if should_record {
            let ms = duration_ms.unwrap();
            drop(jobs); // Release jobs lock before eta lock
            self.eta.record(&resource_key, &operation, bucket, ms);
        }
        Ok(())
    }

    /// Mark a job as completed and produce a completion summary.
    ///
    /// Automatically determines whether it's `Completed` or `CompletedWithErrors`
    /// based on item statuses.
    pub fn mark_completed(&self, job_id: &str) -> anyhow::Result<Option<BatchCompletionSummary>> {
        let mut jobs = self.jobs.lock().map_err(|e| anyhow::anyhow!("{}", e))?;
        let job = jobs
            .iter_mut()
            .find(|j| j.id == job_id)
            .ok_or_else(|| anyhow::anyhow!("job {} not found", job_id))?;
        if job.status != BatchJobStatus::Running && job.status != BatchJobStatus::Cancelled {
            anyhow::bail!(
                "job {} must be running or cancelled before completion",
                job_id
            );
        }

        let failed = job
            .items
            .iter()
            .filter(|i| i.status == BatchItemStatus::Failed)
            .count();
        let succeeded = job
            .items
            .iter()
            .filter(|i| i.status == BatchItemStatus::Completed)
            .count();
        let skipped = job
            .items
            .iter()
            .filter(|i| {
                i.status == BatchItemStatus::Cancelled || i.status == BatchItemStatus::Skipped
            })
            .count();

        if job.status == BatchJobStatus::Cancelled
            || job
                .items
                .iter()
                .all(|item| item.status == BatchItemStatus::Cancelled)
        {
            job.status = BatchJobStatus::Cancelled;
        } else if failed > 0 {
            job.status = BatchJobStatus::CompletedWithErrors;
        } else {
            job.status = BatchJobStatus::Completed;
        }
        job.completed_at = Some(chrono::Utc::now().to_rfc3339());

        let total_ms: u64 = job.items.iter().filter_map(|i| i.duration_ms).sum();
        let processed = succeeded + failed;
        let avg_ms = if processed > 0 {
            total_ms / processed as u64
        } else {
            0
        };

        Ok(Some(BatchCompletionSummary {
            job_id: job.id.clone(),
            operation: job.operation.clone(),
            resource_key: job.resource_key.clone(),
            total: job.items.len(),
            succeeded,
            failed,
            skipped,
            total_duration_ms: total_ms,
            avg_duration_ms: avg_ms,
        }))
    }

    /// Cancel a single pending item within a job.
    pub fn cancel_item(&self, job_id: &str, item_id: &str) -> anyhow::Result<()> {
        let mut jobs = self.jobs.lock().map_err(|e| anyhow::anyhow!("{}", e))?;
        let job = jobs
            .iter_mut()
            .find(|j| j.id == job_id)
            .ok_or_else(|| anyhow::anyhow!("job {} not found", job_id))?;
        let item = job
            .items
            .iter_mut()
            .find(|i| i.id == item_id)
            .ok_or_else(|| anyhow::anyhow!("item {} not found in job {}", item_id, job_id))?;

        if item.status == BatchItemStatus::Completed || item.status == BatchItemStatus::Failed {
            anyhow::bail!("item {} is already finalized", item_id);
        }
        item.status = BatchItemStatus::Cancelled;
        Ok(())
    }

    /// Cancel an entire batch job. Running items finish; pending items are cancelled.
    pub fn cancel_job(&self, job_id: &str) -> anyhow::Result<()> {
        let mut jobs = self.jobs.lock().map_err(|e| anyhow::anyhow!("{}", e))?;
        let job = jobs
            .iter_mut()
            .find(|j| j.id == job_id)
            .ok_or_else(|| anyhow::anyhow!("job {} not found", job_id))?;
        for item in &mut job.items {
            if item.status == BatchItemStatus::Pending || item.status == BatchItemStatus::Running {
                item.status = BatchItemStatus::Cancelled;
            }
        }
        job.status = BatchJobStatus::Cancelled;
        job.completed_at = Some(chrono::Utc::now().to_rfc3339());
        Ok(())
    }

    /// Stamp a fresh `TrialId` on a batch item before execution.
    ///
    /// Called by the executor immediately before processing each item.
    /// Each concrete execution gets a unique `TrialId` for diagnostics
    /// correlation. Silently no-ops if the job or item is not found
    /// (the executor will detect this via status checks).
    pub(crate) fn stamp_trial_id(&self, job_id: &str, item_id: &str) {
        if let Ok(mut jobs) = self.jobs.lock() {
            if let Some(job) = jobs.iter_mut().find(|j| j.id == job_id) {
                if let Some(item) = job.items.iter_mut().find(|i| i.id == item_id) {
                    item.trial_id = Some(TrialId::generate());
                }
            }
        }
    }

    /// Retry all failed items in a completed job by resetting them to Pending.
    /// The job is re-queued and reordering is applied.
    ///
    /// Retry lineage: each retried item keeps its existing `attempt_id`
    /// (same logical retry family) but clears its `trial_id` so the executor
    /// will mint a fresh one on the next execution. This keeps batch-item
    /// retries distinguishable from outer job-level retries.
    pub fn retry_failed(&self, job_id: &str) -> anyhow::Result<()> {
        let mut jobs = self.jobs.lock().map_err(|e| anyhow::anyhow!("{}", e))?;
        if let Some(job) = jobs.iter_mut().find(|j| j.id == job_id) {
            let has_failed = job
                .items
                .iter()
                .any(|i| i.status == BatchItemStatus::Failed);
            if !has_failed {
                anyhow::bail!("No failed items to retry in job {}", job_id);
            }
            for item in &mut job.items {
                if item.status == BatchItemStatus::Failed {
                    item.status = BatchItemStatus::Pending;
                    item.error = None;
                    item.duration_ms = None;
                    // Clear trial_id so the executor mints a fresh one.
                    // Keep attempt_id stable (same retry family).
                    item.trial_id = None;
                }
            }
            job.status = BatchJobStatus::Queued;
            job.completed_at = None;
            Self::reorder_queued_jobs(&mut jobs, &self.scheduling);
        }
        Ok(())
    }

    /// Get all jobs (cloned snapshot).
    pub fn list_jobs(&self) -> Vec<BatchJob<D>> {
        self.jobs.lock().map(|j| j.clone()).unwrap_or_default()
    }

    /// Get a specific job by ID.
    pub fn get_job(&self, job_id: &str) -> Option<BatchJob<D>> {
        self.jobs
            .lock()
            .ok()?
            .iter()
            .find(|j| j.id == job_id)
            .cloned()
    }

    /// Estimate remaining processing time for a job in milliseconds.
    /// Returns `None` if no historical data is available.
    pub fn estimate_remaining_ms(&self, job_id: &str) -> Option<u64> {
        self.estimate_remaining(job_id)
            .map(|estimate| estimate.remaining_ms)
    }

    /// Estimate remaining processing time with confidence and sample metadata.
    pub fn estimate_remaining(&self, job_id: &str) -> Option<EtaEstimate> {
        let jobs = self.jobs.lock().ok()?;
        let job = jobs.iter().find(|j| j.id == job_id)?;

        let remaining_buckets: Vec<SizeBucket> = job
            .items
            .iter()
            .filter(|i| {
                i.status == BatchItemStatus::Pending || i.status == BatchItemStatus::Running
            })
            .map(|i| i.size_bucket)
            .collect();

        if remaining_buckets.is_empty() {
            return Some(EtaEstimate {
                remaining_ms: 0,
                items_remaining: 0,
                avg_item_ms: 0,
                confidence: EtaConfidence::High,
                sample_count: 0,
            });
        }

        self.eta
            .estimate(&job.resource_key, &job.operation, &remaining_buckets)
    }

    /// Check if any batch job is currently running.
    pub fn has_running_job(&self) -> bool {
        self.jobs
            .lock()
            .map(|j| j.iter().any(|job| job.status == BatchJobStatus::Running))
            .unwrap_or(false)
    }

    /// Get the number of ETA samples for a specific resource/operation/size combination.
    pub fn eta_sample_count(
        &self,
        resource_key: &str,
        operation: &str,
        size_bucket: SizeBucket,
    ) -> u64 {
        self.eta.sample_count(resource_key, operation, size_bucket)
    }

    /// Get the number of queued (waiting) jobs.
    pub fn queued_count(&self) -> usize {
        self.jobs
            .lock()
            .map(|j| {
                j.iter()
                    .filter(|job| job.status == BatchJobStatus::Queued)
                    .count()
            })
            .unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_items(count: usize) -> Vec<BatchItem<String>> {
        (0..count)
            .map(|i| BatchItem {
                id: format!("item-{}", i),
                data: format!("data-{}", i),
                status: BatchItemStatus::Pending,
                error: None,
                duration_ms: None,
                size_bucket: SizeBucket::Medium,
                trace_ctx: None,
                attempt_id: None,
                trial_id: None,
            })
            .collect()
    }

    fn make_job(resource: &str, op: &str, count: usize) -> BatchJob<String> {
        BatchJob {
            id: String::new(),
            resource_key: resource.to_string(),
            operation: op.to_string(),
            overwrite_policy: OverwritePolicy::Skip,
            items: make_items(count),
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

    #[test]
    fn test_enqueue_assigns_id() {
        let queue: BatchQueue<String> = BatchQueue::new();
        let job = make_job("model-a", "tag", 3);
        let id = queue.enqueue(job).unwrap();
        assert!(!id.is_empty());
    }

    #[test]
    fn test_next_queued() {
        let queue: BatchQueue<String> = BatchQueue::new();
        assert!(queue.next_queued().is_none());

        let job = make_job("model-a", "tag", 2);
        let id = queue.enqueue(job).unwrap();

        let next = queue.next_queued().unwrap();
        assert_eq!(next.id, id);
    }

    #[test]
    fn test_mark_running() {
        let queue: BatchQueue<String> = BatchQueue::new();
        let id = queue.enqueue(make_job("model-a", "tag", 1)).unwrap();

        queue.mark_running(&id).unwrap();
        let job = queue.get_job(&id).unwrap();
        assert_eq!(job.status, BatchJobStatus::Running);
        assert!(job.started_at.is_some());
    }

    #[test]
    fn test_update_item_and_complete() {
        let queue: BatchQueue<String> = BatchQueue::new();
        let id = queue.enqueue(make_job("model-a", "tag", 2)).unwrap();
        queue.mark_running(&id).unwrap();

        queue
            .update_item(&id, "item-0", BatchItemStatus::Completed, None, Some(1000))
            .unwrap();
        queue
            .update_item(&id, "item-1", BatchItemStatus::Completed, None, Some(2000))
            .unwrap();

        let summary = queue.mark_completed(&id).unwrap().unwrap();
        assert_eq!(summary.succeeded, 2);
        assert_eq!(summary.failed, 0);
        assert_eq!(summary.total_duration_ms, 3000);
        assert_eq!(summary.avg_duration_ms, 1500);
    }

    #[test]
    fn test_completed_with_errors() {
        let queue: BatchQueue<String> = BatchQueue::new();
        let id = queue.enqueue(make_job("model-a", "tag", 2)).unwrap();
        queue.mark_running(&id).unwrap();

        queue
            .update_item(&id, "item-0", BatchItemStatus::Completed, None, Some(1000))
            .unwrap();
        queue
            .update_item(
                &id,
                "item-1",
                BatchItemStatus::Failed,
                Some("timeout".to_string()),
                Some(5000),
            )
            .unwrap();

        let summary = queue.mark_completed(&id).unwrap().unwrap();
        assert_eq!(summary.succeeded, 1);
        assert_eq!(summary.failed, 1);

        let job = queue.get_job(&id).unwrap();
        assert_eq!(job.status, BatchJobStatus::CompletedWithErrors);
    }

    #[test]
    fn test_cancel_job() {
        let queue: BatchQueue<String> = BatchQueue::new();
        let id = queue.enqueue(make_job("model-a", "tag", 3)).unwrap();

        queue.cancel_job(&id).unwrap();
        let job = queue.get_job(&id).unwrap();
        assert_eq!(job.status, BatchJobStatus::Cancelled);
        assert!(job
            .items
            .iter()
            .all(|i| i.status == BatchItemStatus::Cancelled));
    }

    #[test]
    fn test_cancel_single_item() {
        let queue: BatchQueue<String> = BatchQueue::new();
        let id = queue.enqueue(make_job("model-a", "tag", 3)).unwrap();

        queue.cancel_item(&id, "item-1").unwrap();
        let job = queue.get_job(&id).unwrap();
        assert_eq!(job.items[0].status, BatchItemStatus::Pending);
        assert_eq!(job.items[1].status, BatchItemStatus::Cancelled);
        assert_eq!(job.items[2].status, BatchItemStatus::Pending);
    }

    #[test]
    fn test_retry_failed() {
        let queue: BatchQueue<String> = BatchQueue::new();
        let id = queue.enqueue(make_job("model-a", "tag", 2)).unwrap();
        queue.mark_running(&id).unwrap();

        queue
            .update_item(&id, "item-0", BatchItemStatus::Completed, None, Some(1000))
            .unwrap();
        queue
            .update_item(
                &id,
                "item-1",
                BatchItemStatus::Failed,
                Some("err".to_string()),
                None,
            )
            .unwrap();
        queue.mark_completed(&id).unwrap();

        queue.retry_failed(&id).unwrap();
        let job = queue.get_job(&id).unwrap();
        assert_eq!(job.status, BatchJobStatus::Queued);
        assert_eq!(job.items[1].status, BatchItemStatus::Pending);
        assert!(job.items[1].error.is_none());
    }

    #[test]
    fn test_model_aware_reordering() {
        let queue: BatchQueue<String> = BatchQueue::new();
        queue.enqueue(make_job("model-b", "tag", 1)).unwrap();
        queue.enqueue(make_job("model-a", "caption", 1)).unwrap();
        queue.enqueue(make_job("model-b", "caption", 1)).unwrap();

        let jobs = queue.list_jobs();
        // After reordering: model-a first, then model-b jobs
        assert_eq!(jobs[0].resource_key, "model-a");
        assert_eq!(jobs[1].resource_key, "model-b");
        assert_eq!(jobs[2].resource_key, "model-b");
    }

    #[test]
    fn test_reorder_preserves_running_jobs() {
        let queue: BatchQueue<String> = BatchQueue::new();
        let id1 = queue.enqueue(make_job("model-b", "tag", 1)).unwrap();
        queue.mark_running(&id1).unwrap();

        // Running job should not be reordered
        queue.enqueue(make_job("model-a", "tag", 1)).unwrap();
        queue.enqueue(make_job("model-b", "tag", 1)).unwrap();

        let jobs = queue.list_jobs();
        assert_eq!(jobs[0].resource_key, "model-b"); // running, stays first
        assert_eq!(jobs[0].status, BatchJobStatus::Running);
        // Queued jobs reordered: model-a before model-b
        assert_eq!(jobs[1].resource_key, "model-a");
        assert_eq!(jobs[2].resource_key, "model-b");
    }

    #[test]
    fn test_list_and_count() {
        let queue: BatchQueue<String> = BatchQueue::new();
        assert_eq!(queue.queued_count(), 0);
        assert!(!queue.has_running_job());

        let id = queue.enqueue(make_job("model-a", "tag", 1)).unwrap();
        assert_eq!(queue.queued_count(), 1);

        queue.mark_running(&id).unwrap();
        assert!(queue.has_running_job());
        assert_eq!(queue.queued_count(), 0);
    }

    #[test]
    fn test_eta_integration() {
        let queue: BatchQueue<String> = BatchQueue::new();
        let id = queue.enqueue(make_job("model-a", "tag", 3)).unwrap();
        queue.mark_running(&id).unwrap();

        // No ETA data yet
        assert!(queue.estimate_remaining_ms(&id).is_none());

        // Complete first item with timing
        queue
            .update_item(&id, "item-0", BatchItemStatus::Completed, None, Some(1000))
            .unwrap();

        // Now we have data: 2 remaining items * 1000ms avg = 2000ms
        let eta = queue.estimate_remaining_ms(&id);
        assert_eq!(eta, Some(2000));
    }

    #[test]
    fn test_stamp_trial_id() {
        let queue: BatchQueue<String> = BatchQueue::new();
        let id = queue.enqueue(make_job("model-a", "tag", 2)).unwrap();

        // Items start with no trial_id
        let job = queue.get_job(&id).unwrap();
        assert!(job.items[0].trial_id.is_none());

        // Stamp a trial_id
        queue.stamp_trial_id(&id, "item-0");
        let job = queue.get_job(&id).unwrap();
        assert!(job.items[0].trial_id.is_some());
        assert!(job.items[1].trial_id.is_none());

        // Stamping again produces a different trial_id
        let first_trial = job.items[0].trial_id.clone().unwrap();
        queue.stamp_trial_id(&id, "item-0");
        let job = queue.get_job(&id).unwrap();
        let second_trial = job.items[0].trial_id.clone().unwrap();
        assert_ne!(first_trial, second_trial);
    }

    #[test]
    fn test_retry_clears_trial_preserves_attempt() {
        use stack_ids::AttemptId;

        let queue: BatchQueue<String> = BatchQueue::new();
        let id = queue.enqueue(make_job("model-a", "tag", 2)).unwrap();
        queue.mark_running(&id).unwrap();

        // Manually set attempt_id and trial_id on item-1 to simulate traced execution
        {
            let mut jobs = queue.jobs.lock().unwrap();
            let job = jobs.iter_mut().find(|j| j.id == id).unwrap();
            job.items[1].attempt_id = Some(AttemptId::generate());
            job.items[1].trial_id = Some(TrialId::generate());
        }

        // Complete item-0, fail item-1
        queue
            .update_item(&id, "item-0", BatchItemStatus::Completed, None, Some(1000))
            .unwrap();
        queue
            .update_item(
                &id,
                "item-1",
                BatchItemStatus::Failed,
                Some("err".to_string()),
                None,
            )
            .unwrap();
        queue.mark_completed(&id).unwrap();

        // Capture the attempt_id before retry
        let attempt_before = queue.get_job(&id).unwrap().items[1].attempt_id.clone();
        assert!(attempt_before.is_some());

        // Retry
        queue.retry_failed(&id).unwrap();
        let job = queue.get_job(&id).unwrap();

        // attempt_id preserved, trial_id cleared
        assert_eq!(job.items[1].attempt_id, attempt_before);
        assert!(job.items[1].trial_id.is_none());
    }
}
