# v11+ Artifact Family Index

**Status:** Companion reference  
**Purpose:** Define the artifact families introduced, hardened, or reserved by v11+.

This index is not a replacement for the A/B/C specs. It is a compact reference for artifact names, release stage, authority, and minimum required fields.

## 1. Authority legend

| Label | Meaning |
|---|---|
| Truth-authoritative | May participate directly in truth-bearing import/promotion only through existing lawful authority paths. |
| Projection-authoritative | Authoritative only for queryable projected truth in `semantic-memory` or equivalent projection lane. |
| Execution-authoritative | Authoritative only for execution history/receipts, not domain truth. |
| Proof-authoritative | Authoritative only for proof history, proof status, proof obligations, or proof debt. |
| Advisory | May influence scheduling, ranking, warnings, or proposals but cannot self-promote. |
| Horizon | Reserved for future v11C+ surfaces; must not be smuggled into v11A/B authority. |

## 2. v11A core artifact families

| Artifact | Stage | Authority | Purpose | Minimum required fields |
|---|---|---|---|---|
| `ArtifactEnvelopeV1` | A | Depends on enclosed family | Canonical wrapper around a material artifact. | artifact family, schema version, content digest, issuer, created recorded time, namespace, authority class, bitemporal applicability, signature/attestation refs if present. |
| `ArtifactManifestV1` | A | Execution/proof | Lists artifacts involved in an operation. | input refs, output refs, digests, schema identities, canonicalization profile, missing/opaque refs. |
| `EpisodeBundleV1` | A | Truth-authoritative only via import lane | Canonical identity-bearing evidence unit. | episode id, evidence anchors, valid time, recorded time, claims, relations, backpointers, execution refs, fixity digests. |
| `ClaimEvidencePackageV1` | A | Truth-authoritative only via import lane | Cross-plane claim/evidence package. | claim id, episode id, treatment/outcome if causal, support refs, contradiction refs, proof profile, verification state, bitemporal coordinates. |
| `SemanticStateV1` | A | Projection/proof | Product-state representation for claim-like outputs. | provenance carrier, truth carrier, bitemporal carrier, identity carrier, execution carrier, view carrier, exactness/proof carrier, nuisance/degradation carrier, governance carrier. |
| `OperatorContractV1` | A | Proof/execution | Declares what an operator may read, write, widen, infer, repair, or subtract. | operator id, owner plane, input artifact families, output artifact families, effect set, preconditions, postconditions, proof obligations, reference behavior, failure modes. |
| `OperatorInvocationReceiptV1` | A | Execution-authoritative | Receipt for one operator invocation. | invocation id, operator id, input manifest, output manifest, execution context, effect summary, budget use, degradation, proof/refutation refs, error taxonomy. |
| `ExecutionContextEnvelopeV1` | A | Execution-authoritative | Meaning-bearing execution context. | trace id, attempt family id, provider/tool route, queue lineage, deadline, budget, retry lineage, replay handle, truncation/widening/degradation flags. |
| `ToolCallReceiptV1` | A | Execution-authoritative | Canonical receipt for tool calls. | tool id, call id, inputs, outputs, digests, status, latency, error, provider route, redaction state, replayability. |
| `BudgetLedgerEntryV1` | A | Execution/proof | Records budget decisions. | budget id, allocation, consumption, exhaustion, waiver, escalation, decision maker, recorded time. |
| `ProofProfileV1` | A | Proof-authoritative | Declares proof requirements for an output or operator. | proof class, admissible evidence, required refuters, exactness tier, promotion eligibility, expiry, debt handling. |
| `ProofObligationSetV1` | A | Proof-authoritative | Set of proof obligations attached to an artifact/action. | obligations, owner, due event/time, discharge criteria, waiver rules, dependency refs. |
| `ProofDebtLedgerV1` | A | Proof-authoritative | Tracks explicit proof debt. | debt id, affected artifact, reason, risk tier, allowed use, expiry, escalation path, status. |
| `WitnessBundleV1` | A | Proof-authoritative | Supports a claim or operator result. | witness kind, supporting artifacts, derivation, checker, exactness, failure scope. |
| `CertificateBundleV1` | A | Proof-authoritative | Stronger proof/certificate artifact. | certificate type, verifier, assumptions, checked artifacts, result, digest, validity window. |
| `RefutationBundleV1` | A | Proof-authoritative | Records attempted or successful refutation. | refuter type, target, method, input set, result, residual risk, replay handle. |
| `SemanticDiffV1` | A | Proof/projection | Difference between semantic states. | before ref, after ref, carrier deltas, proof impact, promotion impact, time coordinates. |
| `DegradationRecordV1` | A | Execution/proof | Explicit degradation event. | degraded guarantee, reason, affected artifact, scope, user/runtime visibility, allowed downstream use. |
| `ViewDisclosureV1` | A | Execution/projection | Declares which view model answered a query. | view family, time coordinates, widening policy, retrieval mode, omitted guarantees, degradation refs. |
| `BoundaryCompilerProfileV1` | A | Proof/execution | Declares an input/output language boundary. | language/dialect, schema id, canonicalization, parser profile, repair policy, resource ceilings, duplicate/ambiguous handling. |
| `ParseReceiptV1` | A | Execution/proof | Records parsing at a boundary. | raw digest, parsed digest, parser, dialect, errors, ambiguity, canonical digest. |
| `RepairReceiptV1` | A | Execution/proof | Records structured-output repair. | repair operator, before/after digests, changed paths, semantic impact, allowed/disallowed changes, treatment-integrity status. |
| `TreatmentIntegrityReceiptV1` | A | Proof/execution | Confirms boundary/patch/repair did not mutate the effective treatment without declaration. | treatment-critical fields, before/after hashes, differences, decision, waiver if any. |
| `ReferenceInterpreterBundleV1` | A | Proof/conformance | Reference semantics for a surface. | surface id, interpreter version, fixture corpus, oracle outputs, supported dialects, drift budget. |
| `ConformanceRunReceiptV1` | A | Proof/conformance | Result of implementation vs reference test. | implementation id, reference id, fixture set, pass/fail, divergence, drift classification, release impact. |

## 3. v11B regional, recursive, subtractive artifacts

| Artifact | Stage | Authority | Purpose | Minimum required fields |
|---|---|---|---|---|
| `GraphSurfaceDeclarationV1` | B | Execution/proof | Declares selected graph surface and compilation basis. | graph kind, source artifacts, compiler id, information-loss declaration, digest, region refs. |
| `CompiledGraphBundleV1` | B | Execution/proof | Content-addressed compiled graph. | graph id, graph kind, variables/nodes/factors/hyperedges summary, compiler receipt, source manifest, digest. |
| `RegionContractV1` | B | Execution-authoritative | Defines a bounded execution region. | region id, kind, scope, graph surfaces, operators, time bounds, exactness ceiling, budget ceiling, stop rules, incoming/outgoing artifacts. |
| `RegionStateSnapshotV1` | B | Execution/proof | Replay-linked region state. | observed state, latent state, nuisance state, contradiction state, exactness state, budget state, proof/witness refs, execution refs. |
| `RegionBoundaryMessageV1` | B | Execution-authoritative | Typed artifact crossing region boundary. | source region, destination region, artifact family, payload ref, time coords, digest, budget impact, acceptance policy. |
| `RegionBoundaryReceiptV1` | B | Execution-authoritative | Receipt for boundary transfer. | message ref, accept/reject/quarantine, reason, digest, replay link, receiving state impact. |
| `RegionReplaySliceV1` | B | Execution/proof | Minimal replay slice for region work. | region id, input artifact set, policy set, execution context, graph bundle, expected outputs, exactness class. |
| `DeltaEnvelopeV1` | B | Execution/projection | Incremental change envelope. | source, destination, change class, time coordinate, invalidation cone, exactness, replay coords, digest. |
| `ResidualEnvelopeV1` | B | Proof/execution | Residual emitted by approximate/fixpoint step. | residual kind, magnitude/structure, target variables/factors, schedule priority, convergence impact. |
| `SyndromeEnvelopeV1` | B | Proof/execution | Constraint violation/contradiction surface. | violated constraints, support/refutation refs, locality, severity, repair candidates, oracle escalation. |
| `NuisanceEnvelopeV1` | B | Execution/proof | Modeled nuisance state. | nuisance kind, measurement/execution source, scope, effect on interpretation, calibration state. |
| `ConvergenceReportV1` | B | Proof/execution | Damping/stop/oscillation report. | schedule, damping, residual thresholds, iterations, stop reason, oscillation markers, proof impact. |
| `ClosureMinimizationLoopV1` | B | Execution/proof | Governed loop over infer/residualize/subtract/verify/repair/propagate. | loop id, stages, inputs, outputs, stop rules, proof profile, subtraction receipts, repair refs. |
| `SubtractionOperatorContractV1` | B | Proof/execution | Contract for lawful removal/compaction/minimization. | removable families, protected invariants, history budget, proof obligations, replay requirements, challenge path. |
| `SupportCoreV1` | B | Proof-authoritative | Minimal/near-minimal support necessary for a claim/query. | target, support members, minimality class, proof/checker, excluded support, replay path. |
| `RemovalFrontierV1` | B | Proof/execution | Boundary between load-bearing and removable structure. | candidate removals, invariant impact, risk tier, protected refs, challenge refs. |
| `InvariantPreservationReceiptV1` | B | Proof-authoritative | Proof/receipt that subtraction preserved declared invariants. | invariants, before/after refs, checker, result, exceptions, historical-loss impact. |
| `HistoricalLossBudgetV1` | B | Governance/proof | Declares allowable loss under compaction/forgetting. | protected queries, allowed summaries, forbidden loss, retention horizon, challenge policy. |
| `SubtractionChallengeV1` | B | Proof/governance | Challenge to a subtraction result. | challenged subtraction ref, failed invariant/query, evidence, requested action. |
| `RepairCandidateBundleV1` | B | Advisory/proof | Candidate local repair. | target syndrome, repair action, blast radius, proof obligations, rollback path, quarantine policy. |
| `RepairExecutionReceiptV1` | B | Execution/proof | Receipt for applied repair. | repair id, candidate ref, affected artifacts, before/after semantic diff, proof/refutation status, rollback handle. |
| `CausalAttributionBundleV1` | B | Proof/projection | Causal attribution package. | treatment, outcome, unit, covariates/confounders, identification assumptions, estimator, refuters, replay plan. |
| `InterventionPlanV1` | B | Proof/execution | Planned intervention/check. | treatment, control/baseline, workload slice, expected outcome, safety gates, stop rules. |
| `CounterfactualReplayPlanV1` | B | Proof/execution | Replay for counterfactual comparison. | baseline refs, altered refs, controlled variables, expected comparable outputs, confounder controls. |

## 4. v11C horizon and future-admission artifacts

| Artifact | Stage | Authority | Purpose | Minimum required fields |
|---|---|---|---|---|
| `SpecBundleV1` | C | Governance/horizon | Packaged spec artifact. | spec id, version, normative sections, source basis, compatibility claims, digest. |
| `NormativeASTV1` | C | Governance/horizon | Machine-readable normative structure. | clauses, modalities, referenced artifacts, obligations, prohibitions, precedence rules. |
| `SpecCompilerReceiptV1` | C | Governance/horizon | Receipt for compiling spec to schemas/tests/interpreters. | compiler version, input spec digest, outputs, errors, human-review status. |
| `GeneratedSchemaBundleV1` | C | Governance/horizon | Generated schema pack. | schema ids, source clauses, compatibility report, validation results. |
| `GeneratedConformanceCorpusV1` | C | Governance/horizon | Generated tests/fixtures. | fixture ids, covered clauses, expected behavior, unsupported/ambiguous clauses. |
| `AmendmentSimulationV1` | C | Governance/horizon | Simulation of a spec change. | change proposal, affected artifacts, migration impact, proof impact, rollout risk. |
| `HumanVetoReceiptV1` | C | Governance/horizon | Human veto/challenge record. | actor, authority, target, reason, time, resulting state. |
| `AttestedArtifactEnvelopeV1` | C | Execution/proof/horizon | External artifact with attestation. | issuer, trust root, artifact digest, statement type, disclosure policy, verification result. |
| `AdmissionPolicyV1` | C | Governance/horizon | Policy for admitting external artifacts. | trust roots, allowed families, verification requirements, quarantine/default behavior. |
| `ExternalArtifactQuarantineV1` | C | Governance/horizon | Quarantine state for external artifacts. | artifact ref, reason, missing evidence, challenge path, expiry/escalation. |
| `CrossRuntimeEquivalenceBundleV1` | C | Governance/proof | Claims equivalence across runtimes. | local refs, remote refs, equivalence class, evidence, dissent, challenge path. |
| `TreatyBundleV1` | C | Horizon | Federated sharing/settlement agreement. | parties, admitted families, disclosure rules, quorum/evidence rules, dispute process. |
| `SettlementCaseV1` | C | Horizon | Federated dispute/settlement case. | contested claim/artifact, parties, evidence, dispositions, dissent, replay links. |
| `MechanismBundleV1` | C | Advisory/proof/horizon | Candidate mechanism/theory object. | mechanism id, structure, assumptions, data refs, simulator refs, refuter suite, status. |
| `TheoryVersionV1` | C | Advisory/proof/horizon | Versioned theory state. | theory id, version, mechanism refs, supersession, evidence, disputes. |
| `HypothesisLibraryV1` | C | Advisory/horizon | Search library of candidate forms. | library id, families, priors/constraints, admissible operators, exclusion rules. |
| `SimulatorContractV1` | C | Execution/proof/horizon | Contract for simulator/forward model. | inputs, outputs, invariants, stochasticity, replayability, environment, validation suite. |
| `FitRunReceiptV1` | C | Execution/proof/horizon | Receipt for mechanism/theory fitting. | model refs, data refs, optimizer, seeds, metrics, uncertainty, failure modes. |
| `TheoryRefuterSuiteV1` | C | Proof/horizon | Refuter suite for mechanism/theory. | refuters, negative controls, invariance checks, stress tests, pass/fail. |
| `InfluenceClassV1` | C | Governance/agency | Classifies influence/advice risk. | influence kind, personalization, repetition, urgency, vulnerability context, reversibility. |
| `AdviceEnvelopeV1` | C | Governance/agency | Wraps consequential personalized advice. | advice target, evidence basis, uncertainty, alternatives, agency safeguards, disclosure. |
| `AgencyReceiptV1` | C | Governance/agency | Records agency-preserving handling. | influence class, gating decision, user consent/override, repetition budget, refusal/redirect if any. |
| `MemoryInfluenceTraceV1` | C | Governance/agency | Shows memory’s role in influence/advice. | memory refs, personalization path, withheld/used context, user-visible disclosure. |
