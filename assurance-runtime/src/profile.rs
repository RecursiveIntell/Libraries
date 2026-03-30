use crate::error::{
    require_non_empty, require_non_empty_slice, AssuranceValidationError, AssuranceValidationResult,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AssurancePublicationStatusV1 {
    Required,
    AdvisoryOnly,
    Deferred,
    Suppressed,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct DeploymentProfileV1 {
    pub schema_version: String,
    pub deployment_profile_id: String,
    pub system_name: String,
    pub deployment_environment: String,
    pub criticality_class: String,
    pub operating_envelope_id: String,
    pub assurance_case_id: String,
    pub publication_status: AssurancePublicationStatusV1,
}

impl DeploymentProfileV1 {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        deployment_profile_id: impl Into<String>,
        system_name: impl Into<String>,
        deployment_environment: impl Into<String>,
        criticality_class: impl Into<String>,
        operating_envelope_id: impl Into<String>,
        assurance_case_id: impl Into<String>,
        publication_status: AssurancePublicationStatusV1,
    ) -> Result<Self, AssuranceValidationError> {
        let value = Self {
            schema_version: "DeploymentProfileV1".to_string(),
            deployment_profile_id: deployment_profile_id.into(),
            system_name: system_name.into(),
            deployment_environment: deployment_environment.into(),
            criticality_class: criticality_class.into(),
            operating_envelope_id: operating_envelope_id.into(),
            assurance_case_id: assurance_case_id.into(),
            publication_status,
        };
        value.validate()?;
        Ok(value)
    }

    pub fn validate(&self) -> AssuranceValidationResult {
        require_non_empty(&self.deployment_profile_id, "deployment_profile_id")?;
        require_non_empty(&self.system_name, "system_name")?;
        require_non_empty(&self.deployment_environment, "deployment_environment")?;
        require_non_empty(&self.criticality_class, "criticality_class")?;
        require_non_empty(&self.operating_envelope_id, "operating_envelope_id")?;
        require_non_empty(&self.assurance_case_id, "assurance_case_id")?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct OperatingEnvelopeV1 {
    pub schema_version: String,
    pub operating_envelope_id: String,
    pub allowed_regions: Vec<String>,
    pub traffic_ceiling: String,
    pub data_class_ceiling: String,
    pub dependency_constraints: Vec<String>,
    pub degradation_behavior: String,
}

impl OperatingEnvelopeV1 {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        operating_envelope_id: impl Into<String>,
        allowed_regions: Vec<String>,
        traffic_ceiling: impl Into<String>,
        data_class_ceiling: impl Into<String>,
        dependency_constraints: Vec<String>,
        degradation_behavior: impl Into<String>,
    ) -> Result<Self, AssuranceValidationError> {
        let value = Self {
            schema_version: "OperatingEnvelopeV1".to_string(),
            operating_envelope_id: operating_envelope_id.into(),
            allowed_regions,
            traffic_ceiling: traffic_ceiling.into(),
            data_class_ceiling: data_class_ceiling.into(),
            dependency_constraints,
            degradation_behavior: degradation_behavior.into(),
        };
        value.validate()?;
        Ok(value)
    }

    pub fn validate(&self) -> AssuranceValidationResult {
        require_non_empty(&self.operating_envelope_id, "operating_envelope_id")?;
        require_non_empty_slice(&self.allowed_regions, "allowed_regions")?;
        require_non_empty(&self.traffic_ceiling, "traffic_ceiling")?;
        require_non_empty(&self.data_class_ceiling, "data_class_ceiling")?;
        require_non_empty(&self.degradation_behavior, "degradation_behavior")?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct HazardRegisterV1 {
    pub schema_version: String,
    pub hazard_register_id: String,
    pub deployment_profile_id: String,
    pub hazards: Vec<String>,
    pub severity_class: String,
    pub owner_ref: String,
    pub last_reviewed_at: String,
}

impl HazardRegisterV1 {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        hazard_register_id: impl Into<String>,
        deployment_profile_id: impl Into<String>,
        hazards: Vec<String>,
        severity_class: impl Into<String>,
        owner_ref: impl Into<String>,
        last_reviewed_at: impl Into<String>,
    ) -> Result<Self, AssuranceValidationError> {
        let value = Self {
            schema_version: "HazardRegisterV1".to_string(),
            hazard_register_id: hazard_register_id.into(),
            deployment_profile_id: deployment_profile_id.into(),
            hazards,
            severity_class: severity_class.into(),
            owner_ref: owner_ref.into(),
            last_reviewed_at: last_reviewed_at.into(),
        };
        value.validate()?;
        Ok(value)
    }

    pub fn validate(&self) -> AssuranceValidationResult {
        require_non_empty(&self.hazard_register_id, "hazard_register_id")?;
        require_non_empty(&self.deployment_profile_id, "deployment_profile_id")?;
        require_non_empty_slice(&self.hazards, "hazards")?;
        require_non_empty(&self.severity_class, "severity_class")?;
        require_non_empty(&self.owner_ref, "owner_ref")?;
        require_non_empty(&self.last_reviewed_at, "last_reviewed_at")?;
        Ok(())
    }
}
