# Phase 14 Report - Marker Test Replacement

## Scope

- Phase: `Phase 14 conformance/test replacement`
- Backlog rows: `AHD-0841` through `AHD-0900`
- Rows touched: 60
- Final row status: 60 `fixed`, 0 raw `open`

## Changes

- Added `scripts/assert_p29_no_marker_only_hard_gates.py`, including a hostile self-test that proves a fake marker-only verifier script is rejected.
- Wired the marker-only hard-gate scanner into `scripts/p29_verify.sh`.
- Replaced marker-string assertions with behavioral wrappers:
  - `assert_p29_v11a_contracts.py` now runs material-operation contract tests.
  - `assert_p29_receipt_chain.py` now runs receipt-chain durability/tamper/quarantine tests.
  - `assert_p29_proof_debt.py` now runs proof debt, waiver-not-proof, and proof-satisfied tests.
  - `assert_p29_boundary_profiles.py` now runs boundary parser/repair hostile tests.
  - `assert_p29_v11b_seed_surfaces.py` now runs the minimal v11B seed tests and still rejects completion overclaims.
- Upgraded `assert_p29_audit_matrix_closure.py` so matrix closure checks row statuses/resolutions, not only ID coverage. It supports completed-phase scope for phase gates and final all-phase closure for release.

## Files Changed

- `scripts/assert_p29_audit_matrix_closure.py`
- `scripts/assert_p29_boundary_profiles.py`
- `scripts/assert_p29_no_marker_only_hard_gates.py`
- `scripts/assert_p29_proof_debt.py`
- `scripts/assert_p29_receipt_chain.py`
- `scripts/assert_p29_v11a_contracts.py`
- `scripts/assert_p29_v11b_seed_surfaces.py`
- `scripts/p29_verify.sh`
- `matrices/SUPER_PASS_BACKLOG_1020.csv`
- `matrices/SUPER_PASS_BACKLOG_1020.json`

## Tests Run

- `python3 scripts/assert_p29_no_marker_only_hard_gates.py --self-test`
  - Log: `target/super-pass/audit/phase14-no-marker-only-self-test.log`
- `python3 scripts/assert_p29_no_marker_only_hard_gates.py`
  - Log: `target/super-pass/audit/phase14-no-marker-only-hard-gates.log`
- `python3 scripts/assert_p29_v11a_contracts.py`
  - Log: `target/super-pass/audit/phase14-v11a-contracts-behavioral.log`
- `python3 scripts/assert_p29_receipt_chain.py`
  - Log: `target/super-pass/audit/phase14-receipt-chain-behavioral.log`
- `python3 scripts/assert_p29_proof_debt.py`
  - Log: `target/super-pass/audit/phase14-proof-debt-behavioral.log`
- `python3 scripts/assert_p29_boundary_profiles.py`
  - Log: `target/super-pass/audit/phase14-boundary-profiles-behavioral.log`
- `python3 scripts/assert_p29_v11b_seed_surfaces.py`
  - Log: `target/super-pass/audit/phase14-v11b-seed-behavioral.log`
- `python3 scripts/assert_p29_audit_matrix_closure.py --completed-through 14`
  - Log: `target/super-pass/audit/phase14-audit-matrix-closure-through-14.log`
- `cargo fmt --all --check`
  - Log: `target/super-pass/audit/phase14-cargo-fmt-check.log`
- `cargo check --workspace --all-targets`
  - Log: `target/super-pass/audit/phase14-cargo-check-workspace-all-targets.log`
- `cargo clippy --workspace --all-targets -- -D warnings`
  - Log: `target/super-pass/audit/phase14-cargo-clippy-workspace-all-targets.log`
- `cargo test --workspace --all-targets`
  - Log: `target/super-pass/audit/phase14-cargo-test-workspace-all-targets.log`

## Rows Closed

- Fixed: `AHD-0841` through `AHD-0900`
- Quarantined: none
- Deferred: none
- Open-blocking: none

## Unresolved Risk

- Some source-text policy scanners remain appropriate for policy/document claims, packaging paths, and forbidden-label detection. Phase 14 specifically blocks marker-only substitutions for hard gates in `p29_verify.sh`.
- Final matrix closure is intentionally not green until later phases close or classify the remaining rows.

## Exit Decision

Continue. Phase 14 exit gate passed: marker-only hard gates are rejected, behavioral gate wrappers pass, closure through Phase 14 passes, and the broad workspace command bar is green.
