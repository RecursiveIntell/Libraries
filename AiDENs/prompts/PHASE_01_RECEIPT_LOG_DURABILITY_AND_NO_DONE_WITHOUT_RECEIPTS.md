# Codex Phase 01 Prompt — Receipt/log durability and no done without receipts

Use this only after all prior phase gates pass.

## Phase objective

Make all material operations emit durable receipt artifacts before visible done state; add hash-chain verification, corruption quarantine, file locking/single-writer discipline.

## Backlog selection

Load `matrices/SUPER_PASS_BACKLOG_1020.csv` and select rows where `Suggested_Phase` contains `Phase 01` or whose category clearly belongs to this phase.

## Required work

1. Inspect relevant crates/files.
2. Implement fixes or explicit quarantines.
3. Add semantic/hostile tests that fail without the fix.
4. Run targeted tests, then broader command bar if feasible.
5. Update matrix statuses.
6. Write `PHASE_01_REPORT.md`.

## Exit gate

Tests prove final output cannot exist without durable receipt; concurrent append cannot fork chain; corrupt logs quarantine.

## Completion rule

Do not move to the next phase with raw `open` rows in this phase. Use `fixed`, `quarantined`, `deferred`, `superseded`, or `open-blocking`.
