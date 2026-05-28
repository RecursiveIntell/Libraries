# Claude Code Next-Phase Bundle

This bundle is a **self-contained implementation package** for the next development phase of the user's Rust AI/knowledge stack.

It is designed for a coding model that **may not have access to the rest of the project documents**.

## Included files

1. `00_START_HERE.md` — this file
2. `01_PROJECT_CONTEXT_BRIEF.md` — compact but sufficient architecture/context brief
3. `02_PHASE_SPEC.md` — implementation-grade phase specification
4. `03_TASK_PLAN.md` — ordered task breakdown with acceptance criteria
5. `04_CLAUDE_IMPLEMENTATION_PROMPT.md` — direct prompt for Claude Code to execute the phase
6. `05_CLAUDE_HOSTILE_REVIEW_PROMPT.md` — adversarial audit prompt for second-pass review
7. `06_ACCEPTANCE_CHECKLIST.md` — concise done criteria to verify phase completion

## Intended usage

Use the files in this order:

1. Read `01_PROJECT_CONTEXT_BRIEF.md`
2. Read `02_PHASE_SPEC.md`
3. Execute according to `03_TASK_PLAN.md`
4. Use `04_CLAUDE_IMPLEMENTATION_PROMPT.md` as the main coding prompt
5. After implementation, run `05_CLAUDE_HOSTILE_REVIEW_PROMPT.md`
6. Validate against `06_ACCEPTANCE_CHECKLIST.md`

## Objective of this phase

This phase is **not** about inventing more architecture.

It is about making the architecture already described by the project **true in code** through:

- hard crate boundaries,
- canonical identity and lineage rules,
- importer/projection discipline,
- runtime query semantics,
- observability and trace propagation,
- explicit failure behavior,
- tests that prove the invariants.

## Non-goal

Do **not** let the coding model treat this as an invitation to redesign the whole stack. The purpose is to:

- enforce,
- connect,
- finish missing seams,
- and prevent architectural drift.

