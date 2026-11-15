use crate::act::{OracleExecution, PlanKind};
use forge_engine::lab::evidence::{BaselineOrPatch, VerificationTrial};
use forge_engine::{
    BundleScope, CausalHypothesis, ClaimStrength, ExperimentEvidenceBundle, ExperimentResult,
    HypothesisStatus, ReceiptKind, ReceiptRef, ReceiptStorage, ScoreVector, Treatment, TrialSide,
};
use stack_ids::{AttemptId, ContentDigest, TrialId};
use uuid::Uuid;

pub struct OracleBundleInput<'a> {
    pub plan: &'a PlanKind,
    pub target_key: &'a str,
    pub trace_id: Option<String>,
    pub scope_namespace: &'a str,
    pub oracle_execution: &'a OracleExecution,
    pub known_threats: Vec<String>,
}

pub struct PatchBundleInput<'a> {
    pub plan: &'a PlanKind,
    pub target_key: &'a str,
    pub trace_id: Option<String>,
    pub scope_namespace: &'a str,
    pub experiment_result: &'a ExperimentResult,
    pub known_threats: Vec<String>,
}

// The current schema requires numeric scores. Zero denotes unavailable here,
// explicitly qualified in warnings; it is not an estimated probability or utility.
fn unavailable_scores() -> ScoreVector {
    ScoreVector {
        correctness: 0.0,
        novelty: 0.0,
        stability: 0.0,
        weighted_total: 0.0,
        cea_confidence: None,
        cea_predicted_correctness: None,
    }
}

/// Build an unpaired diagnostic observation, not an invented intervention.
pub fn build_bundle_from_oracle(input: OracleBundleInput<'_>) -> ExperimentEvidenceBundle {
    let bundle_id = Uuid::new_v4().to_string();
    let attempt_id = AttemptId::generate();
    let outcome = input.oracle_execution.outcome_summary();
    let receipt_payload = input.oracle_execution.summary_json().to_string();
    let mut warnings = input.known_threats.clone();
    warnings.push("unpaired oracle diagnostic; score fields unavailable (zero), not calibrated confidence or independent acceptance".into());
    ExperimentEvidenceBundle {
        bundle_id: bundle_id.clone(),
        candidate_id: input.target_key.into(),
        eval_id: format!("oracle-eval:{}", Uuid::new_v4()),
        version_id: "forge-pilot.v2".into(),
        supersedes_claim_version_id: input.plan.supersedes_claim_version_id(),
        relation_lineage_hints: Default::default(),
        scores: unavailable_scores(),
        hypotheses: vec![CausalHypothesis {
            hypothesis_id: format!("hypothesis:{bundle_id}"),
            cause_signature: input.target_key.into(),
            effect_signature: outcome
                .clone()
                .unwrap_or_else(|| "no executable oracle outcome".into()),
            confidence: 0.0,
            status: HypothesisStatus::Proposed,
            support_count: 0,
            contradiction_count: 0,
        }],
        verification: None,
        trace_id: input.trace_id,
        experiment_diff: None,
        attribution_json: Some(receipt_payload.clone()),
        assessment: None,
        warnings,
        created_at: chrono::Utc::now().to_rfc3339(),
        run_id: None,
        attempt_id: Some(attempt_id.to_string()),
        causal_question: Some(format!(
            "What did the oracle observe for {}?",
            input.target_key
        )),
        unit_definition: Some("unpaired kernel oracle evaluation".into()),
        bundle_scope: Some(BundleScope {
            workload_id: input.target_key.into(),
            backend_family: "kernel_oracle".into(),
            selected_checks: input.plan.check_names(),
            timeout_class: "bounded".into(),
            config_flags: vec!["canonical_v3".into(), "non_causal_observation".into()],
        }),
        pair_comparability: None,
        claim_strength: ClaimStrength::ObservationOnly,
        identification_rationale: Some(
            "unpaired oracle result; causal pair comparison not applicable".into(),
        ),
        known_threats: input.known_threats,
        patch_hash: None,
        treatment: None,
        outcome,
        covariates: None,
        promotion_state: None,
        primary_effect: None,
        all_effects: vec![],
        hypothesis_edges: vec![],
        receipts: vec![inline_receipt("oracle-summary", &receipt_payload)],
        verification_trials: if input.oracle_execution.outcome_summary().is_some() {
            vec![VerificationTrial {
                trial_id: TrialId::generate(),
                attempt_id,
                baseline_or_patch: BaselineOrPatch::Baseline,
                completed: true,
                receipts: vec!["oracle-summary".into()],
            }]
        } else {
            vec![]
        },
        refutation_artifacts: input.oracle_execution.refutation_artifacts(),
        sealed: false,
    }
}

/// Preserve actual physical trials; diagnostic counters do not grant promotion.
pub fn build_bundle_from_patch(input: PatchBundleInput<'_>) -> ExperimentEvidenceBundle {
    let bundle_id = Uuid::new_v4().to_string();
    let attempt_id = AttemptId::generate();
    let result = input.experiment_result;
    let receipt_payload = serde_json::json!({
        "run_id": result.run_id, "diff": result.diff, "patch_hash": result.patch_hash,
        "baseline": result.baseline_descriptor, "trials": result.trials,
        "pair_comparability": result.pair_comparability,
        "source_hash": result.source_hash, "command_plan": result.command_plan,
    })
    .to_string();
    let mut warnings = input.known_threats.clone();
    warnings.push("raw component-check observations only; score fields unavailable (zero), not calibrated confidence or independent acceptance".into());
    if matches!(input.plan, PlanKind::PairedPatch { description, .. }
        if description.contains("kernel-generated") || description.contains("NonAuthoritativeDerived"))
    {
        warnings.insert(0, "AUTHORITY: NonAuthoritativeDerived — requires human review before authoritative promotion".into());
    }
    ExperimentEvidenceBundle {
        bundle_id: bundle_id.clone(), candidate_id: input.target_key.into(),
        eval_id: format!("patch-eval:{}", Uuid::new_v4()), version_id: "forge-pilot.v2".into(),
        supersedes_claim_version_id: input.plan.supersedes_claim_version_id(),
        relation_lineage_hints: Default::default(), scores: unavailable_scores(),
        hypotheses: vec![CausalHypothesis {
            hypothesis_id: format!("hypothesis:{bundle_id}"), cause_signature: input.target_key.into(),
            effect_signature: format!("improvements={} regressions={}", result.diff.improvements, result.diff.regressions),
            confidence: 0.0, status: HypothesisStatus::Proposed, support_count: 0, contradiction_count: 0,
        }],
        verification: None, trace_id: input.trace_id, experiment_diff: Some(result.diff.clone()),
        attribution_json: None, assessment: None, warnings,
        created_at: chrono::Utc::now().to_rfc3339(), run_id: Some(result.run_id.clone()),
        attempt_id: Some(attempt_id.to_string()),
        causal_question: Some(format!("Does {} improve the selected fixture?", input.target_key)),
        unit_definition: Some("paired component correctness checks".into()),
        bundle_scope: Some(BundleScope {
            workload_id: input.target_key.into(), backend_family: format!("{:?}", result.trials.first().map(|trial| trial.backend_kind)),
            selected_checks: vec!["fmt".into(), "clippy".into(), "test".into()], timeout_class: "forge".into(),
            config_flags: vec!["canonical_v3".into(), "independent_acceptance_not_supplied".into()],
        }),
        pair_comparability: Some(result.pair_comparability.clone()),
        claim_strength: if result.trials.len() > 2 { ClaimStrength::ProvisionalRepeatedChecks } else { ClaimStrength::ProvisionalSinglePair },
        identification_rationale: Some("fresh sides from one content-bound source snapshot; no independent task-level significance".into()),
        known_threats: input.known_threats, patch_hash: Some(result.patch_hash.clone()),
        treatment: Some(Treatment { kind: "patch_applied".into(), patch_hash: result.patch_hash.clone(), patch_summary: format!("{:?}", input.plan) }),
        outcome: Some(format!("component checks completed: improvements={} regressions={}", result.diff.improvements, result.diff.regressions)),
        covariates: None, promotion_state: None, primary_effect: result.diff.effects.first().cloned(),
        all_effects: result.diff.effects.clone(), hypothesis_edges: vec![],
        receipts: vec![inline_receipt("patch-summary", &receipt_payload)],
        verification_trials: result.trials.iter().map(|trial| VerificationTrial {
            trial_id: TrialId::new(trial.trial_id.clone()), attempt_id: attempt_id.clone(),
            baseline_or_patch: match trial.side { TrialSide::Baseline => BaselineOrPatch::Baseline, TrialSide::Patched => BaselineOrPatch::Patched },
            completed: trial.completed, receipts: vec!["patch-summary".into()],
        }).collect(),
        refutation_artifacts: vec![], sealed: false,
    }
}

fn inline_receipt(receipt_id: &str, payload: &str) -> ReceiptRef {
    ReceiptRef {
        receipt_id: receipt_id.into(),
        kind: ReceiptKind::CheckResult,
        storage: ReceiptStorage::Inline(payload.into()),
        content_hash: ContentDigest::compute_str(payload).hex().to_string(),
        trace_id: None,
        replay_handle: None,
    }
}
