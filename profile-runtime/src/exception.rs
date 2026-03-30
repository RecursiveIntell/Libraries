use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use stack_ids::{ApplicabilityContextId, ProfileExceptionBundleId};

pub const PROFILE_EXCEPTION_BUNDLE_V1_SCHEMA: &str = "profile_exception_bundle_v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ProfileExceptionBundleV1 {
    pub schema_version: String,
    pub profile_exception_bundle_id: ProfileExceptionBundleId,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub affected_context_refs: Vec<ApplicabilityContextId>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub affected_profile_refs: Vec<String>,
    pub exception_class: String,
    pub reason: String,
    pub scope: String,
    pub starts_at: String,
    pub expires_at: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub approval_refs: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub residual_obligations: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub added_monitoring_obligations: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub added_post_hoc_review_obligations: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub replay_expectations: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub cleanup_obligations: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub compensation_obligations: Vec<String>,
    pub recorded_at: String,
}

impl ProfileExceptionBundleV1 {
    /// Returns true when the exception bundle is active for the requested validity timestamp.
    pub fn is_active_as_of(&self, valid_as_of: &str) -> bool {
        self.starts_at.as_str() <= valid_as_of && valid_as_of <= self.expires_at.as_str()
    }
}
