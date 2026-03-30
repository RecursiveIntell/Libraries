# AGENTS.md — forge-pilot implementation law

This file is for Codex, Claude Code, Cursor, Zed, or a human implementing `forge-pilot`.

## Read order before touching code

1. `../CANONICAL_STACK_SPEC_V6.md`
2. `../CANONICAL_STACK_SPEC_V7_RECURSIVE_INFERENCE_KERNEL.md`
3. `PACK_README.md`
4. `SOURCE_BASIS.md`
5. `FORGE_PILOT_MATRIX_SPEC.md`
6. `CRATE_BOUNDARY_MAP_FORGE_PILOT.md`
7. `MASTER_ISSUE_MATRIX_FORGE_PILOT.md`
8. `CHANGE_MANIFEST_FORGE_PILOT.md`
9. `CONFORMANCE_GATES_FORGE_PILOT.md`
10. `PHASED_EXECUTION_PLAN_FORGE_PILOT.md`
11. `RISKS_AND_FORBIDDEN_SHORTCUTS_FORGE_PILOT.md`
12. `FILE_AUDIT_INVENTORY_FORGE_PILOT.md`
13. `ACCEPTANCE_CHECKLIST_FORGE_PILOT.md`
14. `CODEX_IMPLEMENTATION_PROMPT.md`

## Prime directive

Land `forge-pilot` **without reopening the core architecture**.

## Non-negotiable law

1. Consume existing authority lanes.
2. No new evidence schema.
3. No bridge invention.
4. No runtime shadow truth.
5. Kernel-oracle reuse beats kernel reimplementation.
6. Explicit degradation beats confident fiction.
7. LLMs may refine text, not authority.
8. Canonical V3 export/import only.

## Workstreams

### Surface agent
Owns crate manifest, workspace wiring, Makefile target, truthful docs.

### Observe/orient agent
Owns observation reconstruction, target extraction, scoring, and exhaustion.

### Act/export agent
Owns oracle plans, paired patch plans, local bundle building, and canonical roundtrip.

### Loop agent
Owns history, budgets, cooldown, halt reasons, reports, and CLI.

### Release agent
Owns conformance gates, acceptance checklist, and final package truth.

## Stop conditions

Stop immediately if:

- you start defining a second evidence schema,
- you start writing directly into projected truth,
- you start cloning kernel compile/execute/oracle code into pilot,
- you hide thin-export or missing-kernel-payload degradation,
- or you let LLM output choose targets or mutate authority state.

## Definition of done

You are done only when:

- the crate builds with default features,
- both oracle and patch action families are real,
- canonical export/import roundtrip is real,
- the loop halts honestly under budgets,
- and the docs tell the truth about what actually shipped.
