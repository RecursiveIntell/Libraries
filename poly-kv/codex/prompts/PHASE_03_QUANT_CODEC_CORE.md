# Phase 03 — Implement `quant-codec-core`

## Objective

Implement shared trait/type boundary with tests.

## Required actions

1. Implement IDs/digests/dtypes/shapes/token spans.
2. Implement codec traits and eval report types.
3. Add validation and error types.
4. Add serde roundtrip and shape validation tests.
5. Run crate-specific fmt/check/test/clippy.

## Acceptance gate

`quant-codec-core` tests pass; `poly-kv` uses shared types instead of duplicating them.

## Phase-boundary report must include

- files inspected;
- files changed;
- commands run;
- tests/checks passed/failed/skipped;
- source-of-truth boundary status;
- unresolved blockers;
- rollback notes.
