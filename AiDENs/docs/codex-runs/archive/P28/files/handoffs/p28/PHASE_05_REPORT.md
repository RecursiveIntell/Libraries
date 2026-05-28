# P28 Phase 05 Report

## Scope

Added v11A boundary compiler profile and receipt facades, with regression coverage for duplicate-key rejection and treatment-integrity hard failure.

## Files changed

- `crates/aidens-contracts/src/boundary.rs`
- `crates/aidens-contracts/src/lib.rs`
- `crates/aidens-boundary-kit/src/lib.rs`
- `handoffs/p28/PHASE_05_REPORT.md`

## Claims made

- Claim: `BoundaryCompilerProfileV1`, `BoundaryCompileReceiptV1`, `BoundaryRepairReceiptV1`, and `TreatmentIntegrityReceiptV1` exist.
  - status: pass
  - evidence: `crates/aidens-contracts/src/boundary.rs`
- Claim: duplicate JSON keys remain hard rejected and can be represented as a v11A boundary compile receipt.
  - status: pass
  - evidence: `target/p28/audit/cargo_test_aidens_boundary_kit_p28_boundary_phase05.log`
- Claim: parser repair cannot silently change treatment-critical fields.
  - status: pass
  - evidence: `target/p28/audit/cargo_test_aidens_boundary_kit_p28_treatment_phase05.log`

## Evidence

- `target/p28/audit/cargo_fmt_phase05.log`
- `target/p28/audit/cargo_check_phase05.log`
- `target/p28/audit/cargo_test_aidens_boundary_kit_p28_boundary_phase05.log`
- `target/p28/audit/cargo_test_aidens_boundary_kit_p28_treatment_phase05.log`

## Tests run

```bash
cargo fmt --all -- --check
cargo check --workspace --all-targets
cargo test -p aidens-boundary-kit p28_boundary
cargo test -p aidens-boundary-kit p28_treatment
```

## Failures / degraded checks

- None in Phase 05 validation.

## Open risks

- Existing boundary CLI outputs still use older P27 display structures. They now have v11A receipt facades available, but full CLI/report replacement is later wiring work.

## Next phase readiness

Ready.
