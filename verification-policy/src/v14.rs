use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use stack_ids::{
    ArtifactAdmissionPolicyId, DisclosureBudgetId, DisclosurePolicyId, ExperimentBudgetId,
    RefuterSuiteId, TrustRootSetId,
};

pub const REFUTER_SUITE_V1_SCHEMA: &str = "refuter_suite_v1";
pub const EXPERIMENT_BUDGET_V1_SCHEMA: &str = "experiment_budget_v1";
pub const ARTIFACT_ADMISSION_POLICY_V1_SCHEMA: &str = "artifact_admission_policy_v1";
pub const DISCLOSURE_POLICY_V1_SCHEMA: &str = "disclosure_policy_v1";
pub const DISCLOSURE_BUDGET_V1_SCHEMA: &str = "disclosure_budget_v1";

fn require_schema_version(
    found: &str,
    expected: &'static str,
    field: &'static str,
) -> Result<(), &'static str> {
    if found != expected {
        return Err(field);
    }
    Ok(())
}

fn require_non_empty(value: &str, field: &'static str) -> Result<(), &'static str> {
    if value.trim().is_empty() {
        return Err(field);
    }
    Ok(())
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum ExperimentBudgetClassV1 {
    Small,
    Medium,
    Large,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum ExperimentMaxExactnessV1 {
    BoundedExact,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
pub enum RefuterAllowanceV1 {
    #[serde(rename = "up_to_1")]
    UpTo1,
    #[serde(rename = "up_to_3")]
    UpTo3,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum OracleAllowanceV1 {
    OneExactSlice,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum ReplayAllowanceV1 {
    FullReplayFamily,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum HumanReviewAllowanceV1 {
    OneRequiredIfRollout,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum BudgetExhaustionBehaviorV1 {
    EmitTypedDegradationAndHoldAdvisory,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum PolicyReplayabilityClassV1 {
    ReplayableWithTicket,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum PolicyDowngradeBehaviorV1 {
    AdvisoryOnlyIfReplayMissing,
    AdvisoryOnlyIfExactMaterialCannotBeShared,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum DisclosureReplayVisibilityV1 {
    TicketRequired,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
pub enum DisclosureRevealClassV1 {
    #[serde(rename = "structured-redacted")]
    StructuredRedacted,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct RefuterSuiteV1 {
    pub schema_version: String,
    pub refuter_suite_id: RefuterSuiteId,
    pub treatment_definition_check: bool,
    pub negative_control_option: bool,
    pub placebo_option: bool,
    pub dummy_outcome_option: bool,
    pub subset_stability_option: bool,
    pub alternative_comparison_option: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub admissibility_notes: Vec<String>,
    pub cost_metadata: String,
}

impl RefuterSuiteV1 {
    #[allow(clippy::too_many_arguments)]
    /// Builds a refuter-suite profile with an explicit cost envelope.
    pub fn new(
        refuter_suite_id: RefuterSuiteId,
        treatment_definition_check: bool,
        negative_control_option: bool,
        placebo_option: bool,
        dummy_outcome_option: bool,
        subset_stability_option: bool,
        alternative_comparison_option: bool,
        admissibility_notes: Vec<String>,
        cost_metadata: impl Into<String>,
    ) -> Result<Self, &'static str> {
        let value = Self {
            schema_version: REFUTER_SUITE_V1_SCHEMA.to_string(),
            refuter_suite_id,
            treatment_definition_check,
            negative_control_option,
            placebo_option,
            dummy_outcome_option,
            subset_stability_option,
            alternative_comparison_option,
            admissibility_notes,
            cost_metadata: cost_metadata.into(),
        };
        value.validate()?;
        Ok(value)
    }

    /// Validates schema identity, non-empty cost metadata, and that at least
    /// one refuter option is enabled.
    pub fn validate(&self) -> Result<(), &'static str> {
        require_schema_version(
            &self.schema_version,
            REFUTER_SUITE_V1_SCHEMA,
            "schema_version",
        )?;
        require_non_empty(&self.cost_metadata, "cost_metadata")?;
        if !self.treatment_definition_check
            && !self.negative_control_option
            && !self.placebo_option
            && !self.dummy_outcome_option
            && !self.subset_stability_option
            && !self.alternative_comparison_option
        {
            return Err("refuter_suite_options");
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ExperimentBudgetV1 {
    pub schema_version: String,
    pub experiment_budget_id: ExperimentBudgetId,
    pub budget_class: ExperimentBudgetClassV1,
    pub max_exactness: ExperimentMaxExactnessV1,
    pub refuter_allowance: RefuterAllowanceV1,
    pub oracle_allowance: OracleAllowanceV1,
    pub replay_allowance: ReplayAllowanceV1,
    pub human_review_allowance: HumanReviewAllowanceV1,
    pub exhaustion_behavior: BudgetExhaustionBehaviorV1,
}

impl ExperimentBudgetV1 {
    #[allow(clippy::too_many_arguments)]
    /// Builds a typed experiment budget profile.
    pub fn new(
        experiment_budget_id: ExperimentBudgetId,
        budget_class: ExperimentBudgetClassV1,
        max_exactness: ExperimentMaxExactnessV1,
        refuter_allowance: RefuterAllowanceV1,
        oracle_allowance: OracleAllowanceV1,
        replay_allowance: ReplayAllowanceV1,
        human_review_allowance: HumanReviewAllowanceV1,
        exhaustion_behavior: BudgetExhaustionBehaviorV1,
    ) -> Result<Self, &'static str> {
        let value = Self {
            schema_version: EXPERIMENT_BUDGET_V1_SCHEMA.to_string(),
            experiment_budget_id,
            budget_class,
            max_exactness,
            refuter_allowance,
            oracle_allowance,
            replay_allowance,
            human_review_allowance,
            exhaustion_behavior,
        };
        value.validate()?;
        Ok(value)
    }

    /// Validates the experiment-budget schema identity.
    pub fn validate(&self) -> Result<(), &'static str> {
        require_schema_version(
            &self.schema_version,
            EXPERIMENT_BUDGET_V1_SCHEMA,
            "schema_version",
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ArtifactAdmissionPolicyV1 {
    pub schema_version: String,
    pub artifact_admission_policy_id: ArtifactAdmissionPolicyId,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub allowed_artifact_families: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub required_trust_root_sets: Vec<TrustRootSetId>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub required_transparency_obligations: Vec<String>,
    pub required_replayability_class: PolicyReplayabilityClassV1,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub disclosure_constraints: Vec<String>,
    pub downgrade_behavior: PolicyDowngradeBehaviorV1,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub disqualifying_failures: Vec<String>,
}

impl ArtifactAdmissionPolicyV1 {
    #[allow(clippy::too_many_arguments)]
    /// Builds a typed artifact-admission policy.
    pub fn new(
        artifact_admission_policy_id: ArtifactAdmissionPolicyId,
        allowed_artifact_families: Vec<String>,
        required_trust_root_sets: Vec<TrustRootSetId>,
        required_transparency_obligations: Vec<String>,
        required_replayability_class: PolicyReplayabilityClassV1,
        disclosure_constraints: Vec<String>,
        downgrade_behavior: PolicyDowngradeBehaviorV1,
        disqualifying_failures: Vec<String>,
    ) -> Result<Self, &'static str> {
        let value = Self {
            schema_version: ARTIFACT_ADMISSION_POLICY_V1_SCHEMA.to_string(),
            artifact_admission_policy_id,
            allowed_artifact_families,
            required_trust_root_sets,
            required_transparency_obligations,
            required_replayability_class,
            disclosure_constraints,
            downgrade_behavior,
            disqualifying_failures,
        };
        value.validate()?;
        Ok(value)
    }

    /// Validates the artifact-admission policy schema identity.
    pub fn validate(&self) -> Result<(), &'static str> {
        require_schema_version(
            &self.schema_version,
            ARTIFACT_ADMISSION_POLICY_V1_SCHEMA,
            "schema_version",
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct DisclosurePolicyV1 {
    pub schema_version: String,
    pub disclosure_policy_id: DisclosurePolicyId,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub allowed_consumers: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub redaction_rules: Vec<String>,
    pub replay_visibility: DisclosureReplayVisibilityV1,
    pub retention_window: String,
    pub downgrade_behavior: PolicyDowngradeBehaviorV1,
}

impl DisclosurePolicyV1 {
    /// Builds a typed disclosure policy.
    pub fn new(
        disclosure_policy_id: DisclosurePolicyId,
        allowed_consumers: Vec<String>,
        redaction_rules: Vec<String>,
        replay_visibility: DisclosureReplayVisibilityV1,
        retention_window: impl Into<String>,
        downgrade_behavior: PolicyDowngradeBehaviorV1,
    ) -> Result<Self, &'static str> {
        let value = Self {
            schema_version: DISCLOSURE_POLICY_V1_SCHEMA.to_string(),
            disclosure_policy_id,
            allowed_consumers,
            redaction_rules,
            replay_visibility,
            retention_window: retention_window.into(),
            downgrade_behavior,
        };
        value.validate()?;
        Ok(value)
    }

    /// Validates disclosure-policy schema identity and retention metadata.
    pub fn validate(&self) -> Result<(), &'static str> {
        require_schema_version(
            &self.schema_version,
            DISCLOSURE_POLICY_V1_SCHEMA,
            "schema_version",
        )?;
        require_non_empty(&self.retention_window, "retention_window")?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct DisclosureBudgetV1 {
    pub schema_version: String,
    pub disclosure_budget_id: DisclosureBudgetId,
    pub allowed_reveal_class: DisclosureRevealClassV1,
    pub current_spend: String,
    pub redaction_ceiling: String,
    pub escalation_path: String,
}

impl DisclosureBudgetV1 {
    /// Builds a typed disclosure budget.
    pub fn new(
        disclosure_budget_id: DisclosureBudgetId,
        allowed_reveal_class: DisclosureRevealClassV1,
        current_spend: impl Into<String>,
        redaction_ceiling: impl Into<String>,
        escalation_path: impl Into<String>,
    ) -> Result<Self, &'static str> {
        let value = Self {
            schema_version: DISCLOSURE_BUDGET_V1_SCHEMA.to_string(),
            disclosure_budget_id,
            allowed_reveal_class,
            current_spend: current_spend.into(),
            redaction_ceiling: redaction_ceiling.into(),
            escalation_path: escalation_path.into(),
        };
        value.validate()?;
        Ok(value)
    }

    /// Validates disclosure-budget schema identity and required budget text
    /// fields.
    pub fn validate(&self) -> Result<(), &'static str> {
        require_schema_version(
            &self.schema_version,
            DISCLOSURE_BUDGET_V1_SCHEMA,
            "schema_version",
        )?;
        require_non_empty(&self.current_spend, "current_spend")?;
        require_non_empty(&self.redaction_ceiling, "redaction_ceiling")?;
        require_non_empty(&self.escalation_path, "escalation_path")?;
        Ok(())
    }
}
