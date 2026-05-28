# Production schema and example backfill plan — 2026-03-18

The current snapshot still lacks several review/adjudication schemas and many example JSON files.
This document freezes the end-state publication set that Codex must complete.

## New schema JSON required

These are missing entirely in the current snapshot and must be registered in `contract-schema-gen`:

- `schemas/effect-review-case-v1.schema.json`
- `schemas/effect-block-receipt-v1.schema.json`
- `schemas/delegation-review-case-v1.schema.json`
- `schemas/release-gate-case-v1.schema.json`
- `schemas/continuity-review-case-v1.schema.json`
- `schemas/effect-adjudication-receipt-v1.schema.json`
- `schemas/release-rollback-decision-v1.schema.json`

## Existing schemas that need updated fields and refreshed examples

- `effect-intent-v1`
- `effect-preflight-report-v1`
- `effect-commit-decision-v1`
- `effect-execution-receipt-v1`
- `effect-observation-bundle-v1`
- `compensation-plan-v1`
- `compensation-execution-receipt-v1`
- `control-receipt-v1`
- `policy-decision-v1`
- `promotion-decision-v1`
- `refutation-decision-v1`
- `rollback-plan-v1`
- `remote-slice-request-v1`
- `remote-slice-result-v1`
- `cross-runtime-replay-ticket-v1`
- `settlement-case-v1`
- `settlement-receipt-v1`
- `shared-replay-slice-v1`
- `shared-divergence-report-v1`
- `shared-view-downgrade-v1`
- `local-dissent-record-v1`

## Example JSON that must exist in the end state

Every one of the following should exist and parse:

- `examples/control-receipt-v1.example.json`
- `examples/effect-review-case-v1.example.json`
- `examples/effect-block-receipt-v1.example.json`
- `examples/delegation-review-case-v1.example.json`
- `examples/release-gate-case-v1.example.json`
- `examples/continuity-review-case-v1.example.json`
- `examples/policy-decision-v1.example.json`
- `examples/promotion-decision-v1.example.json`
- `examples/refutation-decision-v1.example.json`
- `examples/rollback-plan-v1.example.json`
- `examples/effect-adjudication-receipt-v1.example.json`
- `examples/release-rollback-decision-v1.example.json`
- `examples/remote-slice-request-v1.example.json`
- `examples/remote-slice-result-v1.example.json`
- `examples/cross-runtime-replay-ticket-v1.example.json`
- `examples/settlement-case-v1.example.json`
- `examples/settlement-receipt-v1.example.json`
- `examples/shared-replay-slice-v1.example.json`
- `examples/shared-divergence-report-v1.example.json`
- `examples/shared-view-downgrade-v1.example.json`
- `examples/local-dissent-record-v1.example.json`

## Publication rule

Do not hand-author final schema JSON after source fields change if `contract-schema-gen` can generate them.
The only acceptable flow is: update Rust types -> regenerate schemas -> backfill examples -> run the final closure gate.
