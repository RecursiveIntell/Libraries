use crate::error::{ContinuityValidationError, ContinuityValidationResult};
use crate::validation::{require_non_empty, require_non_empty_slice, require_schema_version};
use crate::vocab::{IncidentSeverityV1, IncidentStatusV1};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct IncidentCaseV1 {
    pub schema_version: String,
    pub incident_case_id: String,
    pub service_level_profile_id: String,
    pub severity: IncidentSeverityV1,
    pub opened_at: String,
    pub summary: String,
    pub commander_ref: String,
    pub status: IncidentStatusV1,
}

impl IncidentCaseV1 {
    pub const SCHEMA_VERSION: &'static str = "IncidentCaseV1";

    pub fn new(
        incident_case_id: impl Into<String>,
        service_level_profile_id: impl Into<String>,
        severity: IncidentSeverityV1,
        opened_at: impl Into<String>,
        summary: impl Into<String>,
        commander_ref: impl Into<String>,
        status: IncidentStatusV1,
    ) -> Result<Self, ContinuityValidationError> {
        let value = Self {
            schema_version: Self::SCHEMA_VERSION.to_string(),
            incident_case_id: incident_case_id.into(),
            service_level_profile_id: service_level_profile_id.into(),
            severity,
            opened_at: opened_at.into(),
            summary: summary.into(),
            commander_ref: commander_ref.into(),
            status,
        };
        value.validate()?;
        Ok(value)
    }

    pub fn validate(&self) -> ContinuityValidationResult {
        require_schema_version(&self.schema_version, Self::SCHEMA_VERSION)?;
        require_non_empty(&self.incident_case_id, "incident_case_id")?;
        require_non_empty(&self.service_level_profile_id, "service_level_profile_id")?;
        require_non_empty(&self.opened_at, "opened_at")?;
        require_non_empty(&self.summary, "summary")?;
        require_non_empty(&self.commander_ref, "commander_ref")?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ContainmentDecisionV1 {
    pub schema_version: String,
    pub containment_decision_id: String,
    pub incident_case_id: String,
    pub decision: String,
    pub justification: String,
    pub approved_by: Vec<String>,
    pub generated_at: String,
}

impl ContainmentDecisionV1 {
    pub const SCHEMA_VERSION: &'static str = "ContainmentDecisionV1";

    pub fn new(
        containment_decision_id: impl Into<String>,
        incident_case_id: impl Into<String>,
        decision: impl Into<String>,
        justification: impl Into<String>,
        approved_by: Vec<String>,
        generated_at: impl Into<String>,
    ) -> Result<Self, ContinuityValidationError> {
        let value = Self {
            schema_version: Self::SCHEMA_VERSION.to_string(),
            containment_decision_id: containment_decision_id.into(),
            incident_case_id: incident_case_id.into(),
            decision: decision.into(),
            justification: justification.into(),
            approved_by,
            generated_at: generated_at.into(),
        };
        value.validate()?;
        Ok(value)
    }

    pub fn validate(&self) -> ContinuityValidationResult {
        require_schema_version(&self.schema_version, Self::SCHEMA_VERSION)?;
        require_non_empty(&self.containment_decision_id, "containment_decision_id")?;
        require_non_empty(&self.incident_case_id, "incident_case_id")?;
        require_non_empty(&self.decision, "decision")?;
        require_non_empty(&self.justification, "justification")?;
        require_non_empty_slice(&self.approved_by, "approved_by")?;
        require_non_empty(&self.generated_at, "generated_at")?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ForensicFreezeV1 {
    pub schema_version: String,
    pub forensic_freeze_id: String,
    pub incident_case_id: String,
    pub frozen_surfaces: Vec<String>,
    pub retention_hold_until: String,
    pub initiated_at: String,
}

impl ForensicFreezeV1 {
    pub const SCHEMA_VERSION: &'static str = "ForensicFreezeV1";

    pub fn new(
        forensic_freeze_id: impl Into<String>,
        incident_case_id: impl Into<String>,
        frozen_surfaces: Vec<String>,
        retention_hold_until: impl Into<String>,
        initiated_at: impl Into<String>,
    ) -> Result<Self, ContinuityValidationError> {
        let value = Self {
            schema_version: Self::SCHEMA_VERSION.to_string(),
            forensic_freeze_id: forensic_freeze_id.into(),
            incident_case_id: incident_case_id.into(),
            frozen_surfaces,
            retention_hold_until: retention_hold_until.into(),
            initiated_at: initiated_at.into(),
        };
        value.validate()?;
        Ok(value)
    }

    pub fn validate(&self) -> ContinuityValidationResult {
        require_schema_version(&self.schema_version, Self::SCHEMA_VERSION)?;
        require_non_empty(&self.forensic_freeze_id, "forensic_freeze_id")?;
        require_non_empty(&self.incident_case_id, "incident_case_id")?;
        require_non_empty_slice(&self.frozen_surfaces, "frozen_surfaces")?;
        require_non_empty(&self.retention_hold_until, "retention_hold_until")?;
        require_non_empty(&self.initiated_at, "initiated_at")?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ContinuityExceptionV1 {
    pub schema_version: String,
    pub continuity_exception_id: String,
    pub incident_case_id: String,
    pub exception_kind: String,
    pub reason: String,
    pub expires_at: String,
    pub post_hoc_review_required: bool,
}

impl ContinuityExceptionV1 {
    pub const SCHEMA_VERSION: &'static str = "ContinuityExceptionV1";

    pub fn new(
        continuity_exception_id: impl Into<String>,
        incident_case_id: impl Into<String>,
        exception_kind: impl Into<String>,
        reason: impl Into<String>,
        expires_at: impl Into<String>,
        post_hoc_review_required: bool,
    ) -> Result<Self, ContinuityValidationError> {
        let value = Self {
            schema_version: Self::SCHEMA_VERSION.to_string(),
            continuity_exception_id: continuity_exception_id.into(),
            incident_case_id: incident_case_id.into(),
            exception_kind: exception_kind.into(),
            reason: reason.into(),
            expires_at: expires_at.into(),
            post_hoc_review_required,
        };
        value.validate()?;
        Ok(value)
    }

    pub fn validate(&self) -> ContinuityValidationResult {
        require_schema_version(&self.schema_version, Self::SCHEMA_VERSION)?;
        require_non_empty(&self.continuity_exception_id, "continuity_exception_id")?;
        require_non_empty(&self.incident_case_id, "incident_case_id")?;
        require_non_empty(&self.exception_kind, "exception_kind")?;
        require_non_empty(&self.reason, "reason")?;
        require_non_empty(&self.expires_at, "expires_at")?;
        Ok(())
    }
}
