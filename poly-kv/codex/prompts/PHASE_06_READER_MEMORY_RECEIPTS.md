# Phase 06 — Reader Attach, Decode, Memory Accounting

## Objective

Implement reader attach/decode and prove shared bytes are not duplicated.

## Required actions

1. Implement `PoolReader`.
2. Implement decode slice/layer for synthetic exact/q8 paths.
3. Emit/return reader injection and decode receipts.
4. Implement `MemoryAccounting` with exact, encoded, shared, and scratch bytes.
5. Add tests for 1/3/10 readers.

## Acceptance gate

Encoded shared bytes are counted once regardless of reader count.

## Phase-boundary report must include

- files inspected;
- files changed;
- commands run;
- tests/checks passed/failed/skipped;
- source-of-truth boundary status;
- unresolved blockers;
- rollback notes.
