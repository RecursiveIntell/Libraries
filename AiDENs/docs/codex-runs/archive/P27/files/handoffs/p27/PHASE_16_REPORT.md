# P27 Phase Report

## Phase

- Phase ID: 16
- Phase title: Agency/governance eval harness hardening
- Date: 2026-05-05T11:29:37Z

## Scope

- Intended work: grow agency/governance eval coverage for manipulation, scarcity/urgency, decorative alternatives, relational boundaries, and control cases while keeping the classifier honestly labeled heuristic.
- Issue IDs in scope: `P27-013` classification where touched; agency/governance eval harness hardening from Phase 16 plan.
- Explicit non-goals: no formal classifier claim, no production policy widening, no canonical governance ownership transfer, no broad clippy allow sweep, no high-argument API rewrite outside touched work.

## Files inspected

- `prompts/phases/P27_PHASE_16_PROMPT.md`
- `phase_injections/P27_GATE_AFTER_PHASE_16_BEFORE_PHASE_17.md`
- `P27_PHASE_PLAN.md`
- `P27_MASTER_ISSUE_MATRIX.md`
- `P27_ACCEPTANCE_GATES.md`
- `P27_11A_ALIGNMENT.md`
- `STATUS.md`
- `SUPPORT_PROFILE.md`
- `SOURCE_BASIS.md`
- `crates/aidens-agency-kit/src/lib.rs`
- `crates/aidens-governance-kit/src/lib.rs`
- `crates/aidens-runner/tests/phase_06_agency_v02.rs`
- `crates/aidens-runner/tests/phase_08_agency_gate.rs`
- `crates/aidens-integration-tests/tests/phase_09_reference_hostile_tests.rs`
- `evals/p20_agency_eval_cases.jsonl`
- `evals/p21_agency_eval_cases.jsonl`

## Files changed

- `STATUS.md`
- `crates/aidens-agency-kit/src/lib.rs`
- `evals/p20_agency_eval_cases.jsonl`
- `scripts/assert_p27_agency_eval_harness.py`
- `scripts/p27_verify.sh`
- `handoffs/p27/PHASE_16_REPORT.md`

## Changes made

- Added seven P27 eval cases to `evals/p20_agency_eval_cases.jsonl`:
  - `p27_tool_scarcity_deadline`
  - `p27_high_impact_housing_urgency_single_path`
  - `p27_decorative_alternatives_one_viable`
  - `p27_requested_guilt_manipulation`
  - `p27_dependency_loop_direct`
  - `p27_tool_conflict_without_urgency`
  - `p27_low_impact_urgency_control`
- Hardened the agency-kit eval test to require the P27 case set and at least 28 total cases.
- Preserved `AGENCY_POLICY_CLASSIFIER_V1 = "aidens-heuristic-boundary-classifier-v1"` and the explicit `HeuristicBoundaryClassifier` classifier kind.
- Added `scripts/assert_p27_agency_eval_harness.py` to guard the required P27 eval cases and heuristic label.
- Wired the agency eval harness guard into `scripts/p27_verify.sh`.
- Updated `STATUS.md` to record Phase 16 evidence and classify `P27-013` as only partially classified, not closed.

## Commands run

| Command | Result | Log |
|---|---|---|
| `cargo fmt` | pass | `target/p27/audit/phase16_cargo_fmt.log` |
| JSONL validation/count summary | pass | `target/p27/audit/phase16_validate_agency_eval_jsonl.log` |
| `cargo test -p aidens-agency-kit agency_eval_cases_drive_policy_and_receipts` | pass | `target/p27/audit/phase16_cargo_test_agency_eval_cases.log` |
| `cargo test -p aidens-agency-kit` | pass | `target/p27/audit/phase16_cargo_test_agency_kit.log` |
| `cargo test -p aidens-integration-tests agency_eval_cases_match_decision_semantics_and_required_receipts` | pass | `target/p27/audit/phase16_cargo_test_integration_agency_eval_cases.log` |
| `cargo test -p aidens-runner phase_08_agency_gate` | ran zero tests due filter mismatch; superseded by `--test` run | `target/p27/audit/phase16_cargo_test_runner_phase08_agency_gate.log` |
| `cargo test -p aidens-runner phase_06_agency_v02` | ran zero tests due filter mismatch; superseded by `--test` run | `target/p27/audit/phase16_cargo_test_runner_phase06_agency_v02.log` |
| `cargo test -p aidens-runner --test phase_08_agency_gate` | pass | `target/p27/audit/phase16_cargo_test_runner_phase08_agency_gate_full.log` |
| `cargo test -p aidens-runner --test phase_06_agency_v02` | pass | `target/p27/audit/phase16_cargo_test_runner_phase06_agency_v02_full.log` |
| `cargo test -p aidens-governance-kit` | pass | `target/p27/audit/phase16_cargo_test_governance_kit.log` |
| `cargo check -p aidens-agency-kit -p aidens-runner -p aidens-integration-tests` | pass | `target/p27/audit/phase16_cargo_check_agency_runner_integration.log` |
| `python3 -m py_compile scripts/assert_p27_agency_eval_harness.py` | pass | `target/p27/audit/phase16_py_compile_agency_eval_guard.log` |
| `python3 scripts/assert_p27_agency_eval_harness.py .` | pass | `target/p27/audit/phase16_assert_agency_eval_harness.log` |
| `cargo fmt --check` | pass | `target/p27/audit/phase16_cargo_fmt_check.log` |
| `P27_SKIP_CARGO=1 bash scripts/verify_current.sh` | pass | `target/p27/audit/phase16_verify_current_skip_cargo.log` |
| `python3 scripts/assert_support_claims.py .` | pass | `target/p27/audit/phase16_assert_support_claims.log` |
| `python3 scripts/assert_p27_current_run_truth.py .` | pass | `target/p27/audit/phase16_assert_p27_current_run_truth.log` |
| `python3 scripts/assert_p27_agents_md_current.py .` | pass | `target/p27/audit/phase16_assert_p27_agents_md_current.log` |

## Evidence emitted

- `target/p27/audit/phase16_validate_agency_eval_jsonl.log`
- `target/p27/audit/phase16_assert_agency_eval_harness.log`
- `target/p27/audit/phase16_cargo_test_agency_eval_cases.log`
- `target/p27/audit/phase16_cargo_test_agency_kit.log`
- `target/p27/audit/phase16_cargo_test_integration_agency_eval_cases.log`
- `target/p27/audit/phase16_cargo_test_runner_phase08_agency_gate_full.log`
- `target/p27/audit/phase16_cargo_test_runner_phase06_agency_v02_full.log`
- `target/p27/audit/phase16_cargo_test_governance_kit.log`
- `target/p27/audit/phase16_cargo_check_agency_runner_integration.log`
- `target/p27/audit/phase16_verify_current_skip_cargo.log`
- `target/p27/audit/phase16_assert_support_claims.log`

## 11A semantic impact

- Exact/approx labels touched: no label was changed. The existing heuristic classifier label remains `aidens-heuristic-boundary-classifier-v1`.
- Degradation labels touched: none.
- Support labels touched: no `SUPPORT_PROFILE.md` support-tier claim changed.
- Proof/check hooks added: P27 agency eval guard checks required eval cases and heuristic label; agency-kit and integration tests assert decisions, receipts, forbidden behavior, and classifier kind.

## Support profile impact

- No support-tier claim changed in `SUPPORT_PROFILE.md`.
- `STATUS.md` records Phase 16 evidence and classifies `P27-013` as partially classified rather than closed.
- No agency/governance path was promoted beyond partial/to-be-revalidated support.

## Canonical-owner impact

- No canonical-owner boundary changed.
- Governance truth remains delegated to sibling `verification-*` crates through `aidens-governance-kit`.
- `aidens-agency-kit` remains an AiDENs-facing heuristic boundary classifier and receipt surface, not canonical governance truth.

## Issues closed

- None fully closed.

## Issues classified

- `P27-013`: partially classified. Phase 16 did not touch or widen high-argument governance APIs; broader constructor/builder cleanup remains open for later targeted work.

## New issues / risks

- The agency classifier remains heuristic-v0.1/heuristic-v1 label only; it is not a formal proof system.
- `evals/p21_agency_eval_cases.jsonl` remains a broader aspirational/hostile fixture and was not wired as authoritative Phase 16 acceptance.

## Decision

Rationale: P27 eval coverage was expanded across the requested risk families, governance blocks remain executable in agency-kit/integration/runner tests, the heuristic label is guarded, and no support or canonical ownership claim was widened.

Decision: continue
