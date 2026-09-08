use crate::artifact::ArtifactManifestV1;
use crate::task::TaskStatusV1;
use crate::{validate_nonempty, validate_timestamp, validate_version, ContractError};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use stack_ids::{
    AgentId, AttemptId, ContentDigest, DeliveryId, ExecutionPermitId, PolicyDecisionId, TaskId,
    TrialId,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DeliveryReceiptV1 {
    pub schema_version: String,
    pub delivery_id: DeliveryId,
    pub task_id: TaskId,
    pub worker_agent_id: AgentId,
    pub payload_digest: ContentDigest,
    pub delivered_at: String,
    pub acknowledged_at: Option<String>,
}

impl DeliveryReceiptV1 {
    pub fn validate(&self) -> Result<(), ContractError> {
        validate_version(&self.schema_version)?;
        validate_nonempty(self.delivery_id.as_str(), "delivery_id")?;
        validate_nonempty(self.task_id.as_str(), "task_id")?;
        validate_nonempty(self.worker_agent_id.as_str(), "worker_agent_id")?;
        validate_timestamp(&self.delivered_at, "delivered_at")?;
        if let Some(ack) = &self.acknowledged_at {
            let delivered = validate_timestamp(&self.delivered_at, "delivered_at")?;
            let acknowledged = validate_timestamp(ack, "acknowledged_at")?;
            if acknowledged < delivered {
                return Err(ContractError::InvalidTimeWindow {
                    start: self.delivered_at.clone(),
                    end: ack.clone(),
                });
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AuthorityDecisionRefV1 {
    pub policy_decision_id: Option<PolicyDecisionId>,
    pub execution_permit_id: Option<ExecutionPermitId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TaskReceiptV1 {
    pub schema_version: String,
    pub task_id: TaskId,
    pub attempt_id: AttemptId,
    pub trial_id: TrialId,
    pub status: TaskStatusV1,
    pub request_digest: ContentDigest,
    pub output_manifest: Option<ArtifactManifestV1>,
    pub output_manifest_digest: Option<ContentDigest>,
    pub authority: Option<AuthorityDecisionRefV1>,
    pub completed_at: String,
    pub recorded_at: String,
}

impl TaskReceiptV1 {
    pub fn validate(&self) -> Result<(), ContractError> {
        validate_version(&self.schema_version)?;
        validate_nonempty(self.task_id.as_str(), "task_id")?;
        validate_nonempty(self.attempt_id.as_str(), "attempt_id")?;
        validate_nonempty(self.trial_id.as_str(), "trial_id")?;
        validate_timestamp(&self.completed_at, "completed_at")?;
        validate_timestamp(&self.recorded_at, "recorded_at")?;
        if self.status == TaskStatusV1::Completed
            && (self.output_manifest.is_none() || self.output_manifest_digest.is_none())
        {
            return Err(ContractError::MissingLineage(
                "completed receipt requires output manifest and digest",
            ));
        }
        if let Some(manifest) = &self.output_manifest {
            manifest.validate()?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ConflictRecordV1 {
    pub schema_version: String,
    pub conflict_id: stack_ids::ConflictRecordId,
    pub task_id: TaskId,
    pub authority_agent_id: AgentId,
    pub observed_event_digest: ContentDigest,
    pub expected_event_digest: Option<ContentDigest>,
    pub reason_code: String,
    pub recorded_at: String,
}

impl ConflictRecordV1 {
    pub fn validate(&self) -> Result<(), ContractError> {
        validate_version(&self.schema_version)?;
        validate_nonempty(self.conflict_id.as_str(), "conflict_id")?;
        validate_nonempty(self.task_id.as_str(), "task_id")?;
        validate_nonempty(self.authority_agent_id.as_str(), "authority_agent_id")?;
        validate_nonempty(&self.reason_code, "reason_code")?;
        validate_timestamp(&self.recorded_at, "recorded_at").map(|_| ())
    }
}
