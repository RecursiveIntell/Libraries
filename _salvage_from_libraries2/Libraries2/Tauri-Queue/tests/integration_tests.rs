mod test_helpers;

use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tauri_queue::*;
use tempfile::tempdir;
use test_helpers::TestJob;

#[test]
fn test_queue_creation_in_memory() {
    let config = QueueConfig::default();
    let queue = QueueManager::new(config);
    assert!(queue.is_ok());
}

#[test]
fn test_queue_creation_with_db() {
    let temp = tempdir().unwrap();
    let config = QueueConfig::builder()
        .with_db_path(temp.path().join("test.db"))
        .build();
    let queue = QueueManager::new(config);
    assert!(queue.is_ok());
}

#[test]
fn test_add_job() {
    let config = QueueConfig::default();
    let queue = QueueManager::new(config).unwrap();

    let job = QueueJob::new(TestJob {
        data: "test".to_string(),
    });

    let job_id = queue.add(job).unwrap();
    assert!(!job_id.is_empty());
}

#[test]
fn test_add_job_with_custom_id() {
    let config = QueueConfig::default();
    let queue = QueueManager::new(config).unwrap();

    let job = QueueJob::new(TestJob {
        data: "test".to_string(),
    })
    .with_id("my-custom-id".to_string());

    let job_id = queue.add(job).unwrap();
    assert_eq!(job_id, "my-custom-id");
}

#[test]
fn test_priority_ordering() {
    let config = QueueConfig::default();
    let queue = QueueManager::new(config).unwrap();

    let low = QueueJob::new(TestJob { data: "low".into() })
        .with_priority(QueuePriority::Low)
        .with_id("low".into());

    let high = QueueJob::new(TestJob {
        data: "high".into(),
    })
    .with_priority(QueuePriority::High)
    .with_id("high".into());

    let normal = QueueJob::new(TestJob {
        data: "normal".into(),
    })
    .with_priority(QueuePriority::Normal)
    .with_id("normal".into());

    queue.add(low).unwrap();
    queue.add(high).unwrap();
    queue.add(normal).unwrap();

    let jobs = queue.list_jobs().unwrap();
    assert_eq!(jobs.len(), 3);
    // All pending, sorted by priority: high(1), normal(2), low(3)
    assert_eq!(jobs[0].0, "high");
    assert_eq!(jobs[1].0, "normal");
    assert_eq!(jobs[2].0, "low");
}

#[test]
fn test_cancel_pending_job() {
    let config = QueueConfig::default();
    let queue = QueueManager::new(config).unwrap();

    let job = QueueJob::new(TestJob {
        data: "test".into(),
    })
    .with_id("cancel-me".into());

    queue.add(job).unwrap();
    queue.cancel("cancel-me").unwrap();

    let jobs = queue.list_jobs().unwrap();
    assert_eq!(jobs[0].1, "cancelled");
}

#[test]
fn test_cancel_nonexistent_fails() {
    let config = QueueConfig::default();
    let queue = QueueManager::new(config).unwrap();

    let result = queue.cancel("nonexistent");
    assert!(result.is_err());
}

#[test]
fn test_reorder_pending_job() {
    let config = QueueConfig::default();
    let queue = QueueManager::new(config).unwrap();

    let job = QueueJob::new(TestJob {
        data: "test".into(),
    })
    .with_priority(QueuePriority::Low)
    .with_id("reorder-me".into());

    queue.add(job).unwrap();
    queue.reorder("reorder-me", QueuePriority::High).unwrap();

    // Verify by listing — job should be present
    let jobs = queue.list_jobs().unwrap();
    assert_eq!(jobs.len(), 1);
    assert_eq!(jobs[0].0, "reorder-me");
}

#[test]
fn test_reorder_nonexistent_fails() {
    let config = QueueConfig::default();
    let queue = QueueManager::new(config).unwrap();

    let result = queue.reorder("nonexistent", QueuePriority::High);
    assert!(result.is_err());
}

#[test]
fn test_pause_resume() {
    let config = QueueConfig::default();
    let queue = QueueManager::new(config).unwrap();

    assert!(!queue.is_paused());

    queue.pause();
    assert!(queue.is_paused());

    queue.resume();
    assert!(!queue.is_paused());
}

#[test]
fn test_list_jobs_empty() {
    let config = QueueConfig::default();
    let queue = QueueManager::new(config).unwrap();

    let jobs = queue.list_jobs().unwrap();
    assert!(jobs.is_empty());
}

#[test]
fn test_list_jobs_with_data() {
    let config = QueueConfig::default();
    let queue = QueueManager::new(config).unwrap();

    let job = QueueJob::new(TestJob {
        data: "hello world".into(),
    })
    .with_id("data-job".into());

    queue.add(job).unwrap();

    let jobs = queue.list_jobs_with_data().unwrap();
    assert_eq!(jobs.len(), 1);
    assert_eq!(jobs[0].0, "data-job");
    assert_eq!(jobs[0].1, "pending");
    assert!(jobs[0].2.contains("hello world"));
}

#[test]
fn test_persistence_across_instances() {
    let temp = tempdir().unwrap();
    let db_path = temp.path().join("persist.db");

    // First instance: add jobs
    {
        let config = QueueConfig::builder().with_db_path(db_path.clone()).build();
        let queue = QueueManager::new(config).unwrap();

        queue
            .add(
                QueueJob::new(TestJob {
                    data: "persistent".into(),
                })
                .with_id("p1".into()),
            )
            .unwrap();
        queue
            .add(
                QueueJob::new(TestJob {
                    data: "persistent2".into(),
                })
                .with_id("p2".into()),
            )
            .unwrap();
    }

    // Second instance: jobs should still be there
    {
        let config = QueueConfig::builder().with_db_path(db_path).build();
        let queue = QueueManager::new(config).unwrap();

        let jobs = queue.list_jobs().unwrap();
        assert_eq!(jobs.len(), 2);
    }
}

#[test]
fn test_crash_recovery_requeues_processing() {
    let temp = tempdir().unwrap();
    let db_path = temp.path().join("crash.db");

    // First instance: add a job and mark it processing via raw DB
    {
        let conn = tauri_queue::db::open_database(Some(&db_path)).unwrap();
        tauri_queue::db::insert_job(
            &conn,
            "crashed-job",
            2,
            &serde_json::json!({"data": "was processing"}),
        )
        .unwrap();
        tauri_queue::db::mark_processing(&conn, "crashed-job").unwrap();
    }

    // Second instance: QueueManager should requeue the processing job
    {
        let config = QueueConfig::builder().with_db_path(db_path).build();
        let queue = QueueManager::new(config).unwrap();

        let jobs = queue.list_jobs().unwrap();
        assert_eq!(jobs.len(), 1);
        assert_eq!(jobs[0].0, "crashed-job");
        assert_eq!(jobs[0].1, "pending"); // Requeued from processing
    }
}

#[test]
fn test_prune_no_completed_jobs() {
    let config = QueueConfig::default();
    let queue = QueueManager::new(config).unwrap();

    queue
        .add(QueueJob::new(TestJob {
            data: "pending".into(),
        }))
        .unwrap();

    let pruned = queue.prune(0).unwrap();
    assert_eq!(pruned, 0);
}

#[test]
fn test_multiple_priorities_interleaved() {
    let config = QueueConfig::default();
    let queue = QueueManager::new(config).unwrap();

    for i in 0..3 {
        queue
            .add(
                QueueJob::new(TestJob {
                    data: format!("high-{}", i),
                })
                .with_priority(QueuePriority::High)
                .with_id(format!("h-{}", i)),
            )
            .unwrap();
        queue
            .add(
                QueueJob::new(TestJob {
                    data: format!("low-{}", i),
                })
                .with_priority(QueuePriority::Low)
                .with_id(format!("l-{}", i)),
            )
            .unwrap();
    }

    let jobs = queue.list_jobs().unwrap();
    assert_eq!(jobs.len(), 6);

    // First 3 should be high priority
    for (i, job) in jobs.iter().enumerate().take(3) {
        assert!(
            job.0.starts_with("h-"),
            "Expected high priority at index {}",
            i
        );
    }
    // Last 3 should be low priority
    for (i, job) in jobs.iter().enumerate().skip(3).take(3) {
        assert!(
            job.0.starts_with("l-"),
            "Expected low priority at index {}",
            i
        );
    }
}

// -- Type tests --

#[test]
fn test_queue_priority_roundtrip() {
    assert_eq!(
        QueuePriority::from_i32(QueuePriority::High.as_i32()),
        QueuePriority::High
    );
    assert_eq!(
        QueuePriority::from_i32(QueuePriority::Normal.as_i32()),
        QueuePriority::Normal
    );
    assert_eq!(
        QueuePriority::from_i32(QueuePriority::Low.as_i32()),
        QueuePriority::Low
    );
    // Unknown value defaults to Low
    assert_eq!(QueuePriority::from_i32(99), QueuePriority::Low);
}

#[test]
fn test_job_status_roundtrip() {
    let statuses = [
        QueueJobStatus::Pending,
        QueueJobStatus::Processing,
        QueueJobStatus::Completed,
        QueueJobStatus::Failed,
        QueueJobStatus::Cancelled,
    ];

    for status in &statuses {
        let s = status.as_str();
        let parsed = QueueJobStatus::parse(s);
        assert_eq!(parsed.as_ref(), Some(status));
    }

    assert_eq!(QueueJobStatus::parse("unknown"), None);
}

#[test]
fn test_job_result_variants() {
    let success = JobResult::success();
    assert!(success.success);
    assert!(success.output.is_none());
    assert!(success.error.is_none());

    let with_output = JobResult::success_with_output("done".into());
    assert!(with_output.success);
    assert_eq!(with_output.output.as_deref(), Some("done"));

    let failure = JobResult::failure("oops".into());
    assert!(!failure.success);
    assert_eq!(failure.error.as_deref(), Some("oops"));
}

#[test]
fn test_event_serialization() {
    use tauri_queue::events::*;

    let started = JobStartedEvent {
        job_id: "j1".to_string(),
        trace_id: None,
        worker_id: None,
        attempt_count: None,
        status: None,
        trace_ctx: None,
        attempt_id: None,
        trial_id: None,
    };
    let json = serde_json::to_string(&started).unwrap();
    assert!(json.contains("jobId"));

    let completed = JobCompletedEvent {
        job_id: "j1".to_string(),
        output: Some("result".to_string()),
        trace_id: None,
        worker_id: None,
        attempt_count: None,
        status: None,
        trace_ctx: None,
        attempt_id: None,
        trial_id: None,
    };
    let json = serde_json::to_string(&completed).unwrap();
    assert!(json.contains("jobId"));
    assert!(json.contains("result"));

    let failed = JobFailedEvent {
        job_id: "j1".to_string(),
        error: "something broke".to_string(),
        trace_id: None,
        worker_id: None,
        attempt_count: None,
        status: None,
        failure_class: None,
        next_retry_at: None,
        trace_ctx: None,
        attempt_id: None,
        trial_id: None,
    };
    let json = serde_json::to_string(&failed).unwrap();
    assert!(json.contains("something broke"));

    let progress = JobProgressEvent {
        job_id: "j1".to_string(),
        current_step: 5,
        total_steps: 10,
        progress: 0.5,
        trace_id: None,
        worker_id: None,
        attempt_count: None,
        status: None,
        trace_ctx: None,
        attempt_id: None,
        trial_id: None,
    };
    let json = serde_json::to_string(&progress).unwrap();
    assert!(json.contains("currentStep"));
    assert!(json.contains("totalSteps"));

    let cancelled = JobCancelledEvent {
        job_id: "j1".to_string(),
        trace_id: None,
        worker_id: None,
        attempt_count: None,
        status: None,
        trace_ctx: None,
        attempt_id: None,
        trial_id: None,
    };
    let json = serde_json::to_string(&cancelled).unwrap();
    assert!(json.contains("jobId"));
}

#[test]
fn test_config_builder() {
    use std::time::Duration;

    let config = QueueConfig::builder()
        .with_cooldown(Duration::from_secs(5))
        .with_max_consecutive(10)
        .with_poll_interval(Duration::from_secs(1))
        .build();

    assert_eq!(config.cooldown, Duration::from_secs(5));
    assert_eq!(config.max_consecutive, 10);
    assert_eq!(config.poll_interval, Duration::from_secs(1));
    assert!(config.db_path.is_none());
}

#[test]
fn test_config_defaults() {
    let config = QueueConfig::default();
    assert!(config.db_path.is_none());
    assert_eq!(config.cooldown, std::time::Duration::from_secs(0));
    assert_eq!(config.max_consecutive, 0);
    assert_eq!(config.poll_interval, std::time::Duration::from_secs(3));
}

// ── VG-7 backpressure / coalescing tests ───────────────────

/// Recording emitter that counts calls for testing.
struct RecordingEmitter {
    progress_count: Mutex<u32>,
    started_count: Mutex<u32>,
}

impl RecordingEmitter {
    fn new() -> Self {
        Self {
            progress_count: Mutex::new(0),
            started_count: Mutex::new(0),
        }
    }
}

impl QueueEventEmitter for RecordingEmitter {
    fn emit_job_started(&self, _event: JobStartedEvent) {
        *self.started_count.lock().unwrap() += 1;
    }
    fn emit_job_completed(&self, _event: JobCompletedEvent) {}
    fn emit_job_failed(&self, _event: JobFailedEvent) {}
    fn emit_job_progress(&self, _event: JobProgressEvent) {
        *self.progress_count.lock().unwrap() += 1;
    }
    fn emit_job_cancelled(&self, _event: JobCancelledEvent) {}
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ObservedEmitterEvent {
    Progress { job_id: String, current_step: u32 },
    Completed { job_id: String },
}

struct SequenceEmitter {
    events: Mutex<Vec<ObservedEmitterEvent>>,
}

impl SequenceEmitter {
    fn new() -> Self {
        Self {
            events: Mutex::new(Vec::new()),
        }
    }

    fn events(&self) -> Vec<ObservedEmitterEvent> {
        self.events.lock().unwrap().clone()
    }
}

impl QueueEventEmitter for SequenceEmitter {
    fn emit_job_started(&self, _event: JobStartedEvent) {}
    fn emit_job_completed(&self, event: JobCompletedEvent) {
        self.events.lock().unwrap().push(ObservedEmitterEvent::Completed {
            job_id: event.job_id,
        });
    }
    fn emit_job_failed(&self, _event: JobFailedEvent) {}
    fn emit_job_progress(&self, event: JobProgressEvent) {
        self.events.lock().unwrap().push(ObservedEmitterEvent::Progress {
            job_id: event.job_id,
            current_step: event.current_step,
        });
    }
    fn emit_job_cancelled(&self, _event: JobCancelledEvent) {}
}

fn progress_event(job_id: &str, current_step: u32, total_steps: u32) -> JobProgressEvent {
    JobProgressEvent {
        job_id: job_id.into(),
        current_step,
        total_steps,
        progress: current_step as f64 / total_steps.max(1) as f64,
        trace_id: None,
        worker_id: None,
        attempt_count: None,
        status: Some("processing".into()),
        trace_ctx: None,
        attempt_id: None,
        trial_id: None,
    }
}

fn completed_event(job_id: &str) -> JobCompletedEvent {
    JobCompletedEvent {
        job_id: job_id.into(),
        output: None,
        trace_id: None,
        worker_id: None,
        attempt_count: None,
        status: Some("completed".into()),
        trace_ctx: None,
        attempt_id: None,
        trial_id: None,
    }
}

#[test]
fn test_coalesce_suppresses_rapid_progress() {
    let recorder = Arc::new(RecordingEmitter::new());

    let config = EmitterConfig {
        coalesce_interval_ms: 200, // 200ms coalesce window
        ..Default::default()
    };

    let coalescing = CoalescingEmitter::new(recorder.clone(), config);

    // Fire 50 progress events rapidly (within 200ms)
    for i in 0..50 {
        coalescing.emit_job_progress(JobProgressEvent {
            job_id: "job-1".into(),
            current_step: i,
            total_steps: 50,
            progress: i as f64 / 50.0,
            trace_id: Some("trace-1".into()),
            worker_id: Some("worker-1".into()),
            attempt_count: Some(1),
            status: Some("processing".into()),
            trace_ctx: None,
            attempt_id: None,
            trial_id: None,
        });
    }

    let count = *recorder.progress_count.lock().unwrap();
    // First event always emits; subsequent ones within 200ms are suppressed
    assert!(count < 50, "Expected coalescing to suppress some events, got {count}");
    assert!(count >= 1, "At least the first event should have been emitted");
}

#[test]
fn test_coalesce_passes_started_completed() {
    let recorder = Arc::new(RecordingEmitter::new());

    let config = EmitterConfig {
        coalesce_interval_ms: 1000, // aggressive coalescing
        ..Default::default()
    };

    let coalescing = CoalescingEmitter::new(recorder.clone(), config);

    // Non-progress events should never be coalesced
    coalescing.emit_job_started(JobStartedEvent {
        job_id: "job-1".into(),
        trace_id: None,
        worker_id: None,
        attempt_count: None,
        status: None,
        trace_ctx: None,
        attempt_id: None,
        trial_id: None,
    });
    coalescing.emit_job_started(JobStartedEvent {
        job_id: "job-1".into(),
        trace_id: None,
        worker_id: None,
        attempt_count: None,
        status: None,
        trace_ctx: None,
        attempt_id: None,
        trial_id: None,
    });

    let count = *recorder.started_count.lock().unwrap();
    assert_eq!(count, 2, "Started events should not be coalesced");
}

#[test]
fn test_concurrent_progress_flushes_latest_pending_before_completion() {
    let recorder = Arc::new(SequenceEmitter::new());
    let coalescing = Arc::new(CoalescingEmitter::new(
        recorder.clone(),
        EmitterConfig {
            coalesce_interval_ms: 60_000,
            ..Default::default()
        },
    ));

    coalescing.emit_job_progress(progress_event("job-1", 0, 8));

    let mut handles = Vec::new();
    for step in 1..=8 {
        let emitter = coalescing.clone();
        handles.push(thread::spawn(move || {
            thread::sleep(Duration::from_millis(step as u64 * 10));
            emitter.emit_job_progress(progress_event("job-1", step, 8));
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    coalescing.emit_job_completed(completed_event("job-1"));

    assert_eq!(
        recorder.events(),
        vec![
            ObservedEmitterEvent::Progress {
                job_id: "job-1".into(),
                current_step: 0,
            },
            ObservedEmitterEvent::Progress {
                job_id: "job-1".into(),
                current_step: 8,
            },
            ObservedEmitterEvent::Completed {
                job_id: "job-1".into(),
            },
        ],
    );
}

#[test]
fn test_block_policy_flushes_oldest_pending_without_locking_up() {
    let recorder = Arc::new(SequenceEmitter::new());
    let coalescing = Arc::new(CoalescingEmitter::new(
        recorder.clone(),
        EmitterConfig {
            buffer_size: 1,
            drop_policy: DropPolicy::Block,
            coalesce_interval_ms: 60_000,
            ..Default::default()
        },
    ));

    coalescing.emit_job_progress(progress_event("job-a", 0, 2));
    coalescing.emit_job_progress(progress_event("job-b", 0, 2));

    let a_emitter = coalescing.clone();
    let first_pending = thread::spawn(move || {
        thread::sleep(Duration::from_millis(10));
        a_emitter.emit_job_progress(progress_event("job-a", 1, 2));
    });

    let b_emitter = coalescing.clone();
    let second_pending = thread::spawn(move || {
        thread::sleep(Duration::from_millis(20));
        b_emitter.emit_job_progress(progress_event("job-b", 1, 2));
    });

    first_pending.join().unwrap();
    second_pending.join().unwrap();

    coalescing.emit_job_completed(completed_event("job-b"));

    assert_eq!(
        recorder.events(),
        vec![
            ObservedEmitterEvent::Progress {
                job_id: "job-a".into(),
                current_step: 0,
            },
            ObservedEmitterEvent::Progress {
                job_id: "job-b".into(),
                current_step: 0,
            },
            ObservedEmitterEvent::Progress {
                job_id: "job-a".into(),
                current_step: 1,
            },
            ObservedEmitterEvent::Progress {
                job_id: "job-b".into(),
                current_step: 1,
            },
            ObservedEmitterEvent::Completed {
                job_id: "job-b".into(),
            },
        ],
    );
}

#[allow(deprecated)]
#[test]
fn test_emitter_config_defaults() {
    let config = EmitterConfig::default();
    assert_eq!(config.buffer_size, 256);
    assert_eq!(config.drop_policy, DropPolicy::DropNewest);
    assert_eq!(config.coalesce_interval_ms, 50);
    // include_trace_id defaults to false — legacy flag, use include_trace_ctx instead
    assert!(!config.include_trace_id);
    assert!(config.include_trace_ctx);
}

#[test]
fn test_drop_policy_variants() {
    let oldest = DropPolicy::DropOldest;
    let newest = DropPolicy::DropNewest;
    let block = DropPolicy::Block;
    assert_ne!(oldest, newest);
    assert_ne!(newest, block);
}

// ── Canonical trace context tests ───────────────────

#[test]
fn test_trace_ctx_from_event_trace_id() {
    // None yields None
    let ctx = tauri_queue::trace_ctx_from_event_trace_id(&None);
    assert!(ctx.is_none());

    // Some yields a TraceCtx with the same trace_id
    let ctx = tauri_queue::trace_ctx_from_event_trace_id(&Some("legacy-abc".into()));
    let ctx = ctx.unwrap();
    assert_eq!(ctx.trace_id, "legacy-abc");
    assert!(ctx.parent_id.is_none());
    assert!(ctx.baggage.is_empty());
}

#[test]
fn test_include_trace_ctx_precedence_over_include_trace_id() {
    // When include_trace_id=false but include_trace_ctx=true,
    // the trace string must be preserved (include_trace_ctx wins).

    /// Emitter that captures the last trace_id seen on a started event.
    struct TraceCapture {
        last_trace_id: Mutex<Option<Option<String>>>,
    }
    impl TraceCapture {
        fn new() -> Self {
            Self {
                last_trace_id: Mutex::new(None),
            }
        }
    }
    impl QueueEventEmitter for TraceCapture {
        fn emit_job_started(&self, event: JobStartedEvent) {
            *self.last_trace_id.lock().unwrap() = Some(event.trace_id.clone());
        }
        fn emit_job_completed(&self, _: JobCompletedEvent) {}
        fn emit_job_failed(&self, _: JobFailedEvent) {}
        fn emit_job_progress(&self, _: JobProgressEvent) {}
        fn emit_job_cancelled(&self, _: JobCancelledEvent) {}
    }

    let capture = Arc::new(TraceCapture::new());

    // Case 1: include_trace_id=false, include_trace_ctx=true => trace preserved
    let config = EmitterConfig {
        include_trace_id: false,
        include_trace_ctx: true,
        ..Default::default()
    };
    let coalescing = CoalescingEmitter::new(capture.clone(), config);
    coalescing.emit_job_started(JobStartedEvent {
        job_id: "j1".into(),
        trace_id: Some("my-trace".into()),
        worker_id: None,
        attempt_count: None,
        status: None,
        trace_ctx: None,
        attempt_id: None,
        trial_id: None,
    });
    let captured = capture.last_trace_id.lock().unwrap().clone();
    assert_eq!(captured, Some(Some("my-trace".into())),
        "include_trace_ctx=true should preserve trace even when include_trace_id=false");

    // Case 2: include_trace_id=false, include_trace_ctx=false => trace stripped
    let config = EmitterConfig {
        include_trace_id: false,
        include_trace_ctx: false,
        ..Default::default()
    };
    let coalescing = CoalescingEmitter::new(capture.clone(), config);
    coalescing.emit_job_started(JobStartedEvent {
        job_id: "j2".into(),
        trace_id: Some("my-trace-2".into()),
        worker_id: None,
        attempt_count: None,
        status: None,
        trace_ctx: None,
        attempt_id: None,
        trial_id: None,
    });
    let captured = capture.last_trace_id.lock().unwrap().clone();
    assert_eq!(captured, Some(None),
        "both flags false should strip trace_id");

    // Case 3: include_trace_id=true, include_trace_ctx=false => trace preserved (legacy compat)
    let config = EmitterConfig {
        include_trace_id: true,
        include_trace_ctx: false,
        ..Default::default()
    };
    let coalescing = CoalescingEmitter::new(capture.clone(), config);
    coalescing.emit_job_started(JobStartedEvent {
        job_id: "j3".into(),
        trace_id: Some("my-trace-3".into()),
        worker_id: None,
        attempt_count: None,
        status: None,
        trace_ctx: None,
        attempt_id: None,
        trial_id: None,
    });
    let captured = capture.last_trace_id.lock().unwrap().clone();
    assert_eq!(captured, Some(Some("my-trace-3".into())),
        "include_trace_id=true alone should still preserve trace");
}

#[test]
fn test_trace_ctx_re_export() {
    // Verify that TraceCtx is re-exported and usable
    let ctx = tauri_queue::TraceCtx::from_legacy_trace_id("test-trace-123");
    assert_eq!(ctx.to_legacy_trace_id(), "test-trace-123");
}
