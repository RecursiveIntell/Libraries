# Codex Phase 14 Prompt — Replace marker tests with semantic hostile fixtures

Use this only after all prior phase gates pass.

## Phase objective

Upgrade marker assertion scripts into behavioral tests or retire them; add adversarial fixtures.

## Backlog selection

Load `matrices/SUPER_PASS_BACKLOG_1020.csv` and select rows where `Suggested_Phase` contains `Phase 14` or whose category clearly belongs to this phase.

## Required work

1. Inspect relevant crates/files.
2. Implement fixes or explicit quarantines.
3. Add semantic/hostile tests that fail without the fix.
4. Run targeted tests, then broader command bar if feasible.
5. Update matrix statuses.
6. Write `PHASE_14_REPORT.md`.

## Exit gate

Verifier refuses marker-only completion for hard gates.

## Completion rule

Do not move to the next phase with raw `open` rows in this phase. Use `fixed`, `quarantined`, `deferred`, `superseded`, or `open-blocking`.
