# Codex Prompt — P06 Boundary compiler, schema validation, repair provenance, and canonical digests

Read `AGENTS.md`, `SOURCE_BASIS.md`, `BUILD_ORDER_DAG.md`, and `passes/P06_BOUNDARY_COMPILER_SCHEMA_AND_CANONICALIZATION.md`.

Implement P06 only. Do not start later passes.

## Goal

Upgrade JSON boundary handling from parse/repair helper to evidence-grade compiler front end.

## Primary crates

- `aidens-boundary-kit`
- `aidens-contracts`
- `aidens-cli`
- `aidens-testkit`

## Required artifacts

- `BoundaryCompileRequestV1`
- `BoundaryCompileOutcomeV1`
- `SchemaValidationReceiptV1`
- `JsonRepairReceiptV2`
- `CanonicalDigestV1`
- `DuplicateKeyFindingV1`

## Acceptance gates

- Duplicate-key fixture is rejected with DuplicateKeyFindingV1.
- Schema-invalid tool input is blocked before invocation and emits SchemaValidationReceiptV1.
- Canonical digest is cryptographic, deterministic, and stable across whitespace/key-order changes.
- Repair cannot change treatment-critical fields without a treatment-integrity warning or hard fail.

## Forbidden shortcuts

- Do not use serde_json parse alone as the whole boundary law.
- Do not silently extract substrings from model output without degraded repair receipt.

## Finish by producing a handoff

Include files changed, tests added, commands run, blockers, and next-pass readiness.
