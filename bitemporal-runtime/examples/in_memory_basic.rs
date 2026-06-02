//! Quick-start example: walk through the bitemporal model with a
//! small set of records, and print the as-of state at three different
//! points in time.
//!
//! Run with: `cargo run -p bitemporal-runtime --example in_memory_basic`

use bitemporal_runtime::{append_supersede, as_of_query, temporal_snapshot, BitemporalRecord};
use chrono::{TimeZone, Utc};

fn main() {
    // The fact: a sensor reading that we observed at T=1000 and
    // then revised at T=2000 (the revised value came in later).
    let mut records: Vec<BitemporalRecord<&str>> = Vec::new();

    // Recorded at T=1000: we believed the reading was 72°F.
    append_supersede(
        &mut records,
        BitemporalRecord {
            id: "sensor-42".to_string(),
            valid_time: Utc.timestamp_opt(900, 0).unwrap(),
            recorded_time: Utc.timestamp_opt(1000, 0).unwrap(),
            value: "72F",
        },
    )
    .expect("first insert succeeds");

    // Recorded at T=2000: we revised our belief to 68°F (the original
    // was a calibration error, discovered at T=2000). The original
    // 72°F record is NOT mutated — it's now superseded.
    let receipts = append_supersede(
        &mut records,
        BitemporalRecord {
            id: "sensor-42".to_string(),
            valid_time: Utc.timestamp_opt(900, 0).unwrap(),
            recorded_time: Utc.timestamp_opt(2000, 0).unwrap(),
            value: "68F",
        },
    )
    .expect("second insert succeeds");

    // One receipt per superseded prior version.
    assert_eq!(receipts.len(), 1);
    println!(
        "Supersession receipt: prior={} (recorded at {}) was superseded by {} (recorded at {})",
        receipts[0].superseded.superseded_id,
        receipts[0].superseded.superseded_recorded_time,
        receipts[0].superseding_id,
        receipts[0].superseding_recorded_time,
    );
    println!("  superseded_digest: {}", receipts[0].superseded_digest);
    println!("  superseding_digest: {}", receipts[0].superseding_digest);
    println!("  receipt_digest:     {}", receipts[0].receipt_digest);
    println!();

    // As-of queries: what did we believe at each moment in time?
    for as_of_t in [1000, 1500, 2000, 3000] {
        let t = Utc.timestamp_opt(as_of_t, 0).unwrap();
        let snap = temporal_snapshot(&records, t);
        let reading = snap
            .iter()
            .find(|r| r.id == "sensor-42")
            .map(|r| r.value)
            .unwrap_or("(unknown)");
        println!("As of T={as_of_t}: sensor-42 = {reading}");
    }
    println!();

    // As-of query at the valid_time boundary:
    // "what did we believe was true at V=900, as of T=1500?"
    let result = as_of_query(
        &records,
        Utc.timestamp_opt(900, 0).unwrap(),
        Utc.timestamp_opt(1500, 0).unwrap(),
    );
    let reading = result
        .iter()
        .find(|r| r.id == "sensor-42")
        .map(|r| r.value)
        .unwrap_or("(unknown)");
    println!("\"As of T=1500, what did we believe was true at V=900?\" → {reading}");
}
