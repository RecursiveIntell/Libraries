//! Integration tests for bitemporal-runtime.

use bitemporal_runtime::{
    append_supersede, as_of_query, temporal_snapshot, BitemporalRecord, InMemoryDb,
};
use chrono::TimeZone;

/// Test 1: append_supersede_creates_receipt — call append_supersede, verify receipt returned.
#[test]
fn append_supersede_creates_receipt() {
    let mut records: Vec<BitemporalRecord<&str>> = Vec::new();

    let t0 = chrono::Utc.timestamp_opt(1000, 0).unwrap();
    let v1 = BitemporalRecord {
        id: "ep1".to_string(),
        valid_time: t0,
        recorded_time: t0,
        value: "version1",
    };

    // First insert — no prior versions, no receipts
    let receipts = append_supersede(&mut records, v1).unwrap();
    assert!(receipts.is_empty());
    assert_eq!(records.len(), 1);

    // Second insert — prior version exists, should get a receipt
    let t1 = chrono::Utc.timestamp_opt(2000, 0).unwrap();
    let v2 = BitemporalRecord {
        id: "ep1".to_string(),
        valid_time: t0,
        recorded_time: t1,
        value: "version2",
    };

    let receipts = append_supersede(&mut records, v2).unwrap();
    assert_eq!(receipts.len(), 1);
    assert_eq!(receipts[0].superseded.superseded_id, "ep1");
    assert_eq!(receipts[0].superseding_id, "ep1");
    assert_eq!(records.len(), 2);
}

/// Test 2: as_of_query_returns_correct_version — insert 3 versions, query as_of each time point.
#[test]
fn as_of_query_returns_correct_version() {
    let t0 = chrono::Utc.timestamp_opt(1000, 0).unwrap();
    let t1 = chrono::Utc.timestamp_opt(2000, 0).unwrap();
    let t2 = chrono::Utc.timestamp_opt(3000, 0).unwrap();

    let records: Vec<BitemporalRecord<&str>> = vec![
        BitemporalRecord {
            id: "ep1".to_string(),
            valid_time: t0,
            recorded_time: t0,
            value: "v1",
        },
        BitemporalRecord {
            id: "ep1".to_string(),
            valid_time: t0,
            recorded_time: t1,
            value: "v2",
        },
        BitemporalRecord {
            id: "ep1".to_string(),
            valid_time: t1,
            recorded_time: t2,
            value: "v3",
        },
    ];

    // Query at t0 — should get v1
    let result = as_of_query(&records, t0, t0);
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].value, "v1");

    // Query at t1 — should get v2 (latest as of recorded_time t1)
    let result = as_of_query(&records, t0, t1);
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].value, "v2");

    // Query at t2 — should get v3
    let result = as_of_query(&records, t1, t2);
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].value, "v3");
}

/// Test 3: supersession_chain_preserves_history — chain 10 supersessions, verify all retrievable.
#[test]
fn supersession_chain_preserves_history() {
    let mut records: Vec<BitemporalRecord<i32>> = Vec::new();

    let base_time = chrono::Utc.timestamp_opt(1000, 0).unwrap();

    // Insert 10 versions sequentially
    for i in 0..10 {
        let t = chrono::Utc
            .timestamp_opt((1000 + i * 100) as i64, 0)
            .unwrap();
        let record = BitemporalRecord {
            id: "chain_test".to_string(),
            valid_time: base_time,
            recorded_time: t,
            value: i,
        };
        append_supersede(&mut records, record).unwrap();
    }

    assert_eq!(records.len(), 10);

    // Verify each version is retrievable at its recorded_time
    for i in 0..10 {
        let t = chrono::Utc
            .timestamp_opt((1000 + i * 100) as i64, 0)
            .unwrap();
        let result = as_of_query(&records, base_time, t);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].value, i);
    }
}

/// Test 4: temporal_snapshot_at_time — insert 5 records at different times, snapshot at mid-point.
#[test]
fn temporal_snapshot_at_time() {
    let mut db = InMemoryDb::new();

    let times: Vec<_> = (0..5)
        .map(|i| {
            chrono::Utc
                .timestamp_opt((1000 + i * 200) as i64, 0)
                .unwrap()
        })
        .collect();

    // Insert 5 different records at different times, using () as value type to match InMemoryDb
    for i in 0..5 {
        let record = BitemporalRecord {
            id: format!("record_{}", i),
            valid_time: times[0],
            recorded_time: times[i],
            value: (),
        };
        db.insert(record);
    }

    // Snapshot at mid-point (time index 2)
    let mid_time = times[2];
    let snapshot = db.snapshot_at(mid_time);

    // Should get records that were inserted up to mid_time
    assert_eq!(snapshot.len(), 3);

    // record_0, record_1, and record_2 were inserted at or before mid_time
    assert!(snapshot.iter().any(|r| r.id == "record_0"));
    assert!(snapshot.iter().any(|r| r.id == "record_1"));
    assert!(snapshot.iter().any(|r| r.id == "record_2"));
    // record_3 and record_4 not yet inserted, so not in snapshot
    assert!(!snapshot.iter().any(|r| r.id == "record_3"));
    assert!(!snapshot.iter().any(|r| r.id == "record_4"));
}

/// Test 5: invalid_recorded_time_returns_empty — query before any record existed.
#[test]
fn invalid_recorded_time_returns_empty() {
    let records: Vec<BitemporalRecord<&str>> = vec![BitemporalRecord {
        id: "ep1".to_string(),
        valid_time: chrono::Utc.timestamp_opt(2000, 0).unwrap(),
        recorded_time: chrono::Utc.timestamp_opt(2000, 0).unwrap(),
        value: "v1",
    }];

    // Query at a recorded_time BEFORE any record existed
    let before = chrono::Utc.timestamp_opt(500, 0).unwrap();
    let result = as_of_query(&records, before, before);

    // Should return empty — no records existed at that time
    assert_eq!(result.len(), 0);
}
