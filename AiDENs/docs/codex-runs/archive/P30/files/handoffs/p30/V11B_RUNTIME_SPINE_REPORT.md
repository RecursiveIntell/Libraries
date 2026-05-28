# P30 v11B Runtime Spine Report

P30 preserves the existing v11B executable seed and revalidates it through `bash scripts/verify.sh`.

Evidence:

- `target/p30/audit/scripts_verify.log`: `v11B seed behavioral checks passed without completion claim`.
- `crates/aidens-integration-tests/tests/phase_10_minimal_v11b_region.rs`: wrong-graph rejection, boundary outcomes, failure slices, and degraded/non-promotable region behavior.
- `crates/aidens-contracts/src/reserved_v11.rs`: seed contracts for graph surfaces, regions, residuals/syndromes, convergence reports, kernel run reports, support cores, removal frontiers, subtraction plans, history preservation, and compaction reports.

Claim boundary: this is a v11B executable seed/runtime-spine check, not v11B conformance. Full production path use and release-grade reference coverage remain in `V11B_CONFORMANCE_DEBT_LEDGER.md`.
