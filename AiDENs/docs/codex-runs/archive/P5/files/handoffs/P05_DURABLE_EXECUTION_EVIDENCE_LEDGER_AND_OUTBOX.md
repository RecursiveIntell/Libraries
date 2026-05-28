# P05 Durable Execution Evidence Ledger And Outbox Handoff

## Scope

Implemented P05 only. No P06 boundary compiler/schema-generation work, P09 memory store work, P10 coding tool expansion, or P11 daemon/queue/scheduler behavior was started.

## Files Changed

- `crates/aidens-contracts/src/lib.rs`
- `crates/aidens-receipts/Cargo.toml`
- `crates/aidens-receipts/src/lib.rs`
- `crates/aidens-runner/src/lib.rs`
- `crates/aidens-config/src/lib.rs`
- `crates/aidens-app-kit/src/lib.rs`
- `crates/aidens-app-kit/tests/next_app_plan_facade.rs`
- `crates/aidens-cli/Cargo.toml`
- `crates/aidens-cli/src/lib.rs`
- `Cargo.lock`
- `tests/fixtures/p05/*.json`
- `schemas/poison_receipt_record_v1.sketch.json`
- `schemas/execution_lineage_graph_v1.sketch.json`
- `ARTIFACT_SCHEMA_REGISTRY.md`
- `README.md`
- `STATUS.md`
- `handoffs/P05_DURABLE_EXECUTION_EVIDENCE_LEDGER_AND_OUTBOX.md`

## Tests Added

- Contract constructor and golden-fixture tests for `PoisonReceiptRecordV1` and `ExecutionLineageGraphV1`; canonical library receipt payloads cover runtime receipt semantics.
- Receipt-store tests for restart inspection, stable pretty/compact JSON digests, poison records, and approval/permit receipt expansion.
- Runner tests proving durable restart inspection, provider-unavailable receipt emission, tool/boundary failure receipt emission, and explicit minimal/no-store direct-runner behavior.
- CLI test covering `receipts list`, `receipts inspect`, `receipts export`, and `receipts verify-digest` after a run and store reopen.

## Commands Run

- `cargo check -p aidens-contracts -p aidens-receipts`
- `cargo check -p aidens-runner -p aidens-app-kit -p aidens-cli -p aidens-config -p aidens-receipts`
- `cargo test -p aidens-contracts -p aidens-receipts -p aidens-runner -p aidens-app-kit -p aidens-cli`
- `cargo fmt --all`
- `cargo fmt --all --check`
- `cargo check --workspace`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `bash scripts/verify.sh`

All gate commands passed.

## Blockers

None for P05 acceptance.

Deferred by build order:

- Generated schemas and migration law remain P07.
- Compiler-grade boundary validation remains P06; P05 only links existing boundary repair receipts into durable run evidence.
- Durable memory remains P09.
- Broader coding tool execution remains P10.
- Daemon, queue, schedule, leases, and outbox consumers remain P11. `aidens-daemon-kit` stays scaffold-only.

## Next-Pass Readiness

P06 can start from durable receipt envelopes, stable JSON digests, receipt-linked boundary repair evidence, and CLI/store inspection paths. P05 leaves all later surfaces explicitly deferred rather than promoted.
