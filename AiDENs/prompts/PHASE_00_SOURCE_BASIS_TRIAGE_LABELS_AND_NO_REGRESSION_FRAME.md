# Codex Phase 00 Prompt — Source basis, triage, labels, and no-regression frame

Use this only after all prior phase gates pass.

## Phase objective

Reconcile package-clean fact; normalize issues; classify fixed/quarantined/deferred/open-blocking; do not mark the finished bundle as failed.

## Backlog selection

Load `matrices/SUPER_PASS_BACKLOG_1020.csv` and select rows where `Suggested_Phase` contains `Phase 00` or whose category clearly belongs to this phase.

## Required work

1. Inspect relevant crates/files.
2. Implement fixes or explicit quarantines.
3. Add semantic/hostile tests that fail without the fix.
4. Run targeted tests, then broader command bar if feasible.
5. Update matrix statuses.
6. Write `PHASE_00_REPORT.md`.

## Exit gate

All source-basis docs generated; backlog has statuses; forbidden claims listed.

## Completion rule

Do not move to the next phase with raw `open` rows in this phase. Use `fixed`, `quarantined`, `deferred`, `superseded`, or `open-blocking`.
