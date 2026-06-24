# P25 Current-Run Classification Spec

## Goal

Prevent stale run material from being interpreted as active instructions.

## Required current-run files

```text
docs/codex-runs/CURRENT_RUN.md
docs/codex-runs/CODEX_RUN_INDEX.md
docs/codex-runs/CODEX_ARTIFACT_CLASSIFICATION.json
docs/codex-runs/ARCHIVAL_POLICY.md
```

## Current instruction rules

Only P25 instruction files may be classified as current instructions.

Prior-run files may be:
- previous-run-evidence,
- archive-evidence,
- source-basis,
- known-limitation evidence.

They must not be current run instructions.

Prior runs older than current must not be current run instructions.

## Stale-token failure examples

Fail if active current docs contain:
- `target/p##`
- `handoffs/p##`
- `docs/p##`
- `prior-run current`
- `prior-run current`
- stale prompt classified as `current-instruction`

## Acceptance

A machine check must confirm current-run docs and classification maps do not conflict.
