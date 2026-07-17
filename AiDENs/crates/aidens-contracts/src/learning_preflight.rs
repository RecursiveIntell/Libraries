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
    pub cea_store_identity: String,
    pub receipt_root_owner: String,
}
