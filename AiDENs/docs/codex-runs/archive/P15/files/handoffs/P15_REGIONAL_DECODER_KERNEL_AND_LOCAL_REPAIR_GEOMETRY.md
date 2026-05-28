# P15 Handoff - Regional Decoder Kernel and Local Repair Geometry

## Summary

P15 is implemented. AiDENs now has bounded region-graph kernel contracts, a first executable kernel kit, local contradiction syndrome routing, convergence evidence, oracle-slice agreement checks, durable P15 receipt append support, schema generation, and status/docs updates that keep advanced post-P15 work deferred.

## Files Changed

- `crates/aidens-contracts/src/lib.rs`
  - Added `CompiledRegionGraphV1`, `RegionContractV1`, `SyndromeV1`, `ResidualV1`, `OracleSliceRequestV1`, `KernelRunDisplayReportV1`, `ConvergenceReportV1`, supporting region graph/kernel enums, schema registration, constructors, and P15 fixture tests.
- `crates/aidens-kernel-kit/Cargo.toml`
- `crates/aidens-kernel-kit/src/lib.rs`
  - Replaced scaffold behavior with bounded graph compilation, right-graph-law checks, budgeted message propagation, contradiction-to-syndrome emission, oracle slice comparison, and P15 kernel tests.
- `crates/aidens-memory-kit/src/lib.rs`
  - Added `KernelClaimSliceV1` and `claim_slice_for_kernel` so kernel inputs come from runtime-view projection evidence instead of a shadow truth store.
- `crates/aidens-repair-kit/src/lib.rs`
  - Added `route_syndrome_to_local_repair` and error handling that rejects global recompute requirements for local repair routing.
- `crates/aidens-receipts/src/lib.rs`
  - Added durable append helpers for P15 residual, syndrome, oracle-slice, convergence, and kernel-run receipts.
- `crates/aidens-cli/src/lib.rs`
- `scripts/assert_no_scaffold_promoted.sh`
  - Removed `aidens-kernel-kit` from the scaffold-only surface list and kept remaining scaffold-only crates blocked/deferred.
- `crates/aidens/Cargo.toml`
- `crates/aidens/src/lib.rs`
  - Added `aidens-kernel-kit` and exported P15 artifacts/helpers through the prelude.
- `tests/fixtures/p15/*.json`
  - Added golden fixtures for all required P15 artifacts.
- `schemas/compiled-region-graph/v1.schema.json`
- `schemas/convergence-report/v1.schema.json`
- `schemas/kernel-run-receipt/v1.schema.json`
- `schemas/oracle-slice-request/v1.schema.json`
- `schemas/region-contract/v1.schema.json`
- `schemas/residual/v1.schema.json`
- `schemas/syndrome/v1.schema.json`
- `schemas/generated-schema-manifest/v1.schema.json`
- `schemas/generated_schema_manifest_v1.json`
  - Regenerated schema outputs through `aidens schemas generate`.
- `README.md`
- `STATUS.md`
- `ARTIFACT_SCHEMA_REGISTRY.md`
- `ACCEPTANCE_GATES_AND_CI.md`
  - Updated pass status, scaffold count, schema registry, and P15 gate notes without promoting later advanced kernel work.

## Tests Added

- Contract tests for P15 artifact constructors, right-graph-law preservation, explicit convergence stop-rule evidence, and golden fixture deserialization.
- Kernel-kit tests for rejecting storage graphs as runtime graphs, synthetic contradiction syndrome emission with a local repair candidate, loopy non-convergence degradation, and oracle-slice agreement/bounded-disagreement behavior.
- Memory-kit test proving kernel claim slices use runtime-view projection evidence.
- Repair-kit test proving syndrome routing stays local and rejects global recompute.
- Receipts test proving durable stores append P15 kernel evidence.
- CLI/status tests adjusted so scaffold-only promotion checks now target the remaining deferred scaffold crates, not `aidens-kernel-kit`.

## Commands Run

```bash
cargo check -p aidens-contracts -p aidens-kernel-kit -p aidens-memory-kit -p aidens-repair-kit -p aidens-receipts -p aidens-cli -p aidens
cargo test -p aidens-contracts p15
cargo test -p aidens-kernel-kit
cargo test -p aidens-memory-kit kernel_claim_slice
cargo test -p aidens-repair-kit syndrome_routes
cargo test -p aidens-receipts p15
cargo test -p aidens-cli
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

None for P15.

## Next-Pass Readiness

P16 is unblocked from the P15 kernel substrate perspective. The remaining advanced subtraction/federation/mechanism surfaces are still deferred and must not be treated as implemented until their own passes are explicitly requested.
