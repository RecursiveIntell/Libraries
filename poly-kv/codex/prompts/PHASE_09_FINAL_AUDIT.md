# Phase 09 — Hostile auditor handoff

Produce:

- `.codex-runs/$RUN_ID/changed_files.txt`
- `.codex-runs/$RUN_ID/commands_run.log`
- `.codex-runs/$RUN_ID/validation_results.md`
- `.codex-runs/$RUN_ID/invariant_report.md`
- `.codex-runs/$RUN_ID/risk_register.md`
- `.codex-runs/$RUN_ID/rollback_plan.md`
- `.codex-runs/$RUN_ID/final_audit_report.md`
- `.codex-runs/$RUN_ID/remaining_delta.md`

Final report must include:

- changed files;
- commands run;
- pass/fail/skip results;
- source-of-truth decisions;
- adapters inspected or deferred;
- receipt/accounting changes;
- Python sidecar status;
- benchmark receipts generated;
- unresolved risks;
- rollback instructions;
- exact next pass.

Gate: no completion claim without final handoff.
