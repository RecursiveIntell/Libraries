# Canonical Stack Spec v11A — Constitutional Artifact Runtime Core

**Status:** Proposed canonical v11A target  
**Relationship to v6:** v6 remains the law of authority boundaries, bitemporality, append-plus-supersession, import discipline, and no-shadow-database behavior.  
**Relationship to v7:** v7 remains the law of recursive inference operators, witnesses, certificates, residuals, syndromes, and oracle slices.  
**Relationship to v8:** v8 remains the law of verification control, policy, adjudication, rollback, replay, and approval.  
**Relationship to v9:** v9 remains the law of episode-first identity, execution evidence, bridge invariants, repair records, and verification-plan artifacts.  
**Relationship to v10:** v10 remains the runtime geometry law: distinct compiled graphs, time-aware incremental recomputation, small communicating regions, syndrome-first repair, nuisance-state modeling, and explicit execution geometry.  
**Relationship to v11B/v11C:** v11A is the microkernel. v11B depends on it. v11C reserves future-admission hooks and must not weaken v11A.

---

## 0. Purpose

v11A defines the **constitutional artifact runtime core**.

After v10, the stack has enough geometry to execute sophisticated runtime behavior. The new risk is that the system can become architecturally impressive while silently meaning different things in different paths.

v11A exists to prevent that.

It defines:

1. the canonical material-action model;
2. the canonical artifact lifecycle;
3. the semantic product carried by claim-like outputs;
4. the operator effect system;
5. the proof economy;
6. execution evidence as artifact semantics;
7. boundary compiler law;
8. reference interpreters and conformance surfaces;
9. proof-governed promotion eligibility; and
10. the minimum release bar for anything that wants to claim v11A compliance.

The shortest definition:

> v11A turns the stack from a set of evidence-aware crates into a constitutional runtime where every material action is a typed, receipt-bearing transition over artifacts under declared semantic law.

---

## 1. Normative language

The words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, and **MAY** are normative.

Where this document distinguishes between:

- **logical model** — artifact semantics, required fields, relations, state transitions, invariants, effects, and conformance behavior;
- **physical model** — crates, structs, APIs, tables, queues, jobs, storage engines, generated files, and runtime implementations;

…the logical model is mandatory. The physical model is flexible only where this document explicitly permits flexibility.

If prose conflicts with a normative table, artifact contract, or conformance gate, the stricter interpretation wins.

---

## 2. Core thesis

The stack MUST be treated as a **typed artifact runtime**, not as a model pipeline, graph database, log store, or orchestration script.

A conforming v11A runtime has this canonical work shape:

```text
Artifact(s)
+ OperatorContract
+ ExecutionContext
+ Permit/Policy
+ Budget
→ Artifact(s)
+ Receipt(s)
+ Proof/Refutation/Degradation state
```

No material operation gets to “just do work.”

A **material operation** is any operation that can affect at least one of:

- truth-bearing state;
- projected/queryable state;
- execution evidence;
- proof status;
- promotion eligibility;
- repair status;
- subtraction/minimization eligibility;
- boundary interpretation;
- schema interpretation;
- external admission;
- user-visible completion state;
- future scheduling, ranking, or action selection.

Material operations MUST be represented as artifact-producing transitions with explicit receipts.

---

## 3. Enduring doctrine carried forward

### 3.1 Authority remains asymmetric

- Forge/import lanes remain authoritative for truth-bearing promotion.
- Projection lanes remain authoritative for imported, queryable truth.
- Runtime, tool, pilot, region, scheduler, inference, and learned-model layers remain non-authoritative for domain truth.
- Advisory outputs, scores, candidate repairs, model completions, tool results, and approximate inference results MUST NOT self-promote.

### 3.2 Time remains part of meaning

- Valid time and recorded/transaction time are distinct.
- Append-plus-supersession remains the only lawful durable mutation model for truth-bearing state.
- Repairs, replays, semantic diffs, boundary repairs, proof-state changes, and promotion changes MUST preserve time coordinates.

### 3.3 Execution remains evidence

Retries, queue hops, fan-out, deadlines, truncation, degradation, budget exhaustion, provider/tool route, replay lineage, and attempt-family identity are part of evidence meaning.

They MUST NOT be treated as mere telemetry when they affect reproducibility, comparability, proof status, or downstream use.

### 3.4 No shadow databases

- Runtime caches MUST be derivable.
- Tool receipts MUST NOT become truth stores.
- Generated schemas MUST NOT become hidden law unless admitted through schema governance.
- CI fixtures MUST NOT silently define production semantics unless they are generated or admitted under reference-conformance law.
- Model memory, vector indexes, scratchpads, and notebooks MUST NOT become authoritative truth.

### 3.5 Witnesses outrank naked scores

Where a subsystem can emit a witness, certificate, refutation, semantic diff, conformance run, support core, or invariant receipt, that artifact outranks an opaque scalar score.

Scores MAY triage. They MUST NOT settle.

### 3.6 Verification outranks inference

An inferred structure cannot outrank explicit verification. An approximate path cannot suppress contradiction surfaced by a stricter path. A learned ranker cannot erase proof debt.

### 3.7 Learning remains downstream

Learning MAY assist scheduling, ranking, abstention, calibration, retrieval, region selection, and proof-economics estimation.

Learning MUST NOT rewrite truth semantics, identity semantics, time semantics, causal semantics, schema law, proof obligations, or promotion law.

---

## 4. Canonical planes under v11A

| Plane | Authoritative for | v11A hardening | MUST NOT |
|---|---|---|---|
| Primitive plane | IDs, digests, canonical refs, trace primitives | Stable artifact IDs and digest semantics | Add domain truth or policy logic |
| Evidence plane | Raw evidence, attempts, episodes, verification exports | `EpisodeBundleV1`, `ClaimEvidencePackageV1`, evidence fixity | Outsource raw truth interpretation to runtime |
| Bridge plane | Deterministic transforms from raw/export truth to projection import | atomic import receipts, digest preservation, boundary compiler profile | Become a policy engine |
| Projection plane | Queryable imported/projected truth | semantic product, view disclosures, bitemporal query semantics | Hide widening or proof debt |
| Runtime plane | Planning, retrieval, merge, degradation, execution over projected truth | material-operation receipts, operator effects, proof budgets | Become a second durable truth store |
| Tool/execution plane | Tool invocation history, route, retries, deadlines, results | execution context and tool receipts | Claim domain truth without import |
| Proof/conformance plane | Proof history, reference behavior, proof obligations | proof profiles, debt ledger, conformance receipts | Promote domain truth directly |
| Advisory/learning plane | Ranking, scheduling, abstention, calibration | learned-output disclosure and training-label lineage | Rewrite semantics or promotion law |
| Governance plane | approvals, vetoes, waivers, release gates | proof waivers, human veto, release-bar evidence | Hide policy exceptions |

---

## 5. Artifact lifecycle law

### 5.1 Canonical lifecycle states

Every material artifact family MUST define a lifecycle compatible with the following states:

| State | Meaning |
|---|---|
| `created` | Artifact has been produced but not yet admitted. |
| `validated` | Artifact satisfies syntax/shape/canonicalization checks. |
| `admitted` | Artifact is allowed into the local runtime under policy. |
| `projected` | Artifact has a projection-visible representation. |
| `proposed` | Artifact is advisory or candidate-bearing only. |
| `verified` | Artifact has satisfied its declared proof profile. |
| `refuted` | Artifact failed a refuter or contradiction check. |
| `contradicted` | Artifact coexists with support and contradiction. |
| `quarantined` | Artifact is isolated pending proof, repair, admission, or dispute. |
| `promoted` | Artifact is trusted for a declared authority scope through lawful path. |
| `superseded` | Artifact has been replaced by later artifact while history remains queryable. |
| `retired` | Artifact is no longer active for ordinary use but remains historically queryable. |
| `subtracted` | Artifact or structure has been lawfully removed from active representation under subtraction receipt. |
| `disputed` | Artifact is under human, policy, treaty, proof, or semantic challenge. |

### 5.2 State transition requirements

Every state transition MUST have:

- triggering operator or policy;
- actor/agent/runtime identity;
- recorded time;
- affected artifact refs;
- previous state;
- new state;
- proof/degradation/waiver refs if relevant;
- execution context refs if the transition was produced by runtime work;
- digest-preserving backpointers.

Silent state changes are forbidden.

### 5.3 Promotion eligibility

An artifact MAY become promotion-eligible only if:

1. its artifact family is allowed to participate in promotion;
2. its schema version is admitted;
3. its source authority is admissible;
4. its bitemporal coordinates are valid or explicitly degraded;
5. its execution evidence is present or explicitly non-applicable;
6. its proof profile is satisfied or explicitly waived by lawful governance;
7. no unresolved higher-priority contradiction blocks promotion;
8. any boundary repairs are disclosed and treatment-integrity checked.

---

## 6. Canonical semantic product

Every materially important claim-like output MUST be representable as a product of semantic carriers rather than a naked payload plus score.

Let the v11A semantic product be:

```text
S = P × T × B × I × X × V × E × N × G
```

Where:

- **P — provenance/support carrier**: alternative support, conjunctive support, recursive support, superseded support, degraded support, proof-linked support.
- **T — truth-state carrier**: supported, contradicted, underdetermined, both-supported-and-contradicted, superseded, quarantined, degraded-but-queryable, refuted.
- **B — bitemporal carrier**: valid-time coordinates, recorded-time coordinates, supersession lineage, as-of query semantics, replay coordinates.
- **I — identity carrier**: episode ID, claim ID, relation ID, derivation ID, repair lineage, canonical backpointers, namespace.
- **X — execution-evidence carrier**: trace context, attempt family, queue lineage, tool/provider route, replay handle, receipt family, control decision linkage.
- **V — view carrier**: semantic, temporal, entity, causal, repair, control, execution, audit/explain, degraded view.
- **E — exactness/proof carrier**: approximate, bounded exact, replay exact, checked, refuted, unknown, proof debt, proof waiver, exactness budget.
- **N — nuisance/degradation carrier**: stale context, retry/deadline contamination, provider instability, retrieval-path contamination, parser repair involvement, missing evidence, truncation.
- **G — governance/authority carrier**: owner plane, authority class, approval state, waiver state, admission state, dispute state, user/human veto state where relevant.

### 6.1 Required semantic object surfaces

The stack MUST define canonical logical types for:

- `SemanticStateV1`
- `ClaimStateV1`
- `RelationStateV1`
- `QueryAnswerStateV1`
- `RepairCandidateStateV1`
- `ContradictionStateV1`
- `ExecutionResultStateV1`
- `BoundaryResultStateV1`

### 6.2 Scalar collapse prohibition

No implementation may collapse `S` into one scalar confidence if doing so hides any of:

- contradiction;
- time scope;
- proof debt;
- exactness class;
- degraded view;
- execution contamination;
- parser/boundary repair;
- admission state;
- authority mismatch;
- unresolved dispute.

### 6.3 Algebraic guidance

Implementations SHOULD use compositional carriers:

- provenance/support through symbolic expressions, circuits, semiring-like structures, or equivalent;
- truth state through contradiction-tolerant bilattice-like structures or equivalent;
- time through explicit bitemporal ranges;
- exactness/degradation through finite partially ordered classes;
- governance through explicit state machines;
- execution through receipt-linked DAGs or equivalent lineage structures.

The physical representation MAY vary. The logical distinctions MUST NOT.

---

## 7. Operator effect system

### 7.1 Required effect declaration

Every material operator MUST declare its effects using `OperatorContractV1`.

At minimum, the effect set MUST be able to distinguish:

| Effect | Meaning |
|---|---|
| `READS_TRUTH` | Reads authoritative or projected truth. |
| `PROJECTS_TRUTH` | Produces or updates queryable projection state. |
| `PROPOSES_INFERENCE` | Emits advisory/inferred outputs. |
| `EMITS_RECEIPT` | Emits execution/proof/governance receipts. |
| `WIDENS_VIEW` | Expands scope, relaxes identity, degrades time, or falls back semantically. |
| `REPAIRS_STATE` | Proposes or applies repair. |
| `SUBTRACTS_STRUCTURE` | Removes, compacts, summarizes, or minimizes active structure. |
| `CHANGES_PROMOTION` | Affects trust/promotion status. |
| `CHANGES_SCHEMA` | Changes schema, schema interpretation, migration, or compatibility. |
| `CROSSES_TRUST_BOUNDARY` | Imports, exports, admits, or relies on external artifacts. |
| `AFFECTS_FUTURE_EXECUTION` | Changes scheduling, ranking, budgets, region choice, or policy for later runs. |
| `AFFECTS_USER_AGENCY` | Produces personalized, consequential, repeated, persuasive, or action-guiding output. |

### 7.2 Operator contract fields

`OperatorContractV1` MUST include:

- operator id;
- operator family;
- owner plane;
- input artifact families;
- output artifact families;
- allowed effects;
- forbidden effects;
- preconditions;
- postconditions;
- proof obligations;
- exactness class;
- degradation behavior;
- boundary compiler profile if applicable;
- replay requirements;
- failure taxonomy;
- conformance surface;
- human-approval requirement if applicable.

### 7.3 Effect safety rules

An operator MUST NOT perform effects absent from its contract.

An operator with `WIDENS_VIEW` MUST emit `ViewDisclosureV1` and `DegradationRecordV1` when guarantees weaken.

An operator with `REPAIRS_STATE` MUST emit repair candidate and/or repair execution receipts.

An operator with `SUBTRACTS_STRUCTURE` MUST conform to v11B lawful subtraction once v11B is in force.

An operator with `CHANGES_PROMOTION` MUST require proof profile satisfaction or lawful waiver.

An operator with `AFFECTS_USER_AGENCY` MUST conform to the v11C agency-preservation hooks if user-facing consequential use is enabled.

---

## 8. Execution evidence law

### 8.1 Execution context is artifact semantics

For material operations, execution context MUST be recorded as `ExecutionContextEnvelopeV1` or an equivalent admitted artifact.

Required fields:

- execution id;
- trace id;
- span/operation id if applicable;
- attempt family id;
- retry family id;
- queue/message lineage if applicable;
- provider/tool route;
- environment/runtime fingerprint;
- start/end recorded time;
- deadline/budget allocation;
- budget consumption;
- timeout/cancellation state;
- truncation state;
- degradation/widening state;
- replay handle or non-replayability reason;
- redaction/disclosure state.

### 8.2 Tool call receipts

Every material tool call MUST emit `ToolCallReceiptV1` with:

- tool id;
- call id;
- call input digest;
- call output digest;
- status;
- error taxonomy;
- latency;
- provider route;
- environment fingerprint;
- redaction state;
- replayability;
- affected artifact refs.

### 8.3 Done-state prohibition

A runtime MUST NOT claim user-visible completion for material work unless:

- required receipts exist;
- failure/degradation is disclosed;
- output artifact refs are available;
- proof debt or proof waiver is visible if relevant.

“Done” without receipts is non-conforming.

---

## 9. Proof economy

### 9.1 Proof is finite, but proof debt must be explicit

v11A recognizes that not every artifact can receive maximal proof. However, the stack MUST NOT hide proof gaps.

Every risk-bearing artifact MUST have a `ProofProfileV1`.

Every missing or deferred proof requirement MUST be recorded in `ProofDebtLedgerV1` or explicitly waived through lawful governance.

### 9.2 Proof classes

A conforming stack MUST distinguish at least:

| Class | Meaning | Promotion eligibility |
|---|---|---|
| `none` | No proof required for this artifact family/use. | Only if family/use is non-risk-bearing. |
| `receipt_only` | Execution receipt exists; no semantic proof. | Advisory only unless policy says otherwise. |
| `witnessed` | Witness supports result. | Limited promotion possible if proof profile allows. |
| `checked` | Independent checker validated relevant properties. | Promotion possible within declared scope. |
| `refuted` | Refuter found failure. | Promotion blocked unless repaired/superseded. |
| `oracle_exact` | Exact or conservative oracle validated bounded slice. | Strong promotion possible within slice. |
| `externally_attested` | External attestation admitted. | Depends on admission policy. |
| `waived` | Proof skipped through lawful waiver. | Must remain queryable as waiver, not verified truth. |

### 9.3 Proof debt ledger

`ProofDebtLedgerV1` MUST include:

- debt id;
- target artifact;
- missing proof obligation;
- reason;
- risk tier;
- allowed uses while debt exists;
- expiry condition;
- escalation path;
- waiver refs if applicable;
- current status.

### 9.4 Proof waivers

Proof waivers MUST be explicit and queryable. A waiver is not proof.

A proof waiver MUST include:

- authorizing actor/policy;
- target obligation;
- reason;
- allowed scope;
- expiry;
- rollback/challenge path;
- user-visible disclosure if user-facing.

---

## 10. Boundary compiler law

### 10.1 Boundary is language recognition

Every structured boundary MUST be treated as a compiler front, not tolerant parsing.

This includes:

- JSON/XML/YAML or other structured payloads;
- patch formats;
- schema migrations;
- tool outputs;
- model structured outputs;
- evidence imports;
- generated spec artifacts;
- external attested artifacts.

### 10.2 Boundary compiler profile

`BoundaryCompilerProfileV1` MUST declare:

- language/dialect;
- schema id;
- schema version;
- canonicalization profile;
- duplicate-key/ambiguous-field policy;
- number/string/null coercion policy;
- unknown field policy;
- repair policy;
- resource ceilings;
- trust boundary;
- treatment-critical paths;
- allowed degradation behavior.

### 10.3 Repair receipt

If repair occurs, `RepairReceiptV1` MUST include:

- before digest;
- after digest;
- repaired paths;
- repair operator;
- repair rationale;
- semantic impact classification;
- treatment-integrity result;
- downstream proof impact;
- replay handle.

### 10.4 Treatment integrity

Any parser, patcher, repairer, or schema migrator that changes treatment-critical fields MUST emit `TreatmentIntegrityReceiptV1`.

If treatment-critical semantics changed and no lawful waiver exists, the output MUST be quarantined or rejected.

### 10.5 Liberal acceptance prohibition

A conforming boundary MUST NOT silently accept malformed, ambiguous, dialect-mismatched, duplicate-key, resource-exhausting, or semantically widened input.

It MAY admit repaired input only with explicit repair artifacts and proof impact.

---

## 11. View and query disclosure law

### 11.1 View family must be explicit

Any query answer or runtime retrieval result that may influence decision, repair, promotion, or user-visible output MUST disclose its view family.

Required view families include at minimum:

- semantic view;
- temporal view;
- entity view;
- causal view;
- repair view;
- control/execution view;
- proof/audit view;
- degraded/widened view.

### 11.2 Widening is a policy event

Widening includes:

- fuzzy entity expansion;
- time-bound relaxation;
- semantic fallback;
- missing index fallback;
- lower-exactness retrieval;
- approximate matching;
- unverified alias use;
- missing proof fallback;
- truncated context fallback.

Any widening MUST emit `ViewDisclosureV1` and, where guarantees weaken, `DegradationRecordV1`.

### 11.3 As-of query semantics

Bitemporal query semantics MUST distinguish:

- `valid_as_of` — what the claim applies to in world/application time;
- `recorded_as_of` — what the system had recorded/believed at a system time;
- `query_executed_at` — when the query ran;
- `view_built_as_of` — which projection/index/derived view version was used.

A query that cannot satisfy these MUST degrade explicitly.

---

## 12. Reference interpreter and conformance law

### 12.1 Reference behavior is required for hard semantics

The stack MUST define reference interpreters or equivalent reference behavior for:

- bitemporal query semantics;
- artifact lifecycle transitions;
- bridge import atomicity and digest preservation;
- view widening/degradation;
- boundary compiler behavior;
- proof profile eligibility;
- semantic product composition;
- execution receipt linkage;
- schema compatibility checks.

v11B extends this list for regions, convergence, repair, and subtraction.

### 12.2 Conformance corpus

Every reference surface MUST have:

- golden fixtures;
- negative fixtures;
- edge-case fixtures;
- generated/property fixtures where applicable;
- expected outputs;
- explicit unsupported/ambiguous cases;
- drift budget if exact equality is not possible.

### 12.3 Differential implementation checks

Production implementations MUST be continuously checked against reference behavior.

Any divergence MUST be classified as:

- permitted implementation difference;
- reference bug;
- implementation bug;
- ambiguous spec clause;
- unsupported case;
- release-blocking semantic drift.

### 12.4 Release-blocking surfaces

A divergence is release-blocking if it affects:

- authority boundary;
- time semantics;
- artifact lifecycle;
- proof eligibility;
- promotion eligibility;
- boundary repair;
- execution receipt completeness;
- schema compatibility;
- user-visible completion truth.

---

## 13. Schema and contract governance

### 13.1 Type-owned contracts

Wire-visible contracts SHOULD be owned by explicit type/schema owner crates or modules. A schema adjacent to code but not governed by code ownership is a drift risk.

### 13.2 Versioned artifact families

Every artifact family MUST be versioned.

Breaking changes MUST create a new version or lawful migration path. Reusing field identity with changed meaning is forbidden.

### 13.3 Compatibility modes

The stack MUST distinguish at least:

- backward compatible;
- forward compatible;
- full compatible;
- transitive backward compatible;
- transitive forward compatible;
- incompatible but migratable;
- incompatible and non-migratable.

### 13.4 Canonicalization and digest law

Artifacts intended for comparison, signing, attestation, deduplication, or replay MUST declare canonicalization and digest profiles.

Digest identity MUST distinguish:

- logical artifact identity;
- byte/content identity;
- canonical representation identity;
- schema identity;
- provenance identity.

---

## 14. Control and governance receipts

### 14.1 Control decisions are artifacts

Approvals, vetoes, rollbacks, waivers, abstentions, escalations, and release decisions MUST produce governance receipts.

### 14.2 Minimum fields

A governance receipt MUST include:

- decision id;
- decision kind;
- actor/policy;
- target artifact(s);
- reason;
- evidence refs;
- proof/debt refs;
- effective scope;
- valid/recorded time;
- expiry/review condition;
- challenge path.

### 14.3 No invisible waiver

Any exception to proof, schema, boundary, time, identity, or authority law MUST be visible as a waiver artifact.

---

## 15. v11A crate responsibility map

This is logical and may map to different physical crate names.

| Surface | Likely owner(s) | v11A responsibility |
|---|---|---|
| IDs/digests | `stack-ids` | artifact IDs, canonical refs, digest types, trace primitives. |
| Contracts/schemas | `contract-schema-gen` or artifact contract crate | artifact family schemas, schema compatibility, schema metadata, generated docs. |
| Evidence/export | `semantic-memory-forge`, export surfaces | episode/evidence packages, raw truth envelopes, evidence fixity. |
| Bridge/import | `forge-memory-bridge` | atomic import receipts, digest preservation, boundary compiler integration. |
| Projection/memory | `semantic-memory` | semantic product projection, bitemporal query semantics, view disclosure. |
| Runtime | `knowledge-runtime` | material operation receipts, view/query execution, degradation disclosure. |
| Tool runtime | `llm-tool-runtime` | tool receipts, provider route, execution context, replay linkage. |
| Control | `verification-*`, `forge-pilot` | proof profiles, governance receipts, control decisions, waiver/debt handling. |
| Conformance | dedicated harness or workspace tests | reference interpreters, fixture corpora, differential checks. |

---

## 16. v11A release bar

A release may claim v11A compliance only if all of the following are true:

1. `ArtifactEnvelopeV1`, `ArtifactManifestV1`, `ExecutionContextEnvelopeV1`, `OperatorContractV1`, `OperatorInvocationReceiptV1`, `ProofProfileV1`, and `DegradationRecordV1` exist as canonical logical contracts.
2. Material operations emit invocation receipts.
3. Tool calls that affect artifacts emit tool receipts.
4. Boundary compiler profiles exist for structured import/export/tool-output surfaces.
5. Boundary repairs emit repair receipts and treatment-integrity receipts where applicable.
6. Bitemporal query semantics have reference behavior and fixtures.
7. Artifact lifecycle transitions have receipts or auditable transition records.
8. Proof debt is explicit for risk-bearing outputs.
9. View widening is explicit and queryable.
10. Schema compatibility checks are release-gated.
11. At least one production path is differentially checked against a reference interpreter.
12. No user-visible “done” state for material work is emitted without receipts.

---

## 17. v11A non-goals

v11A does not require full regional message passing. That belongs to v11B.

v11A does not require lawful subtraction implementation. It requires hooks and effect declarations; v11B makes subtraction first-class.

v11A does not require full federation, theory search, agency governance, or constitutional self-hosting. v11C reserves those surfaces.

v11A does not require every output to be fully proven. It requires proof profile, proof debt, and waiver honesty.

---

## 18. Design warning

A v11A stack that produces beautiful artifacts but cannot enforce operator effects is not compliant.

A v11A stack that emits receipts but leaves proof debt invisible is not compliant.

A v11A stack that has bitemporal fields but cannot answer reference `as_of(valid, recorded)` fixtures is not compliant.

A v11A stack that parses structured outputs leniently and calls it robustness is not compliant.

A v11A stack that lets learned ranking or runtime convenience silently rewrite semantic law is not compliant.

The constitutional core is not paperwork. It is the runtime’s immune system.
