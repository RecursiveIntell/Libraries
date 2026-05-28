# Codex Prompt — P14 Release-grade product surface, operator UX, and status truth

Read `AGENTS.md`, `SOURCE_BASIS.md`, `BUILD_ORDER_DAG.md`, and `passes/P14_RELEASE_PRODUCT_SURFACE_AND_OPERATOR_UX.md`.

Implement P14 only. Do not start later passes.

## Goal

Make AiDENs usable as a product/platform: clear commands, diagnostics, docs, packaging, examples, and release bar.

## Primary crates

- `aidens-cli`
- `aidens-app-kit`
- `aidens`
- `docs`
- `examples`
- `CI`

## Required artifacts

- `ReleaseReadinessReportV1`
- `OperatorStatusReportV1`
- `ExampleAppManifestV1`
- `InstallSmokeReceiptV1`

## Acceptance gates

- A new user can create an app, run provider-check, inspect tools, run mock turn, inspect receipts, and run verify.sh.
- Release readiness report blocks if any public docs claim a scaffolded crate is complete.
- CI exercises examples as compile/test fixtures.

## Forbidden shortcuts

- Do not document horizon features as available.
- Do not hide degraded modes in friendly prose.

## Finish by producing a handoff

Include files changed, tests added, commands run, blockers, and next-pass readiness.
