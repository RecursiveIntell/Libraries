use aidens_contracts::{CanonicalToolSideEffectClass, JobStateV1};
use aidens_daemon_kit::DaemonControllerV1;
use chrono::Utc;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

fn temp_root(name: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after unix epoch")
        .as_nanos();
    let root = std::env::temp_dir().join(format!(
        "aidens-phase-07-{name}-{}-{nonce}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&root);
    root
}

fn cleanup(root: &Path) {
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn daemon_namespace_isolated() {
    let root = temp_root("namespace-isolated");
    let ns_a = DaemonControllerV1::namespace(&root, "same-app-name", "owner-a");
    let ns_b = DaemonControllerV1::namespace(&root, "same-app-name", "owner-b");

    assert_ne!(ns_a.namespace_id, ns_b.namespace_id);
    assert_eq!(ns_a.queue_root, ns_b.queue_root);

    let daemon_a = DaemonControllerV1::open(&root, ns_a.clone(), "owner-a").unwrap();
    let daemon_b = DaemonControllerV1::open(&root, ns_b.clone(), "owner-b").unwrap();

    let a = daemon_a
        .enqueue_schedule_occurrence(
            "daily",
            "same-logical-time",
            Utc::now(),
            serde_json::json!({"task":"refresh"}),
            CanonicalToolSideEffectClass::ReadOnly,
        )
        .unwrap();
    let b = daemon_b
        .enqueue_schedule_occurrence(
            "daily",
            "same-logical-time",
            Utc::now(),
            serde_json::json!({"task":"refresh"}),
            CanonicalToolSideEffectClass::ReadOnly,
        )
        .unwrap();

    assert!(a.enqueued);
    assert!(b.enqueued);
    assert_ne!(
        a.job.as_ref().unwrap().job_id,
        b.job.as_ref().unwrap().job_id
    );
    assert_eq!(daemon_a.snapshot().unwrap().jobs.len(), 1);
    assert_eq!(daemon_b.snapshot().unwrap().jobs.len(), 1);
    assert_eq!(
        daemon_a.snapshot().unwrap().namespace.namespace_id,
        ns_a.namespace_id
    );
    assert_eq!(
        daemon_b.snapshot().unwrap().namespace.namespace_id,
        ns_b.namespace_id
    );

    cleanup(&root);
}

#[test]
fn schedule_no_duplicate_storm() {
    let root = temp_root("duplicate-storm");
    let ns = DaemonControllerV1::namespace(&root, "schedule-storm", "daemon-a");
    let daemon = DaemonControllerV1::open(&root, ns, "daemon-a").unwrap();

    let first_schedule = daemon
        .enqueue_schedule_occurrence(
            "hourly",
            "2026-04-29T12:00:00Z",
            Utc::now(),
            serde_json::json!({"task":"sync"}),
            CanonicalToolSideEffectClass::ReadOnly,
        )
        .unwrap();
    assert!(first_schedule.enqueued);

    for _ in 0..10 {
        let duplicate = daemon
            .enqueue_schedule_occurrence(
                "hourly",
                "2026-04-29T12:00:00Z",
                Utc::now(),
                serde_json::json!({"task":"sync"}),
                CanonicalToolSideEffectClass::ReadOnly,
            )
            .unwrap();
        assert!(!duplicate.enqueued);
        assert!(duplicate.duplicate_suppression_receipt.is_some());
    }

    let first_wake = daemon
        .enqueue_wake_signal(
            "filesystem",
            "repo-change",
            serde_json::json!({"path":"Cargo.toml"}),
            CanonicalToolSideEffectClass::ReadOnly,
        )
        .unwrap();
    assert!(first_wake.enqueued);

    for _ in 0..10 {
        let duplicate = daemon
            .enqueue_wake_signal(
                "filesystem",
                "repo-change",
                serde_json::json!({"path":"Cargo.toml"}),
                CanonicalToolSideEffectClass::ReadOnly,
            )
            .unwrap();
        assert!(!duplicate.enqueued);
        assert!(duplicate.duplicate_suppression_receipt.is_some());
    }

    let snapshot = daemon.snapshot().unwrap();
    assert_eq!(snapshot.jobs.len(), 2);
    assert_eq!(
        snapshot.logical_job_count_for("schedule:"),
        0,
        "logical lookup must require the exact idempotency key"
    );

    cleanup(&root);
}

#[test]
fn restart_does_not_reenqueue_completed_jobs() {
    let root = temp_root("restart-completed");
    let ns = DaemonControllerV1::namespace(&root, "restart-completed", "daemon-a");
    let daemon = DaemonControllerV1::open(&root, ns.clone(), "daemon-a").unwrap();

    let enqueued = daemon
        .enqueue_schedule_occurrence(
            "once",
            "completed-logical-job",
            Utc::now(),
            serde_json::json!({"task":"done-once"}),
            CanonicalToolSideEffectClass::ReadOnly,
        )
        .unwrap()
        .job
        .unwrap();
    let lease = daemon.acquire_next("daemon-a", 30).unwrap().unwrap().lease;
    daemon.complete(&enqueued.job_id, &lease).unwrap();

    let restarted = DaemonControllerV1::open(&root, ns, "daemon-a").unwrap();
    let duplicate = restarted
        .enqueue_schedule_occurrence(
            "once",
            "completed-logical-job",
            Utc::now(),
            serde_json::json!({"task":"done-once"}),
            CanonicalToolSideEffectClass::ReadOnly,
        )
        .unwrap();

    assert!(!duplicate.enqueued);
    assert!(duplicate.duplicate_suppression_receipt.is_some());
    assert!(restarted.acquire_next("daemon-a", 30).unwrap().is_none());

    let snapshot = restarted.snapshot().unwrap();
    assert_eq!(snapshot.jobs.len(), 1);
    assert_eq!(
        snapshot.job(&enqueued.job_id).unwrap().state,
        JobStateV1::Completed
    );

    cleanup(&root);
}
