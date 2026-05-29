# Codex Phase 05 Prompt — Command execution receipts and environment control

Use this only after all prior phase gates pass.

## Phase objective

Structured argv; process-group kill; stdout/stderr caps; env/toolchain/package fingerprints; replay handles.

## Backlog selection

Load `matrices/SUPER_PASS_BACKLOG_1020.csv` and select rows where `Suggested_Phase` contains `Phase 05` or whose category clearly belongs to this phase.

## Required work

1. Inspect relevant crates/files.
2. Implement fixes or explicit quarantines.
3. Add semantic/hostile tests that fail without the fix.
4. Run targeted tests, then broader command bar if feasible.
5. Update matrix statuses.
6. Write `PHASE_05_REPORT.md`.

## Exit gate

Quoted args, grandchild timeout, output cap, PATH drift fixtures pass.

## Completion rule

Do not move to the next phase with raw `open` rows in this phase. Use `fixed`, `quarantined`, `deferred`, `superseded`, or `open-blocking`.
