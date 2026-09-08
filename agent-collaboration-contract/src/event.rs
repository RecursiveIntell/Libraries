use crate::task::TaskStatusV1;
use crate::{validate_nonempty, validate_timestamp, validate_version, ContractError};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use stack_ids::{AgentId, AttemptId, ContentDigest, TaskEventId, TaskId, TrialId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum TaskEventKindV1 {
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TaskEventV1 {
    pub schema_version: String,
    pub event_id: TaskEventId,
    pub task_id: TaskId,
    pub owner_agent_id: AgentId,
    pub kind: TaskEventKindV1,
    pub status: TaskStatusV1,
    pub attempt_id: Option<AttemptId>,
    pub trial_id: Option<TrialId>,
    pub occurred_at: String,
    pub recorded_at: String,
    pub previous_event_digest: Option<ContentDigest>,
    pub reason_code: Option<String>,
}

impl TaskEventV1 {
    pub fn validate(&self) -> Result<(), ContractError> {
        validate_version(&self.schema_version)?;
        validate_nonempty(self.event_id.as_str(), "event_id")?;
        validate_nonempty(self.task_id.as_str(), "task_id")?;
        validate_nonempty(self.owner_agent_id.as_str(), "owner_agent_id")?;
        validate_timestamp(&self.occurred_at, "occurred_at")?;
        validate_timestamp(&self.recorded_at, "recorded_at")?;
        if let Some(reason) = &self.reason_code {
            validate_nonempty(reason, "reason_code")?;
        }
        if self.status == TaskStatusV1::Running
            && (self.attempt_id.is_none() || self.trial_id.is_none())
        {
            return Err(ContractError::MissingLineage(
                "running event requires attempt and trial",
            ));
        }
        Ok(())
    }
}
