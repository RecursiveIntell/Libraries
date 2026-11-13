# receipt-bench

Replayable benchmark substrate for Rust projects. Captures structured receipts
timestamped and keyed to commit hash + machine fingerprint, enabling diffs
between runs.

## Public types

- `BenchmarkSuite` — semantic search, compression round-trip, memory lookups
- `BenchmarkReceipt` — provenance receipt (timestamp, commit hash, machine fingerprint)
- `ReceiptDiff` — comparison utility between benchmark receipts

## Usage

```rust
use receipt_bench::{BenchmarkSuite, BenchmarkReceipt, MachineFingerprint};

// Register benchmark callbacks with suite.register(name, callback) before running.
let suite = BenchmarkSuite::new();
let receipt = suite.run()?;
println!("{:?}", receipt);
```

A new suite has no registered benchmarks; the example returns an empty receipt. The detected commit can be `unknown` outside a Git checkout. A receipt records the configured measurements and does not independently certify their protocol.

## Design Goals

- Local-first: no network, no external services
- Minimal dependencies: serde, thiserror, chrono, sha2
- MSRV 1.75, Rust 2021 edition
- No `unwrap()` in production code