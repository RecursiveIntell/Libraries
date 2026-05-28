# Codex Phase 20 Prompt — Final package, extracted replay, and release bar

Use this only after all prior phase gates pass.

## Phase objective

Run full command bar; generate package sidecars; extracted-package replay; update final labels only if gates pass.

## Backlog selection

Load `matrices/SUPER_PASS_BACKLOG_1020.csv` and select rows where `Suggested_Phase` contains `Phase 20` or whose category clearly belongs to this phase.

## Required work

1. Inspect relevant crates/files.
2. Implement fixes or explicit quarantines.
3. Add semantic/hostile tests that fail without the fix.
4. Run targeted tests, then broader command bar if feasible.
5. Update matrix statuses.
6. Write `PHASE_20_REPORT.md`.

## Exit gate

Clean package sidecars; extracted replay passes; labels truthful.

## Completion rule

Do not move to the next phase with raw `open` rows in this phase. Use `fixed`, `quarantined`, `deferred`, `superseded`, or `open-blocking`.
