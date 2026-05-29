# Final Hostile-Auditor Prompt

Act as a hostile auditor. Inspect the changed files, commands run, validation outputs, README claims, and receipt/schema tests.

Find:
- source-of-truth drift;
- duplicate codec/profile/shape types;
- hidden fallback;
- silent shape coercion;
- fake adapter compatibility;
- public overclaiming;
- missing exact fallback;
- missing tests;
- missing rollback;
- missing receipts;
- skipped checks without reason.

Output a defect matrix with severity, file path, evidence, root cause, fix direction, acceptance gate, and release-blocker status.
