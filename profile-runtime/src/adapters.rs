use assurance_runtime::{
    EvidenceCollectionPlanV1, HazardLibraryV1, HazardScenarioV1, MitigationPlaybookV1,
    MonitorCatalogV1, RecertificationScheduleV1, RegulatoryRegimeProfileV1,
    RequirementControlMapV1,
};
use attestation_exchange::{
    VendorCertificationAdapterV1, VendorEvidenceTranslationV1, VendorRevocationHandlingV1,
    VendorTrustRootBindingV1,
};
use authority_delegation::{
    ApprovalMatrixV1, ConflictClassCatalogV1, DelegationMatrixV1, RoleCatalogV1,
};
use continuity_runtime::{
    EscalationClockPolicyV1, IncidentTaxonomyV1, PagerRouteProfileV1, SeverityMatrixV1,
};
use verification_policy::{
    AccessPurposeMatrixV1, AuditExtractionPolicyV1, ContinuityPolicyProfileV1,
    CrossBoundaryTransferClassV1, DelegationPolicyProfileV1, EffectPolicyProfileV1,
    PrivacyRetentionProfileV1, RedactionRuleSetV1, ReleasePolicyProfileV1,
    ResidencyPolicyProfileV1, TenantBoundaryProfileV1,
};

use crate::compose::ObligationContributionV1;
use crate::rules::{CompiledObligationKindV1, FoldClassV1};

#[allow(clippy::too_many_arguments)]
fn push_strings(
    out: &mut Vec<ObligationContributionV1>,
    family: impl Into<String>,
    key: impl Into<String>,
    kind: CompiledObligationKindV1,
    fold: FoldClassV1,
    values: Vec<String>,
    source_profile_ref: &str,
    explanation: impl Into<String>,
) {
    if values.is_empty() {
        return;
    }
    out.push(ObligationContributionV1 {
        obligation_family: family.into(),
        obligation_key: key.into(),
        output_kind: kind,
        fold_class: fold,
        string_values: values,
        numeric_value: None,
        expiry_at: None,
        blocking: false,
        source_profile_ref: source_profile_ref.to_string(),
        admissible_exception_classes: Vec::new(),
        explanation: explanation.into(),
    });
}

fn to_wire_string<T: serde::Serialize>(value: &T) -> String {
    serde_json::to_string(value)
        .unwrap_or_else(|e| {
            tracing::warn!(error = %e, "failed to serialize constitutional enum; using Debug fallback");
            format!("{e}")
        })
        .trim_matches('"')
        .to_string()
}

fn to_wire_strings<T: serde::Serialize>(values: &[T]) -> Vec<String> {
    values.iter().map(to_wire_string).collect()
}

#[allow(clippy::too_many_arguments)]
fn push_numeric(
    out: &mut Vec<ObligationContributionV1>,
    family: impl Into<String>,
    key: impl Into<String>,
    kind: CompiledObligationKindV1,
    fold: FoldClassV1,
    numeric_value: i64,
    source_profile_ref: &str,
    explanation: impl Into<String>,
    admissible_exception_classes: Vec<String>,
) {
    out.push(ObligationContributionV1 {
        obligation_family: family.into(),
        obligation_key: key.into(),
        output_kind: kind,
        fold_class: fold,
        string_values: Vec::new(),
        numeric_value: Some(numeric_value),
        expiry_at: None,
        blocking: false,
        source_profile_ref: source_profile_ref.to_string(),
        admissible_exception_classes,
        explanation: explanation.into(),
    });
}

#[allow(clippy::too_many_arguments)]
fn push_block(
    out: &mut Vec<ObligationContributionV1>,
    family: impl Into<String>,
    key: impl Into<String>,
    kind: CompiledObligationKindV1,
    values: Vec<String>,
    source_profile_ref: &str,
    explanation: impl Into<String>,
    admissible_exception_classes: Vec<String>,
) {
    out.push(ObligationContributionV1 {
        obligation_family: family.into(),
        obligation_key: key.into(),
        output_kind: kind,
        fold_class: FoldClassV1::BlockDominant,
        string_values: values,
        numeric_value: None,
        expiry_at: None,
        blocking: true,
        source_profile_ref: source_profile_ref.to_string(),
        admissible_exception_classes,
        explanation: explanation.into(),
    });
}

#[allow(clippy::too_many_arguments)]
fn push_conflict(
    out: &mut Vec<ObligationContributionV1>,
    family: impl Into<String>,
    key: impl Into<String>,
    kind: CompiledObligationKindV1,
    values: Vec<String>,
    source_profile_ref: &str,
    explanation: impl Into<String>,
    admissible_exception_classes: Vec<String>,
) {
    if values.is_empty() {
        return;
    }
    out.push(ObligationContributionV1 {
        obligation_family: family.into(),
        obligation_key: key.into(),
        output_kind: kind,
        fold_class: FoldClassV1::ConflictIfDifferent,
        string_values: values,
        numeric_value: None,
        expiry_at: None,
        blocking: false,
        source_profile_ref: source_profile_ref.to_string(),
        admissible_exception_classes,
        explanation: explanation.into(),
    });
}

/// Projects an effect policy profile into obligation contributions for composition.
pub fn from_effect_policy_profile(
    profile: &EffectPolicyProfileV1,
) -> Vec<ObligationContributionV1> {
    let src = profile.effect_policy_profile_id.to_string();
    let mut out = Vec::new();
    push_strings(
        &mut out,
        "effect.allowed_run_modes",
        "run_modes",
        CompiledObligationKindV1::Effect,
        FoldClassV1::Intersection,
        to_wire_strings(&profile.allowed_run_modes),
        &src,
        "effect policy allowed run modes",
    );
    push_strings(
        &mut out,
        "effect.required_preflight_checks",
        "preflight_checks",
        CompiledObligationKindV1::Check,
        FoldClassV1::Union,
        to_wire_strings(&profile.required_preflight_checks),
        &src,
        "effect policy required preflight checks",
    );
    push_strings(
        &mut out,
        "monitor.required",
        "observation_classes",
        CompiledObligationKindV1::Monitor,
        FoldClassV1::Union,
        to_wire_strings(&profile.required_observation_classes),
        &src,
        "effect policy required observation classes",
    );
    push_strings(
        &mut out,
        "compensation.required",
        "effect_classes",
        CompiledObligationKindV1::Compensation,
        FoldClassV1::Union,
        to_wire_strings(&profile.requires_compensation_plan_for),
        &src,
        "effect policy compensation requirements",
    );
    if profile.block_live_without_commit {
        push_block(
            &mut out,
            "effect.live_commit_required",
            "live_without_commit",
            CompiledObligationKindV1::Block,
            vec!["live_without_commit".into()],
            &src,
            "live execution without commit is blocked",
            vec!["break_glass".into()],
        );
    }
    out
}

/// Projects a delegation policy profile into obligation contributions for composition.
pub fn from_delegation_policy_profile(
    profile: &DelegationPolicyProfileV1,
) -> Vec<ObligationContributionV1> {
    let src = profile.delegation_policy_profile_id.to_string();
    let mut out = Vec::new();
    push_numeric(
        &mut out,
        "delegation.max_depth",
        "delegation_depth",
        CompiledObligationKindV1::Delegation,
        FoldClassV1::MinOfMaxima,
        profile.max_delegation_depth,
        &src,
        "delegation depth cap",
        vec!["break_glass".into()],
    );
    if profile.break_glass_requires_post_hoc_review {
        push_strings(
            &mut out,
            "review.post_hoc",
            "break_glass",
            CompiledObligationKindV1::PostHocReview,
            FoldClassV1::Union,
            vec!["delegation_break_glass_review".into()],
            &src,
            "break-glass requires post-hoc review",
        );
    }
    for forbidden in &profile.forbidden_role_combinations {
        push_block(
            &mut out,
            "delegation.forbidden_role_combination",
            to_wire_string(forbidden),
            CompiledObligationKindV1::Block,
            vec![to_wire_string(forbidden)],
            &src,
            "forbidden role combination",
            vec!["conflict_override".into(), "break_glass".into()],
        );
    }
    if profile.require_typed_authority_chain {
        push_strings(
            &mut out,
            "delegation.required_chain_shape",
            "typed_authority_chain",
            CompiledObligationKindV1::Delegation,
            FoldClassV1::Union,
            vec!["typed_authority_chain".into()],
            &src,
            "delegation requires typed authority chain",
        );
    }
    out
}

/// Projects a release policy profile into obligation contributions for composition.
pub fn from_release_policy_profile(
    profile: &ReleasePolicyProfileV1,
) -> Vec<ObligationContributionV1> {
    let src = profile.release_policy_profile_id.to_string();
    let mut out = Vec::new();
    push_strings(
        &mut out,
        "assurance.required_sections",
        "assurance_sections",
        CompiledObligationKindV1::Assurance,
        FoldClassV1::Union,
        to_wire_strings(&profile.required_assurance_sections),
        &src,
        "required assurance sections",
    );
    push_strings(
        &mut out,
        "monitor.required",
        "release_monitor_classes",
        CompiledObligationKindV1::Monitor,
        FoldClassV1::Union,
        to_wire_strings(&profile.required_monitor_classes),
        &src,
        "release monitor requirements",
    );
    if profile.block_on_open_obligations {
        push_block(
            &mut out,
            "release.block_on_open_obligations",
            "open_obligations",
            CompiledObligationKindV1::Block,
            vec!["open_obligations".into()],
            &src,
            "release blocks on open obligations",
            Vec::new(),
        );
    }
    if profile.forbid_score_only_gate {
        push_block(
            &mut out,
            "release.forbid_score_only_gate",
            "score_only_gate",
            CompiledObligationKindV1::Block,
            vec!["score_only_gate".into()],
            &src,
            "score-only gate is forbidden",
            Vec::new(),
        );
    }
    out
}

/// Projects a continuity policy profile into obligation contributions for composition.
pub fn from_continuity_policy_profile(
    profile: &ContinuityPolicyProfileV1,
) -> Vec<ObligationContributionV1> {
    let src = profile.continuity_policy_profile_id.to_string();
    let mut out = Vec::new();
    push_strings(
        &mut out,
        "continuity.forensic_freeze_surfaces",
        "freeze_surfaces",
        CompiledObligationKindV1::Continuity,
        FoldClassV1::Union,
        to_wire_strings(&profile.required_forensic_freeze_surfaces),
        &src,
        "required forensic freeze surfaces",
    );
    push_numeric(
        &mut out,
        "continuity.exception_ttl_minutes",
        "exception_ttl",
        CompiledObligationKindV1::Continuity,
        FoldClassV1::MinOfMaxima,
        profile.continuity_exception_ttl_minutes,
        &src,
        "continuity exception ttl cap",
        vec!["incident_override".into()],
    );
    push_strings(
        &mut out,
        "review.post_hoc",
        "postmortem_severity",
        CompiledObligationKindV1::PostHocReview,
        FoldClassV1::Union,
        to_wire_strings(&profile.requires_postmortem_for_severity),
        &src,
        "required postmortems by severity",
    );
    if profile.require_error_budget_linkage {
        push_block(
            &mut out,
            "continuity.require_error_budget_linkage",
            "error_budget_linkage",
            CompiledObligationKindV1::Block,
            vec!["error_budget_linkage".into()],
            &src,
            "continuity paths require error budget linkage",
            vec!["incident_override".into()],
        );
    }
    out
}

/// Projects a privacy and retention profile into obligation contributions for composition.
pub fn from_privacy_retention_profile(
    profile: &PrivacyRetentionProfileV1,
) -> Vec<ObligationContributionV1> {
    let src = profile.privacy_retention_profile_id.clone();
    let mut out = Vec::new();
    push_conflict(
        &mut out,
        "privacy.retention_class",
        "default_retention_class",
        CompiledObligationKindV1::Disclosure,
        vec![to_wire_string(&profile.default_retention_class)],
        &src,
        "default retention class",
        vec!["privacy_exception".into()],
    );
    push_conflict(
        &mut out,
        "privacy.archive_restore_expectation",
        "archive_restore_expectation",
        CompiledObligationKindV1::Replay,
        vec![to_wire_string(&profile.archive_restore_expectation)],
        &src,
        "archive restore expectation",
        Vec::new(),
    );
    push_conflict(
        &mut out,
        "privacy.cross_border_transfer_default",
        "cross_border_transfer_default",
        CompiledObligationKindV1::Disclosure,
        vec![to_wire_string(&profile.cross_border_transfer_default)],
        &src,
        "cross-border transfer default",
        vec!["locality_exception".into()],
    );
    push_strings(
        &mut out,
        "disclosure.required_redaction_rule_set",
        "default_redaction_rule_set",
        CompiledObligationKindV1::Disclosure,
        FoldClassV1::Union,
        vec![profile.default_redaction_rule_set_id.clone()],
        &src,
        "default redaction rule set",
    );
    if profile.compaction_requires_receipt {
        push_strings(
            &mut out,
            "replay.compaction_receipt_required",
            "compaction_receipt",
            CompiledObligationKindV1::Replay,
            FoldClassV1::Union,
            vec!["compaction_receipt_required".into()],
            &src,
            "compaction requires receipt",
        );
    }
    out
}

/// Projects a redaction rule set into obligation contributions for composition.
pub fn from_redaction_rule_set(rule_set: &RedactionRuleSetV1) -> Vec<ObligationContributionV1> {
    let src = rule_set.redaction_rule_set_id.clone();
    let mut out = Vec::new();
    push_strings(
        &mut out,
        "disclosure.redaction_actions",
        "field_actions",
        CompiledObligationKindV1::Disclosure,
        FoldClassV1::Union,
        rule_set
            .field_actions
            .iter()
            .map(|action| format!("{}:{}", action.field, action.action))
            .collect(),
        &src,
        "redaction actions",
    );
    push_conflict(
        &mut out,
        "disclosure.redaction_reversibility",
        "reversibility_class",
        CompiledObligationKindV1::Disclosure,
        vec![to_wire_string(&rule_set.reversibility_class)],
        &src,
        "redaction reversibility",
        vec!["disclosure_exception".into()],
    );
    push_strings(
        &mut out,
        "approval.required",
        "redaction_approval_requirement",
        CompiledObligationKindV1::Approval,
        FoldClassV1::Union,
        vec![to_wire_string(&rule_set.approval_requirement)],
        &src,
        "redaction approval requirement",
    );
    push_conflict(
        &mut out,
        "disclosure.budget_class",
        "default_disclosure_budget_class",
        CompiledObligationKindV1::Disclosure,
        vec![to_wire_string(&rule_set.default_disclosure_budget_class)],
        &src,
        "disclosure budget class",
        vec!["disclosure_exception".into()],
    );
    out
}

/// Projects an access-purpose matrix into obligation contributions for composition.
pub fn from_access_purpose_matrix(matrix: &AccessPurposeMatrixV1) -> Vec<ObligationContributionV1> {
    let src = matrix.access_purpose_matrix_id.clone();
    let mut out = Vec::new();
    push_strings(
        &mut out,
        "disclosure.access_purpose_rules",
        "purpose_rules",
        CompiledObligationKindV1::Disclosure,
        FoldClassV1::Union,
        matrix
            .purpose_rules
            .iter()
            .map(|rule| format!("{}:{}", rule.actor_class, rule.purpose))
            .collect(),
        &src,
        "access purpose rules",
    );
    push_conflict(
        &mut out,
        "disclosure.default_access_decision",
        "default_decision",
        CompiledObligationKindV1::Disclosure,
        vec![to_wire_string(&matrix.default_decision)],
        &src,
        "default access decision",
        vec!["privacy_exception".into()],
    );
    push_strings(
        &mut out,
        "approval.required",
        "access_elevation_path",
        CompiledObligationKindV1::Approval,
        FoldClassV1::Union,
        vec![matrix.elevation_path.clone()],
        &src,
        "access elevation path",
    );
    if matrix.audit_logging_required {
        push_strings(
            &mut out,
            "replay.audit_logging_required",
            "access_audit_logging",
            CompiledObligationKindV1::Replay,
            FoldClassV1::Union,
            vec!["audit_logging_required".into()],
            &src,
            "access requires audit logging",
        );
    }
    out
}

/// Projects an audit extraction policy into obligation contributions for composition.
pub fn from_audit_extraction_policy(
    policy: &AuditExtractionPolicyV1,
) -> Vec<ObligationContributionV1> {
    let src = policy.audit_extraction_policy_id.clone();
    let mut out = Vec::new();
    push_strings(
        &mut out,
        "disclosure.allowed_audit_artifact_families",
        "allowed_audit_artifacts",
        CompiledObligationKindV1::Disclosure,
        FoldClassV1::Intersection,
        policy.allowed_artifact_families.clone(),
        &src,
        "allowed audit artifact families",
    );
    push_strings(
        &mut out,
        "disclosure.required_redaction_rule_set",
        "audit_redaction_rule_set",
        CompiledObligationKindV1::Disclosure,
        FoldClassV1::Union,
        vec![policy.required_redaction_rule_set_id.clone()],
        &src,
        "required audit redaction rule set",
    );
    push_conflict(
        &mut out,
        "disclosure.audit_budget_class",
        "audit_budget_class",
        CompiledObligationKindV1::Disclosure,
        vec![to_wire_string(&policy.disclosure_budget_class)],
        &src,
        "audit disclosure budget class",
        vec!["disclosure_exception".into()],
    );
    push_conflict(
        &mut out,
        "disclosure.export_format",
        "export_package_format",
        CompiledObligationKindV1::Disclosure,
        vec![to_wire_string(&policy.export_package_format)],
        &src,
        "audit export package format",
        vec!["disclosure_exception".into()],
    );
    push_numeric(
        &mut out,
        "disclosure.audit_expiry_hours",
        "audit_expiry_hours",
        CompiledObligationKindV1::Disclosure,
        FoldClassV1::MinOfMaxima,
        policy.expiry_hours,
        &src,
        "audit extraction expiry",
        Vec::new(),
    );
    if policy.evidence_preservation_required {
        push_strings(
            &mut out,
            "evidence.preservation_required",
            "audit_evidence_preservation",
            CompiledObligationKindV1::Evidence,
            FoldClassV1::Union,
            vec!["evidence_preservation_required".into()],
            &src,
            "audit evidence preservation required",
        );
    }
    out
}

/// Projects a residency policy profile into obligation contributions for composition.
pub fn from_residency_policy_profile(
    profile: &ResidencyPolicyProfileV1,
) -> Vec<ObligationContributionV1> {
    let src = profile.residency_policy_profile_id.clone();
    let mut out = Vec::new();
    push_strings(
        &mut out,
        "residency.allowed_storage_regions",
        "storage_regions",
        CompiledObligationKindV1::Residency,
        FoldClassV1::Intersection,
        profile.allowed_storage_regions.clone(),
        &src,
        "allowed storage regions",
    );
    push_strings(
        &mut out,
        "residency.allowed_execution_regions",
        "execution_regions",
        CompiledObligationKindV1::Residency,
        FoldClassV1::Intersection,
        profile.allowed_execution_regions.clone(),
        &src,
        "allowed execution regions",
    );
    push_strings(
        &mut out,
        "residency.allowed_replay_regions",
        "replay_regions",
        CompiledObligationKindV1::Replay,
        FoldClassV1::Intersection,
        profile.allowed_replay_regions.clone(),
        &src,
        "allowed replay regions",
    );
    for transfer_class in &profile.forbidden_transfer_classes {
        push_block(
            &mut out,
            "residency.forbidden_transfer_classes",
            transfer_class.clone(),
            CompiledObligationKindV1::Block,
            vec![transfer_class.clone()],
            &src,
            "forbidden transfer class",
            vec!["locality_exception".into()],
        );
    }
    push_strings(
        &mut out,
        "approval.required",
        "default_exception_path",
        CompiledObligationKindV1::Approval,
        FoldClassV1::Union,
        vec![profile.default_exception_path.clone()],
        &src,
        "default locality exception path",
    );
    out
}

/// Projects a tenant-boundary profile into obligation contributions for composition.
pub fn from_tenant_boundary_profile(
    profile: &TenantBoundaryProfileV1,
) -> Vec<ObligationContributionV1> {
    let src = profile.tenant_boundary_profile_id.clone();
    let mut out = Vec::new();
    push_conflict(
        &mut out,
        "tenancy.key_kind",
        "tenant_key_kind",
        CompiledObligationKindV1::Tenancy,
        vec![to_wire_string(&profile.tenant_key_kind)],
        &src,
        "tenant key kind",
        vec!["boundary_exception".into()],
    );
    push_conflict(
        &mut out,
        "tenancy.isolation_class",
        "isolation_class",
        CompiledObligationKindV1::Tenancy,
        vec![to_wire_string(&profile.isolation_class)],
        &src,
        "tenant isolation class",
        vec!["boundary_exception".into()],
    );
    push_strings(
        &mut out,
        "tenancy.shared_service_allowances",
        "shared_service_allowances",
        CompiledObligationKindV1::Tenancy,
        FoldClassV1::Intersection,
        profile.shared_service_allowances.clone(),
        &src,
        "shared service allowances",
    );
    push_block(
        &mut out,
        "tenancy.cross_tenant_query_default",
        to_wire_string(&profile.cross_tenant_query_default),
        CompiledObligationKindV1::Block,
        vec![to_wire_string(&profile.cross_tenant_query_default)],
        &src,
        "cross-tenant query default",
        vec!["boundary_exception".into()],
    );
    push_conflict(
        &mut out,
        "tenancy.key_management_segregation",
        "key_management_segregation",
        CompiledObligationKindV1::Tenancy,
        vec![profile.key_management_segregation.clone()],
        &src,
        "key management segregation",
        vec!["boundary_exception".into()],
    );
    out
}

/// Projects a cross-boundary transfer class into obligation contributions for composition.
pub fn from_cross_boundary_transfer_class(
    transfer_class: &CrossBoundaryTransferClassV1,
) -> Vec<ObligationContributionV1> {
    let src = transfer_class.cross_boundary_transfer_class_id.clone();
    let mut out = Vec::new();
    push_strings(
        &mut out,
        "residency.allowed_transfer_artifact_families",
        format!(
            "{}:{}",
            transfer_class.source_class, transfer_class.destination_class
        ),
        CompiledObligationKindV1::Residency,
        FoldClassV1::Intersection,
        transfer_class.allowed_artifact_families.clone(),
        &src,
        "allowed artifact families for cross-boundary transfer",
    );
    push_strings(
        &mut out,
        "evidence.required_attestation",
        "cross_boundary_attestation",
        CompiledObligationKindV1::Evidence,
        FoldClassV1::Union,
        vec![to_wire_string(&transfer_class.required_attestation)],
        &src,
        "required attestation for cross-boundary transfer",
    );
    push_strings(
        &mut out,
        "disclosure.required_policy_class",
        "cross_boundary_disclosure_class",
        CompiledObligationKindV1::Disclosure,
        FoldClassV1::Union,
        vec![to_wire_string(
            &transfer_class.required_disclosure_policy_class,
        )],
        &src,
        "required disclosure policy class",
    );
    push_conflict(
        &mut out,
        "residency.downgrade_behavior",
        "cross_boundary_downgrade_behavior",
        CompiledObligationKindV1::Warning,
        vec![to_wire_string(&transfer_class.downgrade_behavior)],
        &src,
        "downgrade behavior for cross-boundary transfer",
        vec!["locality_exception".into()],
    );
    out
}

/// Projects a role catalog into obligation contributions for composition.
pub fn from_role_catalog(catalog: &RoleCatalogV1) -> Vec<ObligationContributionV1> {
    let src = catalog.role_catalog_id.clone();
    let mut out = Vec::new();
    push_conflict(
        &mut out,
        "delegation.default_autonomy_ceiling",
        "autonomy_ceiling",
        CompiledObligationKindV1::Delegation,
        vec![catalog.default_autonomy_ceiling.clone()],
        &src,
        "default autonomy ceiling",
        vec!["break_glass".into()],
    );
    push_conflict(
        &mut out,
        "delegation.scope_rule",
        "scope_rule",
        CompiledObligationKindV1::Delegation,
        vec![catalog.scope_rule.clone()],
        &src,
        "delegation scope rule",
        Vec::new(),
    );
    push_numeric(
        &mut out,
        "review.role_catalog_cycle_days",
        "role_catalog_cycle_days",
        CompiledObligationKindV1::PostHocReview,
        FoldClassV1::MinOfMaxima,
        catalog.review_cycle_days,
        &src,
        "role catalog review cycle days",
        Vec::new(),
    );
    push_strings(
        &mut out,
        "delegation.role_definitions",
        "role_definitions",
        CompiledObligationKindV1::Delegation,
        FoldClassV1::Union,
        catalog
            .role_definitions
            .iter()
            .map(|role| format!("{}:{}", role.role, role.capabilities.join("|")))
            .collect(),
        &src,
        "role definitions",
    );
    out
}

/// Projects a delegation matrix into obligation contributions for composition.
pub fn from_delegation_matrix(matrix: &DelegationMatrixV1) -> Vec<ObligationContributionV1> {
    let src = matrix.delegation_matrix_id.clone();
    let mut out = Vec::new();
    push_strings(
        &mut out,
        "delegation.allowed_edges",
        "allowed_edges",
        CompiledObligationKindV1::Delegation,
        FoldClassV1::Intersection,
        matrix
            .allowed_edges
            .iter()
            .map(|edge| format!("{}>{}:{}", edge.from_role, edge.to_role, edge.capability))
            .collect(),
        &src,
        "allowed delegation edges",
    );
    push_numeric(
        &mut out,
        "delegation.max_depth",
        "delegation_depth",
        CompiledObligationKindV1::Delegation,
        FoldClassV1::MinOfMaxima,
        matrix.max_delegation_depth,
        &src,
        "delegation matrix max depth",
        vec!["break_glass".into()],
    );
    push_strings(
        &mut out,
        "delegation.required_lease_classes",
        "required_lease_classes",
        CompiledObligationKindV1::Delegation,
        FoldClassV1::Union,
        matrix.required_lease_classes.clone(),
        &src,
        "required delegation lease classes",
    );
    for pattern in &matrix.forbidden_chain_patterns {
        push_block(
            &mut out,
            "delegation.forbidden_chain_pattern",
            pattern.clone(),
            CompiledObligationKindV1::Block,
            vec![pattern.clone()],
            &src,
            "forbidden delegation chain pattern",
            vec!["break_glass".into()],
        );
    }
    out
}

/// Projects an approval matrix into obligation contributions for composition.
pub fn from_approval_matrix(matrix: &ApprovalMatrixV1) -> Vec<ObligationContributionV1> {
    let src = matrix.approval_matrix_id.clone();
    let mut out = Vec::new();
    push_strings(
        &mut out,
        "approval.required",
        "approval_rules",
        CompiledObligationKindV1::Approval,
        FoldClassV1::Union,
        matrix
            .approval_rules
            .iter()
            .map(|rule| format!("{}:{}", rule.action_class, rule.required_roles.join("|")))
            .collect(),
        &src,
        "approval rules",
    );
    push_conflict(
        &mut out,
        "approval.default_quorum",
        "default_quorum",
        CompiledObligationKindV1::Approval,
        vec![matrix.default_quorum.clone()],
        &src,
        "default approval quorum",
        Vec::new(),
    );
    push_strings(
        &mut out,
        "approval.required",
        "independent_review",
        CompiledObligationKindV1::Approval,
        FoldClassV1::Union,
        matrix.requires_independent_review_for.clone(),
        &src,
        "independent review requirements",
    );
    push_numeric(
        &mut out,
        "review.post_hoc_window_hours",
        "break_glass_post_hoc_review_hours",
        CompiledObligationKindV1::PostHocReview,
        FoldClassV1::MinOfMaxima,
        matrix.break_glass_post_hoc_review_hours,
        &src,
        "break-glass post-hoc review window",
        vec!["break_glass".into()],
    );
    out
}

/// Projects a conflict-class catalog into obligation contributions for composition.
pub fn from_conflict_class_catalog(
    catalog: &ConflictClassCatalogV1,
) -> Vec<ObligationContributionV1> {
    let src = catalog.conflict_class_catalog_id.clone();
    let mut out = Vec::new();
    push_strings(
        &mut out,
        "warning.conflict_classes",
        "conflict_classes",
        CompiledObligationKindV1::Warning,
        FoldClassV1::Union,
        catalog
            .conflict_classes
            .iter()
            .map(|rule| {
                format!(
                    "{}:{}",
                    rule.conflict_class,
                    rule.disallowed_roles.join("|")
                )
            })
            .collect(),
        &src,
        "conflict class catalog entries",
    );
    push_conflict(
        &mut out,
        "approval.default_recusal_behavior",
        "default_recusal_behavior",
        CompiledObligationKindV1::Approval,
        vec![catalog.default_recusal_behavior.clone()],
        &src,
        "default recusal behavior",
        vec!["conflict_override".into()],
    );
    push_strings(
        &mut out,
        "approval.required",
        "conflict_override_path",
        CompiledObligationKindV1::Approval,
        FoldClassV1::Union,
        vec![catalog.override_path.clone()],
        &src,
        "conflict override path",
    );
    if catalog.disclosure_required {
        push_strings(
            &mut out,
            "disclosure.conflict_disclosure_required",
            "conflict_disclosure",
            CompiledObligationKindV1::Disclosure,
            FoldClassV1::Union,
            vec!["conflict_disclosure_required".into()],
            &src,
            "conflict disclosure required",
        );
    }
    out
}

/// Projects a regulatory-regime profile into obligation contributions for composition.
pub fn from_regulatory_regime_profile(
    profile: &RegulatoryRegimeProfileV1,
) -> Vec<ObligationContributionV1> {
    let src = profile.regulatory_regime_profile_id.clone();
    let mut out = Vec::new();
    push_strings(
        &mut out,
        "assurance.mandatory_control_families",
        "control_families",
        CompiledObligationKindV1::Assurance,
        FoldClassV1::Union,
        profile.mandatory_control_families.clone(),
        &src,
        "mandatory control families",
    );
    push_strings(
        &mut out,
        "assurance.covered_products",
        "covered_products",
        CompiledObligationKindV1::Assurance,
        FoldClassV1::Union,
        profile.covered_products.clone(),
        &src,
        "covered products",
    );
    push_numeric(
        &mut out,
        "assurance.audit_cycle_days",
        "audit_cycle_days",
        CompiledObligationKindV1::Assurance,
        FoldClassV1::MinOfMaxima,
        profile.audit_cycle_days,
        &src,
        "regime audit cycle days",
        Vec::new(),
    );
    push_conflict(
        &mut out,
        "assurance.regime_version",
        "regime_version",
        CompiledObligationKindV1::Assurance,
        vec![profile.regime_version.clone()],
        &src,
        "regime version",
        Vec::new(),
    );
    out
}

/// Projects a requirement-to-control map into obligation contributions for composition.
pub fn from_requirement_control_map(
    map: &RequirementControlMapV1,
) -> Vec<ObligationContributionV1> {
    let src = map.requirement_control_map_id.clone();
    let mut out = Vec::new();
    push_strings(
        &mut out,
        "assurance.requirement_mappings",
        "requirement_control_mappings",
        CompiledObligationKindV1::Assurance,
        FoldClassV1::Union,
        map.mappings
            .iter()
            .map(|entry| {
                format!(
                    "{}:{}:{}",
                    entry.requirement, entry.control, entry.evidence_family
                )
            })
            .collect(),
        &src,
        "requirement-to-control mappings",
    );
    push_conflict(
        &mut out,
        "assurance.gap_classification_default",
        "gap_classification_default",
        CompiledObligationKindV1::Assurance,
        vec![map.gap_classification_default.clone()],
        &src,
        "gap classification default",
        Vec::new(),
    );
    push_strings(
        &mut out,
        "assurance.owner_ref",
        "requirement_control_map_owner",
        CompiledObligationKindV1::Assurance,
        FoldClassV1::Union,
        vec![map.owner_ref.clone()],
        &src,
        "requirement-control map owner",
    );
    out
}

/// Projects an evidence-collection plan into obligation contributions for composition.
pub fn from_evidence_collection_plan(
    plan: &EvidenceCollectionPlanV1,
) -> Vec<ObligationContributionV1> {
    let src = plan.evidence_collection_plan_id.clone();
    let mut out = Vec::new();
    push_strings(
        &mut out,
        "evidence.required_classes",
        "required_evidence_classes",
        CompiledObligationKindV1::Evidence,
        FoldClassV1::Union,
        plan.required_evidence_classes
            .iter()
            .map(|entry| format!("{}:{}", entry.evidence_class, entry.minimum_family))
            .collect(),
        &src,
        "required evidence classes",
    );
    push_conflict(
        &mut out,
        "evidence.collection_cadence",
        "collection_cadence",
        CompiledObligationKindV1::Evidence,
        vec![plan.collection_cadence.clone()],
        &src,
        "evidence collection cadence",
        Vec::new(),
    );
    push_conflict(
        &mut out,
        "evidence.retention_class",
        "retention_class",
        CompiledObligationKindV1::Evidence,
        vec![plan.retention_class.clone()],
        &src,
        "evidence retention class",
        Vec::new(),
    );
    push_strings(
        &mut out,
        "evidence.owner_ref",
        "evidence_collection_owner",
        CompiledObligationKindV1::Evidence,
        FoldClassV1::Union,
        vec![plan.owner_ref.clone()],
        &src,
        "evidence collection owner",
    );
    out
}

/// Projects a recertification schedule into obligation contributions for composition.
pub fn from_recertification_schedule(
    schedule: &RecertificationScheduleV1,
) -> Vec<ObligationContributionV1> {
    let src = schedule.recertification_schedule_id.clone();
    let mut out = Vec::new();
    push_numeric(
        &mut out,
        "assurance.review_interval_days",
        "recertification_review_interval",
        CompiledObligationKindV1::Assurance,
        FoldClassV1::MinOfMaxima,
        schedule.review_interval_days,
        &src,
        "recertification review interval",
        Vec::new(),
    );
    push_strings(
        &mut out,
        "assurance.recertification_triggers",
        "trigger_classes",
        CompiledObligationKindV1::Assurance,
        FoldClassV1::Union,
        schedule.trigger_classes.clone(),
        &src,
        "recertification trigger classes",
    );
    push_numeric(
        &mut out,
        "assurance.grace_window_days",
        "grace_window_days",
        CompiledObligationKindV1::Assurance,
        FoldClassV1::MinOfMaxima,
        schedule.grace_window_days,
        &src,
        "recertification grace window",
        Vec::new(),
    );
    push_block(
        &mut out,
        "assurance.blocked_state_on_expiry",
        schedule.blocked_state_on_expiry.clone(),
        CompiledObligationKindV1::Block,
        vec![schedule.blocked_state_on_expiry.clone()],
        &src,
        "blocked state on recertification expiry",
        Vec::new(),
    );
    out
}

/// Projects a hazard library into obligation contributions for composition.
pub fn from_hazard_library(library: &HazardLibraryV1) -> Vec<ObligationContributionV1> {
    let src = library.hazard_library_id.clone();
    let mut out = Vec::new();
    push_strings(
        &mut out,
        "warning.hazard_families",
        "hazard_families",
        CompiledObligationKindV1::Warning,
        FoldClassV1::Union,
        library.hazard_families.clone(),
        &src,
        "hazard families",
    );
    push_conflict(
        &mut out,
        "warning.hazard_scoring_model",
        "scoring_model_ref",
        CompiledObligationKindV1::Warning,
        vec![library.scoring_model_ref.clone()],
        &src,
        "hazard scoring model",
        Vec::new(),
    );
    push_strings(
        &mut out,
        "assurance.linked_operating_envelopes",
        "linked_operating_envelopes",
        CompiledObligationKindV1::Assurance,
        FoldClassV1::Union,
        library.linked_operating_envelopes.clone(),
        &src,
        "linked operating envelopes",
    );
    out
}

/// Projects a hazard scenario into obligation contributions for composition.
pub fn from_hazard_scenario(scenario: &HazardScenarioV1) -> Vec<ObligationContributionV1> {
    let src = scenario.hazard_scenario_id.clone();
    let mut out = Vec::new();
    push_strings(
        &mut out,
        "warning.hazard_trigger_conditions",
        "trigger_conditions",
        CompiledObligationKindV1::Warning,
        FoldClassV1::Union,
        scenario.trigger_conditions.clone(),
        &src,
        "hazard trigger conditions",
    );
    push_strings(
        &mut out,
        "continuity.affected_surfaces",
        "affected_surfaces",
        CompiledObligationKindV1::Continuity,
        FoldClassV1::Union,
        scenario.affected_surfaces.clone(),
        &src,
        "hazard affected surfaces",
    );
    push_conflict(
        &mut out,
        "warning.hazard_severity_baseline",
        "severity_baseline",
        CompiledObligationKindV1::Warning,
        vec![scenario.severity_baseline.clone()],
        &src,
        "hazard severity baseline",
        Vec::new(),
    );
    push_strings(
        &mut out,
        "monitor.required",
        "hazard_required_monitor_refs",
        CompiledObligationKindV1::Monitor,
        FoldClassV1::Union,
        scenario.required_monitor_refs.clone(),
        &src,
        "hazard required monitor refs",
    );
    out
}

/// Projects a monitor catalog into obligation contributions for composition.
pub fn from_monitor_catalog(catalog: &MonitorCatalogV1) -> Vec<ObligationContributionV1> {
    let src = catalog.monitor_catalog_id.clone();
    let mut out = Vec::new();
    push_strings(
        &mut out,
        "monitor.required",
        "monitor_definitions",
        CompiledObligationKindV1::Monitor,
        FoldClassV1::Union,
        catalog
            .monitor_definitions
            .iter()
            .map(|entry| format!("{}:{}", entry.monitor, entry.threshold))
            .collect(),
        &src,
        "monitor definitions",
    );
    push_conflict(
        &mut out,
        "monitor.evaluation_cadence",
        "evaluation_cadence",
        CompiledObligationKindV1::Monitor,
        vec![catalog.evaluation_cadence.clone()],
        &src,
        "monitor evaluation cadence",
        Vec::new(),
    );
    push_conflict(
        &mut out,
        "monitor.false_positive_budget",
        "false_positive_budget",
        CompiledObligationKindV1::Monitor,
        vec![catalog.false_positive_budget.clone()],
        &src,
        "monitor false-positive budget",
        Vec::new(),
    );
    push_strings(
        &mut out,
        "monitor.owner_ref",
        "monitor_catalog_owner",
        CompiledObligationKindV1::Monitor,
        FoldClassV1::Union,
        vec![catalog.owner_ref.clone()],
        &src,
        "monitor catalog owner",
    );
    out
}

/// Projects a mitigation playbook into obligation contributions for composition.
pub fn from_mitigation_playbook(playbook: &MitigationPlaybookV1) -> Vec<ObligationContributionV1> {
    let src = playbook.mitigation_playbook_id.clone();
    let mut out = Vec::new();
    push_strings(
        &mut out,
        "continuity.containment_steps",
        "containment_steps",
        CompiledObligationKindV1::Continuity,
        FoldClassV1::Union,
        playbook.containment_steps.clone(),
        &src,
        "mitigation containment steps",
    );
    push_strings(
        &mut out,
        "continuity.recovery_steps",
        "recovery_steps",
        CompiledObligationKindV1::Continuity,
        FoldClassV1::Union,
        playbook.recovery_steps.clone(),
        &src,
        "mitigation recovery steps",
    );
    push_strings(
        &mut out,
        "approval.required",
        "mitigation_approvals",
        CompiledObligationKindV1::Approval,
        FoldClassV1::Union,
        playbook.approval_refs.clone(),
        &src,
        "mitigation approval refs",
    );
    push_strings(
        &mut out,
        "continuity.success_criteria",
        "success_criteria",
        CompiledObligationKindV1::Continuity,
        FoldClassV1::Union,
        playbook.success_criteria.clone(),
        &src,
        "mitigation success criteria",
    );
    push_strings(
        &mut out,
        "warning.hazard_refs",
        "hazard_refs",
        CompiledObligationKindV1::Warning,
        FoldClassV1::Union,
        playbook.hazard_refs.clone(),
        &src,
        "mitigation hazard refs",
    );
    out
}

/// Projects a vendor certification adapter into obligation contributions for composition.
pub fn from_vendor_certification_adapter(
    adapter: &VendorCertificationAdapterV1,
) -> Vec<ObligationContributionV1> {
    let src = adapter.vendor_certification_adapter_id.clone();
    let mut out = Vec::new();
    push_conflict(
        &mut out,
        "vendor.translation_mode",
        "translation_mode",
        CompiledObligationKindV1::Warning,
        vec![adapter.translation_mode.to_string()],
        &src,
        "vendor translation mode",
        Vec::new(),
    );
    push_strings(
        &mut out,
        "vendor.covered_artifact_families",
        "covered_artifact_families",
        CompiledObligationKindV1::Evidence,
        FoldClassV1::Intersection,
        adapter.covered_artifact_families.clone(),
        &src,
        "vendor covered artifact families",
    );
    push_conflict(
        &mut out,
        "vendor.support_window",
        "support_window",
        CompiledObligationKindV1::Warning,
        vec![adapter.support_window.clone()],
        &src,
        "vendor support window",
        Vec::new(),
    );
    push_strings(
        &mut out,
        "vendor.product_surface",
        "product_surface",
        CompiledObligationKindV1::Evidence,
        FoldClassV1::Union,
        vec![adapter.product_surface.clone()],
        &src,
        "vendor product surface",
    );
    out
}

/// Projects a vendor evidence translation policy into obligation contributions for composition.
pub fn from_vendor_evidence_translation(
    translation: &VendorEvidenceTranslationV1,
) -> Vec<ObligationContributionV1> {
    let src = translation.vendor_evidence_translation_id.clone();
    let mut out = Vec::new();
    push_strings(
        &mut out,
        "vendor.source_shapes",
        "source_shapes",
        CompiledObligationKindV1::Evidence,
        FoldClassV1::Union,
        translation.source_shapes.clone(),
        &src,
        "vendor source shapes",
    );
    push_strings(
        &mut out,
        "vendor.canonical_targets",
        "canonical_targets",
        CompiledObligationKindV1::Evidence,
        FoldClassV1::Union,
        translation.canonical_targets.clone(),
        &src,
        "vendor canonical targets",
    );
    push_strings(
        &mut out,
        "vendor.lossy_fields",
        "lossy_fields",
        CompiledObligationKindV1::Warning,
        FoldClassV1::Union,
        translation.lossy_fields.clone(),
        &src,
        "vendor lossy fields",
    );
    push_strings(
        &mut out,
        "disclosure.vendor_required_caveats",
        "required_caveats",
        CompiledObligationKindV1::Disclosure,
        FoldClassV1::Union,
        translation.required_caveats.clone(),
        &src,
        "vendor required caveats",
    );
    out
}

/// Projects a vendor trust-root binding into obligation contributions for composition.
pub fn from_vendor_trust_root_binding(
    binding: &VendorTrustRootBindingV1,
) -> Vec<ObligationContributionV1> {
    let src = binding.vendor_trust_root_binding_id.clone();
    let mut out = Vec::new();
    push_strings(
        &mut out,
        "evidence.trust_root_refs",
        "trust_root_refs",
        CompiledObligationKindV1::Evidence,
        FoldClassV1::Union,
        binding.trust_root_refs.clone(),
        &src,
        "vendor trust root refs",
    );
    push_strings(
        &mut out,
        "evidence.signer_classes",
        "signer_classes",
        CompiledObligationKindV1::Evidence,
        FoldClassV1::Union,
        binding.signer_classes.clone(),
        &src,
        "vendor signer classes",
    );
    push_conflict(
        &mut out,
        "vendor.rotation_channel",
        "rotation_channel",
        CompiledObligationKindV1::Warning,
        vec![binding.rotation_channel.to_string()],
        &src,
        "vendor rotation channel",
        Vec::new(),
    );
    push_conflict(
        &mut out,
        "vendor.revocation_channel",
        "revocation_channel",
        CompiledObligationKindV1::Warning,
        vec![binding.revocation_channel.to_string()],
        &src,
        "vendor revocation channel",
        Vec::new(),
    );
    out
}

/// Projects a vendor revocation-handling policy into obligation contributions for composition.
pub fn from_vendor_revocation_handling(
    handling: &VendorRevocationHandlingV1,
) -> Vec<ObligationContributionV1> {
    let src = handling.vendor_revocation_handling_id.clone();
    let mut out = Vec::new();
    push_strings(
        &mut out,
        "warning.vendor_revocation_inputs",
        "revocation_inputs",
        CompiledObligationKindV1::Warning,
        FoldClassV1::Union,
        handling.revocation_inputs.clone(),
        &src,
        "vendor revocation inputs",
    );
    push_strings(
        &mut out,
        "rollback.vendor_invalidation_actions",
        "local_invalidation_actions",
        CompiledObligationKindV1::Rollback,
        FoldClassV1::Union,
        handling.local_invalidation_actions.clone(),
        &src,
        "vendor local invalidation actions",
    );
    push_conflict(
        &mut out,
        "vendor.replay_impact",
        "replay_impact",
        CompiledObligationKindV1::Warning,
        vec![handling.replay_impact.to_string()],
        &src,
        "vendor replay impact",
        Vec::new(),
    );
    push_block(
        &mut out,
        "vendor.admission_impact",
        handling.admission_impact.to_string(),
        CompiledObligationKindV1::Block,
        vec![handling.admission_impact.to_string()],
        &src,
        "vendor admission impact",
        vec!["vendor_override".into()],
    );
    out
}

/// Projects an incident taxonomy into obligation contributions for composition.
pub fn from_incident_taxonomy(taxonomy: &IncidentTaxonomyV1) -> Vec<ObligationContributionV1> {
    let src = taxonomy.incident_taxonomy_id.clone();
    let mut out = Vec::new();
    push_strings(
        &mut out,
        "continuity.incident_classes",
        "incident_classes",
        CompiledObligationKindV1::Continuity,
        FoldClassV1::Union,
        taxonomy
            .incident_classes
            .iter()
            .map(|rule| format!("{}:{}", rule.incident_class, rule.default_severity))
            .collect(),
        &src,
        "incident classes",
    );
    push_strings(
        &mut out,
        "continuity.default_routes",
        "default_routes",
        CompiledObligationKindV1::Continuity,
        FoldClassV1::Union,
        taxonomy
            .default_routes
            .iter()
            .map(|route| format!("{}:{}", route.incident_class, route.route))
            .collect(),
        &src,
        "default incident routes",
    );
    push_strings(
        &mut out,
        "evidence.required_artifact_families",
        "required_artifact_families",
        CompiledObligationKindV1::Evidence,
        FoldClassV1::Union,
        taxonomy.required_artifact_families.clone(),
        &src,
        "required incident artifact families",
    );
    out
}

/// Projects a severity matrix into obligation contributions for composition.
pub fn from_severity_matrix(matrix: &SeverityMatrixV1) -> Vec<ObligationContributionV1> {
    let src = matrix.severity_matrix_id.clone();
    let mut out = Vec::new();
    push_strings(
        &mut out,
        "continuity.severity_rules",
        "severity_rules",
        CompiledObligationKindV1::Continuity,
        FoldClassV1::Union,
        matrix
            .severity_rules
            .iter()
            .map(|rule| format!("{}:{}", rule.condition, rule.severity))
            .collect(),
        &src,
        "severity rules",
    );
    push_conflict(
        &mut out,
        "continuity.customer_impact_rubric",
        "customer_impact_rubric",
        CompiledObligationKindV1::Continuity,
        vec![matrix.customer_impact_rubric.clone()],
        &src,
        "customer impact rubric",
        Vec::new(),
    );
    push_conflict(
        &mut out,
        "continuity.internal_impact_rubric",
        "internal_impact_rubric",
        CompiledObligationKindV1::Continuity,
        vec![matrix.internal_impact_rubric.clone()],
        &src,
        "internal impact rubric",
        Vec::new(),
    );
    push_conflict(
        &mut out,
        "continuity.override_rule",
        "override_rule",
        CompiledObligationKindV1::Continuity,
        vec![matrix.override_rule.clone()],
        &src,
        "severity override rule",
        vec!["incident_override".into()],
    );
    out
}

/// Projects a pager-route profile into obligation contributions for composition.
pub fn from_pager_route_profile(profile: &PagerRouteProfileV1) -> Vec<ObligationContributionV1> {
    let src = profile.pager_route_profile_id.clone();
    let mut out = Vec::new();
    push_strings(
        &mut out,
        "continuity.rotation_refs",
        "rotation_refs",
        CompiledObligationKindV1::Continuity,
        FoldClassV1::Union,
        profile.rotation_refs.clone(),
        &src,
        "pager rotation refs",
    );
    push_strings(
        &mut out,
        "continuity.handoff_rules",
        "handoff_rules",
        CompiledObligationKindV1::Continuity,
        FoldClassV1::Union,
        profile.handoff_rules.clone(),
        &src,
        "pager handoff rules",
    );
    push_numeric(
        &mut out,
        "continuity.ack_timeout_minutes",
        "ack_timeout_minutes",
        CompiledObligationKindV1::Continuity,
        FoldClassV1::MinOfMaxima,
        profile.ack_timeout_minutes,
        &src,
        "pager acknowledgement timeout",
        vec!["incident_override".into()],
    );
    push_numeric(
        &mut out,
        "continuity.max_escalation_levels",
        "max_levels",
        CompiledObligationKindV1::Continuity,
        FoldClassV1::MinOfMaxima,
        profile.max_levels,
        &src,
        "pager escalation level cap",
        vec!["incident_override".into()],
    );
    out
}

/// Projects an escalation-clock policy into obligation contributions for composition.
pub fn from_escalation_clock_policy(
    policy: &EscalationClockPolicyV1,
) -> Vec<ObligationContributionV1> {
    let src = policy.escalation_clock_policy_id.clone();
    let mut out = Vec::new();
    push_strings(
        &mut out,
        "continuity.response_clocks",
        "response_clock_minutes",
        CompiledObligationKindV1::Continuity,
        FoldClassV1::Union,
        policy
            .response_clock_minutes
            .iter()
            .map(|clock| format!("{}:{}", clock.severity, clock.minutes))
            .collect(),
        &src,
        "response clocks by severity",
    );
    push_strings(
        &mut out,
        "review.post_hoc",
        "postmortem_clock_hours",
        CompiledObligationKindV1::PostHocReview,
        FoldClassV1::Union,
        policy
            .postmortem_clock_hours
            .iter()
            .map(|clock| format!("{}:{}", clock.severity, clock.hours))
            .collect(),
        &src,
        "postmortem clocks by severity",
    );
    push_strings(
        &mut out,
        "continuity.pause_rules",
        "pause_rules",
        CompiledObligationKindV1::Continuity,
        FoldClassV1::Union,
        policy.pause_rules.clone(),
        &src,
        "continuity pause rules",
    );
    push_strings(
        &mut out,
        "approval.required",
        "continuity_exception_path",
        CompiledObligationKindV1::Approval,
        FoldClassV1::Union,
        vec![policy.exception_path.clone()],
        &src,
        "continuity exception path",
    );
    out
}
