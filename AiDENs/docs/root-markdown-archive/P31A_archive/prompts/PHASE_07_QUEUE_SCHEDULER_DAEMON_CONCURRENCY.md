# Codex Phase 07 Prompt — Queue, scheduler, daemon concurrency

Use this only after all prior phase gates pass.

## Phase objective

Lock/single-writer queue log; race-free idempotency/leasing/completion; safe-mode quarantine; queue-hop receipts.

## Backlog selection

Load `matrices/SUPER_PASS_BACKLOG_1020.csv` and select rows where `Suggested_Phase` contains `Phase 07` or whose category clearly belongs to this phase.

## Required work

1. Inspect relevant crates/files.
2. Implement fixes or explicit quarantines.
3. Add semantic/hostile tests that fail without the fix.
4. Run targeted tests, then broader command bar if feasible.
5. Update matrix statuses.
6. Write `PHASE_07_REPORT.md`.

## Exit gate

Concurrent enqueue/lease/complete tests pass; late completion after TTL rejected.

## Completion rule

Do not move to the next phase with raw `open` rows in this phase. Use `fixed`, `quarantined`, `deferred`, `superseded`, or `open-blocking`.
