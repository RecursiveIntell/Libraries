# P18 Handoff - Mechanism Theory Search and Experiment Runtime

## Summary

P18 is implemented only. Candidate mechanisms, theory versions, hypothesis libraries, simulator contracts, fit runs, invariance reports, and refuter suites are now typed artifacts with generated schemas, golden fixtures, durable receipt support, memory recording, kernel replay helpers, and governance gates that prevent score-only promotion.

## Files Changed

- `crates/aidens-contracts/src/lib.rs`
- `crates/aidens-kernel-kit/src/lib.rs`
- `crates/aidens-governance-kit/src/lib.rs`
- `crates/aidens-memory-kit/src/lib.rs`
- `crates/aidens-receipts/src/lib.rs`
- `tests/fixtures/p18/*.json`
- `schemas/` regenerated, including P18 schema directories
- `README.md`
- `STATUS.md`
- `ARTIFACT_SCHEMA_REGISTRY.md`
- `handoffs/P18_MECHANISM_THEORY_SEARCH_AND_EXPERIMENT_RUNTIME.md`

## Tests Added

- Contract tests for P18 constructors, golden fixture deserialization, fit/refuter linkage, replay handles, and explicit alias law.
- Kernel tests proving a candidate mechanism can be fit, refuted, versioned, superseded, and replayed from artifacts.
- Kernel test proving observationally equivalent mechanisms remain distinct until an explicit alias decision is admitted.
- Governance tests proving high fit score alone cannot promote a theory, refuter/governance artifacts can promote it, and aliasing requires an explicit equivalence decision.
- Memory test proving P18 artifacts are append-only and replayable after store reopen.
- Receipt-store test proving P18 artifacts append as durable receipt envelopes and outbox rows.

## Commands Run

```bash
cargo check -p aidens-contracts
cargo test -p aidens-kernel-kit
cargo test -p aidens-governance-kit
cargo test -p aidens-memory-kit
cargo test -p aidens-contracts -p aidens-receipts
cargo run -q -p aidens-cli -- schemas generate
cargo run -q -p aidens-cli -- schemas check
cargo fmt --all
cargo fmt --all --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings
bash scripts/verify.sh
```

An attempted parallel `schemas generate` plus `schemas check` raced on partially written schema files and produced transient `schema-json-invalid` output. The sequential `cargo run -q -p aidens-cli -- schemas check` rerun passed with 113 compatible schemas.

## Blockers

None for P18 after the final gate rerun.

## Next-Pass Readiness

P19 is unblocked from the P18 substrate perspective. Final integration should audit the full P00-P18 surface without starting new mechanism-theory features, and should preserve the P18 laws: fit score is not verification, refuters and governance are required for promotion, and equivalent mechanisms remain distinct unless an explicit alias decision is admitted.
