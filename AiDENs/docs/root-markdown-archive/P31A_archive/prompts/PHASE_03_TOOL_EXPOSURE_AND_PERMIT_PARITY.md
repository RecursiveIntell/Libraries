# Codex Phase 03 Prompt — Tool exposure and permit parity

Use this only after all prior phase gates pass.

## Phase objective

Reduce default tool exposure; ensure disabled tools are unreachable; bind descriptor risk to permit policy.

## Backlog selection

Load `matrices/SUPER_PASS_BACKLOG_1020.csv` and select rows where `Suggested_Phase` contains `Phase 03` or whose category clearly belongs to this phase.

## Required work

1. Inspect relevant crates/files.
2. Implement fixes or explicit quarantines.
3. Add semantic/hostile tests that fail without the fix.
4. Run targeted tests, then broader command bar if feasible.
5. Update matrix statuses.
6. Write `PHASE_03_REPORT.md`.

## Exit gate

Default safe plan excludes admin-risk tools; disabled tool routing fails with receipt.

## Completion rule

Do not move to the next phase with raw `open` rows in this phase. Use `fixed`, `quarantined`, `deferred`, `superseded`, or `open-blocking`.
