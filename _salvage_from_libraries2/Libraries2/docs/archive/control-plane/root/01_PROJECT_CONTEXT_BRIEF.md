# Project Context Brief

This document is a compact, self-contained summary of the current architecture and intended boundaries of the Rust project ecosystem. It is written so that a coding model can operate **without seeing the user's other design docs**.

---

# 1. High-level ecosystem shape

The user has a Rust-first AI/knowledge stack composed of several crates. The key ones for this phase are:

- `semantic-memory`
- `knowledge-runtime`
- `semantic-memory-forge` (also described in prior docs as the Forge / living-memory layer)
- a likely bridge/import crate that should mediate Forge-to-memory projection import
- supporting orchestration/execution crates such as `agent-graph`, `job-queue`, `ai-batch-queue`, `llm-pipeline`

This phase focuses **only** on the knowledge / verification / runtime seam.

---

# 2. Current crate roles

## `semantic-memory`

`semantic-memory` is the authoritative local store for **queryable knowledge state**.

It already functions as a serious local-first memory/retrieval engine with:

- SQLite persistence
- FTS5 full-text search
- embedding-backed semantic retrieval
- optional HNSW ANN indexing
- documents, chunks, facts, sessions, messages, episodes
- hybrid scoring and explainable result breakdowns
- a **derived** graph view

It should remain authoritative for:

- facts
- documents/chunks
- sessions/messages
- episodes
- queryable projections of entities/relations/claims
- search indexes and query-facing storage

It should **not** become a hidden Forge, policy engine, or evaluation system.

## `knowledge-runtime`

`knowledge-runtime` is a bounded orchestration / retrieval composition layer **over** `semantic-memory`.

It already has or partially has:

- query classification
- route planning
- scoped entity resolution
- provenance-preserving merge
- projection status tracking
- warnings for degraded behavior

It is **not** the durable source of truth.

It should own:

- query planning
- retrieval composition
- scoped entity resolution
- merge policy
- provenance-preserving result assembly
- runtime warnings and explainability surfaces

It should **not** become a second database or silently store authoritative records.

## `semantic-memory-forge`

Forge is authoritative for **raw verification state** and experiment/evaluation lineage.

It is the verification / attribution / evaluation layer.

It should own:

- attempts
- trials
- eval runs
- raw receipts
- causal/experimental provenance
- archive and promotion state
- export envelopes for downstream projections

It should **not** be reduced to “just another cache” or forced to share authority with memory.

## Bridge/import layer

A narrow bridge/import layer should exist between Forge exports and `semantic-memory` projection writes.

This is important because transformation from raw verification truth into queryable projections is domain-specific and should not be buried inside `semantic-memory`.

Tentative placement:

- `forge-memory-bridge`
- or `semantic-memory-forge/importer`

This phase may implement that bridge as a new crate or a tightly-scoped module depending on the current repo layout.

---

# 3. Core doctrine

Two sentences govern the architecture:

1. **Raw experimental truth is not the same thing as queryable memory truth.**
2. **Multiple views may share IDs, but they do not share authority.**

Consequences:

- Forge truth and memory projections are linked but not conflated.
- `knowledge-runtime` may reason over imported projections, but does not own raw truth.
- Default user-facing retrieval should come from imported/queryable projections.
- Audit and drill-down paths may consult Forge raw receipts explicitly.
- No default path should perform hidden cross-store joins that combine stale projections with fresher raw receipts and present them as a coherent single truth.

---

# 4. Current known implementation gaps

The architecture docs indicate that `knowledge-runtime` is **promising but incomplete**. Gaps include:

- temporal execution downgraded to hybrid search
- full scope enforcement not pushed all the way down
- persistence accepted/configured but not truly implemented
- Forge causal projection adapter missing
- fuzzy entity resolution incomplete or absent
- rebuild execution and lifecycle not fully realized

This phase should prioritize:

- enforcement of boundaries,
- importer discipline,
- explicit runtime semantics,
- observability,
- deterministic failure behavior,
- and tests.

Do **not** try to solve every future capability in one shot.

---

# 5. Shared primitives and contracts

The architecture strongly implies that some cross-stack primitives should be stable and explicit.

Examples of likely shared concepts:

- `AttemptId`
- `TrialId`
- `ArtifactId`
- `EpisodeId`
- `EntityId`
- `ClaimId`
- `EnvelopeId`
- `TraceCtx`

The exact location of these types may vary by repository structure, but this phase should ensure:

- canonical ownership is clear,
- they are semantically stable,
- and they are not duplicated inconsistently across crates.

Identity/trace primitives should be low-churn and broadly reusable.

Evidence/export schemas should remain Forge-owned and be consumed through adapters.

---

# 6. Consistency model

The intended model is:

- **eventually consistent across stores**
- **monotonic within a claim lineage**

Practical meaning:

1. Forge commits raw verification state first.
2. Forge exports versioned envelopes.
3. The bridge/import layer transforms those envelopes into queryable projections.
4. `semantic-memory` atomically ingests those projections.
5. `knowledge-runtime` queries imported state and exposes projection freshness / lag / invalidation clearly.

Key requirements:

- import must be atomic per envelope
- delivery may be at-least-once
- ingest must be idempotent
- stale or failed imports must surface in status and warnings
- history must not be silently rewritten

---

# 7. What this phase must defend against

The architecture is especially trying to avoid these failure modes:

## A. `knowledge-runtime` becomes a shadow database

Bad signs:

- it starts persisting authoritative records,
- owns durable source truth,
- or silently builds state that diverges from memory/Forge.

## B. `semantic-memory` becomes a secret second Forge

Bad signs:

- it starts interpreting raw Forge payloads,
- owns verification semantics,
- decides comparability or promotion policy,
- or contains Forge-specific ETL/business logic.

## C. Hidden cross-store truth synthesis

Bad signs:

- raw receipts and stale projections are merged invisibly,
- freshness is not explicit,
- or user answers imply stronger consistency than exists.

## D. Retry/trace ambiguity

Bad signs:

- retries cannot be attributed cleanly,
- trace context disappears across boundaries,
- lineage becomes ambiguous,
- or observability is insufficient to explain why a result exists.

## E. Opaque retrieval behavior

Bad signs:

- ranking is not explainable,
- tie-breaking is nondeterministic,
- scope semantics are unclear,
- or degraded behavior happens silently.

---

# 8. Desired end-state for this phase

At the end of this phase, the codebase should have:

- enforced crate boundaries for authority and transformation
- canonical IDs and lineage/version semantics
- a narrow importer/projection boundary
- explicit runtime query semantics
- strong trace/provenance propagation
- deterministic failure behavior
- tests that prove the above

This phase should make the current architectural doctrine **operational**, not merely documented.

