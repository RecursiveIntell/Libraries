# Codex Phase 12 Prompt — Artifact lifecycle and operator effect enforcement

Use this only after all prior phase gates pass.

## Phase objective

Material-operation registry; operator effect declarations; proof profile/debt enforcement; terminal-state budget consumption.

## Backlog selection

Load `matrices/SUPER_PASS_BACKLOG_1020.csv` and select rows where `Suggested_Phase` contains `Phase 12` or whose category clearly belongs to this phase.

## Required work

1. Inspect relevant crates/files.
2. Implement fixes or explicit quarantines.
3. Add semantic/hostile tests that fail without the fix.
4. Run targeted tests, then broader command bar if feasible.
5. Update matrix statuses.
6. Write `PHASE_12_REPORT.md`.

## Exit gate

All material operations have contracts/effects/receipts; proof waiver cannot promote as proof.

## Completion rule

Do not move to the next phase with raw `open` rows in this phase. Use `fixed`, `quarantined`, `deferred`, `superseded`, or `open-blocking`.
