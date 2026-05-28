# P16 Handoff - Lawful Subtraction, Compaction, and Invariant-Preserving Reduction

## Summary

P16 is implemented. AiDENs now has typed lawful-subtraction contracts, support-core extraction, dry-run removal frontiers, invariant budgets, append-only compaction receipts, history-preservation reports, and durable receipt support. Reduction is blocked when it would remove accepted-claim support unless the claim has first been superseded or quarantined. Memory compaction records evidence without destructive deletion and preserves as-of query behavior under a full-history budget.

## Files Changed

- `crates/aidens-contracts/src/lib.rs`
  - Added `SubtractionPlanV1`, `SupportCoreV1`, `RemovalFrontierV1`, `InvariantBudgetV1`, `CompactionReceiptV1`, `HistoryPreservationReportV1`, supporting enums, receipt-kind registration, schema registration, constructors, policy helpers, and P16 fixture tests.
- `crates/aidens-kernel-kit/src/lib.rs`
  - Added support-core extraction and dry-run claim subtraction planning. Accepted support is blocked unless superseded/quarantined; claim planning does not silently target support evidence.
- `crates/aidens-memory-kit/src/lib.rs`
  - Added append-only history compaction, visible-history digests, compaction transactions, durable P16 receipt append wiring, and an as-of preservation regression test.
- `crates/aidens-repair-kit/src/lib.rs`
  - Added reduction readiness evidence for support claims after repair supersession.
- `crates/aidens-receipts/src/lib.rs`
  - Added durable append helpers for P16 support, frontier, budget, plan, history report, and compaction receipts.
- `crates/aidens/src/lib.rs`
  - Exported P16 artifacts and kernel helpers through the prelude.
- `tests/fixtures/p16/*.json`
  - Added golden fixtures for all required P16 artifacts.
- `schemas/compaction-receipt/v1.schema.json`
- `schemas/history-preservation-report/v1.schema.json`
- `schemas/invariant-budget/v1.schema.json`
- `schemas/removal-frontier/v1.schema.json`
- `schemas/subtraction-plan/v1.schema.json`
- `schemas/support-core/v1.schema.json`
- `schemas/generated-schema-manifest/v1.schema.json`
- `schemas/generated_schema_manifest_v1.json`
  - Regenerated schema outputs through `aidens schemas generate` for 99 registered schema files.
- `README.md`
- `STATUS.md`
- `ARTIFACT_SCHEMA_REGISTRY.md`
- `ACCEPTANCE_GATES_AND_CI.md`
  - Updated pass status, schema registry, and gate notes without promoting P17/P18/P19 surfaces.

## Tests Added

- Contract tests for P16 constructor policy, support-loss blocking, append-only compaction receipt construction, and golden fixture deserialization.
- Kernel test proving dry-run subtraction blocks accepted support until the support claim is superseded.
- Memory test proving append-only compaction emits receipts/reports and preserves as-of query answers under a full-history budget.
- Repair test proving supersession unlocks reduction readiness for a previously supported claim.
- Receipts test proving durable stores append all P16 receipt families.

## Commands Run

```bash
cargo check -p aidens-contracts -p aidens-kernel-kit -p aidens-memory-kit -p aidens-repair-kit -p aidens-receipts -p aidens
cargo test -p aidens-contracts p16
cargo test -p aidens-kernel-kit subtraction
cargo test -p aidens-memory-kit compaction
cargo test -p aidens-repair-kit reducible
cargo test -p aidens-receipts p16
cargo run -p aidens-cli -- schemas generate
cargo run -p aidens-cli -- schemas check
cargo fmt --all --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings
bash scripts/verify.sh
bash scripts/assert_no_fake_completion.sh .
bash scripts/assert_no_scaffold_promoted.sh .
rg -n "20260425|2026-04-25|2026/04/25" .
```

All build/test/schema/fake-ready/scaffold gates passed. The stale 20260425 scan returned only explicit historical references, P00 acceptance text, and command records.

## Blockers

None for P16.

## Next-Pass Readiness

P17 is unblocked from the P16 substrate perspective. P17 federation/admission work remains untouched and should only start when explicitly requested.
