# Codex Prompt — P19 Final integration, release bar, and completion audit

Read `AGENTS.md`, `SOURCE_BASIS.md`, `BUILD_ORDER_DAG.md`, and `passes/P19_FINAL_INTEGRATION_RELEASE_BAR_AND_COMPLETION_AUDIT.md`.

Implement P19 only. Do not start later passes.

## Goal

Run cross-pass integration, package the release, and generate a falsifiable completion report that says exactly what is done, partial, deferred, and blocked.

## Primary crates

- `workspace root`
- `all crates`
- `docs`
- `CI`

## Required artifacts

- `CompletionAuditReportV1`
- `ReleaseArtifactManifestV1`
- `CrossPassTraceabilityMatrixV1`
- `KnownLimitationsRegisterV1`
- `RegressionDebtLedgerV1`

## Acceptance gates

- cargo fmt/check/test/clippy pass for workspace with all features.
- All pass acceptance gates are satisfied or explicitly waived by governance artifact.
- Completion report contains no unsubstantiated “healthy” claim for incomplete advanced surfaces.

## Forbidden shortcuts

- Do not ship without a current limitations register.
- Do not mark horizon features complete to close the plan.

## Finish by producing a handoff

Include files changed, tests added, commands run, blockers, and next-pass readiness.
