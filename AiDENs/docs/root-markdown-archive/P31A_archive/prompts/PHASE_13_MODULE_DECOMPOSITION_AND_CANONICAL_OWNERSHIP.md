# Codex Phase 13 Prompt — Module decomposition and canonical ownership

Use this only after all prior phase gates pass.

## Phase objective

Split large crates/files; enforce owner boundaries; add boundary scanner to verify gate.

## Backlog selection

Load `matrices/SUPER_PASS_BACKLOG_1020.csv` and select rows where `Suggested_Phase` contains `Phase 13` or whose category clearly belongs to this phase.

## Required work

1. Inspect relevant crates/files.
2. Implement fixes or explicit quarantines.
3. Add semantic/hostile tests that fail without the fix.
4. Run targeted tests, then broader command bar if feasible.
5. Update matrix statuses.
6. Write `PHASE_13_REPORT.md`.

## Exit gate

Mega-file budgets enforced; owner scanner passes; no shadow truth owners.

## Completion rule

Do not move to the next phase with raw `open` rows in this phase. Use `fixed`, `quarantined`, `deferred`, `superseded`, or `open-blocking`.
