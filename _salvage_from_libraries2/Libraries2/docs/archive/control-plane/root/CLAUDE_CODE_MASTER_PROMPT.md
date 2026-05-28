# Claude Code Master Prompt — Next Stack Closure Pass

You are working on a Rust workspace whose architecture is already settled. This is **not** a greenfield architecture pass.

Your job is to execute a **target-state closure pass** that turns a partially migrated but architecturally corrected stack into a more canonically wired, contract-complete, regression-resistant implementation.

## Read these files first if they exist

1. `MASTER_DELTA_SPEC_NEXT_CLAUDE_PASS.md`
2. `DELTA_CRATE_MATRIX.md`
3. `DELTA_ACCEPTANCE_AND_UGLY_CASES.md`
4. `CLAUDE.md`

Treat those as the primary handoff set.

If one or more are missing, use the rules below directly. Do **not** assume you have access to any hidden prior discussion.

---

## Architecture that is already frozen

These are settled and must not be casually reopened:

- Forge (`semantic-memory-forge`) owns **raw verification truth**.
- Memory (`semantic-memory`) owns **queryable projected truth**.
- Bridge (`forge-memory-bridge`) owns **projection transformation only**.
- Runtime (`knowledge-runtime`) owns **query planning, route planning, retrieval composition, provenance-preserving merge, degradation reporting, and projection lifecycle interpretation only**.
- `stack-ids` owns canonical cross-crate IDs, digests, and trace primitives.
- Compatibility shims are tolerated only as migration residue. They are **not** normative architecture.

You must not casually reopen:

- whether memory should own Forge transformation logic,
- whether runtime should persist authoritative knowledge state,
- whether bridge should fold into runtime,
- whether raw receipts should be silently joined into ranking paths,
- whether shared IDs should be redefined crate-by-crate,
- whether compat APIs should define the long-term architecture.

---

## Canonical normal path

All new normal-path work must target:

```text
Forge raw truth
  -> ExportEnvelopeV1
  -> forge-memory-bridge
  -> ProjectionImportBatchV1
  -> semantic-memory importer transaction
  -> queryable projections
```

Export and import version fields must remain distinct.

Legacy envelope/import shapes may remain only as compat shims while callers are still migrating.

---

## What is still wrong in the live stack

Assume these are real current-state issues unless code inspection proves otherwise:

- active compat surfaces still exist around legacy import envelopes, `import_envelope()`, legacy trace forms, and numeric attempt counters,
- runtime still materially lags on true temporal execution, full scope enforcement, projection rebuild execution, fuzzy entity resolution, and fully canonical causal projection consumption,
- the current `ProjectionImportBatchV1` shape may be semantically incomplete relative to the full canonical logical contract,
- runtime still has in-memory identity/lifecycle machinery that must not become authoritative,
- evidence bundle direction is correct, but learned graph scoring should not be a primary deliverable until verified-edge quality and observability invariants are in place.

---

## Your goals

You must, in roughly this order:

1. inventory and protect already-correct behavior,
2. finish canonical primitive adoption in normal-path code,
3. finish canonical import-path closure for new work,
4. close `semantic-memory` logical importer/storage gaps,
5. add derivation / invalidation / bounded recomputation substrate,
6. close runtime target-state gaps without turning runtime into an authority,
7. land minimal Forge evidence-bundle substrate with real schema/invalidation semantics,
8. finish trace / retry / replay law across execution boundaries,
9. add ugly-case tests and tighten docs/comments.

---

## Detailed constraints you must follow

### Primitive law

- `TraceCtx` from `stack-ids` is the canonical in-process trace form.
- Canonical IDs and digests come from `stack-ids`.
- Do not introduce new crate-local duplicates of canonical primitives.
- Keep compat conversion helpers only while still needed by real callers.

### Importer / storage law

`semantic-memory` must logically support operations equivalent to:

- `begin_projection_import(meta)`
- `upsert_claim_projections(rows, tx)`
- `upsert_relation_versions(rows, tx)`
- `upsert_entity_aliases(rows, tx)`
- `upsert_evidence_refs(rows, tx)`
- `upsert_episode_links(rows, tx)`
- `commit_projection_import(tx)`
- `abort_projection_import(tx)`

Importer must **not**:

- infer causal states,
- choose comparability policy,
- synthesize aliases from raw receipts,
- reinterpret malformed exports,
- decide promotion/refutation,
- dereference raw Forge receipts for ranking.

Import batches must be atomic and idempotent, and failures must leave durable import status.

### Memory logical contract

`semantic-memory` must be able to represent, query, and freshness-track at least:

- claim projection versions,
- relation versions,
- aliases and merge/split history,
- evidence refs,
- episode links,
- contradiction/supersession state,
- import log/status,
- derivation edges for memory-local derived artifacts.

If current types are semantically incomplete, fix that explicitly. Do not assume type existence equals contract completion.

### Runtime law

Runtime must:

- support logical routing across semantic / entity / temporal / mixed query modes,
- preserve canonical scope semantics,
- enforce strongest available scope filtering,
- warn when only partial scope pushdown exists,
- support true temporal execution or explicit honest degradation,
- use canonical memory identity state for entity resolution,
- never mutate authoritative identity state during query execution,
- keep merge provenance-preserving, deterministic, and explainable,
- strengthen projection lifecycle interpretation and rebuild orchestration.

Runtime must not:

- become authoritative for identity or projection truth,
- persist shadow copies of authoritative state,
- silently join fresher raw Forge receipts into results or ranking,
- fake temporal support via silent fallback.

### Derivation / invalidation law

You must add or complete the ability to represent at least these edges:

- raw receipt -> evidence bundle,
- evidence bundle -> claim projection,
- episode -> alias or relation candidate,
- relation version -> graph/index materialization,
- source projection -> future runtime cache/overlay artifact.

All derived artifacts must declare an invalidation mode.

Invalidation triggers include at least:

- contradiction import,
- refutation result,
- alias split/unmerge,
- envelope supersession,
- estimator version change,
- policy-profile change.

No correction may trigger a blind global rebuild unless explicitly requested.

### Evidence bundle law

Minimal evidence bundle schema should preserve at least:

- causal question,
- unit definition,
- treatment specification,
- outcome specification,
- covariates/confounders,
- identification rationale,
- estimator and estimate,
- refutations attempted + results,
- raw receipt / trace / replay handles.

If Python sidecar estimation/refutation exists or is introduced, record:

- estimator kind,
- estimator version,
- parameters,
- random seed if applicable,
- environment fingerprint,
- timeout,
- failure mode,
- versioned request/response schema.

### Trace / retry / replay law

Any boundary that can spawn work, enqueue work, call tools, import/export data, or emit durable records must propagate canonical trace context.

Verification-related logs/events must include relevant structured identifiers when available:

- `attempt_id`
- `trial_id`
- `claim_id`
- `patch_hash`
- `baseline_or_patch`

Retry ownership must stay singular and explicit:

- transport retry -> `llm-pipeline`
- parse/validator correction retry -> `llm-pipeline`
- graph node re-execution -> `agent-graph`
- durable import/export/rebuild retry -> `job-queue`
- AI batch item retry -> `ai-batch-queue`
- verification backend execution retry -> Forge orchestration layer
- envelope import transaction retry -> outer owner only; no hidden inner retry owner

A logical retry family has:

- exactly one `attempt_id`,
- exactly one primary retry owner,
- one or more `trial_id`s,
- explicit lineage links.

Replay gets:

- a new root trace,
- linkage to the original,
- either the same `attempt_id` only by explicit policy,
- or a new `attempt_id` with explicit replay linkage.

Queue/replay hops must use honest link semantics, not fake parent/child chains when the work actually forks or resumes.

### Direct-write law

Direct memory write-through is not the normal path.

If any emergency direct-write facility exists:

- keep it exceptional,
- keep it auditable,
- disable by default in release builds,
- do not let such records inherit normal comparability/import-lineage assumptions.

---

## Required ugly-case coverage

Add or preserve tests for:

- out-of-order envelope arrival,
- duplicate-but-not-identical envelope content,
- rollback on mid-import failure,
- late-arriving older-validity data,
- restart during migration window,
- version mismatch without corruption,
- alias unmerge after downstream projections,
- contradiction import affecting preferred status,
- estimator version invalidation,
- nested retry-owner conflict,
- replay linkage semantics,
- retry storms not collapsing logical trials,
- batch-item retries remaining distinguishable,
- temporal degrade honesty,
- no silent scope widening,
- deterministic tie-break behavior,
- no raw Forge join in runtime ranking path.

Where high ROI already exists, also add fuzz/property tests for patching, parser robustness, and workspace-bound safety invariants.

---

## Non-goals

Do not spend this pass on:

- inventing a new architecture,
- replacing explainable ranking with opaque learned reranking,
- collapsing crates into a mega-crate,
- deleting all compat shims on day one regardless of live callers,
- turning runtime into durable knowledge storage,
- making derived graph/index overlays authoritative,
- speculative performance rewrites unrelated to target-state closure.

---

## Working style

- Inspect before editing.
- Prefer minimal, explicit edits over broad rewrites.
- Preserve already-good behavior unless canonical law requires change.
- If a field/invariant is satisfied indirectly via normalized storage or batch-level metadata, make that explicit in code/tests/docs.
- Do not claim full conformance if any gate remains open.

---

## Required final output

At the end, provide:

1. completed changes by crate,
2. partial changes by crate,
3. blockers/unresolved ambiguities,
4. tests added or updated,
5. remaining compat surfaces and why they remain,
6. honest assessment against the acceptance gates.
