use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use stack_ids::{
    ApplicabilityContextId, ContinuityPolicyProfileId, DelegationPolicyProfileId,
    DeploymentProfileId, EffectPolicyProfileId, ProfileExceptionBundleId, ReleasePolicyProfileId,
    ScopeKey,
};

pub const APPLICABILITY_CONTEXT_V1_SCHEMA: &str = "applicability_context_v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ApplicabilityContextV1 {
    pub schema_version: String,
    pub applicability_context_id: ApplicabilityContextId,
    pub namespace: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scope_key: Option<ScopeKey>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tenant_ref: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tenancy_class: Option<String>,
    pub valid_as_of: String,
    pub recorded_as_of: String,
    pub target_surface_kind: String,
    pub actor_class: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub role_refs: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deployment_profile_ref: Option<DeploymentProfileId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub effect_policy_profile_ref: Option<EffectPolicyProfileId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub delegation_policy_profile_ref: Option<DelegationPolicyProfileId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub release_policy_profile_ref: Option<ReleasePolicyProfileId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub continuity_policy_profile_ref: Option<ContinuityPolicyProfileId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub effect_class: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub incident_mode: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub treaty_or_disclosure_class: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub execution_context_ref: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub admitted_exception_refs: Vec<ProfileExceptionBundleId>,
}

impl ApplicabilityContextV1 {
    #[allow(clippy::too_many_arguments)]
    /// Creates the applicability context that anchors profile selection for one evaluation scope.
    pub fn new(
        namespace: impl Into<String>,
        scope_key: Option<ScopeKey>,
        valid_as_of: impl Into<String>,
        recorded_as_of: impl Into<String>,
        target_surface_kind: impl Into<String>,
        actor_class: impl Into<String>,
        role_refs: Vec<String>,
        deployment_profile_ref: Option<DeploymentProfileId>,
        effect_class: Option<String>,
        incident_mode: Option<String>,
    ) -> Self {
        Self {
            schema_version: APPLICABILITY_CONTEXT_V1_SCHEMA.into(),
            applicability_context_id: ApplicabilityContextId::generate(),
            namespace: namespace.into(),
            scope_key,
            tenant_ref: None,
            tenancy_class: None,
            valid_as_of: valid_as_of.into(),
            recorded_as_of: recorded_as_of.into(),
            target_surface_kind: target_surface_kind.into(),
            actor_class: actor_class.into(),
            role_refs,
            deployment_profile_ref,
            effect_policy_profile_ref: None,
            delegation_policy_profile_ref: None,
            release_policy_profile_ref: None,
            continuity_policy_profile_ref: None,
            effect_class,
            incident_mode,
            treaty_or_disclosure_class: None,
            execution_context_ref: None,
            admitted_exception_refs: Vec::new(),
        }
    }
}
