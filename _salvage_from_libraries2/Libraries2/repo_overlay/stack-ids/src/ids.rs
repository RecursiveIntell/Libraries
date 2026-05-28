//! Opaque ID newtypes for cross-crate identity.
//!
//! Each newtype wraps a `String` and is intentionally opaque — callers
//! should not parse the inner value. IDs are assigned by their respective
//! authority crates; `stack-ids` only provides the type contract.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// Macro to generate an opaque string-wrapper ID type with standard impls.
macro_rules! define_id {
    (
        $(#[$meta:meta])*
        $name:ident
    ) => {
        $(#[$meta])*
        #[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, JsonSchema)]
        #[serde(transparent)]
        pub struct $name(pub String);

        impl $name {
            /// Create from any string-like value.
            pub fn new(value: impl Into<String>) -> Self {
                Self(value.into())
            }

            /// Generate a new random UUID v4 ID.
            pub fn generate() -> Self {
                Self(uuid::Uuid::new_v4().to_string())
            }

            /// Borrow as a string slice.
            pub fn as_str(&self) -> &str {
                &self.0
            }

            /// Returns true if the inner string is empty.
            pub fn is_empty(&self) -> bool {
                self.0.is_empty()
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str(&self.0)
            }
        }

        impl From<String> for $name {
            fn from(value: String) -> Self {
                Self(value)
            }
        }

        impl From<&str> for $name {
            fn from(value: &str) -> Self {
                Self(value.to_string())
            }
        }

        impl AsRef<str> for $name {
            fn as_ref(&self) -> &str {
                &self.0
            }
        }

        impl FromStr for $name {
            type Err = std::convert::Infallible;

            fn from_str(value: &str) -> Result<Self, Self::Err> {
                Ok(Self::new(value))
            }
        }
    };
}

define_id!(
    /// Opaque identifier for an import/export envelope.
    ///
    /// Assigned by the exporting authority (e.g. Forge). Stable across
    /// re-exports of the same logical unit.
    EnvelopeId
);

define_id!(
    /// Opaque identifier for a claim (knowledge assertion).
    ///
    /// Assigned by the projection importer. Stable within a claim lineage.
    ClaimId
);

define_id!(
    /// Opaque identifier for a specific version of a claim.
    ///
    /// Each mutation to a claim's validity, status, or content produces
    /// a new version with a new `ClaimVersionId`. The `ClaimId` remains
    /// stable across versions.
    ClaimVersionId
);

define_id!(
    /// Opaque identifier for an entity (person, concept, code unit, etc.).
    ///
    /// The inner string is intentionally unstructured. Use domain-specific
    /// constructors to create these (e.g. code_entity_id).
    EntityId
);

define_id!(
    /// Opaque identifier for an episode (causal record).
    ///
    /// Assigned by the episode creator. Stable within the episode's lifecycle.
    EpisodeId
);

define_id!(
    /// Opaque identifier for a logical retry family within one retry-owner boundary.
    ///
    /// One `AttemptId` exists per logical retry family. Retries inside that
    /// boundary produce new `TrialId`s, NOT new `AttemptId`s. A new `AttemptId`
    /// is created only when the retry owner changes (e.g. node-level retry
    /// after transport retries are exhausted) or on explicit replay/re-enqueue.
    AttemptId
);

define_id!(
    /// Opaque identifier for a concrete execution within a logical retry family.
    ///
    /// Every retry within one owner boundary creates a new `TrialId` under
    /// the same `AttemptId`. Each `TrialId` is globally unique (UUID v4).
    TrialId
);

define_id!(
    /// Opaque identifier for a stored artifact (patch, snapshot, file).
    ///
    /// Assigned by the artifact store. Stable across reads.
    ArtifactId
);

define_id!(
    /// Opaque identifier for a projection instance.
    ///
    /// Identifies a specific derived view (entity registry entry, temporal
    /// claim set, etc.) within a scope.
    ProjectionId
);

define_id!(
    /// Opaque identifier for a relation (edge between entities).
    ///
    /// Assigned by the projection importer. Stable within a relation lineage.
    /// Each mutation produces a new `RelationVersionId` under the same `RelationId`.
    RelationId
);

define_id!(
    /// Opaque identifier for a relation version.
    ///
    /// Each mutation to a relation produces a new version with a new
    /// `RelationVersionId`. The `RelationId` remains stable across versions.
    RelationVersionId
);

define_id!(
    /// Opaque identifier for an import batch produced by the bridge.
    ///
    /// Assigned by the bridge transformation pipeline. Unique per batch.
    ImportBatchId
);

define_id!(
    /// Opaque identifier for a claim family spanning multiple claim versions.
    ///
    /// Assigned by the authoritative exporter when version lineage is known.
    ClaimFamilyId
);

define_id!(
    /// Opaque identifier for a higher-order assertion group.
    ///
    /// Used to preserve joint assertion meaning across export, bridge, and
    /// deterministic compilation. The bridge preserves it; it does not invent it.
    AssertionGroupId
);

define_id!(
    /// Opaque identifier for a relation group exported by the authority.
    ///
    /// Relation groups allow the compiler to recover hyperedge structure without
    /// flattening upstream semantics into pairwise folklore.
    RelationGroupId
);

define_id!(
    /// Opaque identifier for a joint evidence group.
    ///
    /// Used to tie multiple observations or assertions to a shared evidence basis.
    JointEvidenceGroupId
);

define_id!(
    /// Opaque identifier for a contradiction candidate group.
    ///
    /// Assigned upstream when the exporting authority exposes contradiction seed
    /// structure. Kernel/runtime layers must not synthesize it from absence.
    ContradictionGroupId
);

define_id!(
    /// Opaque identifier for a mutual exclusion or constraint seed group.
    ConstraintGroupId
);

define_id!(
    /// Opaque identifier for a recursive kernel run.
    ///
    /// Assigned by the kernel execution layer for one bounded, rebuildable run.
    KernelRunId
);

define_id!(
    /// Opaque identifier for a compiled constraint unit.
    ConstraintId
);

define_id!(
    /// Opaque identifier for a compiled hyperedge recovered from upstream grouping semantics.
    HyperedgeId
);

define_id!(
    /// Opaque identifier for a residual artifact emitted by bounded execution.
    ResidualId
);

define_id!(
    /// Opaque identifier for a stable syndrome signature emitted by the kernel.
    SyndromeId
);

define_id!(
    /// Opaque identifier for a witness artifact produced by execution or oracles.
    WitnessId
);

define_id!(
    /// Opaque identifier for a certificate artifact produced by execution or oracles.
    CertificateId
);

define_id!(
    /// Opaque identifier for a bounded oracle slice definition.
    OracleSliceId
);

define_id!(
    /// Opaque identifier for a bounded execution region.
    RegionId
);

define_id!(
    /// Opaque identifier for a stable digest of one bounded execution region.
    RegionDigestId
);

define_id!(
    /// Opaque identifier for a typed artifact transport crossing region boundaries.
    ArtifactTransportId
);

define_id!(
    /// Opaque identifier for syndrome-first repair routing.
    RepairRouteId
);

define_id!(
    /// Opaque identifier for one ranked local repair candidate.
    RepairCandidateId
);

define_id!(
    /// Opaque identifier for a nuisance-state artifact.
    NuisanceStateId
);

define_id!(
    /// Opaque identifier for a convergence-governance artifact.
    ConvergenceReportId
);

define_id!(
    /// Opaque identifier for a registered recursive operator contract.
    OperatorId
);

define_id!(
    /// Opaque identifier for a specific version of a registered operator contract.
    ///
    /// `OperatorId` stays stable across versions; `OperatorVersionId` changes when
    /// the executable contract or policy-relevant operator surface changes.
    OperatorVersionId
);

define_id!(
    /// Opaque identifier for a structured refutation result artifact.
    RefutationResultId
);

define_id!(
    /// Opaque identifier for a structured calibration report artifact.
    CalibrationReportId
);

define_id!(
    /// Opaque identifier for a verification-control case.
    VerificationCaseId
);

define_id!(
    /// Opaque identifier for a canonical verification check plan.
    CheckPlanId
);

define_id!(
    /// Opaque identifier for a canonical control-plane receipt.
    ControlReceiptId
);

define_id!(
    /// Opaque identifier for an append-only verification ledger entry.
    LedgerEntryId
);

define_id!(
    /// Opaque identifier for a policy decision artifact.
    PolicyDecisionId
);

define_id!(
    /// Opaque identifier for an approval record artifact.
    ApprovalRecordId
);

define_id!(
    /// Opaque identifier for a promotion decision artifact.
    PromotionDecisionId
);

define_id!(
    /// Opaque identifier for a refutation decision artifact.
    RefutationDecisionId
);

define_id!(
    /// Opaque identifier for a rollback plan artifact.
    RollbackPlanId
);

define_id!(
    /// Opaque identifier for a calibration snapshot artifact.
    CalibrationSnapshotId
);

define_id!(
    /// Opaque identifier for a versioned learning update artifact.
    LearningUpdateId
);

define_id!(
    /// Opaque identifier for a deterministic boundary repair record artifact.
    BoundaryRepairRecordId
);

define_id!(
    /// Opaque identifier for a semantics profile artifact.
    SemanticsProfileId
);

define_id!(
    /// Opaque identifier for a claim-state artifact.
    ClaimStateId
);

define_id!(
    /// Opaque identifier for a semantic diff artifact.
    SemanticDiffId
);

define_id!(
    /// Opaque identifier for a causal-attribution bundle artifact.
    CausalAttributionBundleId
);

define_id!(
    /// Opaque identifier for an explicit degradation record artifact.
    DegradationRecordId
);

define_id!(
    /// Opaque identifier for an exactness-budget artifact.
    ExactnessBudgetId
);

define_id!(
    /// Opaque identifier for one exported or imported support-set artifact.
    ///
    /// Stable for one snapshot of the support expression attached to a claim.
    SupportSetId
);

define_id!(
    /// Opaque identifier for one contradiction-witness artifact.
    ///
    /// Used to point at a stored explanation of why a claim is simultaneously
    /// supported and refuted within one semantics profile.
    ContradictionWitnessId
);

define_id!(
    /// Opaque identifier for one retraction / supersession artifact.
    ///
    /// This identifies the transaction-time event that closes currentness for
    /// a claim version without erasing historical visibility.
    RetractionRecordId
);

define_id!(
    /// Opaque identifier for an intervention bundle artifact.
    InterventionId
);

define_id!(
    /// Opaque identifier for an outcome-schema artifact.
    OutcomeSchemaId
);

define_id!(
    /// Opaque identifier for an experiment case artifact.
    ExperimentCaseId
);

define_id!(
    /// Opaque identifier for a cohort-contract artifact.
    CohortContractId
);

define_id!(
    /// Opaque identifier for a comparability-matrix artifact.
    ComparabilityMatrixId
);

define_id!(
    /// Opaque identifier for a counterfactual-slice artifact.
    CounterfactualSliceId
);

define_id!(
    /// Opaque identifier for a decision-trace artifact.
    DecisionTraceId
);

define_id!(
    /// Opaque identifier for a refuter-suite artifact.
    RefuterSuiteId
);

define_id!(
    /// Opaque identifier for a refuter-result artifact.
    RefuterResultId
);

define_id!(
    /// Opaque identifier for a rollout-decision artifact.
    RolloutDecisionId
);

define_id!(
    /// Opaque identifier for a rollback-decision artifact.
    RollbackDecisionId
);

define_id!(
    /// Opaque identifier for an experiment-budget artifact.
    ExperimentBudgetId
);

define_id!(
    /// Opaque identifier for an attestation-envelope artifact.
    AttestationEnvelopeId
);

define_id!(
    /// Opaque identifier for a trust-root-set artifact.
    TrustRootSetId
);

define_id!(
    /// Opaque identifier for an artifact-admission-policy artifact.
    ArtifactAdmissionPolicyId
);

define_id!(
    /// Opaque identifier for a transparency-receipt artifact.
    TransparencyReceiptId
);

define_id!(
    /// Opaque identifier for an attestation-revocation artifact.
    AttestationRevocationId
);

define_id!(
    /// Opaque identifier for an attestation-supersession artifact.
    AttestationSupersessionId
);

define_id!(
    /// Opaque identifier for a remote-oracle lease artifact.
    RemoteOracleLeaseId
);

define_id!(
    /// Opaque identifier for a remote-slice request artifact.
    RemoteSliceRequestId
);

define_id!(
    /// Opaque identifier for a remote-slice result artifact.
    RemoteSliceResultId
);

define_id!(
    /// Opaque identifier for a cross-runtime replay-ticket artifact.
    CrossRuntimeReplayTicketId
);

define_id!(
    /// Opaque identifier for a dispute-bundle artifact.
    DisputeBundleId
);

define_id!(
    /// Opaque identifier for a disclosure-policy artifact.
    DisclosurePolicyId
);

define_id!(
    /// Opaque identifier for a disclosure-budget artifact.
    DisclosureBudgetId
);

define_id!(
    /// Opaque identifier for a treaty bundle artifact.
    TreatyBundleId
);

define_id!(
    /// Opaque identifier for a runtime identity set artifact.
    RuntimeIdentitySetId
);

define_id!(
    /// Opaque identifier for a cross-runtime equivalence bundle artifact.
    CrossRuntimeEquivalenceBundleId
);

define_id!(
    /// Opaque identifier for a settlement case artifact.
    SettlementCaseId
);

define_id!(
    /// Opaque identifier for a shared disposition artifact.
    SharedDispositionId
);

define_id!(
    /// Opaque identifier for a local dissent record artifact.
    LocalDissentId
);

define_id!(
    /// Opaque identifier for a shared-view downgrade artifact.
    SharedViewDowngradeId
);

define_id!(
    /// Opaque identifier for a settlement receipt artifact.
    SettlementReceiptId
);

define_id!(
    /// Opaque identifier for a shared replay slice artifact.
    SharedReplaySliceId
);

define_id!(
    /// Opaque identifier for a shared divergence report artifact.
    SharedDivergenceReportId
);

define_id!(
    /// Opaque identifier for a treaty suspension artifact.
    TreatySuspensionId
);

define_id!(
    /// Opaque identifier for a mechanism bundle artifact.
    MechanismBundleId
);

define_id!(
    /// Opaque identifier for a theory version artifact.
    TheoryVersionId
);

define_id!(
    /// Opaque identifier for a theory library artifact.
    TheoryLibraryId
);

define_id!(
    /// Opaque identifier for a hypothesis library artifact.
    HypothesisLibraryId
);

define_id!(
    /// Opaque identifier for a simulation contract artifact.
    SimulationContractId
);

define_id!(
    /// Opaque identifier for a fit-run artifact.
    FitRunId
);

define_id!(
    /// Opaque identifier for a theory refuter suite artifact.
    TheoryRefuterSuiteId
);

define_id!(
    /// Opaque identifier for a rollout stability report artifact.
    RolloutStabilityReportId
);

define_id!(
    /// Opaque identifier for a discovery program artifact.
    DiscoveryProgramId
);

define_id!(
    /// Opaque identifier for a portfolio plan artifact.
    PortfolioPlanId
);

define_id!(
    /// Opaque identifier for an experiment campaign artifact.
    ExperimentCampaignId
);

define_id!(
    /// Opaque identifier for a campaign decision trace artifact.
    CampaignDecisionTraceId
);

define_id!(
    /// Opaque identifier for an information-value estimate artifact.
    InformationValueEstimateId
);

define_id!(
    /// Opaque identifier for a verification-load budget artifact.
    VerificationLoadBudgetId
);

define_id!(
    /// Opaque identifier for a charter bundle artifact.
    CharterBundleId
);

define_id!(
    /// Opaque identifier for a doctrine snapshot artifact.
    DoctrineSnapshotId
);

define_id!(
    /// Opaque identifier for an amendment proposal artifact.
    AmendmentProposalId
);

define_id!(
    /// Opaque identifier for an amendment decision artifact.
    AmendmentDecisionId
);

define_id!(
    /// Opaque identifier for an archive manifest artifact.
    ArchiveManifestId
);

define_id!(
    /// Opaque identifier for a compaction receipt artifact.
    CompactionReceiptId
);

define_id!(
    /// Opaque identifier for a historical query guarantee artifact.
    HistoricalQueryGuaranteeId
);

define_id!(
    /// Opaque identifier for a deprecation bundle artifact.
    DeprecationBundleId
);

define_id!(
    /// Opaque identifier for a retirement bundle artifact.
    RetirementBundleId
);

define_id!(
    /// Opaque identifier for a spec bundle artifact.
    SpecBundleId
);

define_id!(
    /// Opaque identifier for a normative AST artifact.
    NormativeAstId
);

define_id!(
    /// Opaque identifier for a generated schema bundle artifact.
    GeneratedSchemaBundleId
);

define_id!(
    /// Opaque identifier for a generated interpreter bundle artifact.
    GeneratedInterpreterBundleId
);

define_id!(
    /// Opaque identifier for a generated conformance corpus artifact.
    GeneratedConformanceCorpusId
);

define_id!(
    /// Opaque identifier for a generated migration plan artifact.
    GeneratedMigrationPlanId
);

define_id!(
    /// Opaque identifier for a proof-obligation set artifact.
    ProofObligationSetId
);

define_id!(
    /// Opaque identifier for a proof evaluation receipt artifact.
    ProofEvaluationReceiptId
);

define_id!(
    /// Opaque identifier for a human-veto bundle artifact.
    HumanVetoBundleId
);

define_id!(
    /// Opaque identifier for a meta-challenge bundle artifact.
    MetaChallengeBundleId
);

define_id!(
    /// Opaque identifier for a self-hosting build receipt artifact.
    SelfHostingBuildReceiptId
);

define_id!(
    /// Opaque identifier for an effect intent artifact.
    EffectIntentId
);

define_id!(
    /// Opaque identifier for an effect preflight report artifact.
    EffectPreflightReportId
);

define_id!(
    /// Opaque identifier for an effect execution window artifact.
    EffectWindowId
);

define_id!(
    /// Opaque identifier for an effect commit decision artifact.
    EffectCommitDecisionId
);

define_id!(
    /// Opaque identifier for an effect execution receipt artifact.
    EffectExecutionReceiptId
);

define_id!(
    /// Opaque identifier for an effect observation bundle artifact.
    EffectObservationBundleId
);

define_id!(
    /// Opaque identifier for a compensation plan artifact.
    CompensationPlanId
);

define_id!(
    /// Opaque identifier for a compensation execution receipt artifact.
    CompensationExecutionReceiptId
);

define_id!(
    /// Opaque identifier for an external effect ledger entry artifact.
    ExternalEffectLedgerEntryId
);

define_id!(
    /// Opaque identifier for a capability class artifact.
    CapabilityClassId
);

define_id!(
    /// Opaque identifier for an authority lease artifact.
    AuthorityLeaseId
);

define_id!(
    /// Opaque identifier for a delegation bundle artifact.
    DelegationBundleId
);

define_id!(
    /// Opaque identifier for an authority chain artifact.
    AuthorityChainId
);

define_id!(
    /// Opaque identifier for a separation-of-duties policy artifact.
    SeparationOfDutiesPolicyId
);

define_id!(
    /// Opaque identifier for a dual-control approval artifact.
    DualControlApprovalId
);

define_id!(
    /// Opaque identifier for a break-glass grant artifact.
    BreakGlassGrantId
);

define_id!(
    /// Opaque identifier for a delegation revocation artifact.
    DelegationRevocationId
);

define_id!(
    /// Opaque identifier for an acting-on-behalf receipt artifact.
    ActingOnBehalfReceiptId
);

define_id!(
    /// Opaque identifier for a conflict disclosure artifact.
    ConflictDisclosureId
);

define_id!(
    /// Opaque identifier for a deployment profile artifact.
    DeploymentProfileId
);

define_id!(
    /// Opaque identifier for an operating envelope artifact.
    OperatingEnvelopeId
);

define_id!(
    /// Opaque identifier for an assurance case artifact.
    AssuranceCaseId
);

define_id!(
    /// Opaque identifier for a hazard register artifact.
    HazardRegisterId
);

define_id!(
    /// Opaque identifier for a control mapping artifact.
    ControlMappingId
);

define_id!(
    /// Opaque identifier for a residual-risk acceptance artifact.
    ResidualRiskAcceptanceId
);

define_id!(
    /// Opaque identifier for a release-readiness decision artifact.
    ReleaseReadinessDecisionId
);

define_id!(
    /// Opaque identifier for a field monitoring plan artifact.
    FieldMonitoringPlanId
);

define_id!(
    /// Opaque identifier for a certification bundle artifact.
    CertificationBundleId
);

define_id!(
    /// Opaque identifier for a recertification trigger artifact.
    RecertificationTriggerId
);

define_id!(
    /// Opaque identifier for a service-level profile artifact.
    ServiceLevelProfileId
);

define_id!(
    /// Opaque identifier for an error-budget ledger artifact.
    ErrorBudgetLedgerId
);

define_id!(
    /// Opaque identifier for an incident case artifact.
    IncidentCaseId
);

define_id!(
    /// Opaque identifier for a containment decision artifact.
    ContainmentDecisionId
);

define_id!(
    /// Opaque identifier for a forensic freeze artifact.
    ForensicFreezeId
);

define_id!(
    /// Opaque identifier for a recovery plan artifact.
    RecoveryPlanId
);

define_id!(
    /// Opaque identifier for a recovery replay slice artifact.
    RecoveryReplaySliceId
);

define_id!(
    /// Opaque identifier for a bounded continuity exception artifact.
    ContinuityExceptionId
);

define_id!(
    /// Opaque identifier for a postmortem bundle artifact.
    PostmortemBundleId
);

define_id!(
    /// Opaque identifier for a resilience exercise artifact.
    ResilienceExerciseId
);

define_id!(
    /// Opaque identifier for an effect review case artifact.
    EffectReviewCaseId
);

define_id!(
    /// Opaque identifier for an effect block receipt artifact.
    EffectBlockReceiptId
);

define_id!(
    /// Opaque identifier for a delegation review case artifact.
    DelegationReviewCaseId
);

define_id!(
    /// Opaque identifier for a release gate case artifact.
    ReleaseGateCaseId
);

define_id!(
    /// Opaque identifier for a continuity review case artifact.
    ContinuityReviewCaseId
);

define_id!(
    /// Opaque identifier for an effect policy profile artifact.
    EffectPolicyProfileId
);

define_id!(
    /// Opaque identifier for a delegation policy profile artifact.
    DelegationPolicyProfileId
);

define_id!(
    /// Opaque identifier for a release policy profile artifact.
    ReleasePolicyProfileId
);

define_id!(
    /// Opaque identifier for a continuity policy profile artifact.
    ContinuityPolicyProfileId
);

define_id!(
    /// Opaque identifier for an effect adjudication receipt artifact.
    EffectAdjudicationReceiptId
);

define_id!(
    /// Opaque identifier for a release rollback decision artifact.
    ReleaseRollbackDecisionId
);

define_id!(
    /// Opaque identifier for a tool effect dispatch receipt artifact.
    ToolEffectDispatchReceiptId
);

define_id!(
    /// Opaque identifier for a privacy/retention profile artifact.
    PrivacyRetentionProfileId
);

define_id!(
    /// Opaque identifier for a redaction rule-set artifact.
    RedactionRuleSetId
);

define_id!(
    /// Opaque identifier for an access-purpose matrix artifact.
    AccessPurposeMatrixId
);

define_id!(
    /// Opaque identifier for an audit extraction policy artifact.
    AuditExtractionPolicyId
);

define_id!(
    /// Opaque identifier for a residency policy profile artifact.
    ResidencyPolicyProfileId
);

define_id!(
    /// Opaque identifier for a tenant-boundary profile artifact.
    TenantBoundaryProfileId
);

define_id!(
    /// Opaque identifier for a cross-boundary transfer class artifact.
    CrossBoundaryTransferClassId
);

define_id!(
    /// Opaque identifier for a locality exception artifact.
    LocalityExceptionId
);

define_id!(
    /// Opaque identifier for a role catalog artifact.
    RoleCatalogId
);

define_id!(
    /// Opaque identifier for a delegation matrix artifact.
    DelegationMatrixId
);

define_id!(
    /// Opaque identifier for an approval matrix artifact.
    ApprovalMatrixId
);

define_id!(
    /// Opaque identifier for a conflict-class catalog artifact.
    ConflictClassCatalogId
);

define_id!(
    /// Opaque identifier for a regulatory-regime profile artifact.
    RegulatoryRegimeProfileId
);

define_id!(
    /// Opaque identifier for a requirement-to-control map artifact.
    RequirementControlMapId
);

define_id!(
    /// Opaque identifier for an evidence collection plan artifact.
    EvidenceCollectionPlanId
);

define_id!(
    /// Opaque identifier for a recertification schedule artifact.
    RecertificationScheduleId
);

define_id!(
    /// Opaque identifier for a hazard library artifact.
    HazardLibraryId
);

define_id!(
    /// Opaque identifier for a hazard scenario artifact.
    HazardScenarioId
);

define_id!(
    /// Opaque identifier for a monitor catalog artifact.
    MonitorCatalogId
);

define_id!(
    /// Opaque identifier for a mitigation playbook artifact.
    MitigationPlaybookId
);

define_id!(
    /// Opaque identifier for a vendor certification adapter artifact.
    VendorCertificationAdapterId
);

define_id!(
    /// Opaque identifier for a vendor evidence translation artifact.
    VendorEvidenceTranslationId
);

define_id!(
    /// Opaque identifier for a vendor trust-root binding artifact.
    VendorTrustRootBindingId
);

define_id!(
    /// Opaque identifier for a vendor revocation handling artifact.
    VendorRevocationHandlingId
);

define_id!(
    /// Opaque identifier for an incident taxonomy artifact.
    IncidentTaxonomyId
);

define_id!(
    /// Opaque identifier for a severity matrix artifact.
    SeverityMatrixId
);

define_id!(
    /// Opaque identifier for a pager-route profile artifact.
    PagerRouteProfileId
);

define_id!(
    /// Opaque identifier for an escalation-clock policy artifact.
    EscalationClockPolicyId
);


#[cfg(test)]
mod tests {
    use super::*;
    use std::any::TypeId;

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    struct VersionedOperatorIdentity {
        operator_id: OperatorId,
        operator_version_id: OperatorVersionId,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    struct KernelArtifactIdentity {
        kernel_run_id: KernelRunId,
        constraint_id: ConstraintId,
        hyperedge_id: HyperedgeId,
        residual_id: ResidualId,
        syndrome_id: SyndromeId,
        witness_id: WitnessId,
        certificate_id: CertificateId,
        oracle_slice_id: OracleSliceId,
        region_id: RegionId,
        region_digest_id: RegionDigestId,
        artifact_transport_id: ArtifactTransportId,
        repair_route_id: RepairRouteId,
        repair_candidate_id: RepairCandidateId,
        nuisance_state_id: NuisanceStateId,
        convergence_report_id: ConvergenceReportId,
        refutation_result_id: RefutationResultId,
        calibration_report_id: CalibrationReportId,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    struct ControlPlaneIdentity {
        verification_case_id: VerificationCaseId,
        check_plan_id: CheckPlanId,
        control_receipt_id: ControlReceiptId,
        ledger_entry_id: LedgerEntryId,
        policy_decision_id: PolicyDecisionId,
        approval_record_id: ApprovalRecordId,
        promotion_decision_id: PromotionDecisionId,
        refutation_decision_id: RefutationDecisionId,
        rollback_plan_id: RollbackPlanId,
        calibration_snapshot_id: CalibrationSnapshotId,
        learning_update_id: LearningUpdateId,
        boundary_repair_record_id: BoundaryRepairRecordId,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    struct SemanticContractIdentity {
        semantics_profile_id: SemanticsProfileId,
        claim_state_id: ClaimStateId,
        semantic_diff_id: SemanticDiffId,
        causal_attribution_bundle_id: CausalAttributionBundleId,
        degradation_record_id: DegradationRecordId,
        exactness_budget_id: ExactnessBudgetId,
        support_set_id: SupportSetId,
        contradiction_witness_id: ContradictionWitnessId,
        retraction_record_id: RetractionRecordId,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    struct EndgameIdentity {
        intervention_id: InterventionId,
        outcome_schema_id: OutcomeSchemaId,
        experiment_case_id: ExperimentCaseId,
        cohort_contract_id: CohortContractId,
        comparability_matrix_id: ComparabilityMatrixId,
        counterfactual_slice_id: CounterfactualSliceId,
        decision_trace_id: DecisionTraceId,
        refuter_suite_id: RefuterSuiteId,
        refuter_result_id: RefuterResultId,
        rollout_decision_id: RolloutDecisionId,
        rollback_decision_id: RollbackDecisionId,
        experiment_budget_id: ExperimentBudgetId,
        attestation_envelope_id: AttestationEnvelopeId,
        trust_root_set_id: TrustRootSetId,
        artifact_admission_policy_id: ArtifactAdmissionPolicyId,
        transparency_receipt_id: TransparencyReceiptId,
        attestation_revocation_id: AttestationRevocationId,
        attestation_supersession_id: AttestationSupersessionId,
        remote_oracle_lease_id: RemoteOracleLeaseId,
        remote_slice_request_id: RemoteSliceRequestId,
        remote_slice_result_id: RemoteSliceResultId,
        cross_runtime_replay_ticket_id: CrossRuntimeReplayTicketId,
        dispute_bundle_id: DisputeBundleId,
        disclosure_policy_id: DisclosurePolicyId,
        disclosure_budget_id: DisclosureBudgetId,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    struct HorizonIdentity {
        treaty_bundle_id: TreatyBundleId,
        runtime_identity_set_id: RuntimeIdentitySetId,
        cross_runtime_equivalence_bundle_id: CrossRuntimeEquivalenceBundleId,
        settlement_case_id: SettlementCaseId,
        shared_disposition_id: SharedDispositionId,
        local_dissent_id: LocalDissentId,
        shared_view_downgrade_id: SharedViewDowngradeId,
        settlement_receipt_id: SettlementReceiptId,
        shared_replay_slice_id: SharedReplaySliceId,
        shared_divergence_report_id: SharedDivergenceReportId,
        treaty_suspension_id: TreatySuspensionId,
        mechanism_bundle_id: MechanismBundleId,
        theory_version_id: TheoryVersionId,
        theory_library_id: TheoryLibraryId,
        hypothesis_library_id: HypothesisLibraryId,
        simulation_contract_id: SimulationContractId,
        fit_run_id: FitRunId,
        theory_refuter_suite_id: TheoryRefuterSuiteId,
        rollout_stability_report_id: RolloutStabilityReportId,
        discovery_program_id: DiscoveryProgramId,
        portfolio_plan_id: PortfolioPlanId,
        experiment_campaign_id: ExperimentCampaignId,
        campaign_decision_trace_id: CampaignDecisionTraceId,
        information_value_estimate_id: InformationValueEstimateId,
        verification_load_budget_id: VerificationLoadBudgetId,
        charter_bundle_id: CharterBundleId,
        doctrine_snapshot_id: DoctrineSnapshotId,
        amendment_proposal_id: AmendmentProposalId,
        amendment_decision_id: AmendmentDecisionId,
        archive_manifest_id: ArchiveManifestId,
        compaction_receipt_id: CompactionReceiptId,
        historical_query_guarantee_id: HistoricalQueryGuaranteeId,
        deprecation_bundle_id: DeprecationBundleId,
        retirement_bundle_id: RetirementBundleId,
        spec_bundle_id: SpecBundleId,
        normative_ast_id: NormativeAstId,
        generated_schema_bundle_id: GeneratedSchemaBundleId,
        generated_interpreter_bundle_id: GeneratedInterpreterBundleId,
        generated_conformance_corpus_id: GeneratedConformanceCorpusId,
        generated_migration_plan_id: GeneratedMigrationPlanId,
        proof_obligation_set_id: ProofObligationSetId,
        proof_evaluation_receipt_id: ProofEvaluationReceiptId,
        human_veto_bundle_id: HumanVetoBundleId,
        meta_challenge_bundle_id: MetaChallengeBundleId,
        self_hosting_build_receipt_id: SelfHostingBuildReceiptId,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    struct FinalCloseoutIdentity {
        effect_intent_id: EffectIntentId,
        effect_preflight_report_id: EffectPreflightReportId,
        effect_window_id: EffectWindowId,
        effect_commit_decision_id: EffectCommitDecisionId,
        effect_execution_receipt_id: EffectExecutionReceiptId,
        effect_observation_bundle_id: EffectObservationBundleId,
        compensation_plan_id: CompensationPlanId,
        compensation_execution_receipt_id: CompensationExecutionReceiptId,
        external_effect_ledger_entry_id: ExternalEffectLedgerEntryId,
        capability_class_id: CapabilityClassId,
        authority_lease_id: AuthorityLeaseId,
        delegation_bundle_id: DelegationBundleId,
        authority_chain_id: AuthorityChainId,
        separation_of_duties_policy_id: SeparationOfDutiesPolicyId,
        dual_control_approval_id: DualControlApprovalId,
        break_glass_grant_id: BreakGlassGrantId,
        delegation_revocation_id: DelegationRevocationId,
        acting_on_behalf_receipt_id: ActingOnBehalfReceiptId,
        conflict_disclosure_id: ConflictDisclosureId,
        deployment_profile_id: DeploymentProfileId,
        operating_envelope_id: OperatingEnvelopeId,
        assurance_case_id: AssuranceCaseId,
        hazard_register_id: HazardRegisterId,
        control_mapping_id: ControlMappingId,
        residual_risk_acceptance_id: ResidualRiskAcceptanceId,
        release_readiness_decision_id: ReleaseReadinessDecisionId,
        field_monitoring_plan_id: FieldMonitoringPlanId,
        certification_bundle_id: CertificationBundleId,
        recertification_trigger_id: RecertificationTriggerId,
        service_level_profile_id: ServiceLevelProfileId,
        error_budget_ledger_id: ErrorBudgetLedgerId,
        incident_case_id: IncidentCaseId,
        containment_decision_id: ContainmentDecisionId,
        forensic_freeze_id: ForensicFreezeId,
        recovery_plan_id: RecoveryPlanId,
        recovery_replay_slice_id: RecoveryReplaySliceId,
        continuity_exception_id: ContinuityExceptionId,
        postmortem_bundle_id: PostmortemBundleId,
        resilience_exercise_id: ResilienceExerciseId,
        effect_review_case_id: EffectReviewCaseId,
        effect_block_receipt_id: EffectBlockReceiptId,
        delegation_review_case_id: DelegationReviewCaseId,
        release_gate_case_id: ReleaseGateCaseId,
        continuity_review_case_id: ContinuityReviewCaseId,
        effect_policy_profile_id: EffectPolicyProfileId,
        delegation_policy_profile_id: DelegationPolicyProfileId,
        release_policy_profile_id: ReleasePolicyProfileId,
        continuity_policy_profile_id: ContinuityPolicyProfileId,
        effect_adjudication_receipt_id: EffectAdjudicationReceiptId,
        release_rollback_decision_id: ReleaseRollbackDecisionId,
        tool_effect_dispatch_receipt_id: ToolEffectDispatchReceiptId,
    }

    macro_rules! assert_id_roundtrip {
        ($ty:ty, $raw:expr) => {{
            let id = <$ty>::new($raw);
            let parsed: $ty = $raw.parse().unwrap();
            assert_eq!(parsed, id);
            assert_eq!(parsed.to_string(), $raw);

            let encoded = serde_json::to_string(&id).unwrap();
            let decoded: $ty = serde_json::from_str(&encoded).unwrap();
            assert_eq!(decoded, id);
        }};
    }

    #[test]
    fn id_creation_and_display() {
        let id = EnvelopeId::new("env-001");
        assert_eq!(id.as_str(), "env-001");
        assert_eq!(id.to_string(), "env-001");
        assert!(!id.is_empty());
    }

    #[test]
    fn id_from_string() {
        let id: ClaimId = "claim-123".into();
        assert_eq!(id.as_str(), "claim-123");
    }

    #[test]
    fn id_generate_is_unique() {
        let a = AttemptId::generate();
        let b = AttemptId::generate();
        assert_ne!(a, b);
    }

    #[test]
    fn id_empty_check() {
        let id = EntityId::new("");
        assert!(id.is_empty());

        let id = EntityId::new("e-1");
        assert!(!id.is_empty());
    }

    #[test]
    fn id_serde_roundtrip() {
        let id = TrialId::new("trial-42");
        let json = serde_json::to_string(&id).unwrap();
        assert_eq!(json, "\"trial-42\"");
        let back: TrialId = serde_json::from_str(&json).unwrap();
        assert_eq!(back, id);
    }

    #[test]
    fn id_ordering() {
        let a = EnvelopeId::new("aaa");
        let b = EnvelopeId::new("bbb");
        assert!(a < b);
    }

    #[test]
    fn kernel_artifact_ids_support_parse_display_and_serde_roundtrip() {
        assert_id_roundtrip!(KernelRunId, "run-1");
        assert_id_roundtrip!(ConstraintId, "constraint-1");
        assert_id_roundtrip!(HyperedgeId, "hyperedge-1");
        assert_id_roundtrip!(ResidualId, "residual-1");
        assert_id_roundtrip!(SyndromeId, "syndrome-1");
        assert_id_roundtrip!(WitnessId, "witness-1");
        assert_id_roundtrip!(CertificateId, "certificate-1");
        assert_id_roundtrip!(OracleSliceId, "oracle-slice-1");
        assert_id_roundtrip!(RegionId, "region-1");
        assert_id_roundtrip!(RegionDigestId, "region-digest-1");
        assert_id_roundtrip!(ArtifactTransportId, "artifact-transport-1");
        assert_id_roundtrip!(RepairRouteId, "repair-route-1");
        assert_id_roundtrip!(RepairCandidateId, "repair-candidate-1");
        assert_id_roundtrip!(NuisanceStateId, "nuisance-state-1");
        assert_id_roundtrip!(ConvergenceReportId, "convergence-report-1");
        assert_id_roundtrip!(RefutationResultId, "refutation-1");
        assert_id_roundtrip!(OperatorId, "operator-1");
        assert_id_roundtrip!(OperatorVersionId, "operator-version-1");
        assert_id_roundtrip!(CalibrationReportId, "calibration-1");
        assert_id_roundtrip!(VerificationCaseId, "case-1");
        assert_id_roundtrip!(CheckPlanId, "plan-1");
        assert_id_roundtrip!(ControlReceiptId, "control-receipt-1");
        assert_id_roundtrip!(LedgerEntryId, "ledger-entry-1");
        assert_id_roundtrip!(PolicyDecisionId, "policy-1");
        assert_id_roundtrip!(ApprovalRecordId, "approval-1");
        assert_id_roundtrip!(PromotionDecisionId, "promotion-1");
        assert_id_roundtrip!(RefutationDecisionId, "refutation-decision-1");
        assert_id_roundtrip!(RollbackPlanId, "rollback-1");
        assert_id_roundtrip!(CalibrationSnapshotId, "calibration-snapshot-1");
        assert_id_roundtrip!(LearningUpdateId, "learning-update-1");
        assert_id_roundtrip!(BoundaryRepairRecordId, "boundary-repair-1");
        assert_id_roundtrip!(SemanticsProfileId, "semantics-profile-1");
        assert_id_roundtrip!(ClaimStateId, "claim-state-1");
        assert_id_roundtrip!(SemanticDiffId, "semantic-diff-1");
        assert_id_roundtrip!(CausalAttributionBundleId, "causal-attribution-1");
        assert_id_roundtrip!(DegradationRecordId, "degradation-record-1");
        assert_id_roundtrip!(ExactnessBudgetId, "exactness-budget-1");
    }

    #[test]
    fn endgame_ids_support_parse_display_and_serde_roundtrip() {
        assert_id_roundtrip!(InterventionId, "intervention-1");
        assert_id_roundtrip!(OutcomeSchemaId, "outcome-schema-1");
        assert_id_roundtrip!(ExperimentCaseId, "experiment-case-1");
        assert_id_roundtrip!(CohortContractId, "cohort-contract-1");
        assert_id_roundtrip!(ComparabilityMatrixId, "comparability-matrix-1");
        assert_id_roundtrip!(CounterfactualSliceId, "counterfactual-slice-1");
        assert_id_roundtrip!(DecisionTraceId, "decision-trace-1");
        assert_id_roundtrip!(RefuterSuiteId, "refuter-suite-1");
        assert_id_roundtrip!(RefuterResultId, "refuter-result-1");
        assert_id_roundtrip!(RolloutDecisionId, "rollout-decision-1");
        assert_id_roundtrip!(RollbackDecisionId, "rollback-decision-1");
        assert_id_roundtrip!(ExperimentBudgetId, "experiment-budget-1");
        assert_id_roundtrip!(AttestationEnvelopeId, "attestation-envelope-1");
        assert_id_roundtrip!(TrustRootSetId, "trust-root-set-1");
        assert_id_roundtrip!(ArtifactAdmissionPolicyId, "artifact-admission-policy-1");
        assert_id_roundtrip!(TransparencyReceiptId, "transparency-receipt-1");
        assert_id_roundtrip!(AttestationRevocationId, "attestation-revocation-1");
        assert_id_roundtrip!(AttestationSupersessionId, "attestation-supersession-1");
        assert_id_roundtrip!(RemoteOracleLeaseId, "remote-oracle-lease-1");
        assert_id_roundtrip!(RemoteSliceRequestId, "remote-slice-request-1");
        assert_id_roundtrip!(RemoteSliceResultId, "remote-slice-result-1");
        assert_id_roundtrip!(CrossRuntimeReplayTicketId, "cross-runtime-replay-ticket-1");
        assert_id_roundtrip!(DisputeBundleId, "dispute-bundle-1");
        assert_id_roundtrip!(DisclosurePolicyId, "disclosure-policy-1");
        assert_id_roundtrip!(DisclosureBudgetId, "disclosure-budget-1");
        assert_id_roundtrip!(TreatyBundleId, "treaty-bundle-1");
        assert_id_roundtrip!(RuntimeIdentitySetId, "runtime-identity-set-1");
        assert_id_roundtrip!(
            CrossRuntimeEquivalenceBundleId,
            "cross-runtime-equivalence-bundle-1"
        );
        assert_id_roundtrip!(SettlementCaseId, "settlement-case-1");
        assert_id_roundtrip!(SharedDispositionId, "shared-disposition-1");
        assert_id_roundtrip!(LocalDissentId, "local-dissent-1");
        assert_id_roundtrip!(SharedViewDowngradeId, "shared-view-downgrade-1");
        assert_id_roundtrip!(SettlementReceiptId, "settlement-receipt-1");
        assert_id_roundtrip!(SharedReplaySliceId, "shared-replay-slice-1");
        assert_id_roundtrip!(SharedDivergenceReportId, "shared-divergence-report-1");
        assert_id_roundtrip!(TreatySuspensionId, "treaty-suspension-1");
        assert_id_roundtrip!(MechanismBundleId, "mechanism-bundle-1");
        assert_id_roundtrip!(TheoryVersionId, "theory-version-1");
        assert_id_roundtrip!(TheoryLibraryId, "theory-library-1");
        assert_id_roundtrip!(HypothesisLibraryId, "hypothesis-library-1");
        assert_id_roundtrip!(SimulationContractId, "simulation-contract-1");
        assert_id_roundtrip!(FitRunId, "fit-run-1");
        assert_id_roundtrip!(TheoryRefuterSuiteId, "theory-refuter-suite-1");
        assert_id_roundtrip!(RolloutStabilityReportId, "rollout-stability-report-1");
        assert_id_roundtrip!(DiscoveryProgramId, "discovery-program-1");
        assert_id_roundtrip!(PortfolioPlanId, "portfolio-plan-1");
        assert_id_roundtrip!(ExperimentCampaignId, "experiment-campaign-1");
        assert_id_roundtrip!(CampaignDecisionTraceId, "campaign-decision-trace-1");
        assert_id_roundtrip!(InformationValueEstimateId, "information-value-estimate-1");
        assert_id_roundtrip!(VerificationLoadBudgetId, "verification-load-budget-1");
        assert_id_roundtrip!(CharterBundleId, "charter-bundle-1");
        assert_id_roundtrip!(DoctrineSnapshotId, "doctrine-snapshot-1");
        assert_id_roundtrip!(AmendmentProposalId, "amendment-proposal-1");
        assert_id_roundtrip!(AmendmentDecisionId, "amendment-decision-1");
        assert_id_roundtrip!(ArchiveManifestId, "archive-manifest-1");
        assert_id_roundtrip!(CompactionReceiptId, "compaction-receipt-1");
        assert_id_roundtrip!(HistoricalQueryGuaranteeId, "historical-query-guarantee-1");
        assert_id_roundtrip!(DeprecationBundleId, "deprecation-bundle-1");
        assert_id_roundtrip!(RetirementBundleId, "retirement-bundle-1");
        assert_id_roundtrip!(SpecBundleId, "spec-bundle-1");
        assert_id_roundtrip!(NormativeAstId, "normative-ast-1");
        assert_id_roundtrip!(GeneratedSchemaBundleId, "generated-schema-bundle-1");
        assert_id_roundtrip!(
            GeneratedInterpreterBundleId,
            "generated-interpreter-bundle-1"
        );
        assert_id_roundtrip!(
            GeneratedConformanceCorpusId,
            "generated-conformance-corpus-1"
        );
        assert_id_roundtrip!(GeneratedMigrationPlanId, "generated-migration-plan-1");
        assert_id_roundtrip!(ProofObligationSetId, "proof-obligation-set-1");
        assert_id_roundtrip!(ProofEvaluationReceiptId, "proof-evaluation-receipt-1");
        assert_id_roundtrip!(HumanVetoBundleId, "human-veto-bundle-1");
        assert_id_roundtrip!(MetaChallengeBundleId, "meta-challenge-bundle-1");
        assert_id_roundtrip!(SelfHostingBuildReceiptId, "self-hosting-build-receipt-1");
    }

    #[test]
    fn final_closeout_ids_support_parse_display_and_serde_roundtrip() {
        assert_id_roundtrip!(EffectIntentId, "effect-intent-1");
        assert_id_roundtrip!(EffectPreflightReportId, "effect-preflight-report-1");
        assert_id_roundtrip!(EffectWindowId, "effect-window-1");
        assert_id_roundtrip!(EffectCommitDecisionId, "effect-commit-decision-1");
        assert_id_roundtrip!(EffectExecutionReceiptId, "effect-execution-receipt-1");
        assert_id_roundtrip!(EffectObservationBundleId, "effect-observation-bundle-1");
        assert_id_roundtrip!(CompensationPlanId, "compensation-plan-1");
        assert_id_roundtrip!(
            CompensationExecutionReceiptId,
            "compensation-execution-receipt-1"
        );
        assert_id_roundtrip!(ExternalEffectLedgerEntryId, "external-effect-ledger-entry-1");
        assert_id_roundtrip!(CapabilityClassId, "capability-class-1");
        assert_id_roundtrip!(AuthorityLeaseId, "authority-lease-1");
        assert_id_roundtrip!(DelegationBundleId, "delegation-bundle-1");
        assert_id_roundtrip!(AuthorityChainId, "authority-chain-1");
        assert_id_roundtrip!(
            SeparationOfDutiesPolicyId,
            "separation-of-duties-policy-1"
        );
        assert_id_roundtrip!(DualControlApprovalId, "dual-control-approval-1");
        assert_id_roundtrip!(BreakGlassGrantId, "break-glass-grant-1");
        assert_id_roundtrip!(DelegationRevocationId, "delegation-revocation-1");
        assert_id_roundtrip!(ActingOnBehalfReceiptId, "acting-on-behalf-receipt-1");
        assert_id_roundtrip!(ConflictDisclosureId, "conflict-disclosure-1");
        assert_id_roundtrip!(DeploymentProfileId, "deployment-profile-1");
        assert_id_roundtrip!(OperatingEnvelopeId, "operating-envelope-1");
        assert_id_roundtrip!(AssuranceCaseId, "assurance-case-1");
        assert_id_roundtrip!(HazardRegisterId, "hazard-register-1");
        assert_id_roundtrip!(ControlMappingId, "control-mapping-1");
        assert_id_roundtrip!(ResidualRiskAcceptanceId, "residual-risk-acceptance-1");
        assert_id_roundtrip!(
            ReleaseReadinessDecisionId,
            "release-readiness-decision-1"
        );
        assert_id_roundtrip!(FieldMonitoringPlanId, "field-monitoring-plan-1");
        assert_id_roundtrip!(CertificationBundleId, "certification-bundle-1");
        assert_id_roundtrip!(RecertificationTriggerId, "recertification-trigger-1");
        assert_id_roundtrip!(ServiceLevelProfileId, "service-level-profile-1");
        assert_id_roundtrip!(ErrorBudgetLedgerId, "error-budget-ledger-1");
        assert_id_roundtrip!(IncidentCaseId, "incident-case-1");
        assert_id_roundtrip!(ContainmentDecisionId, "containment-decision-1");
        assert_id_roundtrip!(ForensicFreezeId, "forensic-freeze-1");
        assert_id_roundtrip!(RecoveryPlanId, "recovery-plan-1");
        assert_id_roundtrip!(RecoveryReplaySliceId, "recovery-replay-slice-1");
        assert_id_roundtrip!(ContinuityExceptionId, "continuity-exception-1");
        assert_id_roundtrip!(PostmortemBundleId, "postmortem-bundle-1");
        assert_id_roundtrip!(ResilienceExerciseId, "resilience-exercise-1");
        assert_id_roundtrip!(EffectReviewCaseId, "effect-review-case-1");
        assert_id_roundtrip!(EffectBlockReceiptId, "effect-block-receipt-1");
        assert_id_roundtrip!(DelegationReviewCaseId, "delegation-review-case-1");
        assert_id_roundtrip!(ReleaseGateCaseId, "release-gate-case-1");
        assert_id_roundtrip!(ContinuityReviewCaseId, "continuity-review-case-1");
        assert_id_roundtrip!(EffectPolicyProfileId, "effect-policy-profile-1");
        assert_id_roundtrip!(DelegationPolicyProfileId, "delegation-policy-profile-1");
        assert_id_roundtrip!(ReleasePolicyProfileId, "release-policy-profile-1");
        assert_id_roundtrip!(ContinuityPolicyProfileId, "continuity-policy-profile-1");
        assert_id_roundtrip!(
            EffectAdjudicationReceiptId,
            "effect-adjudication-receipt-1"
        );
        assert_id_roundtrip!(
            ReleaseRollbackDecisionId,
            "release-rollback-decision-1"
        );
        assert_id_roundtrip!(
            ToolEffectDispatchReceiptId,
            "tool-effect-dispatch-receipt-1"
        );
    }

    #[test]
    fn profile_completion_ids_support_parse_display_and_serde_roundtrip() {
        assert_id_roundtrip!(PrivacyRetentionProfileId, "privacy-retention-profile-1");
        assert_id_roundtrip!(RedactionRuleSetId, "redaction-rule-set-1");
        assert_id_roundtrip!(AccessPurposeMatrixId, "access-purpose-matrix-1");
        assert_id_roundtrip!(AuditExtractionPolicyId, "audit-extraction-policy-1");
        assert_id_roundtrip!(ResidencyPolicyProfileId, "residency-policy-profile-1");
        assert_id_roundtrip!(TenantBoundaryProfileId, "tenant-boundary-profile-1");
        assert_id_roundtrip!(CrossBoundaryTransferClassId, "cross-boundary-transfer-class-1");
        assert_id_roundtrip!(LocalityExceptionId, "locality-exception-1");
        assert_id_roundtrip!(RoleCatalogId, "role-catalog-1");
        assert_id_roundtrip!(DelegationMatrixId, "delegation-matrix-1");
        assert_id_roundtrip!(ApprovalMatrixId, "approval-matrix-1");
        assert_id_roundtrip!(ConflictClassCatalogId, "conflict-class-catalog-1");
        assert_id_roundtrip!(RegulatoryRegimeProfileId, "regulatory-regime-profile-1");
        assert_id_roundtrip!(RequirementControlMapId, "requirement-control-map-1");
        assert_id_roundtrip!(EvidenceCollectionPlanId, "evidence-collection-plan-1");
        assert_id_roundtrip!(RecertificationScheduleId, "recertification-schedule-1");
        assert_id_roundtrip!(HazardLibraryId, "hazard-library-1");
        assert_id_roundtrip!(HazardScenarioId, "hazard-scenario-1");
        assert_id_roundtrip!(MonitorCatalogId, "monitor-catalog-1");
        assert_id_roundtrip!(MitigationPlaybookId, "mitigation-playbook-1");
        assert_id_roundtrip!(VendorCertificationAdapterId, "vendor-certification-adapter-1");
        assert_id_roundtrip!(VendorEvidenceTranslationId, "vendor-evidence-translation-1");
        assert_id_roundtrip!(VendorTrustRootBindingId, "vendor-trust-root-binding-1");
        assert_id_roundtrip!(VendorRevocationHandlingId, "vendor-revocation-handling-1");
        assert_id_roundtrip!(IncidentTaxonomyId, "incident-taxonomy-1");
        assert_id_roundtrip!(SeverityMatrixId, "severity-matrix-1");
        assert_id_roundtrip!(PagerRouteProfileId, "pager-route-profile-1");
        assert_id_roundtrip!(EscalationClockPolicyId, "escalation-clock-policy-1");

        #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
        struct ProfileCompletionIdentity {
            privacy_retention_profile_id: PrivacyRetentionProfileId,
            redaction_rule_set_id: RedactionRuleSetId,
            access_purpose_matrix_id: AccessPurposeMatrixId,
            audit_extraction_policy_id: AuditExtractionPolicyId,
            residency_policy_profile_id: ResidencyPolicyProfileId,
            tenant_boundary_profile_id: TenantBoundaryProfileId,
            cross_boundary_transfer_class_id: CrossBoundaryTransferClassId,
            locality_exception_id: LocalityExceptionId,
            role_catalog_id: RoleCatalogId,
            delegation_matrix_id: DelegationMatrixId,
            approval_matrix_id: ApprovalMatrixId,
            conflict_class_catalog_id: ConflictClassCatalogId,
            regulatory_regime_profile_id: RegulatoryRegimeProfileId,
            requirement_control_map_id: RequirementControlMapId,
            evidence_collection_plan_id: EvidenceCollectionPlanId,
            recertification_schedule_id: RecertificationScheduleId,
            hazard_library_id: HazardLibraryId,
            hazard_scenario_id: HazardScenarioId,
            monitor_catalog_id: MonitorCatalogId,
            mitigation_playbook_id: MitigationPlaybookId,
            vendor_certification_adapter_id: VendorCertificationAdapterId,
            vendor_evidence_translation_id: VendorEvidenceTranslationId,
            vendor_trust_root_binding_id: VendorTrustRootBindingId,
            vendor_revocation_handling_id: VendorRevocationHandlingId,
            incident_taxonomy_id: IncidentTaxonomyId,
            severity_matrix_id: SeverityMatrixId,
            pager_route_profile_id: PagerRouteProfileId,
            escalation_clock_policy_id: EscalationClockPolicyId,
        }

        let identity = ProfileCompletionIdentity {
            privacy_retention_profile_id: PrivacyRetentionProfileId::new("privacy-retention-profile-1"),
            redaction_rule_set_id: RedactionRuleSetId::new("redaction-rule-set-1"),
            access_purpose_matrix_id: AccessPurposeMatrixId::new("access-purpose-matrix-1"),
            audit_extraction_policy_id: AuditExtractionPolicyId::new("audit-extraction-policy-1"),
            residency_policy_profile_id: ResidencyPolicyProfileId::new("residency-policy-profile-1"),
            tenant_boundary_profile_id: TenantBoundaryProfileId::new("tenant-boundary-profile-1"),
            cross_boundary_transfer_class_id: CrossBoundaryTransferClassId::new("cross-boundary-transfer-class-1"),
            locality_exception_id: LocalityExceptionId::new("locality-exception-1"),
            role_catalog_id: RoleCatalogId::new("role-catalog-1"),
            delegation_matrix_id: DelegationMatrixId::new("delegation-matrix-1"),
            approval_matrix_id: ApprovalMatrixId::new("approval-matrix-1"),
            conflict_class_catalog_id: ConflictClassCatalogId::new("conflict-class-catalog-1"),
            regulatory_regime_profile_id: RegulatoryRegimeProfileId::new("regulatory-regime-profile-1"),
            requirement_control_map_id: RequirementControlMapId::new("requirement-control-map-1"),
            evidence_collection_plan_id: EvidenceCollectionPlanId::new("evidence-collection-plan-1"),
            recertification_schedule_id: RecertificationScheduleId::new("recertification-schedule-1"),
            hazard_library_id: HazardLibraryId::new("hazard-library-1"),
            hazard_scenario_id: HazardScenarioId::new("hazard-scenario-1"),
            monitor_catalog_id: MonitorCatalogId::new("monitor-catalog-1"),
            mitigation_playbook_id: MitigationPlaybookId::new("mitigation-playbook-1"),
            vendor_certification_adapter_id: VendorCertificationAdapterId::new("vendor-certification-adapter-1"),
            vendor_evidence_translation_id: VendorEvidenceTranslationId::new("vendor-evidence-translation-1"),
            vendor_trust_root_binding_id: VendorTrustRootBindingId::new("vendor-trust-root-binding-1"),
            vendor_revocation_handling_id: VendorRevocationHandlingId::new("vendor-revocation-handling-1"),
            incident_taxonomy_id: IncidentTaxonomyId::new("incident-taxonomy-1"),
            severity_matrix_id: SeverityMatrixId::new("severity-matrix-1"),
            pager_route_profile_id: PagerRouteProfileId::new("pager-route-profile-1"),
            escalation_clock_policy_id: EscalationClockPolicyId::new("escalation-clock-policy-1"),
        };

        let encoded = serde_json::to_string(&identity).unwrap();
        let decoded: ProfileCompletionIdentity = serde_json::from_str(&encoded).unwrap();
        assert_eq!(decoded, identity);
    }


    #[test]
    fn kernel_identity_types_remain_distinct() {
        assert_ne!(
            TypeId::of::<OperatorId>(),
            TypeId::of::<OperatorVersionId>()
        );
        assert_ne!(TypeId::of::<ConstraintId>(), TypeId::of::<HyperedgeId>());
        assert_ne!(TypeId::of::<WitnessId>(), TypeId::of::<CertificateId>());
        assert_ne!(TypeId::of::<RegionId>(), TypeId::of::<RegionDigestId>());
        assert_ne!(
            TypeId::of::<ArtifactTransportId>(),
            TypeId::of::<RepairRouteId>()
        );
        assert_ne!(
            TypeId::of::<RefutationResultId>(),
            TypeId::of::<CalibrationReportId>()
        );
        assert_ne!(
            TypeId::of::<VerificationCaseId>(),
            TypeId::of::<CheckPlanId>()
        );
        assert_ne!(
            TypeId::of::<ControlReceiptId>(),
            TypeId::of::<LedgerEntryId>()
        );
        assert_ne!(
            TypeId::of::<SemanticsProfileId>(),
            TypeId::of::<ClaimStateId>()
        );
        assert_ne!(
            TypeId::of::<SemanticDiffId>(),
            TypeId::of::<CausalAttributionBundleId>()
        );
        assert_ne!(
            TypeId::of::<InterventionId>(),
            TypeId::of::<OutcomeSchemaId>()
        );
        assert_ne!(
            TypeId::of::<ExperimentCaseId>(),
            TypeId::of::<ComparabilityMatrixId>()
        );
        assert_ne!(
            TypeId::of::<DecisionTraceId>(),
            TypeId::of::<RolloutDecisionId>()
        );
        assert_ne!(
            TypeId::of::<AttestationEnvelopeId>(),
            TypeId::of::<TrustRootSetId>()
        );
        assert_ne!(
            TypeId::of::<RemoteOracleLeaseId>(),
            TypeId::of::<RemoteSliceResultId>()
        );
        assert_ne!(
            TypeId::of::<DisputeBundleId>(),
            TypeId::of::<DisclosureBudgetId>()
        );
        assert_ne!(
            TypeId::of::<TreatyBundleId>(),
            TypeId::of::<SettlementCaseId>()
        );
        assert_ne!(
            TypeId::of::<SharedReplaySliceId>(),
            TypeId::of::<SharedDivergenceReportId>()
        );
        assert_ne!(
            TypeId::of::<MechanismBundleId>(),
            TypeId::of::<SimulationContractId>()
        );
        assert_ne!(
            TypeId::of::<DiscoveryProgramId>(),
            TypeId::of::<VerificationLoadBudgetId>()
        );
        assert_ne!(
            TypeId::of::<CharterBundleId>(),
            TypeId::of::<AmendmentDecisionId>()
        );
        assert_ne!(
            TypeId::of::<SpecBundleId>(),
            TypeId::of::<ProofEvaluationReceiptId>()
        );
        assert_ne!(
            TypeId::of::<MetaChallengeBundleId>(),
            TypeId::of::<SelfHostingBuildReceiptId>()
        );
    }

    #[test]
    fn versioned_operator_identity_json_format_is_stable() {
        let identity = VersionedOperatorIdentity {
            operator_id: OperatorId::new("operator-1"),
            operator_version_id: OperatorVersionId::new("operator-version-1"),
        };

        let encoded = serde_json::to_string(&identity).unwrap();
        assert_eq!(
            encoded,
            r#"{"operator_id":"operator-1","operator_version_id":"operator-version-1"}"#
        );

        let decoded: VersionedOperatorIdentity = serde_json::from_str(&encoded).unwrap();
        assert_eq!(decoded, identity);
    }

    #[test]
    fn kernel_artifact_identity_json_format_is_stable() {
        let identity = KernelArtifactIdentity {
            kernel_run_id: KernelRunId::new("run-1"),
            constraint_id: ConstraintId::new("constraint-1"),
            hyperedge_id: HyperedgeId::new("hyperedge-1"),
            residual_id: ResidualId::new("residual-1"),
            syndrome_id: SyndromeId::new("syndrome-1"),
            witness_id: WitnessId::new("witness-1"),
            certificate_id: CertificateId::new("certificate-1"),
            oracle_slice_id: OracleSliceId::new("oracle-slice-1"),
            region_id: RegionId::new("region-1"),
            region_digest_id: RegionDigestId::new("region-digest-1"),
            artifact_transport_id: ArtifactTransportId::new("artifact-transport-1"),
            repair_route_id: RepairRouteId::new("repair-route-1"),
            repair_candidate_id: RepairCandidateId::new("repair-candidate-1"),
            nuisance_state_id: NuisanceStateId::new("nuisance-state-1"),
            convergence_report_id: ConvergenceReportId::new("convergence-report-1"),
            refutation_result_id: RefutationResultId::new("refutation-1"),
            calibration_report_id: CalibrationReportId::new("calibration-1"),
        };

        let encoded = serde_json::to_string(&identity).unwrap();
        assert_eq!(
            encoded,
            r#"{"kernel_run_id":"run-1","constraint_id":"constraint-1","hyperedge_id":"hyperedge-1","residual_id":"residual-1","syndrome_id":"syndrome-1","witness_id":"witness-1","certificate_id":"certificate-1","oracle_slice_id":"oracle-slice-1","region_id":"region-1","region_digest_id":"region-digest-1","artifact_transport_id":"artifact-transport-1","repair_route_id":"repair-route-1","repair_candidate_id":"repair-candidate-1","nuisance_state_id":"nuisance-state-1","convergence_report_id":"convergence-report-1","refutation_result_id":"refutation-1","calibration_report_id":"calibration-1"}"#
        );

        let decoded: KernelArtifactIdentity = serde_json::from_str(&encoded).unwrap();
        assert_eq!(decoded, identity);
    }

    #[test]
    fn control_plane_identity_json_format_is_stable() {
        let identity = ControlPlaneIdentity {
            verification_case_id: VerificationCaseId::new("case-1"),
            check_plan_id: CheckPlanId::new("plan-1"),
            control_receipt_id: ControlReceiptId::new("control-receipt-1"),
            ledger_entry_id: LedgerEntryId::new("ledger-entry-1"),
            policy_decision_id: PolicyDecisionId::new("policy-1"),
            approval_record_id: ApprovalRecordId::new("approval-1"),
            promotion_decision_id: PromotionDecisionId::new("promotion-1"),
            refutation_decision_id: RefutationDecisionId::new("refutation-decision-1"),
            rollback_plan_id: RollbackPlanId::new("rollback-1"),
            calibration_snapshot_id: CalibrationSnapshotId::new("calibration-snapshot-1"),
            learning_update_id: LearningUpdateId::new("learning-update-1"),
            boundary_repair_record_id: BoundaryRepairRecordId::new("boundary-repair-1"),
        };

        let encoded = serde_json::to_string(&identity).unwrap();
        assert_eq!(
            encoded,
            r#"{"verification_case_id":"case-1","check_plan_id":"plan-1","control_receipt_id":"control-receipt-1","ledger_entry_id":"ledger-entry-1","policy_decision_id":"policy-1","approval_record_id":"approval-1","promotion_decision_id":"promotion-1","refutation_decision_id":"refutation-decision-1","rollback_plan_id":"rollback-1","calibration_snapshot_id":"calibration-snapshot-1","learning_update_id":"learning-update-1","boundary_repair_record_id":"boundary-repair-1"}"#
        );

        let decoded: ControlPlaneIdentity = serde_json::from_str(&encoded).unwrap();
        assert_eq!(decoded, identity);
    }

    #[test]
    fn semantic_contract_identity_json_format_is_stable() {
        let identity = SemanticContractIdentity {
            semantics_profile_id: SemanticsProfileId::new("semantics-profile-1"),
            claim_state_id: ClaimStateId::new("claim-state-1"),
            semantic_diff_id: SemanticDiffId::new("semantic-diff-1"),
            causal_attribution_bundle_id: CausalAttributionBundleId::new("causal-attribution-1"),
            degradation_record_id: DegradationRecordId::new("degradation-record-1"),
            exactness_budget_id: ExactnessBudgetId::new("exactness-budget-1"),
            support_set_id: SupportSetId::new("support-set-1"),
            contradiction_witness_id: ContradictionWitnessId::new("contradiction-witness-1"),
            retraction_record_id: RetractionRecordId::new("retraction-record-1"),
        };

        let encoded = serde_json::to_string(&identity).unwrap();
        assert_eq!(
            encoded,
            r#"{"semantics_profile_id":"semantics-profile-1","claim_state_id":"claim-state-1","semantic_diff_id":"semantic-diff-1","causal_attribution_bundle_id":"causal-attribution-1","degradation_record_id":"degradation-record-1","exactness_budget_id":"exactness-budget-1","support_set_id":"support-set-1","contradiction_witness_id":"contradiction-witness-1","retraction_record_id":"retraction-record-1"}"#
        );

        let decoded: SemanticContractIdentity = serde_json::from_str(&encoded).unwrap();
        assert_eq!(decoded, identity);
    }

    #[test]
    fn endgame_identity_json_format_is_stable() {
        let identity = EndgameIdentity {
            intervention_id: InterventionId::new("intervention-1"),
            outcome_schema_id: OutcomeSchemaId::new("outcome-schema-1"),
            experiment_case_id: ExperimentCaseId::new("experiment-case-1"),
            cohort_contract_id: CohortContractId::new("cohort-contract-1"),
            comparability_matrix_id: ComparabilityMatrixId::new("comparability-matrix-1"),
            counterfactual_slice_id: CounterfactualSliceId::new("counterfactual-slice-1"),
            decision_trace_id: DecisionTraceId::new("decision-trace-1"),
            refuter_suite_id: RefuterSuiteId::new("refuter-suite-1"),
            refuter_result_id: RefuterResultId::new("refuter-result-1"),
            rollout_decision_id: RolloutDecisionId::new("rollout-decision-1"),
            rollback_decision_id: RollbackDecisionId::new("rollback-decision-1"),
            experiment_budget_id: ExperimentBudgetId::new("experiment-budget-1"),
            attestation_envelope_id: AttestationEnvelopeId::new("attestation-envelope-1"),
            trust_root_set_id: TrustRootSetId::new("trust-root-set-1"),
            artifact_admission_policy_id: ArtifactAdmissionPolicyId::new(
                "artifact-admission-policy-1",
            ),
            transparency_receipt_id: TransparencyReceiptId::new("transparency-receipt-1"),
            attestation_revocation_id: AttestationRevocationId::new("attestation-revocation-1"),
            attestation_supersession_id: AttestationSupersessionId::new(
                "attestation-supersession-1",
            ),
            remote_oracle_lease_id: RemoteOracleLeaseId::new("remote-oracle-lease-1"),
            remote_slice_request_id: RemoteSliceRequestId::new("remote-slice-request-1"),
            remote_slice_result_id: RemoteSliceResultId::new("remote-slice-result-1"),
            cross_runtime_replay_ticket_id: CrossRuntimeReplayTicketId::new(
                "cross-runtime-replay-ticket-1",
            ),
            dispute_bundle_id: DisputeBundleId::new("dispute-bundle-1"),
            disclosure_policy_id: DisclosurePolicyId::new("disclosure-policy-1"),
            disclosure_budget_id: DisclosureBudgetId::new("disclosure-budget-1"),
        };

        let encoded = serde_json::to_string(&identity).unwrap();
        assert_eq!(
            encoded,
            r#"{"intervention_id":"intervention-1","outcome_schema_id":"outcome-schema-1","experiment_case_id":"experiment-case-1","cohort_contract_id":"cohort-contract-1","comparability_matrix_id":"comparability-matrix-1","counterfactual_slice_id":"counterfactual-slice-1","decision_trace_id":"decision-trace-1","refuter_suite_id":"refuter-suite-1","refuter_result_id":"refuter-result-1","rollout_decision_id":"rollout-decision-1","rollback_decision_id":"rollback-decision-1","experiment_budget_id":"experiment-budget-1","attestation_envelope_id":"attestation-envelope-1","trust_root_set_id":"trust-root-set-1","artifact_admission_policy_id":"artifact-admission-policy-1","transparency_receipt_id":"transparency-receipt-1","attestation_revocation_id":"attestation-revocation-1","attestation_supersession_id":"attestation-supersession-1","remote_oracle_lease_id":"remote-oracle-lease-1","remote_slice_request_id":"remote-slice-request-1","remote_slice_result_id":"remote-slice-result-1","cross_runtime_replay_ticket_id":"cross-runtime-replay-ticket-1","dispute_bundle_id":"dispute-bundle-1","disclosure_policy_id":"disclosure-policy-1","disclosure_budget_id":"disclosure-budget-1"}"#
        );

        let decoded: EndgameIdentity = serde_json::from_str(&encoded).unwrap();
        assert_eq!(decoded, identity);
    }

    #[test]
    fn final_closeout_identity_json_format_is_stable() {
        let identity = FinalCloseoutIdentity {
            effect_intent_id: EffectIntentId::new("effect-intent-1"),
            effect_preflight_report_id: EffectPreflightReportId::new("effect-preflight-report-1"),
            effect_window_id: EffectWindowId::new("effect-window-1"),
            effect_commit_decision_id: EffectCommitDecisionId::new("effect-commit-decision-1"),
            effect_execution_receipt_id: EffectExecutionReceiptId::new(
                "effect-execution-receipt-1",
            ),
            effect_observation_bundle_id: EffectObservationBundleId::new(
                "effect-observation-bundle-1",
            ),
            compensation_plan_id: CompensationPlanId::new("compensation-plan-1"),
            compensation_execution_receipt_id: CompensationExecutionReceiptId::new(
                "compensation-execution-receipt-1",
            ),
            external_effect_ledger_entry_id: ExternalEffectLedgerEntryId::new(
                "external-effect-ledger-entry-1",
            ),
            capability_class_id: CapabilityClassId::new("capability-class-1"),
            authority_lease_id: AuthorityLeaseId::new("authority-lease-1"),
            delegation_bundle_id: DelegationBundleId::new("delegation-bundle-1"),
            authority_chain_id: AuthorityChainId::new("authority-chain-1"),
            separation_of_duties_policy_id: SeparationOfDutiesPolicyId::new(
                "separation-of-duties-policy-1",
            ),
            dual_control_approval_id: DualControlApprovalId::new("dual-control-approval-1"),
            break_glass_grant_id: BreakGlassGrantId::new("break-glass-grant-1"),
            delegation_revocation_id: DelegationRevocationId::new("delegation-revocation-1"),
            acting_on_behalf_receipt_id: ActingOnBehalfReceiptId::new(
                "acting-on-behalf-receipt-1",
            ),
            conflict_disclosure_id: ConflictDisclosureId::new("conflict-disclosure-1"),
            deployment_profile_id: DeploymentProfileId::new("deployment-profile-1"),
            operating_envelope_id: OperatingEnvelopeId::new("operating-envelope-1"),
            assurance_case_id: AssuranceCaseId::new("assurance-case-1"),
            hazard_register_id: HazardRegisterId::new("hazard-register-1"),
            control_mapping_id: ControlMappingId::new("control-mapping-1"),
            residual_risk_acceptance_id: ResidualRiskAcceptanceId::new(
                "residual-risk-acceptance-1",
            ),
            release_readiness_decision_id: ReleaseReadinessDecisionId::new(
                "release-readiness-decision-1",
            ),
            field_monitoring_plan_id: FieldMonitoringPlanId::new("field-monitoring-plan-1"),
            certification_bundle_id: CertificationBundleId::new("certification-bundle-1"),
            recertification_trigger_id: RecertificationTriggerId::new(
                "recertification-trigger-1",
            ),
            service_level_profile_id: ServiceLevelProfileId::new("service-level-profile-1"),
            error_budget_ledger_id: ErrorBudgetLedgerId::new("error-budget-ledger-1"),
            incident_case_id: IncidentCaseId::new("incident-case-1"),
            containment_decision_id: ContainmentDecisionId::new("containment-decision-1"),
            forensic_freeze_id: ForensicFreezeId::new("forensic-freeze-1"),
            recovery_plan_id: RecoveryPlanId::new("recovery-plan-1"),
            recovery_replay_slice_id: RecoveryReplaySliceId::new("recovery-replay-slice-1"),
            continuity_exception_id: ContinuityExceptionId::new("continuity-exception-1"),
            postmortem_bundle_id: PostmortemBundleId::new("postmortem-bundle-1"),
            resilience_exercise_id: ResilienceExerciseId::new("resilience-exercise-1"),
            effect_review_case_id: EffectReviewCaseId::new("effect-review-case-1"),
            effect_block_receipt_id: EffectBlockReceiptId::new("effect-block-receipt-1"),
            delegation_review_case_id: DelegationReviewCaseId::new("delegation-review-case-1"),
            release_gate_case_id: ReleaseGateCaseId::new("release-gate-case-1"),
            continuity_review_case_id: ContinuityReviewCaseId::new("continuity-review-case-1"),
            effect_policy_profile_id: EffectPolicyProfileId::new("effect-policy-profile-1"),
            delegation_policy_profile_id: DelegationPolicyProfileId::new(
                "delegation-policy-profile-1",
            ),
            release_policy_profile_id: ReleasePolicyProfileId::new("release-policy-profile-1"),
            continuity_policy_profile_id: ContinuityPolicyProfileId::new(
                "continuity-policy-profile-1",
            ),
            effect_adjudication_receipt_id: EffectAdjudicationReceiptId::new(
                "effect-adjudication-receipt-1",
            ),
            release_rollback_decision_id: ReleaseRollbackDecisionId::new(
                "release-rollback-decision-1",
            ),
            tool_effect_dispatch_receipt_id: ToolEffectDispatchReceiptId::new(
                "tool-effect-dispatch-receipt-1",
            ),
        };

        let encoded = serde_json::to_string(&identity).unwrap();
        let decoded: FinalCloseoutIdentity = serde_json::from_str(&encoded).unwrap();
        assert_eq!(decoded, identity);
    }

    #[test]
    fn horizon_identity_json_format_is_stable() {
        let identity = HorizonIdentity {
            treaty_bundle_id: TreatyBundleId::new("treaty-bundle-1"),
            runtime_identity_set_id: RuntimeIdentitySetId::new("runtime-identity-set-1"),
            cross_runtime_equivalence_bundle_id: CrossRuntimeEquivalenceBundleId::new(
                "cross-runtime-equivalence-bundle-1",
            ),
            settlement_case_id: SettlementCaseId::new("settlement-case-1"),
            shared_disposition_id: SharedDispositionId::new("shared-disposition-1"),
            local_dissent_id: LocalDissentId::new("local-dissent-1"),
            shared_view_downgrade_id: SharedViewDowngradeId::new("shared-view-downgrade-1"),
            settlement_receipt_id: SettlementReceiptId::new("settlement-receipt-1"),
            shared_replay_slice_id: SharedReplaySliceId::new("shared-replay-slice-1"),
            shared_divergence_report_id: SharedDivergenceReportId::new(
                "shared-divergence-report-1",
            ),
            treaty_suspension_id: TreatySuspensionId::new("treaty-suspension-1"),
            mechanism_bundle_id: MechanismBundleId::new("mechanism-bundle-1"),
            theory_version_id: TheoryVersionId::new("theory-version-1"),
            theory_library_id: TheoryLibraryId::new("theory-library-1"),
            hypothesis_library_id: HypothesisLibraryId::new("hypothesis-library-1"),
            simulation_contract_id: SimulationContractId::new("simulation-contract-1"),
            fit_run_id: FitRunId::new("fit-run-1"),
            theory_refuter_suite_id: TheoryRefuterSuiteId::new("theory-refuter-suite-1"),
            rollout_stability_report_id: RolloutStabilityReportId::new(
                "rollout-stability-report-1",
            ),
            discovery_program_id: DiscoveryProgramId::new("discovery-program-1"),
            portfolio_plan_id: PortfolioPlanId::new("portfolio-plan-1"),
            experiment_campaign_id: ExperimentCampaignId::new("experiment-campaign-1"),
            campaign_decision_trace_id: CampaignDecisionTraceId::new("campaign-decision-trace-1"),
            information_value_estimate_id: InformationValueEstimateId::new(
                "information-value-estimate-1",
            ),
            verification_load_budget_id: VerificationLoadBudgetId::new(
                "verification-load-budget-1",
            ),
            charter_bundle_id: CharterBundleId::new("charter-bundle-1"),
            doctrine_snapshot_id: DoctrineSnapshotId::new("doctrine-snapshot-1"),
            amendment_proposal_id: AmendmentProposalId::new("amendment-proposal-1"),
            amendment_decision_id: AmendmentDecisionId::new("amendment-decision-1"),
            archive_manifest_id: ArchiveManifestId::new("archive-manifest-1"),
            compaction_receipt_id: CompactionReceiptId::new("compaction-receipt-1"),
            historical_query_guarantee_id: HistoricalQueryGuaranteeId::new(
                "historical-query-guarantee-1",
            ),
            deprecation_bundle_id: DeprecationBundleId::new("deprecation-bundle-1"),
            retirement_bundle_id: RetirementBundleId::new("retirement-bundle-1"),
            spec_bundle_id: SpecBundleId::new("spec-bundle-1"),
            normative_ast_id: NormativeAstId::new("normative-ast-1"),
            generated_schema_bundle_id: GeneratedSchemaBundleId::new("generated-schema-bundle-1"),
            generated_interpreter_bundle_id: GeneratedInterpreterBundleId::new(
                "generated-interpreter-bundle-1",
            ),
            generated_conformance_corpus_id: GeneratedConformanceCorpusId::new(
                "generated-conformance-corpus-1",
            ),
            generated_migration_plan_id: GeneratedMigrationPlanId::new(
                "generated-migration-plan-1",
            ),
            proof_obligation_set_id: ProofObligationSetId::new("proof-obligation-set-1"),
            proof_evaluation_receipt_id: ProofEvaluationReceiptId::new(
                "proof-evaluation-receipt-1",
            ),
            human_veto_bundle_id: HumanVetoBundleId::new("human-veto-bundle-1"),
            meta_challenge_bundle_id: MetaChallengeBundleId::new("meta-challenge-bundle-1"),
            self_hosting_build_receipt_id: SelfHostingBuildReceiptId::new(
                "self-hosting-build-receipt-1",
            ),
        };

        let encoded = serde_json::to_string(&identity).unwrap();
        let decoded: HorizonIdentity = serde_json::from_str(&encoded).unwrap();
        assert_eq!(decoded, identity);
    }
}
