# P20 Phase 06 Report - Runner Vertical Slice Proof

Phase: `06`
Scope: proof of execution
Result: `PASS`

## Operator Injection

Proceed to Phase 06 only.

Focus: proof of execution.

Implement/prove one vertical slice:

```text
config -> runner -> provider/mock or ollama -> tool exposure -> permit check -> tool call parse/repair -> tool execution -> final response -> event log -> receipts/control records -> audit report
```

Forbidden:

- mocks that bypass runner control flow;
- tool execution without permit/exposure record;
- final answer without receipts;
- parse repair without repair receipt.

## Files Changed

- `crates/aidens-runner/src/lib.rs`
- `crates/aidens-app-kit/tests/phase_06_runner_vertical_slice.rs`
- `tests/fixtures/p06/runner_vertical_slice_aidens.toml`
- `README.md`
- `STATUS.md`
- `docs/MASTER_ISSUE_MATRIX.md`
- `docs/CURRENT_AIDENS_AUDIT.md`
- `docs/p20/reports/PHASE_06_REPORT.md`

## Vertical Slice Evidence

The new fixture-backed integration test is:

- `crates/aidens-app-kit/tests/phase_06_runner_vertical_slice.rs`

The stable config fixture is:

- `tests/fixtures/p06/runner_vertical_slice_aidens.toml`

The test proves the required path through real app and runner control flow:

| Required step | Evidence |
|---|---|
| config | Test writes the fixture config to a temp run directory and builds through `AiDENsApp::builder().config_file(...)`. |
| runner | Test executes `app.run_once(...)`, which calls `AiDENsRunner::run(...)`. |
| provider/mock | Fixture uses executable `mock` provider with scripted multi-turn output. |
| tool exposure | Test asserts `ToolExposureSetV1` includes `aidens:repo-read:1` and has a matching durable `tool-exposure-plan-v1` event-log record. |
| permit check | Test asserts the read-only tool is exposed with `permit_required=false` and the side-effect `aidens:patch-apply:1` decision is blocked with `permit-required:write` and an approval request. |
| tool call parse/repair | Mock response emits a fenced JSON tool call; runner parser fallback repairs the boundary and emits one changed boundary repair receipt. |
| tool execution | Runner dispatches `aidens:repo-read:1`; test asserts a successful invocation receipt with run/attempt IDs and output digest. |
| final response | Mock provider receives tool results on the second loop and returns final text containing the tool output. |
| event log | Test reopens `receipts/canonical-receipts.ndjson` and verifies all event-log record digests. |
| receipts/control records | Event log contains `tool-exposure-plan-v1`, `run-report-v1`, and a `verification-control` `control-receipt` for `final-output-produced`. |
| audit report | The durable `run-report-v1` event-log record contains the run, turn, tool call, repair, invocation, and stop-rule receipt evidence. |

## Failures Found

- Successful runner turns persisted only the `run-report-v1` record. The in-memory `ToolExposureSetV1` was not separately appended to the canonical event log, so a restarted auditor could inspect the run report but not a first-class durable tool-exposure record.
- Successful final-output turns did not append a canonical control receipt. Failure paths had control receipts; the final-output path lacked equivalent durable control-plane evidence.
- Permit-use receipts returned by the tool dispatcher were referenced by invocation receipt IDs but were not added to `RunReportV1::permit_use_receipts`.
- Active docs still described the runner vertical slice as Phase 06 pending.

## Fixes Applied

- `TurnExecutorV1` now copies exposure approval and permit-use evidence into the run report at turn start.
- Completed runs with a canonical receipt store now append a durable `tool-exposure-plan-v1` orchestration record before the durable `run-report-v1`.
- Successful final-output turns now append a `verification-control` control receipt for the final-output control decision using the existing `CanonicalGovernanceAdapter`.
- Tool dispatcher permit-use receipts are now copied into the run report with the runner context when a permitted side-effect tool executes.
- Added the Phase 06 fixture and integration test to prove the full config-to-runner-to-receipts path without bypassing runner logic.
- Updated active status docs to show Phase 06 as `partial/proved`, while keeping final P20 and later phases explicitly unreached.

## Command Evidence

| Command | Result | Evidence |
|---|---:|---|
| `cargo fmt --all -- --check` | pass | `target/p20-phase06/logs/01_cargo_fmt_check.log` |
| `cargo test -p aidens-app-kit --test phase_06_runner_vertical_slice -- --nocapture` | pass | `target/p20-phase06/logs/02_phase06_vertical_slice_test.log` |
| `cargo test -p aidens-runner --lib` | pass | `target/p20-phase06/logs/03_runner_lib_tests.log` |
| `cargo check --workspace --all-targets --all-features` | pass | `target/p20-phase06/logs/04_cargo_check.log` |
| `cargo test --workspace --all-targets --all-features` | pass | `target/p20-phase06/logs/05_cargo_test.log` |
| `cargo clippy --workspace --all-targets --all-features -- -D warnings` | pass | `target/p20-phase06/logs/06_cargo_clippy.log` |
| `python3 scripts/p20_scan_aidens.py --root . --out target/p20-phase06/scan-through-06 --require-phase-reports-through 6 --fail-on-blocking` | pass | `target/p20-phase06/logs/07_p20_scan_through_06.log`, `target/p20-phase06/scan-through-06/p20_scan.json`, `target/p20-phase06/scan-through-06/p20_scan.md` |
| `P20_REQUIRED_PHASE_REPORT_THROUGH=6 bash scripts/p20_verify.sh` | pass | `target/p20-phase06/logs/08_p20_verify_through_06.log`, `target/aidens-final-audit/` |
| `python3 scripts/p20_scan_aidens.py --root . --out target/p20-phase06/scan-through-06-final --require-phase-reports-through 6 --fail-on-blocking` | pass | `target/p20-phase06/logs/09_p20_scan_through_06_final.log`, `target/p20-phase06/scan-through-06-final/p20_scan.json`, `target/p20-phase06/scan-through-06-final/p20_scan.md` |

## Unresolved Blockers

None for Phase 06.

P20 is not final-complete. Phases 07-10 have not run, and the final audit bundle has not been generated.

## Phase Gate

Phase 06 gate: `PASS`

Stop here and wait for the Phase 07 operator injection.
