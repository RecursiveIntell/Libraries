use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use stack_ids::{
    AccessPurposeMatrixId, ApplicabilityContextId, ApprovalMatrixId, AuditExtractionPolicyId,
    ConflictClassCatalogId, ContinuityPolicyProfileId, CrossBoundaryTransferClassId,
    DelegationMatrixId, DelegationPolicyProfileId, EffectPolicyProfileId, EscalationClockPolicyId,
    EvidenceCollectionPlanId, HazardLibraryId, HazardScenarioId, IncidentTaxonomyId,
    LocalityExceptionId, MitigationPlaybookId, MonitorCatalogId, PagerRouteProfileId,
    PrivacyRetentionProfileId, ProfileExceptionBundleId, ProfileSetId, RecertificationScheduleId,
    RedactionRuleSetId, RegulatoryRegimeProfileId, ReleasePolicyProfileId, RequirementControlMapId,
    ResidencyPolicyProfileId, RoleCatalogId, SeverityMatrixId, TenantBoundaryProfileId,
    VendorCertificationAdapterId, VendorEvidenceTranslationId, VendorRevocationHandlingId,
    VendorTrustRootBindingId,
};

pub const PROFILE_SET_V1_SCHEMA: &str = "profile_set_v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, Default)]
pub struct ProfileRefGroupV1 {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub privacy_retention_profile_id: Option<PrivacyRetentionProfileId>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub redaction_rule_set_ids: Vec<RedactionRuleSetId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub access_purpose_matrix_id: Option<AccessPurposeMatrixId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub audit_extraction_policy_id: Option<AuditExtractionPolicyId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub residency_policy_profile_id: Option<ResidencyPolicyProfileId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tenant_boundary_profile_id: Option<TenantBoundaryProfileId>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub cross_boundary_transfer_class_ids: Vec<CrossBoundaryTransferClassId>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub locality_exception_ids: Vec<LocalityExceptionId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role_catalog_id: Option<RoleCatalogId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub delegation_matrix_id: Option<DelegationMatrixId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub approval_matrix_id: Option<ApprovalMatrixId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub conflict_class_catalog_id: Option<ConflictClassCatalogId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub regulatory_regime_profile_id: Option<RegulatoryRegimeProfileId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub requirement_control_map_id: Option<RequirementControlMapId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub evidence_collection_plan_id: Option<EvidenceCollectionPlanId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recertification_schedule_id: Option<RecertificationScheduleId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hazard_library_id: Option<HazardLibraryId>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub hazard_scenario_ids: Vec<HazardScenarioId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub monitor_catalog_id: Option<MonitorCatalogId>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub mitigation_playbook_ids: Vec<MitigationPlaybookId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vendor_certification_adapter_id: Option<VendorCertificationAdapterId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vendor_evidence_translation_id: Option<VendorEvidenceTranslationId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vendor_trust_root_binding_id: Option<VendorTrustRootBindingId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vendor_revocation_handling_id: Option<VendorRevocationHandlingId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub incident_taxonomy_id: Option<IncidentTaxonomyId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub severity_matrix_id: Option<SeverityMatrixId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pager_route_profile_id: Option<PagerRouteProfileId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub escalation_clock_policy_id: Option<EscalationClockPolicyId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub effect_policy_profile_id: Option<EffectPolicyProfileId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub delegation_policy_profile_id: Option<DelegationPolicyProfileId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub release_policy_profile_id: Option<ReleasePolicyProfileId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub continuity_policy_profile_id: Option<ContinuityPolicyProfileId>,
}

impl ProfileRefGroupV1 {
    /// Returns the deduplicated set of profile references carried by this group.
    pub fn all_profile_refs(&self) -> Vec<String> {
        let mut refs = Vec::new();
        macro_rules! push_opt {
            ($field:expr) => {
                if let Some(value) = &$field {
                    refs.push(value.to_string());
                }
            };
        }
        macro_rules! push_vec {
            ($field:expr) => {
                refs.extend($field.iter().map(ToString::to_string));
            };
        }

        push_opt!(self.privacy_retention_profile_id);
        push_vec!(self.redaction_rule_set_ids);
        push_opt!(self.access_purpose_matrix_id);
        push_opt!(self.audit_extraction_policy_id);
        push_opt!(self.residency_policy_profile_id);
        push_opt!(self.tenant_boundary_profile_id);
        push_vec!(self.cross_boundary_transfer_class_ids);
        push_vec!(self.locality_exception_ids);
        push_opt!(self.role_catalog_id);
        push_opt!(self.delegation_matrix_id);
        push_opt!(self.approval_matrix_id);
        push_opt!(self.conflict_class_catalog_id);
        push_opt!(self.regulatory_regime_profile_id);
        push_opt!(self.requirement_control_map_id);
        push_opt!(self.evidence_collection_plan_id);
        push_opt!(self.recertification_schedule_id);
        push_opt!(self.hazard_library_id);
        push_vec!(self.hazard_scenario_ids);
        push_opt!(self.monitor_catalog_id);
        push_vec!(self.mitigation_playbook_ids);
        push_opt!(self.vendor_certification_adapter_id);
        push_opt!(self.vendor_evidence_translation_id);
        push_opt!(self.vendor_trust_root_binding_id);
        push_opt!(self.vendor_revocation_handling_id);
        push_opt!(self.incident_taxonomy_id);
        push_opt!(self.severity_matrix_id);
        push_opt!(self.pager_route_profile_id);
        push_opt!(self.escalation_clock_policy_id);
        push_opt!(self.effect_policy_profile_id);
        push_opt!(self.delegation_policy_profile_id);
        push_opt!(self.release_policy_profile_id);
        push_opt!(self.continuity_policy_profile_id);

        refs.sort();
        refs.dedup();
        refs
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ProfileOmissionV1 {
    pub family: String,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ProfileSetV1 {
    pub schema_version: String,
    pub profile_set_id: ProfileSetId,
    pub applicability_context_ref: ApplicabilityContextId,
    pub profile_refs: ProfileRefGroupV1,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub source_admission_refs: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub omissions: Vec<ProfileOmissionV1>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub active_exception_refs: Vec<ProfileExceptionBundleId>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub normalized_profile_order: Vec<String>,
}

impl ProfileSetV1 {
    /// Creates a profile set and captures its initial normalized profile ordering.
    pub fn new(
        applicability_context_ref: ApplicabilityContextId,
        profile_refs: ProfileRefGroupV1,
        source_admission_refs: Vec<String>,
    ) -> Self {
        let normalized_profile_order = profile_refs.all_profile_refs();
        Self {
            schema_version: PROFILE_SET_V1_SCHEMA.into(),
            profile_set_id: ProfileSetId::generate(),
            applicability_context_ref,
            profile_refs,
            source_admission_refs,
            omissions: Vec::new(),
            active_exception_refs: Vec::new(),
            normalized_profile_order,
        }
    }

    /// Recomputes the normalized profile ordering after in-memory edits.
    pub fn refresh_normalized_profile_order(&mut self) {
        self.normalized_profile_order = self.profile_refs.all_profile_refs();
    }

    /// Returns the deduplicated profile references visible through this set.
    pub fn all_profile_refs(&self) -> Vec<String> {
        self.profile_refs.all_profile_refs()
    }
}
