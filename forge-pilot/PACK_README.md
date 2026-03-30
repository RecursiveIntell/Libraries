# forge-pilot finish pack — 2026-03-11

This pack is the implementation control plane for **`forge-pilot`**, a new crate that closes the loop between:

- `knowledge-runtime` advisories,
- public projection reads from `semantic-memory`,
- recursive-kernel compile / execute / oracle surfaces,
- real Forge experiments from `forge-engine`, and
- the canonical `ExportEnvelopeV3 -> transform_envelope_v3 -> import_projection_batch` lane.

It is a **supplement** to the current repo, not a replacement for the repo-wide v6/v7 stack specs.

## What the current code already gives you

The current archive is already ahead of the older “research only” state in four ways:

1. The **authority split is already real**: Forge raw truth, bridge transformation, projected truth, runtime orchestration, and advisory/kernel layers are already separated.
2. The repo already contains the **recursive-kernel lane**: `recursive-kernel-core`, `constraint-compiler`, `kernel-execution`, `kernel-oracles`, and `kernel-conformance`.
3. `knowledge-runtime` already publishes **non-authoritative advisory surfaces** (`latest_inference_advisory`, `latest_inference_explanation`, `latest_risk_gate`).
4. `semantic-memory` already exposes the **public projection query surfaces** needed for a loop crate (`query_claim_versions`, `query_relation_versions`, `query_episodes`, `query_entity_aliases`, `query_evidence_refs`, and import-log queries).

That means `forge-pilot` should be written as a **thin orchestrator / consumer crate**, not as a second kernel, second bridge, or second evidence schema.

## What this pack does

This pack freezes:

- the crate boundary law for `forge-pilot`,
- the target-state spec,
- the file change manifest,
- the issue matrix,
- the phase plan,
- the conformance gates,
- the implementation-agent law, and
- the Codex prompt.

## Read order

1. `../CANONICAL_STACK_SPEC_V6.md`
2. `../CANONICAL_STACK_SPEC_V7_RECURSIVE_INFERENCE_KERNEL.md`
3. `SOURCE_BASIS.md`
4. `FORGE_PILOT_MATRIX_SPEC.md`
5. `CRATE_BOUNDARY_MAP_FORGE_PILOT.md`
6. `MASTER_ISSUE_MATRIX_FORGE_PILOT.md`
7. `CHANGE_MANIFEST_FORGE_PILOT.md`
8. `CONFORMANCE_GATES_FORGE_PILOT.md`
9. `ACCEPTANCE_CHECKLIST_FORGE_PILOT.md`
10. `PHASED_EXECUTION_PLAN_FORGE_PILOT.md`
11. `RISKS_AND_FORBIDDEN_SHORTCUTS_FORGE_PILOT.md`
12. `FILE_AUDIT_INVENTORY_FORGE_PILOT.md`
13. `AGENTS.md`
14. `CODEX_IMPLEMENTATION_PROMPT.md`

## Included files

- `PACK_README.md`
- `SOURCE_BASIS.md`
- `FORGE_PILOT_MATRIX_SPEC.md`
- `MASTER_ISSUE_MATRIX_FORGE_PILOT.md`
- `MASTER_ISSUE_MATRIX_FORGE_PILOT.csv`
- `CRATE_BOUNDARY_MAP_FORGE_PILOT.md`
- `CHANGE_MANIFEST_FORGE_PILOT.md`
- `CONFORMANCE_GATES_FORGE_PILOT.md`
- `ACCEPTANCE_CHECKLIST_FORGE_PILOT.md`
- `PHASED_EXECUTION_PLAN_FORGE_PILOT.md`
- `RISKS_AND_FORBIDDEN_SHORTCUTS_FORGE_PILOT.md`
- `FILE_AUDIT_INVENTORY_FORGE_PILOT.md`
- `AGENTS.md`
- `CODEX_IMPLEMENTATION_PROMPT.md`

## Definition of done

`forge-pilot` is done only when all of the following are true:

- it consumes the existing v6/v7 authority lanes without reopening them;
- it can reconstruct a bounded observation context from public advisory + projection surfaces;
- it can turn that observation into deterministic, scored targets;
- it can execute both **kernel-oracle plans** and **paired patch plans**;
- it can export/import results only through the canonical V3 lane;
- it never writes authoritative truth directly;
- it degrades explicitly when export richness or kernel payloads are insufficient; and
- its end-to-end loop fixtures prove the above.
