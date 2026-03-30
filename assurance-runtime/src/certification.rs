use crate::error::{
    require_non_empty, require_non_empty_slice, AssuranceValidationError, AssuranceValidationResult,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum CertificationStatusV1 {
    AdvisoryUntilAdmitted,
    Admitted,
    Revoked,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RecertificationTriggerKindV1 {
    SloBreach,
    Incident,
    ControlGap,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct FieldMonitoringPlanV1 {
    pub schema_version: String,
    pub field_monitoring_plan_id: String,
    pub deployment_profile_id: String,
    pub monitors: Vec<String>,
    pub sampling_window: String,
    pub alert_routes: Vec<String>,
    pub recertification_triggers: Vec<String>,
}

impl FieldMonitoringPlanV1 {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        field_monitoring_plan_id: impl Into<String>,
        deployment_profile_id: impl Into<String>,
        monitors: Vec<String>,
        sampling_window: impl Into<String>,
        alert_routes: Vec<String>,
        recertification_triggers: Vec<String>,
    ) -> Result<Self, AssuranceValidationError> {
        let value = Self {
            schema_version: "FieldMonitoringPlanV1".to_string(),
            field_monitoring_plan_id: field_monitoring_plan_id.into(),
            deployment_profile_id: deployment_profile_id.into(),
            monitors,
            sampling_window: sampling_window.into(),
            alert_routes,
            recertification_triggers,
        };
        value.validate()?;
        Ok(value)
    }

    pub fn validate(&self) -> AssuranceValidationResult {
        require_non_empty(&self.field_monitoring_plan_id, "field_monitoring_plan_id")?;
        require_non_empty(&self.deployment_profile_id, "deployment_profile_id")?;
        require_non_empty_slice(&self.monitors, "monitors")?;
        require_non_empty(&self.sampling_window, "sampling_window")?;
        require_non_empty_slice(&self.alert_routes, "alert_routes")?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct CertificationBundleV1 {
    pub schema_version: String,
    pub certification_bundle_id: String,
    pub deployment_profile_id: String,
    pub assurance_case_id: String,
    pub release_readiness_decision_id: String,
    pub issued_by: Vec<String>,
    pub issued_at: String,
    pub status: CertificationStatusV1,
}

impl CertificationBundleV1 {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        certification_bundle_id: impl Into<String>,
        deployment_profile_id: impl Into<String>,
        assurance_case_id: impl Into<String>,
        release_readiness_decision_id: impl Into<String>,
        issued_by: Vec<String>,
        issued_at: impl Into<String>,
        status: CertificationStatusV1,
    ) -> Result<Self, AssuranceValidationError> {
        let value = Self {
            schema_version: "CertificationBundleV1".to_string(),
            certification_bundle_id: certification_bundle_id.into(),
            deployment_profile_id: deployment_profile_id.into(),
            assurance_case_id: assurance_case_id.into(),
            release_readiness_decision_id: release_readiness_decision_id.into(),
            issued_by,
            issued_at: issued_at.into(),
            status,
        };
        value.validate()?;
        Ok(value)
    }

    pub fn validate(&self) -> AssuranceValidationResult {
        require_non_empty(&self.certification_bundle_id, "certification_bundle_id")?;
        require_non_empty(&self.deployment_profile_id, "deployment_profile_id")?;
        require_non_empty(&self.assurance_case_id, "assurance_case_id")?;
        require_non_empty(
            &self.release_readiness_decision_id,
            "release_readiness_decision_id",
        )?;
        require_non_empty_slice(&self.issued_by, "issued_by")?;
        require_non_empty(&self.issued_at, "issued_at")?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct RecertificationTriggerV1 {
    pub schema_version: String,
    pub recertification_trigger_id: String,
    pub deployment_profile_id: String,
    pub trigger_kind: RecertificationTriggerKindV1,
    pub triggered_at: String,
    pub required_action: String,
}

impl RecertificationTriggerV1 {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        recertification_trigger_id: impl Into<String>,
        deployment_profile_id: impl Into<String>,
        trigger_kind: RecertificationTriggerKindV1,
        triggered_at: impl Into<String>,
        required_action: impl Into<String>,
    ) -> Result<Self, AssuranceValidationError> {
        let value = Self {
            schema_version: "RecertificationTriggerV1".to_string(),
            recertification_trigger_id: recertification_trigger_id.into(),
            deployment_profile_id: deployment_profile_id.into(),
            trigger_kind,
            triggered_at: triggered_at.into(),
            required_action: required_action.into(),
        };
        value.validate()?;
        Ok(value)
    }

    pub fn validate(&self) -> AssuranceValidationResult {
        require_non_empty(
            &self.recertification_trigger_id,
            "recertification_trigger_id",
        )?;
        require_non_empty(&self.deployment_profile_id, "deployment_profile_id")?;
        require_non_empty(&self.triggered_at, "triggered_at")?;
        require_non_empty(&self.required_action, "required_action")?;
        Ok(())
    }
}
