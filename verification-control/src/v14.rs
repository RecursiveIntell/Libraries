use schemars::JsonSchema;
use semantic_memory_forge::EvidenceAdmissibilityV1;
use serde::{Deserialize, Serialize};
use stack_ids::{
    CohortContractId, ComparabilityMatrixId, CounterfactualSliceId, DecisionTraceId,
    DisputeBundleId, ExperimentBudgetId, ExperimentCaseId, InterventionId, OutcomeSchemaId,
    RefuterResultId, RefuterSuiteId,
};

pub const EXPERIMENT_CASE_V1_SCHEMA: &str = "experiment_case_v1";
pub const COMPARABILITY_MATRIX_V1_SCHEMA: &str = "comparability_matrix_v1";
pub const REFUTER_RESULT_V1_SCHEMA: &str = "refuter_result_v1";
pub const DECISION_TRACE_V1_SCHEMA: &str = "decision_trace_v1";
pub const DISPUTE_BUNDLE_V1_SCHEMA: &str = "dispute_bundle_v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExperimentRiskClassV1 {
    PromotionAdjacent,
    ObservationOnly,
    ResearchOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExperimentLifecycleStateV1 {
    Scheduled,
    Running,
    Completed,
    Cancelled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExperimentFinalDispositionV1 {
    Pending,
    Promoted,
    Blocked,
    RolledBack,
    AdvisoryOnly,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ExperimentCaseV1 {
    pub schema_version: String,
    pub experiment_case_id: ExperimentCaseId,
    pub intervention_id: InterventionId,
    pub outcome_schema_id: OutcomeSchemaId,
    pub cohort_contract_id: CohortContractId,
    pub comparability_matrix_id: ComparabilityMatrixId,
    pub refuter_suite_id: RefuterSuiteId,
    pub experiment_budget_id: ExperimentBudgetId,
    pub risk_class: ExperimentRiskClassV1,
    pub lifecycle_state: ExperimentLifecycleStateV1,
    pub final_disposition: ExperimentFinalDispositionV1,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ComparabilityMatrixV1 {
    pub schema_version: String,
    pub comparability_matrix_id: ComparabilityMatrixId,
    pub workload_comparability: String,
    pub environment_comparability: String,
    pub config_comparability: String,
    pub time_window_comparability: String,
    pub retry_replay_comparability: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub missingness_markers: Vec<String>,
    pub admissibility_judgment: EvidenceAdmissibilityV1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RefuterKindV1 {
    NegativeControl,
    Placebo,
    SubsetStability,
    HumanReview,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RefuterStateV1 {
    Pass,
    Fail,
    Skipped,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RefuterPromotionImpactV1 {
    KeepsCanaryEligible,
    BlocksPromotion,
    RequiresHumanReview,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct RefuterResultV1 {
    pub schema_version: String,
    pub refuter_result_id: RefuterResultId,
    pub refuter_kind: RefuterKindV1,
    pub counterfactual_slice_id: CounterfactualSliceId,
    pub result_summary: String,
    pub state: RefuterStateV1,
    pub replay_linkage: String,
    pub promotion_impact: RefuterPromotionImpactV1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExactnessSpendV1 {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SelectedDecisionV1 {
    CanaryOnly,
    RetryExact,
    HumanReview,
    Rollback,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct DecisionTraceV1 {
    pub schema_version: String,
    pub decision_trace_id: DecisionTraceId,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub triggering_artifact_refs: Vec<String>,
    pub intervention_id: InterventionId,
    pub counterfactual_slice_id: CounterfactualSliceId,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub refuters_satisfied: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub refuters_failed: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub refuters_skipped: Vec<String>,
    pub exactness_spend: ExactnessSpendV1,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub policy_basis: Vec<String>,
    pub selected_decision: SelectedDecisionV1,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub rollback_prerequisites: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub cheap_next_checks: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum DisputeDispositionV1 {
    Open,
    Resolved,
    Escalated,
    Rejected,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct DisputeBundleV1 {
    pub schema_version: String,
    pub dispute_bundle_id: DisputeBundleId,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub challenged_artifact_refs: Vec<String>,
    pub basis_of_challenge: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub counterevidence_refs: Vec<String>,
    pub replay_or_recheck_request: String,
    pub escalation_target: String,
    pub current_disposition: DisputeDispositionV1,
}

impl ExperimentCaseV1 {
    /// Rejects experiment cases with invalid schema versions or terminal-state drift.
    pub fn validate(&self) -> Result<(), String> {
        if self.schema_version != EXPERIMENT_CASE_V1_SCHEMA {
            return Err(format!(
                "expected schema_version `{EXPERIMENT_CASE_V1_SCHEMA}`, found `{}`",
                self.schema_version
            ));
        }
        if matches!(
            self.final_disposition,
            ExperimentFinalDispositionV1::Pending
        ) && matches!(self.lifecycle_state, ExperimentLifecycleStateV1::Completed)
        {
            return Err("completed experiment case cannot retain pending disposition".into());
        }
        Ok(())
    }
}

impl ComparabilityMatrixV1 {
    /// Rejects comparability matrices with invalid schema versions.
    pub fn validate(&self) -> Result<(), String> {
        if self.schema_version != COMPARABILITY_MATRIX_V1_SCHEMA {
            return Err(format!(
                "expected schema_version `{COMPARABILITY_MATRIX_V1_SCHEMA}`, found `{}`",
                self.schema_version
            ));
        }
        Ok(())
    }
}

impl RefuterResultV1 {
    /// Rejects refuter results whose pass/fail state conflicts with promotion impact.
    pub fn validate(&self) -> Result<(), String> {
        if self.schema_version != REFUTER_RESULT_V1_SCHEMA {
            return Err(format!(
                "expected schema_version `{REFUTER_RESULT_V1_SCHEMA}`, found `{}`",
                self.schema_version
            ));
        }
        if matches!(self.state, RefuterStateV1::Pass)
            && matches!(
                self.promotion_impact,
                RefuterPromotionImpactV1::BlocksPromotion
            )
        {
            return Err("passing refuter result cannot directly block promotion".into());
        }
        Ok(())
    }
}

impl DecisionTraceV1 {
    /// Rejects decision traces with invalid schema versions or empty policy basis.
    pub fn validate(&self) -> Result<(), String> {
        if self.schema_version != DECISION_TRACE_V1_SCHEMA {
            return Err(format!(
                "expected schema_version `{DECISION_TRACE_V1_SCHEMA}`, found `{}`",
                self.schema_version
            ));
        }
        if self.policy_basis.is_empty() {
            return Err("decision trace requires at least one policy basis reference".into());
        }
        Ok(())
    }
}

impl DisputeBundleV1 {
    /// Rejects dispute bundles with invalid schema versions or empty challenge scope.
    pub fn validate(&self) -> Result<(), String> {
        if self.schema_version != DISPUTE_BUNDLE_V1_SCHEMA {
            return Err(format!(
                "expected schema_version `{DISPUTE_BUNDLE_V1_SCHEMA}`, found `{}`",
                self.schema_version
            ));
        }
        if self.challenged_artifact_refs.is_empty() {
            return Err("dispute bundle requires at least one challenged artifact ref".into());
        }
        Ok(())
    }
}
