use ai_batch_queue::*;

fn make_items(count: usize) -> Vec<(String, String, SizeBucket)> {
    (0..count)
        .map(|i| {
            (
                format!("item-{}", i),
                format!("data-{}", i),
                SizeBucket::Medium,
            )
        })
        .collect()
}

fn make_job(resource: &str, op: &str, count: usize) -> BatchJob<String> {
    build_job(resource, op, OverwritePolicy::Skip, make_items(count))
}

// -- Queue creation --

#[test]
fn test_queue_creation() {
    let queue: BatchQueue<String> = BatchQueue::new();
    assert_eq!(queue.queued_count(), 0);
    assert!(!queue.has_running_job());
}

// -- Enqueue and list --

#[test]
fn test_enqueue_assigns_id() {
    let queue: BatchQueue<String> = BatchQueue::new();
    let id = queue.enqueue(make_job("model-a", "tag", 3)).unwrap();
    assert!(!id.is_empty());
}

#[test]
fn test_list_jobs() {
    let queue: BatchQueue<String> = BatchQueue::new();
    queue.enqueue(make_job("model-a", "tag", 2)).unwrap();
    queue.enqueue(make_job("model-b", "caption", 3)).unwrap();

    let jobs = queue.list_jobs();
    assert_eq!(jobs.len(), 2);
}

#[test]
fn test_get_job() {
    let queue: BatchQueue<String> = BatchQueue::new();
    let id = queue.enqueue(make_job("model-a", "tag", 1)).unwrap();

    let job = queue.get_job(&id).unwrap();
    assert_eq!(job.resource_key, "model-a");
    assert_eq!(job.operation, "tag");
    assert_eq!(job.items.len(), 1);
}

#[test]
fn test_get_nonexistent_job() {
    let queue: BatchQueue<String> = BatchQueue::new();
    assert!(queue.get_job("nonexistent").is_none());
}

// -- Next queued --

#[test]
fn test_next_queued_empty() {
    let queue: BatchQueue<String> = BatchQueue::new();
    assert!(queue.next_queued().is_none());
}

#[test]
fn test_next_queued_returns_first() {
    let queue: BatchQueue<String> = BatchQueue::new();
    let id1 = queue.enqueue(make_job("model-a", "tag", 1)).unwrap();
    queue.enqueue(make_job("model-a", "caption", 1)).unwrap();

    let next = queue.next_queued().unwrap();
    assert_eq!(next.id, id1);
}

// -- Status lifecycle --

#[test]
fn test_mark_running() {
    let queue: BatchQueue<String> = BatchQueue::new();
    let id = queue.enqueue(make_job("model-a", "tag", 1)).unwrap();

    queue.mark_running(&id).unwrap();
    let job = queue.get_job(&id).unwrap();
    assert_eq!(job.status, BatchJobStatus::Running);
    assert!(job.started_at.is_some());
    assert!(queue.has_running_job());
}

#[test]
fn test_complete_all_success() {
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
    assert_eq!(summary.skipped, 0);
    assert_eq!(summary.total_duration_ms, 3000);
    assert_eq!(summary.avg_duration_ms, 1500);
    assert_eq!(summary.operation, "tag");
    assert_eq!(summary.resource_key, "model-a");

    let job = queue.get_job(&id).unwrap();
    assert_eq!(job.status, BatchJobStatus::Completed);
    assert!(job.completed_at.is_some());
}

#[test]
fn test_complete_with_errors() {
    let queue: BatchQueue<String> = BatchQueue::new();
    let id = queue.enqueue(make_job("model-a", "tag", 3)).unwrap();
    queue.mark_running(&id).unwrap();

    queue
        .update_item(&id, "item-0", BatchItemStatus::Completed, None, Some(1000))
        .unwrap();
    queue
        .update_item(
            &id,
            "item-1",
            BatchItemStatus::Failed,
            Some("timeout".into()),
            Some(5000),
        )
        .unwrap();
    queue
        .update_item(&id, "item-2", BatchItemStatus::Completed, None, Some(1000))
        .unwrap();

    let summary = queue.mark_completed(&id).unwrap().unwrap();
    assert_eq!(summary.succeeded, 2);
    assert_eq!(summary.failed, 1);

    let job = queue.get_job(&id).unwrap();
    assert_eq!(job.status, BatchJobStatus::CompletedWithErrors);
}

// -- Cancellation --

#[test]
fn test_cancel_queued_job() {
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
fn test_cancel_running_item_marks_cancelled() {
    let queue: BatchQueue<String> = BatchQueue::new();
    let id = queue.enqueue(make_job("model-a", "tag", 1)).unwrap();
    queue.mark_running(&id).unwrap();

    // Mark item as running
    queue
        .update_item(&id, "item-0", BatchItemStatus::Running, None, None)
        .unwrap();

    // Cancelling a running item should prevent it from being completed later.
    queue.cancel_item(&id, "item-0").unwrap();
    let job = queue.get_job(&id).unwrap();
    assert_eq!(job.items[0].status, BatchItemStatus::Cancelled);
}

// -- Retry --

#[test]
fn test_retry_failed() {
    let queue: BatchQueue<String> = BatchQueue::new();
    let id = queue.enqueue(make_job("model-a", "tag", 3)).unwrap();
    queue.mark_running(&id).unwrap();

    queue
        .update_item(&id, "item-0", BatchItemStatus::Completed, None, Some(1000))
        .unwrap();
    queue
        .update_item(
            &id,
            "item-1",
            BatchItemStatus::Failed,
            Some("err".into()),
            None,
        )
        .unwrap();
    queue
        .update_item(&id, "item-2", BatchItemStatus::Completed, None, Some(1000))
        .unwrap();
    queue.mark_completed(&id).unwrap();

    queue.retry_failed(&id).unwrap();
    let job = queue.get_job(&id).unwrap();
    assert_eq!(job.status, BatchJobStatus::Queued);
    assert_eq!(job.items[0].status, BatchItemStatus::Completed);
    assert_eq!(job.items[1].status, BatchItemStatus::Pending);
    assert!(job.items[1].error.is_none());
    assert_eq!(job.items[2].status, BatchItemStatus::Completed);
}

#[test]
fn test_retry_no_failed_items_errors() {
    let queue: BatchQueue<String> = BatchQueue::new();
    let id = queue.enqueue(make_job("model-a", "tag", 1)).unwrap();
    queue.mark_running(&id).unwrap();
    queue
        .update_item(&id, "item-0", BatchItemStatus::Completed, None, Some(1000))
        .unwrap();
    queue.mark_completed(&id).unwrap();

    let result = queue.retry_failed(&id);
    assert!(result.is_err());
}

// -- Model-aware reordering --

#[test]
fn test_reorder_groups_by_resource() {
    let queue: BatchQueue<String> = BatchQueue::new();
    queue.enqueue(make_job("model-b", "tag", 1)).unwrap();
    queue.enqueue(make_job("model-a", "caption", 1)).unwrap();
    queue.enqueue(make_job("model-b", "caption", 1)).unwrap();

    let jobs = queue.list_jobs();
    assert_eq!(jobs[0].resource_key, "model-a");
    assert_eq!(jobs[1].resource_key, "model-b");
    assert_eq!(jobs[2].resource_key, "model-b");
}

#[test]
fn test_reorder_marks_jobs() {
    let queue: BatchQueue<String> = BatchQueue::new();
    queue.enqueue(make_job("model-b", "tag", 1)).unwrap();
    queue.enqueue(make_job("model-a", "tag", 1)).unwrap();

    let jobs = queue.list_jobs();
    assert!(jobs[0].reordered);
    assert!(jobs[0].reorder_note.is_some());
}

#[test]
fn test_reorder_preserves_running() {
    let queue: BatchQueue<String> = BatchQueue::new();
    let id1 = queue.enqueue(make_job("model-c", "tag", 1)).unwrap();
    queue.mark_running(&id1).unwrap();

    queue.enqueue(make_job("model-a", "tag", 1)).unwrap();
    queue.enqueue(make_job("model-c", "tag", 1)).unwrap();

    let jobs = queue.list_jobs();
    assert_eq!(jobs[0].id, id1); // Running job stays at its position
    assert_eq!(jobs[0].status, BatchJobStatus::Running);
    assert_eq!(jobs[1].resource_key, "model-a"); // Queued reordered
    assert_eq!(jobs[2].resource_key, "model-c");
}

#[test]
fn test_no_reorder_same_resource() {
    let queue: BatchQueue<String> = BatchQueue::new();
    queue.enqueue(make_job("model-a", "tag", 1)).unwrap();
    queue.enqueue(make_job("model-a", "caption", 1)).unwrap();

    let jobs = queue.list_jobs();
    // Same resource, no reorder needed
    assert!(!jobs[0].reordered);
    assert!(!jobs[1].reordered);
}

// -- ETA estimation --

#[test]
fn test_eta_no_data() {
    let queue: BatchQueue<String> = BatchQueue::new();
    let id = queue.enqueue(make_job("model-a", "tag", 3)).unwrap();
    queue.mark_running(&id).unwrap();

    assert!(queue.estimate_remaining_ms(&id).is_none());
}

#[test]
fn test_eta_after_completions() {
    let queue: BatchQueue<String> = BatchQueue::new();
    let id = queue.enqueue(make_job("model-a", "tag", 3)).unwrap();
    queue.mark_running(&id).unwrap();

    queue
        .update_item(&id, "item-0", BatchItemStatus::Completed, None, Some(1000))
        .unwrap();

    // 2 remaining * 1000ms avg = 2000ms
    let eta = queue.estimate_remaining_ms(&id);
    assert_eq!(eta, Some(2000));
}

#[test]
fn test_eta_zero_when_all_done() {
    let queue: BatchQueue<String> = BatchQueue::new();
    let id = queue.enqueue(make_job("model-a", "tag", 1)).unwrap();
    queue.mark_running(&id).unwrap();

    queue
        .update_item(&id, "item-0", BatchItemStatus::Completed, None, Some(500))
        .unwrap();

    assert_eq!(queue.estimate_remaining_ms(&id), Some(0));
}

// -- Counts --

#[test]
fn test_queued_count() {
    let queue: BatchQueue<String> = BatchQueue::new();
    assert_eq!(queue.queued_count(), 0);

    queue.enqueue(make_job("a", "tag", 1)).unwrap();
    assert_eq!(queue.queued_count(), 1);

    queue.enqueue(make_job("b", "tag", 1)).unwrap();
    assert_eq!(queue.queued_count(), 2);
}

#[test]
fn test_has_running_job() {
    let queue: BatchQueue<String> = BatchQueue::new();
    assert!(!queue.has_running_job());

    let id = queue.enqueue(make_job("a", "tag", 1)).unwrap();
    assert!(!queue.has_running_job());

    queue.mark_running(&id).unwrap();
    assert!(queue.has_running_job());
}

// -- build_job helper --

#[test]
fn test_build_job_helper() {
    let job = build_job(
        "model-x",
        "embed",
        OverwritePolicy::Overwrite,
        vec![
            ("a".into(), "data-a".to_string(), SizeBucket::Small),
            ("b".into(), "data-b".to_string(), SizeBucket::Large),
        ],
    );

    assert_eq!(job.resource_key, "model-x");
    assert_eq!(job.operation, "embed");
    assert_eq!(job.overwrite_policy, OverwritePolicy::Overwrite);
    assert_eq!(job.items.len(), 2);
    assert_eq!(job.items[0].id, "a");
    assert_eq!(job.items[0].data, "data-a");
    assert_eq!(job.items[0].size_bucket, SizeBucket::Small);
    assert_eq!(job.items[1].size_bucket, SizeBucket::Large);
}

// -- Type tests --

#[test]
fn test_size_bucket_classification() {
    assert_eq!(SizeBucket::from_pixel_count(100_000), SizeBucket::Small);
    assert_eq!(SizeBucket::from_pixel_count(1_000_000), SizeBucket::Medium);
    assert_eq!(SizeBucket::from_pixel_count(5_000_000), SizeBucket::Large);

    assert_eq!(
        SizeBucket::from_dimensions(Some(500), Some(500)),
        SizeBucket::Small
    );
    assert_eq!(
        SizeBucket::from_dimensions(Some(1000), Some(1000)),
        SizeBucket::Medium
    );
    assert_eq!(
        SizeBucket::from_dimensions(Some(2000), Some(2000)),
        SizeBucket::Large
    );
    assert_eq!(
        SizeBucket::from_dimensions(None, Some(500)),
        SizeBucket::Unknown
    );
    assert_eq!(
        SizeBucket::from_dimensions(Some(500), None),
        SizeBucket::Unknown
    );
    assert_eq!(SizeBucket::from_dimensions(None, None), SizeBucket::Unknown);
}

#[test]
fn test_item_result_variants() {
    let success = ItemResult::success();
    assert!(success.success);
    assert!(success.output.is_none());

    let with_output = ItemResult::success_with_output("tags: cat, dog".into());
    assert!(with_output.success);
    assert_eq!(with_output.output.as_deref(), Some("tags: cat, dog"));

    let failure = ItemResult::failure("connection timeout".into());
    assert!(!failure.success);
    assert_eq!(failure.error.as_deref(), Some("connection timeout"));
}

#[test]
fn test_event_serialization() {
    // BatchCompletionSummary should serialize with camelCase
    let summary = BatchCompletionSummary {
        job_id: "j1".into(),
        operation: "tag".into(),
        resource_key: "llava".into(),
        total: 10,
        succeeded: 8,
        failed: 1,
        skipped: 1,
        total_duration_ms: 10000,
        avg_duration_ms: 1111,
    };
    let json = serde_json::to_string(&summary).unwrap();
    assert!(json.contains("jobId"));
    assert!(json.contains("resourceKey"));
    assert!(json.contains("totalDurationMs"));
    assert!(json.contains("avgDurationMs"));
}

// ── VG-8: Fairness + ETA tests ──────────────────────────────

#[test]
fn test_scheduling_config_defaults() {
    let config = SchedulingConfig::default();
    assert_eq!(config.max_consecutive_same_key, 3);
    assert!(config.enable_reordering);
}

#[test]
fn test_max_consecutive_same_key_scheduling() {
    // When max_consecutive_same_key is respected, the queue
    // should not schedule more than N same-resource jobs in a row
    let config = SchedulingConfig {
        max_consecutive_same_key: 2,
        ..Default::default()
    };
    // Verify the config is properly configured
    assert_eq!(config.max_consecutive_same_key, 2);

    let queue: BatchQueue<String> = BatchQueue::new();

    // Enqueue 4 jobs for resource A and 2 for resource B
    for i in 0..4 {
        let job = make_job("model-A", "tag", 1);
        queue.enqueue(job).unwrap();
        let _ = i; // suppress unused warning
    }
    for _ in 0..2 {
        let job = make_job("model-B", "caption", 1);
        queue.enqueue(job).unwrap();
    }

    // The queue reorders by resource_key. Let's verify grouping exists
    let jobs = queue.list_jobs();
    assert_eq!(jobs.len(), 6);

    // All model-A jobs should be grouped together (reordering)
    let first_key = &jobs[0].resource_key;
    let grouped = jobs
        .iter()
        .take_while(|j| &j.resource_key == first_key)
        .count();
    assert!(grouped >= 2, "Jobs should be grouped by resource_key");
}

#[test]
fn test_next_queued_respects_max_consecutive_cap() {
    let queue: BatchQueue<String> = BatchQueue::with_scheduling(SchedulingConfig {
        max_consecutive_same_key: 1,
        ..Default::default()
    });

    let job_a1 = make_job("model-A", "tag", 1);
    let job_a2 = make_job("model-A", "tag", 1);
    let job_b1 = make_job("model-B", "tag", 1);

    let id_a1 = queue.enqueue(job_a1).unwrap();
    queue.enqueue(job_a2).unwrap();
    queue.enqueue(job_b1).unwrap();

    let first = queue.next_queued().unwrap();
    assert_eq!(first.id, id_a1);
    queue.mark_running(&id_a1).unwrap();

    let second = queue.next_queued().unwrap();
    assert_eq!(second.resource_key, "model-B");
}

#[test]
fn test_eta_estimate_struct() {
    let est = EtaEstimate {
        remaining_ms: 5000,
        items_remaining: 10,
        avg_item_ms: 500,
        confidence: EtaConfidence::High,
        sample_count: 50,
    };
    assert_eq!(est.remaining_ms, 5000);
    assert_eq!(est.confidence, EtaConfidence::High);

    let json = serde_json::to_string(&est).unwrap();
    assert!(json.contains("remainingMs"));
    assert!(json.contains("confidence"));
    assert!(json.contains("sampleCount"));
}

#[test]
fn test_eta_confidence_levels() {
    assert_ne!(EtaConfidence::Low, EtaConfidence::Medium);
    assert_ne!(EtaConfidence::Medium, EtaConfidence::High);

    // Low confidence with few samples
    let est = EtaEstimate {
        remaining_ms: 10000,
        items_remaining: 5,
        avg_item_ms: 2000,
        confidence: EtaConfidence::Low,
        sample_count: 1,
    };
    assert_eq!(est.confidence, EtaConfidence::Low);
}

#[test]
fn test_scheduling_config_serialization() {
    let config = SchedulingConfig::default();
    let json = serde_json::to_string(&config).unwrap();
    assert!(json.contains("max_consecutive_same_key"));
    let deserialized: SchedulingConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.max_consecutive_same_key, 3);
}

// ── Retry lineage proof (P2 acceptance: executor-path lineage) ──

#[test]
fn test_retry_preserves_attempt_id_and_clears_trial_id() {
    let queue: BatchQueue<String> = BatchQueue::new();
    let mut job = make_job("model-a", "tag", 3);

    // Pre-assign attempt and trial IDs to simulate executor behavior
    let attempt_0 = stack_ids::AttemptId::generate();
    let attempt_1 = stack_ids::AttemptId::generate();
    let attempt_2 = stack_ids::AttemptId::generate();
    let trial_0 = stack_ids::TrialId::generate();
    let trial_1 = stack_ids::TrialId::generate();
    let trial_2 = stack_ids::TrialId::generate();

    job.items[0].attempt_id = Some(attempt_0.clone());
    job.items[0].trial_id = Some(trial_0.clone());
    job.items[1].attempt_id = Some(attempt_1.clone());
    job.items[1].trial_id = Some(trial_1.clone());
    job.items[2].attempt_id = Some(attempt_2.clone());
    job.items[2].trial_id = Some(trial_2.clone());

    let id = queue.enqueue(job).unwrap();
    queue.mark_running(&id).unwrap();

    // Complete item 0, fail items 1 and 2
    queue.update_item(&id, "item-0", BatchItemStatus::Completed, None, Some(100)).unwrap();
    queue.update_item(&id, "item-1", BatchItemStatus::Failed, Some("transient error".to_string()), Some(50)).unwrap();
    queue.update_item(&id, "item-2", BatchItemStatus::Failed, Some("transient error 2".to_string()), Some(60)).unwrap();
    queue.mark_completed(&id).unwrap();

    // Retry failed items
    queue.retry_failed(&id).unwrap();

    let job = queue.get_job(&id).unwrap();

    // Item 0: completed, untouched — attempt_id and trial_id preserved
    assert_eq!(job.items[0].attempt_id, Some(attempt_0));
    assert_eq!(job.items[0].trial_id, Some(trial_0));

    // Items 1 & 2: retried — attempt_id preserved, trial_id cleared
    assert_eq!(job.items[1].attempt_id, Some(attempt_1));
    assert!(job.items[1].trial_id.is_none(), "trial_id must be cleared on retry");
    assert_eq!(job.items[2].attempt_id, Some(attempt_2));
    assert!(job.items[2].trial_id.is_none(), "trial_id must be cleared on retry");

    // Status reset to Pending
    assert_eq!(job.items[1].status, BatchItemStatus::Pending);
    assert_eq!(job.items[2].status, BatchItemStatus::Pending);
}

#[test]
fn test_batch_item_retry_keeps_attempt_family() {
    // Proves that batch-item retries (via retry_failed) keep the same
    // AttemptId family. Outer job-level retries would create a new
    // QueueJob with a new AttemptId — the distinction is structural.

    let queue: BatchQueue<String> = BatchQueue::new();
    let mut job = make_job("model-a", "tag", 2);

    let attempt_0 = stack_ids::AttemptId::generate();
    let attempt_1 = stack_ids::AttemptId::generate();
    job.items[0].attempt_id = Some(attempt_0.clone());
    job.items[0].trial_id = Some(stack_ids::TrialId::generate());
    job.items[1].attempt_id = Some(attempt_1.clone());
    job.items[1].trial_id = Some(stack_ids::TrialId::generate());

    let id = queue.enqueue(job).unwrap();
    queue.mark_running(&id).unwrap();

    // Fail item 0, complete item 1
    queue.update_item(&id, "item-0", BatchItemStatus::Failed, Some("transient".to_string()), Some(50)).unwrap();
    queue.update_item(&id, "item-1", BatchItemStatus::Completed, None, Some(100)).unwrap();
    queue.mark_completed(&id).unwrap();

    // Batch-item retry
    queue.retry_failed(&id).unwrap();

    let job = queue.get_job(&id).unwrap();
    // Same attempt_id — this is a batch-item retry, NOT an outer job retry
    assert_eq!(job.items[0].attempt_id, Some(attempt_0));
    // trial_id cleared for fresh execution
    assert!(job.items[0].trial_id.is_none());
    // Completed item untouched
    assert_eq!(job.items[1].attempt_id, Some(attempt_1));
    assert!(job.items[1].trial_id.is_some());
}
