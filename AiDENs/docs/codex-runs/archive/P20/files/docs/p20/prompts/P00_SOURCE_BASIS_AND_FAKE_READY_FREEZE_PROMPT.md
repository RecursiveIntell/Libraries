# Codex Prompt — P00 Source-basis lock, fake-ready freeze, and repo hygiene gate

Read `AGENTS.md`, `SOURCE_BASIS.md`, `BUILD_ORDER_DAG.md`, and `passes/P00_SOURCE_BASIS_AND_FAKE_READY_FREEZE.md`.

Implement P00 only. Do not start later passes.

## Goal

Freeze the source basis and make it impossible to claim readiness while scaffold-only or fake-ready surfaces remain hidden.

## Primary crates

- `workspace root`
- `scripts`
- `docs`
- `aidens-testkit`

## Required artifacts

- `SourceBasisLockV1`
- `ScaffoldSurfaceReportV1`
- `FakeReadyFindingV1`
- `SuperPassStatusV1`

## Acceptance gates

- bash scripts/verify.sh exists and is referenced by README, AGENTS.md, and CI.
- grep for stale 20260425 metrics either returns none or only explicit historical references.
- Scaffold-only crates are listed in status and doctor as deferred/blocked, never healthy.
- assert_no_fake_completion.sh and assert_no_scaffold_promoted.sh both pass.

## Forbidden shortcuts

- Do not delete scaffold crates to make metrics look cleaner.
- Do not rename scaffolds “experimental” while exposing them as usable.

## Finish by producing a handoff

Include files changed, tests added, commands run, blockers, and next-pass readiness.
