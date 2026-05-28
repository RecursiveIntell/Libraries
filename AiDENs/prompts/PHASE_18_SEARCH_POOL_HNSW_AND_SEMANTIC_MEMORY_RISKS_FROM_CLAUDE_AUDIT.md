# Codex Phase 18 Prompt — Search, pool, HNSW, and semantic-memory risks from Claude audit

Use this only after all prior phase gates pass.

## Phase objective

Fix HNSW TOCTOU/atomic ordering/ID recycling posture; vector scan circuit breaker; timestamp parsing; pool timeout handling.

## Backlog selection

Load `matrices/SUPER_PASS_BACKLOG_1020.csv` and select rows where `Suggested_Phase` contains `Phase 18` or whose category clearly belongs to this phase.

## Required work

1. Inspect relevant crates/files.
2. Implement fixes or explicit quarantines.
3. Add semantic/hostile tests that fail without the fix.
4. Run targeted tests, then broader command bar if feasible.
5. Update matrix statuses.
6. Write `PHASE_18_REPORT.md`.

## Exit gate

HNSW concurrency tests; vector scan hard-block/degrade; parse warnings; pool error fixtures.

## Completion rule

Do not move to the next phase with raw `open` rows in this phase. Use `fixed`, `quarantined`, `deferred`, `superseded`, or `open-blocking`.
