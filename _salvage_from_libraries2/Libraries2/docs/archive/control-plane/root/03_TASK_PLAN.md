# Ordered Task Plan

This task plan is deliberately concrete. It is intended for an implementation model, not for general brainstorming.

---

# Workstream 1 — Boundary and type consolidation

## Task 1.1 — Inventory current identity/provenance types

### Goal

Find all existing definitions and uses of:

- entity IDs
- claim/projection IDs
- envelope/import IDs
- attempt/trial/artifact IDs
- trace/provenance carriers
- projection state/freshness enums

### Deliverables

- a short internal inventory note or code comments identifying duplicates/conflicts
- a decision on canonical ownership location for each concept

### Acceptance criteria

- duplicate or conflicting definitions are identified
- canonical ownership is explicit
- no unresolved “we will clean this later” ambiguity remains for core IDs

### Must not do

- introduce three new type layers for the sake of abstraction theater

---

## Task 1.2 — Consolidate or wrap canonical identity/provenance types

### Goal

Make core types explicit and stable enough to support the rest of the phase.

### Deliverables

- canonical ID/provenance definitions or wrappers
- updated imports/usages in affected crates/modules
- explicit version/lineage metadata types where missing

### Acceptance criteria

- affected code compiles against the canonical types
- no inconsistent local redefinition remains for core concepts
- lineage/version relationships are representable explicitly

### Must not do

- force large-scale unrelated refactors
- break semantically useful existing types without replacement

---

# Workstream 2 — Projection import boundary

## Task 2.1 — Create/formalize the importer boundary

### Goal

Introduce an explicit projection-import seam between Forge outputs and `semantic-memory` storage writes.

### Deliverables

One of:

- a new bridge crate, or
- a sharply bounded integration module

The boundary must have clearly named responsibilities and no hidden raw-truth interpretation.

### Acceptance criteria

- there is exactly one intended path for projection import
- transformation ownership is explicit
- `semantic-memory` public APIs do not encourage raw Forge payload interpretation

### Must not do

- bury the bridge logic inside unrelated storage/query code

---

## Task 2.2 — Add storage-oriented import API to `semantic-memory`

### Goal

Expose transactional import primitives that are storage-facing rather than policy-facing.

### Deliverables

Conceptual operations such as:

- begin import
- write projection rows
- write relation versions
- write aliases
- write evidence refs
- write episode links
- commit/abort

### Acceptance criteria

- API is narrow and storage-oriented
- API does not decide verification semantics
- imports can be atomic and testable

### Must not do

- add Forge policy into `semantic-memory`
- make generic search code responsible for envelope interpretation

---

## Task 2.3 — Implement idempotent envelope ingest

### Goal

Ensure repeated delivery of the same import unit does not corrupt or duplicate projection state.

### Deliverables

- dedupe strategy
- content/version-aware import guardrails
- tests for repeated ingest

### Acceptance criteria

- repeated ingest of the same envelope is safe
- partial failure does not expose partial visibility
- dedupe is not based on brittle ad hoc heuristics alone

### Must not do

- use weak dedupe keys that collapse future refutations/supersessions incorrectly

---

# Workstream 3 — Projection lifecycle and query semantics

## Task 3.1 — Extend projection lifecycle representation

### Goal

Represent freshness/health/import state explicitly enough for runtime warnings and explanation.

### Deliverables

- projection lifecycle state enum/struct updates
- version/supersession/invalidation metadata where needed

### Acceptance criteria

- stale/import-lagging/import-failed/superseded style states can be expressed
- runtime can query and surface them

### Must not do

- imply binary “fresh or not” semantics if the architecture requires richer states

---

## Task 3.2 — Tighten runtime route/merge semantics

### Goal

Make runtime retrieval composition deterministic and observable.

### Deliverables

- explicit tie-breaking rules
- deterministic duplicate fusion
- explicit source-leg provenance preservation
- warning emission for degraded paths

### Acceptance criteria

- repeated equivalent queries produce stable ordering where expected
- merge behavior is explainable
- degraded behavior is not silent

### Must not do

- hide scoring/ranking weirdness behind “best effort” vagueness

---

## Task 3.3 — Make scope enforcement transparency explicit

### Goal

Clearly distinguish pushdown enforcement from runtime-local enforcement.

### Deliverables

- code/doc clarifications in runtime/adapters
- warning or trace metadata when scope is only partially enforced downstream

### Acceptance criteria

- users of the runtime can tell when scope is only partially pushed down
- tests verify warnings exist for degraded scope semantics

### Must not do

- present partial scope enforcement as full enforcement

---

# Workstream 4 — Trace and provenance propagation

## Task 4.1 — Normalize trace/provenance carrier usage

### Goal

Ensure trace/provenance context survives import and runtime execution.

### Deliverables

- common trace carrier usage pattern
- propagation through import, query, and merge paths
- explanation metadata surfaces updated as needed

### Acceptance criteria

- a result can be traced back to its source path/version/import context
- tests show trace/provenance does not disappear across the key seams

### Must not do

- add ornamental tracing that cannot answer practical debugging questions

---

## Task 4.2 — Surface failure/degradation explicitly

### Goal

Make runtime and import anomalies inspectable.

### Deliverables

- warning/error variants where needed
- structured explanation surfaces or debug metadata

### Acceptance criteria

- import failures and degraded query paths are inspectable
- no silent false freshness or silent hidden downgrade remains in the touched paths

### Must not do

- swallow errors and continue with ambiguous state

---

# Workstream 5 — Test hardening and cleanup

## Task 5.1 — Add boundary/invariant tests

### Goal

Prove the architecture is enforced, not merely documented.

### Deliverables

Tests for:

- invalid import rejection
- partial import rollback
- repeated ingest idempotency
- projection state transitions
- deterministic merge behavior
- degraded-warning emission
- trace/provenance preservation

### Acceptance criteria

- tests fail before implementation and pass after
- important invariants are not left untested

### Must not do

- rely only on happy-path tests

---

## Task 5.2 — Minimal documentation alignment

### Goal

Update code-level docs/comments so the enforced architecture is visible to future contributors.

### Deliverables

- concise module/crate docs
- import boundary docs
- runtime warning/semantics docs where helpful

### Acceptance criteria

- a future engineer can infer the authority model from the code and docs
- docs match reality closely enough not to become lies immediately

### Must not do

- write aspirational novel-length docs detached from implementation

---

# Final sequencing guidance

Preferred execution order:

1. Task 1.1
2. Task 1.2
3. Task 2.1
4. Task 2.2
5. Task 2.3
6. Task 3.1
7. Task 3.2
8. Task 3.3
9. Task 4.1
10. Task 4.2
11. Task 5.1
12. Task 5.2

If the repo structure reveals a better local sequence, preserve the same dependency logic:

- boundaries and types first,
- import discipline second,
- runtime semantics third,
- observability fourth,
- tests and cleanup throughout.

