# Phased execution plan for forge-pilot

## Phase 0 — surface and crate law

### Deliverables

- add crate manifest,
- add workspace wiring,
- add config/error/type skeleton,
- freeze default feature set (no LLM required).

### Exit gate

`cargo test -p forge-pilot` can run even if most tests are still placeholders.

---

## Phase 1 — observation reconstruction

### Deliverables

- runtime advisory reads,
- projection import log reads,
- kernel payload parse + compile + schedule,
- public projection queries,
- `ScopeHealthSummary`.

### Exit gate

Observation fixture tests prove deterministic reconstruction and explicit degradation.

---

## Phase 2 — targeting and decision

### Deliverables

- target taxonomy,
- stable target keys,
- scoring + retry decay,
- deterministic candidate selection,
- optional feature-gated LLM refinement adapter.

### Exit gate

Scoring tests prove ordering, exhaustion, and halt threshold behavior.

---

## Phase 3 — act + bundle + canonical roundtrip

### Deliverables

- kernel-oracle act path,
- paired patch act path,
- local bundle builder,
- canonical export/import glue.

### Exit gate

At least one oracle plan and one patch plan both produce importable loop outputs.

---

## Phase 4 — loop runner and history

### Deliverables

- in-memory target history,
- loop runner with cooldown and budgets,
- loop report,
- CLI or JSON report surface.

### Exit gate

A fixture namespace can run through multiple bounded iterations and halt honestly.

---

## Phase 5 — conformance, docs, optional polish

### Deliverables

- full end-to-end roundtrip tests,
- degradation honesty tests,
- Makefile target,
- truthful docs,
- optional LLM refinement wired behind feature flag.

### Exit gate

All conformance gates are green and the surface docs tell the truth.
