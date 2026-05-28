# P20 Phase 08 Report - Agency and Influence Governance

Phase: `08`
Scope: agency and influence governance
Result: `PASS`

## Operator Injection

Proceed to Phase 08 only.

Focus: agency and influence governance.

Implement a real policy gate, not prompt-only language.

Required:

- influence classification;
- agency policy input/decision/outcome;
- high-impact recommendation gate;
- memory influence trace;
- repeated nudge counter/budget;
- tool-output persuasion risk gate;
- advice/influence receipts;
- evals from `evals/p20_agency_eval_cases.jsonl`.

At least one real runner/generation path must call the agency gate before output.

Forbidden:

- decorative alternatives;
- unreceipted memory personalization;
- repeated semantic paraphrase nudges bypassing counters;
- emotional dependence or exit-resistance hooks.

Stop after Phase 08.

## Files Changed

- `Cargo.toml`
- `Cargo.lock`
- `crates/aidens-agency-kit/Cargo.toml`
- `crates/aidens-agency-kit/src/lib.rs`
- `crates/aidens-runner/Cargo.toml`
- `crates/aidens-runner/src/lib.rs`
- `crates/aidens-runner/tests/phase_08_agency_gate.rs`
- `crates/aidens-contracts/src/lib.rs`
- `crates/aidens-app-kit/tests/phase_06_runner_vertical_slice.rs`
- `crates/aidens/Cargo.toml`
- `crates/aidens/src/lib.rs`
- `README.md`
- `STATUS.md`
- `docs/MASTER_ISSUE_MATRIX.md`
- `docs/CURRENT_AIDENS_AUDIT.md`
- `docs/P20_RISK_REGISTER.md`
- `docs/p20/CURRENT_AIDENS_AUDIT.md`
- `docs/p20/DOCS_CODE_TRUTH_REPORT.md`
- `docs/p20/P20_RISK_REGISTER.md`
- `docs/p20/reports/PHASE_08_REPORT.md`

## Gate Implementation

Added `aidens-agency-kit` as an AiDENs-owned boundary crate. It implements:

- `InfluenceClassV1`;
- `AgencyPolicyInputV1`;
- `AgencyPolicyDecisionV1`;
- `AgencyPolicyOutcomeV1`;
- high-impact recommendation classification and gating;
- memory influence traces and sensitive-signal redaction receipts;
- semantic repeated-nudge counting through `NudgeLedgerV1` and `PersuasionBudgetV1`;
- tool-output persuasion risk classification for urgency/scarcity claims;
- delegated influence aggregation;
- advice, influence, high-impact, repeated-steering, memory, tool-output, privacy, and incident receipts.

The policy is executable Rust logic. It is not prompt-only text.

## Runner Integration Proof

`aidens-runner` now owns an `AgencyPolicyEngineV1` and shared `NudgeLedgerV1`.

The real runner path calls the agency gate:

- after successful tool output and before the next provider generation can use that tool output;
- before returning any final provider output.

When a gate allows output with disclosure, the returned text includes the agency disclosure. When a gate requires alternatives, confirmation, external review, block, or quarantine, the runner stops before returning the provider's candidate output, records `StopRuleV1::AgencyPolicy`, and emits agency policy reports.

Durable runs with `CanonicalEventLog` now append `aidens-agency-kit` / `agency-policy-report-v1` records. `RunReportV1` carries `agency_receipt_ids` so the orchestration receipt points to the agency evidence.

## Eval Coverage

`aidens-agency-kit` unit tests load `evals/p20_agency_eval_cases.jsonl` and verify, for every case:

- expected policy outcome;
- all required receipt type names;
- forbidden behavior handled by the gate.

Covered eval surfaces include high-impact single-path advice, decorative alternatives, repeated paraphrased nudges, memory-personalization vulnerability use, tool-output urgency, delegated influence aggregation, exit-resistance/guilt hooks, sycophancy overvalidation, user-requested manipulation, and sensitive receipt redaction.

## Failures Found

- No `aidens-agency-kit` crate existed.
- The runner final-output path returned provider text without an agency policy decision.
- Tool outputs could feed follow-up generation without a tool-output persuasion risk gate.
- Repeated semantic paraphrase nudges had no counter or budget.
- Phase 06/runner durable-record tests assumed pre-agency record counts and needed to assert the new agency receipt records.
- Initial focused tests caught missing sycophancy advice receipts and semantic paraphrase counter coverage.
- Initial clippy caught derivable default implementations in the new agency crate.

## Fixes Applied

- Added `aidens-agency-kit` and workspace/umbrella wiring.
- Added runner agency gate wiring for final output and tool-output-to-follow-up generation.
- Added `StopRuleV1::AgencyPolicy` and `RunReportV1::agency_receipt_ids`.
- Added `crates/aidens-runner/tests/phase_08_agency_gate.rs`.
- Updated Phase 06 and runner durable-log tests to require agency records in addition to existing control, tool-exposure, and run records.
- Updated active docs to mark agency governance `partial/proved` for the tested paths and to keep P20 phases 09-10 unreached.

## Command Evidence

| Command | Result | Evidence |
|---|---:|---|
| `cargo fmt --all -- --check` | pass | `target/p20-phase08/logs/01_cargo_fmt_check.log` |
| `cargo test -p aidens-agency-kit -- --nocapture` | pass | `target/p20-phase08/logs/02_agency_kit_tests.log` |
| `cargo test -p aidens-runner --test phase_08_agency_gate -- --nocapture` | pass | `target/p20-phase08/logs/03_runner_phase08_agency_gate.log` |
| `python3 scripts/p20_validate_agency_cases.py evals/p20_agency_eval_cases.jsonl` | pass | `target/p20-phase08/logs/04_agency_eval_fixture_validation.log` |
| `cargo check --workspace --all-targets --all-features` | pass | `target/p20-phase08/logs/05_cargo_check_workspace.log` |
| `cargo test --workspace --all-targets --all-features` | pass | `target/p20-phase08/logs/06_cargo_test_workspace.log` |
| `cargo clippy --workspace --all-targets --all-features -- -D warnings` | pass | `target/p20-phase08/logs/07_cargo_clippy_workspace.log` |
| `cargo test -p aidens-app-kit --test phase_06_runner_vertical_slice` | pass | `target/p20-phase08/logs/08_phase06_vertical_slice_regression.log` |
| `cargo test -p aidens-runner --lib` | pass | `target/p20-phase08/logs/09_runner_lib_regression.log` |
| `python3 scripts/p20_scan_aidens.py --root . --out target/p20-phase08/scan-through-08 --require-phase-reports-through 8 --fail-on-blocking` | pass | log: `target/p20-phase08/logs/10_p20_scan_through_08.log`; JSON: `target/p20-phase08/scan-through-08/p20_scan.json`; markdown: `target/p20-phase08/scan-through-08/p20_scan.md`; blocking findings: `0`; warnings: `21` |
| `P20_REQUIRED_PHASE_REPORT_THROUGH=8 bash scripts/p20_verify.sh` | pass | log: `target/p20-phase08/logs/11_p20_verify_through_08.log`; scanner output: `target/p20-scan/p20_scan.md`; blocking findings: `0`; agency eval fixture shape: `10 cases`; completed successfully |
| `python3 scripts/p20_scan_aidens.py --root . --out target/p20-phase08/scan-through-08-final --require-phase-reports-through 8 --fail-on-blocking` | pass | log: `target/p20-phase08/logs/12_p20_scan_through_08_final.log`; JSON: `target/p20-phase08/scan-through-08-final/p20_scan.json`; markdown: `target/p20-phase08/scan-through-08-final/p20_scan.md`; blocking findings: `0`; warnings: `21` |

## Unresolved Blockers

None for Phase 08.

P20 is not final-complete. Phases 09-10 have not run, and the final audit bundle has not been generated.

## Phase Gate

Phase 08 gate: `PASS`

Stop here and wait for the Phase 09 operator injection.
