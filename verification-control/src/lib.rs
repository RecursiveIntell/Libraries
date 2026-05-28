//! Verification control-plane artifact types and scheduling logic.
//!
//! This crate defines the canonical schema types for verification cases,
//! check plans, attempts, control receipts, ledger entries, and governance
//! review artifacts (effect, delegation, release-gate, continuity). It also
//! provides scheduling, promotion-eligibility, and ledger-replay functions
//! consumed by `forge-pilot` and the verification pipeline crates.
#![cfg_attr(test, allow(clippy::expect_used))]

use llm_tool_runtime::{ToolExposureDecision, ToolReceipt, ToolRetryOwner};
use schemars::JsonSchema;
use semantic_memory_forge::{
    DegradationKindV1, EvidenceAdmissibilityV1, ExactnessBudgetV1, ExactnessEscalationRuleV1,
    ExactnessLevelV1, ForgeToolReceiptV2,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use stack_ids::{
    ApplicabilityContextId, ArtifactTransportId, AssuranceCaseId, AttemptId, AuthorityChainId,
    BoundaryRepairRecordId, CheckPlanId, ClaimVersionId, CompiledObligationSetId,
    CompositionConflictSetId, CompositionReceiptId, ContentDigest, ContinuityExceptionId,
    ContinuityReviewCaseId, ControlReceiptId, DelegationReviewCaseId, DeploymentProfileId,
    EffectBlockReceiptId, EffectCommitDecisionId, EffectIntentId, EffectPreflightReportId,
    EffectReviewCaseId, EffectiveConstitutionId, IncidentCaseId, LedgerEntryId,
    ProfileExceptionBundleId, ProfileSetId, RecoveryReplaySliceId, RegionDigestId, RegionId,
    ReleaseGateCaseId, ReleaseReadinessDecisionId, RepairCandidateId, RepairRouteId, ScopeKey,
    SeparationOfDutiesPolicyId, SyndromeId, TraceCtx, TrialId, VerificationCaseId,
};

pub mod v14;
pub use v14::{
    ComparabilityMatrixV1, DecisionTraceV1, DisputeBundleV1, ExperimentCaseV1, RefuterResultV1,
    COMPARABILITY_MATRIX_V1_SCHEMA, DECISION_TRACE_V1_SCHEMA, DISPUTE_BUNDLE_V1_SCHEMA,
    EXPERIMENT_CASE_V1_SCHEMA, REFUTER_RESULT_V1_SCHEMA,
};

pub const VERIFICATION_CASE_V1_SCHEMA: &str = "verification_case_v1";
pub const CHECK_PLAN_V1_SCHEMA: &str = "check_plan_v1";
pub const DISPATCH_DECISION_V1_SCHEMA: &str = "dispatch_decision_v1";
pub const TOOL_EXPOSURE_V1_SCHEMA: &str = "tool_exposure_v1";
pub const RETRY_DECISION_V1_SCHEMA: &str = "retry_decision_v1";
pub const BUDGET_DEBIT_V1_SCHEMA: &str = "budget_debit_v1";
pub const ROLLBACK_DECISION_V1_SCHEMA: &str = "rollback_decision_v1";
pub const VERIFICATION_ATTEMPT_V1_SCHEMA: &str = "verification_attempt_v1";
pub const CONTROL_RECEIPT_V1_SCHEMA: &str = "control_receipt_v1";
pub const LEDGER_ENTRY_V1_SCHEMA: &str = "ledger_entry_v1";
pub const SCHEDULER_DECISION_V1_SCHEMA: &str = "scheduler_decision_v1";
pub const BOUNDARY_REPAIR_RECORD_V1_SCHEMA: &str = "boundary_repair_record_v1";
pub const REGION_ARTIFACT_TRANSPORT_V1_SCHEMA: &str = "region_artifact_transport_v1";
pub const SYNDROME_ROUTE_RECORD_V1_SCHEMA: &str = "syndrome_route_record_v1";
pub const EFFECT_REVIEW_CASE_V1_SCHEMA: &str = "effect_review_case_v1";
pub const EFFECT_BLOCK_RECEIPT_V1_SCHEMA: &str = "effect_block_receipt_v1";
pub const DELEGATION_REVIEW_CASE_V1_SCHEMA: &str = "delegation_review_case_v1";
pub const RELEASE_GATE_CASE_V1_SCHEMA: &str = "release_gate_case_v1";
pub const CONTINUITY_REVIEW_CASE_V1_SCHEMA: &str = "continuity_review_case_v1";
pub const CONTROL_CASE_V1_SCHEMA: &str = VERIFICATION_CASE_V1_SCHEMA;
pub const EXECUTION_PLAN_V1_SCHEMA: &str = CHECK_PLAN_V1_SCHEMA;
pub const REPAIR_RECORD_V1_SCHEMA: &str = BOUNDARY_REPAIR_RECORD_V1_SCHEMA;
pub const ROLLBACK_RECORD_V1_SCHEMA: &str = ROLLBACK_DECISION_V1_SCHEMA;

pub use BoundaryRepairRecord as RepairRecord;
pub use CheckPlan as ExecutionPlan;
pub use VerificationCase as ControlCase;

/// Constitutional citation context attached to governance review artifacts.
///
/// Tracks which applicability, profile, composition, and obligation artifacts
/// were consulted when producing a review decision.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct V25CitationContext {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub applicability_context_id: Option<ApplicabilityContextId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub profile_set_id: Option<ProfileSetId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub composition_receipt_id: Option<CompositionReceiptId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub effective_constitution_id: Option<EffectiveConstitutionId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub compiled_obligation_set_id: Option<CompiledObligationSetId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub composition_conflict_set_id: Option<CompositionConflictSetId>,
    #[serde(default)]
    pub profile_exception_bundle_ids: Vec<ProfileExceptionBundleId>,
}

/// Completeness status of the constitutional context payload on a review artifact.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ConstitutionalContextStatus {
    Missing,
    Partial,
    Complete,
}

impl ConstitutionalContextStatus {
    /// Returns true only when the full constitutional context is present.
    pub fn is_complete(self) -> bool {
        matches!(self, Self::Complete)
    }
}

impl V25CitationContext {
    /// Constructs an explicit missing constitutional citation payload.
    pub fn missing() -> Self {
        Self {
            applicability_context_id: None,
            profile_set_id: None,
            composition_receipt_id: None,
            effective_constitution_id: None,
            compiled_obligation_set_id: None,
            composition_conflict_set_id: None,
            profile_exception_bundle_ids: Vec::new(),
        }
    }

    /// Classifies how much of the required constitutional citation payload is present.
    pub fn context_status(&self) -> ConstitutionalContextStatus {
        let required_present = [
            self.applicability_context_id.is_some(),
            self.profile_set_id.is_some(),
            self.composition_receipt_id.is_some(),
            self.effective_constitution_id.is_some(),
            self.compiled_obligation_set_id.is_some(),
        ];
        let present = required_present
            .into_iter()
            .filter(|present| *present)
            .count();
        if present == 0 {
            ConstitutionalContextStatus::Missing
        } else if present == required_present.len() {
            ConstitutionalContextStatus::Complete
        } else {
            ConstitutionalContextStatus::Partial
        }
    }
}

/// Obligation references attached to a governance review artifact.
///
/// Captures required, blocking, and monitoring obligation refs from the
/// compiled obligation set that applied at review time.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct V25ControlObligationRefs {
    #[serde(default)]
    pub required_obligation_refs: Vec<String>,
    #[serde(default)]
    pub blocking_obligation_refs: Vec<String>,
    #[serde(default)]
    pub monitoring_obligation_refs: Vec<String>,
}

impl V25ControlObligationRefs {
    /// Constructs an explicit missing obligation-reference payload.
    pub fn missing() -> Self {
        Self {
            required_obligation_refs: Vec::new(),
            blocking_obligation_refs: Vec::new(),
            monitoring_obligation_refs: Vec::new(),
        }
    }

    /// Classifies whether obligation references are absent or present on the artifact.
    pub fn context_status(&self) -> ConstitutionalContextStatus {
        if self.required_obligation_refs.is_empty()
            && self.blocking_obligation_refs.is_empty()
            && self.monitoring_obligation_refs.is_empty()
        {
            ConstitutionalContextStatus::Missing
        } else {
            ConstitutionalContextStatus::Complete
        }
    }
}

/// A review case for an effect-system action (intent, preflight, commit decision).
///
/// Gates side-effecting actions through constitutional citation and obligation checks.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct EffectReviewCaseV1 {
    pub schema_version: String,
    pub effect_review_case_id: EffectReviewCaseId,
    pub effect_intent_id: EffectIntentId,
    pub effect_preflight_report_id: EffectPreflightReportId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub effect_commit_decision_id: Option<EffectCommitDecisionId>,
    #[serde(flatten)]
    pub citation: V25CitationContext,
    #[serde(flatten)]
    pub obligation_refs: V25ControlObligationRefs,
    pub citation_status: ConstitutionalContextStatus,
    pub obligation_refs_status: ConstitutionalContextStatus,
    #[serde(default)]
    pub required_policy_refs: Vec<String>,
    pub decision_basis: String,
    pub final_state: EffectReviewFinalStateV1,
    pub advisory_only: bool,
    pub generated_at: String,
}

/// Terminal state of an effect review case.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum EffectReviewFinalStateV1 {
    Approved,
    Authorized,
    AdvisoryOnly,
    Denied,
    Blocked,
}

/// Receipt emitted when an effect action is blocked by policy or budget constraints.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct EffectBlockReceiptV1 {
    pub schema_version: String,
    pub effect_block_receipt_id: EffectBlockReceiptId,
    pub effect_review_case_id: EffectReviewCaseId,
    #[serde(flatten)]
    pub citation: V25CitationContext,
    #[serde(flatten)]
    pub obligation_refs: V25ControlObligationRefs,
    pub citation_status: ConstitutionalContextStatus,
    pub obligation_refs_status: ConstitutionalContextStatus,
    pub block_reason: EffectBlockReasonV1,
    pub generated_at: String,
}

/// Reason an effect action was blocked.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum EffectBlockReasonV1 {
    PolicyRequiredObligationsOutstanding,
    #[serde(alias = "budget exhausted")]
    BudgetExhausted,
}

/// A review case for authority delegation decisions.
///
/// Validates that delegation chains and separation-of-duties policies
/// are satisfied before approving delegated authority.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct DelegationReviewCaseV1 {
    pub schema_version: String,
    pub delegation_review_case_id: DelegationReviewCaseId,
    pub authority_chain_id: AuthorityChainId,
    pub separation_of_duties_policy_id: SeparationOfDutiesPolicyId,
    #[serde(flatten)]
    pub citation: V25CitationContext,
    #[serde(flatten)]
    pub obligation_refs: V25ControlObligationRefs,
    pub citation_status: ConstitutionalContextStatus,
    pub obligation_refs_status: ConstitutionalContextStatus,
    pub decision_state: DelegationReviewDecisionStateV1,
    pub generated_at: String,
}

/// Terminal decision state of a delegation review case.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum DelegationReviewDecisionStateV1 {
    Approved,
    Valid,
    Denied,
    Blocked,
}

/// A review case for release-gate decisions.
///
/// Evaluates deployment profile, assurance case, and readiness decision
/// against constitutional context before approving a release.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ReleaseGateCaseV1 {
    pub schema_version: String,
    pub release_gate_case_id: ReleaseGateCaseId,
    pub deployment_profile_id: DeploymentProfileId,
    pub assurance_case_id: AssuranceCaseId,
    pub release_readiness_decision_id: ReleaseReadinessDecisionId,
    #[serde(flatten)]
    pub citation: V25CitationContext,
    #[serde(flatten)]
    pub obligation_refs: V25ControlObligationRefs,
    pub citation_status: ConstitutionalContextStatus,
    pub obligation_refs_status: ConstitutionalContextStatus,
    pub score_only_gate: bool,
    pub final_state: ReleaseGateFinalStateV1,
}

/// Terminal state of a release-gate review case.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ReleaseGateFinalStateV1 {
    Approved,
    #[serde(alias = "approved_with_monitors")]
    ApprovedWithMonitoring,
    Blocked,
}

/// A review case for continuity incidents (outage, recovery, exception handling).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ContinuityReviewCaseV1 {
    pub schema_version: String,
    pub continuity_review_case_id: ContinuityReviewCaseId,
    pub incident_case_id: IncidentCaseId,
    #[serde(flatten)]
    pub citation: V25CitationContext,
    #[serde(flatten)]
    pub obligation_refs: V25ControlObligationRefs,
    pub citation_status: ConstitutionalContextStatus,
    pub obligation_refs_status: ConstitutionalContextStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub continuity_exception_id: Option<ContinuityExceptionId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recovery_replay_slice_id: Option<RecoveryReplaySliceId>,
    pub post_hoc_review_due: bool,
    pub final_state: ContinuityReviewFinalStateV1,
}

/// Terminal state of a continuity review case.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ContinuityReviewFinalStateV1 {
    OpenReview,
    Closed,
}

/// Classification of the issue that opened a verification case.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum VerificationCaseClass {
    ActiveSyndrome,
    FragileNode,
    ThinExport,
    UnverifiedClaimVersion,
    RefutationGap,
    SupersessionVerification,
    ComparabilityDrift,
    CalibrationCaveat,
    ScopeStale,
    QueryTurn,
}

/// Scope and location metadata for a verification case target.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct CaseRegion {
    pub namespace: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scope_key: Option<ScopeKey>,
    pub target_key: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub region_id: Option<RegionId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub region_digest_id: Option<RegionDigestId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub claim_version_id: Option<ClaimVersionId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub as_of_recorded_at: Option<String>,
}

/// Lifecycle state of a verification case from open to closed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum VerificationCaseLifecycleState {
    Open,
    Planned,
    Executing,
    Closed,
}

/// Terminal outcome of a closed verification case.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum TerminalDisposition {
    EligibleForPromotion,
    AdvisoryOnly,
    DegradedNoPromotion,
    Refuted,
    Quarantined,
    RolledBack,
    Exhausted,
    Blocked,
}

/// Priority tier for promotion eligibility (P0 = highest).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum PromotionClass {
    P0,
    P1,
    P2,
    P3,
}

/// How difficult it is to reverse a promoted verification result.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ReversibilityClass {
    ReversibleLocal,
    ReversibleScoped,
    RequiresSupersession,
    RequiresAuthorityWorkflow,
}

/// A single verification case tracking a governance issue from open to closed.
///
/// Cases are opened by the observation phase, assigned check plans, executed
/// through verification attempts, and closed with a terminal disposition.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct VerificationCase {
    pub schema_version: String,
    pub case_id: VerificationCaseId,
    pub class: VerificationCaseClass,
    pub region: CaseRegion,
    pub trace_ctx: TraceCtx,
    pub attempt_id: AttemptId,
    pub lifecycle_state: VerificationCaseLifecycleState,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub terminal_disposition: Option<TerminalDisposition>,
    pub non_authoritative: bool,
    pub advisory_only: bool,
    pub degraded: bool,
    pub opened_at: String,
}

impl VerificationCase {
    /// Creates an open verification case with the supplied scope, trace, and degradation flags.
    pub fn new(
        class: VerificationCaseClass,
        region: CaseRegion,
        trace_ctx: TraceCtx,
        attempt_id: AttemptId,
        opened_at: impl Into<String>,
        degraded: bool,
        advisory_only: bool,
    ) -> Self {
        Self {
            schema_version: VERIFICATION_CASE_V1_SCHEMA.into(),
            case_id: VerificationCaseId::generate(),
            class,
            region,
            trace_ctx,
            attempt_id,
            lifecycle_state: VerificationCaseLifecycleState::Open,
            terminal_disposition: None,
            non_authoritative: true,
            advisory_only,
            degraded,
            opened_at: opened_at.into(),
        }
    }

    /// Closes the verification case with its terminal disposition.
    pub fn close(mut self, disposition: TerminalDisposition) -> Self {
        self.lifecycle_state = VerificationCaseLifecycleState::Closed;
        self.terminal_disposition = Some(disposition);
        self
    }
}

/// Verification method used by a check plan to evaluate a case.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum CheckMethod {
    ExactBoundedOracle,
    ConservativeOracle,
    DeltaParityOracle,
    TemporalReplayOracle,
    CausalRefuter,
    MinimalPerturbationOracle,
    PairedPatch,
    AdvisoryOnly,
}

/// Scheduler workload class used for budgeting and queue routing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum VerificationWorkloadClass {
    Orchestration,
    Oracle,
    Patch,
    Advisory,
}

/// A single queue-to-queue hop in the scheduler routing chain.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct QueueHop {
    pub hop_index: u32,
    pub from_queue: String,
    pub to_queue: String,
    pub enqueued_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dequeued_at: Option<String>,
}

/// Budget and retry lineage for a scheduled verification plan.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct BudgetLineage {
    pub budget_family: String,
    pub retry_family: AttemptId,
    pub queue_hop_count: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_time_budget_ms: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub remaining_time_budget_ms: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_cost_budget_units: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub remaining_cost_budget_units: Option<u64>,
    pub exhausted: bool,
}

/// Kind of lightweight preflight check in the cheap-check ladder.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum CheapCheckKindV1 {
    SchemaValidation,
    ProvenanceAudit,
    ReplayPreflight,
    OraclePreflight,
    RefutationPreflight,
}

/// Status of a single cheap-check preflight evaluation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct CheapCheckStatusV1 {
    pub check: CheapCheckKindV1,
    pub satisfied: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

/// Proof profile describing evidence admissibility, cheap-check status, and remaining obligations.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ProofProfileV1 {
    pub profile_name: String,
    pub evidence_admissibility: EvidenceAdmissibilityV1,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub cheap_checks: Vec<CheapCheckStatusV1>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub proof_obligations_remaining: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub admissible_evidence: Vec<String>,
}

/// A marker indicating degraded verification quality and whether it blocks promotion.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct DegradationMarker {
    pub kind: String,
    pub reason: String,
    pub blocks_promotion: bool,
}

/// Scheduler decision record preserving budget, degradation, and routing lineage for a plan.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct SchedulerDecision {
    pub schema_version: String,
    pub case_id: VerificationCaseId,
    pub plan_id: CheckPlanId,
    pub workload_class: VerificationWorkloadClass,
    pub cheapest_adequate: bool,
    pub queue_hops: Vec<QueueHop>,
    pub budget_lineage: BudgetLineage,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exactness_budget: Option<ExactnessBudgetV1>,
    #[serde(default)]
    pub degradation_markers: Vec<DegradationMarker>,
    pub promotion_blocked: bool,
}

/// How a turn ultimately dispatched after planning completed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum DispatchPathKind {
    ProviderNative,
    RuntimeNative,
    ParserFallback,
    NoTools,
    PolicyBlocked,
}

/// Terminal reason why a turn stopped.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionStopReason {
    Success,
    PolicyBlocked,
    RoundTripBudgetExhausted,
    BudgetExhausted,
    RetryExhausted,
    EscalationRequested,
    Abstained,
}

/// Per-turn dispatch truth: which backend ran and why.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct DispatchDecision {
    pub schema_version: String,
    pub case_id: VerificationCaseId,
    pub plan_id: CheckPlanId,
    pub trace_ctx: TraceCtx,
    pub provider_kind: String,
    pub backend_kind: String,
    pub execution_mode_label: String,
    pub dispatch_path: DispatchPathKind,
    pub query_mode: String,
    pub dispatch_reason: String,
    pub stop_reason: ExecutionStopReason,
}

impl DispatchDecision {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        case_id: VerificationCaseId,
        plan_id: CheckPlanId,
        trace_ctx: TraceCtx,
        provider_kind: impl Into<String>,
        backend_kind: impl Into<String>,
        execution_mode_label: impl Into<String>,
        dispatch_path: DispatchPathKind,
        query_mode: impl Into<String>,
        dispatch_reason: impl Into<String>,
        stop_reason: ExecutionStopReason,
    ) -> Self {
        Self {
            schema_version: DISPATCH_DECISION_V1_SCHEMA.into(),
            case_id,
            plan_id,
            trace_ctx,
            provider_kind: provider_kind.into(),
            backend_kind: backend_kind.into(),
            execution_mode_label: execution_mode_label.into(),
            dispatch_path,
            query_mode: query_mode.into(),
            dispatch_reason: dispatch_reason.into(),
            stop_reason,
        }
    }
}

/// Explicit per-turn tool exposure, including the empty set.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ToolExposureDecisionV1 {
    pub tool_name: String,
    pub exposed: bool,
    pub reason: String,
}

impl From<ToolExposureDecision> for ToolExposureDecisionV1 {
    fn from(value: ToolExposureDecision) -> Self {
        Self {
            tool_name: value.tool_name,
            exposed: value.exposed,
            reason: value.reason,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ToolExposure {
    pub schema_version: String,
    pub case_id: VerificationCaseId,
    pub plan_id: CheckPlanId,
    pub trace_ctx: TraceCtx,
    pub planner_stage: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub exposed_tools: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub decisions: Vec<ToolExposureDecisionV1>,
    pub explicit_empty: bool,
}

impl ToolExposure {
    pub fn new(
        case_id: VerificationCaseId,
        plan_id: CheckPlanId,
        trace_ctx: TraceCtx,
        planner_stage: impl Into<String>,
        decisions: Vec<ToolExposureDecision>,
    ) -> Self {
        let exposed_tools = decisions
            .iter()
            .filter(|decision| decision.exposed)
            .map(|decision| decision.tool_name.clone())
            .collect::<Vec<_>>();
        let explicit_empty = exposed_tools.is_empty();
        let decisions = decisions
            .into_iter()
            .map(ToolExposureDecisionV1::from)
            .collect();
        Self {
            schema_version: TOOL_EXPOSURE_V1_SCHEMA.into(),
            case_id,
            plan_id,
            trace_ctx,
            planner_stage: planner_stage.into(),
            exposed_tools,
            decisions,
            explicit_empty,
        }
    }
}

/// Queryable retry lineage for a turn or tool family.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct RetryDecision {
    pub schema_version: String,
    pub case_id: VerificationCaseId,
    pub plan_id: CheckPlanId,
    pub retry_family: String,
    pub attempt_number: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub backoff_ms: Option<u64>,
    pub will_retry: bool,
    pub stop_reason: ExecutionStopReason,
    pub owner: ControlRetryOwnerV1,
    pub reason: String,
}

impl RetryDecision {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        case_id: VerificationCaseId,
        plan_id: CheckPlanId,
        retry_family: impl Into<String>,
        attempt_number: u32,
        backoff_ms: Option<u64>,
        will_retry: bool,
        stop_reason: ExecutionStopReason,
        owner: ControlRetryOwnerV1,
        reason: impl Into<String>,
    ) -> Self {
        Self {
            schema_version: RETRY_DECISION_V1_SCHEMA.into(),
            case_id,
            plan_id,
            retry_family: retry_family.into(),
            attempt_number,
            backoff_ms,
            will_retry,
            stop_reason,
            owner,
            reason: reason.into(),
        }
    }
}

/// Budget lineage for one debit against a turn's execution envelope.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct BudgetDebit {
    pub schema_version: String,
    pub case_id: VerificationCaseId,
    pub plan_id: CheckPlanId,
    pub debit_kind: String,
    pub unit: String,
    pub amount: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub remaining: Option<u64>,
    pub exhausted: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

impl BudgetDebit {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        case_id: VerificationCaseId,
        plan_id: CheckPlanId,
        debit_kind: impl Into<String>,
        unit: impl Into<String>,
        amount: u64,
        remaining: Option<u64>,
        exhausted: bool,
        detail: Option<String>,
    ) -> Self {
        Self {
            schema_version: BUDGET_DEBIT_V1_SCHEMA.into(),
            case_id,
            plan_id,
            debit_kind: debit_kind.into(),
            unit: unit.into(),
            amount,
            remaining,
            exhausted,
            detail,
        }
    }
}

/// Canonical rollback/compensation requirement for a turn.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct RollbackDecision {
    pub schema_version: String,
    pub case_id: VerificationCaseId,
    pub plan_id: CheckPlanId,
    pub required: bool,
    pub reason: String,
    pub source: String,
}

impl RollbackDecision {
    pub fn new(
        case_id: VerificationCaseId,
        plan_id: CheckPlanId,
        required: bool,
        reason: impl Into<String>,
        source: impl Into<String>,
    ) -> Self {
        Self {
            schema_version: ROLLBACK_DECISION_V1_SCHEMA.into(),
            case_id,
            plan_id,
            required,
            reason: reason.into(),
            source: source.into(),
        }
    }
}

/// A check plan specifying the verification method, promotion class, and proof profile for a case.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct CheckPlan {
    pub schema_version: String,
    pub plan_id: CheckPlanId,
    pub case_id: VerificationCaseId,
    pub method: CheckMethod,
    pub planner_family: String,
    #[serde(default)]
    pub check_names: Vec<String>,
    pub promotion_class: PromotionClass,
    pub reversibility_class: ReversibilityClass,
    pub cheapest_adequate: bool,
    pub promotable_if_completed: bool,
    pub advisory_only: bool,
    pub degraded: bool,
    pub target_exactness: ExactnessLevelV1,
    pub evidence_admissibility: EvidenceAdmissibilityV1,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub proof_obligations_remaining: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub cheap_check_ladder: Vec<CheapCheckStatusV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub proof_profile: Option<ProofProfileV1>,
    pub rationale: String,
    pub input: Value,
}

impl CheckPlan {
    #[allow(clippy::too_many_arguments)]
    /// Creates a check plan with the default planner family and baseline admissibility policy.
    pub fn new(
        case_id: VerificationCaseId,
        method: CheckMethod,
        check_names: Vec<String>,
        promotion_class: PromotionClass,
        reversibility_class: ReversibilityClass,
        promotable_if_completed: bool,
        advisory_only: bool,
        degraded: bool,
        rationale: impl Into<String>,
        input: Value,
    ) -> Self {
        Self {
            schema_version: CHECK_PLAN_V1_SCHEMA.into(),
            plan_id: CheckPlanId::generate(),
            case_id,
            method,
            planner_family: "forge-pilot".into(),
            check_names,
            promotion_class,
            reversibility_class,
            cheapest_adequate: true,
            promotable_if_completed,
            advisory_only,
            degraded,
            target_exactness: if degraded {
                ExactnessLevelV1::Conservative
            } else {
                ExactnessLevelV1::Exact
            },
            evidence_admissibility: if degraded {
                EvidenceAdmissibilityV1::Restricted
            } else {
                EvidenceAdmissibilityV1::Admissible
            },
            proof_obligations_remaining: Vec::new(),
            cheap_check_ladder: Vec::new(),
            proof_profile: None,
            rationale: rationale.into(),
            input,
        }
    }
}

/// Execution state of a verification attempt.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum VerificationAttemptState {
    Scheduled,
    Running,
    Succeeded,
    Failed,
    AdvisoryOnly,
    Blocked,
}

/// A single execution attempt within a verification case and plan.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct VerificationAttempt {
    pub schema_version: String,
    pub case_id: VerificationCaseId,
    pub plan_id: CheckPlanId,
    pub attempt_id: AttemptId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trial_id: Option<TrialId>,
    pub state: VerificationAttemptState,
    pub advisory_only: bool,
    pub degraded: bool,
    pub started_at: String,
    pub finished_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub outcome_signature: Option<String>,
}

impl VerificationAttempt {
    #[allow(clippy::too_many_arguments)]
    /// Records a completed verification attempt and its terminal execution metadata.
    pub fn completed(
        case_id: VerificationCaseId,
        plan_id: CheckPlanId,
        attempt_id: AttemptId,
        trial_id: Option<TrialId>,
        state: VerificationAttemptState,
        advisory_only: bool,
        degraded: bool,
        started_at: impl Into<String>,
        finished_at: impl Into<String>,
        outcome_signature: Option<String>,
    ) -> Self {
        Self {
            schema_version: VERIFICATION_ATTEMPT_V1_SCHEMA.into(),
            case_id,
            plan_id,
            attempt_id,
            trial_id,
            state,
            advisory_only,
            degraded,
            started_at: started_at.into(),
            finished_at: finished_at.into(),
            outcome_signature,
        }
    }
}

/// Kind of action recorded in a control receipt.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ControlActionKind {
    ToolExecution,
    OrchestrationLoop,
    VerificationExecution,
}

/// Budget context attached to a control receipt for cost and time tracking.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ReceiptBudgetContext {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub budget_kind: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_steps: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub time_budget_ms: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cost_budget_units: Option<u64>,
}

/// An immutable receipt recording a control-plane action with full provenance.
///
/// Control receipts carry constitutional citation context, budget context,
/// retry ownership, and promotion eligibility. They are the atomic audit
/// unit for the verification ledger.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ControlReceipt {
    pub schema_version: String,
    pub receipt_id: ControlReceiptId,
    pub action_kind: ControlActionKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub case_id: Option<VerificationCaseId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub plan_id: Option<CheckPlanId>,
    pub trace_ctx: TraceCtx,
    pub attempt_id: AttemptId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trial_id: Option<TrialId>,
    pub actor: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub planner_stage: Option<String>,
    pub started_at: String,
    pub finished_at: String,
    #[serde(flatten)]
    pub citation: V25CitationContext,
    #[serde(flatten)]
    pub obligation_refs: V25ControlObligationRefs,
    pub citation_status: ConstitutionalContextStatus,
    pub obligation_refs_status: ConstitutionalContextStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deadline: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workload_class: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub budget_context: Option<ReceiptBudgetContext>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_receipt_id: Option<ControlReceiptId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub family_receipt_id: Option<ControlReceiptId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub replay_parent_receipt_id: Option<ControlReceiptId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub retry_owner: Option<ControlRetryOwnerV1>,
    pub non_authoritative: bool,
    pub advisory_only: bool,
    pub degraded: bool,
    pub promotable: bool,
    pub source_receipt_kind: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_receipt_id: Option<String>,
    pub details: Value,
}

/// Which subsystem owns retry responsibility for a control action.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ControlRetryOwnerV1 {
    LlmPipeline,
    AgentGraph,
    JobQueue,
    AiBatchQueue,
    ForgeOrchestration,
    External,
}

impl From<&ToolRetryOwner> for ControlRetryOwnerV1 {
    fn from(value: &ToolRetryOwner) -> Self {
        match value {
            ToolRetryOwner::LlmPipeline => Self::LlmPipeline,
            ToolRetryOwner::AgentGraph => Self::AgentGraph,
            ToolRetryOwner::JobQueue => Self::JobQueue,
            ToolRetryOwner::AiBatchQueue => Self::AiBatchQueue,
            ToolRetryOwner::ForgeOrchestration => Self::ForgeOrchestration,
            ToolRetryOwner::External => Self::External,
        }
    }
}

/// Kind of artifact subject to boundary repair.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum BoundaryArtifactKind {
    LoopReport,
    LoopIterationReport,
    ControlReceipt,
}

/// Record of a single field-level repair applied to a boundary artifact.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct BoundaryRepairRecord {
    pub schema_version: String,
    pub repair_record_id: BoundaryRepairRecordId,
    pub artifact_kind: BoundaryArtifactKind,
    pub artifact_schema_version: String,
    pub repair_kind: String,
    pub field_path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub original_value: Option<Value>,
    pub repaired_value: Value,
    pub rationale: String,
}

/// Kind of artifact transported between compiled regions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RegionArtifactKind {
    Delta,
    Residual,
    Syndrome,
    Witness,
    ControlReceipt,
    NuisanceState,
    OracleParity,
}

/// Transport record for an artifact crossing between compiled verification regions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct RegionArtifactTransport {
    pub schema_version: String,
    pub transport_id: ArtifactTransportId,
    pub artifact_kind: RegionArtifactKind,
    pub artifact_id: String,
    pub artifact_digest: ContentDigest,
    pub from_region_id: RegionId,
    pub to_region_id: RegionId,
    pub from_surface: String,
    pub to_surface: String,
    pub replay_ref: String,
}

/// Routing record linking a syndrome to a repair candidate and its blast radius.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct SyndromeRouteRecord {
    pub schema_version: String,
    pub route_id: RepairRouteId,
    pub syndrome_id: SyndromeId,
    pub region_id: RegionId,
    pub candidate_id: RepairCandidateId,
    pub repair_surface: String,
    pub blast_radius_node_ids: Vec<String>,
    pub witness_refs: Vec<String>,
    pub routed_before_global_invalidation: bool,
}

fn validate_review_artifact_context(
    schema_version: &str,
    expected_schema: &str,
    citation: &V25CitationContext,
    citation_status: ConstitutionalContextStatus,
    obligation_refs: &V25ControlObligationRefs,
    obligation_refs_status: ConstitutionalContextStatus,
) -> Result<(), String> {
    if schema_version != expected_schema {
        return Err(format!(
            "expected schema_version `{expected_schema}`, found `{schema_version}`"
        ));
    }
    let actual_citation_status = citation.context_status();
    if citation_status != actual_citation_status {
        return Err(format!(
            "citation_status {:?} does not match citation payload {:?}",
            citation_status, actual_citation_status
        ));
    }
    let actual_obligation_status = obligation_refs.context_status();
    if obligation_refs_status != actual_obligation_status {
        return Err(format!(
            "obligation_refs_status {:?} does not match obligation payload {:?}",
            obligation_refs_status, actual_obligation_status
        ));
    }
    if !citation_status.is_complete() {
        return Err("review artifact is missing constitutional citation context".into());
    }
    if !obligation_refs_status.is_complete() {
        return Err("review artifact is missing constitutional obligation refs".into());
    }
    Ok(())
}

impl EffectReviewCaseV1 {
    #[allow(clippy::too_many_arguments)]
    /// Creates an effect review case with computed constitutional context status.
    pub fn new(
        effect_intent_id: EffectIntentId,
        effect_preflight_report_id: EffectPreflightReportId,
        effect_commit_decision_id: Option<EffectCommitDecisionId>,
        citation: V25CitationContext,
        obligation_refs: V25ControlObligationRefs,
        required_policy_refs: Vec<String>,
        decision_basis: impl Into<String>,
        final_state: EffectReviewFinalStateV1,
        advisory_only: bool,
        generated_at: impl Into<String>,
    ) -> Result<Self, String> {
        let review = Self {
            schema_version: EFFECT_REVIEW_CASE_V1_SCHEMA.into(),
            effect_review_case_id: EffectReviewCaseId::generate(),
            effect_intent_id,
            effect_preflight_report_id,
            effect_commit_decision_id,
            citation_status: citation.context_status(),
            obligation_refs_status: obligation_refs.context_status(),
            citation,
            obligation_refs,
            required_policy_refs,
            decision_basis: decision_basis.into(),
            final_state,
            advisory_only,
            generated_at: generated_at.into(),
        };
        review.validate()?;
        Ok(review)
    }

    /// Rejects review artifacts whose context/status/state combinations are impossible.
    pub fn validate(&self) -> Result<(), String> {
        validate_review_artifact_context(
            &self.schema_version,
            EFFECT_REVIEW_CASE_V1_SCHEMA,
            &self.citation,
            self.citation_status,
            &self.obligation_refs,
            self.obligation_refs_status,
        )?;
        if self.required_policy_refs.is_empty() {
            return Err("effect review case requires at least one policy ref".into());
        }
        if self.advisory_only != matches!(self.final_state, EffectReviewFinalStateV1::AdvisoryOnly)
        {
            return Err("advisory_only must match effect review final_state".into());
        }
        Ok(())
    }
}

impl EffectBlockReceiptV1 {
    /// Creates an effect block receipt with computed constitutional context status.
    pub fn new(
        effect_review_case_id: EffectReviewCaseId,
        citation: V25CitationContext,
        obligation_refs: V25ControlObligationRefs,
        block_reason: EffectBlockReasonV1,
        generated_at: impl Into<String>,
    ) -> Result<Self, String> {
        let receipt = Self {
            schema_version: EFFECT_BLOCK_RECEIPT_V1_SCHEMA.into(),
            effect_block_receipt_id: EffectBlockReceiptId::generate(),
            effect_review_case_id,
            citation_status: citation.context_status(),
            obligation_refs_status: obligation_refs.context_status(),
            citation,
            obligation_refs,
            block_reason,
            generated_at: generated_at.into(),
        };
        receipt.validate()?;
        Ok(receipt)
    }

    /// Rejects block receipts that drift from their constitutional context payload.
    pub fn validate(&self) -> Result<(), String> {
        validate_review_artifact_context(
            &self.schema_version,
            EFFECT_BLOCK_RECEIPT_V1_SCHEMA,
            &self.citation,
            self.citation_status,
            &self.obligation_refs,
            self.obligation_refs_status,
        )
    }
}

impl DelegationReviewCaseV1 {
    /// Creates a delegation review case with computed constitutional context status.
    pub fn new(
        authority_chain_id: AuthorityChainId,
        separation_of_duties_policy_id: SeparationOfDutiesPolicyId,
        citation: V25CitationContext,
        obligation_refs: V25ControlObligationRefs,
        decision_state: DelegationReviewDecisionStateV1,
        generated_at: impl Into<String>,
    ) -> Result<Self, String> {
        let review = Self {
            schema_version: DELEGATION_REVIEW_CASE_V1_SCHEMA.into(),
            delegation_review_case_id: DelegationReviewCaseId::generate(),
            authority_chain_id,
            separation_of_duties_policy_id,
            citation_status: citation.context_status(),
            obligation_refs_status: obligation_refs.context_status(),
            citation,
            obligation_refs,
            decision_state,
            generated_at: generated_at.into(),
        };
        review.validate()?;
        Ok(review)
    }

    /// Rejects delegation review cases with inconsistent constitutional context status.
    pub fn validate(&self) -> Result<(), String> {
        validate_review_artifact_context(
            &self.schema_version,
            DELEGATION_REVIEW_CASE_V1_SCHEMA,
            &self.citation,
            self.citation_status,
            &self.obligation_refs,
            self.obligation_refs_status,
        )
    }
}

impl ReleaseGateCaseV1 {
    /// Creates a release-gate review case with computed constitutional context status.
    pub fn new(
        deployment_profile_id: DeploymentProfileId,
        assurance_case_id: AssuranceCaseId,
        release_readiness_decision_id: ReleaseReadinessDecisionId,
        citation: V25CitationContext,
        obligation_refs: V25ControlObligationRefs,
        score_only_gate: bool,
        final_state: ReleaseGateFinalStateV1,
    ) -> Result<Self, String> {
        let review = Self {
            schema_version: RELEASE_GATE_CASE_V1_SCHEMA.into(),
            release_gate_case_id: ReleaseGateCaseId::generate(),
            deployment_profile_id,
            assurance_case_id,
            release_readiness_decision_id,
            citation_status: citation.context_status(),
            obligation_refs_status: obligation_refs.context_status(),
            citation,
            obligation_refs,
            score_only_gate,
            final_state,
        };
        review.validate()?;
        Ok(review)
    }

    /// Rejects release-gate review cases with impossible state/context combinations.
    pub fn validate(&self) -> Result<(), String> {
        validate_review_artifact_context(
            &self.schema_version,
            RELEASE_GATE_CASE_V1_SCHEMA,
            &self.citation,
            self.citation_status,
            &self.obligation_refs,
            self.obligation_refs_status,
        )?;
        if self.score_only_gate && matches!(self.final_state, ReleaseGateFinalStateV1::Approved) {
            return Err("score-only release gates cannot be fully approved".into());
        }
        Ok(())
    }
}

impl ContinuityReviewCaseV1 {
    #[allow(clippy::too_many_arguments)]
    /// Creates a continuity review case with computed constitutional context status.
    pub fn new(
        incident_case_id: IncidentCaseId,
        citation: V25CitationContext,
        obligation_refs: V25ControlObligationRefs,
        continuity_exception_id: Option<ContinuityExceptionId>,
        recovery_replay_slice_id: Option<RecoveryReplaySliceId>,
        post_hoc_review_due: bool,
        final_state: ContinuityReviewFinalStateV1,
    ) -> Result<Self, String> {
        let review = Self {
            schema_version: CONTINUITY_REVIEW_CASE_V1_SCHEMA.into(),
            continuity_review_case_id: ContinuityReviewCaseId::generate(),
            incident_case_id,
            citation_status: citation.context_status(),
            obligation_refs_status: obligation_refs.context_status(),
            citation,
            obligation_refs,
            continuity_exception_id,
            recovery_replay_slice_id,
            post_hoc_review_due,
            final_state,
        };
        review.validate()?;
        Ok(review)
    }

    /// Rejects continuity review cases that claim closure while post-hoc review is still due.
    pub fn validate(&self) -> Result<(), String> {
        validate_review_artifact_context(
            &self.schema_version,
            CONTINUITY_REVIEW_CASE_V1_SCHEMA,
            &self.citation,
            self.citation_status,
            &self.obligation_refs,
            self.obligation_refs_status,
        )?;
        if self.post_hoc_review_due
            && matches!(self.final_state, ContinuityReviewFinalStateV1::Closed)
        {
            return Err(
                "continuity review cannot be closed while post-hoc review is still due".into(),
            );
        }
        Ok(())
    }
}

impl BoundaryRepairRecord {
    /// Creates a boundary-repair record for one repaired artifact field.
    pub fn new(
        artifact_kind: BoundaryArtifactKind,
        artifact_schema_version: impl Into<String>,
        repair_kind: impl Into<String>,
        field_path: impl Into<String>,
        original_value: Option<Value>,
        repaired_value: Value,
        rationale: impl Into<String>,
    ) -> Self {
        Self {
            schema_version: BOUNDARY_REPAIR_RECORD_V1_SCHEMA.into(),
            repair_record_id: BoundaryRepairRecordId::generate(),
            artifact_kind,
            artifact_schema_version: artifact_schema_version.into(),
            repair_kind: repair_kind.into(),
            field_path: field_path.into(),
            original_value,
            repaired_value,
            rationale: rationale.into(),
        }
    }
}

impl RegionArtifactTransport {
    #[allow(clippy::too_many_arguments)]
    /// Creates a transport record for an artifact crossing between compiled regions.
    pub fn new(
        artifact_kind: RegionArtifactKind,
        artifact_id: impl Into<String>,
        artifact_bytes: &[u8],
        from_region_id: RegionId,
        to_region_id: RegionId,
        from_surface: impl Into<String>,
        to_surface: impl Into<String>,
        replay_ref: impl Into<String>,
    ) -> Self {
        Self {
            schema_version: REGION_ARTIFACT_TRANSPORT_V1_SCHEMA.into(),
            transport_id: ArtifactTransportId::generate(),
            artifact_kind,
            artifact_id: artifact_id.into(),
            artifact_digest: ContentDigest::compute(artifact_bytes),
            from_region_id,
            to_region_id,
            from_surface: from_surface.into(),
            to_surface: to_surface.into(),
            replay_ref: replay_ref.into(),
        }
    }
}

impl SyndromeRouteRecord {
    /// Creates the default syndrome route record for a compiled repair path.
    pub fn new(
        syndrome_id: SyndromeId,
        region_id: RegionId,
        blast_radius_node_ids: Vec<String>,
        witness_refs: Vec<String>,
    ) -> Self {
        Self {
            schema_version: SYNDROME_ROUTE_RECORD_V1_SCHEMA.into(),
            route_id: RepairRouteId::generate(),
            syndrome_id,
            region_id,
            candidate_id: RepairCandidateId::generate(),
            repair_surface: "compiled_repair_graph".into(),
            blast_radius_node_ids,
            witness_refs,
            routed_before_global_invalidation: true,
        }
    }
}

impl ControlReceipt {
    /// Creates a verification execution receipt with explicit missing constitutional context status.
    pub fn new_case_execution(
        case: &VerificationCase,
        plan: &CheckPlan,
        attempt: &VerificationAttempt,
        promotable: bool,
        details: Value,
    ) -> Self {
        let citation = V25CitationContext::missing();
        let citation_status = citation.context_status();
        let obligation_refs = V25ControlObligationRefs::missing();
        let obligation_refs_status = obligation_refs.context_status();
        let receipt = Self {
            schema_version: CONTROL_RECEIPT_V1_SCHEMA.into(),
            receipt_id: ControlReceiptId::generate(),
            action_kind: ControlActionKind::VerificationExecution,
            case_id: Some(case.case_id.clone()),
            plan_id: Some(plan.plan_id.clone()),
            trace_ctx: case.trace_ctx.clone(),
            attempt_id: attempt.attempt_id.clone(),
            trial_id: attempt.trial_id.clone(),
            actor: "forge-pilot".into(),
            planner_stage: Some("verification_execution".into()),
            started_at: attempt.started_at.clone(),
            finished_at: attempt.finished_at.clone(),
            citation,
            obligation_refs,
            citation_status,
            obligation_refs_status,
            deadline: None,
            workload_class: Some("orchestration".into()),
            budget_context: None,
            parent_receipt_id: None,
            family_receipt_id: None,
            replay_parent_receipt_id: None,
            retry_owner: Some(ControlRetryOwnerV1::ForgeOrchestration),
            non_authoritative: true,
            advisory_only: attempt.advisory_only,
            degraded: attempt.degraded,
            promotable: promotable
                && citation_status.is_complete()
                && obligation_refs_status.is_complete(),
            source_receipt_kind: "forge_pilot.loop_iteration".into(),
            source_receipt_id: None,
            details,
        };
        debug_assert!(receipt.validate().is_ok());
        receipt
    }

    /// Rejects receipts whose constitutional status payload or promotability claims are incoherent.
    pub fn validate(&self) -> Result<(), String> {
        if self.schema_version != CONTROL_RECEIPT_V1_SCHEMA {
            return Err(format!(
                "expected schema_version `{CONTROL_RECEIPT_V1_SCHEMA}`, found `{}`",
                self.schema_version
            ));
        }
        let actual_citation_status = self.citation.context_status();
        if self.citation_status != actual_citation_status {
            return Err(format!(
                "citation_status {:?} does not match citation payload {:?}",
                self.citation_status, actual_citation_status
            ));
        }
        let actual_obligation_status = self.obligation_refs.context_status();
        if self.obligation_refs_status != actual_obligation_status {
            return Err(format!(
                "obligation_refs_status {:?} does not match obligation payload {:?}",
                self.obligation_refs_status, actual_obligation_status
            ));
        }
        if self.promotable
            && (!self.citation_status.is_complete() || !self.obligation_refs_status.is_complete())
        {
            return Err(
                "promotable control receipt requires complete citation and obligation refs".into(),
            );
        }
        Ok(())
    }
}

impl From<&ToolReceipt> for ControlReceipt {
    fn from(value: &ToolReceipt) -> Self {
        let citation = V25CitationContext::missing();
        let citation_status = citation.context_status();
        let obligation_refs = V25ControlObligationRefs::missing();
        let obligation_refs_status = obligation_refs.context_status();
        Self {
            schema_version: CONTROL_RECEIPT_V1_SCHEMA.into(),
            receipt_id: ControlReceiptId::generate(),
            action_kind: ControlActionKind::ToolExecution,
            case_id: None,
            plan_id: None,
            trace_ctx: value.trace_ctx.clone(),
            attempt_id: value.attempt_id.clone(),
            trial_id: Some(value.trial_id.clone()),
            actor: value.tool_name.clone(),
            planner_stage: Some(format!("{:?}", value.planner_stage)),
            started_at: value.started_at.clone(),
            finished_at: value.finished_at.clone(),
            citation,
            obligation_refs,
            citation_status,
            obligation_refs_status,
            deadline: value.deadline.clone(),
            workload_class: value.workload_class.clone(),
            budget_context: value
                .budget_context
                .as_ref()
                .map(|budget| ReceiptBudgetContext {
                    budget_kind: budget.budget_kind.clone(),
                    max_steps: budget.max_steps,
                    time_budget_ms: budget.time_budget_ms,
                    cost_budget_units: budget.cost_budget_units,
                }),
            parent_receipt_id: value
                .parent_receipt_id
                .as_ref()
                .map(|id| ControlReceiptId::new(id.clone())),
            family_receipt_id: value
                .family_receipt_id
                .as_ref()
                .map(|id| ControlReceiptId::new(id.clone())),
            replay_parent_receipt_id: value
                .replay_parent_receipt_id
                .as_ref()
                .map(|id| ControlReceiptId::new(id.clone())),
            retry_owner: Some(ControlRetryOwnerV1::from(&value.retry_owner)),
            non_authoritative: true,
            advisory_only: false,
            degraded: false,
            promotable: false,
            source_receipt_kind: "llm_tool_runtime.tool_receipt".into(),
            source_receipt_id: Some(value.receipt_id.clone()),
            details: json!({
                "tool_name": value.tool_name,
                "tool_version": value.tool_version,
                "backend_kind": value.backend_kind.as_str(),
                "approval_state": value.approval_state.as_str(),
            }),
        }
    }
}

impl From<&ForgeToolReceiptV2> for ControlReceipt {
    fn from(value: &ForgeToolReceiptV2) -> Self {
        let citation = V25CitationContext::missing();
        let citation_status = citation.context_status();
        let obligation_refs = V25ControlObligationRefs::missing();
        let obligation_refs_status = obligation_refs.context_status();
        Self {
            schema_version: CONTROL_RECEIPT_V1_SCHEMA.into(),
            receipt_id: ControlReceiptId::generate(),
            action_kind: ControlActionKind::ToolExecution,
            case_id: None,
            plan_id: None,
            trace_ctx: value.trace_ctx.clone(),
            attempt_id: value.attempt_id.clone(),
            trial_id: Some(value.trial_id.clone()),
            actor: value.tool_name.clone(),
            planner_stage: Some(value.planner_stage.clone()),
            started_at: value.started_at.clone(),
            finished_at: value.finished_at.clone(),
            citation,
            obligation_refs,
            citation_status,
            obligation_refs_status,
            deadline: value.deadline.clone(),
            workload_class: value.workload_class.clone(),
            budget_context: value
                .budget_context
                .as_ref()
                .map(|budget| ReceiptBudgetContext {
                    budget_kind: budget.budget_kind.clone(),
                    max_steps: budget.max_steps,
                    time_budget_ms: budget.time_budget_ms,
                    cost_budget_units: budget.cost_budget_units,
                }),
            parent_receipt_id: value
                .parent_receipt_id
                .as_ref()
                .map(|id| ControlReceiptId::new(id.clone())),
            family_receipt_id: value
                .family_receipt_id
                .as_ref()
                .map(|id| ControlReceiptId::new(id.clone())),
            replay_parent_receipt_id: value
                .replay_parent_receipt_id
                .as_ref()
                .map(|id| ControlReceiptId::new(id.clone())),
            retry_owner: Some(match value.retry_owner.as_str() {
                "llm_pipeline" => ControlRetryOwnerV1::LlmPipeline,
                "agent_graph" => ControlRetryOwnerV1::AgentGraph,
                "job_queue" => ControlRetryOwnerV1::JobQueue,
                "ai_batch_queue" => ControlRetryOwnerV1::AiBatchQueue,
                "forge_orchestration" => ControlRetryOwnerV1::ForgeOrchestration,
                _ => ControlRetryOwnerV1::External,
            }),
            non_authoritative: true,
            advisory_only: false,
            degraded: false,
            promotable: false,
            source_receipt_kind: "semantic_memory_forge.forge_tool_receipt_v2".into(),
            source_receipt_id: Some(value.receipt_id.clone()),
            details: json!({
                "tool_name": value.tool_name,
                "tool_version": value.tool_version,
                "backend_kind": value.backend_kind,
                "approval_state": value.approval_state,
            }),
        }
    }
}

/// An event in the verification case ledger.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "event_type", rename_all = "snake_case")]
#[allow(clippy::large_enum_variant)]
pub enum LedgerEvent {
    CaseOpened {
        case: VerificationCase,
    },
    PlanAdopted {
        plan: CheckPlan,
    },
    AttemptRecorded {
        attempt: VerificationAttempt,
    },
    ReceiptAppended {
        receipt: Box<ControlReceipt>,
    },
    CaseClosed {
        case_id: VerificationCaseId,
        disposition: TerminalDisposition,
    },
}

/// A sequenced, timestamped entry in the verification case ledger.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct LedgerEntry {
    pub schema_version: String,
    pub entry_id: LedgerEntryId,
    pub case_id: VerificationCaseId,
    pub sequence_no: u64,
    pub recorded_at: String,
    pub event: LedgerEvent,
}

impl LedgerEntry {
    /// Appends a timestamped ledger event for a single verification case.
    pub fn new(case_id: VerificationCaseId, sequence_no: u64, event: LedgerEvent) -> Self {
        Self {
            schema_version: LEDGER_ENTRY_V1_SCHEMA.into(),
            entry_id: LedgerEntryId::generate(),
            case_id,
            sequence_no,
            recorded_at: chrono::Utc::now().to_rfc3339(),
            event,
        }
    }
}

/// Aggregate state reconstructed by replaying a verification case ledger.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplayedCaseState {
    pub case: Option<VerificationCase>,
    pub plan_count: usize,
    pub attempt_count: usize,
    pub receipt_count: usize,
    pub terminal_disposition: Option<TerminalDisposition>,
}

/// Maps a check method onto the scheduler workload class used for budgeting and routing.
pub fn infer_workload_class(method: CheckMethod) -> VerificationWorkloadClass {
    match method {
        CheckMethod::PairedPatch => VerificationWorkloadClass::Patch,
        CheckMethod::AdvisoryOnly => VerificationWorkloadClass::Advisory,
        _ => VerificationWorkloadClass::Oracle,
    }
}

/// Combines policy, calibration, degradation, budget, and method outcome into promotion eligibility.
pub fn interpret_promotion_eligibility(
    policy_allows_promotion: bool,
    calibration_forces_advisory_only: bool,
    degraded: bool,
    budget_exhausted: bool,
    method_succeeded: bool,
) -> bool {
    policy_allows_promotion
        && !calibration_forces_advisory_only
        && !degraded
        && !budget_exhausted
        && method_succeeded
}

/// Returns true when a refuted artifact must be rolled back from an already promoted state.
pub fn interpret_rollback_requirement(refuted: bool, currently_promoted: bool) -> bool {
    refuted && currently_promoted
}

/// Builds the scheduler decision record that preserves budget and degradation lineage.
pub fn schedule_check_plan(
    case: &VerificationCase,
    plan: &CheckPlan,
    budget_lineage: BudgetLineage,
    degradation_markers: Vec<DegradationMarker>,
    queue_hops: Vec<QueueHop>,
) -> SchedulerDecision {
    let exactness_budget = Some(build_exactness_budget(
        plan,
        &budget_lineage,
        &degradation_markers,
    ));
    let proof_blocked = !plan.proof_obligations_remaining.is_empty()
        || matches!(
            plan.evidence_admissibility,
            EvidenceAdmissibilityV1::Inadmissible | EvidenceAdmissibilityV1::Unknown
        );

    SchedulerDecision {
        schema_version: SCHEDULER_DECISION_V1_SCHEMA.into(),
        case_id: case.case_id.clone(),
        plan_id: plan.plan_id.clone(),
        workload_class: infer_workload_class(plan.method),
        cheapest_adequate: plan.cheapest_adequate,
        exactness_budget,
        promotion_blocked: budget_lineage.exhausted
            || degradation_markers
                .iter()
                .any(|marker| marker.blocks_promotion)
            || proof_blocked,
        queue_hops,
        budget_lineage,
        degradation_markers,
    }
}

fn build_exactness_budget(
    plan: &CheckPlan,
    budget_lineage: &BudgetLineage,
    degradation_markers: &[DegradationMarker],
) -> ExactnessBudgetV1 {
    let degradation = if degradation_markers.is_empty() && !plan.degraded {
        Vec::new()
    } else {
        let mut mapped = degradation_markers
            .iter()
            .map(|marker| match marker.kind.as_str() {
                "thin_export" => DegradationKindV1::ThinExport,
                "missing_proof" => DegradationKindV1::MissingProof,
                "missing_replay" => DegradationKindV1::MissingReplay,
                "budget_exhausted" => DegradationKindV1::BudgetExceeded,
                _ => DegradationKindV1::AdvisoryOnly,
            })
            .collect::<Vec<_>>();
        if plan.degraded && !mapped.contains(&DegradationKindV1::ExactnessDowngraded) {
            mapped.push(DegradationKindV1::ExactnessDowngraded);
        }
        mapped
    };
    let budget_units = budget_lineage
        .max_cost_budget_units
        .unwrap_or_else(|| budget_lineage.max_time_budget_ms.unwrap_or_default());
    let remaining_units = budget_lineage
        .remaining_cost_budget_units
        .or(budget_lineage.remaining_time_budget_ms)
        .unwrap_or_default();
    let units_consumed = budget_units.saturating_sub(remaining_units);
    ExactnessBudgetV1 {
        schema_version: semantic_memory_forge::EXACTNESS_BUDGET_V1_SCHEMA.into(),
        exactness_budget_id: stack_ids::ExactnessBudgetId::generate(),
        semantics_profile_id: stack_ids::SemanticsProfileId::new("semantics-profile:control-plane"),
        subject: plan.plan_id.to_string(),
        requested_exactness: plan.target_exactness.clone(),
        achieved_exactness: if plan.degraded {
            ExactnessLevelV1::Conservative
        } else {
            plan.target_exactness.clone()
        },
        budget_units,
        units_consumed,
        degradation,
        escalation_rules: vec![ExactnessEscalationRuleV1 {
            when_degraded: DegradationKindV1::ExactnessDowngraded,
            escalate_to: plan.target_exactness.clone(),
            block_action: Some("promotion".into()),
        }],
        failure_artifact_refs: plan
            .proof_obligations_remaining
            .iter()
            .map(|obligation| format!("proof-obligation:{obligation}"))
            .collect(),
    }
}

/// Replays a case ledger in sequence order and reconstructs the terminal aggregate state.
pub fn replay_case(entries: &[LedgerEntry]) -> Result<ReplayedCaseState, String> {
    let mut ordered = entries.to_vec();
    ordered.sort_by_key(|entry| entry.sequence_no);

    let mut case = None;
    let mut plan_count = 0;
    let mut attempt_count = 0;
    let mut receipt_count = 0;
    let mut terminal_disposition = None;
    let mut expected_case_id = None::<VerificationCaseId>;

    for entry in ordered {
        if let Some(expected) = &expected_case_id {
            if entry.case_id != *expected {
                return Err("ledger replay encountered mixed case ids".into());
            }
        } else {
            expected_case_id = Some(entry.case_id.clone());
        }

        match entry.event {
            LedgerEvent::CaseOpened { case: opened_case } => case = Some(opened_case),
            LedgerEvent::PlanAdopted { .. } => plan_count += 1,
            LedgerEvent::AttemptRecorded { .. } => attempt_count += 1,
            LedgerEvent::ReceiptAppended { .. } => receipt_count += 1,
            LedgerEvent::CaseClosed { disposition, .. } => {
                terminal_disposition = Some(disposition.clone());
                if let Some(open_case) = case.take() {
                    case = Some(open_case.close(disposition));
                }
            }
        }
    }

    Ok(ReplayedCaseState {
        case,
        plan_count,
        attempt_count,
        receipt_count,
        terminal_disposition,
    })
}

/// Converts retry ownership into the stable label used on control-plane artifacts.
pub fn retry_owner_label(retry_owner: &ToolRetryOwner) -> &'static str {
    match retry_owner {
        ToolRetryOwner::LlmPipeline => "llm_pipeline",
        ToolRetryOwner::AgentGraph => "agent_graph",
        ToolRetryOwner::JobQueue => "job_queue",
        ToolRetryOwner::AiBatchQueue => "ai_batch_queue",
        ToolRetryOwner::ForgeOrchestration => "forge_orchestration",
        ToolRetryOwner::External => "external",
    }
}

#[cfg(test)]
#[path = "lib_tests.rs"]
mod tests;
