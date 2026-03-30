use std::collections::{BTreeMap, BTreeSet};

type ObligationEntryMapValue = (Vec<String>, Option<i64>, Option<String>, bool);
type ObligationEntryMapKey = (String, String);

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use stack_ids::{
    CompiledObligationSetId, CompositionConflictSetId, CompositionReceiptId, ContentDigest,
    EffectiveConstitutionId, PolicyImpactDiffId,
};
use thiserror::Error;

use crate::{
    applicability::ApplicabilityContextV1,
    constitution::{
        BlockEntryV1, CompiledObligationEntryV1, CompiledObligationSetV1, CompositionConflictSetV1,
        CompositionConflictV1, CompositionReceiptV1, ConstitutionModeV1, EffectiveConstitutionV1,
        PolicyImpactDiffV1, COMPILED_OBLIGATION_SET_V1_SCHEMA, COMPOSITION_CONFLICT_SET_V1_SCHEMA,
        COMPOSITION_RECEIPT_V1_SCHEMA, EFFECTIVE_CONSTITUTION_V1_SCHEMA,
        POLICY_IMPACT_DIFF_V1_SCHEMA,
    },
    exception::ProfileExceptionBundleV1,
    profile_set::ProfileSetV1,
    rules::{
        CompiledObligationKindV1, CompositionRuleSetV1, FoldClassV1, REFERENCE_EVALUATOR_VERSION_V1,
    },
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ObligationContributionV1 {
    pub obligation_family: String,
    pub obligation_key: String,
    pub output_kind: CompiledObligationKindV1,
    pub fold_class: FoldClassV1,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub string_values: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub numeric_value: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expiry_at: Option<String>,
    pub blocking: bool,
    pub source_profile_ref: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub admissible_exception_classes: Vec<String>,
    pub explanation: String,
}

impl ObligationContributionV1 {
    /// Builds a simple contribution without numeric or expiry metadata.
    pub fn simple(
        obligation_family: impl Into<String>,
        obligation_key: impl Into<String>,
        output_kind: CompiledObligationKindV1,
        fold_class: FoldClassV1,
        string_values: Vec<String>,
        source_profile_ref: impl Into<String>,
        explanation: impl Into<String>,
    ) -> Self {
        Self {
            obligation_family: obligation_family.into(),
            obligation_key: obligation_key.into(),
            output_kind,
            fold_class,
            string_values,
            numeric_value: None,
            expiry_at: None,
            blocking: false,
            source_profile_ref: source_profile_ref.into(),
            admissible_exception_classes: Vec::new(),
            explanation: explanation.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompositionOutcomeV1 {
    pub receipt: CompositionReceiptV1,
    pub effective_constitution: EffectiveConstitutionV1,
    pub compiled_obligation_set: CompiledObligationSetV1,
    pub conflict_set: Option<CompositionConflictSetV1>,
}

#[derive(Debug, Error)]
pub enum CompositionError {
    #[error("profile set does not match applicability context")]
    ApplicabilityMismatch,
    #[error("unsupported fold configuration for family '{family}' key '{key}'")]
    UnsupportedFold { family: String, key: String },
    #[error("digest computation failed: {reason}")]
    DigestFailed { reason: String },
}

impl CompositionError {
    /// Returns a stable string discriminant for this error kind.
    pub fn kind(&self) -> &'static str {
        match self {
            Self::ApplicabilityMismatch => "applicability_mismatch",
            Self::UnsupportedFold { .. } => "unsupported_fold",
            Self::DigestFailed { .. } => "digest_failed",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct GroupKey {
    family: String,
    key: String,
    output_kind: CompiledObligationKindV1,
    fold_class: FoldClassV1,
}

/// Folds profile contributions and admitted exceptions into the effective constitution outputs.
pub fn compose_profile_runtime(
    context: &ApplicabilityContextV1,
    profile_set: &ProfileSetV1,
    rule_set: &CompositionRuleSetV1,
    contributions: &[ObligationContributionV1],
    exceptions: &[ProfileExceptionBundleV1],
    generated_at: impl Into<String>,
) -> Result<CompositionOutcomeV1, CompositionError> {
    if profile_set.applicability_context_ref != context.applicability_context_id {
        return Err(CompositionError::ApplicabilityMismatch);
    }

    let generated_at = generated_at.into();
    let active_exceptions = active_exceptions(context, exceptions);

    let mut grouped: BTreeMap<GroupKey, Vec<&ObligationContributionV1>> = BTreeMap::new();
    for contribution in contributions {
        grouped
            .entry(GroupKey {
                family: contribution.obligation_family.clone(),
                key: contribution.obligation_key.clone(),
                output_kind: contribution.output_kind,
                fold_class: contribution.fold_class,
            })
            .or_default()
            .push(contribution);
    }

    let mut entries = Vec::new();
    let mut block_entries = Vec::new();
    let mut conflicts = Vec::new();

    for (group_key, group) in grouped {
        let matched_exception_ids = matched_exception_ids(&group, &active_exceptions, rule_set);
        let matched_exception_classes =
            matched_exception_classes(&group, &active_exceptions, rule_set);
        let blocking_contributions: Vec<_> = group
            .iter()
            .copied()
            .filter(|entry| entry.blocking)
            .collect();

        match group_key.fold_class {
            FoldClassV1::Union => {
                let string_values = union_values(&group);
                let numeric_value = single_numeric_if_all_equal(&group);
                let expiry_at = earliest_expiry_value(&group);
                if string_values.is_empty() && numeric_value.is_none() && expiry_at.is_none() {
                    continue;
                }
                entries.push(compiled_entry_from_group(
                    &group_key,
                    &group,
                    string_values,
                    numeric_value,
                    expiry_at,
                    !blocking_contributions.is_empty(),
                    matched_exception_ids,
                ));
            }
            FoldClassV1::Intersection => {
                let intersection = intersect_values(&group);
                if intersection.is_empty()
                    && group.iter().any(|entry| !entry.string_values.is_empty())
                {
                    if matched_exception_classes.is_empty() {
                        conflicts.push(conflict_from_group(
                            &group_key,
                            &group,
                            rule_set,
                            "empty_intersection",
                        ));
                    } else {
                        entries.push(compiled_entry_from_group(
                            &group_key,
                            &group,
                            union_values(&group),
                            single_numeric_if_all_equal(&group),
                            earliest_expiry_value(&group),
                            false,
                            matched_exception_ids,
                        ));
                    }
                } else if !intersection.is_empty() {
                    entries.push(compiled_entry_from_group(
                        &group_key,
                        &group,
                        intersection,
                        single_numeric_if_all_equal(&group),
                        earliest_expiry_value(&group),
                        false,
                        matched_exception_ids,
                    ));
                }
            }
            FoldClassV1::MinOfMaxima => {
                if let Some(value) = min_numeric(&group) {
                    entries.push(compiled_entry_from_group(
                        &group_key,
                        &group,
                        Vec::new(),
                        Some(value),
                        None,
                        false,
                        matched_exception_ids,
                    ));
                } else if matched_exception_classes.is_empty() && has_incomparable_strings(&group) {
                    conflicts.push(conflict_from_group(
                        &group_key,
                        &group,
                        rule_set,
                        "unsupported_numeric_fold",
                    ));
                }
            }
            FoldClassV1::MaxOfMinima => {
                if let Some(value) = max_numeric(&group) {
                    entries.push(compiled_entry_from_group(
                        &group_key,
                        &group,
                        Vec::new(),
                        Some(value),
                        None,
                        false,
                        matched_exception_ids,
                    ));
                } else if matched_exception_classes.is_empty() && has_incomparable_strings(&group) {
                    conflicts.push(conflict_from_group(
                        &group_key,
                        &group,
                        rule_set,
                        "unsupported_numeric_fold",
                    ));
                }
            }
            FoldClassV1::EarliestExpiry => {
                if let Some(expiry_at) = earliest_expiry_value(&group) {
                    entries.push(compiled_entry_from_group(
                        &group_key,
                        &group,
                        Vec::new(),
                        None,
                        Some(expiry_at),
                        false,
                        matched_exception_ids,
                    ));
                }
            }
            FoldClassV1::ConflictIfDifferent => {
                let values = normalized_group_values(&group);
                if values.len() > 1 && matched_exception_classes.is_empty() {
                    conflicts.push(conflict_from_group(
                        &group_key,
                        &group,
                        rule_set,
                        "incomparable_values",
                    ));
                } else if !values.is_empty() || single_numeric_if_all_equal(&group).is_some() {
                    entries.push(compiled_entry_from_group(
                        &group_key,
                        &group,
                        values,
                        single_numeric_if_all_equal(&group),
                        earliest_expiry_value(&group),
                        false,
                        matched_exception_ids,
                    ));
                }
            }
            FoldClassV1::BlockDominant => {
                if !blocking_contributions.is_empty() && matched_exception_classes.is_empty() {
                    block_entries.push(BlockEntryV1 {
                        block_key: format!("{}:{}", group_key.family, group_key.key),
                        reason: group
                            .iter()
                            .map(|entry| entry.explanation.clone())
                            .collect::<Vec<_>>()
                            .join("; "),
                        source_profile_refs: source_profile_refs(&group),
                        admissible_exception_classes: admissible_exception_classes(
                            &group, rule_set,
                        ),
                    });
                } else {
                    let string_values = union_values(&group);
                    let numeric_value = single_numeric_if_all_equal(&group);
                    let expiry_at = earliest_expiry_value(&group);
                    if !string_values.is_empty() || numeric_value.is_some() || expiry_at.is_some() {
                        entries.push(compiled_entry_from_group(
                            &group_key,
                            &group,
                            string_values,
                            numeric_value,
                            expiry_at,
                            false,
                            matched_exception_ids,
                        ));
                    }
                }
            }
        }
    }

    entries.sort_by(|a, b| {
        (
            &a.output_kind,
            &a.obligation_family,
            &a.obligation_key,
            &a.string_values,
        )
            .cmp(&(
                &b.output_kind,
                &b.obligation_family,
                &b.obligation_key,
                &b.string_values,
            ))
    });
    block_entries.sort_by(|a, b| a.block_key.cmp(&b.block_key));
    conflicts.sort_by(|a, b| {
        (&a.obligation_family, &a.obligation_key, &a.conflict_class).cmp(&(
            &b.obligation_family,
            &b.obligation_key,
            &b.conflict_class,
        ))
    });

    let hard_block_summary = block_entries
        .iter()
        .map(|entry| entry.block_key.clone())
        .collect::<Vec<_>>();
    let active_warning_summary = entries
        .iter()
        .filter(|entry| entry.output_kind == CompiledObligationKindV1::Warning)
        .flat_map(|entry| entry.string_values.clone())
        .collect::<Vec<_>>();
    let approval_requirements =
        collect_strings_by_kind(&entries, CompiledObligationKindV1::Approval);
    let disclosure_obligations =
        collect_strings_by_kind(&entries, CompiledObligationKindV1::Disclosure);
    let residency_obligations =
        collect_strings_by_kind(&entries, CompiledObligationKindV1::Residency);
    let tenancy_obligations = collect_strings_by_kind(&entries, CompiledObligationKindV1::Tenancy);
    let assurance_obligations =
        collect_strings_by_kind(&entries, CompiledObligationKindV1::Assurance);
    let monitoring_obligations =
        collect_strings_by_kind(&entries, CompiledObligationKindV1::Monitor);
    let continuity_obligations =
        collect_strings_by_kind(&entries, CompiledObligationKindV1::Continuity);
    let effect_obligations = collect_strings_by_kind(&entries, CompiledObligationKindV1::Effect);
    let delegation_obligations =
        collect_strings_by_kind(&entries, CompiledObligationKindV1::Delegation);
    let replay_obligations = collect_strings_by_kind(&entries, CompiledObligationKindV1::Replay);
    let required_checks = collect_strings_by_kind(&entries, CompiledObligationKindV1::Check);
    let required_monitors = monitoring_obligations.clone();
    let required_evidence_obligations =
        collect_strings_by_kind(&entries, CompiledObligationKindV1::Evidence);
    let required_disclosure_obligations = disclosure_obligations.clone();
    let required_rollback_obligations =
        collect_strings_by_kind(&entries, CompiledObligationKindV1::Rollback);
    let required_compensation_obligations =
        collect_strings_by_kind(&entries, CompiledObligationKindV1::Compensation);
    let required_post_hoc_review_obligations =
        collect_strings_by_kind(&entries, CompiledObligationKindV1::PostHocReview);
    let blocking_conflicts = conflicts.iter().any(|conflict| conflict.blocking);
    let current_mode_classification = if !hard_block_summary.is_empty() || blocking_conflicts {
        ConstitutionModeV1::Blocked
    } else if context.incident_mode.as_deref().unwrap_or("normal") != "normal" {
        ConstitutionModeV1::Incident
    } else {
        ConstitutionModeV1::Normal
    };

    let effective_constitution_id = EffectiveConstitutionId::generate();
    let compiled_obligation_set_id = CompiledObligationSetId::generate();
    let conflict_set_id = if conflicts.is_empty() {
        None
    } else {
        Some(CompositionConflictSetId::generate())
    };

    let effective_constitution = EffectiveConstitutionV1 {
        schema_version: EFFECTIVE_CONSTITUTION_V1_SCHEMA.into(),
        effective_constitution_id: effective_constitution_id.clone(),
        applicability_context_ref: context.applicability_context_id.clone(),
        profile_set_ref: profile_set.profile_set_id.clone(),
        composition_rule_set_ref: rule_set.composition_rule_set_id.clone(),
        base_law_refs: vec!["v25_effective_constitution".into()],
        doctrine_refs: vec![REFERENCE_EVALUATOR_VERSION_V1.into()],
        admitted_profile_refs: profile_set.all_profile_refs(),
        admitted_exception_refs: active_exceptions
            .iter()
            .map(|exception| exception.profile_exception_bundle_id.clone())
            .collect(),
        current_mode_classification,
        hard_block_summary,
        active_warning_summary,
        approval_requirements,
        disclosure_obligations,
        residency_obligations,
        tenancy_obligations,
        assurance_obligations,
        monitoring_obligations,
        continuity_obligations,
        effect_obligations,
        delegation_obligations,
        replay_obligations,
        valid_as_of: context.valid_as_of.clone(),
        recorded_as_of: context.recorded_as_of.clone(),
        explanation_summary: explanation_summary(context, &block_entries, &conflicts),
    };

    let compiled_obligation_set = CompiledObligationSetV1 {
        schema_version: COMPILED_OBLIGATION_SET_V1_SCHEMA.into(),
        compiled_obligation_set_id: compiled_obligation_set_id.clone(),
        effective_constitution_ref: effective_constitution_id.clone(),
        obligation_entries: entries,
        block_entries,
        required_approvals: effective_constitution.approval_requirements.clone(),
        required_checks,
        required_monitors,
        required_evidence_obligations,
        required_disclosure_obligations,
        required_rollback_obligations,
        required_compensation_obligations,
        required_post_hoc_review_obligations,
        promotion_eligibility_summary: eligibility_summary(
            effective_constitution.current_mode_classification,
            conflicts.iter().any(|entry| entry.blocking),
            !effective_constitution.approval_requirements.is_empty(),
        ),
        release_eligibility_summary: eligibility_summary(
            effective_constitution.current_mode_classification,
            conflicts.iter().any(|entry| entry.blocking),
            !effective_constitution.approval_requirements.is_empty(),
        ),
        effect_eligibility_summary: eligibility_summary(
            effective_constitution.current_mode_classification,
            conflicts.iter().any(|entry| entry.blocking),
            !effective_constitution.approval_requirements.is_empty(),
        ),
        unresolved_dependency_refs: conflicts
            .iter()
            .map(|conflict| format!("{}:{}", conflict.obligation_family, conflict.obligation_key))
            .collect(),
    };

    let input_digest = ContentDigest::compute_json(&serde_json::json!({
        "context": context,
        "profile_set": profile_set,
        "rule_set": rule_set,
        "contributions": contributions,
        "active_exception_ids": active_exceptions
            .iter()
            .map(|exception| exception.profile_exception_bundle_id.to_string())
            .collect::<Vec<_>>(),
    }))
    .map_err(|err| CompositionError::DigestFailed {
        reason: err.to_string(),
    })?;

    let receipt_id = CompositionReceiptId::generate();
    let receipt = CompositionReceiptV1 {
        schema_version: COMPOSITION_RECEIPT_V1_SCHEMA.into(),
        composition_receipt_id: receipt_id.clone(),
        applicability_context_ref: context.applicability_context_id.clone(),
        profile_set_ref: profile_set.profile_set_id.clone(),
        composition_rule_set_ref: rule_set.composition_rule_set_id.clone(),
        effective_constitution_ref: Some(effective_constitution_id),
        compiled_obligation_set_ref: Some(compiled_obligation_set_id),
        composition_conflict_set_ref: conflict_set_id.clone(),
        degradation_markers: if conflicts.is_empty() {
            Vec::new()
        } else {
            vec!["conflict_set_emitted".into()]
        },
        execution_context_ref: context.execution_context_ref.clone(),
        replay_handle: format!(
            "profile-runtime:{}:{}:{}",
            context.applicability_context_id,
            profile_set.profile_set_id,
            rule_set.composition_rule_set_id
        ),
        normalized_input_digest: input_digest,
        generated_at: generated_at.clone(),
    };

    let conflict_set = if conflicts.is_empty() {
        None
    } else {
        Some(CompositionConflictSetV1 {
            schema_version: COMPOSITION_CONFLICT_SET_V1_SCHEMA.into(),
            composition_conflict_set_id: conflict_set_id.ok_or_else(|| CompositionError::DigestFailed {
                reason: "conflict_set_id is None despite non-empty conflicts".into(),
            })?,
            effective_constitution_ref: Some(
                effective_constitution.effective_constitution_id.clone(),
            ),
            composition_receipt_ref: Some(receipt_id),
            conflicts,
            blocking: matches!(
                effective_constitution.current_mode_classification,
                ConstitutionModeV1::Blocked
            ),
        })
    };

    Ok(CompositionOutcomeV1 {
        receipt,
        effective_constitution,
        compiled_obligation_set,
        conflict_set,
    })
}

/// Computes the observable policy delta between two effective constitutions and obligation sets.
pub fn diff_policy_impact(
    from_constitution: &EffectiveConstitutionV1,
    from_obligations: &CompiledObligationSetV1,
    to_constitution: &EffectiveConstitutionV1,
    to_obligations: &CompiledObligationSetV1,
    simulation_only: bool,
) -> PolicyImpactDiffV1 {
    let from_map = entry_map(from_obligations);
    let to_map = entry_map(to_obligations);

    let mut changed_families = BTreeSet::new();
    let mut unchanged_families = BTreeSet::new();
    let all_keys = from_map
        .keys()
        .chain(to_map.keys())
        .cloned()
        .collect::<BTreeSet<_>>();

    for key in all_keys {
        match (from_map.get(&key), to_map.get(&key)) {
            (Some(left), Some(right)) if left == right => {
                unchanged_families.insert(key.0.clone());
            }
            _ => {
                changed_families.insert(key.0.clone());
            }
        }
    }

    let from_blocks = from_obligations
        .block_entries
        .iter()
        .map(|entry| entry.block_key.clone())
        .collect::<BTreeSet<_>>();
    let to_blocks = to_obligations
        .block_entries
        .iter()
        .map(|entry| entry.block_key.clone())
        .collect::<BTreeSet<_>>();

    let newly_blocked_paths = to_blocks
        .difference(&from_blocks)
        .cloned()
        .collect::<Vec<_>>();
    let newly_admitted_paths = from_blocks
        .difference(&to_blocks)
        .cloned()
        .collect::<Vec<_>>();

    let changed_approval_behavior = symmetric_difference_strings(
        &from_obligations.required_approvals,
        &to_obligations.required_approvals,
    );
    let changed_monitoring_behavior = symmetric_difference_strings(
        &from_obligations.required_monitors,
        &to_obligations.required_monitors,
    );
    let changed_disclosure_behavior = symmetric_difference_strings(
        &from_obligations.required_disclosure_obligations,
        &to_obligations.required_disclosure_obligations,
    );
    let changed_residency_behavior = symmetric_difference_strings(
        &from_constitution.residency_obligations,
        &to_constitution.residency_obligations,
    );
    let changed_incident_behavior = if from_constitution.current_mode_classification
        != to_constitution.current_mode_classification
    {
        vec![format!(
            "mode:{:?}->{:?}",
            from_constitution.current_mode_classification,
            to_constitution.current_mode_classification
        )]
    } else {
        symmetric_difference_strings(
            &from_constitution.continuity_obligations,
            &to_constitution.continuity_obligations,
        )
    };

    let mut migration_consequences = Vec::new();
    if !newly_blocked_paths.is_empty() {
        migration_consequences.push("additional paths now require review or exception".into());
    }
    if !newly_admitted_paths.is_empty() {
        migration_consequences
            .push("previously blocked paths are now conditionally admissible".into());
    }
    if !changed_monitoring_behavior.is_empty() {
        migration_consequences.push("monitoring plan update required".into());
    }

    PolicyImpactDiffV1 {
        schema_version: POLICY_IMPACT_DIFF_V1_SCHEMA.into(),
        policy_impact_diff_id: PolicyImpactDiffId::generate(),
        from_effective_constitution_ref: from_constitution.effective_constitution_id.clone(),
        to_effective_constitution_ref: to_constitution.effective_constitution_id.clone(),
        changed_obligation_families: changed_families.into_iter().collect(),
        unchanged_obligation_families: unchanged_families.into_iter().collect(),
        newly_blocked_paths,
        newly_admitted_paths,
        changed_approval_behavior,
        changed_monitoring_behavior,
        changed_disclosure_behavior,
        changed_residency_behavior,
        changed_incident_behavior,
        migration_consequences,
        simulation_only,
        explanation_summary: "policy impact diff derived from compiled obligation deltas".into(),
    }
}

fn active_exceptions<'a>(
    context: &ApplicabilityContextV1,
    exceptions: &'a [ProfileExceptionBundleV1],
) -> Vec<&'a ProfileExceptionBundleV1> {
    let mut active = exceptions
        .iter()
        .filter(|exception| exception.is_active_as_of(&context.valid_as_of))
        .filter(|exception| {
            exception.affected_context_refs.is_empty()
                || exception
                    .affected_context_refs
                    .iter()
                    .any(|id| id == &context.applicability_context_id)
        })
        .collect::<Vec<_>>();
    active.sort_by(|a, b| {
        a.profile_exception_bundle_id
            .cmp(&b.profile_exception_bundle_id)
    });
    active
}

fn matched_exception_ids(
    group: &[&ObligationContributionV1],
    active_exceptions: &[&ProfileExceptionBundleV1],
    rule_set: &CompositionRuleSetV1,
) -> Vec<stack_ids::ProfileExceptionBundleId> {
    let mut ids = active_exceptions
        .iter()
        .filter(|exception| exception_matches_group(group, exception, rule_set))
        .map(|exception| exception.profile_exception_bundle_id.clone())
        .collect::<Vec<_>>();
    ids.sort();
    ids.dedup();
    ids
}

fn matched_exception_classes(
    group: &[&ObligationContributionV1],
    active_exceptions: &[&ProfileExceptionBundleV1],
    rule_set: &CompositionRuleSetV1,
) -> Vec<String> {
    let mut classes = active_exceptions
        .iter()
        .filter(|exception| exception_matches_group(group, exception, rule_set))
        .map(|exception| exception.exception_class.clone())
        .collect::<Vec<_>>();
    classes.sort();
    classes.dedup();
    classes
}

fn exception_matches_group(
    group: &[&ObligationContributionV1],
    exception: &ProfileExceptionBundleV1,
    rule_set: &CompositionRuleSetV1,
) -> bool {
    group.iter().any(|entry| {
        let allowed_by_group = entry
            .admissible_exception_classes
            .iter()
            .any(|class| class == &exception.exception_class);
        let allowed_by_rule = rule_set
            .family_rule(&entry.obligation_family)
            .map(|rule| {
                rule.admissible_exception_classes
                    .iter()
                    .any(|class| class == &exception.exception_class)
            })
            .unwrap_or(false);
        allowed_by_group || allowed_by_rule
    })
}

fn compiled_entry_from_group(
    group_key: &GroupKey,
    group: &[&ObligationContributionV1],
    string_values: Vec<String>,
    numeric_value: Option<i64>,
    expiry_at: Option<String>,
    blocking: bool,
    source_exception_refs: Vec<stack_ids::ProfileExceptionBundleId>,
) -> CompiledObligationEntryV1 {
    CompiledObligationEntryV1 {
        obligation_entry_id: format!("{}:{}", group_key.family, group_key.key),
        obligation_family: group_key.family.clone(),
        obligation_key: group_key.key.clone(),
        output_kind: group_key.output_kind,
        fold_class: group_key.fold_class,
        string_values,
        numeric_value,
        expiry_at,
        blocking,
        source_profile_refs: source_profile_refs(group),
        source_exception_refs,
        explanation: group
            .iter()
            .map(|entry| entry.explanation.clone())
            .collect::<Vec<_>>()
            .join("; "),
    }
}

fn conflict_from_group(
    group_key: &GroupKey,
    group: &[&ObligationContributionV1],
    rule_set: &CompositionRuleSetV1,
    conflict_class: &str,
) -> CompositionConflictV1 {
    let conflict_rule = rule_set.conflict_class(conflict_class);
    CompositionConflictV1 {
        obligation_family: group_key.family.clone(),
        obligation_key: group_key.key.clone(),
        conflict_class: conflict_class.into(),
        blocking: conflict_rule.map(|rule| rule.blocking).unwrap_or(true),
        source_profile_refs: source_profile_refs(group),
        implicated_values: normalized_group_values(group),
        admissible_exception_classes: admissible_exception_classes(group, rule_set),
        review_path: conflict_rule
            .map(|rule| rule.review_path.clone())
            .unwrap_or_else(|| "human_constitution_review".into()),
        explanation: conflict_rule
            .map(|rule| rule.explanation.clone())
            .unwrap_or_else(|| {
                group
                    .iter()
                    .map(|entry| entry.explanation.clone())
                    .collect::<Vec<_>>()
                    .join("; ")
            }),
    }
}

fn admissible_exception_classes(
    group: &[&ObligationContributionV1],
    rule_set: &CompositionRuleSetV1,
) -> Vec<String> {
    let mut classes = group
        .iter()
        .flat_map(|entry| entry.admissible_exception_classes.clone())
        .collect::<Vec<_>>();
    for family in group.iter().map(|entry| entry.obligation_family.as_str()) {
        if let Some(rule) = rule_set.family_rule(family) {
            classes.extend(rule.admissible_exception_classes.iter().cloned());
        }
    }
    classes.sort();
    classes.dedup();
    classes
}

fn source_profile_refs(group: &[&ObligationContributionV1]) -> Vec<String> {
    let mut refs = group
        .iter()
        .map(|entry| entry.source_profile_ref.clone())
        .collect::<Vec<_>>();
    refs.sort();
    refs.dedup();
    refs
}

fn normalized_group_values(group: &[&ObligationContributionV1]) -> Vec<String> {
    let mut values = union_values(group);
    if values.is_empty() {
        for entry in group {
            if let Some(value) = entry.numeric_value {
                values.push(value.to_string());
            }
            if let Some(value) = &entry.expiry_at {
                values.push(value.clone());
            }
        }
    }
    values.sort();
    values.dedup();
    values
}

fn union_values(group: &[&ObligationContributionV1]) -> Vec<String> {
    let mut values = group
        .iter()
        .flat_map(|entry| entry.string_values.iter().cloned())
        .collect::<Vec<_>>();
    values.sort();
    values.dedup();
    values
}

fn intersect_values(group: &[&ObligationContributionV1]) -> Vec<String> {
    let sets = group
        .iter()
        .filter(|entry| !entry.string_values.is_empty())
        .map(|entry| entry.string_values.iter().cloned().collect::<BTreeSet<_>>())
        .collect::<Vec<_>>();
    if sets.is_empty() {
        return Vec::new();
    }
    let mut iter = sets.into_iter();
    let Some(first) = iter.next() else {
        return Vec::new();
    };
    let intersection = iter.fold(first, |acc, item| {
        acc.intersection(&item).cloned().collect()
    });
    intersection.into_iter().collect()
}

fn numeric_value(entry: &ObligationContributionV1) -> Option<i64> {
    entry.numeric_value.or_else(|| {
        entry
            .string_values
            .first()
            .and_then(|value| value.parse::<i64>().ok())
    })
}

fn min_numeric(group: &[&ObligationContributionV1]) -> Option<i64> {
    group.iter().filter_map(|entry| numeric_value(entry)).min()
}

fn max_numeric(group: &[&ObligationContributionV1]) -> Option<i64> {
    group.iter().filter_map(|entry| numeric_value(entry)).max()
}

fn single_numeric_if_all_equal(group: &[&ObligationContributionV1]) -> Option<i64> {
    let mut values = group
        .iter()
        .filter_map(|entry| numeric_value(entry))
        .collect::<Vec<_>>();
    values.sort();
    values.dedup();
    match values.as_slice() {
        [value] => Some(*value),
        _ => None,
    }
}

fn earliest_expiry_value(group: &[&ObligationContributionV1]) -> Option<String> {
    group
        .iter()
        .filter_map(|entry| entry.expiry_at.clone())
        .min()
}

fn has_incomparable_strings(group: &[&ObligationContributionV1]) -> bool {
    let values = union_values(group);
    values.len() > 1
}

fn collect_strings_by_kind(
    entries: &[CompiledObligationEntryV1],
    kind: CompiledObligationKindV1,
) -> Vec<String> {
    let mut values = entries
        .iter()
        .filter(|entry| entry.output_kind == kind)
        .flat_map(|entry| {
            let mut strings = entry.string_values.clone();
            if let Some(value) = entry.numeric_value {
                strings.push(value.to_string());
            }
            if let Some(value) = &entry.expiry_at {
                strings.push(value.clone());
            }
            strings
        })
        .collect::<Vec<_>>();
    values.sort();
    values.dedup();
    values
}

fn explanation_summary(
    context: &ApplicabilityContextV1,
    block_entries: &[BlockEntryV1],
    conflicts: &[CompositionConflictV1],
) -> String {
    if !block_entries.is_empty() {
        format!(
            "context '{}' is blocked by {} hard blocks and {} conflicts",
            context.target_surface_kind,
            block_entries.len(),
            conflicts.len()
        )
    } else if !conflicts.is_empty() {
        format!(
            "context '{}' emitted {} composition conflicts",
            context.target_surface_kind,
            conflicts.len()
        )
    } else {
        format!(
            "context '{}' compiled without blocking conflicts",
            context.target_surface_kind
        )
    }
}

fn eligibility_summary(
    current_mode: ConstitutionModeV1,
    blocking_conflicts: bool,
    approval_required: bool,
) -> String {
    if matches!(current_mode, ConstitutionModeV1::Blocked) || blocking_conflicts {
        "blocked".into()
    } else if approval_required {
        "approval_required".into()
    } else {
        "eligible".into()
    }
}

fn entry_map(
    obligations: &CompiledObligationSetV1,
) -> BTreeMap<ObligationEntryMapKey, ObligationEntryMapValue> {
    obligations
        .obligation_entries
        .iter()
        .map(|entry| {
            (
                (
                    entry.obligation_family.clone(),
                    entry.obligation_key.clone(),
                ),
                (
                    entry.string_values.clone(),
                    entry.numeric_value,
                    entry.expiry_at.clone(),
                    entry.blocking,
                ),
            )
        })
        .collect()
}

fn symmetric_difference_strings(left: &[String], right: &[String]) -> Vec<String> {
    let left = left.iter().cloned().collect::<BTreeSet<_>>();
    let right = right.iter().cloned().collect::<BTreeSet<_>>();
    left.symmetric_difference(&right).cloned().collect()
}
