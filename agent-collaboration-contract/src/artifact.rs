use crate::{validate_nonempty, validate_timestamp, validate_version, ContractError};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use stack_ids::{AgentId, ArtifactId, ArtifactManifestId, ContentDigest, TaskId};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ArtifactRefV1 {
    pub artifact_id: ArtifactId,
    pub digest: ContentDigest,
    pub size_bytes: u64,
    pub media_type: String,
}

impl ArtifactRefV1 {
    pub fn validate(&self) -> Result<(), ContractError> {
        validate_nonempty(self.artifact_id.as_str(), "artifact_id")?;
        if self.size_bytes == 0 {
            return Err(ContractError::ResourceLimit {
                field: "size_bytes",
                value: 0,
                limit: 1,
            });
        }
        validate_nonempty(&self.media_type, "media_type")?;
        if self.media_type.len() > 128 {
            return Err(ContractError::ResourceLimit {
                field: "media_type",
                value: self.media_type.len(),
                limit: 128,
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ArtifactManifestV1 {
    pub schema_version: String,
    pub manifest_id: ArtifactManifestId,
    pub task_id: TaskId,
    pub producer_agent_id: AgentId,
    pub artifacts: Vec<ArtifactRefV1>,
    pub manifest_digest: ContentDigest,
    pub created_at: String,
}

impl ArtifactManifestV1 {
    pub fn validate(&self) -> Result<(), ContractError> {
        validate_version(&self.schema_version)?;
        validate_nonempty(self.manifest_id.as_str(), "manifest_id")?;
        validate_nonempty(self.task_id.as_str(), "task_id")?;
        validate_nonempty(self.producer_agent_id.as_str(), "producer_agent_id")?;
        if self.artifacts.is_empty() {
            return Err(ContractError::EmptyField("artifacts"));
        }
        if self.artifacts.len() > 64 {
            return Err(ContractError::ResourceLimit {
                field: "artifacts",
                value: self.artifacts.len(),
                limit: 64,
            });
        }
        for artifact in &self.artifacts {
            artifact.validate()?;
        }
        validate_timestamp(&self.created_at, "created_at").map(|_| ())
    }
}
