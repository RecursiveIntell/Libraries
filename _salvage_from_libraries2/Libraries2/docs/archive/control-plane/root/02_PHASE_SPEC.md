# Next-Phase Implementation Spec

## Title

**Invariant-Enforced Knowledge / Verification Runtime Phase**

## Purpose

Turn the current architecture from a well-specified design into an implementation that enforces its key contracts in code.

This phase focuses on the seam between:

- `semantic-memory`
- `knowledge-runtime`
- `semantic-memory-forge`
- the bridge/import layer between Forge exports and memory projections

The core goal is not feature sprawl. The core goal is **architectural truthfulness**.

---

# 1. Goals

## 1.1 Primary goals

1. Enforce authority boundaries between the major crates.
2. Make canonical identity, lineage, versioning, and provenance rules explicit.
3. Implement a narrow, storage-oriented projection import boundary.
4. Make runtime query semantics legible, deterministic, and explainable.
5. Propagate trace/provenance context across boundaries.
6. Surface stale/import/degraded behavior explicitly.
7. Add tests that prove the invariants and reject invalid paths.

## 1.2 Secondary goals

1. Reduce future architectural drift.
2. Prevent silent behavior that contradicts the doctrine.
3. Create a base for later temporal/causal/entity retrieval expansion.

---

# 2. Non-goals

This phase is **not** for:

1. Redesigning the entire stack.
2. Inventing a large number of new crates unless unavoidable.
3. Solving every future temporal/causal reasoning feature in one pass.
4. Hiding unfinished behavior behind convenience fallbacks.
5. Adding caches/materializations without freshness contracts.
6. Implementing Forge policy logic inside `semantic-memory`.
7. Turning `knowledge-runtime` into durable source-of-truth storage.

---

# 3. Architectural doctrine to enforce

## 3.1 Authority doctrine

### `semantic-memory`

Authoritative for queryable knowledge state:

- facts
- documents/chunks
- sessions/messages
- episodes
- imported claim/entity/relation projections
- search indexes and query-facing persistence

### Forge

Authoritative for raw verification state:

- attempts
- trials
- raw receipts
- evaluation lineage
- promotion/archive state
- raw causal/experimental evidence
- export envelopes

### `knowledge-runtime`

Authoritative for planning and merge policy only:

- query classification
- route planning
- scoped entity resolution
- retrieval composition
- provenance-preserving merge
- warning/explain surfaces
- projection lifecycle interpretation

It must never own durable source truth.

## 3.2 Shared-ID doctrine

IDs may be shared across views, but authority is not shared.

That means:

- common identifiers are allowed,
- but storage ownership and interpretation ownership remain explicit.

## 3.3 Consistency doctrine

The system is:

- eventually consistent across stores,
- monotonic within a lineage.

No silent history rewrite.
No hidden cross-store synthesis masquerading as single-source truth.

---

# 4. Required implementation outcomes

## 4.1 Canonical identity and lineage

The code must define or consolidate canonical semantics for at least the following conceptual identifiers:

- `EntityId`
- `ClaimId`
- `EpisodeId`
- `EnvelopeId`
- `AttemptId`
- `TrialId`
- `ArtifactId`
- `TraceCtx` or equivalent trace carrier

Requirements:

1. ID ownership and crate placement must be explicit.
2. Where duplicate local definitions exist, remove or reconcile them.
3. Versioning/lineage metadata must be represented explicitly for imported claims/relations.
4. Invalid or ambiguous lineage states must be rejectable.

## 4.2 Projection import boundary

A narrow importer boundary must exist for writing queryable projections into `semantic-memory`.

This may be a crate or module depending on repo structure, but the boundary must be explicit.

### The importer boundary must:

- accept already-interpreted projection inputs,
- perform atomic import by envelope,
- support idempotent ingest,
- record projection provenance/freshness/version metadata,
- update only storage/index-facing state.

### The importer boundary must not:

- infer Forge semantics,
- decide promotion policy,
- invent comparability policy,
- reinterpret malformed raw receipts,
- contain hidden domain transformation logic that properly belongs to Forge.

### `semantic-memory` should expose storage-oriented methods, such as conceptually:

- begin import transaction
- upsert projection rows
- upsert relation versions
- upsert aliases
- upsert evidence refs
- upsert episode links
- commit/abort import

Exact names may differ. The semantic boundary must not.

## 4.3 Envelope/import behavior

The import pipeline must satisfy:

1. **Atomicity per envelope**
2. **At-least-once delivery compatibility**
3. **Idempotent ingest**
4. **Explicit dedupe semantics**
5. **No partial visibility**

Recommended dedupe basis conceptually includes:

- `envelope_id`
- schema/export version
- content digest

Do not rely on brittle ad hoc tuple dedupe such as attempt ID alone.

## 4.4 Projection lifecycle status

Imported projections must expose lifecycle/freshness/error semantics such as conceptual states like:

- healthy/current
- stale
- superseded
- import lagging
- import failed
- invalidated

The exact enum can differ, but the runtime must be able to explain projection state instead of implying freshness blindly.

## 4.5 Runtime query semantics

`knowledge-runtime` must have explicit and testable semantics for:

- classification
- route planning
- per-leg execution
- result merge
- scope handling
- degraded path warnings
- tie-breaking / determinism
- provenance/explanation metadata

### Requirements

1. Degraded behavior must produce warnings, not silent downgrades.
2. Scope enforcement must be explicit about what is pushed down vs enforced locally.
3. Result merge must preserve source-leg provenance.
4. Ranking behavior must be explainable enough to debug.
5. Duplicate fusion must be deterministic.

## 4.6 Trace/provenance propagation

A trace/provenance carrier must survive across:

- import boundaries
- runtime query execution
- merge stages
- downstream explanation surfaces

At minimum, the code should make it possible to answer:

- which path produced this result?
- from which projection/import version?
- was this data imported, stale, degraded, or partially enforced?
- what trace/operation context led here?

## 4.7 Failure behavior

The system must fail explicitly when invariants are violated.

Examples:

- invalid envelope metadata
- ambiguous lineage/version relationships
- partial import attempts
- projection writes outside importer boundary
- missing required provenance/trace info where mandated
- forbidden crate-boundary violations

No silent “best effort” that manufactures false confidence.

---

# 5. Crate-level requirements

## 5.1 `semantic-memory`

### Must do

- expose a narrow storage-facing projection import API
- store projection/version/provenance metadata required by the phase
- keep imports transactional
- keep search/query paths compatible with projection-state awareness where needed
- support explainability surfaces needed by the runtime

### Must not do

- own Forge policy logic
- interpret raw verification semantics
- decide comparability/promotions/evaluation outcomes
- become a domain ETL layer for raw Forge payloads

## 5.2 `knowledge-runtime`

### Must do

- remain bounded to planning/retrieval/merge/projection interpretation
- make warnings and degraded behavior explicit
- preserve provenance in multi-leg merges
- expose deterministic semantics for duplicate fusion and tie-breaking
- surface projection freshness/state clearly

### Must not do

- persist authoritative source truth
- synthesize hidden truth by mixing stale projections and fresh receipts in default user-facing paths
- hide partial scope enforcement

## 5.3 Forge / `semantic-memory-forge`

### Must do

- remain source of raw verification truth
- export well-formed projection envelopes or already-interpreted bridge inputs
- own evaluation/refutation/comparability semantics

### Must not do

- offload raw-truth interpretation into `semantic-memory`
- depend on runtime to decide verification policy

## 5.4 Bridge/import layer

### Must do

- mediate projection transformation/import
- validate importable envelope structure
- map export payloads into storage-facing projection rows/records
- be the place where projection transformation is explicit

### Must not do

- become a second query runtime
- become a second source-of-truth store
- quietly absorb Forge or memory authority

---

# 6. Suggested implementation strategy

## 6.1 Preferred order

1. Consolidate identity/provenance types and boundary rules.
2. Introduce or formalize importer/projection interfaces.
3. Add transactional/idempotent import path.
4. Extend projection lifecycle metadata.
5. Tighten runtime query semantics and explanation/warnings.
6. Add trace propagation improvements.
7. Add tests and boundary-violation rejections.
8. Only after invariants are real, fill smaller adapters/gaps.

## 6.2 When to create a new crate

Create a new bridge crate only if the repo structure currently forces one of these bad outcomes otherwise:

- Forge-specific transformation logic inside `semantic-memory`
- runtime-specific policy inside Forge
- untestable or tangled import transformation ownership

If a module boundary is sufficient and cleaner in the current repo, that is acceptable.

---

# 7. Testing requirements

This phase is incomplete without tests.

Required classes of tests:

## 7.1 Boundary tests

Prove that:

- projection writes occur only through the intended boundary
- forbidden crate-level responsibilities are not exercised through public APIs
- invalid transformation states are rejected

## 7.2 Import correctness tests

Prove:

- atomic import per envelope
- no partial visibility on failure
- idempotent repeated ingest
- supersede/stale/invalidation behavior is explicit
- dedupe semantics work correctly

## 7.3 Runtime semantics tests

Prove:

- degraded temporal behavior emits warnings
- partial scope enforcement is surfaced clearly
- duplicate fusion is deterministic
- result provenance survives merge
- tie-breaking is deterministic and documented

## 7.4 Provenance/trace tests

Prove:

- trace/provenance is preserved across import/query/merge
- explanation surfaces expose meaningful causality of result origin

## 7.5 Regression tests

Add tests for every bug, ambiguity, or boundary leak found during implementation.

---

# 8. Documentation requirements

The coding model should update crate/module docs where necessary so the enforced design is visible in code.

Required documentation themes:

- who owns what
- what the importer boundary is for
- what the runtime may and may not do
- what “projection freshness/state” means
- where degraded behavior is expected and how it surfaces

Documentation should be concise and aligned to actual code, not aspirational fiction.

---

# 9. Done criteria

This phase is done only when:

1. The relevant crates enforce the authority split in code structure and public APIs.
2. Import behavior is atomic, idempotent, and explicit.
3. Runtime semantics are deterministic and observable.
4. Projection lifecycle state is queryable and explainable.
5. Trace/provenance survives the key boundaries.
6. Tests prove the invariants.
7. No major unfinished behavior is hidden behind silent fallback.

