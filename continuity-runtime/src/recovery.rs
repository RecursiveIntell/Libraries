use crate::error::{ContinuityValidationError, ContinuityValidationResult};
use crate::validation::{require_non_empty, require_non_empty_slice, require_schema_version};
use crate::vocab::RecoveryReplayStateV1;
use crate::vocab::ResilienceExerciseKindV1;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct RecoveryPlanV1 {
    pub schema_version: String,
    pub recovery_plan_id: String,
    pub incident_case_id: String,
    pub recovery_steps: Vec<String>,
    pub success_criteria: Vec<String>,
    pub owner_ref: String,
    pub approved_at: String,
}

impl RecoveryPlanV1 {
    pub const SCHEMA_VERSION: &'static str = "RecoveryPlanV1";

    pub fn new(
        recovery_plan_id: impl Into<String>,
        incident_case_id: impl Into<String>,
        recovery_steps: Vec<String>,
        success_criteria: Vec<String>,
        owner_ref: impl Into<String>,
        approved_at: impl Into<String>,
    ) -> Result<Self, ContinuityValidationError> {
        let value = Self {
            schema_version: Self::SCHEMA_VERSION.to_string(),
            recovery_plan_id: recovery_plan_id.into(),
            incident_case_id: incident_case_id.into(),
            recovery_steps,
            success_criteria,
            owner_ref: owner_ref.into(),
            approved_at: approved_at.into(),
        };
        value.validate()?;
        Ok(value)
    }

    pub fn validate(&self) -> ContinuityValidationResult {
        require_schema_version(&self.schema_version, Self::SCHEMA_VERSION)?;
        require_non_empty(&self.recovery_plan_id, "recovery_plan_id")?;
        require_non_empty(&self.incident_case_id, "incident_case_id")?;
        require_non_empty_slice(&self.recovery_steps, "recovery_steps")?;
        require_non_empty_slice(&self.success_criteria, "success_criteria")?;
        require_non_empty(&self.owner_ref, "owner_ref")?;
        require_non_empty(&self.approved_at, "approved_at")?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct RecoveryReplaySliceV1 {
    pub schema_version: String,
    pub recovery_replay_slice_id: String,
    pub incident_case_id: String,
    pub artifact_refs: Vec<String>,
    pub time_window: String,
    pub replay_state: RecoveryReplayStateV1,
    pub gaps: Vec<String>,
}

impl RecoveryReplaySliceV1 {
    pub const SCHEMA_VERSION: &'static str = "RecoveryReplaySliceV1";

    pub fn new(
        recovery_replay_slice_id: impl Into<String>,
        incident_case_id: impl Into<String>,
        artifact_refs: Vec<String>,
        time_window: impl Into<String>,
        replay_state: RecoveryReplayStateV1,
        gaps: Vec<String>,
    ) -> Result<Self, ContinuityValidationError> {
        let value = Self {
            schema_version: Self::SCHEMA_VERSION.to_string(),
            recovery_replay_slice_id: recovery_replay_slice_id.into(),
            incident_case_id: incident_case_id.into(),
            artifact_refs,
            time_window: time_window.into(),
            replay_state,
            gaps,
        };
        value.validate()?;
        Ok(value)
    }

    pub fn validate(&self) -> ContinuityValidationResult {
        require_schema_version(&self.schema_version, Self::SCHEMA_VERSION)?;
        require_non_empty(&self.recovery_replay_slice_id, "recovery_replay_slice_id")?;
        require_non_empty(&self.incident_case_id, "incident_case_id")?;
        require_non_empty_slice(&self.artifact_refs, "artifact_refs")?;
        require_non_empty(&self.time_window, "time_window")?;
        if matches!(self.replay_state, RecoveryReplayStateV1::Partial) && self.gaps.is_empty() {
            return Err(ContinuityValidationError::InvalidState("gaps"));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct PostmortemBundleV1 {
    pub schema_version: String,
    pub postmortem_bundle_id: String,
    pub incident_case_id: String,
    pub root_cause_summary: String,
    pub corrective_actions: Vec<String>,
    pub owner_ref: String,
    pub published_at: String,
}

impl PostmortemBundleV1 {
    pub const SCHEMA_VERSION: &'static str = "PostmortemBundleV1";

    pub fn new(
        postmortem_bundle_id: impl Into<String>,
        incident_case_id: impl Into<String>,
        root_cause_summary: impl Into<String>,
        corrective_actions: Vec<String>,
        owner_ref: impl Into<String>,
        published_at: impl Into<String>,
    ) -> Result<Self, ContinuityValidationError> {
        let value = Self {
            schema_version: Self::SCHEMA_VERSION.to_string(),
            postmortem_bundle_id: postmortem_bundle_id.into(),
            incident_case_id: incident_case_id.into(),
            root_cause_summary: root_cause_summary.into(),
            corrective_actions,
            owner_ref: owner_ref.into(),
            published_at: published_at.into(),
        };
        value.validate()?;
        Ok(value)
    }

    pub fn validate(&self) -> ContinuityValidationResult {
        require_schema_version(&self.schema_version, Self::SCHEMA_VERSION)?;
        require_non_empty(&self.postmortem_bundle_id, "postmortem_bundle_id")?;
        require_non_empty(&self.incident_case_id, "incident_case_id")?;
        require_non_empty(&self.root_cause_summary, "root_cause_summary")?;
        require_non_empty_slice(&self.corrective_actions, "corrective_actions")?;
        require_non_empty(&self.owner_ref, "owner_ref")?;
        require_non_empty(&self.published_at, "published_at")?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ResilienceExerciseV1 {
    pub schema_version: String,
    pub resilience_exercise_id: String,
    pub service_level_profile_id: String,
    pub exercise_kind: ResilienceExerciseKindV1,
    pub scenario: String,
    pub completed_at: String,
    pub follow_up_actions: Vec<String>,
}

impl ResilienceExerciseV1 {
    pub const SCHEMA_VERSION: &'static str = "ResilienceExerciseV1";

    pub fn new(
        resilience_exercise_id: impl Into<String>,
        service_level_profile_id: impl Into<String>,
        exercise_kind: ResilienceExerciseKindV1,
        scenario: impl Into<String>,
        completed_at: impl Into<String>,
        follow_up_actions: Vec<String>,
    ) -> Result<Self, ContinuityValidationError> {
        let value = Self {
            schema_version: Self::SCHEMA_VERSION.to_string(),
            resilience_exercise_id: resilience_exercise_id.into(),
            service_level_profile_id: service_level_profile_id.into(),
            exercise_kind,
            scenario: scenario.into(),
            completed_at: completed_at.into(),
            follow_up_actions,
        };
        value.validate()?;
        Ok(value)
    }

    pub fn validate(&self) -> ContinuityValidationResult {
        require_schema_version(&self.schema_version, Self::SCHEMA_VERSION)?;
        require_non_empty(&self.resilience_exercise_id, "resilience_exercise_id")?;
        require_non_empty(&self.service_level_profile_id, "service_level_profile_id")?;
        require_non_empty(&self.scenario, "scenario")?;
        require_non_empty(&self.completed_at, "completed_at")?;
        require_non_empty_slice(&self.follow_up_actions, "follow_up_actions")?;
        Ok(())
    }
}
