# Master Issue Matrix — Codex V7

**Source snapshot:** current library snapshot plus the uploaded control docs, canonical spec, research set, and the static completion audit.  
**Method:** full-tree static synthesis across the core authority lane plus research-to-implementation mapping.  
**Execution target:** Codex handoff for a **truthful completion pass**, not a redesign pass.  
**Important caveat:** this remains a **static inspection grounded in source and documents**. No successful full-workspace build or runtime verification was executed in this environment.

## Why V7 exists

V6 is no longer sufficient as the active issue matrix because:

- several V6 P0 items are already substantially landed,
- the status language in older docs is now partially stale,
- the dominant blocker has shifted to **temporal proof coverage** and **derivation/invalidation breadth**,
- the broader research lane has clarified missing work around
  **temporal query semantics**, **identity integrity**, **trace/retry economics**, and **parser/patch hardening**.

## Snapshot verdict

- **Core Codex closure:** ~78/100
- **Broader research-program closure:** ~35–40/100
- **Main blocker:** `TMP-102` temporal integrity checks now have explicit importer-path coverage; next blocker is `OBS-102` / `EXE-101` economics and retry policy hardening
- **Main risk after that:** incomplete temporal integrity proofs and compatibility seams that can drift.

## Status legend

- **Landed** — functionally in place; keep only if tightening
- **Mostly landed** — real and useful, but bounded or incomplete
- **Partial** — meaningful progress exists, but closure is not honest yet
- **Open** — still a major gap
- **Deferred** — real idea, deliberately not on the critical path

## Priority summary

- **P0:** do now; current truth blockers
- **P1:** immediate closure work after P0
- **P2:** important hardening that must land before honest “complete”
- **P3:** control-plane / hygiene / secondary lane closure
- **P4:** explicitly deferred research lane

---

## Core authority lane matrix

| ID | Pri | Status | Area | Issue | Required fix / acceptance |
|---|---|---|---|---|---|
| BRG-101 | P0 | Landed | Bitemporal store law | Authoritative `recorded_at` now belongs in the importing store, not the bridge. Keep it that way and prevent regressions. | Acceptance: imported rows keep store commit time as authoritative `recorded_at`; bridge provenance remains separate and named separately. |
| SM-101 | P0 | Landed | Projection query surface | Public projection read APIs now exist. The risk is regression or bypass. | Acceptance: runtime/adapters continue using supported projection query APIs rather than fake fact/chunk paths. |
| KR-101 | P0 | Mostly landed | Runtime retrieval | Imported projections are now a real retrieval substrate on supported routes. | Acceptance: proof tests continue passing without `add_fact()` scaffolding; imported claim/relation/episode answers remain query-visible end-to-end. |
| KR-102 | P0 | Mostly landed | Temporal retrieval | Supported projection routes now perform real valid-time and recorded-time filtering, but the surface is still narrower than the full target state. | Acceptance: `strict_temporal` succeeds on supported projection routes, degrades only on truly unsupported ones, and `as_of(valid_t, recorded_t)` semantics are explicit. |
| KR-103 | P0 | Mostly landed | Scope enforcement | Projection-backed routes enforce more than namespace-only scope, but full truthfulness still needs to become explicit policy. | Acceptance: domain/workspace/repo filters are guaranteed on supported routes and visible degradation remains on unsupported ones. |
| LIV-101 | P0 | Landed | Forge export richness | Export now emits a richer causal envelope (claim, relation, entity, episode, evidence) from current Forge data. | Acceptance: export preserves real causal structure, not synthetic-only envelopes. |
| SMF-101 | P0 | Landed | Version lineage | `living-memory` exporter now carries `supersedes_claim_version_id` on claims when the bundle provides it, and never invents lineage. | Acceptance: prior version lineage round-trips where known; no synthetic version IDs are invented. |
| KR-104 | P1 | Partial | Causal projection consumption | Runtime can consume imported causal projections, with route coverage still limited by supported projection-query routes. | Acceptance: at least one supported causal query can be answered from imported causal projections with provenance intact and evidence still opaque by default. |
| KR-105 | P1 | Mostly landed | Entity candidate expansion | Bounded alias-based candidate expansion exists, but it is not yet the final research-grade identity layer. | Acceptance: bounded fuzzy expansion remains explicit, ambiguity is visible, and no authority mutation occurs at query time. |
| SM-102 | P1 | Landed | Derivation breadth | Derivation edges now cover imported claim/relation/episode/evidence flows with bounded invalidation semantics. | Acceptance: imported claim/relation/episode/evidence flows all leave sufficient derivation structure to support bounded recomputation and invalidation. |
| SM-103 | P2 | Partial | Durable trace representation | Durable trace semantics are still not as explicit as the architecture deserves. | Acceptance: declare and test the durable policy (`trace_id` only vs richer `TraceCtx` persistence) and stop implying more than is actually stored. |
| SM-104 | P2 | Partial | Compatibility fencing | Legacy import surfaces still exist and can fossilize by inertia. | Acceptance: active docs/examples/tests teach canonical batch import first; legacy surfaces are visibly fenced as compatibility-only. |
| BRG-102 | P2 | Partial | Bridge compatibility debt | Legacy bridge helpers remain too visible. | Acceptance: compat-only labeling or gating is clear enough that new work cannot accidentally normalize them. |
| DOC-101 | P1 | Landed | Control-plane drift | Root docs and status dashboards are now governed by V7 authority and stale guidance is being archived/fenced. | Acceptance: active control-plane docs are updated together with implemented gaps, with stale artifacts labeled historical. |
| DOC-102 | P2 | Mostly landed | Misleading permanence language | Some docs previously used “will not be” language more absolute than the architecture allowed. | Acceptance: docs describe present implementation and explicit non-goals truthfully, not permanent impossibilities unless the spec truly forbids them. |

---

## Research-governed completion matrix

| ID | Pri | Status | Area | Issue | Required fix / acceptance |
|---|---|---|---|---|---|
| EVD-101 | P1 | Landed | Evidence substrate | A causal claim object with durable evidence bundle, estimator metadata, provenance, and refutation outputs is now productized. | Acceptance: a stored/exported/imported causal claim object exists with treatment, outcome, confounders, estimate metadata, and evidence bundle references. |
| VER-101 | P1 | Landed | Verification harness | Baseline-vs-patched paired trials are now a required proof surface across the stack. | Acceptance: paired verification trials are represented explicitly with stable attempt/trial identity and patch/baseline tagging. |
| VER-102 | P1 | Landed | Refutation / falsification | Placebo, dummy outcome, and subsample-stability outputs are implemented as named verification artifacts with pass/fail semantics. | Acceptance: at least placebo, dummy outcome, and subsample stability are implemented as persisted artifacts with visible outcomes. |
| TMP-101 | P1 | Mostly landed | Bitemporal query surface | `as_of(valid_t, recorded_t)` semantics are now available via `query_temporal(...)` and supported routes can execute full recorded-time cuts; remaining work is coverage parity on all route families. | Acceptance: public query surfaces can express bitemporal as-of semantics directly or through explicit equivalent parameters. |
| TMP-102 | P2 | Mostly landed | Temporal integrity constraints | Temporal integrity rules are now enforced in the importer flow with explicit interval ordering checks, preferred-open overlap checks across batch and DB state, and test coverage for invalid-order/overlap failure modes. | Acceptance: preferred-open / no-overlap / temporal referential-integrity style invariants are encoded in importer checks and regression tests; schema constraints still require periodic re-alignment. |
| IDN-101 | P2 | Partial | Identity integrity | Alias expansion exists, but merge/split provenance and bitemporal identity-state discipline are not fully closed. | Acceptance: alias / merge / split decisions preserve provenance, confidence, and supersession history in a replayable way. |
| OBS-101 | P2 | Partial | Trace / retry law | The secondary lane shows promising lineage support, but end-to-end attempt/trial/trace propagation is not yet fully audited. | Acceptance: every canonical queue hop / retry / replay path has recoverable attempt/trial/trace lineage with one primary retry owner. |
| OBS-102 | P2 | Open | Deadlines, budgets, retry economics | Deadline propagation, retry budgets, and queue-hop truthfulness remain under-specified as release gates. | Acceptance: deadline/budget propagation and retry-budget policy are documented, testable, and visible in emitted evidence. |
| EXE-101 | P3 | Open | Scheduler economics | Verification work classes, fairness, and WIP/budget accounting are not yet part of the control plane. | Acceptance: queue classes, priority policy, and bounded resource accounting are documented and testable. |
| PAR-101 | P3 | Open | Parser / patch reliability | Parser and patch surfaces still need dedicated fuzz/property hardening before “perfection” talk becomes honest. | Acceptance: cargo-fuzz and property tests cover patch idempotence, path traversal bounds, round-trip/repair invariants, and malformed structured outputs. |
| GOV-101 | P3 | Open | Control-plane minimization | Too many historical matrices/docs/prompts remain in the root orbit. | Acceptance: one active control-plane set, one archival area, and no duplicate status dashboards with contradictory truth claims. |
| ML-001 | P4 | Deferred | Learned graph scoring | GNN / learned ranking / mechanistic alarm layers are real research directions, but they are not on the current closure path. | Acceptance: defer until verified labels, calibrated uncertainty, and provenance-complete training data exist. |

---

## Explicitly deferred research lane

These are real ideas, but **not** current blockers for honest completion:

| ID | Status | Why deferred |
|---|---|---|
| RSH-ABACUS-001 | Deferred | Abacus-inspired factor-graph / differential-dataflow / ranking ideas are useful architecture analogies, not immediate blockers. |
| RSH-PHYS-001 | Deferred | Physics algorithm execution-graph analogies reinforce orchestration thinking, but they do not close current stack truth gaps. |
| RSH-BIO-001 | Deferred | Bio workflow / provenance lessons are useful for reproducibility discipline, but direct domain import is not current Codex work. |
| RSH-MECH-001 | Deferred | Mechanistic interpretability / SAE alarms require verified external truth first. |
| RSH-GNN-001 | Deferred | Learned scoring with noisy labels is an attractive way to build a smarter liar. Not yet. |
| RSH-DI-001 | Deferred | Decision-intelligence framing is strategically aligned but does not replace closure of evidence, verification, and identity substrates. |

---

## Recommended sequencing

1. `DOC-101` + install V7 control plane
2. `SM-102`
3. `KR-104` + `EVD-101` + `VER-101` + `VER-102`
4. `TMP-101` + `TMP-102` + `IDN-101`
5. `SM-104` + `BRG-102` + `GOV-101`
6. `OBS-101` + `OBS-102` + `EXE-101`
7. `PAR-101`
8. only then revisit `ML-001` and deferred research imports

Guardrail checks:

- `LIV-101` + `SMF-101` remain active as regression checks to prevent reversion to synthetic-only export.

---

## Bottom line

The V7 matrix treats the current state honestly:

- the architecture is strong,
- the read lane is much more real than older docs admit,
- but the stack is still not “done except for polish.”

The decisive remaining blocker is **control-plane drift**.  
The decisive structural risk is **derivation breadth and invalidation completeness**.  
The decisive research gap is **verification-aware causal rigor as a real subsystem instead of a compelling theory slide**.
