# Repo Surface Repair Spec

## Problem

The root `README.md`, root `Makefile`, `docs/README.md`, and `scripts/check_repo_surface.sh` all described a finish-pack surface that did not actually exist in the shipped source snapshot.

## Repair performed by this pack

### Added root finish-pack docs

- `PACK_README.md`
- `MASTER_ISSUE_MATRIX.md`
- `SOURCE_BASIS.md`
- `SUPPORT_PROFILE.md`
- `IMPLEMENTATION_PLAYBOOK.md`
- `CONFORMANCE_GATES.md`
- `PHASED_EXECUTION_PLAN.md`
- `RISKS_AND_FORBIDDEN_SHORTCUTS.md`
- `STATUS_DASHBOARD.md`
- `STATUS_EVIDENCE_MANIFEST.json`
- `RELEASE_CHECKLIST.md`
- `CLAUDE_AUDIT_RECONCILIATION.md`
- `AGENTS.md`
- `PROMPT.md`

### Archived duplicate root surfaces

The machine-readable and stale snapshot siblings that were briefly restored at the root are now archived under `docs/archive/root_closeout_history/root_pack_duplicates_20260323/`.
They are historical support material, not active release authority.

### Added support docs

- `docs/DIGEST_MIGRATION_RUNBOOK.md`
- `docs/REPO_SURFACE_REPAIR_SPEC.md`
- `docs/TOOL_RUNTIME_INTEGRATION_PLAN.md`
- `docs/TEST_STRATEGY_AND_FIXTURE_PLAN.md`
- `docs/COMPATIBILITY_BURNDOWN_PLAN.md`

### Added archive pointer

- `ARCHIVE/legacy_front_door_and_control_plane.md`

### Added repo-local canonical spec copies

- `CANONICAL_STACK_SPEC_V6.md`
- `CANONICAL_STACK_SPEC_V7_RECURSIVE_INFERENCE_KERNEL.md`

These copies carry the current-state dependency notes required by the doc-truth gate.

## Still open after the repair

- committed `schemas/` artifacts are still missing (`SCHEMA-001` / `SCHEMA-003`);
- the top CEA beta-learning bug is still open (`CEA-001`);
- control-plane receipts remain a real design/code gap (`CONTROL-001`).
