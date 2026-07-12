//! SQLite-backed bitemporal store tests.

#![cfg(feature = "sqlite")]
#![allow(clippy::expect_used)]

use bitemporal_runtime::{BitemporalRecord, SqliteDb};

#[test]
fn sqlite_preserves_multiple_subsecond_updates() {
    let db = SqliteDb::open_in_memory().unwrap();
    let base = chrono::DateTime::from_timestamp(1000, 100).unwrap();
    for (n, value) in [(100, "a"), (200, "b")] {
        db.insert(BitemporalRecord {
            id: "x".into(),
            valid_time: chrono::DateTime::from_timestamp(1000, n).unwrap(),
            recorded_time: chrono::DateTime::from_timestamp(1000, n).unwrap(),
            value: serde_json::json!(value),
        })
        .unwrap();
    }
    let snapshot = db
        .snapshot_at(base + chrono::Duration::nanoseconds(100))
        .unwrap();
    assert_eq!(snapshot[0].value, serde_json::json!("b"));
}

#[test]
fn sqlite_migrates_legacy_seconds_schema() {
    let file = tempfile::NamedTempFile::new().unwrap();
    {
        let conn = rusqlite::Connection::open(file.path()).unwrap();
        conn.execute_batch("CREATE TABLE bitemporal_records (record_id TEXT NOT NULL, valid_time INTEGER NOT NULL, recorded_time INTEGER NOT NULL, superseded_by TEXT, value_json BLOB NOT NULL, PRIMARY KEY(record_id, valid_time, recorded_time)); INSERT INTO bitemporal_records VALUES ('x', 1000, 1000, NULL, X'31');").unwrap();
    }
    let db = SqliteDb::open(file.path()).unwrap();
    let snapshot = db
        .snapshot_at(chrono::DateTime::from_timestamp(1000, 0).unwrap())
        .unwrap();
    assert_eq!(snapshot.len(), 1);
    assert_eq!(
        snapshot[0].recorded_time,
        chrono::DateTime::from_timestamp(1000, 0).unwrap()
    );
}
use chrono::TimeZone;

fn record(
    id: &str,
    valid: i64,
    recorded: i64,
    value: serde_json::Value,
) -> BitemporalRecord<serde_json::Value> {
    BitemporalRecord {
        id: id.to_string(),
        valid_time: chrono::Utc.timestamp_opt(valid, 0).unwrap(),
        recorded_time: chrono::Utc.timestamp_opt(recorded, 0).unwrap(),
        value,
    }
}

#[test]
fn sqlite_db_insert_first_version_returns_zero_superseded() {
    let db = SqliteDb::open_in_memory().unwrap();
    let superseded = db
        .insert(record("alpha", 100, 1000, serde_json::json!({"v": 1})))
        .unwrap();
    assert_eq!(superseded, 0, "first version supersedes nothing");
}

#[test]
fn sqlite_db_insert_second_version_supersedes_first() {
    let db = SqliteDb::open_in_memory().unwrap();
    db.insert(record("alpha", 100, 1000, serde_json::json!({"v": 1})))
        .unwrap();
    let superseded = db
        .insert(record("alpha", 100, 2000, serde_json::json!({"v": 2})))
        .unwrap();
    assert_eq!(superseded, 1, "second version supersedes the one prior row");
}

#[test]
fn sqlite_db_snapshot_returns_current_state_at_time() {
    let db = SqliteDb::open_in_memory().unwrap();
    db.insert(record("alpha", 100, 1000, serde_json::json!({"v": 1})))
        .unwrap();
    db.insert(record("alpha", 100, 2000, serde_json::json!({"v": 2})))
        .unwrap();
    db.insert(record("beta", 200, 1500, serde_json::json!({"v": 1})))
        .unwrap();

    // At T=1500 we knew alpha v1 and beta v1.
    let snap = db
        .snapshot_at(chrono::Utc.timestamp_opt(1500, 0).unwrap())
        .unwrap();
    assert_eq!(snap.len(), 2);
    let alpha = snap.iter().find(|r| r.id == "alpha").unwrap();
    assert_eq!(alpha.value, serde_json::json!({"v": 1}));
    assert_eq!(alpha.recorded_time.timestamp(), 1000);

    // At T=2500 we know alpha v2 and beta v1.
    let snap = db
        .snapshot_at(chrono::Utc.timestamp_opt(2500, 0).unwrap())
        .unwrap();
    assert_eq!(snap.len(), 2);
    let alpha = snap.iter().find(|r| r.id == "alpha").unwrap();
    assert_eq!(alpha.value, serde_json::json!({"v": 2}));
    assert_eq!(alpha.recorded_time.timestamp(), 2000);
}

#[test]
fn sqlite_db_persists_across_open_close() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("bitemporal.db");

    {
        let db = SqliteDb::open(&path).unwrap();
        db.insert(record("alpha", 100, 1000, serde_json::json!({"v": 1})))
            .unwrap();
    }

    // Re-open and verify the row persisted.
    let db2 = SqliteDb::open(&path).unwrap();
    let snap = db2
        .snapshot_at(chrono::Utc.timestamp_opt(2000, 0).unwrap())
        .unwrap();
    assert_eq!(snap.len(), 1);
    assert_eq!(snap[0].id, "alpha");
    assert_eq!(snap[0].value, serde_json::json!({"v": 1}));
}

#[test]
fn sqlite_db_snapshot_at_time_before_any_recording_returns_empty() {
    let db = SqliteDb::open_in_memory().unwrap();
    db.insert(record("alpha", 100, 1000, serde_json::json!({"v": 1})))
        .unwrap();
    let snap = db
        .snapshot_at(chrono::Utc.timestamp_opt(500, 0).unwrap())
        .unwrap();
    assert!(
        snap.is_empty(),
        "no rows were recorded before T=1000, so T=500 returns empty"
    );
}
