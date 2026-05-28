# MASTER CLAUDE CODE PROMPT — Canonical Stack v4 Implementation + Delta Cleanup

You are working inside the user's Rust workspace. Your job is to execute the **next implementation pass** for the stack architecture centered on:

- `semantic-memory`
- `knowledge-runtime`
- `semantic-memory-forge`
- new `stack-ids`
- new `forge-memory-bridge`

This is **not** a greenfield design exercise. It is a **conformance-and-migration pass** against the canonical architecture.

## Source priority — obey in this exact order

1. `MASTER_SUPPORTING_DELTA.md` — use this as the correction layer for unresolved contradictions and underspecified semantics.
2. `CANONICAL_STACK_SPEC_V4.md` — this is the canonical implementation spec for the next phase.
3. `LATEST4.md` — this is the current-state snapshot only, useful for understanding what exists now and what must be migrated.
4. `LIBRARY_SPEC.md` and `LIBRARY_SPEC_PATCH.md` — reference only where they do not conflict with the two files above.

If any older doc conflicts with the supporting delta or canonical v4, **the older doc loses**.

---

## Non-negotiable doctrine

Keep these laws intact throughout the implementation:

1. **Raw experimental truth is not the same thing as queryable memory truth.**
2. **Multiple views may share IDs, but they do not share authority.**

That means:

- Forge remains authoritative for raw verification truth.
- `semantic-memory` remains authoritative for queryable memory truth.
- `knowledge-runtime` remains a planner / merger / explainer, not a database.
- `forge-memory-bridge` transforms exports into import batches; it does not become a policy engine.
- `stack-ids` contains only shared ID/trace primitives and nothing else.

---

## Current-state facts you must respect

The live stack currently has these realities:

- `semantic-memory` still has the old `import_envelope()` path and legacy `ImportRecord = Fact | Episode`.
- `knowledge-runtime` currently supports `Semantic | Entity | Temporal | Mixed`, but temporal execution downgrades to hybrid, scope pushdown is partial, projection persistence is in-memory only, there is no Forge causal adapter, and entity resolution is exact-only.
- `semantic-memory-forge` currently exports through `EpisodeExport::to_import_envelope()` and exposes `danger-sm-write` as an explicit opt-in feature.

Do **not** pretend these are already migrated. Implement from this real state.

---

## Your mission

Make the workspace conform to the canonical architecture and supporting delta by doing all of the following:

1. create `stack-ids`,
2. create `forge-memory-bridge`,
3. define and adopt `ExportEnvelopeV1`,
4. add the projection-import boundary and projection storage in `semantic-memory`,
5. preserve one migration-cycle compatibility for the legacy envelope path,
6. migrate runtime and supporting crates to shared ID/trace law,
7. update docs so the repo stops teaching the wrong map,
8. add ugly-case tests and migration tests,
9. leave behind a clean, explicit implementation record.

This pass is **not done** until the code, docs, migrations, and tests all line up.

---

## Critical corrections you must apply from the supporting delta

You must implement these decisions exactly.

### A. Canonical terminology
Use only these names in new code and new docs:

- `ExportEnvelopeV1`
- `ProjectionImportBatchV1`
- `LegacyImportEnvelopeV1`

The bare term `ImportEnvelope` is forbidden in new code and new docs except inside clearly marked legacy compatibility sections.

### B. Bridge/API contradiction resolution
`ProjectionImportBatchV1` is bridge-owned, but bridge row-batch types must not leak into public memory APIs.

Resolve this by using one of these acceptable patterns:

- preferred: a **non-public integration boundary** consumed by `forge-memory-bridge`, while `semantic-memory` public APIs remain memory-owned, or
- acceptable: a memory-owned public importer DTO layer, with bridge-owned batch types mapped internally before crossing the public boundary.

Do **not** leave bridge-owned row-batch structs exposed as normal public memory API contracts.

### C. Claim-version identity
Add an explicit `claim_version_id` to the logical and physical claim projection model. Do not keep claim versions as anonymous temporal blobs.

### D. Relation-version parity
Relation versions must preserve the same audit-grade metadata shape as claim versions where semantically applicable. Do not leave relation rows weaker by accident. Preserve at least:

- source authority,
- trace linkage,
- freshness / import state where applicable,
- envelope provenance.

### E. Alias scope and review durability
Alias / merge state must include explicit scope semantics and durable review state. At minimum:

- aliases/merge decisions must not bleed silently across scopes,
- pending review must survive restart and be queryable,
- automated flows must never mark `human_confirmed_final`.

### F. Namespace-to-scope migration rule
Legacy `namespace` is not the long-term partition contract. The canonical partition key is `ScopeKey`.

During migration, implement a deterministic reversible mapping from legacy `namespace` to canonical `ScopeKey` for legacy imports. Document the mapping clearly and use it consistently in bridge/import logic.

### G. Evidence-ref tightening
Evidence refs must remain opaque by default, but their fetch path must not be hand-wavy. Implement a clear audit fetch contract. At minimum:

- preserve `claim_id`,
- preserve version-local linkage when available (`claim_version_id` nullable/optional is acceptable),
- preserve source authority,
- preserve envelope provenance,
- preserve a canonical raw-evidence fetch handle,
- keep audit dereference explicit only.

### H. Digest law
Do not use fuzzy digest computation. Define and document a canonical content-digest algorithm for export/import idempotency. Use deterministic canonical serialization and a stable hash scheme.

### I. Phase tags
Where a target feature is not current-state reality, mark it clearly in docs and code comments as one of:

- current / implemented now
- compatibility / migration-only
- phase-gated target

Do not write docs that make a future-phase behavior look already implemented.

---

## Required implementation phases

Execute in this order unless the codebase absolutely forces a tiny dependency inversion.

### Phase 1 — shared primitives and canonical export vocabulary

1. Add new crate `stack-ids`.
2. Move or replace crate-local shared IDs / trace wrappers with `stack-ids` newtypes:
   - `AttemptId`
   - `TrialId`
   - `ArtifactId`
   - `EpisodeId`
   - `ClaimId`
   - `EntityId`
   - `EnvelopeId`
   - `ProjectionId`
   - `ScopeKey`
   - `TraceCtx`
3. Add W3C trace-context helpers in `stack-ids`.
4. Remove or deprecate crate-local copies of shared ID/trace concepts.
5. Define `ExportEnvelopeV1` in Forge-owned schema.
6. Keep `LegacyImportEnvelopeV1` intact for migration compatibility.

### Phase 2 — bridge crate

1. Create `forge-memory-bridge`.
2. It must:
   - consume `ExportEnvelopeV1`,
   - validate export/import version compatibility,
   - preserve envelope ID, content digest, lineage, receipt refs, and trace metadata,
   - transform into `ProjectionImportBatchV1`,
   - reject malformed exports before memory writes begin.
3. It must not:
   - evaluate comparability,
   - decide promotion,
   - guess missing semantics from live memory,
   - become a query service.

### Phase 3 — semantic-memory importer boundary and storage

1. Add a storage-oriented projection importer boundary with the logical operations from canonical v4.
2. Add the projection storage needed for:
   - claim projection versions,
   - relation versions,
   - entity aliases / merge decisions / review state,
   - evidence refs,
   - episode links,
   - contradiction/supersession state,
   - import log/status,
   - derivation edges.
3. Add `claim_version_id`.
4. Preserve logical invariants for valid time, transaction time, preferred-open uniqueness, overlap law, derivation, and freshness.
5. Keep `graph_view()` derived only.
6. Keep `import_envelope()` working, but demote it to shim / compatibility-path status.

### Phase 4 — Forge export and migration support

1. Add / harden `ExportEnvelopeV1`.
2. Keep `EpisodeExport::to_import_envelope()` only as migration compatibility.
3. Preserve Forge authority for:
   - attempts,
   - trials,
   - eval runs,
   - comparability snapshots,
   - estimator metadata,
   - replay/refutation outputs,
   - raw receipts,
   - promotion/archive state.
4. Keep `danger-sm-write` exceptional and auditable.

### Phase 5 — runtime conformance

1. Migrate `knowledge-runtime` shared IDs to `stack-ids`.
2. Keep runtime non-authoritative and ephemeral.
3. Add machine-readable adapter capability negotiation.
4. Expand retrieval-leg result contracts to include:
   - source view,
   - score components,
   - freshness/version metadata,
   - degradation markers,
   - pagination state,
   - consistency marker.
5. Keep causal query modes feature-gated until a causal adapter exists.
6. Keep silent downgrade forbidden; degradation must be explicit.

### Phase 6 — retry / trace conformance across supporting crates

Conform `llm-pipeline`, `agent-graph`, `job-queue`, `ai-batch-queue`, and relevant import/export orchestration to one retry/trace law:

- one retry owner per retry family,
- one `attempt_id`,
- many `trial_id`s if retried,
- explicit linkage,
- W3C trace propagation across boundaries,
- no fake parent/child chains for queue hops or replay hops where span links are correct.

### Phase 7 — docs, migration, and release hardening

1. Update docs so the repo stops teaching the old architecture as the normative one.
2. Explicitly mark `LATEST4.md`-style facts as current-state snapshot only if you touch equivalent reference docs.
3. Add migration/backfill logic so old imported `Fact | Episode` records remain queryable with preserved provenance/history.
4. Add compatibility notes and release notes.

---

## Required storage/model rules

Implement these without weasel wording.

### Claim projection versions
Must have an explicit version identity and preserve at minimum:

- `claim_version_id`
- `claim_id`
- `claim_state`
- `projection_family`
- `subject_entity_id`
- `predicate`
- `object_anchor`
- `scope_key`
- `valid_from`
- `valid_to`
- `recorded_at`
- `preferred_open`
- `source_envelope_id`
- `source_authority`
- `trace_id`
- `freshness`
- `contradiction_status`
- `supersedes_claim_version_id` where applicable

### Relation versions
Must preserve at minimum:

- `relation_version_id`
- `subject_entity_id`
- `predicate`
- `object_anchor`
- `scope_key`
- `claim_id` or `source_episode_id`
- `valid_from`
- `valid_to`
- `recorded_at`
- `preferred_open`
- `supersedes_relation_version_id`
- `contradiction_status`
- `source_confidence`
- `projection_family`
- `source_envelope_id`
- `source_authority`
- `trace_id` when available / applicable
- freshness / import-state metadata where applicable

### Entity alias / merge state
Must preserve at minimum:

- canonical `entity_id`
- alias text
- alias source
- match evidence blob
- confidence
- merge decision provenance
- explicit scope semantics
- durable review state
- `is_human_confirmed`
- `is_human_confirmed_final`
- `superseded_by_entity_id` if any
- split history pointer if any
- provenance to envelope/artifact
- `recorded_at`

### Evidence refs
Must preserve at minimum:

- `claim_id`
- version-local linkage when available
- canonical raw-evidence fetch handle
- source authority
- source envelope provenance
- `recorded_at`
- enough metadata for explicit audit dereference only

---

## Required behavioral rules

Implement all of these.

### Import semantics
- one `ProjectionImportBatchV1` == one atomic import unit
- at-least-once delivery
- idempotent ingest
- stable dedupe keys on both bridge and storage sides
- no partial visibility on failure

### Freshness
Projection freshness must distinguish at least:

- `Current`
- `Stale`
- `Superseded`
- `ImportFailed`
- `NeverImported`
- `ImportLagging`

### Temporal law
- valid time and transaction time stay distinct
- timestamps UTC, millisecond precision minimum
- late older-validity arrivals are preserved, not discarded
- at most one preferred-open version per logical key
- preferred overlap is forbidden
- query defaults must reflect valid-time vs transaction-time semantics

### Entity resolution
- deterministic blocking
- probabilistic scoring
- merge/no-merge/human-review
- pending review durable and queryable
- pre-review fuzzy matches may aid recall but may not become canonical identity

### Comparability and lifecycle
Preserve immutable comparability snapshots and lifecycle rules. Do not move policy semantics into memory or runtime.

### Runtime ranking
Final result scoring must remain explainable additive composition, not opaque fused magic.

---

## Tests you must add or strengthen

Do not stop at unit tests that merely prove the sun rose this morning.

At minimum cover:

- out-of-order older-validity imports,
- duplicate-but-not-identical envelopes,
- mid-import rollback,
- WAL crash / restart during projection import,
- HNSW rebuild after projection migration,
- dual-path migration restart,
- alias unmerge after downstream projections exist,
- human-confirmed-final merge protection,
- import-lag propagation to runtime warnings,
- unsupported causal legs remaining gated,
- retry-owner singularity,
- replay / queue hops preserving linkage,
- deterministic score explainability,
- backfill correctness for old imported `Fact | Episode` data,
- bridge/version incompatibility rejection,
- explicit audit-only evidence dereference,
- `danger-sm-write` audit tagging and non-default behavior.

---

## Required documentation deliverables

Leave these artifacts behind in the repo after implementation:

1. `SPEC_PATCH_IMPLEMENTED.md`
   - list every spec contradiction or underspecified area you resolved,
   - list the exact decision taken,
   - explain the migration impact.

2. `MIGRATION_NOTES_V4.md`
   - rollout order,
   - compatibility path,
   - namespace -> scope mapping,
   - backfill behavior,
   - deprecations.

3. `TRACE_RETRY_CONTRACT.md`
   - retry-owner matrix,
   - trace propagation rules,
   - queue-hop / replay-link semantics.

4. updated crate docs / READMEs / module docs where old wording still teaches the wrong architecture.

---

## Forbidden shortcuts

Do not do any of these:

- do not keep the old `import_envelope()` path as if it were still normative,
- do not let `knowledge-runtime` persist shadow truth,
- do not let bridge types become accidental public memory contracts,
- do not add new broad shared crates,
- do not silently mix stale projections with fresh raw Forge receipts,
- do not implement fuzzy entity resolution as authoritative merge without durable review,
- do not use `danger-sm-write` as a normal-path dependency,
- do not resolve missing semantics by stuffing arbitrary JSON blobs everywhere and calling it “flexible”.

---

## Implementation style expectations

- Prefer additive migrations over destructive ones.
- Preserve backward compatibility for one migration cycle where required.
- Keep public APIs conservative.
- Use typed enums / structs where the current code is too stringly.
- Add comments only where they reduce future misimplementation.
- When you must choose between cleverness and auditability, choose auditability.

---

## Final output requirements

When you finish, provide:

1. a concise architecture summary of what changed,
2. a crate-by-crate change list,
3. migration/backfill notes,
4. test coverage added,
5. any intentionally deferred items that remain phase-gated,
6. a list of doc files updated so the repo no longer lies to the implementer.

Do the work. Do not just describe the work.
