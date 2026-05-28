# Canonical Stack Spec v9 — Episodic Authority and Execution Evidence

**Status:** Proposed canonical supplement  
**Supersedes:** No prior episode-first / execution-evidence supplement  
**Relationship to v6:** v6 remains the law of authority, temporality, import discipline, retry lineage, and anti-shadow-database behavior.  
**Relationship to v7:** v7 remains the law of recursive inference execution, witnesses, certificates, residuals, syndromes, and bounded oracle behavior.  
**Relationship to v8:** v8 remains the law of governed verification, policy, adjudication, approval, rollback, and replayable control decisions.  
**Scope:** `stack-ids`, `semantic-memory-forge`, `forge-memory-bridge`, `semantic-memory`, `knowledge-runtime`, `llm-tool-runtime`, `forge-pilot`, `verification-*`, `contract-schema-gen`, and any satellite execution surface that can create, relay, replay, or budget epistemically relevant work.

---

## 0. Purpose

This document defines the next target state for the stack after:

- v6 made the authority and time law explicit,
- v7 made recursive inference execution explicit,
- v8 made governed verification explicit,
- and the codebase grew enough that **episode identity and execution conditions** can no longer remain partly inferred from surrounding structure.

v9 exists to do nine things:

1. make **episode identity** first-class across the canonical planes;
2. freeze one canonical **episode / claim / evidence package contract**;
3. make **execution context** a versioned artifact family rather than telemetry exhaust;
4. require runtime queries to disclose the **view model** they used;
5. formalize **bridge invariants and atomic import** for episode-bearing packages;
6. define explicit **repair / contradiction law** for downstream corrections;
7. require a **verification-plan artifact** for risk-bearing outputs;
8. make closed-loop orchestrators explicitly **consumer-only** with respect to truth; and
9. turn schema evolution and reference semantics into a **governed conformance surface**.

The end state is not “more orchestration.”  
The end state is a stack where you can answer all of the following without archaeology:

- what episode this claim came from,
- which execution conditions produced it,
- what view model surfaced it,
- what cheap checks remain,
- what contradiction or repair touched it,
- and why the system did not get to silently improvise semantics on the way through.

### 0.1 Design basis (non-normative)

This supplement is synthesized from:

- `CANONICAL_STACK_SPEC_V6.md`
- `CANONICAL_STACK_SPEC_V7_RECURSIVE_INFERENCE_KERNEL.md`
- `canonical_stack_spec_v_8_verification_control_plane.md`
- the current workspace structure,
- the `semantic-memory` episode migration and episode-cause normalization seams,
- the `living-memory` export surfaces,
- the `verification-*` crates,
- `forge-pilot`,
- `contract-schema-gen`,
- and the currently loaded design corpus around temporal truth, causal verification, execution evidence, contract governance, and bounded repair.

The design basis compresses to one sentence:

> v6 made truth lawful, v8 made control lawful, and v9 makes **episode-bearing evidence plus execution conditions** lawful.

### 0.2 Normative language

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, and **MAY** are to be interpreted as normative requirements.

Where this document distinguishes between:

- **logical model** — required semantics, artifact classes, invariants, provenance, and transitions,
- **physical model** — concrete tables, crates, adapters, jobs, APIs, schemas, and storage layouts,

…the logical model is mandatory. The physical model is flexible only where this document explicitly allows flexibility.

---

## 1. Enduring doctrine carried forward from v6, v7, and v8

Nothing in this supplement weakens the prior laws below.

### 1.1 Authority remains asymmetric

- Forge and import boundaries remain authoritative for truth-bearing promotion.
- `semantic-memory` remains the authoritative queryable projection layer.
- Runtime, kernel, pilot, tool-runtime, and control layers remain non-authoritative with respect to domain truth.
- Advisory or orchestration artifacts MUST NOT self-promote into truth.

### 1.2 Time remains part of meaning

- Valid time and recorded time remain distinct.
- Append-plus-supersession remains the required mutation model for truth-bearing state.
- Repair, rollback, and contradiction artifacts MUST preserve the time coordinates of the state they refer to.

### 1.3 Execution lineage remains evidence

Retries, replay linkage, queue hops, deadlines, workload class, budget exhaustion, provider/tool route, and degradation markers remain part of the evidence model. They are not “just ops.”

### 1.4 Verification outranks inference, and both outrank convenience

- No inferred structure may outrank explicit verification state.
- No control or runtime shortcut may erase cheap-check obligations.
- “The model agreed with itself repeatedly” remains non-authoritative theater.

### 1.5 No shadow database rule still applies

- Runtime MUST NOT become a second durable truth store.
- The control plane MAY be authoritative for control history only.
- Execution evidence artifacts MAY be authoritative for execution history only.
- Hidden mutable state that cannot be reconstructed from canonical artifacts plus declared policy is forbidden.

### 1.6 Learning remains downstream of verified outcomes

Calibration, abstention, ranking, and scheduling may improve from replayable outcomes.  
They MUST NOT mutate truth semantics, rewrite episode identity, or silently reinterpret prior evidence.

---

## 2. What v9 adds

v9 adds three tightly related things:

1. **episode-first artifact law**  
2. **execution-evidence artifact law**  
3. **multi-view runtime law**

The combination matters. A first-class episode with no execution context is incomplete provenance.  
A first-class execution receipt with no episode identity is a very precise orphan.  
A runtime that uses multiple views without declaring them becomes a plausible nonsense engine.

v9 therefore makes these inseparable.

---

## 3. Episode identity law

### 3.1 Episode identity is first-class

An **episode** is the canonical identity-bearing unit for a coherent evidentiary situation, experimental situation, causal situation, or operational event that may give rise to claims, relations, verification work, or repair work.

An episode MUST have a stable `episode_id`.

An episode MUST NOT be treated as identical to its `document_id`.

A document MAY still be:

- the primary textual container for an episode,
- one source among many for an episode,
- or a projection surface that references one or more episodes.

But the document is not the authoritative identity story.

### 3.2 Multiple episodes per document are canonical

The stack MUST support multiple episodes per document without treating that as an edge case.

Legacy single-episode-per-document data MAY exist.  
Where legacy data exists, compatibility mapping MAY assign a deterministic legacy `episode_id`, but that compatibility mapping MUST be explicit and temporary.

### 3.3 Episode identity must survive every canonical seam

The following surfaces MUST preserve episode identity:

- Forge export surfaces,
- bridge transforms,
- import batches,
- projected memory state,
- runtime query provenance,
- verification/control artifacts when applicable,
- repair records when applicable.

Episode identity MUST NOT be reconstructed from free text, document path, or best-effort matching when it was already available in a canonical upstream artifact.

### 3.4 Episode causes are normalized structure

A list of causes stored only as an opaque JSON blob is insufficient as the canonical long-term story.

The stack MUST support normalized, queryable causal backlinks for episodes.  
Materialized or compatibility fields MAY still exist for convenience, but normalized causal structure is the conformance surface.

---

## 4. Canonical episode package law

### 4.1 One package contract is required

The stack MUST define one versioned cross-plane package contract for episode-bearing evidence.  
This document refers to that logical artifact family as **`EpisodeBundleV1`**.

The exact Rust type and file path MAY vary, but the logical contract is mandatory.

### 4.2 `EpisodeBundleV1` minimum contents

`EpisodeBundleV1` MUST include, at minimum:

- `bundle_id`
- `episode_id`
- `primary_document_id` or equivalent primary attachment
- `namespace`
- `scope_key` where applicable
- valid-time coordinates where applicable
- recorded/exported-at coordinates
- content digest(s) for package integrity
- source evidence pointer set
- source receipt digest set where applicable
- claim version payloads or references
- relation version payloads or references
- verification summary payloads or references
- refutation payloads or references
- control-plane references where applicable
- execution context or a strong reference to it
- thin-export / degraded-export markers
- supersession / replacement lineage where applicable

### 4.3 Canonical package law

There MUST be one canonical logical package contract even if:

- Forge stores richer internal forms,
- runtime uses thinner derived forms,
- or compatibility adapters still exist.

Thin adapters are permitted.  
Competing truth-bearing package contracts are forbidden.

### 4.4 Claim and relation lineage must backpoint through the bundle

Every imported claim version or relation version that came through the canonical lane MUST be able to answer, through canonical fields or backpointers:

- which `EpisodeBundleV1` it came from,
- which `episode_id` it came from,
- which export or source envelope it came from,
- and which evidence or receipt digests materially produced it.

---

## 5. Execution context as artifact law

### 5.1 Execution context is meaning-bearing provenance

For relevant artifacts, the execution conditions under which evidence or control outputs were produced are part of semantics, not just observability.

This document refers to the logical artifact family as **`ExecutionContextV1`**.

### 5.2 `ExecutionContextV1` minimum contents

`ExecutionContextV1` MUST include, at minimum:

- `trace_ctx` or equivalent root trace identity
- logical `attempt_id` / attempt-family identity
- concrete `trial_id` where trials exist
- replay linkage where applicable
- workload class
- queue-hop lineage where applicable
- deadline or remaining-time budget
- cost budget lineage where applicable
- degradation markers
- dispatch outcome or failure taxonomy
- environment or execution-scope fingerprint sufficient for replay interpretation
- provider/model/tool route where applicable
- cancellation or exhaustion reason where applicable

### 5.3 Embedded or strongly referenced

Relevant artifacts MUST either:

- embed `ExecutionContextV1`, or
- carry a strong, versioned reference to it that is queryable through canonical surfaces.

“Look in logs” is not a conforming answer.

### 5.4 Relevant artifacts

At minimum, the following artifact families MUST carry or strongly reference execution context:

- episode bundles,
- verification attempts,
- tool receipts,
- control receipts,
- repair records,
- import failure records,
- replay records,
- runtime query provenance records for degraded or widened queries.

---

## 6. Verification-plan artifact law

### 6.1 Risk-bearing outputs require a plan artifact

Any output that may materially influence:

- promotion,
- rollback,
- quarantine,
- operator trust,
- escalated review,
- or priority/ranking for further work

MUST emit a **verification-plan artifact**.

### 6.2 Minimum contents

A conforming verification-plan artifact MUST include:

- cheapest known admissible checks
- replay recipe or replay preconditions
- blocked checks and why they were blocked
- refutation or falsification suggestions
- degradation flags
- policy or approval blockers
- expiration or obsolescence conditions

### 6.3 Result-only outputs are non-conforming

A scalar score, ranked item, synthesized answer, or kernel output without a checkable plan MAY remain advisory-only, but it MUST NOT cross a risk-bearing or promotion-bearing boundary.

---

## 7. Multi-view runtime law

### 7.1 Runtime uses declared views, not one blended truth soup

The runtime MUST treat at least the following as distinct query surfaces:

- **semantic view**
- **temporal view**
- **entity / alias view**
- **causal view**
- **control view**

These MAY be physically backed by shared tables or compiled projections, but they are semantically distinct and MUST NOT be silently collapsed.

### 7.2 Query plans must declare view use

A canonical runtime query plan MUST disclose:

- requested view(s)
- widening policy
- valid-time coordinates
- recorded-time coordinates
- entity-resolution mode
- whether audit-only evidence dereference was permitted
- degradation markers

### 7.3 Widening is a policy event

The runtime MUST disclose any widening step such as:

- exact entity -> alias candidate set
- alias candidate set -> semantic neighborhood
- exact time slice -> bounded time window
- verified-only -> advisory-inclusive
- canonical view -> fallback view

Silent widening is forbidden.

### 7.4 View provenance is part of the result

Result provenance MUST make it possible to answer:

- which view supplied a result,
- whether the result arrived through widening,
- which episode or bundle anchored the result,
- and whether any control/repair state materially affected ranking or eligibility.

---

## 8. Bridge invariants and atomic import law

### 8.1 The bridge still transforms and does not invent

The bridge MAY normalize, repackage, validate, and deterministically map structures.  
It MUST NOT invent:

- new episode identity,
- new claim semantics,
- new relation semantics,
- new promotion state,
- new verification state absent in the source package.

### 8.2 Digest and backpointer preservation

At the canonical boundary, imports MUST preserve:

- bundle identity,
- episode identity,
- source envelope identity,
- source digest(s),
- and backpointers from imported projections to source bundle identity.

### 8.3 Atomic import is mandatory

Canonical import of a bundle-bearing package MUST be all-or-nothing.

Partial visibility of:

- some claim versions without their bundle identity,
- some relation versions without their originating episode,
- or some package members without their execution context

is forbidden.

### 8.4 Import failure is itself an artifact

Failed canonical imports MUST emit a typed, replayable **import failure record** rather than disappearing into logs.

---

## 9. Repair / contradiction law

### 9.1 Contradictions become explicit repair surfaces

Contradictions, drift, invalid lineage, stale promotion, broken alias assumptions, and temporal reinterpretations MUST NOT be handled as ad hoc “cleanup.”

They MUST emit typed repair or contradiction artifacts.

### 9.2 `RepairRecordV1` minimum contents

A conforming repair record MUST include:

- `repair_record_id`
- affected identity set (`episode_id`, claim version IDs, relation version IDs, or equivalent)
- repair class
- trigger artifact(s)
- blast radius classification
- reversibility classification
- action taken or proposed
- supersession / rollback / quarantine linkage where applicable
- execution context
- opened-at and resolved-at coordinates
- explicit statement of what did **not** change

### 9.3 Required repair classes

At minimum, the stack MUST distinguish repair classes equivalent to:

- identity repair
- temporal repair
- supersession repair
- rollback repair
- bundle / import repair
- scope / widening repair
- verification-state repair

### 9.4 Minimal-change rule

A repair action MUST prefer the smallest admissible change set that restores invariants.  
If a larger blast radius is chosen, the artifact MUST say why.

---

## 10. Closed-loop orchestrator law

### 10.1 Orchestrators are consumer-only with respect to truth

A closed-loop orchestrator such as `forge-pilot` MAY:

- observe public surfaces,
- propose work,
- choose among admissible plans,
- execute through canonical tool/runtime seams,
- emit receipts and reports,
- request export/import on lawful seams.

It MUST NOT:

- invent episode identity,
- directly mutate authoritative truth,
- bypass policy or approval law,
- or create a hidden second control model that outranks canonical artifacts.

### 10.2 Stop rules and escalation rules are mandatory

A conforming orchestrator MUST declare:

- stop rules
- escalation rules
- budget classes
- retry limits
- cooldown or damping behavior where loops are possible
- degradation or abstention behavior

### 10.3 Orchestrator outputs are auditable artifacts

Loop reports, iteration reports, target history, exhaustion state, and exported action traces MUST be queryable artifacts or strongly referenced by them.

---

## 11. Schema governance and reference-interpreter law

### 11.1 All wire-visible v9 artifacts require canonical schemas

`contract-schema-gen` or an equivalent canonical owner MUST emit schemas for all wire-visible v9 artifact families, including at minimum:

- episode bundle
- execution context
- verification-plan artifact
- repair record
- import failure record
- runtime query provenance
- all still-active control-plane artifacts

### 11.2 Compatibility policy is mandatory

Schema evolution MUST declare:

- additive vs breaking change class,
- compatibility window,
- migration owner,
- and required proof artifacts.

### 11.3 Reference interpreters are required for the hard semantic seams

Some behaviors are too semantic to police with shape validation alone.

The stack MUST therefore provide executable reference behavior for at least:

- valid-time / recorded-time query semantics,
- view widening semantics,
- bridge atomicity invariants,
- and repair-record invariants.

A prose promise is not enough.

---

## 12. Canonical placement update

### 12.1 v9 does not add a new truth plane

v9 does not create a ninth secret truth plane.  
It sharpens the responsibilities of the existing planes and adds canonical artifact families that cross them lawfully.

### 12.2 Updated placement matrix

| Layer | Canonical owners | Responsible for | MAY write | MUST NOT write |
|---|---|---|---|---|
| **Primitive plane** | `stack-ids` | IDs, digests, traces, replay links | opaque primitives | domain semantics |
| **Evidence plane** | `semantic-memory-forge`, `living-memory` canonical lane | raw truth-bearing evidence, export packages, refutations | authoritative raw/export artifacts | silent control or runtime semantics |
| **Bridge plane** | `forge-memory-bridge` | deterministic package -> import transform | import batches, import failure artifacts | invented semantics |
| **Projection plane** | `semantic-memory` | imported queryable state, episode identity, derivation edges | projected truth, lineage, normalized episode causes | raw evidence reinterpretation |
| **Runtime plane** | `knowledge-runtime` | declared multi-view query planning and merge | query plans, result provenance, degradation disclosures | durable truth mutation |
| **Inference plane** | compiler / kernel crates | witnesses, certificates, residuals, syndromes, operator outputs | rebuildable derived artifacts | authoritative truth |
| **Control plane** | `verification-*`, `forge-pilot` control outputs | case law, policy, approval, adjudication, control receipts | control history, decision summaries | hidden promotion or hidden rollback |
| **Execution-evidence lane** | `llm-tool-runtime` + satellites + referenced receipts | execution context, dispatch outcomes, queue/replay lineage | execution provenance artifacts | domain truth decisions |

---

## 13. Required crate responsibilities for v9

### 13.1 `stack-ids`
MUST remain the canonical home for any new shared IDs required by the v9 artifact family.

### 13.2 `semantic-memory-forge`
MUST own or co-own the canonical episode-bearing export package lane and MUST preserve episode identity and source digests.

### 13.3 `living-memory`
MAY still hold local authoring structures, but any divergence from the canonical episode package contract MUST be explicit, thin, and round-trip tested.

### 13.4 `forge-memory-bridge`
MUST prove atomic import, digest preservation, and backpointer preservation.

### 13.5 `semantic-memory`
MUST expose first-class episode identity, normalized episode-cause structure, and queryable backpointers into source package lineage.

### 13.6 `knowledge-runtime`
MUST implement declared view usage, explicit widening, and query provenance disclosure.

### 13.7 `llm-tool-runtime`
MUST serve as a canonical provider/tool receipt seam rather than a convenience wrapper with disappearing lineage.

### 13.8 `verification-control`, `verification-policy`, `verification-calibration`, `verification-adjudication`
MUST integrate with the v9 episode and execution-evidence families rather than remaining a separate governance island.

### 13.9 `forge-pilot`
MUST remain a thin orchestrator over public surfaces and must not create a shadow truth lane.

### 13.10 `contract-schema-gen`
MUST own schema publication and compatibility proof for v9 artifact families.

---

## 14. Migration and compatibility

### 14.1 Compatibility is allowed, but not the mental model

Legacy document-keyed episode behavior MAY be preserved temporarily.  
It MUST be documented as compatibility behavior, not as the canonical v9 story.

### 14.2 Compatibility windows must be finite

Every legacy compatibility path MUST have:

- an owner,
- an exit criterion,
- a removal window,
- and explicit proof that the new canonical path is covering the same semantics.

### 14.3 The old story must not remain the taught story

Examples, docs, fixtures, and walkthroughs MUST teach the canonical episode-first lane once it exists.

---

## 15. Explicit non-goals for v9

The following are intentionally deferred:

- full multi-graph execution geometry,
- time-aware differential runtime maintenance,
- small communicating region scheduling as the canonical runtime shape,
- generalized syndrome-first repair routing,
- nuisance-state subgraph execution,
- ambitious learned triage above advisory-only status.

Those belong to the v10 horizon unless a narrow slice can be proven without distorting the v9 finish line.

---

## 16. Conformance headline

A system conforms to v9 only if it can answer, for a materially important artifact:

1. **Which episode did this come from?**
2. **Which execution conditions produced it?**
3. **Which bundle or package carried it across the canonical boundary?**
4. **Which runtime view surfaced it?**
5. **Which cheap checks remain?**
6. **Which repairs or contradictions touched it?**
7. **What prevented silent semantic invention on the way through?**

If the system cannot answer those questions with canonical artifacts, it is not yet a v9 system.
