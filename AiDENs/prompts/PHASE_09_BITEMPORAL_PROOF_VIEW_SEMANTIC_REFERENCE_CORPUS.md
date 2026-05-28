# Codex Phase 09 Prompt — Bitemporal/proof/view semantic reference corpus

Use this only after all prior phase gates pass.

## Phase objective

Reference interpreter for valid/recorded time, retroactive correction, supersession, stale projections, view widening, proof debt/refutation.

## Backlog selection

Load `matrices/SUPER_PASS_BACKLOG_1020.csv` and select rows where `Suggested_Phase` contains `Phase 09` or whose category clearly belongs to this phase.

## Required work

1. Inspect relevant crates/files.
2. Implement fixes or explicit quarantines.
3. Add semantic/hostile tests that fail without the fix.
4. Run targeted tests, then broader command bar if feasible.
5. Update matrix statuses.
6. Write `PHASE_09_REPORT.md`.

## Exit gate

Reference fixture corpus passes; degraded answer cannot masquerade as exact.

## Completion rule

Do not move to the next phase with raw `open` rows in this phase. Use `fixed`, `quarantined`, `deferred`, `superseded`, or `open-blocking`.
