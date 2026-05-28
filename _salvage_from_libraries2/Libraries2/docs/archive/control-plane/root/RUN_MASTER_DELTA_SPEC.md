# RUN_MASTER_DELTA_SPEC.md

> **Purpose:** Subordinate session-scoped conformance order for the next Claude Code pass.
>
> **Authority:** This document is **not** a peer normative spec. It is subordinate to:
>
> 1. `CANONICAL_STACK_SPEC_V4.md` — single source of truth for target architecture
> 2. `MIGRATION_NOTES_V4.md` — rollout order, compatibility law, deprecation timing
> 3. `TRACE_RETRY_CONTRACT.md` — canonical retry/trace law
> 4. `SPEC_PATCH_IMPLEMENTED.md` — already-adopted corrections that MUST NOT regress
> 5. `LATEST5.md` — code-only current-state snapshot
>
> **Goal of this run:** Finish the remaining **code-facing conformance work** revealed by `LATEST5.md` without inventing new architecture, removing migration-only compatibility surfaces early, or regressing already-closed corrections.

---

## 0. What this run is and is not

### This run IS
A **crate-by-crate conformance pass** focused on:
- core-layer completion where code still lags canon,
- compatibility-surface containment where legacy public shapes remain visible,
- supporting-crate propagation of shared ID / trace / retry semantics,
- proof obligations and current-state reporting.

### This run is NOT
- a redesign pass,
- a “clean up old code aggressively” pass,
- a permission slip to remove compatibility surfaces before the migration window closes,
- a permission slip to create new shared crates or move authority boundaries.

If a change would alter the architecture rather than complete the existing one, do not do it.

---

## 1. Canonical interpretation for this run

Use the following interpretation order when documents appear to disagree:

1. `CANONICAL_STACK_SPEC_V4.md`
2. `MIGRATION_NOTES_V4.md`
3. `TRACE_RETRY_CONTRACT.md`
4. `SPEC_PATCH_IMPLEMENTED.md`
5. `LATEST5.md`

`LATEST5.md` is a **generated code-only snapshot**, not normative law.  
If `LATEST5.md` appears to omit a private/internal surface, verify the source before changing code.

---

## 2. Workspace classification: three migration states

Treat the workspace as three different migration states, not one:

### A. Core migrated / partly visible
These crates already show the new spine and must be completed or validated:
- `stack-ids`
- `forge-memory-bridge`
- `semantic-memory`
- `knowledge-runtime`

### B. Compatibility-contained but still legacy-facing
These areas are allowed to remain temporarily, but only as **compatibility / migration-only** surfaces:
- `semantic-memory::projection_import::ImportEnvelope`
- `MemoryStore::import_envelope()`
- legacy trace conversion helpers
- legacy namespace-only partition assumptions
- any old Forge -> memory seam retained solely for compatibility

### C. Supporting pre-migration / propagation lag
These crates still visibly use older retry/trace/identity shapes and must be brought toward the shared model:
- `llm-pipeline`
- `agent-graph`
- `job-queue`
- `ai-batch-queue`
- `tauri-queue`
- any other supporting crate still using local `trace_id: String`, `attempt: u32`, local attempt trace wrappers, or string checkpoint identity where canon now expects shared types / semantics

All work in this run must name which class each change belongs to.

---

## 3. Hard preserve rules (MUST NOT regress)

Do not reopen or weaken the already-adopted fixes recorded in `SPEC_PATCH_IMPLEMENTED.md`.

The following are already considered resolved and MUST remain true after this run:

- canonical envelope naming law:
  - `ExportEnvelopeV1`
  - `ProjectionImportBatchV1`
  - `LegacyImportEnvelopeV1`
  - bare `ImportEnvelope` forbidden in new normative code/docs outside compat-labeled legacy surfaces
- non-public `semantic-memory` integration boundary for bridge batch ingestion
- `ClaimVersionId`
- relation-version parity work already adopted
- scoped alias review durability
- canonical namespace -> `ScopeKey` helper law
- canonical digest law
- audit-only evidence dereference discipline
- `danger-sm-write` remains **non-shippable this phase** unless all required governance is implemented

If you find code or docs that appear to conflict with these, fix the conflicting code/docs — do not soften the adopted correction.

---

## 4. Primary objectives for this run

## 4.1 `semantic-memory`: finish visibility and containment

### Problem
`LATEST5.md` proves:
- schema V11 exists,
- `projection_import_log` exists,
- but the visible public surface is still legacy-shaped (`ImportEnvelope`, `import_envelope()`),
- and `stack-ids` adoption is not visibly reflected in the snapshot.

### Required outcome
By the end of this run, `semantic-memory` must satisfy all of the following:

1. The crate visibly depends on `stack-ids` if that dependency truly exists in source.
2. The code path for `ProjectionImportBatchV1` ingestion exists and is clearly documented as the canonical new path.
3. The bridge ingestion boundary remains **non-public**.
4. Legacy `ImportEnvelope` / `import_envelope()` remain functional only as **compatibility / migration-only** surfaces.
5. Public docs/comments/examples must not present the legacy path as the normal path.
6. If `LATEST5.md` failed to surface the non-public boundary or dependency, the current-state generator/reporting must be improved so the next snapshot reflects it accurately.

### Forbidden outcome
- removing legacy import surfaces early,
- turning the bridge boundary public,
- leaving public docs that still imply `import_envelope()` is the normal path.

---

## 4.2 `stack-ids`: exact missing-type closure

### Problem
The canon expects `stack-ids` to own the shared ID / trace / scope primitives.  
`LATEST5.md` visibly shows only a subset.

### Required outcome
Produce an **exact inventory** with one line for each canon-listed shared primitive:

- `AttemptId`
- `TrialId`
- `ArtifactId`
- `EpisodeId`
- `ClaimId`
- `ClaimVersionId`
- `EntityId`
- `EnvelopeId`
- `ProjectionId`
- `ScopeKey`
- `TraceCtx`
- any other shared primitive already explicitly adopted in code (`RelationId`, `RelationVersionId`, `ImportBatchId`, `ContentDigest`, etc.)

For each item, classify it as exactly one of:
- **implemented in `stack-ids`**
- **intentionally deferred and still canonical debt**
- **missing and must be added now**
- **present elsewhere and must be migrated**
- **hidden from `LATEST5` but present in source**

### Required behavior
- Add missing canon-required primitives that are due now.
- Do not add business logic.
- Keep `stack-ids` primitive-only.
- If a type is intentionally deferred, say so explicitly in the output instead of silently pretending it is done.

---

## 4.3 `forge-memory-bridge`: resolve code vs patch-record ambiguity

### Problem
The patch record says richer bridge contract details were adopted, but `LATEST5.md` only proves a thinner visible bridge surface.

### Required outcome
Resolve this ambiguity explicitly:

**Case A — code already matches patch record**
- Do not redesign anything.
- Improve source docs and/or current-state reporting so the next code snapshot visibly reflects:
  - tighter evidence refs
  - richer version-local linkage
  - alias/review-related bridge-side contract pieces, if implemented
  - any internal structs intentionally omitted from `LATEST5`

**Case B — code still lags patch record**
- Close the missing code to the level already claimed by `SPEC_PATCH_IMPLEMENTED.md`.
- Do not extend beyond what the patch record already committed to.

### Required evidence
Your output must say which case was true.

---

## 4.4 Supporting crates: real propagation, not prose alignment

### Problem
Supporting crates still visibly use local/stringly retry-trace identity models.

### Required outcome
For each of the following crates:
- `llm-pipeline`
- `agent-graph`
- `job-queue`
- `ai-batch-queue`
- `tauri-queue`

produce a crate-specific delta covering:

1. current visible old shape(s):
   - local `trace_id: String`
   - `attempt: u32`
   - `attempt_count`
   - local attempt-trace wrappers
   - string checkpoint/job/item identifiers used where the shared model should apply
2. what is replaced now
3. what remains only as compatibility
4. which docs/examples/comments were updated
5. which tests/schemas/events/checkpoints were touched

### Required semantic rule
The implemented retry/trace behavior must match the canonical law:

- `ai-batch-queue` owns leaf-level batch retry only
- a logical retry family has one `AttemptId`
- concrete executions / retries within that family are `TrialId`s
- queue hops use links, not fake parent/child chains
- compatibility helpers may survive only where migration notes still permit them

Do not stop at documentation. This is a code-facing propagation pass.

---

## 4.5 Forge -> memory coupling: classify, do not hand-wave

### Problem
The remaining Forge -> `semantic-memory` relationship may now be compat-only, or it may still be unresolved entanglement.

### Required outcome
Classify the remaining dependency/seam explicitly as one of:

- **compat-only and acceptable this phase**
- **normal-path entanglement and must be reduced now**

If compat-only:
- phase-label it,
- say what exact removal condition applies,
- ensure docs/comments do not present it as the normal path.

If unresolved entanglement:
- reduce it now without violating migration law.

Do not write “reduce coupling” and move on. Name the exact surviving seam.

---

## 5. Compatibility law for this run

The following compatibility surfaces are allowed to remain **only** if they are visibly marked:

- `ImportEnvelope`
- `import_envelope()`
- legacy trace conversion helpers
- namespace-only assumptions used solely for migration
- any retained old Forge -> memory seam preserved for one-cycle compatibility

For every surviving compatibility surface, ensure:
1. it is labeled `compatibility / migration-only`,
2. it is not presented as the normal public path,
3. its removal condition is documented,
4. no new code chooses it as the default.

---

## 6. Do not overfit to `LATEST5`

`LATEST5.md` is the best current-state artifact, but it is still a generated summary.

If a supposedly missing surface is only absent from `LATEST5`:
1. inspect source,
2. determine whether the code actually has the surface,
3. if the code has it, improve the snapshot/reporting instead of rewriting good code just to satisfy the summary,
4. if the code lacks it, implement/fix it.

This rule applies especially to:
- non-public `semantic-memory` bridge ingestion boundaries,
- richer bridge internals,
- private/helper types that the snapshot may not currently surface.

---

## 7. Mechanical conformance checks (mandatory)

Run and report grep-style or equivalent checks for all of the following:

### Forbidden / legacy shape checks
- bare `ImportEnvelope` in new normative code/docs outside compat-labeled files
- new crate-local `TraceId`
- manual namespace -> scope conversion outside canonical `ScopeKey` helpers
- new `trace_id: String` in crates that are migrated this pass
- new `attempt: u32` / `attempt_count` shapes in crates that are migrated this pass
- new direct normal-path Forge -> memory bypasses
- compatibility surfaces missing phase labels

### Shared primitive checks
- exact inventory of shared ID / trace types in `stack-ids`
- search for surviving local equivalents of canon-owned primitives

### Retry / trace checks
- no stale examples/docs that still imply “new AttemptId per retry”
- no stale owner matrix entries contradicting canonical retry ownership
- no pad/truncate trace serialization behavior

Report results, not just that the checks were run.

---

## 8. Test-obligation matrix (mandatory)

Do not rely on raw test counts.

For this run, produce a named matrix that maps required obligations to concrete tests or explicitly marks them still missing. Include at minimum:

### Migration / import obligations
- legacy path still works during migration
- new bridge path works
- `import_log` preserved
- `projection_import_log` works
- dual-path coexistence
- restart/rollback behavior for migration-sensitive paths
- legacy queryability preserved

### Retry / trace obligations
- canonical `AttemptId` / `TrialId` semantics reflected in tests
- `ai-batch-queue` owner behavior
- queue hop trace linkage
- legacy trace interop helper behavior where still permitted

### Boundary obligations
- compat-only surfaces are labeled and not normal-path public guidance
- no normal-path Forge -> memory bypass
- runtime remains non-authoritative

If a required test does not yet exist, say so explicitly.

---

## 9. Required post-pass artifact

This pass is not complete until a fresh **code-only current-state snapshot** is generated.

Call it `LATEST6.md` (or equivalent) and ensure it is generated from direct source reads, not prior markdown.

The final output must compare the new snapshot against `LATEST5.md` and state, at minimum:

- what became visibly more canonical,
- what legacy surfaces remain and why,
- what shared ID / trace primitives are now present,
- whether bridge/reporting parity was resolved,
- which supporting crates still lag, if any.

---

## 10. File/work order for Claude

Use this order:

1. read `CANONICAL_STACK_SPEC_V4.md`
2. read `MIGRATION_NOTES_V4.md`
3. read `TRACE_RETRY_CONTRACT.md`
4. read `SPEC_PATCH_IMPLEMENTED.md`
5. read `LATEST5.md`
6. inspect source for the exact gaps named in this brief
7. patch core crates and supporting crates
8. patch docs/comments/examples/current-state generator as needed
9. run mechanical checks
10. build test-obligation matrix
11. generate fresh code-only snapshot
12. produce final structured report

Do not start by editing markdown in isolation.

---

## 11. Acceptance criteria for this run

This run is complete only if all of the following are true:

1. `semantic-memory` is clearly canonical-path capable while legacy import remains compat-only.
2. `stack-ids` has an exact, explicit shared primitive inventory, with missing vs deferred vs implemented status resolved.
3. `forge-memory-bridge` ambiguity versus the patch record is explicitly resolved.
4. supporting crates have real code-facing propagation of retry/trace/shared-ID semantics, not just prose alignment.
5. remaining Forge -> memory coupling is explicitly classified and phase-labeled.
6. required compatibility surfaces remain only as compat-only, not as default public guidance.
7. mechanical conformance checks were run and reported.
8. a named test-obligation matrix exists.
9. a fresh code-only snapshot was generated and compared.
10. no already-adopted correction from `SPEC_PATCH_IMPLEMENTED.md` regressed.

If any item is not complete, say so clearly instead of pretending this pass closed it.
