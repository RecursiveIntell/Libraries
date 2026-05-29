# Codex Phase 11 Prompt — Schema governance and generated artifacts

Use this only after all prior phase gates pass.

## Phase objective

Generate schemas from canonical types; meta-validate; digest schemas; compatibility diff gates; reject unsupported schema features.

## Backlog selection

Load `matrices/SUPER_PASS_BACKLOG_1020.csv` and select rows where `Suggested_Phase` contains `Phase 11` or whose category clearly belongs to this phase.

## Required work

1. Inspect relevant crates/files.
2. Implement fixes or explicit quarantines.
3. Add semantic/hostile tests that fail without the fix.
4. Run targeted tests, then broader command bar if feasible.
5. Update matrix statuses.
6. Write `PHASE_11_REPORT.md`.

## Exit gate

Schema gen/diff/meta-validation gate passes.

## Completion rule

Do not move to the next phase with raw `open` rows in this phase. Use `fixed`, `quarantined`, `deferred`, `superseded`, or `open-blocking`.
