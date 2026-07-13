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

#[test]
fn legacy_migration_event_ids_ignore_rowid_and_emit_durable_receipt() {
    fn migrated_event_id(dummy_rows: usize) -> (String, bitemporal_runtime::MigrationReceipt) {
        let file = tempfile::NamedTempFile::new().unwrap();
        {
            let conn = rusqlite::Connection::open(file.path()).unwrap();
            conn.execute_batch("CREATE TABLE bitemporal_records (record_id TEXT NOT NULL, valid_time INTEGER NOT NULL, recorded_time INTEGER NOT NULL, superseded_by TEXT, value_json BLOB NOT NULL, PRIMARY KEY(record_id, valid_time, recorded_time));").unwrap();
            for index in 0..dummy_rows {
                conn.execute(
                    "INSERT INTO bitemporal_records VALUES (?1, 1, 1, NULL, X'30')",
                    [format!("dummy-{index}")],
                )
                .unwrap();
            }
            conn.execute(
                "INSERT INTO bitemporal_records VALUES ('stable', 1000, 1000, NULL, X'31')",
                [],
            )
            .unwrap();
            for index in 0..dummy_rows {
                conn.execute(
                    "DELETE FROM bitemporal_records WHERE record_id = ?1",
                    [format!("dummy-{index}")],
                )
                .unwrap();
            }
        }
        let db = SqliteDb::open(file.path()).unwrap();
        let event_id = db.event_id_for_record("stable").unwrap().unwrap();
        let receipt = db.migration_receipts().unwrap().pop().unwrap();
        (event_id, receipt)
    }

    let (first_id, first_receipt) = migrated_event_id(0);
    let (shifted_id, shifted_receipt) = migrated_event_id(3);
    assert_eq!(first_id, shifted_id);
    assert_eq!(
        first_receipt.schema_version,
        "bitemporal_migration_receipt_v1"
    );
    assert_eq!(
        shifted_receipt.schema_version,
        "bitemporal_migration_receipt_v1"
    );
    assert_eq!(first_receipt.from_schema_version, "seconds_v1");
    assert_eq!(first_receipt.to_schema_version, "canonical_event_v3");
    assert_eq!(first_receipt.migrated_row_count, 1);
}

#[test]
fn legacy_seconds_rows_use_the_same_canonical_event_id_as_new_rows() {
    let file = tempfile::NamedTempFile::new().unwrap();
    {
        let conn = rusqlite::Connection::open(file.path()).unwrap();
        conn.execute_batch("CREATE TABLE bitemporal_records (record_id TEXT NOT NULL, valid_time INTEGER NOT NULL, recorded_time INTEGER NOT NULL, superseded_by TEXT, value_json BLOB NOT NULL, PRIMARY KEY(record_id, valid_time, recorded_time)); INSERT INTO bitemporal_records VALUES ('stable', 1000, 1001, NULL, X'7B2261223A317D');").unwrap();
    }
    let expected = BitemporalRecord {
        id: "stable".into(),
        valid_time: chrono::DateTime::from_timestamp(1000, 0).unwrap(),
        recorded_time: chrono::DateTime::from_timestamp(1001, 0).unwrap(),
        value: serde_json::json!({"a": 1}),
    }
    .try_event_id()
    .unwrap();

    let db = SqliteDb::open(file.path()).unwrap();
    assert_eq!(db.event_id_for_record("stable").unwrap(), Some(expected));
}

#[test]
fn event_v2_rows_are_rekeyed_and_logical_supersession_targets_are_rewritten() {
    let file = tempfile::NamedTempFile::new().unwrap();
    {
        let conn = rusqlite::Connection::open(file.path()).unwrap();
        conn.execute_batch(
            "CREATE TABLE bitemporal_records (
               event_id TEXT NOT NULL PRIMARY KEY,
               record_id TEXT NOT NULL,
               valid_time_ns INTEGER NOT NULL,
               recorded_time_ns INTEGER NOT NULL,
               superseded_by TEXT,
               value_json BLOB NOT NULL
             );
             CREATE TABLE bitemporal_schema (
               singleton INTEGER PRIMARY KEY CHECK(singleton = 1),
               schema_version TEXT NOT NULL
             );
             INSERT INTO bitemporal_schema VALUES (1, 'event_v2');
             INSERT INTO bitemporal_records VALUES
               ('old-event-a', 'x', 1000000000000, 1000000000000, 'x', X'31'),
               ('old-event-b', 'x', 1000000000000, 1001000000000, NULL, X'32');",
        )
        .unwrap();
    }

    let old = record("x", 1000, 1000, serde_json::json!(1));
    let new = record("x", 1000, 1001, serde_json::json!(2));
    let old_id = old.try_event_id().unwrap();
    let new_id = new.try_event_id().unwrap();
    let db = SqliteDb::open(file.path()).unwrap();
    assert_eq!(db.event_id_for_record("x").unwrap(), Some(new_id.clone()));

    let conn = rusqlite::Connection::open(file.path()).unwrap();
    let schema: String = conn
        .query_row("SELECT schema_version FROM bitemporal_schema", [], |row| {
            row.get(0)
        })
        .unwrap();
    let superseded_by: String = conn
        .query_row(
            "SELECT superseded_by FROM bitemporal_records WHERE event_id = ?1",
            [&old_id],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(schema, "canonical_event_v3");
    assert_eq!(superseded_by, new_id);
}

#[test]
fn sqlite_uses_the_same_valid_time_then_event_id_tie_break_as_free_queries() {
    let recorded = chrono::DateTime::from_timestamp(2000, 0).unwrap();
    let mut chosen = None;
    for nanos in 1..1000 {
        let low = BitemporalRecord {
            id: "tie".into(),
            valid_time: chrono::DateTime::from_timestamp(1000, nanos).unwrap(),
            recorded_time: recorded,
            value: serde_json::Value::Null,
        };
        let high = BitemporalRecord {
            id: "tie".into(),
            valid_time: chrono::DateTime::from_timestamp(1000, nanos + 1).unwrap(),
            recorded_time: recorded,
            value: serde_json::Value::Null,
        };
        if low.try_event_id().unwrap() > high.try_event_id().unwrap() {
            chosen = Some((low, high));
            break;
        }
    }
    let (low, high) = chosen.expect("fixture must find hash order opposing valid-time order");
    let expected =
        bitemporal_runtime::try_as_of_query(&[low.clone(), high.clone()], recorded, recorded)
            .unwrap();
    assert_eq!(expected[0].valid_time, high.valid_time);

    let db = SqliteDb::open_in_memory().unwrap();
    db.insert(low).unwrap();
    db.insert(high).unwrap();
    let actual = db.snapshot_at(recorded).unwrap();
    assert_eq!(actual[0].valid_time, expected[0].valid_time);
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
