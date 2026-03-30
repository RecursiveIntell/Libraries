use crate::config::LoopConfig;
use crate::history::PilotHistory;
use crate::orient::TargetCandidate;
use crate::types::{BlockedStepRecord, DecisionAudit, LawfulStepKind, PlannedStep};
use semantic_memory_forge::ExactnessLevelV1;
use verification_control::{CheapCheckKindV1, CheapCheckStatusV1};

#[derive(Debug, Clone)]
pub struct Decision {
    pub selected: Option<TargetCandidate>,
    pub exhausted_candidate_count: usize,
    pub audit: Option<DecisionAudit>,
}

/// Selects the next executable candidate from the scored target set.
pub fn select_candidate(
    mut candidates: Vec<TargetCandidate>,
    history: &PilotHistory,
    config: &LoopConfig,
) -> Decision {
    let exhausted_candidate_count = candidates
        .iter()
        .filter(|candidate| history.is_exhausted(&candidate.stable_key))
        .count();
    candidates.retain(|candidate| !history.is_exhausted(&candidate.stable_key));
    let selected = candidates
        .into_iter()
        .find(|candidate| candidate.urgency >= config.halt_urgency_threshold);
    let audit = selected.as_ref().map(build_decision_audit);

    Decision {
        selected,
        exhausted_candidate_count,
        audit,
    }
}

/// Builds the public decision-audit artifact for a selected candidate.
pub fn build_decision_audit(candidate: &TargetCandidate) -> DecisionAudit {
    let mut fallback_steps = Vec::new();
    let mut blocked_steps = Vec::new();
    let mut cheapest_admissible = None;

    for (index, step_kind) in candidate
        .target
        .lawful_step_ladder()
        .into_iter()
        .enumerate()
    {
        if let Some(reason) = block_reason(candidate, step_kind) {
            blocked_steps.push(BlockedStepRecord { step_kind, reason });
            continue;
        }

        let planned = PlannedStep {
            step_kind,
            cost_rank: index as u8,
            required_artifact_families: required_artifacts(step_kind),
        };
        if cheapest_admissible.is_none() {
            cheapest_admissible = Some(step_kind);
        } else {
            fallback_steps.push(planned);
        }
    }

    let advisory_only = cheapest_admissible.is_none()
        || matches!(
            candidate.plan,
            crate::act::PlanKind::AdvisoryOnlyVerificationPlan(_)
        );

    DecisionAudit {
        stable_target_key: candidate.stable_key.clone(),
        canonical_case_class: candidate.normalization.canonical_case_class,
        budget_class: candidate.normalization.budget_class,
        target_exactness: if candidate.rationale.contains("degradation active") {
            ExactnessLevelV1::Conservative
        } else {
            ExactnessLevelV1::Exact
        },
        cheapest_admissible,
        fallback_steps,
        blocked_steps,
        cheap_check_ladder: cheap_check_ladder(candidate),
        advisory_only,
    }
}

fn cheap_check_ladder(candidate: &TargetCandidate) -> Vec<CheapCheckStatusV1> {
    vec![
        CheapCheckStatusV1 {
            check: CheapCheckKindV1::SchemaValidation,
            satisfied: true,
            detail: Some("target normalized into canonical verification case".into()),
        },
        CheapCheckStatusV1 {
            check: CheapCheckKindV1::ProvenanceAudit,
            satisfied: !candidate.rationale.contains("missing provenance"),
            detail: Some(candidate.rationale.clone()),
        },
        CheapCheckStatusV1 {
            check: CheapCheckKindV1::ReplayPreflight,
            satisfied: !candidate.rationale.contains("degradation active"),
            detail: Some(if candidate.rationale.contains("degradation active") {
                "degraded observation blocks exact replay path".into()
            } else {
                "exact replay path remains admissible".into()
            }),
        },
    ]
}

fn block_reason(candidate: &TargetCandidate, step_kind: LawfulStepKind) -> Option<String> {
    if candidate.normalization.missing_falsifier
        && matches!(step_kind, LawfulStepKind::MinimalPerturbationRefuter)
    {
        return Some("missing falsifier artifact".into());
    }

    if candidate.normalization.comparability_required
        && matches!(
            step_kind,
            LawfulStepKind::PairedComparativeCheck | LawfulStepKind::NuisanceComparabilityAudit
        )
        && !candidate
            .check_hints
            .iter()
            .any(|hint| hint.contains("comparability"))
    {
        return Some("comparability witness set unavailable".into());
    }

    if candidate.rationale.contains("degradation active")
        && matches!(
            step_kind,
            LawfulStepKind::ExactOracleSlice
                | LawfulStepKind::ExactReplay
                | LawfulStepKind::CanonicalExportRequest
                | LawfulStepKind::CanonicalImportRequest
        )
    {
        return Some("degraded observation blocks exact path".into());
    }

    None
}

fn required_artifacts(step_kind: LawfulStepKind) -> Vec<String> {
    match step_kind {
        LawfulStepKind::ContractSchemaCheck => vec!["VerificationPlanV1".into()],
        LawfulStepKind::ProvenanceReceiptAudit => {
            vec!["RuntimeQueryProvenanceV1".into(), "ControlReceipt".into()]
        }
        LawfulStepKind::TemporalConsistencyCheck => vec!["ExecutionContextV1".into()],
        LawfulStepKind::ExactReplay => vec!["ReplayArtifact".into(), "ExecutionContextV1".into()],
        LawfulStepKind::PairedComparativeCheck | LawfulStepKind::NuisanceComparabilityAudit => {
            vec!["ComparableWitnessSet".into()]
        }
        LawfulStepKind::ExactOracleSlice | LawfulStepKind::ConservativeOracleSlice => {
            vec!["RuntimeQueryProvenanceV1".into()]
        }
        LawfulStepKind::MinimalPerturbationRefuter => vec!["FalsifierWitness".into()],
        LawfulStepKind::HumanReviewRequest => Vec::new(),
        LawfulStepKind::CanonicalExportRequest | LawfulStepKind::CanonicalImportRequest => {
            vec!["EpisodeBundleV1".into()]
        }
    }
}
