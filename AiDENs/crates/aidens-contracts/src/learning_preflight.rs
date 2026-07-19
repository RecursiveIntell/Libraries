use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LearningPreflightReceiptV1 {
    pub schema: String,
    pub material_id: String,
    pub fixture_tree_digest: String,
    pub patch_digest: String,
    pub patch_policy_digest: String,
    pub permit_grant_id: String,
    pub permit_use_id: String,
    pub permit_scope_digest: String,
    pub image: String,
    pub backend_limits_digest: String,
    pub run_id: String,
    pub attempt_id: String,
    pub trial_id: String,
    pub trace_id: String,
    pub requested_recorded_at: String,
    pub cea_store_identity: String,
    pub receipt_root_owner: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub memory_store_owner: String,
}

impl LearningPreflightReceiptV1 {
    pub const SCHEMA: &'static str = "AiDENsLearningPreflightReceiptV1";

    pub fn validate(&self) -> Result<(), &'static str> {
        if self.schema != Self::SCHEMA {
            return Err("learning preflight schema mismatch");
        }
        let required = [
            self.material_id.as_str(),
            self.fixture_tree_digest.as_str(),
            self.patch_digest.as_str(),
            self.patch_policy_digest.as_str(),
            self.permit_grant_id.as_str(),
            self.permit_use_id.as_str(),
            self.permit_scope_digest.as_str(),
            self.image.as_str(),
            self.backend_limits_digest.as_str(),
            self.run_id.as_str(),
            self.attempt_id.as_str(),
            self.trial_id.as_str(),
            self.trace_id.as_str(),
            self.requested_recorded_at.as_str(),
            self.cea_store_identity.as_str(),
            self.receipt_root_owner.as_str(),
        ];
        if required
            .iter()
            .any(|value| value.trim().is_empty() || value.contains("local-process-seq"))
        {
            return Err("learning preflight contains missing or process-local material");
        }
        Ok(())
    }
}

/// Destination-bound preflight contract for terminal Forge V3 publication.
///
/// V1 remains readable for prior runs. V2 embeds it unchanged and adds the
/// canonical Forge destination plus publication namespace required by the
/// terminal lane.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LearningPreflightReceiptV2 {
    pub schema: String,
    pub execution: LearningPreflightReceiptV1,
    pub forge_store_owner: String,
    pub publication_namespace: String,
}

impl LearningPreflightReceiptV2 {
    pub const SCHEMA: &'static str = "AiDENsLearningPreflightReceiptV2";

    pub fn validate(&self) -> Result<(), &'static str> {
        if self.schema != Self::SCHEMA {
            return Err("learning preflight v2 schema mismatch");
        }
        self.execution.validate()?;
        if [
            self.execution.memory_store_owner.as_str(),
            self.forge_store_owner.as_str(),
            self.publication_namespace.as_str(),
        ]
        .iter()
        .any(|value| value.trim().is_empty() || value.contains("local-process-seq"))
        {
            return Err("learning preflight v2 contains missing destination material");
        }
        Ok(())
    }
}
