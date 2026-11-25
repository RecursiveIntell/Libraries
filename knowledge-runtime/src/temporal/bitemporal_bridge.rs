//! As-of temporal bridge over `bitemporal-runtime`.
//!
//! Scope: process-lifetime store injection only. Durable persistence is
//! evidence debt; callers must inject a store.
//!
//! Signatures mirror `bitemporal-runtime/src/queries.rs`:
//! - `as_of_query` -> `Vec<BitemporalRecord<T>>`
//! - `append_supersede` -> `Result<Vec<SupersessionReceipt>, BitemporalError>`
//! - `temporal_snapshot(records, as_of_time)` -> `Vec<BitemporalRecord<T>>`

use bitemporal_runtime::{
    append_supersede, as_of_query, temporal_snapshot, BitemporalError, BitemporalRecord,
    SupersessionReceipt,
};
use chrono::{DateTime, Utc};

/// Records visible as of `valid_time` and `recorded_time`.
pub fn as_of(
    records: &[BitemporalRecord<String>],
    valid_time: DateTime<Utc>,
    recorded_time: DateTime<Utc>,
) -> Vec<BitemporalRecord<String>> {
    as_of_query(records, valid_time, recorded_time)
}

/// Append a supersession; returns receipts for each superseded prior version.
pub fn supersede(
    records: &mut Vec<BitemporalRecord<String>>,
    new_record: BitemporalRecord<String>,
) -> Result<Vec<SupersessionReceipt>, BitemporalError> {
    append_supersede(records, new_record)
}

/// Full snapshot at a recorded time.
pub fn snapshot(
    records: &[BitemporalRecord<String>],
    recorded_time: DateTime<Utc>,
) -> Vec<BitemporalRecord<String>> {
    temporal_snapshot(records, recorded_time)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn rec(id: &str, valid: i64, recorded: i64, value: &str) -> BitemporalRecord<String> {
        BitemporalRecord {
            id: id.to_string(),
            valid_time: Utc.timestamp_opt(valid, 0).unwrap(),
            recorded_time: Utc.timestamp_opt(recorded, 0).unwrap(),
            value: value.to_string(),
        }
    }

    #[test]
    fn as_of_returns_version_valid_at_that_time() {
        let r1 = rec("x", 100, 100, "old");
        let r2 = rec("x", 200, 200, "new");
        let records = vec![r1, r2];
        let at_150 = as_of(
            &records,
            Utc.timestamp_opt(150, 0).unwrap(),
            Utc.timestamp_opt(250, 0).unwrap(),
        );
        assert_eq!(at_150.len(), 1);
        assert_eq!(at_150[0].value, "old");
        let at_250 = as_of(
            &records,
            Utc.timestamp_opt(250, 0).unwrap(),
            Utc.timestamp_opt(250, 0).unwrap(),
        );
        assert_eq!(at_250.len(), 1);
        assert_eq!(at_250[0].value, "new");
    }

    #[test]
    fn supersede_emits_receipt() {
        let mut records = vec![rec("y", 100, 100, "v1")];
        let receipts = supersede(&mut records, rec("y", 200, 200, "v2")).unwrap();
        assert_eq!(receipts.len(), 1);
    }

    #[test]
    fn snapshot_reflects_latest_recorded() {
        let records = vec![rec("z", 100, 100, "a"), rec("z", 150, 150, "b")];
        let snap = snapshot(&records, Utc.timestamp_opt(160, 0).unwrap());
        assert_eq!(snap.len(), 1);
        assert_eq!(snap[0].value, "b");
    }
}
