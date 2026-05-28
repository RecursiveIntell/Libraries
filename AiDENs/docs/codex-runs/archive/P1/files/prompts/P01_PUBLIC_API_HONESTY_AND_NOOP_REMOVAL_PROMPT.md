# Codex Prompt — P01 Public API honesty, no-op removal, and plan/runtime parity

Read `AGENTS.md`, `SOURCE_BASIS.md`, `BUILD_ORDER_DAG.md`, and `passes/P01_PUBLIC_API_HONESTY_AND_NOOP_REMOVAL.md`.

Implement P01 only. Do not start later passes.

## Goal

Remove or implement API methods that accept inputs without honoring them; align app plans, builder inputs, and runtime configuration.

## Primary crates

- `aidens-app-kit`
- `aidens-runner`
- `aidens-cli`
- `aidens-contracts`

## Required artifacts

- `ApiHonestyReceiptV1`
- `PlanRuntimeParityReportV1`
- `ConfigApplyReceiptV1`

## Acceptance gates

- No public method accepts meaningful input and silently discards it.
- from_plan cannot produce a disabled-provider runner when provider_required=true unless the plan explicitly selects disabled and returns a blocked run.
- plan compile output and doctor report agree on provider route, tool exposure, memory mode, and scaffold state.

## Forbidden shortcuts

- Do not fix no-ops by hiding methods behind doc comments only.
- Do not treat disabled provider as a valid provider when provider_required=true.

## Finish by producing a handoff

Include files changed, tests added, commands run, blockers, and next-pass readiness.
