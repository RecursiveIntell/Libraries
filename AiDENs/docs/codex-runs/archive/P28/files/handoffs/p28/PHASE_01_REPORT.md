# P28 Phase 01 Report

## Scope

Closed the declared P0 bugfix lane before v11A scaffolding. The work focused on identity determinism, fail-closed packaging paths, result-bearing profile expansion, patch/symlink safety, waiver/proof semantics, degradation honesty, and aggregate manifest truth.

## Files changed

- `crates/aidens-contracts/src/lib.rs`
- `crates/aidens-contracts/src/tests.rs`
- `crates/aidens-app-kit/src/lib.rs`
- `crates/aidens-cli/src/lib.rs`
- `crates/aidens-tool-kit/src/lib.rs`
- `z.py`
- `scripts/assert_package_validation.py`
- `scripts/assert_package_self_replay.py`
- `scripts/assert_p28_zpy_safe_relative.py`
- `scripts/assert_p28_manifest_semantic_aggregate.py`
- `scripts/assert_p28_package_validation_paths.py`
- `P27_STATUS_EVIDENCE_MANIFEST.json`
- `handoffs/p28/PHASE_00_REPORT.md`
- `handoffs/p28/PHASE_01_REPORT.md`

## Claims made

- Claim: C05 queue lease identity no longer uses `timestamp_nanos_opt().unwrap_or_default()`.
  - status: pass
  - evidence: `crates/aidens-contracts/src/tests.rs`, `target/p28/audit/cargo_test_aidens_contracts_p28_phase01.log`
- Claim: C07 `z.py safe_relative` fails closed on outside paths and symlink escapes.
  - status: pass
  - evidence: `scripts/assert_p28_zpy_safe_relative.py`, `target/p28/audit/assert_p28_zpy_safe_relative_phase01.log`
- Claim: C11 profile expansion is result-bearing and no longer panics on assembly failure.
  - status: pass
  - evidence: `crates/aidens-app-kit/src/lib.rs`, `target/p28/audit/cargo_test_aidens_app_kit_p28_phase01.log`
- Claim: C24/C25 patch apply does not create dirty parent directories on failed writes and rejects symlink write targets.
  - status: pass
  - evidence: `crates/aidens-tool-kit/src/lib.rs`, `target/p28/audit/cargo_test_aidens_tool_kit_p28_phase01.log`
- Claim: C32 generated local display IDs no longer use random UUIDs, and stable material-based IDs are available for replay-sensitive paths.
  - status: pass
  - evidence: `crates/aidens-contracts/src/lib.rs`, `target/p28/audit/cargo_test_aidens_contracts_p28_phase01.log`
- Claim: C53 package validation rejects mismatched P27/P28 package path naming.
  - status: pass
  - evidence: `scripts/assert_package_validation.py`, `target/p28/audit/assert_p28_package_validation_paths_phase01.log`
- Claim: C54 an empty known-limitations register no longer blocks completion.
  - status: pass
  - evidence: `target/p28/audit/cargo_test_aidens_contracts_p28_phase01.log`
- Claim: C55 waiver IDs do not satisfy blocked traceability rows unless the row state is `Waived`.
  - status: pass
  - evidence: `target/p28/audit/cargo_test_aidens_contracts_p28_phase01.log`
- Claim: C59 convergence degraded semantics distinguish convergence from exactness loss due to residuals or oscillation.
  - status: pass
  - evidence: `target/p28/audit/cargo_test_aidens_contracts_p28_phase01.log`
- Claim: C66 history preservation no longer requires before/after digest equality when invariant evidence exists.
  - status: pass
  - evidence: `target/p28/audit/cargo_test_aidens_contracts_p28_phase01.log`
- Claim: C72 aggregate semantic status is downgraded when a validation subcheck is degraded.
  - status: pass
  - evidence: `P27_STATUS_EVIDENCE_MANIFEST.json`, `target/p28/audit/assert_p28_manifest_semantic_aggregate_phase01.log`

## Evidence

- `target/p28/audit/cargo_fmt_phase01.log`
- `target/p28/audit/cargo_check_phase01.log`
- `target/p28/audit/cargo_test_aidens_contracts_p28_phase01.log`
- `target/p28/audit/cargo_test_aidens_app_kit_p28_phase01.log`
- `target/p28/audit/cargo_test_aidens_tool_kit_p28_phase01.log`
- `target/p28/audit/assert_p28_zpy_safe_relative_phase01.log`
- `target/p28/audit/assert_p28_manifest_semantic_aggregate_phase01.log`
- `target/p28/audit/assert_p28_package_validation_paths_phase01.log`

## Tests run

```bash
cargo fmt --all -- --check
cargo check --workspace --all-targets
cargo test -p aidens-contracts p28
cargo test -p aidens-app-kit p28
cargo test -p aidens-tool-kit p28
python3 scripts/assert_p28_zpy_safe_relative.py
python3 scripts/assert_p28_manifest_semantic_aggregate.py P27_STATUS_EVIDENCE_MANIFEST.json
python3 scripts/assert_p28_package_validation_paths.py
```

## Failures / degraded checks

- None in Phase 01 validation.

## Open risks

- `generated_artifact_id` is now process-local and non-random, but full replay-sensitive call-site replacement should continue in Phase 02 with material-addressed v11A artifact IDs.
- `z.py` has pre-existing uncommitted P25/P27 archive changes in this workspace; Phase 01 only changed the `safe_relative` fallback behavior.

## Next phase readiness

Ready.
