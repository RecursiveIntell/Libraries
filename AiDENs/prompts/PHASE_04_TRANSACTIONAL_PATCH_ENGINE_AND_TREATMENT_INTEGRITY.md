# Codex Phase 04 Prompt — Transactional patch engine and treatment integrity

Use this only after all prior phase gates pass.

## Phase objective

Replace patch_apply narrow string replacement or relabel it; add real transactional patch subsystem, before/after digests, rollback/quarantine.

## Backlog selection

Load `matrices/SUPER_PASS_BACKLOG_1020.csv` and select rows where `Suggested_Phase` contains `Phase 04` or whose category clearly belongs to this phase.

## Required work

1. Inspect relevant crates/files.
2. Implement fixes or explicit quarantines.
3. Add semantic/hostile tests that fail without the fix.
4. Run targeted tests, then broader command bar if feasible.
5. Update matrix statuses.
6. Write `PHASE_04_REPORT.md`.

## Exit gate

Multi-file patch atomicity tests; repeated-content hunk tests; read failure cannot create/replace silently.

## Completion rule

Do not move to the next phase with raw `open` rows in this phase. Use `fixed`, `quarantined`, `deferred`, `superseded`, or `open-blocking`.
