# Phase 09 — Full Validation and Release-Readiness Check

## Objective

Run all gates and classify failures.

## Required actions

1. Run fmt/check/test/clippy/doc.
2. Run schema/claim/final-state scripts.
3. Run `cargo package --allow-dirty --no-verify` only as dry-run inspection if appropriate.
4. Check crate-name availability manually with `cargo search poly-kv` / crates.io.
5. Record all outputs.

## Acceptance gate

All required gates pass or blockers are explicit.

## Phase-boundary report must include

- files inspected;
- files changed;
- commands run;
- tests/checks passed/failed/skipped;
- source-of-truth boundary status;
- unresolved blockers;
- rollback notes.
