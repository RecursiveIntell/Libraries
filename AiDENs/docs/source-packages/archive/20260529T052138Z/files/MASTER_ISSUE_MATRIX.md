# Master Issue Matrix - P20 Active

This is the active issue matrix for `P20_TRUTHFUL_FINISH_AND_RELEASE_HARDENING`. Older P00-P19 task files remain historical evidence and should not be read as current support claims.

| ID | Phase | Status | Issue | Acceptance / evidence |
|---|---:|---|---|---|
| P20-000 | 00 | supported | Operator arbitration, source basis, and baseline plan | `docs/p20/reports/PHASE_00_REPORT.md` |
| P20-001 | 01 | supported | Build truth: fmt/check/test/clippy/verify | `docs/p20/reports/PHASE_01_REPORT.md`; `target/p20-phase01/logs/` |
| P20-002 | 02 | supported | Documentation honesty | `docs/p20/DOCS_CODE_TRUTH_REPORT.md`; active docs patched |
| P20-003 | 03 | supported | Contract ownership and shadow-truth collapse | `docs/p20/CONTRACT_OWNERSHIP_INVENTORY.md`; `docs/p20/CONTRACT_OWNERSHIP_INVENTORY.json` |
| P20-004 | 04 | supported | Boundary scanner and verify gate integration | `scripts/p20_scan_aidens.py`; `scripts/p20_verify.sh`; `target/p20-scan/` |
| P20-005 | 05 | supported | Provider capability truth | `docs/p20/PROVIDER_CAPABILITY_MATRIX.md`; provider readiness tests |
| P20-006 | 06 | partial/proved | Runner vertical slice proof | `crates/aidens-app-kit/tests/phase_06_runner_vertical_slice.rs`; `tests/fixtures/p06/runner_vertical_slice_aidens.toml` |
| P20-007 | 07 | partial/proved | Canonical adapter proof | `crates/aidens-integration-tests/tests/phase_07_canonical_adapter_proof.rs` |
| P20-008 | 08 | partial/proved | Agency/influence governance | `crates/aidens-agency-kit/src/lib.rs`; `crates/aidens-runner/tests/phase_08_agency_gate.rs`; `evals/p20_agency_eval_cases.jsonl` |
| P20-009 | 09 | partial/proved | Reference interpreters and hostile tests | `crates/aidens-testkit/src/lib.rs`; `crates/aidens-integration-tests/tests/phase_09_reference_hostile_tests.rs` |
| P20-010 | 10 | deferred | Final audit bundle and hostile auditor handoff | Requires `scripts/p20_verify.sh` and `scripts/p20_generate_audit_bundle.sh` pass |

## Current Blockers After Phase 09

| Blocker | Label | Owner phase |
|---|---|---:|
| Cloud provider HTTP execution is unavailable | `deferred/unavailable` | 05 |
| Native provider tool loops are unavailable | `deferred` | 05 |
| P20 final audit bundle is not generated | `deferred` | 10 |
