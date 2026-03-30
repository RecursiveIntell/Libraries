use crate::error::{
    require_non_empty, require_non_empty_slice, AssuranceValidationError, AssuranceValidationResult,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ControlCoverageStateV1 {
    Covered,
    Partial,
    GapPresent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ReleaseDecisionStateV1 {
    Approved,
    ApprovedWithMonitoring,
    Blocked,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct AssuranceCaseV1 {
    pub schema_version: String,
    pub assurance_case_id: String,
    pub claim_summary: String,
    pub evidence_refs: Vec<String>,
    pub argument_structure: String,
    pub open_obligations: Vec<String>,
    pub ready_for_release: bool,
}

impl AssuranceCaseV1 {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        assurance_case_id: impl Into<String>,
        claim_summary: impl Into<String>,
        evidence_refs: Vec<String>,
        argument_structure: impl Into<String>,
        open_obligations: Vec<String>,
        ready_for_release: bool,
    ) -> Result<Self, AssuranceValidationError> {
        let value = Self {
            schema_version: "AssuranceCaseV1".to_string(),
            assurance_case_id: assurance_case_id.into(),
            claim_summary: claim_summary.into(),
            evidence_refs,
            argument_structure: argument_structure.into(),
            open_obligations,
            ready_for_release,
        };
        value.validate()?;
        Ok(value)
    }

    pub fn validate(&self) -> AssuranceValidationResult {
        require_non_empty(&self.assurance_case_id, "assurance_case_id")?;
        require_non_empty(&self.claim_summary, "claim_summary")?;
        require_non_empty_slice(&self.evidence_refs, "evidence_refs")?;
        require_non_empty(&self.argument_structure, "argument_structure")?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ControlMappingV1 {
    pub schema_version: String,
    pub control_mapping_id: String,
    pub hazard_register_id: String,
    pub control_refs: Vec<String>,
    pub coverage_state: ControlCoverageStateV1,
    pub gaps: Vec<String>,
}

impl ControlMappingV1 {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        control_mapping_id: impl Into<String>,
        hazard_register_id: impl Into<String>,
        control_refs: Vec<String>,
        coverage_state: ControlCoverageStateV1,
        gaps: Vec<String>,
    ) -> Result<Self, AssuranceValidationError> {
        let value = Self {
            schema_version: "ControlMappingV1".to_string(),
            control_mapping_id: control_mapping_id.into(),
            hazard_register_id: hazard_register_id.into(),
            control_refs,
            coverage_state,
            gaps,
        };
        value.validate()?;
        Ok(value)
    }

    pub fn validate(&self) -> AssuranceValidationResult {
        require_non_empty(&self.control_mapping_id, "control_mapping_id")?;
        require_non_empty(&self.hazard_register_id, "hazard_register_id")?;
        require_non_empty_slice(&self.control_refs, "control_refs")?;
        if matches!(self.coverage_state, ControlCoverageStateV1::Covered) && !self.gaps.is_empty() {
            return Err(AssuranceValidationError::InvalidState(
                "covered mapping must not retain gaps",
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ResidualRiskAcceptanceV1 {
    pub schema_version: String,
    pub residual_risk_acceptance_id: String,
    pub deployment_profile_id: String,
    pub accepted_by: Vec<String>,
    pub accepted_risks: Vec<String>,
    pub expires_at: String,
    pub notes: Vec<String>,
}

impl ResidualRiskAcceptanceV1 {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        residual_risk_acceptance_id: impl Into<String>,
        deployment_profile_id: impl Into<String>,
        accepted_by: Vec<String>,
        accepted_risks: Vec<String>,
        expires_at: impl Into<String>,
        notes: Vec<String>,
    ) -> Result<Self, AssuranceValidationError> {
        let value = Self {
            schema_version: "ResidualRiskAcceptanceV1".to_string(),
            residual_risk_acceptance_id: residual_risk_acceptance_id.into(),
            deployment_profile_id: deployment_profile_id.into(),
            accepted_by,
            accepted_risks,
            expires_at: expires_at.into(),
            notes,
        };
        value.validate()?;
        Ok(value)
    }

    pub fn validate(&self) -> AssuranceValidationResult {
        require_non_empty(
            &self.residual_risk_acceptance_id,
            "residual_risk_acceptance_id",
        )?;
        require_non_empty(&self.deployment_profile_id, "deployment_profile_id")?;
        require_non_empty_slice(&self.accepted_by, "accepted_by")?;
        require_non_empty_slice(&self.accepted_risks, "accepted_risks")?;
        require_non_empty(&self.expires_at, "expires_at")?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ReleaseReadinessDecisionV1 {
    pub schema_version: String,
    pub release_readiness_decision_id: String,
    pub deployment_profile_id: String,
    pub decision_state: ReleaseDecisionStateV1,
    pub blocking_gaps: Vec<String>,
    pub required_monitors: Vec<String>,
    pub advisory_only: bool,
    pub generated_at: String,
}

impl ReleaseReadinessDecisionV1 {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        release_readiness_decision_id: impl Into<String>,
        deployment_profile_id: impl Into<String>,
        decision_state: ReleaseDecisionStateV1,
        blocking_gaps: Vec<String>,
        required_monitors: Vec<String>,
        advisory_only: bool,
        generated_at: impl Into<String>,
    ) -> Result<Self, AssuranceValidationError> {
        let value = Self {
            schema_version: "ReleaseReadinessDecisionV1".to_string(),
            release_readiness_decision_id: release_readiness_decision_id.into(),
            deployment_profile_id: deployment_profile_id.into(),
            decision_state,
            blocking_gaps,
            required_monitors,
            advisory_only,
            generated_at: generated_at.into(),
        };
        value.validate()?;
        Ok(value)
    }

    pub fn validate(&self) -> AssuranceValidationResult {
        require_non_empty(
            &self.release_readiness_decision_id,
            "release_readiness_decision_id",
        )?;
        require_non_empty(&self.deployment_profile_id, "deployment_profile_id")?;
        require_non_empty(&self.generated_at, "generated_at")?;
        match self.decision_state {
            ReleaseDecisionStateV1::ApprovedWithMonitoring => {
                require_non_empty_slice(&self.required_monitors, "required_monitors")?;
            }
            ReleaseDecisionStateV1::Blocked => {
                require_non_empty_slice(&self.blocking_gaps, "blocking_gaps")?;
            }
            ReleaseDecisionStateV1::Approved => {}
        }
        Ok(())
    }
}
