# Codex Phase 15 Prompt — Docs, evidence, known limitations, and label closure

Use this only after all prior phase gates pass.

## Phase objective

Populate known limitations; final auditor handoff; classify all issues; update support traceability and forbidden labels.

## Backlog selection

Load `matrices/SUPER_PASS_BACKLOG_1020.csv` and select rows where `Suggested_Phase` contains `Phase 15` or whose category clearly belongs to this phase.

## Required work

1. Inspect relevant crates/files.
2. Implement fixes or explicit quarantines.
3. Add semantic/hostile tests that fail without the fix.
4. Run targeted tests, then broader command bar if feasible.
5. Update matrix statuses.
6. Write `PHASE_15_REPORT.md`.

## Exit gate

Final docs reconcile with actual package and test evidence.

## Completion rule

Do not move to the next phase with raw `open` rows in this phase. Use `fixed`, `quarantined`, `deferred`, `superseded`, or `open-blocking`.
