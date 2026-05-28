# Phase 08 — Validation, fresh unzip, hostile handoff

## Goal

Produce final evidence.

## Tasks

1. Run required validation commands.
2. Generate fresh package if intended.
3. Verify package manifest parity.
4. Fresh-unzip and rerun checks.
5. Create final docs:
   - `docs/P32_COMPLETION_REPORT.md`
   - `docs/P32_COMMAND_RECEIPTS.md`
   - `docs/P32_CHANGED_FILES.md`
   - `docs/P32_UNRESOLVED_RISKS.md`
   - `docs/P32_HOSTILE_AUDITOR_HANDOFF.md`
   - `docs/P32_ROLLBACK_PLAN.md`
6. Do not omit failed/skipped checks. Record exact reason.

## Acceptance gate

- Final artifact set exists.
- Every required command has exit code and output summary.
- Hostile auditor handoff is complete.
