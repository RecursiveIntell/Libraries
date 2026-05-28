# Claude Code Implementation Prompt

You are implementing the next phase of a Rust AI/knowledge stack.

You must treat the accompanying files as the authoritative brief for this task:

- `01_PROJECT_CONTEXT_BRIEF.md`
- `02_PHASE_SPEC.md`
- `03_TASK_PLAN.md`
- `06_ACCEPTANCE_CHECKLIST.md`

Your job is to make the architecture described there **true in code**.

---

## Mission

Implement the invariant-enforcement phase for the seam between:

- `semantic-memory`
- `knowledge-runtime`
- `semantic-memory-forge` / Forge
- the bridge/import layer between Forge exports and memory projections

The objective is **not** to redesign the architecture.
The objective is to:

- enforce boundaries,
- make invalid states harder or impossible,
- preserve provenance,
- make query behavior explicit,
- and prove the invariants with tests.

---

## Operating rules

### 1. Do not redesign unless forced by code reality

Prefer the smallest change set that makes the architecture operationally true.

Do not invent major new subsystems unless the current structure makes the required boundary impossible otherwise.

### 2. Preserve crate authority boundaries

You must preserve the following doctrine:

- `semantic-memory` owns queryable knowledge persistence and storage-facing projection writes.
- Forge owns raw verification truth and verification semantics.
- `knowledge-runtime` owns planning, retrieval composition, scoped resolution, merge policy, and warnings.
- the bridge/import layer owns projection transformation/import mediation.

Do not let any crate silently absorb another crate’s authority.

### 3. Prefer explicitness over “helpful” ambiguity

Use:

- explicit types,
- explicit enums,
- explicit warnings,
- explicit error paths,
- explicit metadata.

Reject hidden magic and silent downgrade behavior.

### 4. Add tests for every invariant you enforce

Every meaningful behavior change in this phase should either add or strengthen tests.

Do not consider the work complete if the invariant is only reflected in comments/docs.

### 5. Keep runtime behavior explainable

Where retrieval/merge behavior is changed or clarified, preserve or improve the system’s ability to explain:

- which route/leg produced a result,
- what projection/import state it came from,
- whether behavior degraded,
- how duplicates were fused,
- and why ranking/order emerged.

---

## Required outcomes

You should complete as many of the following as the codebase structure allows, with correctness prioritized over breadth:

1. Consolidate canonical identity/provenance types or wrap them cleanly.
2. Introduce/formalize a narrow projection-import boundary.
3. Add transactional, storage-oriented projection import API to `semantic-memory`.
4. Implement idempotent import semantics.
5. Extend projection lifecycle/freshness state.
6. Tighten runtime route/merge semantics and degraded warnings.
7. Normalize trace/provenance propagation.
8. Add tests proving boundary, import, and runtime invariants.
9. Update code-level docs/comments where needed to reflect actual enforced behavior.

---

## Specific anti-patterns to avoid

Do **not** do any of the following:

- put Forge-specific policy logic into `semantic-memory`
- let `knowledge-runtime` become authoritative durable storage
- implement hidden cross-store joins that mix stale projections with fresh raw receipts in default user-facing paths
- swallow import failures and expose partial visibility
- add convenience APIs that bypass the intended importer boundary
- use ambiguous/brittle dedupe keys for repeated ingest
- silently downgrade scope or temporal behavior without warnings
- create large speculative abstractions with no concrete benefit
- change unrelated modules just because they are nearby

---

## Implementation style

- Favor narrow, high-signal changes.
- Preserve existing crate identities.
- Keep public APIs intention-revealing.
- Make compile-time structure do as much enforcement as practical.
- Where compile-time enforcement is not realistic, add strong runtime validation and tests.
- Add terse comments/docstrings for the boundary rules where future drift is likely.

---

## Output expectations

When you work:

1. First inspect the relevant crates/modules and identify the current seam.
2. Then implement according to the phase spec and task plan.
3. State any unavoidable deviations from the plan explicitly.
4. Add/update tests.
5. Summarize exactly which invariants are now enforced.
6. List any remaining gaps honestly.

Do not claim completion if major invariants remain only documented and not enforced.

