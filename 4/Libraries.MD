# Libraries Workspace Snapshot -- 2026-04-04

**Total Rust LOC:** ~125,725 (source + tests, excluding excluded ecosystem crates)
**Workspace members:** 30 crates (+ 10 Primitives subcrates excluded from default workspace)
**JSON schemas generated:** 211 files
**Gate/audit scripts:** 50+
**Rust edition:** 2021 | **Resolver:** 2 | **All crates:** `publish = false`

**Workspace-level lint enforcement:**
- `unsafe_code = "deny"`
- `clippy::todo = "deny"`, `clippy::dbg_macro = "deny"`, `clippy::unimplemented = "deny"`
- `clippy::expect_used = "warn"`

---

## Table of Contents

1. [Foundational Crates](#1-foundational-crates)
2. [Kernel Infrastructure](#2-kernel-infrastructure)
3. [Memory & Bridge Layer](#3-memory--bridge-layer)
4. [Verification Stack](#4-verification-stack)
5. [Governance Surface Crates](#5-governance-surface-crates)
6. [Orchestration & Tooling](#6-orchestration--tooling)
7. [Primitives Workspace](#7-primitives-workspace-excluded-from-default-build)
8. [Dependency Graph](#8-dependency-graph)
9. [Summary Matrix](#9-summary-matrix)

---

## 1. Foundational Crates

### stack-ids v0.1.0
**Role:** Single source of truth for all cross-crate identity, scope, trace, and digest primitives.

**No workspace dependencies** -- every other crate depends on this one.

| Area | Types |
|------|-------|
| IDs (50+) | `EnvelopeId`, `ClaimId`, `ClaimVersionId`, `EntityId`, `EpisodeId`, `AttemptId`, `TrialId`, `ArtifactId`, `ProjectionId`, `RelationId`, `KernelRunId`, `ConstraintId`, `HyperedgeId`, `ResidualId`, `SyndromeId`, `WitnessId`, `CertificateId`, `OracleSliceId`, `RefutationResultId`, `OperatorId`, `VerificationCaseId`, `CheckPlanId`, `ControlReceiptId`, `PolicyDecisionId`, `PromotionDecisionId`, `ReleaseReadinessDecisionId`, `RollbackPlanId`, `CalibrationSnapshotId`, `NuisanceStateId`, `RegionId`, `ApprovalRecordId`, `ExecutionPermitId`, `ContinuityExceptionId`, many more |
| Scope | `Scope`, `ScopeKey` -- builder pattern (`new`, `with_domain`, `with_workspace`, `with_repo`) |
| Trace | `TraceCtx`, `BaggageEntry` -- W3C trace-context compatible, max 16 entries, 256 bytes each |
| Digest | `ContentDigest`, `DigestBuilder` -- BLAKE3-based content addressing |
| Status | `SurfaceStatus` enum (surface publication lifecycle) |
| V25 | `V25ConstitutionCitation` (applicability, profile, composition, effective constitution, compiled obligations, exceptions) |

Every ID type: `new(string)`, `generate()` (UUID v4), `as_str()`, `is_empty()`, `Display`, `From<String>`, `FromStr`. Opaque string-wrapper design prevents direct inner parsing.

**LOC:** ~3,383 | **Maturity:** Production/Core

---

### recursive-kernel-core v0.1.0
**Role:** Non-authoritative recursive inference kernel schemas and operator contracts.

**Deps:** stack-ids

| Category | Types |
|----------|-------|
| Enums | `ExactnessClass` (Exact, Conservative, Approximate), `ConvergenceKind` (AcyclicSinglePass, ResidualBounded, FixedPoint), `StopRule`, `DegradationBehavior`, `ArtifactAuthorityClass` (NonAuthoritativeDerived), `KernelArtifactKind`, `KernelRefutationOutcome` |
| Structs | `OperatorContract`, `OperatorMetadata`, `KernelRun`, `ConstraintUnit`, `Syndrome`, `ResidualArtifact`, `WitnessArtifact`, `CertificateArtifact`, `CalibrationReport`, `PerturbationWitness`, `KernelRefutationResult` |
| Functions | `constraint_compiler_operator()`, `message_passing_operator()`, `OperatorMetadata::validate()` |
| Constants | `CONSTRAINT_COMPILER_OPERATOR_ID`, `RECURSIVE_MESSAGE_PASSING_OPERATOR_ID` |

**LOC:** ~583 | **Tests:** operator contracts, authority markers, serialization | **Maturity:** Early/Stable

---

## 2. Kernel Infrastructure

### constraint-compiler v0.1.0
**Role:** Compiles projection import batches into bounded inference graphs for the recursive kernel.

**Deps:** blake3, serde, serde_json, schemars, forge-memory-bridge, recursive-kernel-core, semantic-memory-forge, stack-ids

**Public API:**
- `compile_batch(batch: &ProjectionImportBatchV3, policy: &CompilerPolicy) -> CompileOutput`
- Types: `CompilerPolicy`, `InferenceNode`, `InferenceHyperedge`, `InvalidationCone`, `OracleSliceCandidate`, `CompilationBoundary`, `GraphGeometryManifest`, `CompiledRegion`, `CompileOutput`
- Enums: `ConstraintDegradation`, `GraphSurfaceKind`
- Helpers for graph geometry, region compilation, invalidation cones, nuisance state materialization

**LOC:** ~1,378 | **Tests:** 17 embedded (determinism, hyperedges, degradation, oracle candidates, geometry, regions, nuisance) | **Maturity:** Core infrastructure

---

### kernel-execution v0.1.0
**Role:** Deterministic K2 execution baseline for recursive inference graphs. Implements core execution modes and convergence governance.

**Deps:** serde, serde_json, schemars, constraint-compiler, recursive-kernel-core, stack-ids

**Execution Modes:**
| Mode | Description |
|------|-------------|
| `AcyclicBaseline` | Single-pass DAG evaluation |
| `MessagePassingBaseline` | Iterative belief propagation |
| `DeltaPropagation` | Incremental delta updates |
| `ResidualCorrection` | Residual-based convergence |
| `MultiscaleScheduler` | Composite multi-strategy scheduling |

**Key Types:**
- `ExecutionBudget` (max_iterations, max_messages, max_nodes, allow_repair)
- `NodeBelief`, `ConstraintMessage`, `ResidualSample` -- inference state
- `ExecutionWitnessArtifact`, `ExecutionCertificateArtifact` -- proof artifacts
- `ExecutionCalibrationReport` -- nuisance/caution/degradation tracking
- `InvalidationManifest` -- recompute triggers + node/region change sets
- `ConvergenceGovernance`, `ConvergenceReport` -- damping, tolerance, iteration limits
- Constants: `FULL_CONFIDENCE_MICROS` (1M), `DEFAULT_FIXED_POINT_TOLERANCE_MICROS` (1K)

**Functions:** `affected_nodes_for_delta()`, `execute_delta_propagation()`, `execute_message_passing_baseline()`, `execute_residual_correction()`, `schedule_execution()`

**LOC:** ~1,216 | **Maturity:** Stable -- pure data + execution abstractions, no IO

---

### kernel-oracles v0.1.0
**Role:** Bounded exact/conservative oracle paths for the recursive inference kernel. Provides oracle assessment, parity checking, temporal replay, and refutation analysis.

**Deps:** chrono, serde, schemars, constraint-compiler, forge-memory-bridge, kernel-execution, recursive-kernel-core, stack-ids

**Oracle Functions:**
| Function | Returns | Purpose |
|----------|---------|---------|
| `evaluate_exact_bounded()` | `Option<OracleAssessment>` | Small exact-bounded graph slice selection |
| `evaluate_conservative()` | `OracleAssessment` | Fallback for any graph |
| `evaluate_delta_parity()` | `DeltaParityAssessment` | Delta vs full-rerun comparison |
| `evaluate_temporal_replay()` | `TemporalReplayAssessment` | Temporal consistency checking |
| `evaluate_causal_refuter()` | `OracleRefutationResult` | Causal flip-witness analysis |
| `minimal_perturbation_witness()` | `OracleRefutationResult` | Minimal removal-set search |

**Types:** `OracleMode` (ExactBounded, ConservativeFallback), `RefutationMode` (CausalRefuter, MinimalPerturbation), `OracleRefutationOutcome` (FlipWitness, NoFlipFound, NotApplicable), `OracleSliceSelectionPolicy`, `OracleAssessment`, `OracleParityCertificate`, `DeltaParityAssessment`, `TemporalReplaySnapshot`, `TemporalReplayAssessment`

**Constants:** `MAX_REMOVAL_SET_SIZE` (4), `MAX_REMOVAL_COMBINATION_BUDGET` (256)

**LOC:** ~1,006 | **Maturity:** Stable -- focused oracle interface, no IO

---

### kernel-conformance v0.1.0
**Role:** Conformance harness for recursive inference kernel authority, compiler, and oracle gates. Validates v9 constitutional freeze and phase-specific conformance gates.

**Deps:** Almost the entire workspace (18 internal crates) + chrono, serde_json, tempfile, tokio

**Phase Gates:**
| Phase | Gate IDs |
|-------|----------|
| Phase 1 | CF-A1, CF-C1, CF-C2, CF-O1, CF-R1 |
| Phase 2 | CF-A3, CF-C3, CF-C4, CF-O2 |
| Phase 3+ | CF-O3, CF-O4, CF-R2--R4, CF-B1--B3, CF-P1--P2 |
| v16--v20 | FED-001, FED-004, THY-001, POR-001, GOV-001, SFX-001 |
| v9 Constitutional | CF-V9-C001 through CF-V9-C017 |

**Reference Interpreters:** 100+ test cases validating constraint compiler output, bridge envelope transformations, kernel execution modes, oracle evaluation, knowledge runtime integration, recursive kernel core semantics, end-to-end roundtrips.

**LOC:** ~1,983 | **Maturity:** Stable -- gate identifiers frozen for v9

---

## 3. Memory & Bridge Layer

### semantic-memory v0.5.0
**Role:** Hybrid semantic search with SQLite, FTS5, and HNSW -- built for AI agents. The queryable projected truth store.

**Deps:** bytemuck, rusqlite (bundled+blob), reqwest, serde, serde_json, tokio, thiserror, tracing, uuid, chrono, schemars, stack-ids, forge-memory-bridge; optional: hnsw_rs

**Architecture:**
- SQLite is **authoritative** -- HNSW is optional acceleration
- WAL-mode + pooled readers for concurrency
- Hybrid BM25 + vector search with Reciprocal Rank Fusion

**Modules:** chunker, config, conversation, db, documents, embedder, episodes, error, graph, hnsw, knowledge, pool, projection_batch/derivation/lane/storage/legacy_compat, quantize, search, storage, tokenizer, types

**Key Types:**
- `MemoryStore`, `MemoryConfig`, `MemoryStats`
- `ExplainedResult`, `SearchResult`, `ScoreBreakdown`
- `Document`, `TextChunk`, `Fact`, `Message`, `Session`, `Episode`
- `GraphView`, `GraphEdge`, `VerificationStatus`
- `Embedder` (trait), `MockEmbedder`, `OllamaEmbedder`
- `SearchSourceType`, `SearchSource`

**Functions:** `search()`, `search_explained()`, `search_conversations()`, projections, import/export, quantization, integrity verification, reconciliation

**LOC:** ~16,776 src + ~11,438 tests = **~28,214 total** | **Tests:** 26 test files (brute_force_parity, compaction, concurrent_access, conversation, db, hardening, hnsw_hotswap, hnsw_persistence, integration, migration, quantization, storage_lifecycle, tokenizer, ...) | **Maturity:** Production/Mature

---

### semantic-memory-forge v0.1.0
**Role:** Forge verification truth -- evidence bundles, export envelopes, and causal estimation substrate. Primary authority for downstream bridge consumption.

**Deps:** stack-ids, serde, serde_json, chrono, uuid, thiserror, tracing, schemars

**Versioned Modules:** v9, v11, v13, v14

**Key Types:**
| Area | Types |
|------|-------|
| Bundles | `EvidenceBundle`, `CausalQuestion`, `TreatmentSpec`, `OutcomeSpec`, `RefutationAttempt`, `RefutationResult`, `VerificationLifecycleState`, `VerificationSummary`, `VerificationTrialRecord`, `ComparabilitySnapshot`, `PromotionState` |
| Envelopes | `ExportEnvelopeV1/V2/V3` (V3 canonical, V1/V2 deprecated), `ExportAuthority`, `ForgeExportMeta`, `ExportRecord`, `ExportRecordSemanticsV3`, `ExportEpisode`, `ExportEvidenceRef`, `ExportRelation` |
| Semantics | `ProjectionVisibilityClass`, `ConstraintSeedKind` (Hyperedge, MutualExclusion, TemporalCoherence, CausalRefutation, NuisanceDisclosure), `CausalRoleHint` |
| Estimators | `EstimatorMeta`, `EstimatorKind`, `EnvironmentFingerprint`, `SidecarExecution` |
| Tool Receipts | `ForgeToolReceiptV1/V2`, `ForgeToolBudgetContext` |

**LOC:** ~4,919 | **Maturity:** Production/Mature

---

### forge-memory-bridge v0.1.0
**Role:** Transforms Forge export envelopes into projection import batches for semantic-memory consumption. Bridges export vocabulary to import vocabulary.

**Deps:** serde, serde_json, schemars, chrono, uuid, thiserror, semantic-memory-forge, stack-ids

**Three-generation evolution:**
| Version | Status | Purpose |
|---------|--------|---------|
| `transform_envelope()` / `ProjectionImportBatchV1` | Deprecated | Legacy compatibility |
| `transform_envelope_v2()` / `ProjectionImportBatchV2` | Deprecated | Normalized storage shape |
| `transform_envelope_v3()` / `ProjectionImportBatchV3` | **Canonical** | Kernel-ready lane, rich semantics |

**Import Records (V3):** `ImportClaimVersion`, `ImportRelationVersion`, `ImportEpisodeRecord`, `ImportEntityAlias`, `ImportEvidenceRef`

**Enums:** `ClaimState`, `ProjectionFreshness`, `ContradictionStatus`, `MergeDecision`, `ReviewState`

**Design:** Non-inventive -- does not synthesize missing semantics. Preserves v13--v15 experimental surfaces.

**LOC:** ~2,618 | **Maturity:** Stable with clear deprecation/migration path

---

### knowledge-runtime v0.1.0
**Role:** Bounded orchestration scaffold for semantic-memory: classification, routing, scoped entity resolution, provenance-preserving merge, projection status tracking, and inference advisory generation.

**Deps:** constraint-compiler, forge-memory-bridge, kernel-execution, kernel-oracles, recursive-kernel-core, semantic-memory, stack-ids, serde, serde_json, thiserror, tracing, chrono, uuid, schemars

**Modules:** adapters, config, entity, error, ids, inference, obs, projection, query (classify/route/merge), runtime, temporal, views, evidence

**Key Types:**
| Area | Types |
|------|-------|
| Config | `RuntimeConfig`, `QueryConfig`, `EntityConfig`, `ProjectionConfig` |
| Entity | `Entity`, `EntityRegistry`, `EntityKind`, `ResolveResult`, `MatchQuality`, `CodeEntity`, `CodeEntityKind` |
| Query | `ClassifyResult`, `QueryMode`, `RoutePlan`, `RouteLeg`, `RetrievalStrategy`, `MergePolicy`, `MergedResults`, `MergedItem` |
| Projection | `ProjectionTracker`, `ProjectionHealth`, `ProjectionMeta`, `StaleCause`, `RebuildDriver`, `RebuildOutcome` |
| Inference | `InferenceAdvisory`, `InferenceExplanation`, `RiskGateDecision` |
| Observability | `QueryTrace`, `QueryWarning`, `ProjectionTrace`, `ProjectionAction` |
| Runtime Views | `EffectRuntimeViewV1`, `DelegationRuntimeViewV1`, `DeployabilityRuntimeViewV1`, `ContinuityRuntimeViewV1`, `EffectiveConstitutionViewV1`, `CompiledObligationRuntimeViewV1`, `CompositionConflictRuntimeViewV1`, `PolicyImpactDiffRuntimeViewV1` |
| Evidence | `AnswerBundle`, `EvidenceItem`, `EvidenceRelevance`, `SearchEvidenceBundle` |

**Strict enforcements:** KR-001, KR-002 (scope enforcement), KR-003 (rebuild driver interface)

**LOC:** ~5,752 | **Tests:** classification, routing, merging, entity registry, projection identity | **Maturity:** Alpha -- foundation layer complete, temporal handling stable, inference advisory-only

---

## 4. Verification Stack

### verification-control v0.1.0
**Role:** Canonical verification control-plane case, plan, receipt, and ledger artifacts. Central control plane for the verification lifecycle.

**Deps:** chrono, schemars, serde, serde_json, stack-ids, llm-tool-runtime, semantic-memory-forge

**Lifecycle:** Case (open) -> Plan (assigned) -> Attempt (scheduled) -> Receipt (emitted) -> Ledger (closed with terminal disposition)

**Key Types:**
| Area | Types |
|------|-------|
| Core | `VerificationCase`, `CheckPlan`, `VerificationAttempt`, `ControlReceipt`, `LedgerEntry` |
| Classification | `VerificationCaseClass`, `VerificationCaseLifecycleState`, `TerminalDisposition`, `PromotionClass` (P0--P3), `ReversibilityClass`, `CheckMethod`, `ConstitutionalContextStatus` |
| V14 Governance | `EffectReviewCaseV1`, `EffectBlockReceiptV1`, `DelegationReviewCaseV1`, `ReleaseGateCaseV1`, `ContinuityReviewCaseV1` |
| Functions | `interpret_promotion_eligibility()`, `interpret_rollback_requirement()` |

**LOC:** ~2,678 src + ~538 tests | **Maturity:** Production/Mature

---

### verification-policy v0.1.0
**Role:** Canonical verification policy and approval artifacts. Execution permits carry runtime-sealed commit tokens (unforgeable).

**Deps:** schemars, serde, stack-ids, llm-tool-runtime, verification-control

**Modules:** permit, profile_p1_privacy, profile_p2_locality, v14

**Key Types:**
- `PolicyDecision`, `ApprovalRecord`, `PolicySnapshot`
- `EffectPolicyProfileV1`, `DelegationPolicyProfileV1`, `ReleasePolicyProfileV1`, `ContinuityPolicyProfileV1`
- `ExecutionPermit`, `ExecutionPermitScope`, `CommitToken` (unforgeable)
- `EffectPolicyRunModeV1` (Shadow, DryRun, Live)
- Profiles: P1 (privacy), P2 (locality)

**LOC:** ~2,041 src + ~591 tests | **Maturity:** Production/Mature

---

### verification-adjudication v0.1.0
**Role:** Canonical verification disposition, promotion, refutation, and rollback decisions.

**Deps:** schemars, serde, stack-ids, verification-calibration, verification-control, verification-policy

**Key Types:**
- `VerificationDisposition` (EligibleForPromotion, AdvisoryOnly, BlockedByPolicy, PendingApproval, Refuted, DegradedNoPromotion, BudgetExhausted, Failed)
- `RollbackScopeV1` (None, LocalDerivedOnly, ProjectionScope, AuthorityEscalationRequired)
- `RefutationClassV1` (None, CausalRefuted)
- `EffectAdjudicationReceiptV1`, `ReleaseRollbackDecisionV1`, `PromotionDecision`, `RefutationDecision`, `RollbackPlan`
- All structs carry `V25CitationContext` for constitutional provenance
- V14: `RollbackDecisionV1`, `RolloutDecisionV1`

**LOC:** ~706 src + ~664 tests | **Maturity:** Production/Active

---

### verification-calibration v0.1.0
**Role:** Canonical verification calibration and abstention artifacts. Minimal but crucial.

**Deps:** schemars, serde, stack-ids, verification-control

**Key Types:**
- `CalibrationSnapshot` -- `evaluate()` forces `advisory_only` when conditions unmet (comparability, oracle calibration, abstention threshold, drift)
- `NuisanceStateArtifact` -- tracks semantic drift and confounded changes

**LOC:** ~120 src + ~232 tests | **Maturity:** Stable/Focused

---

## 5. Governance Surface Crates

### effect-runtime v0.1.0
**Role:** Governed effect pipeline from intent declaration through commit authorization with full lifecycle artifacts.

**Deps:** chrono, serde, schemars, thiserror, stack-ids

**Lifecycle:**
```
EffectIntentV1 -> EffectPreflightReportV1 -> EffectCommitDecisionV1 -> EffectExecutionReceiptV1 -> EffectObservationBundleV1
                                                                                                    \-> CompensationPlanV1 -> CompensationExecutionReceiptV1
```

**Enums:** `EffectPreflightDispositionV1`, `EffectCommitDispositionV1`, `CommitAtomicityV1`, `RetryOwnerV1`, `EffectClassV1`, `BlastRadiusCeilingV1`, `ReversibilityClassV1`, `RunModeV1`, `CompensationClassV1`, ...

All artifact structs have builder patterns. Includes `V25ObligationRefs` for obligation categorization.

**LOC:** ~1,733 | **Maturity:** Early -- builder patterns suggest production-oriented design

---

### authority-delegation v0.1.0
**Role:** Typed delegated-authority surface with bounded lease and policy artifacts. Not an access-control runtime.

**Deps:** serde, schemars, thiserror

**Key Types:**
| Area | Types |
|------|-------|
| Capability | `CapabilityClassV1`, `AuthorityLeaseV1`, `DelegationBundleV1`, `AuthorityChainV1` |
| Governance | `SeparationOfDutiesPolicyV1`, `DualControlApprovalV1`, `ConflictDisclosureV1` |
| Emergency | `BreakGlassGrantV1`, `DelegationRevocationV1`, `ActingOnBehalfReceiptV1` |
| Profiles | `RoleCatalogV1`, `DelegationMatrixV1`, `ApprovalMatrixV1`, `ConflictClassCatalogV1` |
| Enums | `CapabilityBlastRadiusCeilingV1`, `AuthorityChainValidityStateV1`, `CapabilityDisclosureCeilingV1` |

**LOC:** ~676 | **Tests:** 5 files (fixture conformance, serde roundtrip, validation) | **Maturity:** Early

---

### assurance-runtime v0.1.0
**Role:** Typed deployability and certification surface with bounded readiness profiles.

**Deps:** serde, schemars, thiserror

**Key Types:**
- `AssuranceCaseV1`, `ControlMappingV1`, `ResidualRiskAcceptanceV1`, `ReleaseReadinessDecisionV1`
- `FieldMonitoringPlanV1`, `CertificationBundleV1`, `RecertificationTriggerV1`
- `DeploymentProfileV1`, `OperatingEnvelopeV1`, `HazardRegisterV1`
- `RequirementControlMapV1`, `EvidenceCollectionPlanV1`, `MonitorCatalogV1`, `MitigationPlaybookV1`
- All structs implement `new()` constructors and `validate()` methods

**Modules:** assurance, certification, error, profile, profile_p4_regulated, profile_p5_hazard

**LOC:** ~728 | **Tests:** 8 files (fixtures, roundtrips, property-based) | **Maturity:** Early

---

### attestation-exchange v0.1.0
**Role:** Typed attestation exchange contracts for envelope, trust-root, and transparency artifacts.

**Deps:** schemars, serde, thiserror, stack-ids

**Key Types:**
- `AttestationEnvelopeV1`, `TrustRootSetV1`, `TransparencyReceiptV1`
- `VendorCertificationAdapterV1`, `VendorEvidenceTranslationV1`, `VendorTrustRootBindingV1`, `VendorRevocationHandlingV1`
- All structs use stack_ids types for type-safe identifiers

**LOC:** ~626 | **Tests:** 3 files (vendor profile, roundtrip, validation) | **Maturity:** Early

---

### constitutional-memory v0.1.0
**Role:** Typed charter and archive surface with bounded amendment checks. Not a hidden governance runtime.

**Deps:** schemars, serde, stack-ids

**Key Types:**
- `CharterBundleV1`, `DoctrineSnapshotV1`, `AmendmentProposalV1`, `AmendmentDecisionV1`
- `ArchiveManifestV1`, `CompactionReceiptV1`, `HistoricalQueryGuaranteeV1`
- `DeprecationBundleV1`, `RetirementBundleV1`
- `AmendmentDisposition` (ApprovedWithRollback, BlockedMissingRollback, BlockedMissingSemanticDiff, BlockedMissingArchiveGuarantee, AdvisoryOnly)

**Pure Logic Functions:** `evaluate_amendment()`, `evaluate_archive_compaction()`

**LOC:** ~295 | **Tests:** 3 files (amendment eval, archive compaction, property contracts) | **Maturity:** Early -- lightweight but powerful domain logic

---

### continuity-runtime v0.1.0
**Role:** Typed continuity and incident surface with bounded recovery profiles. Not an incident-command runtime.

**Deps:** serde, schemars, thiserror

**Key Types:**
| Area | Types |
|------|-------|
| Incident | `IncidentCaseV1`, `ContainmentDecisionV1`, `ForensicFreezeV1`, `ContinuityExceptionV1` |
| Recovery | `RecoveryPlanV1`, `RecoveryReplaySliceV1`, `PostmortemBundleV1`, `ResilienceExerciseV1` |
| SLO | `ServiceLevelProfileV1`, `ErrorBudgetLedgerV1` |
| Vocabs | `IncidentSeverityV1`, `IncidentStatusV1`, `RecoveryReplayStateV1`, `ResilienceExerciseKindV1`, `ContainmentCategoryV1`, `ForensicPreservationKindV1`, `PostmortemKindV1` |

**LOC:** ~930 | **Tests:** 5 files (fixture conformance, SLO, validation) | **Maturity:** Early

---

### mechanism-runtime v0.1.0
**Role:** Typed mechanism/theory surface with bounded fit and refuter evaluators.

**Deps:** schemars, serde, stack-ids, thiserror

**Key Types:**
- `MechanismBundleV1`, `TheoryVersionV1`, `TheoryLibraryV1`, `HypothesisLibraryV1`
- `SimulationContractV1`, `FitRunV1`, `TheoryRefuterSuiteV1`, `RolloutStabilityReportV1`
- `FitDisposition` (AdvisoryFitOnly, PromotionBlockedMissingRefuter, PromotionBlockedFailingRefuter, PromotionBlockedStabilityRisk, EligibleForLocalReview)

**Pure Logic:** `evaluate_fit_run()` -- deterministic disposition from refuter suite + stability report

**LOC:** ~254 | **Maturity:** Stable/MVP

---

### federated-settlement v0.1.0
**Role:** Typed treaty and settlement surface with bounded shared-view evaluators. Not a network runtime.

**Deps:** schemars, serde, serde_json, stack-ids

**Key Types:**
- `TreatyBundleV1`, `TreatyPartyV1`, `RuntimeIdentitySetV1`, `CrossRuntimeEquivalenceBundleV1`
- `SharedDispositionV1`, `LocalDissentRecordV1`, `SharedViewDowngradeV1`
- `SettlementCaseV1`, `SettlementReceiptV1`, `SharedReplaySliceV1`, `SharedDivergenceReportV1`, `TreatySuspensionV1`
- `SettlementDisposition`, `DowngradeReason`

**Pure Logic Functions:** `evaluate_settlement()`, `evaluate_shared_replay()`, `evaluate_divergence_or_suspension()`

**LOC:** ~397 | **Maturity:** Early -- purely data-driven, no IO

---

### discovery-portfolio v0.1.0
**Role:** Typed discovery portfolio surface with bounded budget and selection evaluators. Not a scheduler runtime.

**Deps:** schemars, serde, stack-ids

**Key Types:**
- `DiscoveryProgramV1`, `ProgramHypothesisSetV1`, `ExperimentCampaignV1`, `PortfolioPlanV1`
- `VerificationLoadBudgetV1`, `InformationValueEstimateV1`, `CampaignDecisionTraceV1`
- `CampaignDecision` (Launch, Defer, PauseBudgetExhausted)

**Pure Logic:** `evaluate_portfolio_plan()` -- greedy knapsack-style allocation with information-value ratio prioritization

**LOC:** ~266 | **Tests:** 4 files | **Maturity:** Early

---

### remote-oracle-admission v0.1.0
**Role:** Typed remote oracle admission contracts for lease, result, replay, and re-admission artifacts.

**Deps:** schemars, serde, stack-ids, attestation-exchange

**Key Types:**
- `RemoteOracleLeaseV1`, `RemoteSliceRequestV1`, `RemoteSliceResultV1`, `CrossRuntimeReplayTicketV1`
- `AttestationRevocationV1`, `AttestationSupersessionV1`
- `RemoteExactnessClassV1`, `RemoteDisclosureClassV1`, `RemoteReplayObligationV1`
- `LocalAdmissionRecommendationV1`, `ReplayFailureBehaviorV1`

All structs implement `.new()` and `.validate()`. Re-exports `V25ConstitutionCitation`.

**LOC:** ~515 src + ~191 tests | **Maturity:** Stable/Production

---

### spec-execution v0.1.0
**Role:** Typed spec and proof surface with bounded generated-artifact evaluators. Not a compiler runtime.

**Deps:** schemars, serde, serde_json, stack-ids, tracing

**Key Types:**
- `SpecBundleV1`, `NormativeAstNodeV1`, `NormativeASTV1`
- `GeneratedSchemaFileV1`, `GeneratedSchemaBundleV1`, `GeneratedInterpreterBundleV1`, `GeneratedConformanceCorpusV1`, `GeneratedMigrationPlanV1`
- `ProofObligationV1`, `ProofObligationSetV1`, `ProofEvaluationReceiptV1`
- `HumanVetoBundleV1`, `MetaChallengeBundleV1`, `SelfHostingBuildReceiptV1`
- `GeneratedAdmissionState` (AdvisoryOnly, NonAdmitted, HumanVetoed)

**LOC:** ~382 src + ~342 tests | **Maturity:** Early/Stable

---

### profile-runtime v0.1.0
**Role:** Canonical effective constitution and profile composition runtime. Composes profile overlays into replayable effective constitutions with typed conflicts and explicit exceptions.

**Deps:** assurance-runtime, attestation-exchange, authority-delegation, continuity-runtime, verification-policy, stack-ids, serde, serde_json, schemars, thiserror, tracing

**Key Types:**
- `CompositionReceiptV1` -- full composition audit trail
- `EffectiveConstitutionV1` -- mode classification, law/doctrine refs, admitted profiles/exceptions, obligation summaries
- `CompiledObligationSetV1` -- obligation entries, blocks, required approvals/checks/monitors/evidence
- `CompositionConflictSetV1` -- conflicts with exception class support and review paths
- `PolicyImpactDiffV1` -- from/to effective constitution comparison
- `ConstitutionModeV1` (Normal, Incident, AdvisoryOnly, Blocked)

**Modules:** adapters, applicability, compose, constitution, exception, profile_set, rules

**LOC:** ~3,621 | **Maturity:** Beta -- composition and obligation compilation complete, conflict detection integrated

---

## 6. Orchestration & Tooling

### forge-pilot v0.1.0
**Role:** Closed-loop OODA orchestrator over runtime advisories, kernel oracles, and canonical Forge export/import lanes. **The core orchestration engine for the RecursiveIntell stack.**

**Feature:** `governance` (default) -- enables optional governance stack

**Deps (20+ workspace crates):**
- Always: constraint-compiler, forge-engine, forge-memory-bridge, kernel-execution, kernel-oracles, knowledge-runtime, recursive-kernel-core, semantic-memory, semantic-memory-forge, verification-{adjudication,calibration,control,policy}, stack-ids
- Governance-gated: assurance-runtime, attestation-exchange, authority-delegation, constitutional-memory, continuity-runtime, effect-runtime, mechanism-runtime
- External: schemars, serde, serde_json, thiserror, tokio, tracing, blake3, chrono, uuid, rusqlite, reqwest

**OODA Modules:**
| Phase | Module | Key Types |
|-------|--------|-----------|
| Observe | `observe` | Scope health, observation paths, disposition classification |
| Orient | `orient` | Target extraction and scoring |
| Decide | `decide` | `select_candidate()`, `Decision` |
| Act | `act` | `execute_plan()`, `ActionFamily`, `ActionOutcome`, `AdvisoryPlan`, `OracleExecution`, `PatchExecution` |

**Additional Modules:** bootstrap (manifest delta, richness tracking, symbol records), bundle_builder, cli, config, error, export, governance_gate (feature-gated), history, loop_runner, repo_chat (question routing, retrieval, provider grounding), targets, types

**Core Types:** `LoopRunner`, `LoopReport`, `LoopIterationReport`, `LoopReceipt`, `LoopBudgetReceipt`, `ExternalHaltFlag`, `HaltReason`

**LOC:** ~11,056 | **Maturity:** Active -- largest crate, production CLI/TUI, comprehensive loop runner with budget tracking

---

### forge-engine (living-memory) v0.2.0
**Role:** Causal edit attribution and structured patch evaluation engine. Operational verification and evaluation for the local-first AI stack.

**Path:** `living-memory/living-memory`

**Deps (13 workspace + 14 external):**
- Workspace: llm-tool-runtime, semantic-memory, semantic-memory-forge, forge-memory-bridge, stack-ids, forge-policy, sandbox-workspace, typed-patch, effect-signature, check-runner, mindstate-core, stabilizer-core, cea-core, cea-store, cea-sqlite
- External: tokio, serde, serde_json, thiserror, anyhow, uuid, blake3, rusqlite, tempfile, walkdir, once_cell, petgraph, regex, async-trait, tracing, chrono, glob, similar, rand

**Feature:** `danger-sm-write` (compatibility escape hatch for semantic-memory write)

**Key Modules:**
| Module | Purpose |
|--------|---------|
| `adapters` | Cargo/project adapters |
| `baseline` | Baseline capture, provenance, comparability policy |
| `cea` | Causal graph construction, prediction, attribution, coverage analysis |
| `config` | ForgeConfig with limits, caps, mindstate, novelty, stabilization, container, lab, CEA, danger flags |
| `exec` | Check execution (host + container backends), workspace preparation |
| `experiment` | Paired experiment runner, trial records, evidence records, statistics |
| `export` | Bundle export with optional write-through to semantic-memory |
| `lab` | Eval suites, verification planning, hypothesis generation/assessment, causal fingerprinting |
| `runtime` | MindState compilation, Stabilizer, patch validate/apply/render |
| `scoring` | Comparability, objective policies, planned checks |
| `store` | ForgeStore persistence |
| `tool_receipts` | Tool receipt ingestion |

**LOC:** ~9,137 | **Maturity:** Beta -- core execution stable, CEA/hypothesis evaluation solidifying

---

### llm-tool-runtime v0.1.0
**Role:** Provider-agnostic tool contracts, registry, dispatch, and receipt plumbing.

**Deps:** semantic-memory-forge, stack-ids, async-trait, chrono, serde, serde_json, thiserror, tokio, tracing, uuid

**Key Types:**
| Area | Types |
|------|-------|
| Backend | `ToolBackendKind` (LocalFunction, OpenAiFunction, OpenAiBuiltIn, OllamaFunction, RemoteMcp) |
| Classification | `ToolSideEffectClass` (ReadOnly...Admin), `ToolIdempotencyClass`, `ToolExposureMode`, `ToolPlannerStage` |
| Approval | `ToolApprovalKind`, `ToolApprovalState`, `ApprovalGrant`, `ApprovalGrantScope`, `ApprovalGrantEffectClass` |
| Core | `ToolDescriptor`, `ToolCall`, `ToolCtx`, `ToolResult`, `ToolError`, `ToolReceipt`, `ToolExecutionPermit` |
| Registry | `ToolRegistry` (trait), `Tool` (trait: descriptor + async invoke) |
| Functions | `render_openai_tool()`, `render_ollama_tool()`, `validate_arguments_against_schema()`, `plan_exposure()` |

**LOC:** ~3,161 | **Maturity:** Alpha -- contracts stable, provider rendering working, approval model partially integrated

---

### contract-schema-gen v0.1.0
**Role:** Schema generation utility -- derives JSON schemas for all workspace crate types. Binary + library, not published.

**Deps:** All workspace crates + schemars, serde_json, tempfile

**CLI Interface:**
- No args: generate to default `schemas/` directory
- `--check <dir>`: validate existing schemas
- `<path>`: generate to specified path

Generates **150+ schemas** across the entire workspace.

**LOC:** ~1,127 | **Maturity:** Mature utility

---

## 7. Primitives Workspace (excluded from default build)

**Path:** `Primitives/`
**Resolver:** 2 | **Edition:** 2021 | **MSRV:** 1.75 | **License:** MIT | All `publish = false`

These 10 crates form the **low-level execution and attribution substrate** consumed by forge-engine.

### cea-core v0.1.0
**Role:** Causal edit attribution for structured code patches. Converts patches + check results into weighted cause/effect attribution triples.

**Properties:** Deterministic (blake3), multi-cause proportional (not nearest-neighbor collapse), Beta-distribution confidence, compact graph persistence.

**Key API:** `attribute_effects()`, `predict()`, `compute_run_hash()`, `CausalGraph`, `CausalNode`, `CausalEdge`, `EditOpSignature`, `RiskFlag`, `CausalPrediction`, `ConfidenceEnvelope`, `AttributedRunResult`

**LOC:** ~2,200 | **Maturity:** Highly mature

### cea-store v0.1.0
**Role:** Storage contract and row types for causal edit attribution graphs. Tier 3 persistence boundary.

**Traits:** `CeaStoreWriteTx` (write ops), `CeaStore` (reader interface with_write_tx, load)

**Functions:** `update_graph()` (idempotent, run-hash dedup), `load_graph()` (reconstruct from storage)

**LOC:** ~657 | **Maturity:** Mature

### cea-sqlite v0.1.0
**Role:** SQLite implementation of cea-store. WAL mode, foreign keys, auto-migration (v1->v2), 5s busy timeout.

**Schema tables:** `cea_nodes`, `cea_edges`, `cea_run_log`

**LOC:** ~500 | **Maturity:** Production-ready

### typed-patch v0.1.0
**Role:** Structured patch schema plus validation and apply helpers. Transactional application with rollback.

**Key API:** `StructuredPatch`, `FileEdit`, `EditOp` (Insert/Delete/Replace), `Anchor` (AfterLine/BeforeLine/AfterMatch/BeforeMatch), `LineRange`, `LineAttributionMap`, `validate_patch()`, `apply_patch()`, `render_diff()`

**LOC:** ~951 | **Maturity:** Production-grade (property-based tested)

### check-runner v0.1.0
**Role:** Execution backend and check normalization primitives. Host + container (docker/podman/nerdctl) execution.

**Trait:** `ExecutionBackend` (async: prepare_workspace, run_command, collect_logs)

**Key API:** `CheckResult`, `CheckCommand`, `HostBackend`, `ContainerBackend`, `select_backend()`, `detect_runtime()`

**Feature:** `container` (optional Docker/Podman/Nerdctl support)

**LOC:** ~847 | **Maturity:** Production-grade (signal handling, timeouts, process group cleanup)

### sandbox-workspace v0.1.0
**Role:** Workspace sandboxing and patch filesystem primitives. Symlink-safe, escape-proof.

**Trait:** `PatchFs` (read_lines, write_lines, exists, remove_file, create_parent_dirs, snapshot_lines)

**Key API:** `Workspace`, `PatchedWorkspace`, `LocalPatchFs`, `prepare_workspace()`, `copy_dir_recursive_pub()`

**LOC:** ~360 | **Maturity:** Production-grade (proptest-verified)

### forge-policy v0.1.0
**Role:** Workspace and database safety policy primitives. Path safety, patch-cap checks, allowlisting, DB identity validation.

**Key API:** `verify_sqlite_db_identity()`, `ensure_relative_path()`, `reject_symlinks()`, `resolve_workspace_path()`, `validate_forbidden_paths()`, `validate_patch_caps()`, `is_env_allowed()`

**LOC:** ~423 | **Maturity:** Production-ready (property-tested)

### effect-signature v0.1.0
**Role:** Stable check effect signature types and hashing helpers. Deterministic blake3 effect hashing.

**Key API:** `EffectSignature`, `LocatedEffect`, `effect_signature_hash()`

**LOC:** ~132 | **Maturity:** Complete/Minimal

### mindstate-core v0.1.0
**Role:** Serializable mindstate payload types. Deterministic rendering and hashing.

**Key API:** `MindState` (render, hash), `EvidenceItem`, `TraceRef`, `OrderedFloat`, `compute_question_sig()`, `budget_evidence()`

**LOC:** ~285 | **Maturity:** Stable

### stabilizer-core v0.1.0
**Role:** Bounded attempt-phase and delta policy primitives. Models retry/stabilization phases.

**Key API:** `AttemptPhase` (Innovative, Stabilize1, Stabilize2, Clamp), `DeltaPolicy`, `Stabilizer`, `StabilizerConfig`, `extract_strategy_tags()`, `compute_tag_novelty()`, `determine_approach_family()`

**LOC:** ~487 | **Maturity:** Mature

---

## 8. Dependency Graph

```
                          stack-ids (foundation -- no deps)
                              |
          +-------------------+-------------------+-------------------+
          |                   |                   |                   |
  recursive-kernel-core   semantic-memory-forge   attestation-exchange  (+ most other crates)
          |                   |                   |
          |           forge-memory-bridge    remote-oracle-admission
          |                   |
  constraint-compiler --------+
          |
  kernel-execution
          |
  kernel-oracles
          |
          +----------- knowledge-runtime ---------- semantic-memory
          |                                              |
          +--- verification-control <--- verification-policy
          |           |                       |
          |   verification-calibration   verification-adjudication
          |
  profile-runtime <--- assurance-runtime, attestation-exchange,
          |            authority-delegation, continuity-runtime
          |
  forge-pilot (hub -- 20+ deps, OODA orchestrator)
          |
  forge-engine (living-memory -- 13 workspace + Primitives)
          |
  Primitives: cea-core -> cea-store -> cea-sqlite
              typed-patch -> sandbox-workspace -> forge-policy
              check-runner -> effect-signature
              mindstate-core, stabilizer-core
```

---

## 9. Summary Matrix

| Crate | Version | LOC (src) | Tests | Deps (workspace) | Maturity | Role |
|-------|---------|-----------|-------|-------------------|----------|------|
| **stack-ids** | 0.1.0 | 3,383 | inline | 0 | Production | Identity primitives |
| **recursive-kernel-core** | 0.1.0 | 256 | 327 | 1 | Stable | Kernel schemas |
| **constraint-compiler** | 0.1.0 | 1,378 | 17 embedded | 4 | Core | Graph compilation |
| **kernel-execution** | 0.1.0 | 1,216 | inline | 3 | Stable | Execution modes |
| **kernel-oracles** | 0.1.0 | 1,006 | inline | 5 | Stable | Oracle assessment |
| **kernel-conformance** | 0.1.0 | 1,983 | 100+ | 18 | Stable | Conformance harness |
| **semantic-memory** | 0.5.0 | 16,776 | 11,438 | 2 | Production | Projected truth store |
| **semantic-memory-forge** | 0.1.0 | 4,919 | inline | 1 | Production | Raw verification truth |
| **forge-memory-bridge** | 0.1.0 | 2,618 | inline | 2 | Stable | Export->Import bridge |
| **knowledge-runtime** | 0.1.0 | 5,752 | inline | 7 | Alpha | Query orchestration |
| **verification-control** | 0.1.0 | 2,678 | 538 | 3 | Production | Control plane |
| **verification-policy** | 0.1.0 | 2,041 | 591 | 3 | Production | Policy & permits |
| **verification-adjudication** | 0.1.0 | 706 | 664 | 4 | Production | Dispositions |
| **verification-calibration** | 0.1.0 | 120 | 232 | 2 | Stable | Calibration |
| **effect-runtime** | 0.1.0 | 1,733 | inline | 1 | Early | Effect lifecycle |
| **authority-delegation** | 0.1.0 | 676 | 5 files | 0 | Early | Delegation |
| **assurance-runtime** | 0.1.0 | 728 | 8 files | 0 | Early | Deployability |
| **attestation-exchange** | 0.1.0 | 626 | 3 files | 1 | Early | Attestation |
| **constitutional-memory** | 0.1.0 | 295 | 3 files | 1 | Early | Charter & amendment |
| **continuity-runtime** | 0.1.0 | 930 | 5 files | 0 | Early | Incident & SLO |
| **mechanism-runtime** | 0.1.0 | 254 | 0 | 1 | MVP | Theory & fit |
| **federated-settlement** | 0.1.0 | 397 | inline | 1 | Early | Settlement |
| **discovery-portfolio** | 0.1.0 | 266 | 4 files | 1 | Early | Portfolio eval |
| **remote-oracle-admission** | 0.1.0 | 515 | 191 | 2 | Stable | Remote oracle |
| **spec-execution** | 0.1.0 | 382 | 342 | 1 | Stable | Spec & proof |
| **profile-runtime** | 0.1.0 | 3,621 | proptest | 6 | Beta | Constitution compose |
| **forge-pilot** | 0.1.0 | 11,056 | mixed | 20+ | Active | OODA orchestrator |
| **forge-engine** | 0.2.0 | 9,137 | mixed | 13 | Beta | Patch eval engine |
| **llm-tool-runtime** | 0.1.0 | 3,161 | inline | 2 | Alpha | Tool contracts |
| **contract-schema-gen** | 0.1.0 | 1,127 | 1 file | all | Mature | Schema generator |
| **Primitives (10 crates)** | 0.1.0 | ~6,500 | extensive | internal | Production | CEA + patch substrate |

**Grand total:** ~125,725 lines of Rust across 40 crates | 211 JSON schemas | 50+ gate/audit scripts
