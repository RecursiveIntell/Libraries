# P13 Handoff - Multi-view runtime disclosure and query policy

## Scope

Implemented P13 only. Later passes were not started.

## Files Changed

- `crates/aidens-contracts/src/lib.rs`
- `crates/aidens-memory-kit/src/lib.rs`
- `crates/aidens-runner/src/lib.rs`
- `crates/aidens-governance-kit/src/lib.rs`
- `crates/aidens-cli/Cargo.toml`
- `crates/aidens-cli/src/lib.rs`
- `tests/fixtures/p13/*.json`
- `schemas/degradation-event/v1.schema.json`
- `schemas/projection-digest/v1.schema.json`
- `schemas/query-widening-receipt/v1.schema.json`
- `schemas/retrieval-policy/v1.schema.json`
- `schemas/runtime-view-request/v1.schema.json`
- `schemas/view-disclosure-receipt/v1.schema.json`
- `schemas/generated_schema_manifest_v1.json`
- `README.md`
- `STATUS.md`
- `ARTIFACT_SCHEMA_REGISTRY.md`
- `handoffs/P13_MULTI_VIEW_RUNTIME_DISCLOSURE_AND_QUERY_POLICY.md`

## Implementation Notes

- Added P13 wire-visible contracts: `RuntimeViewRequestV1`, `RetrievalPolicyV1`, `QueryWideningReceiptV1`, `DegradationEventV1`, `ProjectionDigestV1`, and `ViewDisclosureReceiptV1`.
- Added generated schema registration and P13 golden fixtures.
- Replaced local memory-store assumptions with canonical memory/runtime adapter calls; runtime views are derived from canonical memory and recorded only as display/disclosure artifacts.
- Time-scoped queries emit degradation evidence when no matching time-scoped result exists; optional timeless fallback is explicit and receipt-bearing.
- Alias expansion is policy-gated and emits `QueryWideningReceiptV1`.
- Projection digests rebuild deterministically from the same policy and memory/evidence set.
- Added `aidens view query`, with view mode and degradation reasons serialized before claim results.
- Added runner/governance helpers for view disclosure and retrieval policy checks without merging execution receipts into domain truth.

## Tests Added

- Contract constructor and fixture tests for P13 artifacts.
- Memory tests for no silent timeless fallback, alias widening receipt emission, durable view receipt append, and stable projection rebuild digest.
- Governance tests for required alias widening receipts and explicit timeless fallback degradation.
- Runner test proving disclosure is receipt-only and separated from domain truth.
- CLI test proving `view query` prints disclosure fields before claims, emits alias widening receipts, records degradation reasons, and rejects aliases without expansion policy.

## Commands Run

```bash
cargo test -p aidens-memory-kit
cargo test -p aidens-governance-kit -p aidens-runner
cargo test -p aidens-cli
cargo test -p aidens-contracts
cargo run -p aidens-cli -- schemas generate
cargo run -p aidens-cli -- schemas check
cargo fmt --all
cargo check --workspace
cargo test --workspace
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
bash scripts/verify.sh
bash scripts/assert_no_fake_completion.sh .
bash scripts/assert_no_scaffold_promoted.sh .
rg -n "20260425|2026-04-25|2026/04/25" .
```

All build/test/schema/fake-ready/scaffold gates passed. The stale 20260425 scan contains only P00 acceptance text and explicit historical references.

## Blockers

- None for P13.

## Next-pass Readiness

P13 is complete. P14 can start next: release-grade product surface, operator UX, and status truth.
