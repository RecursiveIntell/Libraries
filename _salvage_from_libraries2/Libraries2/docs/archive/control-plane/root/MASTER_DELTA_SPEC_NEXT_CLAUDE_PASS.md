# Master Delta Spec — Next Claude Code Pass

**Purpose:** convert the stack from *architecturally corrected but partially migrated* into a **canonically wired, contract-complete, regression-resistant** target-state implementation.

**Audience:** Claude Code operating directly on the workspace.

**Important:** this document is intentionally self-contained. It restates the rules Claude must follow so the pass does **not** depend on hidden context.

---

## 0. Pass thesis

This pass is **not** a fresh architecture-design pass.

This pass is a **target-state closure pass**.

The architecture is already settled:

- Forge owns **raw verification truth**.
- Memory owns **queryable projected truth**.
- Bridge owns **projection transformation only**.
- Runtime owns **query planning, routing, merge, and degradation reporting only**.
- `stack-ids` owns **canonical cross-crate IDs, digests, and trace primitives**.
- Compatibility shims are tolerated only as migration residue; they are **not normative interfaces**.

Any implementation choice that reopens those questions is out of scope unless an explicit superseding spec is written.

---

## 1. Frozen doctrine Claude MUST treat as law

### 1.1 Authority boundaries

Claude MUST preserve these boundaries:

- `semantic-memory` is authoritative for queryable knowledge state and imported projections.
- `semantic-memory-forge` is authoritative for raw verification truth and export envelopes.
- `forge-memory-bridge` is authoritative only for transformation from Forge export into typed import batches.
- `knowledge-runtime` is authoritative only for query planning, retrieval composition, merge, explainability, degradation reporting, and projection lifecycle interpretation.

### 1.2 Forbidden reopenings

Claude MUST NOT casually reopen any of the following:

- whether memory should own Forge-domain transformation logic,
- whether runtime should persist authoritative knowledge state,
- whether the bridge should fold into runtime,
- whether raw receipts should be silently joined into ranking paths,
- whether shared IDs should be redefined crate-by-crate,
- whether compatibility APIs should define the long-term architecture.

### 1.3 Primitive law

Canonical cross-crate primitives come from `stack-ids`.

Claude MUST NOT introduce:

- crate-local copies of canonical ID types,
- crate-local trace wrappers that become de facto public contracts,
- business-domain enums leaking out of storage-local rows,
- new compatibility wrappers that become normative by inertia.

### 1.4 Canonical normal path

All new producers, transformers, and storage work MUST target:

```text
Forge raw truth
  -> ExportEnvelopeV1
  -> forge-memory-bridge
  -> ProjectionImportBatchV1
  -> semantic-memory importer transaction
  -> queryable projections
```

Compatibility shims may exist temporarily, but they are not normative.

### 1.5 Runtime law

`knowledge-runtime` is a bounded orchestration and merge layer over memory-visible truth only.

Claude MUST NOT let runtime:

- persist authoritative source truth,
- become a shadow database,
- silently mix memory projections with fresher raw Forge receipts,
- mutate authoritative identity state during query execution.

### 1.6 Direct-write law

Direct memory write-through is **not** the normal path.

Any emergency direct-write facility must remain exceptional, auditable, disabled by default in release builds, and must not inherit normal comparability/import-lineage assumptions.

---

## 2. Current-state facts this pass must start from

These are not guesses. Treat them as current-state constraints.

### 2.1 Compatibility surfaces still active

The stack still has active compat surfaces, including:

- `semantic-memory::projection_import::ImportEnvelope`,
- `MemoryStore::import_envelope()`,
- compat `TraceId` forms in `semantic-memory` and `llm-pipeline`,
- compat trace fields in `agent-graph` and `job-queue`,
- numeric attempt counters that still need canonical retry identity replacement,
- `forge-memory-bridge::legacy::LegacyImportEnvelopeV1` as a compat seam.

### 2.2 Bridge is already close to target-state

`forge-memory-bridge` is already classified as conforming enough that the remaining Forge→memory seam is compat-only.

Do **not** destabilize it with unnecessary redesign.

### 2.3 Runtime is still lagging materially

Runtime still lags on:

- true temporal execution,
- full scope enforcement,
- projection rebuild execution,
- Forge causal projection wiring through the canonical memory-visible path,
- fuzzy entity resolution,
- and it still carries in-memory identity/lifecycle machinery that can drift toward shadow-authority behavior if handled carelessly.

### 2.4 The current batch shape is not automatically semantically complete

`ProjectionImportBatchV1` exists, but Claude MUST validate its row semantics against the full logical contract.

Do **not** assume that the presence of a type means all canonical fields and invariants are already preserved.

### 2.5 Evidence-bundle direction is correct, but premature learned scoring is not

The next pass should strengthen:

- evidence bundles,
- trace alignment,
- retry/replay law,
- derivation/invalidation,
- bounded recomputation,
- and refutation structure.

Claude MUST NOT prioritize learned graph scoring before verified-edge quality, identity stability, and observability invariants are in place.

---

## 3. Pass goals

This pass MUST accomplish the following.

1. Finish canonical primitive adoption in normal-path code.
2. Finish canonical import-path migration for new work.
3. Close semantic-memory logical importer/storage gaps.
4. Add derivation/invalidation/bounded recomputation substrate required by the architecture.
5. Close runtime target-state gaps without turning runtime into an authority.
6. Land the minimal Forge evidence-bundle substrate needed for real verified-edge semantics.
7. Finish executable trace/retry/replay law across boundaries.
8. Add release-blocking ugly-case tests and conformance checks.
9. Tighten docs/comments so compat shims are never described as normative.

---

## 4. Required execution order

Claude MUST execute the work in this order unless a dependency forces a narrower rearrangement.

### Phase 0 — Inventory and protection pass

Before major edits, Claude MUST:

- identify all live compat surfaces still used in normal-path code,
- identify all public APIs that are already canonical and should not be destabilized,
- identify all current tests that protect existing good behavior,
- mark any intended deprecation/removal points in code comments only where appropriate.

**Do not** start with a blind refactor.

### Phase 1 — Primitive burn-down and public contract cleanup

Required outcomes:

- normal-path APIs use `TraceCtx` instead of crate-local trace types or raw strings,
- retry lineage uses `AttemptId` / `TrialId` semantics instead of ambiguous numeric counters,
- compat helpers remain only where they still shield existing callers.

Required crates:

- `llm-pipeline`
- `agent-graph`
- `job-queue`
- `ai-batch-queue`
- `tauri-queue`
- `semantic-memory` (remaining legacy trace forms)

Forbidden in this phase:

- adding new wrapper types just to ease migration,
- deleting compat shims before callers are migrated,
- moving business logic into `stack-ids`.

### Phase 2 — Canonical import-path closure

Required outcomes:

- all new normal-path work targets `ExportEnvelopeV1 -> forge-memory-bridge -> ProjectionImportBatchV1 -> import_projection_batch()`;
- export/import version semantics are explicit and distinct;
- compat seam remains phase-labeled until callers are gone;
- docs/comments stop describing legacy shapes as desired architecture.

Required checks:

- `LegacyImportEnvelopeV1` remains compat-only,
- `ImportEnvelope` remains compat-only or becomes unused,
- `import_envelope()` remains shim-only or becomes unused,
- no new code path bypasses the bridge in the normal path.

### Phase 3 — `semantic-memory` logical importer and storage closure

Claude MUST make sure memory can represent, query, and freshness-track at least:

- claim projection versions,
- relation versions,
- aliases and merge/split history,
- evidence references,
- episode links,
- contradiction / supersession state,
- import log / import status,
- derivation edges for memory-local derived artifacts.

Claude MUST verify that the importer boundary logically supports:

- `begin_projection_import(meta)`
- `upsert_claim_projections(rows, tx)`
- `upsert_relation_versions(rows, tx)`
- `upsert_entity_aliases(rows, tx)`
- `upsert_evidence_refs(rows, tx)`
- `upsert_episode_links(rows, tx)`
- `commit_projection_import(tx)`
- `abort_projection_import(tx)`

Exact method names may differ. Logical responsibilities may not.

Claude MUST also verify or add preservation of required fields for:

- claim projection versions,
- relation versions,
- aliases / identity state,
- evidence refs.

**Critical instruction:** if a required field is stored at batch-level or normalized side-table rather than row-level, Claude MUST make that preservation explicit, queryable, and testable. Silent “it’s probably implied” does not count.

### Phase 4 — Derivation, invalidation, and bounded recomputation substrate

This phase is mandatory.

Claude MUST add or complete representation for at least these derivation edges:

- raw receipt -> evidence bundle,
- evidence bundle -> claim projection,
- episode -> alias candidate or relation candidate,
- relation version -> graph materialization or repairable index rows,
- source projection -> future runtime cache/overlay artifacts.

Claude MUST ensure:

- derivation edges are append-only or append-plus-supersession,
- every derived artifact declares an invalidation mode,
- invalidation triggers include contradiction import, refutation result, alias split/unmerge, envelope supersession, estimator version change, and policy-profile change,
- recomputation is bounded by lineage, entity neighborhood, scope, time window, or explicit operator intent,
- no correction silently triggers a blind global rebuild unless explicitly requested.

### Phase 5 — Runtime target-state closure

Claude MUST close the runtime gaps without violating runtime’s bounded role.

Required runtime outcomes:

- logical routing across semantic / entity / temporal / mixed modes,
- strongest-available scope enforcement at planning time,
- explicit degradation warnings where only partial scope pushdown exists,
- true temporal execution semantics or explicit phase-accurate degradation,
- query-time entity resolution over canonical memory identity state,
- deterministic provenance-preserving merge,
- projection lifecycle interpretation stronger than tracker-only freshness timestamps,
- rebuild orchestration hooks,
- Forge causal projection consumption only through memory-visible imported projections.

Forbidden in this phase:

- making runtime authoritative for identity or projection truth,
- persisting authoritative claim/entity/relation state inside runtime,
- silently joining raw Forge receipts into answers or ranking,
- pretending hybrid fallback equals true temporal support.

### Phase 6 — Minimal Forge evidence-bundle substrate

Claude MUST land a minimal but real evidence-bundle substrate.

Every causal/effect bundle should be able to preserve at least:

- causal question,
- unit definition,
- treatment specification,
- outcome specification,
- covariates/confounders recorded,
- identification rationale,
- estimator and estimate,
- refutations attempted and results,
- raw receipt / trace / replay handles.

Claude MUST connect bundle evolution to:

- trace alignment,
- retry lineage,
- derivation edges,
- invalidation semantics,
- and importable memory-visible projections where appropriate.

Forbidden in this phase:

- treating evidence bundles as decorative metadata,
- introducing learned graph scoring as a primary deliverable,
- hiding estimator/refuter metadata behind opaque blobs.

### Phase 7 — Trace, retry, replay, and queue-hop completion

Claude MUST enforce the following everywhere relevant:

- `TraceCtx` is the canonical in-process trace form,
- any boundary that spawns work, enqueues work, calls tools, imports/exports data, or emits durable records propagates canonical trace context,
- verification-related logs/events include `attempt_id`, `trial_id`, `claim_id`, `patch_hash`, and `baseline_or_patch` when available,
- retry ownership is singular and explicit,
- a logical retry family has exactly one `attempt_id`, exactly one primary owner, one or more `trial_id`s, and explicit lineage links,
- replay gets a new root trace plus linkage to the original,
- queue and replay hops use trace links or equivalent non-fake lineage semantics.

Retry-owner matrix Claude MUST preserve:

- transport retry -> `llm-pipeline`
- parse/validator correction retry -> `llm-pipeline`
- graph node re-execution -> `agent-graph`
- durable import/export/rebuild retry -> `job-queue`
- AI batch item retry -> `ai-batch-queue`
- verification backend execution retry -> Forge orchestration layer
- envelope import transaction retry -> outer owner only; no hidden inner owner

### Phase 8 — Hardening, tests, docs, and clean finish

Claude MUST finish with:

- release-blocking ugly-case tests,
- fuzz/property tests where high ROI already exists,
- doc/comment cleanup so compat surfaces are never described as target architecture,
- an honest final report of done / partial / blocked items.

---

## 5. Crate-by-crate required outcomes

### 5.1 `stack-ids`

Must remain narrow:

- opaque newtypes,
- parsing / validation,
- serialization,
- W3C trace helpers,
- bounded baggage helpers,
- digest helpers,
- zero business logic.

Required work:

- ensure missing canonical primitives used by adjacent crates are present if and only if they belong here,
- keep legacy trace conversion helpers phase-labeled until remaining callers migrate,
- do not expand into domain policy.

### 5.2 `forge-memory-bridge`

Treat bridge as **mostly correct**.

Required work:

- preserve current canonical path behavior,
- tighten version-law handling,
- validate that exported structure maps cleanly into the logical import contract,
- keep `LegacyImportEnvelopeV1` clearly compat-only.

Forbidden:

- turning bridge into runtime service,
- introducing memory reads to invent semantics,
- moving promotion/comparability policy here.

### 5.3 `semantic-memory`

Required work:

- complete logical importer boundary,
- make atomic/idempotent batch import behavior explicit and test-backed,
- preserve contradiction/supersession semantics,
- explicitly handle row-level vs batch-level logical field obligations,
- keep evidence dereference explicit-only,
- keep derived graph materializations non-authoritative,
- add derivation/invalidation support for memory-local derived artifacts.

Forbidden:

- absorbing Forge ETL,
- using raw Forge receipts for ranking,
- silently treating repairable indexes/graphs as authoritative truth.

### 5.4 `knowledge-runtime`

Required work:

- close the gap between current search pipeline and canonical query-mode contract,
- ensure runtime uses memory canonical identity state instead of drifting toward in-memory authority,
- replace or fence any mutable identity logic that could violate runtime’s non-authoritative role,
- strengthen projection lifecycle handling beyond timestamp freshness tracking.

Forbidden:

- runtime-owned authoritative entity state,
- runtime projection shadow DB,
- false temporal semantics,
- silent scope widening.

### 5.5 `semantic-memory-forge`

Required work:

- strengthen evidence bundle schema and execution metadata,
- preserve raw verification truth semantics,
- ensure export envelopes remain Forge-owned,
- keep direct-write facilities exceptional,
- ensure estimator/refuter versioning and environment metadata are captured if sidecar work exists.

Forbidden:

- treating memory as raw-truth store,
- hiding retry or comparability semantics in opaque blobs,
- bypassing canonical export/import path in normal flow.

### 5.6 `llm-pipeline`

Required work:

- finish `TraceCtx` normal-path adoption,
- preserve retry ownership for transport and parser correction,
- ensure structured identifiers surface where verification-related logs/events exist.

### 5.7 `agent-graph`

Required work:

- replace compat trace fields in normal-path contracts,
- replace numeric attempt counters with canonical retry identity semantics where applicable,
- preserve graph-node retry ownership,
- ensure queue/fork/replay lineage is modeled honestly.

### 5.8 `job-queue`

Required work:

- replace compat trace fields in normal-path job/event surfaces,
- replace ambiguous attempt counters with canonical retry lineage,
- preserve queue ownership for durable import/export/rebuild retries,
- keep replay linkage explicit.

### 5.9 `ai-batch-queue`

Required work:

- complete trace/retry adoption instead of partial pass-through,
- keep batch-item retries distinguishable from outer job retries.

### 5.10 `tauri-queue`

Required work:

- finish passive propagation of canonical trace context where queue/UI boundaries carry it,
- phase-label or remove legacy include-trace-id assumptions once callers migrate.

---

## 6. Explicit live-code mismatches Claude MUST handle carefully

### 6.1 `ProjectionImportBatchV1` field completeness

Claude MUST compare the live `ProjectionImportBatchV1` / `ProjectionRow` shape against the full canonical logical field contract.

If fields are missing, Claude MUST choose one of these explicitly documented paths:

1. add the missing fields directly,
2. store them in normalized side tables with explicit query semantics,
3. satisfy them via batch-level metadata only if the preservation rule is explicit, testable, and query-safe.

Do not hand-wave this.

### 6.2 Runtime-local `EntityRegistry`

The live runtime snapshot includes in-memory entity registry capabilities such as register/merge/unmerge.

Claude MUST ensure runtime does **not** become authoritative identity state.

Acceptable outcomes include:

- making runtime identity machinery clearly adapter-fed / memory-derived query-time cache only,
- removing or fencing mutation paths from normal query execution,
- moving authoritative identity mutation responsibility fully back to memory-side canonical state.

### 6.3 `ProjectionTracker` gap

The live tracker is closer to freshness bookkeeping than full lifecycle/rebuild interpretation.

Claude MUST either evolve it or replace surrounding usage so runtime satisfies canonical lifecycle/rebuild responsibilities.

---

## 7. Migration and compatibility law for this pass

Claude MUST obey the following migration rules:

- old compat paths may remain for one migration cycle only if still shielding real callers,
- new work MUST target canonical path and canonical primitives,
- no new code/comment/spec may describe compat paths as desired architecture,
- legacy removal must wait until callers are actually migrated,
- rollback scenarios must not leave partially visible projection batches,
- older imported records must remain queryable unless an explicit breaking migration plan is implemented.

If a storage/schema change affects search indexes, graph materializations, or repairable accelerators, Claude MUST declare rebuild behavior explicitly.

---

## 8. Ugly-case matrix (release-blocking)

Claude MUST add or preserve tests covering these cases.

### Import / migration / temporal

- out-of-order envelope arrival,
- duplicate-but-not-identical envelope content,
- rollback on mid-import failure,
- late-arriving older-validity envelope,
- WAL/restart during projection import,
- restart during dual-path migration window,
- schema/version mismatch failure without corruption.

### Identity / relation / derivation

- alias unmerge after downstream projections exist,
- competing canonical IDs,
- contradiction import forcing preferred-status recomputation,
- relation/index derivation invalidation and bounded rebuild,
- estimator version change triggering invalidation.

### Retry / replay / queue semantics

- nested retry-owner conflict,
- replay linked-but-not-parented,
- queue retry storm not collapsing logical trials,
- batch-item retries remaining distinguishable from outer job retries,
- envelope import having no hidden inner retry owner.

### Runtime / ranking / degrade honesty

- temporal request not silently downgraded without warning,
- scope request not silently widened,
- equal semantic-base ranking remaining deterministic,
- unsupported causal leg not silently outranking supported evidence,
- runtime never joining fresher raw Forge receipts into ranking path.

### Safety / parser / patch robustness

Where relevant and high ROI, add fuzz/property tests for:

- patch application idempotence,
- apply-then-revert invariants,
- workspace-bound path safety,
- parser/repair round-trip invariants,
- malformed structured-output handling.

---

## 9. Done gates

This pass is not done until all of the following are true.

### 9.1 Boundary conformance

- Authority boundaries match the frozen doctrine.
- No forbidden dependency shapes were introduced.

### 9.2 Primitive conformance

- Canonical IDs and trace context come from `stack-ids`.
- No new public crate-local duplicates were introduced.
- Remaining compat wrappers are phase-labeled and justified.

### 9.3 Import-path conformance

- Canonical new work uses the bridge pipeline.
- Import batches are atomic and idempotent.
- Version fields are distinct and not conflated.

### 9.4 Memory conformance

- Required logical fields exist or are explicitly preserved via safe normalized/batch mechanisms.
- Contradiction/supersession semantics are representable.
- Evidence dereference is explicit-only.
- Derived graph materializations remain non-authoritative.
- Derivation/invalidation/bounded recomputation substrate exists for the required logical cases.

### 9.5 Runtime conformance

- Planning and merge remain bounded to memory-visible truth.
- Degradation is explicit and never silent.
- Scope and temporal semantics are honest.
- Runtime does not become authoritative identity or projection state.

### 9.6 Trace / retry conformance

- Retry ownership is singular and explicit.
- Attempt/trial lineage is recoverable.
- Trace context crosses execution boundaries.

### 9.7 Documentation conformance

- No new document describes compat shims as normative architecture.
- No new comment/spec reopens settled ownership questions without an explicit superseding spec.

### 9.8 Test conformance

- Ugly-case tests exist for the required cases above.
- Existing good behavior remains protected.

---

## 10. Out of scope for this pass

The following are explicitly out of scope unless they are required as a small supporting change:

- inventing a new architecture,
- replacing explainable ranking with opaque learned reranking,
- making runtime a persisted projection database,
- removing all compat shims on day one regardless of live callers,
- collapsing multiple crates into a mega-crate,
- treating graph/index overlays as authoritative truth,
- broad speculative performance rewrites unrelated to canonical closure.

---

## 11. Required final report format

Claude MUST end with a concise but honest report containing:

1. completed items by crate,
2. partially completed items by crate,
3. blockers or ambiguous areas,
4. tests added/updated,
5. compat surfaces still intentionally retained,
6. any follow-up pass recommended.

Do not claim full conformance if any required gate above is still open.
