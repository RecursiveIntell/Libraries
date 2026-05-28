# Codex Phase 10 Prompt — Minimal v11B regional recursive/subtractive slice

Use this only after all prior phase gates pass.

## Phase objective

Right-graph declaration; one region contract; convergence/non-convergence/oscillation; syndrome/residual; local repair; support core; oracle diff.

## Backlog selection

Load `matrices/SUPER_PASS_BACKLOG_1020.csv` and select rows where `Suggested_Phase` contains `Phase 10` or whose category clearly belongs to this phase.

## Required work

1. Inspect relevant crates/files.
2. Implement fixes or explicit quarantines.
3. Add semantic/hostile tests that fail without the fix.
4. Run targeted tests, then broader command bar if feasible.
5. Update matrix statuses.
6. Write `PHASE_10_REPORT.md`.

## Exit gate

One deterministic v11B vertical slice passes; v11B-complete remains forbidden.

## Completion rule

Do not move to the next phase with raw `open` rows in this phase. Use `fixed`, `quarantined`, `deferred`, `superseded`, or `open-blocking`.
