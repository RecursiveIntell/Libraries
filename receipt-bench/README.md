# receipt-bench

Replayable benchmark substrate for Rust projects. Captures structured receipts
timestamped and keyed to commit hash + machine fingerprint, enabling diffs
between runs.

## Crates

- `BenchmarkSuite` — semantic search, compression round-trip, memory lookups
- `BenchmarkReceipt` — provenance receipt (timestamp, commit hash, machine fingerprint)
- `ReceiptDiff` — comparison utility between benchmark receipts

## Usage

```rust
use receipt_bench::{BenchmarkSuite, BenchmarkReceipt, MachineFingerprint};

let suite = BenchmarkSuite::new();
let receipt = suite.run()?;
println!("{:?}", receipt);
```

## Design Goals

- Local-first: no network, no external services
- Minimal dependencies: serde, thiserror, chrono, sha2
- MSRV 1.75, Rust 2021 edition
- No `unwrap()` in production code