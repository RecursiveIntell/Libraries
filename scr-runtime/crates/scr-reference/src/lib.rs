//! SCR-P0A reference evaluator entry point.
//!
//! The public path returns a receipt or an explicit error; it never returns a
//! naked boolean.

pub mod policy;

use policy::{parse_action, CanonicalPolicyV1, PolicyModelV1, SUPPORTED_HARD_RULE_IDS};
pub use scr_kernel::*;
use std::collections::{BTreeMap, BTreeSet};

pub const EVALUATOR_ALGORITHM_ID: &str = "scr-p0a-reference-v1";
const EVALUATOR_SOURCE_BYTES: &[u8] = include_bytes!("lib.rs");

pub fn load_policy_from_toml(source: &str) -> Result<CanonicalPolicyV1, ScrError> {
    PolicyModelV1::from_toml(source)?.canonicalize()
}

pub fn evaluate_with_policy(
    input: ControlEvaluationInputV1,
    policy: &CanonicalPolicyV1,
) -> Result<ControlDecisionReceiptV1, ScrError> {
    if policy.model.policy.algorithm_version != EVALUATOR_ALGORITHM_ID {
        return Err(ScrError::PolicyValidationFailed(format!(
            "unsupported policy algorithm_version: {}",
            policy.model.policy.algorithm_version
        )));
    }
    if policy.model.policy.domain != input.domain.policy_key() {
        return Err(ScrError::PolicyValidationFailed(format!(
            "policy domain mismatch: expected {}",
            input.domain.policy_key()
        )));
    }

    input.validate()?;
    let input_hash = hash_json(&input)?;
    validate_hard_rule_registry(&policy.model.hard_rules)?;
    let signals = SignalSet::from_input(&input);
    let axes = derive_axes(&signals)?;
    let derived_pressures = derive_pressures(&axes)?;
    let score_action = score_derived_action(&policy.model, &derived_pressures)?;
    let authority_basis = authority_basis(&input, &signals)?;
    let evidence_basis = evidence_basis(&input, &signals)?;

    let mut checked = Vec::new();
    let mut triggered = Vec::new();
    let mut floors = Vec::new();
    let mut rule_results = Vec::new();
    let mut candidates = Vec::new();
    let mut reason_codes = Vec::new();

    for rule_id in SUPPORTED_HARD_RULE_IDS {
        let rule = policy.model.hard_rules.get(*rule_id).ok_or_else(|| {
            ScrError::PolicyValidationFailed(format!("missing hard rule {rule_id}"))
        })?;
        if !rule.enabled {
            continue;
        }

        checked.push(rule_id.to_string());
        let is_triggered = hard_rule_triggered(rule_id, &signals);
        if is_triggered {
            triggered.push(rule_id.to_string());
            let reason = ReasonCode::new(rule.reason.clone())?;
            reason_codes.push(reason.clone());
            if let Some(action) = &rule.action {
                candidates.push(ResolvedCandidate {
                    action: parse_action(action)?,
                    reason: reason.clone(),
                    source: rule_id.to_string(),
                    source_kind: CandidateKind::HardVeto,
                });
            }
            if let Some(action) = &rule.minimum_action {
                floors.push(rule_id.to_string());
                candidates.push(ResolvedCandidate {
                    action: parse_action(action)?,
                    reason: reason.clone(),
                    source: rule_id.to_string(),
                    source_kind: CandidateKind::MinimumFloor,
                });
            }
        }
        rule_results.push(HardRuleResultV1 {
            rule_id: rule_id.to_string(),
            checked: true,
            triggered: is_triggered,
            reason_codes: if is_triggered {
                vec![ReasonCode::new(rule.reason.clone())?]
            } else {
                Vec::new()
            },
        });
    }

    for (signal, action) in &policy.model.minimum_actions {
        if signals.contains(signal) {
            floors.push(signal.clone());
            candidates.push(ResolvedCandidate {
                action: parse_action(action)?,
                reason: ReasonCode::new(signal.clone())?,
                source: signal.clone(),
                source_kind: CandidateKind::MinimumFloor,
            });
        }
    }

    candidates.push(ResolvedCandidate {
        action: score_action,
        reason: ReasonCode::new("score_derived_action")?,
        source: "score".to_string(),
        source_kind: CandidateKind::Score,
    });

    reason_codes.extend(signal_reason_codes(&signals)?);
    if reason_codes.is_empty() {
        reason_codes.push(ReasonCode::new("UNCLASSIFIED_REFERENCE_CASE")?);
    }

    let chosen = resolve_candidates(&policy.model, &candidates)?;
    let rejected_actions = rejected_actions(&candidates, &chosen)?;

    let receipt = ControlDecisionReceiptV1 {
        schema_version: ControlDecisionReceiptV1::SCHEMA_VERSION.to_string(),
        input_hash,
        canonical_policy_hash: policy.canonical_hash.clone(),
        evaluator_algorithm_id: EVALUATOR_ALGORITHM_ID.to_string(),
        evaluator_algorithm_hash: Some(hash_bytes(EVALUATOR_SOURCE_BYTES)),
        hard_rules_checked: checked,
        hard_rules_triggered: triggered,
        minimum_action_floors_applied: dedupe(floors),
        hard_rule_results: rule_results,
        axes,
        derived_pressures,
        chosen_action: chosen.action,
        rejected_actions,
        reason_codes: dedupe_reason_codes(reason_codes),
        authority_basis,
        evidence_basis,
        valid_time_basis: if input.valid_time_basis.trim().is_empty() {
            "invalid_input_time_basis".to_string()
        } else {
            input.valid_time_basis
        },
        recorded_time: if input.recorded_time.trim().is_empty() {
            "invalid_input_recorded_time".to_string()
        } else {
            input.recorded_time
        },
        supersession_ref: None,
    };
    receipt.validate()?;
    Ok(receipt)
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SignalSet {
    values: BTreeSet<String>,
}

impl SignalSet {
    fn from_input(input: &ControlEvaluationInputV1) -> Self {
        let mut values = BTreeSet::new();
        for evidence_ref in &input.evidence_refs {
            if evidence_ref.ref_kind == "signal" {
                if let Some(signal) = normalize_signal(&evidence_ref.ref_value) {
                    values.insert(signal);
                }
            }
        }
        Self { values }
    }

    fn contains(&self, value: &str) -> bool {
        self.values.contains(&value.trim().to_ascii_lowercase())
    }
}

fn normalize_signal(value: &str) -> Option<String> {
    let normalized = value.trim().to_ascii_lowercase();
    if normalized.is_empty() {
        None
    } else {
        Some(normalized)
    }
}

fn derive_axes(signals: &SignalSet) -> Result<ScoreAxesV1, ScrError> {
    let high_hazard = signals.contains("high_hazard")
        || signals.contains("source_truth_drift")
        || signals.contains("false_completion_missing_tests")
        || signals.contains("destructive_missing_rollback")
        || signals.contains("forbidden_production_term");
    let low_hazard = signals.contains("low_hazard");
    let uncertain = signals.contains("uncertain") || signals.contains("source_truth_drift");
    let confirmed = signals.contains("confirmed");
    let authority_missing = signals.contains("authority_missing");
    let unknown_owner = signals.contains("unknown_owner_mutation");
    let destructive_missing_rollback = signals.contains("destructive_missing_rollback");
    let forbidden_term = signals.contains("forbidden_production_term");

    Ok(ScoreAxesV1 {
        hazard: ScoreBps::new(if high_hazard {
            9000
        } else if low_hazard {
            1000
        } else {
            4000
        })?,
        evidence_confidence: ScoreBps::new(if confirmed { 9000 } else { 3500 })?,
        uncertainty: ScoreBps::new(if uncertain { 8500 } else { 1500 })?,
        authority: ScoreBps::new(if authority_missing || unknown_owner {
            1000
        } else {
            8500
        })?,
        containment: ScoreBps::new(if destructive_missing_rollback {
            1000
        } else {
            7000
        })?,
        integrity_risk: ScoreBps::new(if forbidden_term {
            9000
        } else if signals.contains("source_truth_drift") {
            6000
        } else {
            2000
        })?,
    })
}

pub fn derive_pressures(axes: &ScoreAxesV1) -> Result<DerivedPressuresV1, ScrError> {
    let authority_gap = 10_000u16.saturating_sub(axes.authority.value());
    let containment_gap = 10_000u16.saturating_sub(axes.containment.value());
    let hazard_autonomy_component = if axes.hazard.value() >= 8500 && containment_gap >= 7000 {
        9000
    } else {
        axes.hazard.value() / 2
    };
    let autonomy_pressure = hazard_autonomy_component
        .max(authority_gap)
        .max(containment_gap);
    let verification_pressure = axes
        .hazard
        .value()
        .max(axes.uncertainty.value())
        .max(10_000u16.saturating_sub(axes.evidence_confidence.value()));
    let repair_priority = if axes.hazard.value() >= 7500 && axes.evidence_confidence.value() >= 7500
    {
        8500
    } else {
        axes.hazard
            .value()
            .saturating_sub(axes.uncertainty.value() / 2)
    };
    let quarantine_pressure = axes
        .integrity_risk
        .value()
        .max(axes.uncertainty.value() / 2);

    Ok(DerivedPressuresV1 {
        autonomy_pressure: ScoreBps::new(autonomy_pressure)?,
        verification_pressure: ScoreBps::new(verification_pressure)?,
        repair_priority: ScoreBps::new(repair_priority)?,
        quarantine_pressure: ScoreBps::new(quarantine_pressure)?,
    })
}

fn score_derived_action(
    policy: &PolicyModelV1,
    pressures: &DerivedPressuresV1,
) -> Result<ControlAction, ScrError> {
    let mut candidates = vec![ControlAction::Backlog];
    threshold_candidates(
        &mut candidates,
        &policy.thresholds.autonomy_pressure,
        pressures.autonomy_pressure.value(),
    )?;
    threshold_candidates(
        &mut candidates,
        &policy.thresholds.verification_pressure,
        pressures.verification_pressure.value(),
    )?;
    threshold_candidates(
        &mut candidates,
        &policy.thresholds.repair_priority,
        pressures.repair_priority.value(),
    )?;
    threshold_candidates(
        &mut candidates,
        &policy.thresholds.quarantine_pressure,
        pressures.quarantine_pressure.value(),
    )?;
    let resolved = candidates
        .into_iter()
        .max_by_key(|action| policy.action_precedence(action).unwrap_or_default())
        .ok_or_else(|| ScrError::PolicyValidationFailed("no score action".to_string()))?;
    Ok(resolved)
}

fn threshold_candidates(
    candidates: &mut Vec<ControlAction>,
    thresholds: &std::collections::BTreeMap<String, u16>,
    pressure: u16,
) -> Result<(), ScrError> {
    for (action, threshold) in thresholds {
        if pressure >= *threshold {
            candidates.push(parse_action(action)?);
        }
    }
    Ok(())
}

fn hard_rule_triggered(rule_id: &str, signals: &SignalSet) -> bool {
    match rule_id {
        "HR_SCHEMA_INVALID" => false,
        "HR_AUTHORITY_MISSING" => signals.contains("authority_missing"),
        "HR_FORBIDDEN_PRODUCTION_TERM" => signals.contains("forbidden_production_term"),
        "HR_UNKNOWN_OWNER_MUTATION" => signals.contains("unknown_owner_mutation"),
        "HR_SOURCE_TRUTH_DRIFT" => signals.contains("source_truth_drift"),
        "HR_FALSE_COMPLETION_MISSING_TESTS" => {
            signals.contains("false_completion_missing_tests")
                || (signals.contains("false_completion") && signals.contains("missing_tests"))
        }
        "HR_DESTRUCTIVE_MISSING_ROLLBACK" => signals.contains("destructive_missing_rollback"),
        _ => false,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ResolvedCandidate {
    action: ControlAction,
    reason: ReasonCode,
    source: String,
    source_kind: CandidateKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum CandidateKind {
    Score,
    MinimumFloor,
    HardVeto,
}

fn resolve_candidates(
    policy: &PolicyModelV1,
    candidates: &[ResolvedCandidate],
) -> Result<ResolvedCandidate, ScrError> {
    candidates
        .iter()
        .cloned()
        .max_by_key(|candidate| {
            (
                candidate.source_kind,
                policy
                    .action_precedence(&candidate.action)
                    .unwrap_or_default(),
                candidate.source.clone(),
            )
        })
        .ok_or_else(|| ScrError::PolicyValidationFailed("no action candidates".to_string()))
}

fn rejected_actions(
    candidates: &[ResolvedCandidate],
    chosen: &ResolvedCandidate,
) -> Result<Vec<RejectedActionV1>, ScrError> {
    let mut rejected = Vec::new();
    let mut seen = BTreeSet::new();
    for candidate in candidates {
        if candidate.action == chosen.action {
            continue;
        }
        let reason = format!(
            "{}_rejected_by_{}",
            candidate.reason.as_str(),
            chosen.reason.as_str()
        );
        let key = (
            candidate.action.name().to_string(),
            candidate.source_kind,
            candidate.source.clone(),
        );
        if seen.insert(key) {
            rejected.push(RejectedActionV1::new(
                candidate.action.clone(),
                vec![ReasonCode::new(reason)?],
            )?);
        }
    }
    Ok(rejected)
}

fn authority_basis(
    input: &ControlEvaluationInputV1,
    signals: &SignalSet,
) -> Result<AuthorityBasisV1, ScrError> {
    let result = if signals.contains("authority_missing") {
        "authority_missing"
    } else {
        "authority_basis_recorded"
    };
    Ok(AuthorityBasisV1 {
        actor_ref: input.actor_ref.clone(),
        permit_ref: input.permit_ref.clone(),
        authority_result: result.to_string(),
        reason_codes: vec![ReasonCode::new(result)?],
    })
}

fn evidence_basis(
    input: &ControlEvaluationInputV1,
    signals: &SignalSet,
) -> Result<EvidenceBasisV1, ScrError> {
    let result = if signals.contains("uncertain") {
        "evidence_uncertain"
    } else {
        "evidence_basis_recorded"
    };
    let refs = input.evidence_refs.clone();
    Ok(EvidenceBasisV1 {
        evidence_refs: refs,
        evidence_result: result.to_string(),
        reason_codes: vec![ReasonCode::new(result)?],
    })
}

fn signal_reason_codes(signals: &SignalSet) -> Result<Vec<ReasonCode>, ScrError> {
    let mut reasons = Vec::new();
    if signals.contains("low_hazard") {
        reasons.push(ReasonCode::new("LOW_HAZARD")?);
    }
    if signals.contains("high_hazard") {
        reasons.push(ReasonCode::new("HIGH_HAZARD")?);
    }
    if signals.contains("confirmed") {
        reasons.push(ReasonCode::new("HIGH_CONFIDENCE")?);
        reasons.push(ReasonCode::new("SUFFICIENT_EVIDENCE")?);
    }
    if signals.contains("uncertain") {
        reasons.push(ReasonCode::new("LOW_CONFIDENCE_OR_HIGH_UNCERTAINTY")?);
    }
    if signals.contains("fixable") {
        reasons.push(ReasonCode::new("FIXABLE")?);
    }
    if signals.contains("false_completion_missing_tests") {
        reasons.push(ReasonCode::new("FALSE_COMPLETION")?);
        reasons.push(ReasonCode::new("MISSING_TESTS")?);
    }
    if signals.contains("unknown_owner_mutation") {
        reasons.push(ReasonCode::new("UNKNOWN_OWNER")?);
        reasons.push(ReasonCode::new("MUTATION_REQUESTED")?);
    }
    if signals.contains("destructive_missing_rollback") {
        reasons.push(ReasonCode::new("DESTRUCTIVE_CHANGE")?);
        reasons.push(ReasonCode::new("MISSING_ROLLBACK")?);
    }
    Ok(reasons)
}

fn hash_json<T: serde::Serialize>(value: &T) -> Result<String, ScrError> {
    let value = serde_json::to_value(value)
        .map_err(|err| ScrError::SerializationFailed(err.to_string()))?;
    let canonical = policy::canonicalize_json_value(value);
    let encoded = serde_json::to_string(&canonical)
        .map_err(|err| ScrError::SerializationFailed(err.to_string()))?;
    Ok(hash_bytes(encoded.as_bytes()))
}

fn hash_bytes(bytes: &[u8]) -> String {
    blake3::hash(bytes).to_hex().to_string()
}

fn dedupe(values: Vec<String>) -> Vec<String> {
    values
        .into_iter()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn dedupe_reason_codes(values: Vec<ReasonCode>) -> Vec<ReasonCode> {
    let mut seen = BTreeSet::new();
    let mut out = Vec::new();
    for value in values {
        if seen.insert(value.as_str().to_string()) {
            out.push(value);
        }
    }
    out
}

fn validate_hard_rule_registry(
    hard_rules: &BTreeMap<String, policy::HardRulePolicyV1>,
) -> Result<(), ScrError> {
    let supported = SUPPORTED_HARD_RULE_IDS
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    for rule_id in hard_rules.keys() {
        if !supported.contains(rule_id.as_str()) {
            return Err(ScrError::PolicyValidationFailed(format!(
                "unknown hard rule: {rule_id}"
            )));
        }
    }
    for rule_id in SUPPORTED_HARD_RULE_IDS {
        if !hard_rules.contains_key(*rule_id) {
            return Err(ScrError::PolicyValidationFailed(format!(
                "missing hard rule: {rule_id}"
            )));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_input() -> ControlEvaluationInputV1 {
        let opaque_ref = ExternalArtifactRef::new("opaque_ref", "value").unwrap();
        ControlEvaluationInputV1 {
            schema_version: ControlEvaluationInputV1::SCHEMA_VERSION.to_string(),
            input_id: "input_001".to_string(),
            actor_ref: opaque_ref.clone(),
            permit_ref: opaque_ref.clone(),
            subject_ref: opaque_ref.clone(),
            domain: Domain::Audit,
            proposed_action: ProposedAction::Analyze,
            requested_effect: RequestedEffect::AdvisoryOnly,
            evidence_refs: Vec::new(),
            environment_ref: opaque_ref,
            valid_time_basis: "2026-05-13T00:00:00Z".to_string(),
            recorded_time: "2026-05-13T00:00:00Z".to_string(),
        }
    }

    fn policy() -> CanonicalPolicyV1 {
        load_policy_from_toml(include_str!("../../../policies/audit_policy_v1.toml")).unwrap()
    }

    #[test]
    fn evaluate_with_policy_requires_valid_input() {
        let mut input = valid_input();
        input.input_id = "".to_string();
        assert_eq!(
            evaluate_with_policy(input, &policy()).unwrap_err().kind(),
            "missing_field"
        );
    }

    #[test]
    fn hard_veto_precedes_score_action() {
        let mut input = valid_input();
        input.evidence_refs = vec![
            ExternalArtifactRef::new("signal", "low_hazard").unwrap(),
            ExternalArtifactRef::new("signal", "confirmed").unwrap(),
            ExternalArtifactRef::new("signal", "forbidden_production_term").unwrap(),
        ];

        let receipt = evaluate_with_policy(input, &policy()).unwrap();

        assert_eq!(receipt.chosen_action, ControlAction::QuarantineArtifact);
        assert!(receipt
            .hard_rules_triggered
            .contains(&"HR_FORBIDDEN_PRODUCTION_TERM".to_string()));
    }

    #[test]
    fn minimum_floor_cannot_be_downgraded() {
        let mut input = valid_input();
        input.evidence_refs = vec![
            ExternalArtifactRef::new("signal", "low_hazard").unwrap(),
            ExternalArtifactRef::new("signal", "confirmed").unwrap(),
            ExternalArtifactRef::new("signal", "source_truth_drift").unwrap(),
        ];

        let receipt = evaluate_with_policy(input, &policy()).unwrap();

        assert_eq!(receipt.chosen_action, ControlAction::RequireVerification);
        assert!(receipt
            .minimum_action_floors_applied
            .contains(&"HR_SOURCE_TRUTH_DRIFT".to_string()));
    }

    #[test]
    fn policy_canonicalizes_deterministically() {
        let source = include_str!("../../../policies/audit_policy_v1.toml");
        let first = load_policy_from_toml(source).unwrap();
        let second = load_policy_from_toml(source).unwrap();

        assert_eq!(first.canonical_json, second.canonical_json);
        assert_eq!(first.canonical_hash, second.canonical_hash);
    }

    #[test]
    fn policy_hash_changes_when_policy_changes() {
        let source = include_str!("../../../policies/audit_policy_v1.toml");
        let first = load_policy_from_toml(source).unwrap();
        let changed = source.replace("version = \"1.0.0\"", "version = \"1.0.1\"");
        let second = load_policy_from_toml(&changed).unwrap();

        assert_ne!(first.canonical_hash, second.canonical_hash);
    }

    #[test]
    fn signals_only_read_from_explicit_signal_refs() {
        let mut input = valid_input();
        input.input_id = "source_truth_drift".to_string();
        input.actor_ref =
            ExternalArtifactRef::new("input_id_value", "unknown_owner_mutation").unwrap();
        input.permit_ref =
            ExternalArtifactRef::new("permit_id", "forbidden_production_term").unwrap();
        input.subject_ref = ExternalArtifactRef::new("subject_id", "high_hazard").unwrap();
        input.environment_ref =
            ExternalArtifactRef::new("environment_id", "destructive_missing_rollback").unwrap();
        input.evidence_refs.clear();

        let receipt = evaluate_with_policy(input, &policy()).unwrap();

        assert!(receipt.minimum_action_floors_applied.is_empty());
        assert!(!receipt
            .reason_codes
            .iter()
            .any(|code| code.as_str() == "SOURCE_TRUTH_DRIFT"));
    }

    #[test]
    fn malformed_input_refs_fail_before_scoring() {
        let mut input = valid_input();
        input.actor_ref = ExternalArtifactRef {
            ref_kind: "actor_ref".to_string(),
            ref_value: "".to_string(),
            owner_hint: None,
        };

        let err = evaluate_with_policy(input, &policy()).unwrap_err();
        assert_eq!(err.kind(), "missing_field");
        assert!(err.to_string().contains("ref_value"));
    }
}
