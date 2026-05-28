use aidens_cli::{daemon_command, DaemonCommand};
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

fn temp_root(name: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after unix epoch")
        .as_nanos();
    let root = std::env::temp_dir().join(format!(
        "aidens-phase-09-{name}-{}-{nonce}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&root);
    root
}

fn cleanup(root: &Path) {
    let _ = std::fs::remove_dir_all(root);
}

fn json(output: String) -> Value {
    serde_json::from_str(&output).expect("daemon command should return JSON")
}

#[test]
fn daemon_smoke_blocks_risky_wake_but_keeps_read_only_workflow_usable() {
    let root = temp_root("daemon-smoke");
    std::fs::create_dir_all(&root).unwrap();
    let root_arg = root.display().to_string();
    let due_at = "2026-05-01T00:00:00Z".parse().unwrap();

    let first = json(
        daemon_command(DaemonCommand::Schedule {
            root: root_arg.clone(),
            name: "phase-09-smoke".into(),
            owner: "daemon-a".into(),
            schedule_id: "once".into(),
            occurrence_key: "same-occurrence".into(),
            due_at,
            payload: r#"{"task":"read-only-refresh"}"#.into(),
            risk: "read-only".into(),
        })
        .unwrap(),
    );
    assert_eq!(first["enqueued"], true);
    assert_eq!(first["queue_hop_receipt"]["hop"], "enqueued");

    let duplicate = json(
        daemon_command(DaemonCommand::Schedule {
            root: root_arg.clone(),
            name: "phase-09-smoke".into(),
            owner: "daemon-a".into(),
            schedule_id: "once".into(),
            occurrence_key: "same-occurrence".into(),
            due_at,
            payload: r#"{"task":"read-only-refresh"}"#.into(),
            risk: "read-only".into(),
        })
        .unwrap(),
    );
    assert_eq!(duplicate["enqueued"], false);
    assert!(duplicate["duplicate_suppression_receipt"]["reason_codes"]
        .as_array()
        .unwrap()
        .iter()
        .any(|code| code == "duplicate-logical-job-suppressed"));

    let lease = json(
        daemon_command(DaemonCommand::Lease {
            root: root_arg.clone(),
            name: "phase-09-smoke".into(),
            owner: "daemon-a".into(),
            ttl_seconds: 60,
        })
        .unwrap(),
    );
    assert_eq!(lease["lease"]["active"], true);
    assert_eq!(lease["queue_hop_receipt"]["hop"], "lease-acquired");

    let safe_mode = json(
        daemon_command(DaemonCommand::SafeMode {
            root: root_arg.clone(),
            name: "phase-09-smoke".into(),
            owner: "daemon-a".into(),
            enabled: true,
            reason: "phase-09-smoke-safe-mode".into(),
        })
        .unwrap(),
    );
    assert_eq!(safe_mode["enabled"], true);
    assert_eq!(safe_mode["operation"], "entered");

    let blocked = json(
        daemon_command(DaemonCommand::Wake {
            root: root_arg.clone(),
            name: "phase-09-smoke".into(),
            owner: "daemon-a".into(),
            source: "filesystem".into(),
            signal_key: "risky-shell".into(),
            payload: r#"{"cmd":"cargo test"}"#.into(),
            risk: "shell".into(),
        })
        .unwrap(),
    );
    assert_eq!(blocked["enqueued"], false);
    assert_eq!(
        blocked["safe_mode_receipt"]["operation"],
        "blocked-risky-job"
    );

    let read_only = json(
        daemon_command(DaemonCommand::Wake {
            root: root_arg.clone(),
            name: "phase-09-smoke".into(),
            owner: "daemon-a".into(),
            source: "filesystem".into(),
            signal_key: "inspect-readme".into(),
            payload: r#"{"path":"README.md"}"#.into(),
            risk: "read-only".into(),
        })
        .unwrap(),
    );
    assert_eq!(read_only["enqueued"], true);

    let drained = json(
        daemon_command(DaemonCommand::Drain {
            root: root_arg.clone(),
            name: "phase-09-smoke".into(),
            owner: "daemon-a".into(),
            reason: "phase-09-smoke-drain".into(),
        })
        .unwrap(),
    );
    assert!(drained.as_array().unwrap().len() >= 2);

    let snapshot = json(
        daemon_command(DaemonCommand::List {
            root: root_arg,
            name: "phase-09-smoke".into(),
            owner: "daemon-a".into(),
        })
        .unwrap(),
    );
    assert_eq!(snapshot["safe_mode_enabled"], true);
    assert!(snapshot["jobs"]
        .as_array()
        .unwrap()
        .iter()
        .all(|job| job["state"] == "cancelled"));

    cleanup(&root);
}
