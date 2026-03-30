use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use stack_ids::{
    CounterfactualSliceId, ExperimentCaseId, InterventionId, RollbackDecisionId, RolloutDecisionId,
};

pub const ROLLOUT_DECISION_V1_SCHEMA: &str = "rollout_decision_v1";
pub const ROLLBACK_DECISION_V1_SCHEMA: &str = "rollback_decision_v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RolloutDecisionClassV1 {
    Canary,
    FullRelease,
    Holdback,
    Blocked,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct RolloutDecisionV1 {
    pub schema_version: String,
    pub rollout_decision_id: RolloutDecisionId,
    pub intervention_id: InterventionId,
    pub decision_class: RolloutDecisionClassV1,
    pub allowed_blast_radius: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub observability_obligations: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub rollback_trigger_conditions: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub quarantine_trigger_conditions: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub policy_basis: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RollbackClassV1 {
    Precautionary,
    Mandatory,
    Emergency,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct RollbackDecisionV1 {
    pub schema_version: String,
    pub rollback_decision_id: RollbackDecisionId,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub triggering_evidence: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub affected_surfaces: Vec<String>,
    pub experiment_case_id: ExperimentCaseId,
    pub counterfactual_slice_id: CounterfactualSliceId,
    pub rollback_class: RollbackClassV1,
    pub remaining_uncertainty: String,
    pub motivation: String,
}

impl RolloutDecisionV1 {
    /// Rejects rollout decisions with invalid schema versions or missing policy basis.
    pub fn validate(&self) -> Result<(), String> {
        if self.schema_version != ROLLOUT_DECISION_V1_SCHEMA {
            return Err(format!(
                "expected schema_version `{ROLLOUT_DECISION_V1_SCHEMA}`, found `{}`",
                self.schema_version
            ));
        }
        if self.policy_basis.is_empty() {
            return Err("rollout decision requires at least one policy basis ref".into());
        }
        Ok(())
    }
}

impl RollbackDecisionV1 {
    /// Rejects rollback decisions with invalid schema versions or missing evidence.
    pub fn validate(&self) -> Result<(), String> {
        if self.schema_version != ROLLBACK_DECISION_V1_SCHEMA {
            return Err(format!(
                "expected schema_version `{ROLLBACK_DECISION_V1_SCHEMA}`, found `{}`",
                self.schema_version
            ));
        }
        if self.triggering_evidence.is_empty() {
            return Err("rollback decision requires at least one triggering evidence ref".into());
        }
        Ok(())
    }
}
