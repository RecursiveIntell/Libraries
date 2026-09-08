use crate::artifact::ArtifactRefV1;
use crate::{validate_nonempty, validate_timestamp, validate_version, ContractError};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use stack_ids::{AgentId, AttemptId, ContentDigest, GraphRunId, TaskEventId, TaskId, TrialId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatusV1 {
    Submitted,
    Accepted,
    Rejected,
    Quarantined,
    Leased,
    Running,
    Interrupted,
    CompletionUnknown,
    Completed,
    Failed,
    Cancelled,
    LeaseExpired,
    ReconciledCompleted,
    ReconciledFailed,
}

impl TaskStatusV1 {
    pub fn is_terminal(self) -> bool {
        matches!(
            self,
            Self::Completed | Self::Failed | Self::Cancelled | Self::Rejected | Self::Quarantined
        )
    }

    pub fn can_transition_to(self, next: Self) -> bool {
        use TaskStatusV1::*;
        matches!(
            (self, next),
            (Submitted, Accepted | Rejected | Quarantined)
                | (Accepted, Leased | Cancelled)
                | (Leased, Running | LeaseExpired | Cancelled)
                | (
                    Running,
                    Interrupted | CompletionUnknown | Completed | Failed | Cancelled
                )
                | (Interrupted, Leased | Failed | Cancelled | Quarantined)
                | (
                    CompletionUnknown,
                    ReconciledCompleted | ReconciledFailed | Quarantined
                )
                | (LeaseExpired, Leased | Quarantined)
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ResourceLimitsV1 {
    pub max_input_bytes: u64,
    pub max_output_bytes: u64,
    pub max_runtime_secs: u64,
    pub max_artifacts: u32,
}

impl ResourceLimitsV1 {
    pub fn validate(&self) -> Result<(), ContractError> {
        if self.max_input_bytes == 0
            || self.max_output_bytes == 0
            || self.max_runtime_secs == 0
            || self.max_artifacts == 0
        {
            return Err(ContractError::ResourceLimit {
                field: "resource_limits",
                value: 0,
                limit: 1,
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TaskEnvelopeV1 {
    pub schema_version: String,
    pub task_id: TaskId,
    pub owner_agent_id: AgentId,
    pub producer_agent_id: AgentId,
    pub capability_manifest_id: stack_ids::CapabilityManifestId,
    pub capability_manifest_digest: ContentDigest,
    pub graph_id: GraphRunId,
    pub graph_version: String,
    pub graph_digest: ContentDigest,
    pub idempotency_key: String,
    pub request_digest: ContentDigest,
    pub input_artifacts: Vec<ArtifactRefV1>,
    pub resource_limits: ResourceLimitsV1,
    pub submitted_at: String,
}

impl TaskEnvelopeV1 {
    pub fn validate(&self) -> Result<(), ContractError> {
        validate_version(&self.schema_version)?;
        validate_nonempty(self.task_id.as_str(), "task_id")?;
        validate_nonempty(self.owner_agent_id.as_str(), "owner_agent_id")?;
        validate_nonempty(self.producer_agent_id.as_str(), "producer_agent_id")?;
        validate_nonempty(
            self.capability_manifest_id.as_str(),
            "capability_manifest_id",
        )?;
        validate_nonempty(self.graph_id.as_str(), "graph_id")?;
        validate_nonempty(&self.graph_version, "graph_version")?;
        validate_nonempty(&self.idempotency_key, "idempotency_key")?;
        if self.idempotency_key.len() > 256 {
            return Err(ContractError::ResourceLimit {
                field: "idempotency_key",
                value: self.idempotency_key.len(),
                limit: 256,
            });
        }
        if self.input_artifacts.len() > 64 {
            return Err(ContractError::ResourceLimit {
                field: "input_artifacts",
                value: self.input_artifacts.len(),
                limit: 64,
            });
        }
        for artifact in &self.input_artifacts {
            artifact.validate()?;
        }
        self.resource_limits.validate()?;
        validate_timestamp(&self.submitted_at, "submitted_at").map(|_| ())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum TaskAcceptanceDispositionV1 {
    Accepted,
    Rejected,
    Quarantined,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TaskAcceptanceV1 {
    pub schema_version: String,
    pub task_id: TaskId,
    pub authority_agent_id: AgentId,
    pub request_digest: ContentDigest,
    pub disposition: TaskAcceptanceDispositionV1,
    pub accepted_at: String,
    pub reason: Option<String>,
}

impl TaskAcceptanceV1 {
    pub fn validate(&self) -> Result<(), ContractError> {
        validate_version(&self.schema_version)?;
        validate_nonempty(self.task_id.as_str(), "task_id")?;
        validate_nonempty(self.authority_agent_id.as_str(), "authority_agent_id")?;
        validate_timestamp(&self.accepted_at, "accepted_at").map(|_| ())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TaskAttemptV1 {
    pub schema_version: String,
    pub task_id: TaskId,
    pub attempt_id: AttemptId,
    pub trial_id: TrialId,
    pub graph_run_id: GraphRunId,
    pub worker_agent_id: AgentId,
    pub started_at: String,
    pub finished_at: Option<String>,
}

impl TaskAttemptV1 {
    pub fn validate(&self) -> Result<(), ContractError> {
        validate_version(&self.schema_version)?;
        validate_nonempty(self.task_id.as_str(), "task_id")?;
        validate_nonempty(self.attempt_id.as_str(), "attempt_id")?;
        validate_nonempty(self.trial_id.as_str(), "trial_id")?;
        validate_nonempty(self.graph_run_id.as_str(), "graph_run_id")?;
        validate_nonempty(self.worker_agent_id.as_str(), "worker_agent_id")?;
        validate_timestamp(&self.started_at, "started_at")?;
        if let Some(finished_at) = &self.finished_at {
            let start = validate_timestamp(&self.started_at, "started_at")?;
            let finish = validate_timestamp(finished_at, "finished_at")?;
            if finish < start {
                return Err(ContractError::InvalidTimeWindow {
                    start: self.started_at.clone(),
                    end: finished_at.clone(),
                });
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TaskEventIdentityV1 {
    pub event_id: TaskEventId,
    pub task_id: TaskId,
    pub previous_event_digest: Option<ContentDigest>,
}
