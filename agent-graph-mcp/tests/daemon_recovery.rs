//! Process-boundary test: daemon starts, acquires lock, second daemon rejected.
use agent_graph_mcp::daemon;
use tempfile::tempdir;

#[test]
fn second_daemon_against_same_data_dir_is_rejected() {
    let dir = tempdir().expect("temp dir");
    let data = dir.path();
    // First daemon acquires the lock
    let (_lock_a, _conn_a) = daemon::open_owned(data, "daemon-a").expect("first daemon acquires");
    // Second daemon against the same data dir must fail
    let result = daemon::open_owned(data, "daemon-b");
    assert!(
        result.is_err(),
        "second daemon must not acquire the same data directory"
    );
    let err = result.unwrap_err();
    assert_eq!(err.code(), "DATA_DIR_ALREADY_OWNED");
}

#[test]
fn releasing_lock_allows_new_daemon() {
    let dir = tempdir().expect("temp dir");
    let data = dir.path();
    {
        let (_lock, _conn) = daemon::open_owned(data, "daemon-a").expect("first daemon");
    }
    // Lock dropped; new daemon can acquire
    let (_lock_b, _conn_b) =
        daemon::open_owned(data, "daemon-b").expect("second daemon after drop");
}

#[test]
fn daemon_identity_is_unique_and_monotonic() {
    let dir = tempdir().expect("temp dir");
    let data = dir.path();
    let (_lock_a, conn_a) = daemon::open_owned(data, "daemon-a").expect("first daemon");
    let id_a = daemon::identity(&conn_a).expect("identity a");
    assert_eq!(id_a.generation, 1);
    drop(_lock_a);
    drop(conn_a);
    let (_lock_b, conn_b) = daemon::open_owned(data, "daemon-b").expect("second daemon");
    let id_b = daemon::identity(&conn_b).expect("identity b");
    assert_eq!(id_b.generation, 2);
    assert_ne!(id_a.instance_id, id_b.instance_id);
}
