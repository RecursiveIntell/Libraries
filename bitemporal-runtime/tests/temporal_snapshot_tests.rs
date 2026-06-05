//! Tests for the public `temporal_snapshot` function on a `Vec` of
//! bitemporal records. The pre-existing supersession_tests.rs exercises
//! the InMemoryDb abstraction; these tests cover the slice-based public
//! function directly, which is what the type's pub API actually exposes.
//!
//! `bitemporal-runtime` has **zero** analogues on crates.io (verified
//! 2026-06-02). This is the canonical "what did we believe at time T
//! about the fact that was valid at time V" query — a foundational
//! capability for audit, regulatory, and scientific-reproducibility
//! workflows.

#![allow(clippy::expect_used)] // test code — expect() on Result/Option is the idiomatic pattern

use bitemporal_runtime::{temporal_snapshot, BitemporalRecord};
use chrono::{TimeZone, Utc};

fn make_record(id: &str, valid_time: i64, recorded_time: i64) -> BitemporalRecord<String> {
    BitemporalRecord {
        id: id.to_string(),
        valid_time: Utc.timestamp_opt(valid_time, 0).unwrap(),
        recorded_time: Utc.timestamp_opt(recorded_time, 0).unwrap(),
        value: format!("value-of-{id}"),
    }
}

#[test]
fn temporal_snapshot_returns_one_record_per_id_at_as_of_time() {
    // Three records for the same id, with different recorded_times —
    // a classic bitemporal history. As of time T=1500, only the
    // earliest recorded version was known (it was recorded at T=1000).
    let records = vec![
        make_record("alpha", 500, 1000), // valid at 500, known since 1000
        make_record("alpha", 500, 1500), // same valid_time, re-recorded at 1500
        make_record("alpha", 500, 2000), // re-recorded again at 2000
        make_record("beta", 700, 1200),  // different id, valid at 700, known since 1200
    ];

    // As of T=1500, alpha is known (it was recorded at T=1000, which
    // is <= 1500). The latest recorded version of alpha as of T=1500
    // is the T=1500 entry itself. beta is also known (recorded at
    // 1200 <= 1500). So we expect 2 records, one per id, with the
    // alpha record being the T=1500 one.
    let as_of = Utc.timestamp_opt(1500, 0).unwrap();
    let snapshot = temporal_snapshot(&records, as_of);

    assert_eq!(snapshot.len(), 2, "one record per unique id");
    let alpha = snapshot
        .iter()
        .find(|r| r.id == "alpha")
        .expect("alpha must be in the snapshot");
    let beta = snapshot
        .iter()
        .find(|r| r.id == "beta")
        .expect("beta must be in the snapshot");
    assert_eq!(
        alpha.recorded_time,
        Utc.timestamp_opt(1500, 0).unwrap(),
        "as-of-1500 must return the version of alpha that was current at T=1500"
    );
    assert_eq!(beta.recorded_time, Utc.timestamp_opt(1200, 0).unwrap());
}

#[test]
fn temporal_snapshot_returns_empty_for_time_before_any_recording() {
    // No record was ever recorded before T=500. As of T=400, we knew
    // nothing — the snapshot must be empty.
    let records = vec![
        make_record("alpha", 100, 1000),
        make_record("beta", 200, 1200),
    ];
    let as_of = Utc.timestamp_opt(400, 0).unwrap();
    let snapshot = temporal_snapshot(&records, as_of);
    assert!(
        snapshot.is_empty(),
        "no records were recorded before T=500, so T=400 returns empty"
    );
}

#[test]
fn temporal_snapshot_preserves_valid_time_through_query() {
    // valid_time is the time the fact was true in the world;
    // recorded_time is the time we learned about it. The query
    // filters by recorded_time (when we knew it) but must NOT
    // mutate or filter valid_time — a downstream consumer asking
    // "what did we believe was true at V?" must be able to inspect
    // valid_time on each returned record.
    let records = vec![make_record("alpha", 500, 1000)];
    let as_of = Utc.timestamp_opt(2000, 0).unwrap();
    let snapshot = temporal_snapshot(&records, as_of);
    assert_eq!(snapshot.len(), 1);
    let alpha = &snapshot[0];
    assert_eq!(alpha.valid_time, Utc.timestamp_opt(500, 0).unwrap());
    assert_eq!(alpha.recorded_time, Utc.timestamp_opt(1000, 0).unwrap());
    assert_eq!(alpha.value, "value-of-alpha");
}

#[test]
fn temporal_snapshot_handles_duplicate_id_with_advancing_recorded_times() {
    // Walk-forward scenario: a fact becomes known, then is superseded
    // by a corrected version. Both are bitemporal records. As of any
    // given time, only the latest-recorded version is "what we believed".
    let records = vec![
        make_record("fact", 100, 1000), // initial belief
        make_record("fact", 100, 1500), // corrected belief
        make_record("fact", 100, 2000), // final correction
    ];

    let as_of_1200 = temporal_snapshot(&records, Utc.timestamp_opt(1200, 0).unwrap());
    assert_eq!(as_of_1200.len(), 1);
    assert_eq!(
        as_of_1200[0].recorded_time,
        Utc.timestamp_opt(1000, 0).unwrap(),
        "as of T=1200, only the T=1000 recording was known"
    );

    let as_of_1700 = temporal_snapshot(&records, Utc.timestamp_opt(1700, 0).unwrap());
    assert_eq!(
        as_of_1700[0].recorded_time,
        Utc.timestamp_opt(1500, 0).unwrap()
    );

    let as_of_2500 = temporal_snapshot(&records, Utc.timestamp_opt(2500, 0).unwrap());
    assert_eq!(
        as_of_2500[0].recorded_time,
        Utc.timestamp_opt(2000, 0).unwrap()
    );
}
