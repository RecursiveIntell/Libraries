# Phase 07 Report - Synthetic Tests and Benchmark Harness

Status: passed.

Added:

- `crates/poly-kv/tests/synthetic_roundtrip.rs`
- `crates/poly-kv/tests/memory_accounting.rs`
- `crates/poly-kv/tests/shape_rejection.rs`
- `crates/poly-kv/tests/receipt_roundtrip.rs`
- `crates/poly-kv/tests/deterministic_replay.rs`
- `crates/poly-kv/benches/synthetic_pool.rs`

Commands run:

- `cargo test --workspace --all-targets`
- `cargo test -p poly-kv synthetic -- --nocapture`
- `cargo test -p poly-kv memory_accounting`

Guardrail result:

- Synthetic MHA/MQA/GQA fixtures are covered.
- Deterministic replay and receipt serde roundtrips are covered.
- Benchmark harness is feature-gated by `bench`.

Blockers: none.
