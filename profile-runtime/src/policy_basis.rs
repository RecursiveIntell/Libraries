//! Task-scoped projection of canonical profile composition outputs.
//!
//! This module does not fold profile contributions or mint authority. It binds
//! exact `profile-runtime` owner artifacts to the mission/task metadata needed by
//! downstream Ares, Graph, and native egress consumers.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use stack_ids::{
    CompiledObligationSetId, CompositionConflictSetId, CompositionReceiptId, ContentDigest,
    EffectiveConstitutionId,
};
use thiserror::Error;

use crate::{
    ApplicabilityContextV1, CompiledObligationSetV1, CompositionOutcomeV1, CompositionRuleSetV1,
    ConstitutionModeV1, ProfileSetV1,
};

pub const RESOLVED_POLICY_BASIS_V1_SCHEMA: &str = "profile-runtime.resolved-policy-basis/v1";
const MAX_REFERENCE_BYTES: usize = 4096;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PolicyBasisTaskContextV1 {
    pub mission_ref: String,
    pub task_ref: String,
    pub instruction_ref: String,
    pub instruction_digest: String,
    pub source_revision: String,
    pub authority_snapshot_ref: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub unresolved_instruction_obligations: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PolicyBasisOwnerRefsV1 {
    pub composition_receipt_ref: CompositionReceiptId,
    pub composition_receipt_digest: ContentDigest,
    pub effective_constitution_ref: EffectiveConstitutionId,
    pub effective_constitution_digest: ContentDigest,
    pub compiled_obligation_set_ref: CompiledObligationSetId,
    pub compiled_obligation_set_digest: ContentDigest,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub conflict_set_ref: Option<CompositionConflictSetId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub conflict_set_digest: Option<ContentDigest>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PolicyBasisLimitsV1 {
    pub max_input_tokens: u64,
    pub max_output_tokens: u64,
    pub max_attempts: u64,
    pub max_concurrency: u64,
    pub max_wall_time_ms: u64,
    pub max_artifact_bytes: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum PolicyBasisStatusV1 {
    Admitted,
    Blocked,
}

/// Immutable, nonauthorizing task policy projection.
///
/// Owner artifact digests are computed from exact `profile-runtime` values.
/// Downstream consumers carry those digests opaquely and must resolve them
/// against the current owner before any effect.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ResolvedPolicyBasisV1 {
    pub schema: String,
    pub basis_ref: String,
    pub mission_ref: String,
    pub task_ref: String,
    pub instruction_ref: String,
    pub instruction_digest: String,
    pub source_revision: String,
    pub authority_snapshot_ref: String,
    pub applicability_context_ref: stack_ids::ApplicabilityContextId,
    pub profile_set_ref: stack_ids::ProfileSetId,
    pub composition_rule_set_ref: stack_ids::CompositionRuleSetId,
    pub owner_refs: PolicyBasisOwnerRefsV1,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub admitted_profile_refs: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub admitted_exception_refs: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub allowed_route_classes: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub allowed_disclosure_classes: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub allowed_effect_classes: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub mandatory_obligations: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub required_check_families: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub residual_exception_obligations: Vec<String>,
    pub limits: PolicyBasisLimitsV1,
    pub not_before: String,
    pub not_after: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub authority_refs: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub provenance_refs: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub unsupported_policy_families: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub degradation_markers: Vec<String>,
    pub status: PolicyBasisStatusV1,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub blocking_reasons: Vec<String>,
    pub basis_digest: ContentDigest,
}

impl ResolvedPolicyBasisV1 {
    /// Recompute the task projection digest and validate closed structural fields.
    pub fn validate(&self) -> Result<(), PolicyBasisError> {
        if self.schema != RESOLVED_POLICY_BASIS_V1_SCHEMA {
            return Err(PolicyBasisError::InvalidProjection {
                reason: "unsupported schema".into(),
            });
        }
        for (name, value) in [
            ("basis_ref", self.basis_ref.as_str()),
            ("mission_ref", self.mission_ref.as_str()),
            ("task_ref", self.task_ref.as_str()),
            ("instruction_ref", self.instruction_ref.as_str()),
            ("instruction_digest", self.instruction_digest.as_str()),
            ("source_revision", self.source_revision.as_str()),
            (
                "authority_snapshot_ref",
                self.authority_snapshot_ref.as_str(),
            ),
            ("not_before", self.not_before.as_str()),
            ("not_after", self.not_after.as_str()),
        ] {
            validate_reference(name, value)?;
        }
        validate_time_window(&self.not_before, &self.not_after)?;
        if !is_algorithm_digest(&self.instruction_digest) {
            return Err(PolicyBasisError::InvalidProjection {
                reason: "instruction_digest is not algorithm-qualified".into(),
            });
        }
        let should_be_admitted = self.blocking_reasons.is_empty()
            && !self.allowed_route_classes.is_empty()
            && !self.allowed_disclosure_classes.is_empty()
            && !self.allowed_effect_classes.is_empty()
            && self.limits.all_positive();
        if should_be_admitted != (self.status == PolicyBasisStatusV1::Admitted) {
            return Err(PolicyBasisError::InvalidProjection {
                reason: "status does not match blocking state".into(),
            });
        }
        if self.basis_digest != self.expected_digest()? {
            return Err(PolicyBasisError::InvalidProjection {
                reason: "basis digest mismatch".into(),
            });
        }
        Ok(())
    }

    fn expected_digest(&self) -> Result<ContentDigest, PolicyBasisError> {
        let mut value = serde_json::to_value(self).map_err(digest_error)?;
        let object = value
            .as_object_mut()
            .ok_or_else(|| PolicyBasisError::InvalidProjection {
                reason: "basis must serialize as an object".into(),
            })?;
        object.remove("basis_digest");
        ContentDigest::compute_json(&value).map_err(digest_error)
    }
}

impl PolicyBasisLimitsV1 {
    fn all_positive(&self) -> bool {
        self.max_input_tokens > 0
            && self.max_output_tokens > 0
            && self.max_attempts > 0
            && self.max_concurrency > 0
            && self.max_wall_time_ms > 0
            && self.max_artifact_bytes > 0
    }
}

#[derive(Debug, Error)]
pub enum PolicyBasisError {
    #[error("profile-runtime owner references do not match")]
    OwnerReferenceMismatch,
    #[error("resolved policy projection is invalid: {reason}")]
    InvalidProjection { reason: String },
    #[error("resolved policy projection digest failed: {reason}")]
    DigestFailed { reason: String },
}

impl PolicyBasisError {
    /// Returns the stable machine-readable error category for this policy-basis failure.
    pub fn kind(&self) -> &'static str {
        match self {
            Self::OwnerReferenceMismatch => "owner_reference_mismatch",
            Self::InvalidProjection { .. } => "invalid_projection",
            Self::DigestFailed { .. } => "digest_failed",
        }
    }
}

/// Project exact current owner outputs into one task-scoped, nonauthorizing basis.
pub fn resolve_policy_basis(
    context: &ApplicabilityContextV1,
    profile_set: &ProfileSetV1,
    rule_set: &CompositionRuleSetV1,
    outcome: &CompositionOutcomeV1,
    task: PolicyBasisTaskContextV1,
) -> Result<ResolvedPolicyBasisV1, PolicyBasisError> {
    validate_owner_references(context, profile_set, rule_set, outcome)?;
    validate_task(&task)?;

    let obligations = &outcome.compiled_obligation_set;
    let mut blocking_reasons = obligations
        .block_entries
        .iter()
        .map(|entry| entry.block_key.clone())
        .collect::<Vec<_>>();
    if let Some(conflicts) = &outcome.conflict_set {
        blocking_reasons.extend(
            conflicts
                .conflicts
                .iter()
                .filter(|conflict| conflict.blocking)
                .map(|conflict| {
                    format!(
                        "conflict:{}:{}",
                        conflict.obligation_family, conflict.obligation_key
                    )
                }),
        );
    }
    if outcome.effective_constitution.current_mode_classification == ConstitutionModeV1::Blocked {
        blocking_reasons.push("effective_constitution:blocked".into());
    }

    let allowed_route_classes = string_values(
        obligations,
        "egress.allowed_route_classes",
        "route",
        &mut blocking_reasons,
    );
    let allowed_disclosure_classes = string_values(
        obligations,
        "disclosure.allowed_classes",
        "classification",
        &mut blocking_reasons,
    );
    let allowed_effect_classes = string_values(
        obligations,
        "effect.allowed_classes",
        "effect",
        &mut blocking_reasons,
    );
    let limits = PolicyBasisLimitsV1 {
        max_input_tokens: numeric_value(
            obligations,
            "budget.max_input_tokens",
            "input",
            &mut blocking_reasons,
        ),
        max_output_tokens: numeric_value(
            obligations,
            "budget.max_output_tokens",
            "output",
            &mut blocking_reasons,
        ),
        max_attempts: numeric_value(
            obligations,
            "budget.max_attempts",
            "attempts",
            &mut blocking_reasons,
        ),
        max_concurrency: numeric_value(
            obligations,
            "budget.max_concurrency",
            "concurrency",
            &mut blocking_reasons,
        ),
        max_wall_time_ms: numeric_value(
            obligations,
            "budget.max_wall_time_ms",
            "wall_time",
            &mut blocking_reasons,
        ),
        max_artifact_bytes: numeric_value(
            obligations,
            "budget.max_artifact_bytes",
            "artifact_bytes",
            &mut blocking_reasons,
        ),
    };
    let not_after = expiry_value(
        obligations,
        "policy.not_after",
        "expiry",
        &mut blocking_reasons,
    );

    let owner_refs = PolicyBasisOwnerRefsV1 {
        composition_receipt_ref: outcome.receipt.composition_receipt_id.clone(),
        composition_receipt_digest: digest_artifact(&outcome.receipt)?,
        effective_constitution_ref: outcome
            .effective_constitution
            .effective_constitution_id
            .clone(),
        effective_constitution_digest: digest_artifact(&outcome.effective_constitution)?,
        compiled_obligation_set_ref: obligations.compiled_obligation_set_id.clone(),
        compiled_obligation_set_digest: digest_artifact(obligations)?,
        conflict_set_ref: outcome
            .conflict_set
            .as_ref()
            .map(|conflicts| conflicts.composition_conflict_set_id.clone()),
        conflict_set_digest: outcome
            .conflict_set
            .as_ref()
            .map(digest_artifact)
            .transpose()?,
    };

    let mut mandatory_obligations = obligations
        .obligation_entries
        .iter()
        .map(|entry| entry.obligation_entry_id.clone())
        .collect::<Vec<_>>();
    mandatory_obligations.extend(obligations.required_checks.iter().cloned());
    mandatory_obligations.extend(obligations.required_evidence_obligations.iter().cloned());
    mandatory_obligations.extend(obligations.required_disclosure_obligations.iter().cloned());
    mandatory_obligations.extend(obligations.required_rollback_obligations.iter().cloned());
    mandatory_obligations.extend(
        obligations
            .required_compensation_obligations
            .iter()
            .cloned(),
    );
    mandatory_obligations.extend(
        obligations
            .required_post_hoc_review_obligations
            .iter()
            .cloned(),
    );
    mandatory_obligations.extend(task.unresolved_instruction_obligations.iter().cloned());
    normalize(&mut mandatory_obligations);

    let mut authority_refs = profile_set.source_admission_refs.clone();
    authority_refs.push(task.authority_snapshot_ref.clone());
    authority_refs.extend(
        outcome
            .effective_constitution
            .admitted_exception_refs
            .iter()
            .map(ToString::to_string),
    );
    normalize(&mut authority_refs);

    let mut provenance_refs = outcome.effective_constitution.base_law_refs.clone();
    provenance_refs.extend(outcome.effective_constitution.doctrine_refs.iter().cloned());
    provenance_refs.extend(profile_set.source_admission_refs.iter().cloned());
    normalize(&mut provenance_refs);
    normalize(&mut blocking_reasons);

    let status = if blocking_reasons.is_empty() && limits.all_positive() {
        PolicyBasisStatusV1::Admitted
    } else {
        PolicyBasisStatusV1::Blocked
    };
    let mut basis = ResolvedPolicyBasisV1 {
        schema: RESOLVED_POLICY_BASIS_V1_SCHEMA.into(),
        basis_ref: format!("profile-runtime:{}", outcome.receipt.composition_receipt_id),
        mission_ref: task.mission_ref,
        task_ref: task.task_ref,
        instruction_ref: task.instruction_ref,
        instruction_digest: task.instruction_digest,
        source_revision: task.source_revision,
        authority_snapshot_ref: task.authority_snapshot_ref,
        applicability_context_ref: context.applicability_context_id.clone(),
        profile_set_ref: profile_set.profile_set_id.clone(),
        composition_rule_set_ref: rule_set.composition_rule_set_id.clone(),
        owner_refs,
        admitted_profile_refs: outcome.effective_constitution.admitted_profile_refs.clone(),
        admitted_exception_refs: outcome
            .effective_constitution
            .admitted_exception_refs
            .iter()
            .map(ToString::to_string)
            .collect(),
        allowed_route_classes,
        allowed_disclosure_classes,
        allowed_effect_classes,
        mandatory_obligations,
        required_check_families: obligations.required_checks.clone(),
        residual_exception_obligations: obligations.residual_exception_obligations.clone(),
        limits,
        not_before: context.valid_as_of.clone(),
        not_after,
        authority_refs,
        provenance_refs,
        unsupported_policy_families: Vec::new(),
        degradation_markers: outcome.receipt.degradation_markers.clone(),
        status,
        blocking_reasons,
        basis_digest: ContentDigest::compute(b"profile-runtime-policy-basis-unsealed"),
    };
    basis.basis_digest = basis.expected_digest()?;
    basis.validate()?;
    Ok(basis)
}

fn validate_owner_references(
    context: &ApplicabilityContextV1,
    profile_set: &ProfileSetV1,
    rule_set: &CompositionRuleSetV1,
    outcome: &CompositionOutcomeV1,
) -> Result<(), PolicyBasisError> {
    if profile_set.applicability_context_ref != context.applicability_context_id
        || outcome.receipt.applicability_context_ref != context.applicability_context_id
        || outcome.receipt.profile_set_ref != profile_set.profile_set_id
        || outcome.receipt.composition_rule_set_ref != rule_set.composition_rule_set_id
        || outcome.receipt.effective_constitution_ref.as_ref()
            != Some(&outcome.effective_constitution.effective_constitution_id)
        || outcome.receipt.compiled_obligation_set_ref.as_ref()
            != Some(&outcome.compiled_obligation_set.compiled_obligation_set_id)
        || outcome.compiled_obligation_set.effective_constitution_ref
            != outcome.effective_constitution.effective_constitution_id
        || outcome.effective_constitution.applicability_context_ref
            != context.applicability_context_id
        || outcome.effective_constitution.profile_set_ref != profile_set.profile_set_id
        || outcome.effective_constitution.composition_rule_set_ref
            != rule_set.composition_rule_set_id
        || outcome.receipt.composition_conflict_set_ref
            != outcome
                .conflict_set
                .as_ref()
                .map(|conflicts| conflicts.composition_conflict_set_id.clone())
    {
        return Err(PolicyBasisError::OwnerReferenceMismatch);
    }
    Ok(())
}

fn validate_task(task: &PolicyBasisTaskContextV1) -> Result<(), PolicyBasisError> {
    for (name, value) in [
        ("mission_ref", task.mission_ref.as_str()),
        ("task_ref", task.task_ref.as_str()),
        ("instruction_ref", task.instruction_ref.as_str()),
        ("instruction_digest", task.instruction_digest.as_str()),
        ("source_revision", task.source_revision.as_str()),
        (
            "authority_snapshot_ref",
            task.authority_snapshot_ref.as_str(),
        ),
    ] {
        validate_reference(name, value)?;
    }
    if !is_algorithm_digest(&task.instruction_digest) {
        return Err(PolicyBasisError::InvalidProjection {
            reason: "instruction digest is not algorithm-qualified".into(),
        });
    }
    Ok(())
}

fn string_values(
    obligations: &CompiledObligationSetV1,
    family: &str,
    key: &str,
    blocking: &mut Vec<String>,
) -> Vec<String> {
    let Some(entry) = exact_entry(obligations, family, key, blocking) else {
        return Vec::new();
    };
    let mut values = entry.string_values.clone();
    normalize(&mut values);
    if values.is_empty() {
        blocking.push(format!("empty:{family}:{key}"));
    }
    values
}

fn numeric_value(
    obligations: &CompiledObligationSetV1,
    family: &str,
    key: &str,
    blocking: &mut Vec<String>,
) -> u64 {
    let Some(entry) = exact_entry(obligations, family, key, blocking) else {
        return 0;
    };
    match entry
        .numeric_value
        .and_then(|value| u64::try_from(value).ok())
    {
        Some(value) if value > 0 => value,
        _ => {
            blocking.push(format!("invalid:{family}:{key}"));
            0
        }
    }
}

fn expiry_value(
    obligations: &CompiledObligationSetV1,
    family: &str,
    key: &str,
    blocking: &mut Vec<String>,
) -> String {
    let Some(entry) = exact_entry(obligations, family, key, blocking) else {
        return "blocked:missing-expiry".into();
    };
    match entry.expiry_at.as_ref() {
        Some(value) if chrono::DateTime::parse_from_rfc3339(value).is_ok() => value.clone(),
        _ => {
            blocking.push(format!("invalid:{family}:{key}"));
            "blocked:invalid-expiry".into()
        }
    }
}

fn exact_entry<'a>(
    obligations: &'a CompiledObligationSetV1,
    family: &str,
    key: &str,
    blocking: &mut Vec<String>,
) -> Option<&'a crate::CompiledObligationEntryV1> {
    let matches = obligations
        .obligation_entries
        .iter()
        .filter(|entry| entry.obligation_family == family && entry.obligation_key == key)
        .collect::<Vec<_>>();
    match matches.as_slice() {
        [entry] => Some(*entry),
        [] => {
            blocking.push(format!("missing:{family}:{key}"));
            None
        }
        _ => {
            blocking.push(format!("ambiguous:{family}:{key}"));
            None
        }
    }
}

fn validate_time_window(not_before: &str, not_after: &str) -> Result<(), PolicyBasisError> {
    let parse = |value: &str| {
        chrono::DateTime::parse_from_rfc3339(value).map_err(|_| {
            PolicyBasisError::InvalidProjection {
                reason: "policy validity timestamps must be RFC3339".into(),
            }
        })
    };
    if parse(not_before)? > parse(not_after)? {
        return Err(PolicyBasisError::InvalidProjection {
            reason: "policy validity window is expired or inverted".into(),
        });
    }
    Ok(())
}

fn digest_artifact<T: Serialize>(value: &T) -> Result<ContentDigest, PolicyBasisError> {
    ContentDigest::compute_json(value).map_err(digest_error)
}

fn digest_error(error: impl std::fmt::Display) -> PolicyBasisError {
    PolicyBasisError::DigestFailed {
        reason: error.to_string(),
    }
}

fn validate_reference(name: &str, value: &str) -> Result<(), PolicyBasisError> {
    if value.is_empty() || value.len() > MAX_REFERENCE_BYTES {
        return Err(PolicyBasisError::InvalidProjection {
            reason: format!("invalid {name}"),
        });
    }
    Ok(())
}

fn is_algorithm_digest(value: &str) -> bool {
    let Some((algorithm, hexadecimal)) = value.split_once(':') else {
        return false;
    };
    matches!(algorithm, "sha256" | "blake3")
        && hexadecimal.len() == 64
        && hexadecimal
            .chars()
            .all(|character| character.is_ascii_hexdigit())
}

fn normalize(values: &mut Vec<String>) {
    values.sort();
    values.dedup();
}
