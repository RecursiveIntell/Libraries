use bitemporal_runtime::{append_supersede, BitemporalRecord, SupersessionReceipt};
use chrono::{TimeZone, Utc};
use serde::Serialize;

fn record<T>(value: T, nanos: u32) -> BitemporalRecord<T> {
    BitemporalRecord {
        id: "same".to_owned(),
        valid_time: Utc.timestamp_opt(1_000, nanos).unwrap(),
        recorded_time: Utc.timestamp_opt(2_000, nanos).unwrap(),
        value,
    }
}

#[test]
fn changing_only_value_changes_receipt_digest() {
    let first = SupersessionReceipt::new(record("old", 1), record("new-a", 2)).unwrap();
    let second = SupersessionReceipt::new(record("old", 1), record("new-b", 2)).unwrap();
    assert_ne!(first.receipt_digest, second.receipt_digest);
}

#[test]
fn changing_only_subsecond_timestamp_changes_receipt_digest() {
    let first = SupersessionReceipt::new(record("old", 1), record("new", 2)).unwrap();
    let second = SupersessionReceipt::new(record("old", 1), record("new", 3)).unwrap();
    assert_ne!(first.receipt_digest, second.receipt_digest);
}

#[derive(Clone)]
struct FailsSerialization;

impl Serialize for FailsSerialization {
    fn serialize<S>(&self, _serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        Err(serde::ser::Error::custom("intentional failure"))
    }
}

#[test]
fn serialization_failure_aborts_receipt_and_append() {
    assert!(
        SupersessionReceipt::new(record(FailsSerialization, 1), record(FailsSerialization, 2))
            .is_err()
    );

    let mut records = vec![record(FailsSerialization, 1)];
    assert!(append_supersede(&mut records, record(FailsSerialization, 2)).is_err());
    assert_eq!(records.len(), 1);
}
