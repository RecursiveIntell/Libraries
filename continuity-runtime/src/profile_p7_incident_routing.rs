use crate::error::{ContinuityValidationError, ContinuityValidationResult};
use crate::validation::{require_non_empty, require_non_empty_slice, require_schema_version};
use crate::vocab::IncidentSeverityV1;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct IncidentClassRuleV1 {
    pub incident_class: String,
    pub default_severity: IncidentSeverityV1,
}

impl IncidentClassRuleV1 {
    pub fn validate(&self) -> ContinuityValidationResult {
        require_non_empty(&self.incident_class, "incident_class")
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct IncidentRouteRuleV1 {
    pub incident_class: String,
    pub route: String,
}

impl IncidentRouteRuleV1 {
    pub fn validate(&self) -> ContinuityValidationResult {
        require_non_empty(&self.incident_class, "incident_class")?;
        require_non_empty(&self.route, "route")
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct SeverityRuleV1 {
    pub condition: String,
    pub severity: IncidentSeverityV1,
}

impl SeverityRuleV1 {
    pub fn validate(&self) -> ContinuityValidationResult {
        require_non_empty(&self.condition, "condition")
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct SeverityResponseClockV1 {
    pub severity: IncidentSeverityV1,
    pub minutes: i64,
}

impl SeverityResponseClockV1 {
    pub fn validate(&self) -> ContinuityValidationResult {
        if self.minutes <= 0 {
            return Err(ContinuityValidationError::InvalidState("minutes"));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct SeverityPostmortemClockV1 {
    pub severity: IncidentSeverityV1,
    pub hours: i64,
}

impl SeverityPostmortemClockV1 {
    pub fn validate(&self) -> ContinuityValidationResult {
        if self.hours <= 0 {
            return Err(ContinuityValidationError::InvalidState("hours"));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct IncidentTaxonomyV1 {
    pub schema_version: String,
    pub incident_taxonomy_id: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub incident_classes: Vec<IncidentClassRuleV1>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub default_routes: Vec<IncidentRouteRuleV1>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub required_artifact_families: Vec<String>,
}

impl IncidentTaxonomyV1 {
    pub const SCHEMA_VERSION: &'static str = "IncidentTaxonomyV1";

    pub fn new(
        incident_taxonomy_id: impl Into<String>,
        incident_classes: Vec<IncidentClassRuleV1>,
        default_routes: Vec<IncidentRouteRuleV1>,
        required_artifact_families: Vec<String>,
    ) -> Result<Self, ContinuityValidationError> {
        let value = Self {
            schema_version: Self::SCHEMA_VERSION.to_string(),
            incident_taxonomy_id: incident_taxonomy_id.into(),
            incident_classes,
            default_routes,
            required_artifact_families,
        };
        value.validate()?;
        Ok(value)
    }

    pub fn validate(&self) -> ContinuityValidationResult {
        require_schema_version(&self.schema_version, Self::SCHEMA_VERSION)?;
        require_non_empty(&self.incident_taxonomy_id, "incident_taxonomy_id")?;
        require_non_empty_slice(&self.incident_classes, "incident_classes")?;
        require_non_empty_slice(&self.default_routes, "default_routes")?;
        require_non_empty_slice(
            &self.required_artifact_families,
            "required_artifact_families",
        )?;
        for rule in &self.incident_classes {
            rule.validate()?;
        }
        for route in &self.default_routes {
            route.validate()?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct SeverityMatrixV1 {
    pub schema_version: String,
    pub severity_matrix_id: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub severity_rules: Vec<SeverityRuleV1>,
    pub customer_impact_rubric: String,
    pub internal_impact_rubric: String,
    pub override_rule: String,
}

impl SeverityMatrixV1 {
    pub const SCHEMA_VERSION: &'static str = "SeverityMatrixV1";

    pub fn new(
        severity_matrix_id: impl Into<String>,
        severity_rules: Vec<SeverityRuleV1>,
        customer_impact_rubric: impl Into<String>,
        internal_impact_rubric: impl Into<String>,
        override_rule: impl Into<String>,
    ) -> Result<Self, ContinuityValidationError> {
        let value = Self {
            schema_version: Self::SCHEMA_VERSION.to_string(),
            severity_matrix_id: severity_matrix_id.into(),
            severity_rules,
            customer_impact_rubric: customer_impact_rubric.into(),
            internal_impact_rubric: internal_impact_rubric.into(),
            override_rule: override_rule.into(),
        };
        value.validate()?;
        Ok(value)
    }

    pub fn validate(&self) -> ContinuityValidationResult {
        require_schema_version(&self.schema_version, Self::SCHEMA_VERSION)?;
        require_non_empty(&self.severity_matrix_id, "severity_matrix_id")?;
        require_non_empty_slice(&self.severity_rules, "severity_rules")?;
        require_non_empty(&self.customer_impact_rubric, "customer_impact_rubric")?;
        require_non_empty(&self.internal_impact_rubric, "internal_impact_rubric")?;
        require_non_empty(&self.override_rule, "override_rule")?;
        for rule in &self.severity_rules {
            rule.validate()?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct PagerRouteProfileV1 {
    pub schema_version: String,
    pub pager_route_profile_id: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub rotation_refs: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub handoff_rules: Vec<String>,
    pub ack_timeout_minutes: i64,
    pub max_levels: i64,
}

impl PagerRouteProfileV1 {
    pub const SCHEMA_VERSION: &'static str = "PagerRouteProfileV1";

    pub fn new(
        pager_route_profile_id: impl Into<String>,
        rotation_refs: Vec<String>,
        handoff_rules: Vec<String>,
        ack_timeout_minutes: i64,
        max_levels: i64,
    ) -> Result<Self, ContinuityValidationError> {
        let value = Self {
            schema_version: Self::SCHEMA_VERSION.to_string(),
            pager_route_profile_id: pager_route_profile_id.into(),
            rotation_refs,
            handoff_rules,
            ack_timeout_minutes,
            max_levels,
        };
        value.validate()?;
        Ok(value)
    }

    pub fn validate(&self) -> ContinuityValidationResult {
        require_schema_version(&self.schema_version, Self::SCHEMA_VERSION)?;
        require_non_empty(&self.pager_route_profile_id, "pager_route_profile_id")?;
        require_non_empty_slice(&self.rotation_refs, "rotation_refs")?;
        require_non_empty_slice(&self.handoff_rules, "handoff_rules")?;
        if self.ack_timeout_minutes <= 0 {
            return Err(ContinuityValidationError::InvalidState(
                "ack_timeout_minutes",
            ));
        }
        if self.max_levels <= 0 {
            return Err(ContinuityValidationError::InvalidState("max_levels"));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct EscalationClockPolicyV1 {
    pub schema_version: String,
    pub escalation_clock_policy_id: String,
    pub severity_matrix_id: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub response_clock_minutes: Vec<SeverityResponseClockV1>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub postmortem_clock_hours: Vec<SeverityPostmortemClockV1>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub pause_rules: Vec<String>,
    pub exception_path: String,
}

impl EscalationClockPolicyV1 {
    pub const SCHEMA_VERSION: &'static str = "EscalationClockPolicyV1";

    pub fn new(
        escalation_clock_policy_id: impl Into<String>,
        severity_matrix_id: impl Into<String>,
        response_clock_minutes: Vec<SeverityResponseClockV1>,
        postmortem_clock_hours: Vec<SeverityPostmortemClockV1>,
        pause_rules: Vec<String>,
        exception_path: impl Into<String>,
    ) -> Result<Self, ContinuityValidationError> {
        let value = Self {
            schema_version: Self::SCHEMA_VERSION.to_string(),
            escalation_clock_policy_id: escalation_clock_policy_id.into(),
            severity_matrix_id: severity_matrix_id.into(),
            response_clock_minutes,
            postmortem_clock_hours,
            pause_rules,
            exception_path: exception_path.into(),
        };
        value.validate()?;
        Ok(value)
    }

    pub fn validate(&self) -> ContinuityValidationResult {
        require_schema_version(&self.schema_version, Self::SCHEMA_VERSION)?;
        require_non_empty(
            &self.escalation_clock_policy_id,
            "escalation_clock_policy_id",
        )?;
        require_non_empty(&self.severity_matrix_id, "severity_matrix_id")?;
        require_non_empty_slice(&self.response_clock_minutes, "response_clock_minutes")?;
        require_non_empty_slice(&self.postmortem_clock_hours, "postmortem_clock_hours")?;
        require_non_empty(&self.exception_path, "exception_path")?;
        for clock in &self.response_clock_minutes {
            clock.validate()?;
        }
        for clock in &self.postmortem_clock_hours {
            clock.validate()?;
        }
        Ok(())
    }
}
