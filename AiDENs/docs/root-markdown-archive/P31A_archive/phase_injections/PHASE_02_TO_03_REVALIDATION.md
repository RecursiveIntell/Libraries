# Manual Guardrail Injection — Phase 02 to 03

**STOP. Do not proceed until this operator injection is pasted by the operator. WAIT for the operator to paste this injection.**

Before starting Phase 03, revalidate and report:

1. No source-of-truth ownership violation was introduced.
2. AiDENs did not create a local canonical substitute for sibling-owned semantics.
3. `z.py`/package changes did not exclude dependencies required by included verifiers.
4. Active stale Codex artifacts are either archived or classified.
5. Current-run logic is generic, not hard-coded to the current phase/run.
6. All phase outputs have evidence under `target/p25/audit/` and `handoffs/p25/`.
7. Capability lane did not regress behind packaging lane.
8. Any failure must stop progression or enter repair/quarantine.

If any invariant fails, halt Phase 03 and repair before continuing.
