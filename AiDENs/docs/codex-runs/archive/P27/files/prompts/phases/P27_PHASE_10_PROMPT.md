# P27 Phase 10 Prompt — Patch engine hardening v0

## Scope

Execute only Phase 10: **Patch engine hardening v0**.

Before editing, restate:

- files you will inspect;
- issue IDs from `P27_MASTER_ISSUE_MATRIX.md` in scope;
- explicit no-go zones;
- commands you expect to run.

## Required law

- Preserve canonical sibling ownership.
- Emit receipts/logs under `target/p27/audit/`.
- Emit `handoffs/p27/PHASE_10_REPORT.md`.
- Apply 11A exact/approx/support/degradation disclosure where outputs are touched.
- Do not advance to the next phase without a stop/continue/quarantine decision.

## Phase-specific reminder

See `P27_PHASE_PLAN.md` Phase 10. Do not exceed that scope unless required to unblock this phase's acceptance gate.

## Report footer

End the phase report with exactly one of:

```text
Decision: continue
Decision: quarantine
Decision: stop
```
