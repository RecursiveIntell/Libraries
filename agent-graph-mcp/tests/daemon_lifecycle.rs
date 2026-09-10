#![cfg(unix)]
#![allow(clippy::expect_used)]

use std::os::unix::fs::MetadataExt;
use std::process::{Child, Command, Output, Stdio};
use std::thread;
use std::time::{Duration, Instant};

fn daemon_binary() -> &'static str {
    env!("CARGO_BIN_EXE_agent-graph-mcpd")
}

fn launch(root: &std::path::Path) -> (Child, std::path::PathBuf, std::path::PathBuf) {
    let data_dir = root.join("data");
    let runtime_dir = root.join("runtime");
    let child = Command::new(daemon_binary())
        .args([
            "--data-dir",
            data_dir.to_str().unwrap(),
            "--runtime-dir",
            runtime_dir.to_str().unwrap(),
        ])
        .env("RUST_LOG", "off")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("daemon binary should launch");
    (child, data_dir, runtime_dir)
}

fn socket_path(runtime_dir: &std::path::Path) -> std::path::PathBuf {
    runtime_dir
        .join("agent-graph")
        .join("default")
        .join("daemon.sock")
}

fn wait_for_socket(path: &std::path::Path) {
    let deadline = Instant::now() + Duration::from_secs(10);
    while Instant::now() < deadline {
        if path.exists() {
            return;
        }
        thread::sleep(Duration::from_millis(20));
    }
    panic!("daemon socket did not appear: {}", path.display());
}

fn terminate_gracefully(child: Child) -> Output {
    let status = Command::new("kill")
        .args(["-TERM", &child.id().to_string()])
        .status()
        .expect("kill command should launch");
    assert!(status.success(), "SIGTERM delivery failed: {status}");
    child
        .wait_with_output()
        .expect("daemon should exit after SIGTERM")
}

#[test]
fn daemon_starts_owns_private_socket_records_and_releases_cleanly() {
    let root = tempfile::tempdir().unwrap();
    let (child, data_dir, runtime_dir) = launch(root.path());
    let socket = socket_path(&runtime_dir);
    wait_for_socket(&socket);

    let runtime_meta = std::fs::metadata(runtime_dir.join("agent-graph").join("default")).unwrap();
    assert_eq!(runtime_meta.mode() & 0o777, 0o700);
    let socket_meta = std::fs::metadata(&socket).unwrap();
    assert_eq!(socket_meta.mode() & 0o777, 0o600);
    assert!(data_dir.join("agent-graph.db").exists());

    let db = rusqlite::Connection::open(data_dir.join("agent-graph.db")).unwrap();
    let instance_count: i64 = db
        .query_row("SELECT count(*) FROM server_instances", [], |row| {
            row.get(0)
        })
        .unwrap();
    assert_eq!(instance_count, 1);

    let output = terminate_gracefully(child);
    assert!(
        output.status.success(),
        "daemon stderr: {:?}",
        output.stderr
    );
    assert!(
        !socket.exists(),
        "daemon socket must be removed on shutdown"
    );

    let db = rusqlite::Connection::open(data_dir.join("agent-graph.db")).unwrap();
    let stopped: Option<String> = db
        .query_row(
            "SELECT stopped_at FROM server_instances ORDER BY started_at DESC LIMIT 1",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert!(stopped.is_some(), "daemon must record a stop timestamp");
}

#[test]
fn second_daemon_is_rejected_before_store_ownership() {
    let root = tempfile::tempdir().unwrap();
    let (first, data_dir, runtime_dir) = launch(root.path());
    wait_for_socket(&socket_path(&runtime_dir));

    let second = Command::new(daemon_binary())
        .args([
            "--data-dir",
            data_dir.to_str().unwrap(),
            "--runtime-dir",
            runtime_dir.to_str().unwrap(),
        ])
        .env("RUST_LOG", "off")
        .output()
        .expect("second daemon should launch and reject ownership");
    assert!(!second.status.success());
    assert!(
        String::from_utf8_lossy(&second.stderr).contains("DATA_DIR_ALREADY_OWNED"),
        "stderr: {:?}",
        second.stderr
    );

    let output = terminate_gracefully(first);
    assert!(
        output.status.success(),
        "daemon stderr: {:?}",
        output.stderr
    );
}
