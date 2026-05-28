# Codex Prompt — P13 Multi-view runtime, retrieval disclosure, and query widening law

Read `AGENTS.md`, `SOURCE_BASIS.md`, `BUILD_ORDER_DAG.md`, and `passes/P13_MULTI_VIEW_RUNTIME_DISCLOSURE_AND_QUERY_POLICY.md`.

Implement P13 only. Do not start later passes.

## Goal

Expose semantic, temporal, entity, causal, control, and execution views without collapsing them into one shadow database.

## Primary crates

- `aidens-runner`
- `aidens-memory-kit`
- `aidens-contracts`
- `aidens-cli`
- `aidens-governance-kit`

## Required artifacts

- `RuntimeViewRequestV1`
- `ViewDisclosureReceiptV1`
- `QueryWideningReceiptV1`
- `RetrievalPolicyV1`
- `ProjectionDigestV1`
- `DegradationEventV1`

## Acceptance gates

- A time-scoped query cannot silently fall back to timeless retrieval.
- Alias expansion/widening emits QueryWideningReceiptV1.
- Projection rebuild from memory/evidence produces identical digest under same policy.

## Forbidden shortcuts

- Do not let runtime become a second durable truth store.
- Do not merge execution receipts into domain truth without explicit relation artifacts.

## Finish by producing a handoff

Include files changed, tests added, commands run, blockers, and next-pass readiness.
