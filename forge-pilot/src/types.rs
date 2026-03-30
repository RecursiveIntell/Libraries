//! Shared types for loop reports, decision audits, and verification artifacts.
//!
//! These types are serialized into loop iteration reports and consumed
//! by downstream verification and adjudication surfaces.

use schemars::JsonSchema;
use semantic_memory_forge::{ExactnessLevelV1, ExecutionContextV1};
use serde::{Deserialize, Serialize};
use verification_control::{CheapCheckStatusV1, ProofProfileV1};

/// The canonical verification case class for a target.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum CanonicalCaseClass {
    ContradictionInvestigation,
    PromotionCandidate,
    ThinExportGap,
    SupersessionVerification,
    ComparabilityDrift,
    CalibrationCaveat,
    ScopeFreshness,
}

/// Execution budget tier assigned to a verification target.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum BudgetClass {
    Micro,
    Standard,
    Expensive,
}

/// An individual verification step in the lawful step ladder for a target.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum LawfulStepKind {
    ContractSchemaCheck,
    ProvenanceReceiptAudit,
    TemporalConsistencyCheck,
    ExactReplay,
    PairedComparativeCheck,
    ExactOracleSlice,
    ConservativeOracleSlice,
    MinimalPerturbationRefuter,
    NuisanceComparabilityAudit,
    HumanReviewRequest,
    CanonicalExportRequest,
    CanonicalImportRequest,
}

/// Normalized representation of a target for cross-iteration comparability.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct TargetNormalization {
    pub stable_target_key: String,
    pub canonical_case_class: CanonicalCaseClass,
    pub bounded_region_digest: String,
    #[serde(default)]
    pub required_artifact_families: Vec<String>,
    pub budget_class: BudgetClass,
    pub comparability_required: bool,
    pub nuisance_sensitive: bool,
    pub missing_falsifier: bool,
}

/// A single step in a verification plan with its cost and artifact requirements.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct PlannedStep {
    pub step_kind: LawfulStepKind,
    pub cost_rank: u8,
    #[serde(default)]
    pub required_artifact_families: Vec<String>,
}

/// Record of a verification step that was blocked and the reason why.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct BlockedStepRecord {
    pub step_kind: LawfulStepKind,
    pub reason: String,
}

/// Audit record capturing the decision rationale for a selected target.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct DecisionAudit {
    pub stable_target_key: String,
    pub canonical_case_class: CanonicalCaseClass,
    pub budget_class: BudgetClass,
    pub target_exactness: ExactnessLevelV1,
    pub cheapest_admissible: Option<LawfulStepKind>,
    #[serde(default)]
    pub fallback_steps: Vec<PlannedStep>,
    #[serde(default)]
    pub blocked_steps: Vec<BlockedStepRecord>,
    #[serde(default)]
    pub cheap_check_ladder: Vec<CheapCheckStatusV1>,
    pub advisory_only: bool,
}

/// Lineage receipt linking a verification case to its consumed and produced artifacts.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ExecutionLineageReceipt {
    pub case_id: String,
    pub plan_id: String,
    pub attempt_id: String,
    pub target_key: String,
    pub execution_context_ref: String,
    #[serde(default)]
    pub consumed_artifact_refs: Vec<String>,
    #[serde(default)]
    pub produced_artifact_refs: Vec<String>,
    #[serde(default)]
    pub degradation_markers: Vec<String>,
    pub outcome_summary: String,
}

/// Trace record for the export-import roundtrip of an action's evidence bundle.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ExportActionTraceV1 {
    pub case_id: String,
    pub plan_id: String,
    pub attempt_id: String,
    pub action_family: String,
    pub bridge_roundtrip_completed: bool,
    pub import_completed: bool,
    #[serde(default)]
    pub produced_artifact_refs: Vec<String>,
}

/// Evaluation of the loop stop rule at the end of an iteration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct StopRuleEvaluation {
    pub halt_reason: String,
    pub retry_cap_reached: bool,
    pub cooldown_applied: bool,
    pub damping_applied: bool,
    pub degraded: bool,
    pub advisory_only: bool,
}

/// Classification of a repair action taken during or after a loop iteration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RepairClassV1 {
    IdentityRepair,
    TemporalRepair,
    SupersessionRepair,
    RollbackRepair,
    BundleImportRepair,
    ScopeWideningRepair,
    VerificationStateRepair,
}

/// Serializable verification plan artifact emitted per iteration.
///
/// Captures proof obligations, admissible evidence, cheapest check ladders,
/// and policy blockers for downstream adjudication.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "VerificationPlanArtifactV1")]
pub struct VerificationPlanArtifact {
    pub schema_version: String,
    pub plan_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub proof_profile: Option<ProofProfileV1>,
    #[serde(default)]
    pub cheapest_checks: Vec<String>,
    #[serde(default)]
    pub cheap_check_ladder: Vec<CheapCheckStatusV1>,
    #[serde(default)]
    pub replay_recipe: Vec<String>,
    #[serde(default)]
    pub replay_preconditions: Vec<String>,
    #[serde(default)]
    pub blocked_checks: Vec<String>,
    #[serde(default)]
    pub proof_obligations_remaining: Vec<String>,
    #[serde(default)]
    pub admissible_evidence: Vec<String>,
    #[serde(default)]
    pub refutation_suggestions: Vec<String>,
    #[serde(default)]
    pub degradation_flags: Vec<String>,
    #[serde(default)]
    pub policy_blockers: Vec<String>,
    pub promotion_blocked_on_missing_proof: bool,
    pub target_exactness: ExactnessLevelV1,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<String>,
}

/// Structured record of a repair action applied to verification state.
///
/// Includes blast radius, reversibility, and a link back to the
/// execution context that triggered the repair.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "RepairRecordV1")]
pub struct RepairRecordV1 {
    pub schema_version: String,
    pub repair_record_id: String,
    #[serde(default)]
    pub affected_identities: Vec<String>,
    pub repair_class: RepairClassV1,
    #[serde(default)]
    pub trigger_artifacts: Vec<String>,
    pub blast_radius: String,
    pub reversibility: String,
    pub action: String,
    pub execution_context: ExecutionContextV1,
    pub opened_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resolved_at: Option<String>,
    pub unchanged_statement: String,
}
