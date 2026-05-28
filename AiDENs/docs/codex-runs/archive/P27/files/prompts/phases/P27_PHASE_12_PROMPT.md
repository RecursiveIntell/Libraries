# P27 Phase 12 Prompt — Memory-grounding durable seam

## Scope

Execute only Phase 12: **Memory-grounding durable seam**.

Before editing, restate:

- files you will inspect;
- issue IDs from `P27_MASTER_ISSUE_MATRIX.md` in scope;
- explicit no-go zones;
- commands you expect to run.

## Required law

- Preserve canonical sibling ownership.
- Emit receipts/logs under `target/p27/audit/`.
- Emit `handoffs/p27/PHASE_12_REPORT.md`.
- Apply 11A exact/approx/support/degradation disclosure where outputs are touched.
- Do not advance to the next phase without a stop/continue/quarantine decision.

## Phase-specific reminder

See `P27_PHASE_PLAN.md` Phase 12. Do not exceed that scope unless required to unblock this phase's acceptance gate.

## Report footer

End the phase report with exactly one of:

```text
Decision: continue
Decision: quarantine
Decision: stop
```
