# Codex Prompt — P08 Reference interpreters and semantic conformance harness

Read `AGENTS.md`, `SOURCE_BASIS.md`, `BUILD_ORDER_DAG.md`, and `passes/P08_REFERENCE_INTERPRETERS_AND_SEMANTIC_CONFORMANCE.md`.

Implement P08 only. Do not start later passes.

## Goal

Add small independent interpreters that define expected semantics for plans, capabilities, receipts, permits, boundary repair, and later temporal queries.

## Primary crates

- `aidens-testkit`
- `aidens-config`
- `aidens-tool-kit`
- `aidens-permit-kit`
- `aidens-boundary-kit`
- `aidens-receipts`

## Required artifacts

- `ReferenceCaseV1`
- `ReferenceInterpreterReportV1`
- `DifferentialConformanceFindingV1`
- `GoldenFixtureManifestV1`

## Acceptance gates

- Reference fixtures cover all provider kinds, risk classes, memory modes, receipt levels, and tool lifecycle states.
- Any production/reference mismatch fails tests with human-readable diff.
- The testkit is dependency-light and does not call production internals in a circular way.

## Forbidden shortcuts

- Do not make the reference interpreter a thin wrapper around production code.
- Do not bless implementation behavior by copying its bugs into fixtures.

## Finish by producing a handoff

Include files changed, tests added, commands run, blockers, and next-pass readiness.
