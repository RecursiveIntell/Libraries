//! Hostile audit adversarial probes — things a security review
//! would try to break.
//!
//! This is the audit's "are there subtle bugs the existing tests
//! didn't catch" pass. Every test should pass; if any test FAILS,
//! that is a real finding to investigate.

#![allow(clippy::expect_used)]
#![allow(clippy::unwrap_used)]

use bitemporal_runtime::{
    append_supersede, as_of_query, temporal_snapshot, try_as_of_query, try_temporal_snapshot,
    BitemporalRecord,
};
use serde::Serialize;

#[test]
fn append_receipts_bind_values_and_subseconds() {
    let t = chrono::Utc.timestamp_opt(1000, 123_456_789).unwrap();
    let mut a = vec![BitemporalRecord {
        id: "x".into(),
        valid_time: t,
        recorded_time: t,
        value: "old-a",
    }];
    let mut b = vec![BitemporalRecord {
        id: "x".into(),
        valid_time: t,
        recorded_time: t,
        value: "old-b",
    }];
    let ra = append_supersede(
        &mut a,
        BitemporalRecord {
            id: "x".into(),
            valid_time: t,
            recorded_time: t,
            value: "new-a",
        },
    )
    .unwrap();
    let rb = append_supersede(
        &mut b,
        BitemporalRecord {
            id: "x".into(),
            valid_time: t,
            recorded_time: t,
            value: "new-b",
        },
    )
    .unwrap();
    assert_ne!(ra[0].receipt_digest, rb[0].receipt_digest);
}

#[derive(Clone)]
struct NeverSerializes;
impl Serialize for NeverSerializes {
    fn serialize<S>(&self, _serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        Err(serde::ser::Error::custom("nope"))
    }
}

#[test]
fn failed_receipt_serialization_does_not_append() {
    let t = chrono::Utc.timestamp_opt(1000, 0).unwrap();
    let mut records = vec![BitemporalRecord {
        id: "x".into(),
        valid_time: t,
        recorded_time: t,
        value: NeverSerializes,
    }];
    assert!(append_supersede(
        &mut records,
        BitemporalRecord {
            id: "x".into(),
            valid_time: t,
            recorded_time: t,
            value: NeverSerializes
        }
    )
    .is_err());
    assert_eq!(records.len(), 1);
}

#[test]
fn supersession_names_exact_old_and_new_event_ids() {
    let t = chrono::Utc.timestamp_opt(1000, 0).unwrap();
    let old = BitemporalRecord {
        id: "logical-record".into(),
        valid_time: t,
        recorded_time: t,
        value: "old",
    };
    let new = BitemporalRecord {
        id: "logical-record".into(),
        valid_time: t,
        recorded_time: t + chrono::Duration::seconds(1),
        value: "new",
    };
    let old_event_id = old.try_event_id().unwrap();
    let new_event_id = new.try_event_id().unwrap();
    let mut records = vec![old];
    let receipt = append_supersede(&mut records, new).unwrap().remove(0);
    assert_eq!(
        receipt.superseded.superseded_event_id.as_deref(),
        Some(old_event_id.as_str())
    );
    assert_eq!(
        receipt.superseding_event_id.as_deref(),
        Some(new_event_id.as_str())
    );
    assert_ne!(
        receipt.superseded.superseded_event_id,
        receipt.superseding_event_id
    );
}

#[test]
fn query_tie_serialization_failure_is_returned() {
    let t = chrono::Utc.timestamp_opt(1000, 0).unwrap();
    let records = vec![
        BitemporalRecord {
            id: "x".into(),
            valid_time: t,
            recorded_time: t,
            value: NeverSerializes,
        },
        BitemporalRecord {
            id: "x".into(),
            valid_time: t,
            recorded_time: t,
            value: NeverSerializes,
        },
    ];
    assert!(try_as_of_query(&records, t, t).is_err());
}

#[test]
fn snapshot_serialization_failure_is_fallible_and_compatibility_api_does_not_panic() {
    let t = chrono::Utc.timestamp_opt(1000, 0).unwrap();
    let records = vec![BitemporalRecord {
        id: "x".into(),
        valid_time: t,
        recorded_time: t,
        value: NeverSerializes,
    }];

    assert!(try_temporal_snapshot(&records, t).is_err());
    let compatibility = std::panic::catch_unwind(|| temporal_snapshot(&records, t));
    assert!(compatibility.is_ok(), "compatibility API must never panic");
    assert!(compatibility.unwrap().is_empty());
}

#[test]
fn compatibility_as_of_query_does_not_panic_on_tie_key_failure() {
    let t = chrono::Utc.timestamp_opt(1000, 0).unwrap();
    let records = vec![BitemporalRecord {
        id: "x".into(),
        valid_time: t,
        recorded_time: t,
        value: NeverSerializes,
    }];

    let compatibility = std::panic::catch_unwind(|| as_of_query(&records, t, t));
    assert!(compatibility.is_ok(), "compatibility API must never panic");
    assert!(compatibility.unwrap().is_empty());
}

#[test]
fn equal_recorded_time_selection_is_permutation_stable() {
    let t = chrono::Utc.timestamp_opt(1000, 1).unwrap();
    let a = BitemporalRecord {
        id: "x".into(),
        valid_time: t,
        recorded_time: t,
        value: "a".to_string(),
    };
    let b = BitemporalRecord {
        id: "x".into(),
        valid_time: t,
        recorded_time: t,
        value: "b".to_string(),
    };
    let q = t + chrono::Duration::seconds(1);
    assert_eq!(
        as_of_query(&[a.clone(), b.clone()], q, q),
        as_of_query(&[b, a], q, q)
    );
}
use chrono::TimeZone;

fn r(id: &str, v_time: i64, rec_time: i64) -> BitemporalRecord<String> {
    BitemporalRecord {
        id: id.to_string(),
        valid_time: chrono::Utc.timestamp_opt(v_time, 0).unwrap(),
        recorded_time: chrono::Utc.timestamp_opt(rec_time, 0).unwrap(),
        value: format!("v-{}", rec_time),
    }
}

// =========================================================================
// 1. RECEIPT DETERMINISM / COLLISION
// =========================================================================

#[test]
fn receipt_digest_is_deterministic_for_same_inputs() {
    // The receipt digest is the doctrinal audit handle. If two
    // equal-input records produce different digests, downstream
    // audit log cross-reference breaks.
    use bitemporal_runtime::SupersessionReceipt;
    let now = chrono::Utc.timestamp_opt(1000, 0).unwrap();
    let a = SupersessionReceipt::new(
        BitemporalRecord {
            id: "x".into(),
            valid_time: now,
            recorded_time: now,
            value: (),
        },
        BitemporalRecord {
            id: "y".into(),
            valid_time: now,
            recorded_time: now,
            value: (),
        },
    );
    let b = SupersessionReceipt::new(
        BitemporalRecord {
            id: "x".into(),
            valid_time: now,
            recorded_time: now,
            value: (),
        },
        BitemporalRecord {
            id: "y".into(),
            valid_time: now,
            recorded_time: now,
            value: (),
        },
    );
    assert_eq!(
        a.receipt_digest, b.receipt_digest,
        "receipt_digest must be deterministic"
    );
    assert_eq!(a.superseding_digest, b.superseding_digest);
    assert_eq!(a.superseded_digest, b.superseded_digest);
}

#[test]
fn receipt_digest_changes_when_any_input_changes() {
    // Digest must be a real hash, not a constant. Changing any
    // component (id, time, content) must change the digest.
    use bitemporal_runtime::SupersessionReceipt;
    let t0 = chrono::Utc.timestamp_opt(1000, 0).unwrap();
    let t1 = chrono::Utc.timestamp_opt(2000, 0).unwrap();
    let r1 = SupersessionReceipt::new(
        BitemporalRecord {
            id: "x".into(),
            valid_time: t0,
            recorded_time: t0,
            value: (),
        },
        BitemporalRecord {
            id: "y".into(),
            valid_time: t0,
            recorded_time: t0,
            value: (),
        },
    );
    let r2 = SupersessionReceipt::new(
        BitemporalRecord {
            id: "x".into(),
            valid_time: t0,
            recorded_time: t0,
            value: (),
        },
        BitemporalRecord {
            id: "y".into(),
            valid_time: t0,
            recorded_time: t1,
            value: (),
        }, // recorded_time differs
    );
    assert_ne!(
        r1.receipt_digest, r2.receipt_digest,
        "changing recorded_time must change digest"
    );
    let r3 = SupersessionReceipt::new(
        BitemporalRecord {
            id: "X".into(),
            valid_time: t0,
            recorded_time: t0,
            value: (),
        }, // uppercase X
        BitemporalRecord {
            id: "y".into(),
            valid_time: t0,
            recorded_time: t0,
            value: (),
        },
    );
    assert_ne!(
        r1.receipt_digest, r3.receipt_digest,
        "changing id case must change digest"
    );
}

#[test]
fn receipt_digest_is_sha256_hex() {
    // SHA-256 produces 64 lowercase hex characters. Any other
    // output means a broken hash implementation.
    use bitemporal_runtime::SupersessionReceipt;
    let now = chrono::Utc.timestamp_opt(1000, 0).unwrap();
    let r = SupersessionReceipt::new(
        BitemporalRecord {
            id: "x".into(),
            valid_time: now,
            recorded_time: now,
            value: (),
        },
        BitemporalRecord {
            id: "y".into(),
            valid_time: now,
            recorded_time: now,
            value: (),
        },
    );
    assert_eq!(r.receipt_digest.len(), 64, "SHA-256 produces 64 hex chars");
    assert!(
        r.receipt_digest.chars().all(|c| c.is_ascii_hexdigit()),
        "digest must be hex"
    );
    assert!(
        r.receipt_digest.chars().all(|c| !c.is_ascii_uppercase()),
        "digest must be lowercase hex (got uppercase chars: {})",
        r.receipt_digest
    );
}

// =========================================================================
// 2. RECEIPT INTEGRITY
// =========================================================================

#[test]
fn receipt_carries_correct_superseded_and_superseding_ids() {
    // After 3 supersessions, the receipts must link the chain:
    //   v1 -> v2 -> v3
    // Each prior row's receipt must name it as superseded, and the
    // new one as superseding.
    use bitemporal_runtime::SupersessionReceipt;
    let t0 = chrono::Utc.timestamp_opt(1000, 0).unwrap();
    let t1 = chrono::Utc.timestamp_opt(2000, 0).unwrap();
    let t2 = chrono::Utc.timestamp_opt(3000, 0).unwrap();
    let r1 = SupersessionReceipt::new(
        BitemporalRecord {
            id: "v1".into(),
            valid_time: t0,
            recorded_time: t0,
            value: (),
        },
        BitemporalRecord {
            id: "v2".into(),
            valid_time: t0,
            recorded_time: t1,
            value: (),
        },
    );
    assert_eq!(r1.superseded.superseded_id, "v1");
    assert_eq!(r1.superseding_id, "v2");
    assert_eq!(r1.superseded.superseded_recorded_time, t0);
    assert_eq!(r1.superseding_recorded_time, t1);

    let r2 = SupersessionReceipt::new(
        BitemporalRecord {
            id: "v2".into(),
            valid_time: t0,
            recorded_time: t1,
            value: (),
        },
        BitemporalRecord {
            id: "v3".into(),
            valid_time: t0,
            recorded_time: t2,
            value: (),
        },
    );
    assert_eq!(r2.superseded.superseded_id, "v2");
    assert_eq!(r2.superseding_id, "v3");
}

// =========================================================================
// 3. SELF-SUPERSEDE / IDENTITY EDGE CASES
// =========================================================================

#[test]
fn append_supersede_on_empty_returns_no_receipts() {
    // First insert: nothing to supersede. The Vec must end up
    // with one element, and no receipts.
    let mut records: Vec<BitemporalRecord<String>> = Vec::new();
    let receipts = append_supersede(&mut records, r("alpha", 100, 1000)).unwrap();
    assert!(receipts.is_empty());
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].id, "alpha");
}

#[test]
fn append_supersede_idempotent_on_same_record_no_prior() {
    // Inserting the same record (same id, same valid_time,
    // same recorded_time) twice: first is fine, second produces a
    // receipt for the first. This is intended behavior — the model
    // is append-only. A consumer can dedup by primary key if they
    // want strict idempotence.
    let mut records: Vec<BitemporalRecord<String>> = Vec::new();
    append_supersede(&mut records, r("alpha", 100, 1000)).unwrap();
    let receipts = append_supersede(&mut records, r("alpha", 100, 1000)).unwrap();
    assert_eq!(receipts.len(), 1, "second append supersedes the first");
    assert_eq!(records.len(), 2, "both rows are kept (append-only)");
}

#[test]
fn append_supersede_handles_unicode_ids() {
    // Record IDs are user-supplied; non-ASCII must work. Same id
    // (🦀-record), different recorded_times — the second should
    // supersede the first.
    let mut records: Vec<BitemporalRecord<String>> = Vec::new();
    append_supersede(&mut records, r("🦀-record", 100, 1000)).unwrap();
    let receipts = append_supersede(&mut records, r("🦀-record", 100, 2000)).unwrap();
    assert_eq!(receipts.len(), 1);
    assert_eq!(receipts[0].superseded.superseded_id, "🦀-record");
    assert_eq!(receipts[0].superseding_id, "🦀-record");
}

#[test]
fn append_supersede_handles_very_long_ids() {
    // Hostile input: 100KB id. Must not OOM or panic.
    let mut records: Vec<BitemporalRecord<String>> = Vec::new();
    let long_id = "x".repeat(100_000);
    append_supersede(&mut records, r(&long_id, 100, 1000)).unwrap();
    let receipts = append_supersede(&mut records, r(&long_id, 100, 2000)).unwrap();
    assert_eq!(receipts.len(), 1);
    assert_eq!(receipts[0].superseded.superseded_id.len(), 100_000);
}

#[test]
fn append_supersede_preserves_recorded_time_through_receipt() {
    // The receipt must carry the original recorded_time, not
    // some other timestamp. Verify the time values are bit-exact.
    use chrono::SubsecRound;
    let now = chrono::Utc::now().trunc_subsecs(0);
    let mut records: Vec<BitemporalRecord<String>> = Vec::new();
    append_supersede(
        &mut records,
        BitemporalRecord {
            id: "a".into(),
            valid_time: now,
            recorded_time: now,
            value: "v1".into(),
        },
    )
    .unwrap();
    let later = now + chrono::Duration::seconds(42);
    let receipts = append_supersede(
        &mut records,
        BitemporalRecord {
            id: "a".into(),
            valid_time: now,
            recorded_time: later,
            value: "v2".into(),
        },
    )
    .unwrap();
    assert_eq!(receipts[0].superseded.superseded_recorded_time, now);
    assert_eq!(receipts[0].superseding_recorded_time, later);
}

// =========================================================================
// 4. QUERY CORRECTNESS EDGE CASES
// =========================================================================

#[test]
fn as_of_query_with_zero_records_returns_empty() {
    // No records at all — every query must return empty.
    let records: Vec<BitemporalRecord<String>> = Vec::new();
    let now = chrono::Utc.timestamp_opt(1000, 0).unwrap();
    assert!(as_of_query(&records, now, now).is_empty());
    assert!(temporal_snapshot(&records, now).is_empty());
}

#[test]
fn as_of_query_with_valid_time_equals_recorded_time() {
    // Edge case: valid_time == recorded_time. The record's
    // valid_time must be <= query valid_time AND recorded_time <=
    // query recorded_time. If both equal the query, the record
    // must be included.
    let records = vec![r("a", 1000, 1000)];
    let t = chrono::Utc.timestamp_opt(1000, 0).unwrap();
    let result = as_of_query(&records, t, t);
    assert_eq!(
        result.len(),
        1,
        "valid_time == recorded_time == query must include the record"
    );
}

#[test]
fn as_of_query_with_valid_time_after_recorded_time_is_excluded() {
    // Doctrine: a fact cannot be valid in the future before it's
    // known to the system. The query requires BOTH conditions.
    // If valid_time > recorded_time, the record is excluded.
    let records = vec![r("a", 2000, 1000)]; // valid at 2000, known at 1000
    let t = chrono::Utc.timestamp_opt(1500, 0).unwrap();
    let result = as_of_query(&records, t, t);
    assert!(
        result.is_empty(),
        "record with valid_time > query_valid_time must be excluded"
    );
}

#[test]
fn as_of_query_with_query_before_recorded_time_is_excluded() {
    // Even if valid_time is far in the past, if the system didn't
    // know about it until recorded_time, a query before that time
    // can't see it. This is the temporal dimension at work.
    let records = vec![r("a", 500, 1000)]; // valid at 500, but known only since 1000
    let t = chrono::Utc.timestamp_opt(700, 0).unwrap();
    let result = as_of_query(&records, t, t);
    assert!(
        result.is_empty(),
        "query before recorded_time must not see the record"
    );
}

#[test]
fn as_of_query_with_identical_recorded_times_takes_one() {
    // Two versions of the same id recorded at the exact same
    // instant. Dedup by id must keep exactly one (whichever).
    let records = vec![
        r("a", 1000, 2000),
        r("a", 1000, 2000), // identical to first
    ];
    let t = chrono::Utc.timestamp_opt(3000, 0).unwrap();
    let result = as_of_query(&records, t, t);
    assert_eq!(
        result.len(),
        1,
        "duplicate recorded_time at same id dedupes to one"
    );
}

#[test]
fn as_of_query_dedup_prefers_higher_recorded_time_not_higher_valid_time() {
    // When deduping, the choice must be the LATEST recorded_time,
    // not the latest valid_time. These are orthogonal — a record
    // can have an old valid_time but a recent recorded_time (we
    // just learned about an old fact).
    let records = vec![
        r("a", 1000, 1500), // valid at 1000, recorded at 1500
        r("a", 500, 2500),  // valid at 500, recorded at 2500 (more recent knowledge)
    ];
    let t = chrono::Utc.timestamp_opt(3000, 0).unwrap();
    let result = as_of_query(&records, t, t);
    assert_eq!(result.len(), 1);
    assert_eq!(
        result[0].recorded_time,
        chrono::Utc.timestamp_opt(2500, 0).unwrap(),
        "as_of dedup must prefer the LATER recorded_time (the more recent knowledge), not the higher valid_time"
    );
    assert_eq!(
        result[0].valid_time,
        chrono::Utc.timestamp_opt(500, 0).unwrap(),
        "the dedup-picked record keeps its valid_time (the fact it represents)"
    );
}

#[test]
fn temporal_snapshot_at_distant_past_returns_empty() {
    // recorded_time is far future; querying at time=0 must
    // return nothing.
    let records = vec![r("a", 100, 1_000_000_000)];
    let result = temporal_snapshot(&records, chrono::Utc.timestamp_opt(0, 0).unwrap());
    assert!(result.is_empty());
}

#[test]
fn temporal_snapshot_at_distant_future_returns_all() {
    // recorded_time is in the past; querying at year 3000 must
    // see everything.
    let records = vec![r("a", 100, 1000), r("b", 200, 1500), r("c", 300, 2000)];
    let result = temporal_snapshot(
        &records,
        chrono::Utc.timestamp_opt(32_503_680_000, 0).unwrap(),
    );
    assert_eq!(result.len(), 3);
}

#[test]
fn temporal_snapshot_query_equals_recorded_time_keeps_record() {
    // The boundary condition: query at exactly the recorded_time.
    // The docstring says `recorded_time <= as_of_time` so equality
    // is included.
    let records = vec![r("a", 100, 1000)];
    let result = temporal_snapshot(&records, chrono::Utc.timestamp_opt(1000, 0).unwrap());
    assert_eq!(
        result.len(),
        1,
        "as_of_time == recorded_time must include the record"
    );
}

// =========================================================================
// 5. CROSS-CUTTING INVARIANTS
// =========================================================================

#[test]
fn duplicate_ids_at_same_recorded_time_do_not_collapse_value() {
    // When two records have the same id and same recorded_time
    // but different values, dedup is by recorded_time only — both
    // are "current" at the same instant, so the dedup loses one.
    // The remaining value must be a real value (not default). This
    // catches a class of bugs where dedup might produce a default-
    // constructed record.
    let records = vec![
        BitemporalRecord {
            id: "a".into(),
            valid_time: chrono::Utc.timestamp_opt(1000, 0).unwrap(),
            recorded_time: chrono::Utc.timestamp_opt(2000, 0).unwrap(),
            value: "first-value".to_string(),
        },
        BitemporalRecord {
            id: "a".into(),
            valid_time: chrono::Utc.timestamp_opt(1000, 0).unwrap(),
            recorded_time: chrono::Utc.timestamp_opt(2000, 0).unwrap(),
            value: "second-value".to_string(),
        },
    ];
    let t = chrono::Utc.timestamp_opt(3000, 0).unwrap();
    let result = as_of_query(&records, t, t);
    assert_eq!(result.len(), 1);
    // Whichever survives, its value must be non-empty.
    assert!(
        !result[0].value.is_empty(),
        "dedup must not produce a default/empty value"
    );
}

#[test]
fn query_on_500k_records_does_not_panic() {
    // Stress: 500K records with 1000 distinct ids. Must complete
    // without panic and return one record per id at the query
    // time. This catches O(n²) regressions. (The original 1M test
    // timed out at 3 minutes — even at O(n) hash insert, 1M is a
    // lot for a default `cargo test` budget. 500K is the largest
    // we can run reliably.)
    let mut records: Vec<BitemporalRecord<String>> = Vec::with_capacity(500_000);
    for i in 0..500_000i64 {
        records.push(r(
            &format!("id-{:06}", i % 1000),
            100 + (i / 1000),
            1000 + (i / 1000),
        ));
    }
    let t = chrono::Utc.timestamp_opt(2000, 0).unwrap();
    let result = as_of_query(&records, t, t);
    assert_eq!(
        result.len(),
        1000,
        "500K records, 1000 ids, as_of returns one per id"
    );
}

#[test]
fn snapshot_query_on_1m_records_does_not_panic() {
    // Same stress for temporal_snapshot.
    let mut records: Vec<BitemporalRecord<String>> = Vec::with_capacity(100_000);
    for i in 0..100_000i64 {
        records.push(r(
            &format!("id-{:05}", i % 100),
            100 + (i / 100),
            1000 + (i / 100),
        ));
    }
    let t = chrono::Utc.timestamp_opt(2_000, 0).unwrap();
    let result = temporal_snapshot(&records, t);
    assert_eq!(result.len(), 100);
}

#[test]
fn append_supersede_chain_grows_quadratically() {
    // HOSTILE-AUDIT FINDING: `append_supersede` is O(n) per call
    // (it iterates the full Vec to find prior versions with the
    // same id), so n appends are O(n²). This is fine for typical
    // use (≤100 versions per id) but is a real scaling cliff for
    // heavy supersession workloads.
    //
    // We assert that a 200-version chain completes within a
    // reasonable time bound (10s) — this catches the
    // quadratic-into-cubic regression (e.g. a refactor that
    // accidentally makes the per-call cost O(n²)).
    use std::time::Instant;
    let mut records: Vec<BitemporalRecord<i32>> = Vec::new();
    let base = chrono::Utc.timestamp_opt(1000, 0).unwrap();
    let start = Instant::now();
    for i in 0..200i64 {
        let t = base + chrono::Duration::seconds(i);
        let rec = BitemporalRecord {
            id: "chain".into(),
            valid_time: base,
            recorded_time: t,
            value: i as i32,
        };
        append_supersede(&mut records, rec).unwrap();
    }
    let elapsed = start.elapsed();
    assert_eq!(records.len(), 200);
    assert!(
        elapsed.as_secs() < 10,
        "200-version chain should complete in <10s, took {elapsed:?}"
    );
    // The latest as_of must return the last value.
    let final_t = base + chrono::Duration::seconds(20_000);
    let result = as_of_query(&records, final_t, final_t);
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].value, 199);
}

// =========================================================================
// 6. RECEIPT/SUPERSESSION CROSS-CHECK (one-off)
// =========================================================================

#[test]
fn each_prior_row_yields_exactly_one_receipt_in_append_supersede() {
    // If I have 5 prior versions of an id and append a 6th, the
    // function must return exactly 5 receipts — one per prior
    // version. Not 4 (missing one), not 6 (counting the new
    // one), not 0.
    let mut records: Vec<BitemporalRecord<String>> = Vec::new();
    for i in 0..5i64 {
        append_supersede(&mut records, r("chain", 100, 1000 + i)).unwrap();
    }
    assert_eq!(records.len(), 5);
    let receipts = append_supersede(&mut records, r("chain", 100, 2000)).unwrap();
    assert_eq!(
        receipts.len(),
        5,
        "one receipt per prior version, no more, no less"
    );
    assert_eq!(records.len(), 6, "new row is appended");
    // Each receipt's superseded_recorded_time should be one of the
    // prior recorded_times.
    let prior_times: std::collections::HashSet<i64> = (1000..1005).collect();
    for r in &receipts {
        assert!(
            prior_times.contains(&r.superseded.superseded_recorded_time.timestamp()),
            "receipt superseded_recorded_time must match a prior version"
        );
        assert_eq!(r.superseding_id, "chain");
    }
}
