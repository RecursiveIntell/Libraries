#![allow(dead_code)]
#![allow(clippy::expect_used)]

#[path = "../src/cli.rs"]
mod cli;
#[path = "../src/daemon.rs"]
mod daemon;
#[path = "../src/fs_security.rs"]
mod fs_security;
#[path = "../src/lifecycle.rs"]
mod lifecycle;
#[path = "../src/migrations.rs"]
mod migrations;
#[path = "../src/owner_lock.rs"]
mod owner_lock;

#[test]
fn second_daemon_fails_before_opening_database() {
    let d = tempfile::tempdir().unwrap();
    let (_lock, _db) = daemon::open_owned(d.path(), "a").unwrap();
    let e = daemon::DaemonLock::acquire(d.path()).unwrap_err();
    assert_eq!(e.code(), "DATA_DIR_ALREADY_OWNED");
}

#[test]
fn durable_proxy_missing_daemon_is_typed_and_does_not_create_database() {
    let root = tempfile::tempdir().unwrap();
    let data_dir = root.path().join("data");
    let runtime_dir = root.path().join("runtime");
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_agent-graph-mcp"))
        .args([
            "--data-dir",
            data_dir.to_str().unwrap(),
            "--runtime-dir",
            runtime_dir.to_str().unwrap(),
        ])
        .output()
        .expect("proxy binary should launch");
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("DAEMON_UNAVAILABLE"), "stderr: {stderr}");
    assert!(
        !data_dir.join("agent-graph.db").exists(),
        "proxy must not create a database when daemon is absent"
    );
}

#[test]
fn no_arguments_require_an_explicit_mode() {
    let error = cli::parse_args(&[]).expect_err("no-args startup must fail closed");
    assert_eq!(error.exit_code, 2);
    assert!(error.message.contains("MODE_REQUIRED"));
}

#[test]
fn timeout_is_completion_unknown_and_requests_cancel() {
    let d = lifecycle::synchronous_timeout();
    assert!(d.completion_unknown && d.cancellation_requested);
}
