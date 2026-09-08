use crate::{validate_nonempty, validate_time_window, validate_version, ContractError};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use stack_ids::{AgentId, AttemptId, ContentDigest, LeaseId, TaskId};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct LeaseV1 {
    pub schema_version: String,
    pub lease_id: LeaseId,
    pub task_id: TaskId,
    pub attempt_id: AttemptId,
    pub worker_agent_id: AgentId,
    pub lease_epoch: u64,
    pub fencing_token: ContentDigest,
    pub issued_at: String,
    pub lease_expires_at: String,
}

impl LeaseV1 {
    pub fn validate(&self) -> Result<(), ContractError> {
        validate_version(&self.schema_version)?;
        validate_nonempty(self.lease_id.as_str(), "lease_id")?;
        validate_nonempty(self.task_id.as_str(), "task_id")?;
        validate_nonempty(self.attempt_id.as_str(), "attempt_id")?;
        validate_nonempty(self.worker_agent_id.as_str(), "worker_agent_id")?;
        if self.lease_epoch == 0 {
            return Err(ContractError::StaleLease {
                expected: 1,
                actual: 0,
            });
        }
        validate_time_window(&self.issued_at, &self.lease_expires_at)
    }
}
