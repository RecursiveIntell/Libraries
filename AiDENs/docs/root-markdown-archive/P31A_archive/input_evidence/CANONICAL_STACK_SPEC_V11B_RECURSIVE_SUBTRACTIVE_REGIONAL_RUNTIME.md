# Canonical Stack Spec v11B — Recursive/Subtractive Regional Runtime

**Status:** Proposed canonical v11B target  
**Relationship to v11A:** v11A is prerequisite law. v11B may not weaken artifact lifecycle, operator effects, proof economy, boundary compiler, execution evidence, or reference-conformance law.  
**Relationship to v10:** v10 made runtime geometry explicit. v11B makes that geometry locally executable, recursively governable, and lawfully subtractive.  
**Scope:** right-graph compiler law, region contracts, typed boundary protocol, convergence governance, recursive/subtractive closure loops, lawful subtraction, local repair, contradiction surfaces, nuisance state, time-aware incremental recomputation, and causal/interventional execution.

---

## 0. Purpose

v11B defines the runtime layer that operates over the v11A constitutional microkernel.

If v11A answers “what counts as lawful work?”, v11B answers:

- which graph should be executed;
- where execution boundaries live;
- how regions communicate;
- when recursive inference stops;
- how contradiction localizes;
- how repair proceeds without global thrash;
- how subtraction removes structure without destroying truth;
- how causal claims become intervention/replay artifacts rather than blame stories.

The shortest definition:

> v11B turns the artifact runtime into a regional recursive/subtractive machine: infer, residualize, subtract, verify, repair, propagate, and govern under typed boundary law.

---

## 1. Normative language

The words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, and **MAY** are normative.

Where logical model and physical implementation differ, the logical model wins.

If v11B appears to conflict with v11A, the stricter interpretation wins unless v11B explicitly adds a stricter specialization.

---

## 2. Core thesis

A conforming v11B runtime is not a giant omniscient graph. It is a set of **small communicating regions** that exchange typed artifacts.

Each region executes over a declared graph surface, emits receipts, preserves bitemporal and execution context, tracks convergence/degradation, models nuisance state, and exposes contradiction/repair/subtraction artifacts.

The canonical regional work shape is:

```text
RegionContract
+ CompiledGraphBundle
+ InputArtifactManifest
+ ExecutionContext
+ Budget/ProofProfile
→ RegionResult
+ BoundaryMessages
+ Residuals/Syndromes
+ Witnesses/Certificates/Refutations
+ ConvergenceReport
+ Receipts
```

---

## 3. Right-graph law

### 3.1 Distinct graph surfaces

The following graph surfaces are distinct logical objects and MUST NOT be silently collapsed:

| Graph surface | Purpose |
|---|---|
| Storage graph | Durable/projection storage relationships and indexes. |
| Retrieval graph | Candidate retrieval/search expansion and ranking. |
| Inference graph | Factor/hypergraph/operator execution for beliefs, constraints, residuals, and estimates. |
| Repair graph | Contradiction localization, blast-radius analysis, repair candidate routing. |
| Subtraction graph | Support-core, removal-frontier, compaction, minimization, and history-budget analysis. |
| Control/receipt graph | Execution lineage, receipts, policy, approvals, budgets, proof debt. |
| Causal/intervention graph | Treatment/outcome/confounder/intervention/counterfactual relations. |
| Federation/admission graph | External artifacts, admission, equivalence, disputes; v11C horizon. |
| Mechanism/theory graph | Hypotheses, simulators, fit runs, refuter suites; v11C horizon. |
| Constitutional graph | spec clauses, obligations, generated schemas/tests/interpreters; v11C horizon. |

### 3.2 Graph surface declaration

Every region MUST declare which graph surface(s) it uses through `GraphSurfaceDeclarationV1`.

The declaration MUST include:

- graph kind;
- source artifact manifest;
- compiler/selector id;
- compilation parameters;
- time coordinates;
- information-loss declaration;
- exactness class;
- digest;
- allowed operator families;
- relation to other graph surfaces if any.

### 3.3 Storage graph is not inference by inertia

A runtime MUST NOT use the storage graph as the inference graph merely because it is available.

If the storage graph is used for inference, the region MUST prove or disclose that:

- required variables/factors/constraints are represented;
- no required compilation step was skipped;
- exactness/degradation is declared;
- conformance fixtures cover the shortcut.

### 3.4 Right-graph conformance tests

The release harness MUST include tests that catch:

- storage graph being used as inference graph without declaration;
- retrieval expansion treated as causal evidence;
- repair graph writing truth directly;
- control graph acting as a hidden database;
- subtraction graph deleting without invariant receipt;
- external/federated graph influencing local truth without admission.

---

## 4. Region model

### 4.1 Region definition

A **Region** is the default execution unit for bounded graph work.

A region MAY:

- compile or select one or more graph surfaces;
- run declared operators;
- consume incoming typed artifacts;
- emit outgoing typed artifacts;
- maintain replay-linked local state;
- request oracle escalation;
- emit residuals, syndromes, witnesses, certificates, refutations, and repair candidates.

A region MUST NOT become a hidden truth store.

### 4.2 `RegionContractV1`

`RegionContractV1` MUST include:

- `region_id`;
- `region_kind`;
- owner plane;
- namespace/scope;
- input artifact family set;
- output artifact family set;
- graph surface declarations;
- valid-time bounds;
- recorded-time bounds;
- admissible operator families;
- exactness ceiling;
- budget ceiling;
- proof profile;
- stop-rule family;
- convergence policy;
- damping/scheduling policy;
- nuisance-state requirements;
- contradiction handling policy;
- repair eligibility;
- subtraction eligibility;
- replay requirements;
- degradation policy;
- human/governance approval triggers.

### 4.3 Region state classes

Every region MUST explicitly distinguish:

| State class | Meaning |
|---|---|
| Observed state | Authoritative or projection-backed inputs. |
| Latent state | Inferred, non-authoritative hidden assignments. |
| Nuisance state | Measurement, execution, retrieval, environment, parser, model, or scheduling factors that affect interpretation. |
| Contradiction state | Local conflict structures, violated constraints, incompatible claims, refutations. |
| Proof state | Witnesses, certificates, refutations, proof debt, waivers. |
| Budget state | Cost, deadline, proof budget, exactness budget, repair/subtraction budget. |
| Boundary state | Incoming/outgoing messages, accept/reject/quarantine receipts. |

A region MUST NOT smuggle nuisance, contradiction, proof, or boundary information into generic metadata if it changes interpretation.

### 4.4 `RegionStateSnapshotV1`

A materially important region execution MUST be able to emit `RegionStateSnapshotV1` containing:

- observed state summary;
- latent state summary;
- nuisance state summary;
- contradiction state summary;
- proof state summary;
- exactness state;
- budget state;
- incoming/outgoing artifact refs;
- execution context refs;
- replay slice refs;
- degradation refs.

Snapshots MAY be used for performance. They MUST remain replay-linked and non-authoritative for domain truth.

---

## 5. Typed boundary protocol

### 5.1 Region communication

Regions communicate only through typed artifacts. Shared mutable global state is forbidden unless it is itself represented as a canonical artifact family with replay law.

### 5.2 Minimum boundary families

The region boundary protocol MUST support:

- `RegionBoundaryMessageV1`
- `RegionBoundaryReceiptV1`
- `DeltaEnvelopeV1`
- `ResidualEnvelopeV1`
- `SyndromeEnvelopeV1`
- `NuisanceEnvelopeV1`
- `BudgetEnvelopeV1`
- `RegionReplaySliceV1`
- `RepairCandidateBundleV1`
- `DegradationRecordV1`
- `SemanticDiffV1`

### 5.3 `RegionBoundaryMessageV1`

A boundary message MUST include:

- source region;
- destination region;
- artifact family;
- payload ref or digest;
- view family;
- valid-time coordinate;
- recorded-time coordinate;
- execution context refs;
- exactness class;
- degradation state;
- budget impact;
- required acceptance policy;
- replay refs.

### 5.4 `RegionBoundaryReceiptV1`

A boundary receipt MUST include:

- message ref;
- accept/reject/quarantine status;
- receiving region;
- reason;
- digest validation result;
- schema validation result;
- proof/admission result if relevant;
- changed local state refs;
- replay handle;
- escalation path.

### 5.5 Boundary failure

If a region rejects or quarantines an incoming message, the rejection MUST be a first-class event with reason and downstream impact. Silent dropping is non-conforming.

---

## 6. Convergence governance

### 6.1 Recursive operators require stop law

Any recursive, iterative, message-passing, fixpoint, residual-correction, or feedback operator MUST declare:

- update rule;
- schedule;
- damping/relaxation policy;
- residual threshold;
- iteration cap;
- budget cap;
- convergence criterion;
- oscillation criterion;
- stagnation criterion;
- exactness claim;
- failure/degradation output.

### 6.2 `ConvergenceReportV1`

Every nontrivial iterative region MUST emit `ConvergenceReportV1` with:

- iterations executed;
- schedule used;
- damping policy;
- residual history summary;
- stop reason;
- oscillation/stagnation markers;
- remaining residuals;
- exactness/degradation impact;
- oracle escalation request or reason not requested;
- proof impact.

### 6.3 Oscillation handling

Oscillation is not success.

If a region oscillates, it MUST emit a degradation, syndrome, or escalation artifact. It MUST NOT emit a stable-looking answer without disclosure.

### 6.4 Residual-driven scheduling

Regions SHOULD prioritize residuals/syndromes rather than sweeping blindly when this improves locality and cost. If a learned scheduler is used, it remains advisory and MUST be bounded by proof/convergence law.

---

## 7. Recursive/subtractive closure loop

### 7.1 Canonical loop

v11B introduces `ClosureMinimizationLoopV1` as the canonical loop for maintaining claim-like state under change:

```text
infer
→ residualize
→ subtract
→ verify/refute
→ repair/quarantine
→ propagate deltas
→ govern
```

### 7.2 Loop stages

| Stage | Required behavior |
|---|---|
| infer | Run declared inference operators over declared graph surface. |
| residualize | Emit residuals/syndromes for unresolved constraints, errors, or uncertainty. |
| subtract | Identify removable structure or minimal support under invariant law. |
| verify/refute | Run proof obligations, refuters, oracle slices, or checkers. |
| repair/quarantine | Localize contradictions, propose repair, apply lawful repair, or quarantine. |
| propagate deltas | Emit typed deltas and boundary messages. |
| govern | Update proof debt, waivers, approvals, release gates, and user-visible disclosure. |

### 7.3 Loop receipt

`ClosureMinimizationLoopV1` MUST include:

- loop id;
- region ids;
- input artifact manifest;
- graph declarations;
- operator invocation receipts;
- residual/syndrome refs;
- subtraction refs;
- proof/refutation refs;
- repair/quarantine refs;
- delta outputs;
- convergence reports;
- governance receipts;
- final semantic state;
- replay slice.

### 7.4 No infinite cleverness

A loop MUST have stop law. A loop that exceeds budget/deadline/iteration/proof limits MUST degrade explicitly.

---

## 8. Lawful subtraction

### 8.1 Subtraction is not deletion

Subtraction includes:

- compaction;
- summarization;
- forgetting;
- minimization;
- deduplication;
- support-core extraction;
- removal-frontier calculation;
- evidence thinning;
- graph simplification;
- region boundary coarsening;
- history-budgeted retention.

A subtraction is lawful only if it is typed, receipt-bearing, challengeable, replay-aware, and governed by declared invariant/history budgets.

### 8.2 Subtraction effect requirement

Any operator that removes, compresses, summarizes, coalesces, prunes, or hides active structure MUST declare `SUBTRACTS_STRUCTURE` in `OperatorContractV1` and MUST use `SubtractionOperatorContractV1`.

### 8.3 `SubtractionOperatorContractV1`

A subtraction contract MUST include:

- target artifact families;
- protected artifact families;
- invariant set;
- protected query set;
- history budget;
- allowed loss class;
- forbidden loss class;
- proof obligations;
- replay obligations;
- challenge path;
- rollback/restore path;
- user/governance visibility.

### 8.4 `SupportCoreV1`

A support core identifies minimal or near-minimal load-bearing support for a target.

Required fields:

- target artifact/query/claim;
- candidate support set;
- minimality class (`exact`, `bounded_exact`, `heuristic`, `unknown`);
- proof/checker;
- excluded artifacts;
- dependency expression;
- replay slice;
- risk/degradation state.

### 8.5 `RemovalFrontierV1`

A removal frontier identifies structure that may be removed without violating declared invariants.

Required fields:

- frontier id;
- target scope;
- removable candidates;
- protected core;
- invariant set;
- estimated blast radius;
- proof status;
- loss budget impact;
- challenge handle.

### 8.6 `InvariantPreservationReceiptV1`

Every applied subtraction MUST emit an invariant-preservation receipt containing:

- subtraction operator;
- before refs;
- after refs;
- removed/compacted refs;
- preserved invariants;
- checker/refuter results;
- protected query results before/after;
- historical-loss impact;
- degradation state;
- rollback/restore handle.

### 8.7 `HistoricalLossBudgetV1`

Any subtraction that weakens historical replay, query fidelity, provenance detail, or evidence granularity MUST be governed by `HistoricalLossBudgetV1`.

The budget MUST define:

- protected as-of queries;
- protected evidence classes;
- allowed summaries;
- forbidden loss;
- retention horizon;
- challenge and restoration policy;
- user/regulatory/contractual constraints where applicable.

### 8.8 Subtraction challenge

A user, runtime, proof checker, or governance surface MAY challenge a subtraction via `SubtractionChallengeV1`.

A valid challenge MUST trigger one of:

- restoration;
- revised receipt;
- degradation disclosure;
- proof escalation;
- governance decision;
- rejection with reason.

---

## 9. Contradiction, residual, syndrome, and repair law

### 9.1 Contradictions are typed objects

Contradictions MUST NOT be hidden as lower confidence.

When the system detects incompatible claims, violated constraints, failed refuters, impossible time overlaps, conflicting identities, or execution/proof inconsistencies, it MUST emit a contradiction surface.

### 9.2 `SyndromeEnvelopeV1`

A syndrome MUST include:

- violated constraint(s);
- affected artifacts;
- support refs;
- refutation refs;
- locality/region;
- severity;
- possible nuisance contribution;
- candidate repair families;
- oracle escalation eligibility;
- proof impact;
- promotion impact.

### 9.3 `ResidualEnvelopeV1`

A residual MUST include:

- residual kind;
- magnitude/structure;
- source operator;
- affected graph elements;
- scheduling priority;
- convergence impact;
- possible repair/subtraction impact;
- degradation if ignored.

### 9.4 Local repair before global rebuild

If a contradiction localizes to a bounded region, the runtime MUST prefer:

1. local proof check;
2. local repair candidate;
3. local quarantine;
4. local rollback;
5. bounded recompile;
6. only then global recomputation.

### 9.5 `RepairCandidateBundleV1`

A repair candidate MUST include:

- target syndrome/residual;
- proposed repair operation;
- affected artifact set;
- blast radius;
- proof obligations;
- treatment-integrity impact;
- rollback path;
- quarantine policy;
- confidence/ranking if advisory;
- exactness class.

### 9.6 Applied repair

An applied repair MUST emit `RepairExecutionReceiptV1` with before/after semantic diffs and proof/refutation impact.

Repair MUST preserve append-plus-supersession. Silent rewrite is forbidden.

---

## 10. Nuisance-state law

### 10.1 Nuisance is modeled, not buried

Nuisance structure includes execution, measurement, retrieval, environment, parser, tool, model, scheduler, workload, and calibration factors that affect interpretation without being the semantic claim itself.

Examples:

- test selection differences;
- dependency/toolchain drift;
- retry contamination;
- workload drift;
- parser repair;
- missing retrieval shard;
- model/provider instability;
- deadline truncation;
- alias uncertainty;
- sensor/measurement bias.

### 10.2 `NuisanceEnvelopeV1`

A nuisance envelope MUST include:

- nuisance kind;
- source;
- affected artifacts/regions;
- time coordinates;
- estimated interpretation impact;
- calibration status;
- mitigation if any;
- proof/promotion impact.

### 10.3 Nuisance and causality

A causal or interventional claim MUST either control for relevant nuisance state or explicitly degrade/refuse causal language.

---

## 11. Time-aware incremental recomputation

### 11.1 Delta propagation over blind rebuild

Derived artifacts SHOULD update by bounded lineage-, scope-, entity-, time-, region-, or manifest-aware recomputation.

Blind global rebuild is a last resort and MUST be disclosed if used.

### 11.2 `DeltaEnvelopeV1`

A delta envelope MUST include:

- source artifact/region;
- destination artifact/region;
- change class;
- valid-time coordinate;
- recorded-time coordinate;
- affected identity set;
- invalidation cone;
- exactness class;
- replay coordinates;
- digest;
- proof/degradation impact.

### 11.3 Invalidation cones

The runtime MUST be able to describe what may be invalidated by a change.

An invalidation cone MAY be approximate, but if approximate it MUST declare exactness and degradation.

### 11.4 Temporal revision across regions

When a retroactive correction changes valid time or recorded belief, region messages MUST preserve both axes and affected regions MUST either:

- recompute;
- quarantine stale outputs;
- emit degraded view;
- prove unaffectedness.

---

## 12. Causal and interventional law

### 12.1 Causal claims are artifact packages

A causal claim MUST NOT be represented as a naked edge with a score.

It MUST be represented as `CausalAttributionBundleV1` or equivalent.

### 12.2 `CausalAttributionBundleV1`

Required fields:

- causal question;
- unit definition;
- treatment definition;
- outcome definition;
- covariates/confounders;
- nuisance controls;
- identification assumptions;
- estimator or reasoning method;
- refuter suite;
- intervention/counterfactual replay plan if available;
- evidence refs;
- proof profile;
- degradation state;
- promotion eligibility.

### 12.3 Interventional checks

Risk-bearing causal claims SHOULD have at least one of:

- baseline-vs-treated replay;
- paired patch/control run;
- negative-control check;
- placebo treatment check;
- dummy outcome check;
- subsample/bootstrap stability check;
- oracle-slice validation;
- human/governance review.

If none exists, causal language MUST be degraded or restricted.

### 12.4 Code-change attribution

For code-change attribution, the following MUST be modeled where available:

- patch/edit as treatment;
- failure/metric drift as outcome;
- environment/toolchain as nuisance/confounder;
- test selection as nuisance/confounder;
- adjacent edits as confounders;
- workload/config/feature flags;
- execution receipts for baseline and patched runs;
- repair/patch treatment-integrity receipts.

---

## 13. Exactness and oracle slices

### 13.1 Exactness classes

v11B regions MUST distinguish:

- `heuristic`;
- `approximate_message_passing`;
- `bounded_conservative`;
- `bounded_exact`;
- `oracle_exact_on_slice`;
- `replay_exact`;
- `unknown`.

### 13.2 Oracle escalation

A region SHOULD escalate to an exact or conservative oracle slice when:

- contradiction severity is high;
- proof profile requires it;
- convergence fails;
- residual remains above threshold;
- repair blast radius is uncertain;
- promotion would otherwise occur on approximate evidence.

### 13.3 Oracle result

`OracleSliceResultV1` MUST include:

- slice definition;
- method;
- assumptions;
- exactness guarantee;
- result;
- cost;
- relation to approximate path;
- semantic diff;
- proof impact.

---

## 14. v11B crate responsibility map

| Surface | Likely owner(s) | v11B responsibility |
|---|---|---|
| Graph compilation | `constraint-compiler`, kernel crates | graph declarations, compiled graph bundles, right-graph gates. |
| Kernel execution | `kernel-execution`, `recursive-kernel-core` | operator execution, convergence reports, residual/syndrome emission. |
| Oracles | `kernel-oracles`, conformance harness | bounded exact/conservative oracle slices. |
| Region runtime | `knowledge-runtime`, kernel crates | region contracts, boundary protocol, replay slices. |
| Repair/subtraction | `semantic-memory`, runtime/kernel support | repair candidates, subtraction receipts, support cores, removal frontiers. |
| Causal attribution | forge/control/runtime bridge surfaces | causal bundles, intervention plans, replay packages. |
| Conformance | `kernel-conformance`, workspace tests | region fixtures, graph misuse tests, convergence tests, subtraction invariant tests. |

---

## 15. v11B release bar

A release may claim v11B compliance only if:

1. v11A compliance is satisfied.
2. `GraphSurfaceDeclarationV1` exists and is required for region execution.
3. Storage/retrieval/inference/repair/subtraction/control graph surfaces are not silently collapsed.
4. `RegionContractV1`, `RegionBoundaryMessageV1`, and `RegionBoundaryReceiptV1` exist.
5. At least one region execution path emits `RegionStateSnapshotV1`, `ConvergenceReportV1`, and replay slice.
6. Residuals and syndromes are typed artifacts, not logs.
7. Recursive operators have stop rules and degradation behavior.
8. Lawful subtraction artifacts exist: `SupportCoreV1`, `RemovalFrontierV1`, `InvariantPreservationReceiptV1`, and `HistoricalLossBudgetV1`.
9. At least one subtraction operator is tested against invariant-preservation fixtures.
10. Repair candidates and applied repairs emit receipts and semantic diffs.
11. Causal attribution claims use `CausalAttributionBundleV1` or are explicitly prohibited/degraded.
12. Nuisance state exists as a first-class envelope for at least execution/test/retrieval contamination.
13. Region/convergence/subtraction/repair semantics have reference fixtures or explicit temporary gaps recorded as release debt.

---

## 16. v11B non-goals

v11B does not require full distributed federation. That belongs to v11C+.

v11B does not require full mechanism/theory search. It requires causal/interventional substrate and graph/operator discipline that future mechanism search can use.

v11B does not require all regions to be exact. It requires exactness honesty, oracle slices where required, and convergence/degradation reporting.

v11B does not require all subtraction to be minimal. It requires minimality class honesty and invariant-preservation receipts.

---

## 17. Design warning

A v11B stack that uses one graph for everything is not compliant.

A v11B stack that recurses without stop law is not compliant.

A v11B stack that summarizes memory without a historical-loss budget is not compliant.

A v11B stack that hides contradiction as a lower score is not compliant.

A v11B stack that claims causality from proximity without treatment/outcome/confounder/refuter artifacts is not compliant.

A v11B stack that repairs through in-place mutation is not compliant.

The runtime should be powerful, but not feral. Feral cleverness is how you get a haunted graph database with invoices.
