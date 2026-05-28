# Canonical Stack End-State Spec — Recursive, Subtractive, Proof-Governed Runtime

**Status:** Proposed unified end-state canonical spec  
**Intent:** Merge the strongest, non-contradictory laws from the current research corpus into one coherent constitutional document.  
**Relationship to prior materials:** This document integrates and supersedes the architecture split across the v6-v20 stack line *for end-state design purposes*. It does **not** erase the historical value of those documents; it consolidates them into one normative target.  
**Scope:** identity, truth, time, contracts, evidence, execution, recursive inference, lawful subtraction, regional runtime geometry, verification, repair, federation, mechanism search, conformance, and constitutional self-hosting.

---

## 0. Purpose

The stack has crossed the point where the main risk is “missing another good idea.”

The main risk is now **semantic drift across good ideas**:

- truth artifacts and execution evidence drifting apart,
- recursive inference and repair drifting apart,
- subtraction remaining a clever add-on instead of a lawful operator family,
- regions existing without typed boundary law,
- proofs and receipts existing without clear promotion implications,
- and later constitutional automation outrunning the human-governed truth boundary.

This specification defines the end-state target in which all of the following are simultaneously true:

1. **everything materially important is a typed artifact,**
2. **time is part of meaning,**
3. **execution conditions are evidence,**
4. **recursive inference and lawful subtraction are dual operator families,**
5. **contradictions are typed objects, not hidden score drops,**
6. **local regions are the default execution unit,**
7. **approximation is explicit and challengeable,**
8. **promotion is proof-governed,**
9. **federation preserves local authority,** and
10. **the constitution itself can eventually be compiled, checked, replayed, and challenged.**

The shortest honest description is:

> The stack is a **typed artifact machine** that computes, compresses, verifies, repairs, and maintains truth under change by iterating a proof-governed recursion between **inference** and **lawful subtraction**.

---

## 1. Normative language

The words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, and **MAY** are normative.

Where this document distinguishes between:

- **logical model** — required semantics, artifact classes, invariants, and transitions,
- **physical model** — concrete crates, tables, APIs, queues, jobs, wire encodings, or storage engines,

…the logical model is mandatory. The physical model is flexible only where this document explicitly permits flexibility.

---

## 2. Governing doctrine

### 2.1 Evidence before inference
No inference, score, ranking, mechanism, or orchestration output may become more authoritative than the evidence substrate it depends on.

### 2.2 Episode-first identity
The canonical identity-bearing unit is the **episode**, not the document, path, chunk, log line, or projection that currently contains it.

### 2.3 Raw truth, projected truth, execution truth, and advisory judgment are different things
- The evidence plane owns authoritative raw/export truth.
- The projection plane owns authoritative imported/queryable truth.
- The execution-evidence lane owns authoritative execution history only.
- The control plane owns authoritative control history only.
- Advisory systems own proposals, rankings, witnesses, certificates, and abstentions only.

### 2.4 Time is part of meaning
Valid time and recorded/transaction time are distinct and MUST NOT be collapsed.

### 2.5 Append-plus-supersession, never silent rewrite
Truth-bearing state MUST evolve through append, closure, supersession, contradiction state, invalidation metadata, or explicit retirement markers. Silent destructive rewrite is forbidden.

### 2.6 Execution is evidence
Retries, queue hops, deadlines, provider routes, truncations, replay lineage, widening, and degradation are part of meaning, not operational exhaust.

### 2.7 Certificates outrank naked scores
When a subsystem can emit a witness, certificate, refutation, support core, frontier, or semantic diff, that artifact outranks an opaque scalar.

### 2.8 Local repair before global rebuild
If a contradiction or drift localizes to a bounded region, the system MUST prefer local repair or quarantine before global recomputation.

### 2.9 Lawful subtraction is a first-class runtime family
Removal, compaction, forgetting, minimization, and renormalization are conforming only when they are typed, challengeable, receipt-bearing transformations with declared invariant and history budgets.

### 2.10 Explicit widening, never silent approximation
Any widening, fallback, or reduction in guarantees MUST be surfaced as a typed degradation event.

### 2.11 Monotone things may flow without coordination; non-monotone things may not
Monotone, append-only, or commutative artifact families MAY converge under coordination-free replication. Non-monotone surfaces—repairs, retractions, promotions, settlements, contradiction resolution—REQUIRE explicit coordination, approval, or treaty logic.

### 2.12 Learning remains downstream of verified outcomes
Learned scheduling, ranking, calibration, region selection, and proof-economics estimates MAY assist. They MUST NOT rewrite truth, time, identity, causal semantics, or promotion law.

### 2.13 No shadow databases and no shadow constitutions
No runtime, queue, bridge, pilot, notebook, CI-only script, or generated artifact may silently become the real source of truth or the real law.

### 2.14 Human challenge and veto persist
No constitutional or promotion-bearing automation may erase explicit human challenge, rollback, or veto.

---

## 3. Canonical planes

| Plane | Canonical owner(s) | Authoritative for | MUST NOT become |
|---|---|---|---|
| Primitive plane | `stack-ids` | stable IDs, digests, trace primitives, scope keys | a business-logic crate |
| Evidence plane | `semantic-memory-forge`, canonical export lane | raw truth-bearing evidence, bundles, refutations, export envelopes | a runtime planner |
| Bridge plane | `forge-memory-bridge` | deterministic export→import transformation | a policy engine |
| Projection plane | `semantic-memory` | imported queryable truth, derivations, alias state, lineage backpointers | a raw-evidence rewrite layer |
| Runtime plane | `knowledge-runtime` | declared multi-view query planning, merge, explicit widening, result provenance | a durable truth store |
| Kernel plane | constraint compiler / execution / oracle crates | recursive inference, syndromes, residuals, witnesses, support cores, frontiers | an authoritative truth plane |
| Control plane | `verification-*`, approvals, adjudication, rollback lanes | policy, approval, control history, case state | hidden promotion logic |
| Execution-evidence lane | `llm-tool-runtime` + satellites | receipts, retries, queue hops, replay lineage, dispatch outcomes | a domain-truth plane |
| Federation plane | treaty / settlement surfaces | cross-runtime exchange and settlement history | a central truth plane |
| Mechanism plane | theory/mechanism runtime | hypotheses, fit runs, simulator contracts, refuters | truth by elegance |
| Constitutional plane | spec bundles, generators, conformance corpora | generated schemas, reference interpreters, proof obligations, constitutional replay | a self-justifying machine |

---

## 4. Canonical artifact families

The following artifact families are mandatory in the end state.

### 4.1 Identity and bundle families
- `EpisodeBundleV1`
- `ClaimStateBundleV1`
- `RelationStateBundleV1`
- `EvidenceBundleV1`
- `CrossRuntimeEquivalenceBundleV1`

### 4.2 Execution and control families
- `ExecutionContextV1`
- `ToolReceiptV1`
- `ControlReceiptV1`
- `RuntimeQueryProvenanceV1`
- `VerificationPlanV1`
- `DecisionTraceV1`

### 4.3 Kernel and contradiction families
- `ResidualEnvelopeV1`
- `SyndromeEnvelopeV1`
- `WitnessBundleV1`
- `CertificateBundleV1`
- `RefutationBundleV1`
- `RepairCandidateBundleV1`
- `RepairRecordV1`
- `SemanticDiffV1`

### 4.4 Subtraction families
- `SupportCoreV1`
- `RemovalFrontierV1`
- `InvariantPreservationReceiptV1`
- `HistoricalLossBudgetV1`
- `SubtractionRunReportV1`

### 4.5 Regional runtime families
- `RegionContractV1`
- `RegionStateSnapshotV1`
- `RegionBoundaryReceiptV1`
- `DeltaEnvelopeV1`
- `NuisanceEnvelopeV1`
- `BudgetEnvelopeV1`
- `RegionReplaySliceV1`
- `RegionConvergenceReportV1`

### 4.6 Federation families
- `TreatyBundleV1`
- `SettlementCaseV1`
- `SharedDispositionV1`

### 4.7 Mechanism / theory families
- `MechanismBundleV1`
- `TheoryVersionV1`
- `TheoryLibraryV1`
- `HypothesisLibraryV1`
- `TheorySearchProgramV1`
- `SimulationContractV1`
- `FitRunV1`
- `TheoryRefuterSuiteV1`

### 4.8 Constitutional families
- `SpecBundleV1`
- `NormativeASTV1`
- `GeneratedSchemaBundleV1`
- `GeneratedInterpreterBundleV1`
- `GeneratedConformanceCorpusV1`
- `GeneratedMigrationPlanV1`
- `ProofObligationSetV1`
- `SelfHostingBuildReceiptV1`

---

## 5. Canonical semantic product

Every materially important claim-like result MUST be representable as a product of semantic carriers rather than a payload-plus-score.

Let the canonical semantic product be:

\[
\mathcal{S} = P \times T \times B \times I \times N \times E \times V \times X
\]

where:

- **P — provenance/support carrier**
  - alternative support,
  - conjunctive support,
  - recursive support,
  - superseded support,
  - support degraded by widening or boundary repair.
- **T — truth-state carrier**
  - supported,
  - contradicted,
  - underdetermined,
  - both-supported-and-contradicted,
  - superseded,
  - quarantined,
  - degraded-but-queryable.
- **B — bitemporal carrier**
  - valid-time coordinates,
  - recorded-time coordinates,
  - supersession relations,
  - replay coordinates.
- **I — identity carrier**
  - episode identity,
  - claim identity,
  - relation identity,
  - derivation identity,
  - backpointers to canonical bundles.
- **N — nuisance/degradation carrier**
  - stale/incomplete context,
  - execution contamination,
  - replay limitations,
  - retrieval-path contamination,
  - widening/fallback execution,
  - missing-proof state.
- **E — exactness/cost carrier**
  - approximate,
  - bounded exact,
  - replay exact,
  - refuted,
  - exactness unknown,
  - budget class.
- **V — view carrier**
  - semantic,
  - temporal,
  - entity,
  - causal,
  - repair,
  - control.
- **X — execution-evidence linkage carrier**
  - attempt family,
  - trace context,
  - replay handle,
  - queue lineage,
  - provider/tool route,
  - control history linkage.

### 5.1 Algebraic guidance
The product carrier SHOULD be instantiated compositionally:

- provenance as symbolic/circuit or semiring-like support expressions,
- truth state as a bilattice or equivalent contradiction-tolerant truth carrier,
- nuisance and exactness as explicit lattices or bounded enums,
- time as bitemporal scoping plus append-only history,
- subtraction artifacts as derived but first-class members of the overall semantic state.

### 5.2 Retraction law
Retraction MUST be represented durably as supersession plus typed difference/rederivation semantics. Internal negative deltas are allowed; durable silent erasure is forbidden.

---

## 6. Core recursion law: inference + subtraction

### 6.1 Canonical primitive
The end-state runtime MUST expose a first-class primitive referred to here as:

`ClosureMinimizationLoopV1`

Its purpose is to compute the **minimal stable explanation** of evidence under constraints and budgets.

### 6.2 Canonical stages
A conforming loop MUST support the following stages, even if some are fused in implementation:

1. **Infer** — expand support, propagate messages, compile/update candidate structure.
2. **Residualize** — compute syndromes, residuals, witness candidates, and contradiction surfaces.
3. **Subtract** — remove non-load-bearing structure while preserving declared invariants and history budgets.
4. **Verify / Refute** — run cheap checks, replay checks, oracle slices, or falsification suites.
5. **Repair / Quarantine / Roll back** — perform minimal-change local correction or explicit containment.
6. **Propagate deltas** — transmit only affected changes across regions and projections.
7. **Govern** — apply stop rules, budget rules, promotion rules, and degradation disclosure.

### 6.3 Fixed-point condition
The loop reaches a governed fixed point when all of the following hold:

- no admissible inference operator can add promotable structure without changing declared evidence or budgets,
- no admissible subtraction operator can remove additional structure without violating declared invariants or history guarantees,
- no required contradiction surface remains unresolved, quarantined, or explicitly downgraded without being recorded,
- and all stop rules have been satisfied or explicit degradation has been emitted.

### 6.4 Default mathematical interpretation
The stack SHOULD treat the loop as a coupled fixed-point problem over expansion and contraction operators rather than a one-way derivation engine.

Non-normative but strongly relevant analogs include:

- self-consistent fixed-point iteration with acceleration or mixing,
- abstract interpretation with widening and narrowing,
- differential / incremental fixed-point maintenance,
- residual-correction and iterative refinement,
- decoder-style message passing plus bounded exact adjudication,
- and reverse-data-management style minimal interventions.

### 6.5 Acceleration and damping
A conforming implementation MAY use acceleration strategies such as damping, residual prioritization, Anderson-style mixing, or other fixed-point accelerators **only if**:

- the acceleration is disclosed in convergence artifacts,
- the exactness class remains explicit,
- and bounded exact or stricter slice checks remain available for challengeable outputs.

### 6.6 No fake finality
No loop may emit a final-looking result while suppressing known oscillation, stop-rule exhaustion, missing receipts, or unresolved required refutation obligations.

---

## 7. Operator family law

Every operator family MUST publish a semantic contract containing at least:

- operator identity and version,
- input artifact families,
- output artifact families,
- monotonicity expectations if any,
- exactness class,
- widening/narrowing behavior,
- convergence behavior if iterative,
- witness / residual / receipt artifacts emitted,
- checker or oracle path,
- budget class,
- and non-goals / unsupported conditions.

### 7.1 Mandatory operator families
At minimum the stack MUST define semantic contracts for:

- retrieval and view-selection operators,
- message-passing / factor / hypergraph operators,
- region-consensus operators,
- subtraction/minimization operators,
- contradiction detection operators,
- repair operators,
- verification/refutation operators,
- bridge/import operators,
- and mechanism search / fit operators.

### 7.2 Residuals are first-class
Residuals and syndromes are not debugging scraps. They are canonical outputs that guide scheduling, escalation, repair, and proof obligations.

### 7.3 Oracle-slice law
Every approximate recursive operator family MUST define at least one bounded exact or stricter adjudication path (“oracle slice”).

### 7.4 Widening and narrowing law
Recursive operators MAY widen to force convergence and MAY narrow to regain precision. They MUST NOT widen away contradiction visibility.

---

## 8. Episode and execution evidence law

### 8.1 Episode identity is first-class
Episodes MUST have stable identities and MUST survive export, bridge, import, runtime, repair, settlement, and replay seams.

### 8.2 Documents are containers, not canonical identity
A document MAY contain one or more episodes. A document ID MUST NOT be treated as the authoritative episode identity.

### 8.3 Canonical package contract
`EpisodeBundleV1` is the minimum lawful cross-plane package for episode-bearing evidence.

It MUST include at minimum:

- `bundle_id`,
- `episode_id`,
- namespace/scope,
- valid-time and recorded/exported-at coordinates,
- content digests,
- source evidence pointers,
- claim and relation payloads or refs,
- verification summaries or refs,
- execution context or strong ref,
- thin/degraded markers,
- supersession lineage.

### 8.4 Execution context is meaning-bearing provenance
`ExecutionContextV1` MUST at minimum record:

- trace context,
- attempt family and attempt identity,
- replay linkage,
- workload class,
- queue hops,
- deadlines/budgets,
- provider/tool route,
- degradation markers,
- environment fingerprint,
- dispatch outcomes and failure taxonomy.

### 8.5 Relevant artifacts MUST embed or strongly reference execution context
At minimum:

- episode bundles,
- tool receipts,
- verification attempts,
- repair records,
- runtime query provenance,
- and settlement cases

MUST embed or strongly reference execution context.

### 8.6 Execution evidence is admissible history
Execution-evidence artifacts MAY be authoritative for execution history only. They MUST NOT directly write domain truth.

---

## 9. Bitemporal truth and lawful subtraction

### 9.1 Bitemporal law
Every truth-bearing artifact MUST preserve:

- `valid_from`, `valid_to` (inclusive/exclusive),
- `recorded_from`, `recorded_to` (inclusive/exclusive),
- UTC timestamps,
- open-ended semantics,
- append-plus-supersession behavior.

### 9.2 Query law
The system MUST be able to answer, with canonical artifacts:

- what is true now,
- what was believed at record time R,
- what was true at valid time V as of record time R,
- and which execution/evidence conditions were admissible at that time.

### 9.3 Lawful subtraction law
Subtraction is conforming only when it is:

- tied to declared invariants,
- tied to declared historical-query guarantees,
- receipt-bearing,
- replay-aware,
- challengeable,
- and non-destructive at the durable truth boundary.

### 9.4 SupportCore law
A `SupportCoreV1` is a minimal retained structure sufficient to preserve a declared claim or invariant set.

The stack MUST allow:

- a single core,
- a sampled family of cores,
- or a factorized/core-basis representation.

No implementation may assume there is only one meaningful minimal core.

### 9.5 RemovalFrontier law
A `RemovalFrontierV1` is the minimal or near-minimal set of removals/perturbations that breaks a declared claim or invariant.

It MUST carry:

- frontier size or cost,
- affected claim/invariant IDs,
- blast-radius estimate,
- exactness class of the frontier computation,
- and whether the frontier is one member of a larger family.

### 9.6 InvariantPreservationReceipt law
Every conforming subtraction step MUST emit an `InvariantPreservationReceiptV1` that states:

- what was removed,
- which invariant set was checked,
- which query families and time windows remain guaranteed,
- what replay equivalence class is preserved,
- what exactness class the receipt has,
- and how to verify or challenge it.

### 9.7 HistoricalLossBudget law
`HistoricalLossBudgetV1` MUST make forgetting an explicit contract.

It MUST specify:

- preserved query families,
- preserved time windows,
- dropped evidence classes or detail levels,
- replay guarantees retained,
- and what becomes intentionally unspecified.

### 9.8 Retraction implementation guidance
A conforming implementation MAY realize subtraction and recursive retraction by:

- time + override semantics,
- delete-and-rederive style maintenance,
- differential/delta semantics with negative multiplicities,
- or a lawful combination of these.

But the durable public surface MUST remain append-plus-supersession.

---

## 10. Verification, contradiction, and repair law

### 10.1 Risk-bearing outputs require plans
Anything that can influence promotion, rollback, operator trust, triage priority, settlement, or publication MUST emit `VerificationPlanV1`.

### 10.2 Minimum verification-plan contents
- cheapest admissible checks,
- replay recipe or preconditions,
- blocked checks and reasons,
- refutation suggestions,
- degradation flags,
- policy blockers,
- expiry or obsolescence conditions.

### 10.3 Contradictions are typed objects
Contradictions MUST surface as typed artifacts with:

- implicated claim/relation/region IDs,
- conflicting supports,
- contradiction class,
- blast radius,
- witness references,
- local/global scope,
- and allowed repair motifs.

### 10.4 Repair law
Repairs MUST be explicit artifacts, not cleanup folklore.

Every `RepairRecordV1` MUST include:

- trigger,
- repair class,
- implicated surfaces,
- minimal-change estimate,
- blast-radius justification,
- reversibility class,
- downstream invalidation scope,
- execution context,
- approvals required.

### 10.5 Local repair is default
When contradiction localizes to a bounded region, the stack MUST prefer local repair, quarantine, or regional rollback before global rebuild.

### 10.6 No silent stale truth
Downstream invalidation after repair, rollback, or contradiction MUST be typed, bounded, and traceable.

---

## 11. Runtime geometry and region protocol law

### 11.1 Right-graph law
The following are distinct objects and MUST NOT be silently collapsed:

- storage graph,
- retrieval graph,
- inference graph,
- repair graph,
- control graph.

### 11.2 Region is the default execution unit
A region is the smallest bounded surface that may:

- select or compile graph structure,
- run declared operators,
- emit typed artifacts,
- consume typed artifacts from adjacent regions,
- and produce a replayable local result.

### 11.3 Region contracts
`RegionContractV1` MUST include:

- region identity and kind,
- owner plane,
- scope and namespace,
- graph surfaces used,
- view families used,
- valid/recorded time bounds,
- admissible operator families,
- exactness ceiling,
- budget ceiling,
- stop-rule family,
- replay requirements,
- degradation policy,
- nuisance-state requirements,
- allowed inbound and outbound artifact families.

### 11.4 Typed boundary protocol
Regions MUST exchange typed artifacts only. Shared mutable global region state is forbidden.

At minimum, inter-region exchange MUST support:

- `DeltaEnvelopeV1`,
- `ResidualEnvelopeV1`,
- `SyndromeEnvelopeV1`,
- `NuisanceEnvelopeV1`,
- `BudgetEnvelopeV1`,
- `RegionBoundaryReceiptV1`,
- `RepairCandidateBundleV1`,
- `RegionReplaySliceV1`.

### 11.5 Convergence governance
Every materially important recursive region MUST emit or strongly reference `RegionConvergenceReportV1` with:

- iteration count,
- residual trend,
- oscillation flag,
- damping adjustments,
- stop reason,
- budget consumed,
- oracle escalation status,
- final exactness class,
- and whether output remained advisory-only.

### 11.6 No silent oscillation
If a region stops because of budget, damping ceiling, or explicit iteration cap rather than semantic convergence, that MUST be recorded as degradation.

### 11.7 Consensus guidance
Region-boundary reconciliation MAY use consensus or optimization-inspired schemes such as separator-message reconciliation or residual-based boundary consensus. Any such use MUST emit residual and stop-rule artifacts and MUST preserve local authority boundaries.

### 11.8 Monotonicity and coordination
Append-only evidence and other monotone summaries MAY replicate coordination-free. Non-monotone boundary effects MUST use explicit coordination or downgrade semantics.

---

## 12. Incremental, differential, and delete-and-rederive law

### 12.1 Local delta propagation beats blind rebuild
Derived artifacts SHOULD update by bounded delta propagation keyed to affected identities, regions, or time windows.

### 12.2 Differential time/version semantics
A conforming runtime MAY represent updates over partially ordered version/time spaces rather than a single linear clock, provided replay and bitemporal semantics remain explicit.

### 12.3 Delete-and-rederive admissibility
Recursive deletions or retractions MAY be implemented via delete-and-rederive style maintenance, but the system MUST emit the deletions, rederivations, and resulting exactness/degradation state as typed artifacts.

### 12.4 Snapshots are performance aids, not authority
Snapshots MAY accelerate replay but MUST never be the only source of reconstructible truth.

---

## 13. Mechanism and theory law

### 13.1 Theories are lawful artifacts, not notebook residue
A mechanism or theory that can influence action, ranking, or publication MUST exist as canonical typed artifacts.

### 13.2 Mechanism bundles
`MechanismBundleV1` MUST include:

- mechanism identity and family,
- variable/state schema,
- domain/scope,
- related episodes/claims,
- simulator/evaluator refs,
- stability or invariance expectations,
- required refuter classes,
- publication status.

### 13.3 Theory versions
`TheoryVersionV1` MUST include:

- theory identity and version,
- mechanism refs,
- parameterization family,
- search/fit provenance,
- replay and exactness class,
- comparability constraints,
- supersession/dispute linkage.

### 13.4 Observational equivalence remains visible
If multiple mechanisms remain observationally comparable under current evidence, the stack MUST preserve that fact rather than inventing a single winner.

### 13.5 Theory search is typed work
Search programs, fit runs, and simulator runs MUST carry budgets, stop rules, replay handles, exactness classes, and failure taxonomies.

### 13.6 Refutation outranks elegance
Interpretability, sparsity, or structural beauty do not promote a theory. Refuter obligations do.

---

## 14. Federation and treaty law

### 14.1 Federation preserves local authority
Cross-runtime settlement MAY influence local decisions only through lawful admission and control paths. No treaty may directly overwrite local truth.

### 14.2 Treaty bundles
`TreatyBundleV1` MUST declare:

- participating runtimes,
- trust roots,
- admissible artifact families,
- disclosure classes,
- replayability floor,
- admissible exactness classes,
- dispute window,
- suspension/revocation rules,
- effective coordinates.

### 14.3 Identity equivalence is typed work
Cross-runtime “same thing” judgments MUST use `CrossRuntimeEquivalenceBundleV1`; silent identity collapse is forbidden.

### 14.4 Settlement cases are replayable artifacts
Every shared disposition MUST be backed by `SettlementCaseV1` with:

- treaty scope,
- participants,
- local and shared evidence refs,
- local dissent states,
- evidentiary quorum logic,
- witness/certificate classes,
- replay handles,
- downgrade/quarantine outcomes.

### 14.5 Proof outranks count
Majority or quorum agreement MUST NOT outrank a stronger witness class, a valid contradiction core, or a failed refuter suite.

---

## 15. Constitutional self-hosting law

### 15.1 The constitution is an artifact system
The spec itself MUST eventually be representable as:

- `SpecBundleV1`,
- `NormativeASTV1`,
- generated schemas,
- generated reference interpreters,
- generated conformance corpora,
- generated migration plans,
- proof-obligation sets,
- self-hosting build receipts.

### 15.2 Generated is not automatically admitted
Generated schemas, interpreters, or corpora remain advisory until admitted through already lawful constitutional paths.

### 15.3 Human challenge and veto remain mandatory
No self-hosting constitutional machine may bypass human approval, challenge, rollback, or veto.

### 15.4 No prose/execution split drift
No materially important constitutional surface may exist only in prose if generation, conformance, or migration depends on it.

---

## 16. Crate and package responsibility map

### 16.1 Existing canonical crates
- `stack-ids` MUST own shared identifiers, digests, replay handles, and trace primitives.
- `semantic-memory-forge` MUST own canonical evidence/export package families.
- `forge-memory-bridge` MUST prove atomic import, digest preservation, and backpointer preservation.
- `semantic-memory` MUST expose first-class episode identity, normalized causes, derivations, and queryable lineage.
- `knowledge-runtime` MUST implement declared view usage, widening disclosure, result provenance, and region planning.
- Kernel crates MUST emit witnesses, certificates, residuals, syndromes, support cores, frontiers, and convergence artifacts without claiming truth authority.
- `verification-*` crates MUST integrate with canonical episode, execution, and proof artifact families.
- `llm-tool-runtime` MUST be a canonical provider/tool receipt seam, not a convenience wrapper with disappearing lineage.
- `forge-pilot` MUST remain a consumer-only orchestrator over public canonical artifacts.
- schema-generation crates MUST own schema publication, compatibility classification, and migration proof artifacts.

### 16.2 Future canonical ownership
If new crates are introduced for federation, theory search, or constitutional generation, they MUST preserve the same authority asymmetry rather than inventing new truth planes.

---

## 17. Conformance and proof-obligation law

### 17.1 Reference interpreters are mandatory for hard semantic seams
At minimum, the stack MUST provide executable reference behavior for:

- valid-time / recorded-time query semantics,
- bridge atomicity and digest invariants,
- runtime widening semantics,
- subtraction-receipt invariants,
- region boundary protocol invariants,
- repair-record invariants,
- and promoted exactness classes.

### 17.2 Shape validation is necessary but insufficient
Canonical schemas are required for all wire-visible artifact families. Semantic conformance MUST be checked by reference interpreters and fixture corpora, not only shape validators.

### 17.3 Required test tracks
A conforming end-state program SHOULD maintain at least these benchmark tracks:

- **decoder track** — hypergraph correction quality, convergence, and oracle-slice cost,
- **region track** — asynchronous region convergence, damping behavior, local-repair economics,
- **evidence track** — retries, timeouts, reroutes, bitemporal replay, provenance closure,
- **subtraction track** — support-core extraction, frontier discovery, receipt verification, historical-loss budgets,
- **settlement track** — treaty exchange, local dissent, shared disposition, downgrade behavior,
- **constitutional track** — generated schema/interpreter parity, migration proofs, amendment simulation.

### 17.4 Promotion requires proof obligations
A claim, repair, settlement, or theory output is promotable only if its proof obligations and blocked checks are queryable.

---

## 18. Explicit prohibitions

The following are non-conforming:

- a giant omniscient graph with no right-graph distinctions,
- score-only promotion,
- silent widening,
- silent destructive compaction,
- hidden repair loops,
- runtime-owned shadow truth,
- bridge-invented semantics,
- document identity masquerading as episode identity,
- parser/patch repair that changes effective treatment without provenance,
- local dissent erased by federation,
- generated law admitted merely because generation succeeded.

---

## 19. Default execution algorithm (normative pseudocode)

```text
INPUT:
  declared query or task Q
  admissible evidence scope E
  valid/recorded time coordinates (V, R)
  region plan Γ
  invariants I
  historical-loss budget H
  exactness budget B

STATE:
  semantic product S

LOOP:
  1. select regions and right graphs for Γ
  2. load evidence and execution context admissible under (E, V, R)
  3. run infer operators to expand S
  4. emit residuals / syndromes / provisional witnesses
  5. if subtraction admissible, compute SupportCore / RemovalFrontier under (I, H, B)
  6. run cheap checks, refuters, and bounded oracle slices required by policy
  7. if contradiction localized, run local repair or quarantine; emit repair artifacts
  8. propagate only affected deltas across regions
  9. emit convergence, provenance, degradation, and proof artifacts
 10. stop if governed fixed-point condition holds; otherwise iterate

OUTPUT:
  result artifact(s)
  runtime query provenance
  execution evidence linkage
  verification plan
  contradiction / repair artifacts if any
  subtraction artifacts if any
  exactness and degradation disclosure
```

---

## 20. Source basis and cross-reference synthesis (non-normative)

### 20.1 Internal materials directly integrated
This spec consolidates the architecture line represented by the following core internal materials:

- `V9_V10_SPEC_COMPENDIUM.md`
- `CANONICAL_STACK_SPEC_V12_REGIONAL_FIXPOINT_RUNTIME.md`
- `canonical_stack_spec_v_11_executable_semantics_and_proof_governance.md`
- `subtraction.md`
- `semantics.md`
- `decoder.md`
- `region.md`
- `contract.md`
- `passing_recursion.md`
- `deep-brief.md`
- `abacus-research.md`
- `CANONICAL_STACK_SPEC_V16_FEDERATED_CLAIM_SETTLEMENT_AND_TREATY_RUNTIME.md`
- `CANONICAL_STACK_SPEC_V17_MECHANISM_LIBRARY_AND_THEORY_SEARCH_RUNTIME.md`
- `CANONICAL_STACK_SPEC_V20_SELF_HOSTING_SPEC_EXECUTION_AND_CONSTITUTIONAL_AUTOGOVERNANCE.md`

Taken together, these contribute:

- authority, bitemporal, and anti-shadow-database law,
- episode-first identity and execution evidence,
- runtime geometry and region protocols,
- executable semantics and proof governance,
- lawful subtraction / epistemic renormalization,
- provenance, bilattice, recursion, and retraction semantics,
- decoder-style hypergraph kernel behavior,
- contract and truthful artifact law,
- execution-evidence and testimony-oriented control-plane design,
- federation/treaty settlement,
- mechanism/theory object law,
- and self-hosting constitutional generation.

### 20.2 External analogs intentionally incorporated
The spec also intentionally borrows structure from several external bodies of work because they tighten the end-state design rather than merely decorate it:

- **fixed-point acceleration / Anderson-style mixing / SCF practice** for governed acceleration of recursive loops,
- **abstract interpretation** for widening/narrowing, sound approximation, and explicit fixed-point discipline,
- **differential dataflow / Naiad** for nested incremental iteration and delta propagation,
- **DRed / incremental view maintenance** for delete-and-rederive retraction semantics,
- **CALM monotonicity** for the law that only monotone surfaces can avoid coordination,
- **ADMM / residual consensus** for region-boundary coordination intuition,
- **OpenTelemetry messaging conventions** for queue-hop and producer/consumer lineage semantics.

These are design-basis guides, not authority replacements.

---

## 21. Final constitutional statement

A system conforms to this end-state specification only if it can answer, with canonical typed artifacts:

1. **What exactly is the identity-bearing episode here?**
2. **What was true then, and what was only believed then?**
3. **Which execution conditions produced this result?**
4. **Which view and widening choices surfaced it?**
5. **Which support is truly load-bearing?**
6. **How close is the result to breaking?**
7. **Which contradictions, repairs, or quarantines touched it?**
8. **Which proof obligations remain blocked?**
9. **Which regions and boundaries carried the computation?**
10. **What prevents this machine from silently rewriting truth, history, or law?**

If the system cannot answer those questions with canonical artifacts, it is not yet the end-state stack.
