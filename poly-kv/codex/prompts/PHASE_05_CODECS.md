# Phase 05 — Raw Exact, Q8 Keys, Value Boundary

## Objective

Implement exact fallback and q8 key reference path.

## Required actions

1. Implement `RawExactCodec` / exact fallback block storage.
2. Implement `Q8KeyCodec` with documented scheme.
3. Implement `ValueCodec` trait and raw exact value codec.
4. Add optional adapter modules behind feature flags only.
5. Test no NaN/inf, bounded MSE, and malformed inputs.

## Acceptance gate

Exact fallback passes and q8 tests document drift.

## Phase-boundary report must include

- files inspected;
- files changed;
- commands run;
- tests/checks passed/failed/skipped;
- source-of-truth boundary status;
- unresolved blockers;
- rollback notes.
