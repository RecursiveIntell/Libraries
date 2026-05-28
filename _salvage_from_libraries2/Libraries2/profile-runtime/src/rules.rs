use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use stack_ids::CompositionRuleSetId;

pub const COMPOSITION_RULE_SET_V1_SCHEMA: &str = "composition_rule_set_v1";
pub const REFERENCE_EVALUATOR_VERSION_V1: &str = "profile-runtime/reference-evaluator/v1";

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum FoldClassV1 {
    Intersection,
    Union,
    MinOfMaxima,
    MaxOfMinima,
    EarliestExpiry,
    ConflictIfDifferent,
    BlockDominant,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum CompiledObligationKindV1 {
    Approval,
    Check,
    Monitor,
    Evidence,
    Disclosure,
    Residency,
    Tenancy,
    Assurance,
    Continuity,
    Effect,
    Delegation,
    Replay,
    Rollback,
    Compensation,
    PostHocReview,
    Warning,
    Block,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ObligationFamilyRuleV1 {
    pub obligation_family: String,
    pub fold_class: FoldClassV1,
    pub output_kind: CompiledObligationKindV1,
    pub monotone_additions_expected: bool,
    pub block_on_unresolved_conflict: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub admissible_exception_classes: Vec<String>,
    pub downgrade_behavior: String,
    pub explanation: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ConflictClassRuleV1 {
    pub conflict_class: String,
    pub blocking: bool,
    pub review_path: String,
    pub explanation: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ExceptionAdmissibilityRuleV1 {
    pub exception_class: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub allowed_obligation_families: Vec<String>,
    pub requires_approval_refs: bool,
    pub requires_expiry: bool,
    pub residual_obligations_mandatory: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct CompositionRuleSetV1 {
    pub schema_version: String,
    pub composition_rule_set_id: CompositionRuleSetId,
    pub version: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub family_rules: Vec<ObligationFamilyRuleV1>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub conflict_classes: Vec<ConflictClassRuleV1>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub exception_rules: Vec<ExceptionAdmissibilityRuleV1>,
    pub downgrade_behavior: String,
    pub reference_evaluator_version: String,
}

impl CompositionRuleSetV1 {
    /// Returns the reference composition rules used by the current closeout lane.
    pub fn reference_v1() -> Self {
        Self {
            schema_version: COMPOSITION_RULE_SET_V1_SCHEMA.into(),
            composition_rule_set_id: CompositionRuleSetId::generate(),
            version: "reference-v1".into(),
            family_rules: vec![
                ObligationFamilyRuleV1 {
                    obligation_family: "effect.allowed_run_modes".into(),
                    fold_class: FoldClassV1::Intersection,
                    output_kind: CompiledObligationKindV1::Effect,
                    monotone_additions_expected: true,
                    block_on_unresolved_conflict: true,
                    admissible_exception_classes: vec!["break_glass".into()],
                    downgrade_behavior: "advisory_only".into(),
                    explanation: "allowed run modes narrow by intersection".into(),
                },
                ObligationFamilyRuleV1 {
                    obligation_family: "effect.required_preflight_checks".into(),
                    fold_class: FoldClassV1::Union,
                    output_kind: CompiledObligationKindV1::Check,
                    monotone_additions_expected: true,
                    block_on_unresolved_conflict: false,
                    admissible_exception_classes: Vec::new(),
                    downgrade_behavior: "accumulate".into(),
                    explanation: "preflight checks accumulate".into(),
                },
                ObligationFamilyRuleV1 {
                    obligation_family: "residency.allowed_execution_regions".into(),
                    fold_class: FoldClassV1::Intersection,
                    output_kind: CompiledObligationKindV1::Residency,
                    monotone_additions_expected: true,
                    block_on_unresolved_conflict: true,
                    admissible_exception_classes: vec!["locality_exception".into()],
                    downgrade_behavior: "blocked_without_exception".into(),
                    explanation: "execution region allowances narrow by overlap".into(),
                },
                ObligationFamilyRuleV1 {
                    obligation_family: "residency.forbidden_transfer_classes".into(),
                    fold_class: FoldClassV1::BlockDominant,
                    output_kind: CompiledObligationKindV1::Block,
                    monotone_additions_expected: true,
                    block_on_unresolved_conflict: true,
                    admissible_exception_classes: vec!["locality_exception".into()],
                    downgrade_behavior: "blocked".into(),
                    explanation: "forbidden transfer classes block unless explicitly excepted".into(),
                },
                ObligationFamilyRuleV1 {
                    obligation_family: "delegation.max_depth".into(),
                    fold_class: FoldClassV1::MinOfMaxima,
                    output_kind: CompiledObligationKindV1::Delegation,
                    monotone_additions_expected: true,
                    block_on_unresolved_conflict: false,
                    admissible_exception_classes: vec!["break_glass".into()],
                    downgrade_behavior: "tighten".into(),
                    explanation: "maximum delegation depth tightens to the smallest cap".into(),
                },
                ObligationFamilyRuleV1 {
                    obligation_family: "delegation.forbidden_role_combination".into(),
                    fold_class: FoldClassV1::BlockDominant,
                    output_kind: CompiledObligationKindV1::Block,
                    monotone_additions_expected: true,
                    block_on_unresolved_conflict: true,
                    admissible_exception_classes: vec!["conflict_override".into(), "break_glass".into()],
                    downgrade_behavior: "blocked".into(),
                    explanation: "forbidden role combinations survive into the effective constitution".into(),
                },
                ObligationFamilyRuleV1 {
                    obligation_family: "approval.required".into(),
                    fold_class: FoldClassV1::Union,
                    output_kind: CompiledObligationKindV1::Approval,
                    monotone_additions_expected: true,
                    block_on_unresolved_conflict: false,
                    admissible_exception_classes: Vec::new(),
                    downgrade_behavior: "accumulate".into(),
                    explanation: "approval obligations accumulate".into(),
                },
                ObligationFamilyRuleV1 {
                    obligation_family: "monitor.required".into(),
                    fold_class: FoldClassV1::Union,
                    output_kind: CompiledObligationKindV1::Monitor,
                    monotone_additions_expected: true,
                    block_on_unresolved_conflict: false,
                    admissible_exception_classes: Vec::new(),
                    downgrade_behavior: "accumulate".into(),
                    explanation: "monitor obligations accumulate".into(),
                },
                ObligationFamilyRuleV1 {
                    obligation_family: "disclosure.export_format".into(),
                    fold_class: FoldClassV1::ConflictIfDifferent,
                    output_kind: CompiledObligationKindV1::Disclosure,
                    monotone_additions_expected: false,
                    block_on_unresolved_conflict: true,
                    admissible_exception_classes: vec!["disclosure_exception".into()],
                    downgrade_behavior: "blocked_until_reviewed".into(),
                    explanation: "incompatible export format requirements must surface as a conflict".into(),
                },
                ObligationFamilyRuleV1 {
                    obligation_family: "continuity.exception_ttl_minutes".into(),
                    fold_class: FoldClassV1::MinOfMaxima,
                    output_kind: CompiledObligationKindV1::Continuity,
                    monotone_additions_expected: true,
                    block_on_unresolved_conflict: false,
                    admissible_exception_classes: vec!["incident_override".into()],
                    downgrade_behavior: "tighten".into(),
                    explanation: "continuity exception TTL uses the smallest cap".into(),
                },
                ObligationFamilyRuleV1 {
                    obligation_family: "review.post_hoc".into(),
                    fold_class: FoldClassV1::Union,
                    output_kind: CompiledObligationKindV1::PostHocReview,
                    monotone_additions_expected: true,
                    block_on_unresolved_conflict: false,
                    admissible_exception_classes: Vec::new(),
                    downgrade_behavior: "accumulate".into(),
                    explanation: "post-hoc review obligations accumulate".into(),
                },
                ObligationFamilyRuleV1 {
                    obligation_family: "vendor.lossy_fields".into(),
                    fold_class: FoldClassV1::Union,
                    output_kind: CompiledObligationKindV1::Warning,
                    monotone_additions_expected: true,
                    block_on_unresolved_conflict: false,
                    admissible_exception_classes: Vec::new(),
                    downgrade_behavior: "warn".into(),
                    explanation: "lossy translation caveats accumulate and remain visible".into(),
                },
            ],
            conflict_classes: vec![
                ConflictClassRuleV1 {
                    conflict_class: "empty_intersection".into(),
                    blocking: true,
                    review_path: "human_constitution_review".into(),
                    explanation: "intersection fold yielded no lawful overlap".into(),
                },
                ConflictClassRuleV1 {
                    conflict_class: "incomparable_values".into(),
                    blocking: true,
                    review_path: "human_constitution_review".into(),
                    explanation: "conflict-if-different obligations diverged without an admitted exception".into(),
                },
                ConflictClassRuleV1 {
                    conflict_class: "unsupported_numeric_fold".into(),
                    blocking: false,
                    review_path: "rule_set_maintenance".into(),
                    explanation: "the contribution set could not be reduced numerically and requires rule-set maintenance".into(),
                },
            ],
            exception_rules: vec![
                ExceptionAdmissibilityRuleV1 {
                    exception_class: "locality_exception".into(),
                    allowed_obligation_families: vec![
                        "residency.allowed_execution_regions".into(),
                        "residency.forbidden_transfer_classes".into(),
                    ],
                    requires_approval_refs: true,
                    requires_expiry: true,
                    residual_obligations_mandatory: true,
                },
                ExceptionAdmissibilityRuleV1 {
                    exception_class: "break_glass".into(),
                    allowed_obligation_families: vec![
                        "effect.allowed_run_modes".into(),
                        "delegation.max_depth".into(),
                        "delegation.forbidden_role_combination".into(),
                    ],
                    requires_approval_refs: true,
                    requires_expiry: true,
                    residual_obligations_mandatory: true,
                },
                ExceptionAdmissibilityRuleV1 {
                    exception_class: "incident_override".into(),
                    allowed_obligation_families: vec!["continuity.exception_ttl_minutes".into()],
                    requires_approval_refs: true,
                    requires_expiry: true,
                    residual_obligations_mandatory: true,
                },
            ],
            downgrade_behavior: "emit_receipt_and_never_fall_back_to_permissive".into(),
            reference_evaluator_version: REFERENCE_EVALUATOR_VERSION_V1.into(),
        }
    }

    /// Looks up the obligation-family rule that governs a compiled family key.
    pub fn family_rule(&self, family: &str) -> Option<&ObligationFamilyRuleV1> {
        self.family_rules
            .iter()
            .find(|rule| rule.obligation_family == family)
    }

    /// Looks up the conflict-class rule that governs a named conflict class.
    pub fn conflict_class(&self, class: &str) -> Option<&ConflictClassRuleV1> {
        self.conflict_classes
            .iter()
            .find(|rule| rule.conflict_class == class)
    }
}
