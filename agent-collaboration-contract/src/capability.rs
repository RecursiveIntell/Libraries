use crate::{validate_nonempty, validate_time_window, validate_version, ContractError};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use stack_ids::{AgentId, CapabilityManifestId, ContentDigest};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CapabilityManifestV1 {
    pub schema_version: String,
    pub manifest_id: CapabilityManifestId,
    pub agent_id: AgentId,
    pub capabilities: Vec<String>,
    pub max_task_bytes: u64,
    pub max_artifact_bytes: u64,
    pub max_execution_secs: u64,
    pub issued_at: String,
    pub expires_at: String,
    pub manifest_digest: ContentDigest,
}

impl CapabilityManifestV1 {
    pub fn validate(&self) -> Result<(), ContractError> {
        validate_version(&self.schema_version)?;
        validate_nonempty(self.manifest_id.as_str(), "manifest_id")?;
        validate_nonempty(self.agent_id.as_str(), "agent_id")?;
        if self.capabilities.is_empty() {
            return Err(ContractError::EmptyField("capabilities"));
        }
        if self.capabilities.len() > 64 {
            return Err(ContractError::ResourceLimit {
                field: "capabilities",
                value: self.capabilities.len(),
                limit: 64,
            });
        }
        for capability in &self.capabilities {
            validate_nonempty(capability, "capability")?;
            if capability.len() > 128 {
                return Err(ContractError::ResourceLimit {
                    field: "capability",
                    value: capability.len(),
                    limit: 128,
                });
            }
        }
        if self.max_task_bytes == 0 || self.max_artifact_bytes == 0 || self.max_execution_secs == 0
        {
            return Err(ContractError::ResourceLimit {
                field: "resource_ceiling",
                value: 0,
                limit: 1,
            });
        }
        validate_time_window(&self.issued_at, &self.expires_at)
    }
}
