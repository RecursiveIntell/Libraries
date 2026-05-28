# RUN_CLAUDE_CODE_PROMPT.md

You are executing a **code-facing conformance pass**, not a redesign pass.

Read and obey these files in this exact priority order:

1. `CANONICAL_STACK_SPEC_V4.md`
2. `MIGRATION_NOTES_V4.md`
3. `TRACE_RETRY_CONTRACT.md`
4. `SPEC_PATCH_IMPLEMENTED.md`
5. `LATEST5.md`
6. `RUN_MASTER_DELTA_SPEC.md`
7. `RUN_SUPPORTING_CONFORMANCE_BRIEF.md`
8. `RUN_MECHANICAL_CHECKS.md`
9. `RUN_OUTPUT_TEMPLATE.md`

Your task is to complete the remaining **workspace conformance work** revealed by `LATEST5.md` and the subordinate delta, while preserving already-adopted fixes and one-cycle compatibility law.

## Non-negotiable rules

- Do **not** invent new architecture.
- Do **not** create new broad shared crates.
- Do **not** remove compatibility surfaces early.
- Do **not** weaken already-adopted corrections from `SPEC_PATCH_IMPLEMENTED.md`.
- Do **not** treat `LATEST5.md` as normative law; it is a generated source snapshot and may omit private/internal surfaces.
- Do **not** patch markdown only. This is a code-facing pass.
- Do **not** present legacy compatibility paths as the normal path.

## Required work classes

You must explicitly separate your work into these classes:

1. **Core-layer completion**
   - `semantic-memory`
   - `stack-ids`
   - `forge-memory-bridge`
   - `knowledge-runtime` only if directly needed by the current delta

2. **Compatibility-surface containment**
   - legacy `ImportEnvelope`
   - `import_envelope()`
   - legacy trace conversion helpers
   - namespace-only migration assumptions
   - any retained Forge -> memory seam that survives only for migration

3. **Supporting-crate propagation**
   - `llm-pipeline`
   - `agent-graph`
   - `job-queue`
   - `ai-batch-queue`
   - `tauri-queue`

## What you must close

### A. `semantic-memory`
Resolve the mismatch between:
- visible legacy public import surfaces in `LATEST5.md`
- and the canonical requirement for a non-public `ProjectionImportBatchV1` ingestion boundary.

Keep legacy path compatibility, but make the canonical path real, visible in the right way, and not publicly presented as legacy-normal.

### B. `stack-ids`
Build an exact inventory of canon-owned shared primitives.  
For each required type, classify it as:
- implemented,
- missing and added now,
- intentionally deferred,
- hidden from `LATEST5` but present,
- or still wrongly living elsewhere.

Add what is due now. Keep the crate primitive-only.

### C. `forge-memory-bridge`
Resolve the ambiguity between:
- what `SPEC_PATCH_IMPLEMENTED.md` says is already adopted,
- and what `LATEST5.md` visibly proves.

Either:
- expose / document / report the already-present richer contract correctly,
or
- implement the missing code that the patch record already claims.

State which case was true.

### D. Supporting crates
Propagate shared retry/trace semantics for real. Do not stop at wording.

For each supporting crate, identify:
- the old visible local/stringly shapes,
- what changed now,
- what remains compat-only,
- what docs/examples/comments changed,
- what tests/schema/event/checkpoint changes were needed.

### E. Forge -> memory seam
Classify the surviving seam explicitly:
- compat-only and acceptable this phase, or
- unresolved normal-path entanglement that must be reduced now.

Do not hand-wave with “reduce coupling.”

## Mandatory mechanical checks

You must run and report equivalent grep/search checks for:

- bare `ImportEnvelope` outside compat-labeled contexts
- local crate-owned `TraceId`
- ad-hoc namespace -> scope conversion outside canonical helpers
- stale `trace_id: String` / `attempt: u32` / `attempt_count` shapes in crates migrated this pass
- stale docs/examples implying new `AttemptId` per retry
- stale owner matrix entries contradicting canonical retry ownership
- missing phase labels on compatibility surfaces
- any new normal-path Forge -> memory bypass

## Mandatory proof outputs

You must produce all of the following:

1. code changes
2. doc/comment/example changes needed to match the code
3. mechanical-check results
4. named test-obligation matrix
5. fresh code-only snapshot (`LATEST6.md` or equivalent)
6. final report using `RUN_OUTPUT_TEMPLATE.md`

## Required attitude

Be brutally literal.  
Do not optimize for elegance.  
Optimize for **closing drift**.

If something is still incomplete after best effort, say so explicitly.
