# P20 Phase 09 Report - Reference Interpreters and Hostile Tests

Phase: `09`
Scope: reference interpreters and hostile semantic tests
Result: `PASS`

## Operator Injection

Proceed to Phase 09 only.

Focus: reference interpreters and hostile tests.

Search for deferred reference behavior. For every feature still marked supported, implement reference behavior or demote the feature claim.

Required hostile surfaces:

- temporal/as-of semantics if claimed;
- bridge digest/backpointer atomicity;
- provider capability truth;
- agency decision semantics;
- boundary repair/treatment integrity;
- runtime widening disclosure;
- repair-record invariants.

Stop after Phase 09.

## Files Changed

- `crates/aidens-testkit/src/lib.rs`
- `crates/aidens-testkit/Cargo.toml`
- `crates/aidens-integration-tests/tests/phase_09_reference_hostile_tests.rs`
- `tests/fixtures/reference/reference_interpreter_report_v1.json`
- `README.md`
- `STATUS.md`
- `SOURCE_BASIS.md`
- `MASTER_ISSUE_MATRIX.md`
- `NEXT_CODEX_TASK_MATRIX.md`
- `docs/MASTER_ISSUE_MATRIX.md`
- `docs/CURRENT_AIDENS_AUDIT.md`
- `docs/P20_MASTER_ISSUE_MATRIX.md`
- `docs/P20_REFERENCE_INTERPRETER_CLOSEOUT.md`
- `docs/P20_RISK_REGISTER.md`
- `docs/p20/CURRENT_AIDENS_AUDIT.md`
- `docs/p20/DOCS_CODE_TRUTH_REPORT.md`
- `docs/p20/P20_MASTER_ISSUE_MATRIX.md`
- `docs/p20/P20_REFERENCE_INTERPRETER_CLOSEOUT.md`
- `docs/p20/P20_RISK_REGISTER.md`
- `docs/p20/reports/PHASE_09_REPORT.md`

## Deferred Reference Search

Found and removed the live deferred reference behavior in `aidens-testkit`:

- `ReferenceDomainV1::TemporalQuery` no longer returns a `deferred=true` marker.
- `reference_cases()` now includes a temporal query case.
- `evaluate_reference_cases()` now reports interpreter id `aidens-testkit:p09`.

Remaining grep hits are instructional/historical docs or invariant checklist language, not executable supported-feature deferrals.

## Reference Behavior Implemented

Added an independent temporal/as-of reference interpreter for `ReferenceDomainV1::TemporalQuery`.

The interpreter evaluates records by:

- `valid_from <= valid_at`;
- `valid_to` absent or `valid_at < valid_to`;
- `recorded_at <= recorded_at_or_before`;
- sorted visible and hidden claim-version IDs;
- `temporal_mode = "exact"`;
- `deferred = false`.

The reference interpreter remains independent of production memory/runtime code.

## Hostile Surface Coverage

`crates/aidens-integration-tests/tests/phase_09_reference_hostile_tests.rs` covers:

- temporal/as-of reference semantics and canonical memory/runtime comparison;
- bridge digest/backpointer preservation plus atomic rollback and durable failure receipt on import conflict;
- provider capability truth, including cloud/native unavailability and fallback not counting as support;
- agency decision semantics against `evals/p20_agency_eval_cases.jsonl`;
- boundary repair treatment-integrity hard fail with repair receipt and canonical backpointer;
- runtime widening disclosure on degraded temporal query;
- canonical repair record and Forge retraction invariants, including rejection of an empty retraction reason.

## Failures Found

- `ReferenceDomainV1::TemporalQuery` returned a deferred marker despite temporal/as-of behavior being claimed through delegated memory/runtime tests.
- Active docs still said Phase 09 had not run.
- The static reference interpreter report fixture still used `aidens-testkit:p08`.
- The first focused hostile test run had an over-specific reason-code assertion for the exit-resistance agency eval case; the policy outcome, forbidden behavior, and receipt checks were correct.

## Fixes Applied

- Implemented temporal reference behavior in `aidens-testkit`.
- Added Phase 09 hostile/reference tests.
- Added `aidens-agency-kit` and `aidens-boundary-kit` testkit dependencies so hostile tests exercise the real policy and boundary code.
- Updated active docs and risk registers to mark Phase 09 `partial/proved` and Phase 10 still pending.
- Updated the reference interpreter report fixture to `aidens-testkit:p09`.
- Corrected the agency eval hostile test to assert evidence presence without overfitting reason-code text.

## Command Evidence

| Command | Result | Evidence |
|---|---:|---|
| `cargo test -p aidens-testkit --test phase_09_reference_hostile_tests -- --nocapture` | failed first run; one test assertion too strict | `target/p20-phase09/logs/01_phase09_reference_hostile_tests.log` |
| `cargo test -p aidens-testkit --test phase_09_reference_hostile_tests -- --nocapture` | pass | `target/p20-phase09/logs/02_phase09_reference_hostile_tests.log` |
| `cargo test -p aidens-testkit --all-targets --all-features` | pass | `target/p20-phase09/logs/03_cargo_test_aidens_testkit.log` |
| `cargo fmt --all -- --check` | pass | `target/p20-phase09/logs/04_cargo_fmt_check.log` |
| `cargo check --workspace --all-targets --all-features` | pass | `target/p20-phase09/logs/05_cargo_check_workspace.log` |
| `cargo test --workspace --all-targets --all-features` | pass | `target/p20-phase09/logs/06_cargo_test_workspace.log` |
| `cargo clippy --workspace --all-targets --all-features -- -D warnings` | pass | `target/p20-phase09/logs/07_cargo_clippy_workspace.log` |
| Deferred reference grep | reviewed; no live executable deferral remains | `target/p20-phase09/logs/08_deferred_reference_search.log` |
| `cargo fmt --all -- --check` | failed during final rename cleanup; rustfmt then applied | `target/p20-phase09/logs/09_cargo_fmt_check_final.log` |
| `cargo test -p aidens-testkit --test phase_09_reference_hostile_tests -- --nocapture` | pass after final rename | `target/p20-phase09/logs/10_phase09_reference_hostile_tests_final.log` |
| `cargo fmt --all -- --check` | pass after rustfmt | `target/p20-phase09/logs/11_cargo_fmt_check_final.log` |
| `cargo test -p aidens-testkit --test phase_09_reference_hostile_tests -- --nocapture` | pass | `target/p20-phase09/logs/12_phase09_reference_hostile_tests_final.log` |
| `cargo check --workspace --all-targets --all-features` | pass | `target/p20-phase09/logs/13_cargo_check_workspace_final.log` |
| `cargo test --workspace --all-targets --all-features` | pass | `target/p20-phase09/logs/14_cargo_test_workspace_final.log` |
| `cargo clippy --workspace --all-targets --all-features -- -D warnings` | pass | `target/p20-phase09/logs/15_cargo_clippy_workspace_final.log` |
| Deferred reference grep | reviewed; only instructional/report text remains | `target/p20-phase09/logs/16_deferred_reference_search_final.log` |
| `python3 scripts/p20_scan_aidens.py --root . --out target/p20-phase09/scan-through-09 --require-phase-reports-through 9 --fail-on-blocking` | pass | log: `target/p20-phase09/logs/17_p20_scan_through_09.log`; JSON: `target/p20-phase09/scan-through-09/p20_scan.json`; markdown: `target/p20-phase09/scan-through-09/p20_scan.md`; blocking findings: `0`; warnings: `21` |
| `P20_REQUIRED_PHASE_REPORT_THROUGH=9 bash scripts/p20_verify.sh` | pass | log: `target/p20-phase09/logs/18_p20_verify_through_09.log`; scanner output: `target/p20-scan/p20_scan.md`; blocking findings: `0`; agency eval fixture shape: `10 cases`; completed successfully |
| `python3 scripts/p20_scan_aidens.py --root . --out target/p20-phase09/scan-through-09-final --require-phase-reports-through 9 --fail-on-blocking` | pass | log: `target/p20-phase09/logs/19_p20_scan_through_09_final.log`; JSON: `target/p20-phase09/scan-through-09-final/p20_scan.json`; markdown: `target/p20-phase09/scan-through-09-final/p20_scan.md`; blocking findings: `0`; warnings: `21` |

## Unresolved Blockers

None for Phase 09.

P20 is not final-complete. Phase 10 has not run, and the final audit bundle has not been generated.

## Phase Gate

Phase 09 gate: `PASS`

Stop here and wait for the Phase 10 operator injection.
