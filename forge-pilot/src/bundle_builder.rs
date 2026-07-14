use crate::act::{OracleExecution, PlanKind};
use forge_engine::lab::evidence::{
    BaselineOrPatch, PairComparability, RefutationArtifact, RefutationArtifactOutcome,
    RefutationArtifactType, VerificationTrial,
};
use forge_engine::{
    local_hypothesis_support_confidence, AblationClassification, AblationReceipt, BundleScope,
    CausalExperimentResult, CausalHypothesis, ClaimStrength, ExperimentEvidenceBundle,
    HypothesisStatus, PredictionDisposition, ReceiptKind, ReceiptRef, ReceiptStorage, ScoreVector,
    Treatment, TypedLocatedEffect,
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
    pub causal_result: &'a CausalExperimentResult,
    pub ablation_receipts: &'a [AblationReceipt],
    pub known_threats: Vec<String>,
}

/// Builds a Forge experiment-evidence bundle from an oracle execution result.
pub fn build_bundle_from_oracle(input: OracleBundleInput<'_>) -> ExperimentEvidenceBundle {
    let bundle_id = Uuid::new_v4().to_string();
    let eval_id = format!("oracle-eval:{}", Uuid::new_v4());
    let outcome = input
        .oracle_execution
        .outcome_summary()
        .unwrap_or_else(|| "oracle evaluation completed".into());
    let receipt_payload = serde_json::to_string(&input.oracle_execution.summary_json())
        .unwrap_or_else(|_| "{}".into());
    let (support, contradictions, mut warnings) = oracle_evidence_counts(input.oracle_execution);
    warnings.extend(input.known_threats.clone());
    warnings.push(
        "oracle novelty and stability were not measured; compatibility zeros are excluded from the weighted score"
            .into(),
    );
    warnings.sort();
    warnings.dedup();
    let correctness = if support > 0 && contradictions == 0 {
        1.0
    } else {
        0.0
    };

    ExperimentEvidenceBundle {
        bundle_id: bundle_id.clone(),
        candidate_id: input.target_key.to_string(),
        eval_id,
        version_id: "forge-pilot.v1".into(),
        supersedes_claim_version_id: input.plan.supersedes_claim_version_id(),
        relation_lineage_hints: Default::default(),
        scores: ScoreVector {
            correctness,
            novelty: 0.0,
            stability: 0.0,
            weighted_total: correctness,
            cea_confidence: None,
            cea_predicted_correctness: None,
        },
        hypotheses: vec![CausalHypothesis {
            hypothesis_id: format!("hypothesis:{bundle_id}"),
            cause_signature: input.target_key.to_string(),
            effect_signature: outcome.clone(),
            confidence: local_hypothesis_support_confidence(support, contradictions),
            status: hypothesis_status(support, contradictions),
            support_count: support,
            contradiction_count: contradictions,
        }],
        verification: None,
        trace_id: input.trace_id,
        experiment_diff: None,
        attribution_json: Some(receipt_payload.clone()),
        assessment: None,
        warnings: warnings.clone(),
        created_at: chrono::Utc::now().to_rfc3339(),
        run_id: None,
        attempt_id: Some(AttemptId::generate().to_string()),
        causal_question: Some(format!("Should {} remain supported?", input.target_key)),
        unit_definition: Some("kernel oracle evaluation".into()),
        bundle_scope: Some(BundleScope {
            workload_id: input.target_key.to_string(),
            backend_family: "kernel_oracle".into(),
            selected_checks: input.plan.check_names(),
            timeout_class: "bounded".into(),
            config_flags: vec!["canonical_v3".into()],
        }),
        pair_comparability: None,
        claim_strength: ClaimStrength::ProvisionalSinglePair,
        identification_rationale: Some(
            "single bounded kernel-oracle execution; the receipt proves what ran, while support remains local and provisional".into(),
        ),
        known_threats: warnings,
        patch_hash: None,
        treatment: Some(Treatment {
            kind: "oracle_check".into(),
            patch_hash: ContentDigest::compute_str(input.target_key)
                .hex()
                .to_string(),
            patch_summary: format!("{:?}", input.plan),
        }),
        outcome: Some(outcome),
        covariates: None,
        promotion_state: None,
        primary_effect: None,
        all_effects: vec![],
        hypothesis_edges: vec![],
        receipts: vec![inline_receipt(
            "oracle-summary",
            ReceiptKind::CheckResult,
            &receipt_payload,
        )],
        verification_trials: vec![VerificationTrial {
            trial_id: TrialId::generate(),
            attempt_id: AttemptId::generate(),
            baseline_or_patch: BaselineOrPatch::Baseline,
            completed: true,
            receipts: vec!["oracle-summary".into()],
        }],
        refutation_artifacts: input.oracle_execution.refutation_artifacts(),
        sealed: false,
    }
}

/// Builds a Forge experiment-evidence bundle from a patch execution result.
pub fn build_bundle_from_patch(
    input: PatchBundleInput<'_>,
) -> Result<ExperimentEvidenceBundle, serde_json::Error> {
    let bundle_id = Uuid::new_v4().to_string();
    let eval_id = format!("patch-eval:{}", Uuid::new_v4());
    let attempt_id = AttemptId::generate();
    let experiment = &input.causal_result.experiment;
    let attribution_payload = serde_json::to_string(&serde_json::json!({
        "causal_update_receipts": input.causal_result.receipts,
        "ablation_receipts": input.ablation_receipts,
    }))?;
    let patch_hash = input
        .causal_result
        .receipts
        .first()
        .map(|receipt| receipt.patch_digest.clone());
    let correctness = check_correctness(input.causal_result);
    let stability = repeated_pair_agreement(input.causal_result);
    let weighted_total = available_patch_weighted_total(
        correctness,
        (experiment.pairs.len() > 1).then_some(stability),
    );
    let mut support = 0_u64;
    let mut contradictions = 0_u64;
    for receipt in input.ablation_receipts {
        if !receipt.comparable {
            continue;
        }
        match receipt.classification {
            AblationClassification::Supported => support += 1,
            AblationClassification::Contradicted => contradictions += 1,
            AblationClassification::Inconclusive => {}
        }
    }
    let status = hypothesis_status(support, contradictions);
    let cea_confidence = mean_prediction(input.causal_result, |receipt| {
        receipt.post_prediction.confidence
    });
    let cea_predicted_correctness = mean_prediction(input.causal_result, |receipt| {
        receipt.post_prediction.predicted_correctness
    });
    let mut warnings = input.known_threats.clone();
    warnings.push(
        "patch novelty was not measured; compatibility zero is excluded from the weighted score"
            .into(),
    );
    if experiment.pairs.len() <= 1 {
        warnings.push(
            "single-pair stability was not measured; compatibility zero is excluded from the weighted score"
                .into(),
        );
    }
    if input.causal_result.receipts.is_empty() {
        warnings
            .push("CEA produced no update receipt; CEA prediction metrics are unavailable".into());
    }
    for receipt in &input.causal_result.receipts {
        warnings.extend(receipt.degradation_reasons.clone());
        if receipt.prediction_disposition == PredictionDisposition::RunChecks {
            warnings.push(format!(
                "CEA prediction {} is advisory; verification gate requires checks",
                receipt.post_prediction.digest
            ));
        }
    }
    for receipt in input.ablation_receipts {
        warnings.extend(receipt.degradation_reasons.clone());
    }
    let pair_comparability = pair_comparability(input.causal_result);
    if !pair_comparability.valid {
        warnings.extend(pair_comparability.violations.clone());
    }
    let mut receipts = Vec::new();
    for receipt in &input.causal_result.receipts {
        let payload = serde_json::to_string(receipt)?;
        receipts.push(inline_receipt(
            &format!("cea-update:{}", receipt.receipt_digest),
            ReceiptKind::CausalUpdate,
            &payload,
        ));
    }
    for receipt in input.ablation_receipts {
        let payload = serde_json::to_string(receipt)?;
        receipts.push(inline_receipt(
            &format!("cea-ablation:{}", receipt.receipt_digest),
            ReceiptKind::Ablation,
            &payload,
        ));
    }
    let mut verification_trials = Vec::new();
    for pair in &experiment.pairs {
        let matching_receipts = input
            .causal_result
            .receipts
            .iter()
            .filter(|receipt| receipt.observation_identity.trial_id == pair.pair_index.to_string())
            .collect::<Vec<_>>();
        let completed = matching_receipts.len() == 1;
        if !completed {
            warnings.push(format!(
                "pair {} has no matching CEA update receipt for completed verification export (found {})",
                pair.pair_index,
                matching_receipts.len()
            ));
        }
        let receipt_id = matching_receipts
            .first()
            .map(|receipt| vec![format!("cea-update:{}", receipt.receipt_digest)])
            .unwrap_or_default();
        verification_trials.push(VerificationTrial {
            trial_id: TrialId::generate(),
            attempt_id: attempt_id.clone(),
            baseline_or_patch: BaselineOrPatch::Baseline,
            completed,
            receipts: receipt_id.clone(),
        });
        verification_trials.push(VerificationTrial {
            trial_id: TrialId::generate(),
            attempt_id: attempt_id.clone(),
            baseline_or_patch: BaselineOrPatch::Patched,
            completed,
            receipts: receipt_id,
        });
    }
    let mut refutation_artifacts = Vec::new();
    for receipt in input.ablation_receipts {
        let trial_id = TrialId::generate();
        let receipt_id = format!("cea-ablation:{}", receipt.receipt_digest);
        verification_trials.push(VerificationTrial {
            trial_id: trial_id.clone(),
            attempt_id: attempt_id.clone(),
            baseline_or_patch: BaselineOrPatch::Patched,
            completed: receipt.comparable,
            receipts: vec![receipt_id],
        });
        refutation_artifacts.push(RefutationArtifact {
            artifact_id: format!("singleton-ablation:{}", receipt.operation_index),
            artifact_type: RefutationArtifactType::SingletonAblation,
            trial_id: Some(trial_id),
            attempt_id: Some(attempt_id.clone()),
            outcome: ablation_refutation_outcome(receipt),
            estimate_delta: None,
            details: Some(serde_json::to_string(receipt)?),
        });
    }
    warnings.sort();
    warnings.dedup();

    Ok(ExperimentEvidenceBundle {
        bundle_id: bundle_id.clone(),
        candidate_id: input.target_key.to_string(),
        eval_id,
        version_id: "forge-pilot.v1".into(),
        supersedes_claim_version_id: input.plan.supersedes_claim_version_id(),
        relation_lineage_hints: Default::default(),
        scores: ScoreVector {
            correctness,
            novelty: 0.0,
            stability,
            weighted_total,
            cea_confidence,
            cea_predicted_correctness,
        },
        hypotheses: vec![CausalHypothesis {
            hypothesis_id: format!("hypothesis:{bundle_id}"),
            cause_signature: input.target_key.to_string(),
            effect_signature: format!(
                "improvements={} regressions={}",
                experiment.diff.improvements, experiment.diff.regressions
            ),
            confidence: local_hypothesis_support_confidence(support, contradictions),
            status,
            support_count: support,
            contradiction_count: contradictions,
        }],
        verification: None,
        trace_id: input.trace_id,
        experiment_diff: Some(experiment.diff.clone()),
        attribution_json: Some(attribution_payload),
        assessment: None,
        warnings: warnings.clone(),
        created_at: chrono::Utc::now().to_rfc3339(),
        run_id: Some(experiment.run_id.clone()),
        attempt_id: Some(attempt_id.to_string()),
        causal_question: Some(format!(
            "Does {} improve the selected fixture?",
            input.target_key
        )),
        unit_definition: Some("paired patch experiment".into()),
        bundle_scope: Some(BundleScope {
            workload_id: input.target_key.to_string(),
            backend_family: format!("{:?}", experiment.mode),
            selected_checks: vec!["fmt".into(), "clippy".into(), "test".into()],
            timeout_class: "forge".into(),
            config_flags: vec!["canonical_v3".into()],
        }),
        pair_comparability: Some(pair_comparability),
        claim_strength: ClaimStrength::ProvisionalSinglePair,
        identification_rationale: Some(
            "fixed-workload local paired intervention; observational proximity attribution is distinct from bounded singleton-ablation evidence".into(),
        ),
        known_threats: warnings,
        patch_hash: patch_hash.clone(),
        treatment: patch_hash.map(|patch_hash| Treatment {
            kind: "patch_applied".into(),
            patch_hash,
            patch_summary: format!("{} pair(s)", experiment.pairs.len()),
        }),
        outcome: Some(format!(
            "paired patch completed: improvements={} regressions={}",
            experiment.diff.improvements, experiment.diff.regressions
        )),
        covariates: None,
        promotion_state: None,
        primary_effect: experiment.diff.effects.first().map(|effect| {
            TypedLocatedEffect {
                kind: effect.kind.clone(),
                file: effect.file.clone(),
                line: effect.line,
                message: effect.message.clone(),
                in_baseline: effect.in_baseline,
                in_patched: effect.in_patched,
            }
        }),
        all_effects: input
            .causal_result
            .experiment
            .diff
            .effects
            .iter()
            .map(|effect| TypedLocatedEffect {
                kind: effect.kind.clone(),
                file: effect.file.clone(),
                line: effect.line,
                message: effect.message.clone(),
                in_baseline: effect.in_baseline,
                in_patched: effect.in_patched,
            })
            .collect(),
        hypothesis_edges: vec![],
        receipts,
        verification_trials,
        refutation_artifacts,
        sealed: false,
    })
}

fn oracle_evidence_counts(execution: &OracleExecution) -> (u64, u64, Vec<String>) {
    if let Some(assessment) = &execution.assessment {
        return binary_evidence_counts(assessment.supported);
    }
    if let Some(assessment) = &execution.delta_parity {
        return binary_evidence_counts(assessment.parity_match);
    }
    if let Some(assessment) = &execution.temporal_replay {
        return binary_evidence_counts(assessment.matched_expected_hash);
    }
    if let Some(refutation) = &execution.refutation {
        return match &refutation.outcome {
            kernel_oracles::OracleRefutationOutcome::FlipWitness { .. } => {
                binary_evidence_counts(true)
            }
            kernel_oracles::OracleRefutationOutcome::NoFlipFound { .. } => {
                binary_evidence_counts(false)
            }
            kernel_oracles::OracleRefutationOutcome::NotApplicable { reason } => (
                0,
                0,
                vec![format!("oracle refutation was not applicable: {reason}")],
            ),
        };
    }
    (
        0,
        0,
        vec!["oracle execution produced no assessable result".into()],
    )
}

fn binary_evidence_counts(supported: bool) -> (u64, u64, Vec<String>) {
    if supported {
        (1, 0, Vec::new())
    } else {
        (0, 1, Vec::new())
    }
}

fn inline_receipt(receipt_id: &str, kind: ReceiptKind, payload: &str) -> ReceiptRef {
    ReceiptRef {
        receipt_id: receipt_id.into(),
        kind,
        storage: ReceiptStorage::Inline(payload.into()),
        content_hash: ContentDigest::compute_str(payload).hex().to_string(),
        trace_id: None,
        replay_handle: None,
    }
}

fn check_correctness(result: &CausalExperimentResult) -> f64 {
    let checks = result.experiment.pairs.len() * 3;
    if checks == 0 {
        return 0.0;
    }
    let passed = result
        .experiment
        .pairs
        .iter()
        .map(|pair| {
            usize::from(pair.patched_result.fmt_pass)
                + usize::from(pair.patched_result.clippy_pass)
                + usize::from(pair.patched_result.test_pass)
        })
        .sum::<usize>();
    passed as f64 / checks as f64
}

fn repeated_pair_agreement(result: &CausalExperimentResult) -> f64 {
    let Some(reference) = result.experiment.pairs.first() else {
        return 0.0;
    };
    if result.experiment.pairs.len() == 1 {
        return 0.0;
    }
    let reference = (
        reference.patched_result.fmt_pass,
        reference.patched_result.clippy_pass,
        reference.patched_result.test_pass,
        reference.diff.improvements,
        reference.diff.regressions,
    );
    let agreeing = result
        .experiment
        .pairs
        .iter()
        .filter(|pair| {
            (
                pair.patched_result.fmt_pass,
                pair.patched_result.clippy_pass,
                pair.patched_result.test_pass,
                pair.diff.improvements,
                pair.diff.regressions,
            ) == reference
        })
        .count();
    agreeing as f64 / result.experiment.pairs.len() as f64
}

fn available_patch_weighted_total(correctness: f64, stability: Option<f64>) -> f64 {
    let policy = forge_engine::ObjectivePolicy::bug_fix();
    let stability_weight = stability.map(|_| policy.stability_weight).unwrap_or(0.0);
    let available_weight = policy.correctness_weight + stability_weight;
    if available_weight <= f64::EPSILON {
        return 0.0;
    }
    let numerator =
        policy.correctness_weight * correctness + stability_weight * stability.unwrap_or_default();
    (numerator / available_weight).clamp(0.0, 1.0)
}

fn pair_comparability(result: &CausalExperimentResult) -> PairComparability {
    let mut violations = result
        .experiment
        .pairs
        .iter()
        .filter(|pair| !pair.comparable)
        .flat_map(|pair| {
            pair.comparability_reasons
                .iter()
                .map(move |reason| format!("pair {}: {reason}", pair.pair_index))
        })
        .collect::<Vec<_>>();
    if result.experiment.pairs.is_empty() {
        violations.push("experiment produced no paired trials".into());
    }
    PairComparability {
        valid: violations.is_empty(),
        violations,
    }
}

fn mean_prediction(
    result: &CausalExperimentResult,
    value: impl Fn(&forge_engine::CausalUpdateReceipt) -> f64,
) -> Option<f64> {
    (!result.receipts.is_empty())
        .then(|| result.receipts.iter().map(value).sum::<f64>() / result.receipts.len() as f64)
}

fn hypothesis_status(support: u64, contradictions: u64) -> HypothesisStatus {
    if support == 0 && contradictions == 0 {
        HypothesisStatus::Proposed
    } else if contradictions > support {
        HypothesisStatus::Contradicted
    } else if support > contradictions {
        HypothesisStatus::Supported
    } else {
        HypothesisStatus::Neutral
    }
}

fn ablation_refutation_outcome(receipt: &AblationReceipt) -> RefutationArtifactOutcome {
    match receipt.classification {
        AblationClassification::Supported => RefutationArtifactOutcome::Passed,
        AblationClassification::Contradicted => RefutationArtifactOutcome::Failed {
            reason: "ablation did not remove the full-patch effect".into(),
        },
        AblationClassification::Inconclusive => RefutationArtifactOutcome::Inconclusive {
            reason: receipt.degradation_reasons.join("; "),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use forge_engine::EvidenceKind;

    fn ablation(classification: AblationClassification) -> AblationReceipt {
        AblationReceipt {
            operation_index: 0,
            classification,
            evidence_kind: EvidenceKind::Ablation,
            verification_method: "test".into(),
            comparable: true,
            full_patch_digest: "full-patch".into(),
            ablated_patch_digest: Some("ablated-patch".into()),
            baseline_digest: "baseline".into(),
            config_digest: "config".into(),
            baseline_effect_digest: "base".into(),
            full_outcome_digest: "full".into(),
            ablated_outcome_digest: "ablated".into(),
            removed_effect_count: 0,
            persisting_effect_count: 0,
            new_effect_count: 0,
            receipt_digest: "receipt".into(),
            degradation_reasons: vec!["bounded".into()],
        }
    }

    #[test]
    fn ablation_classifications_map_to_explicit_refutation_outcomes() {
        assert!(matches!(
            ablation_refutation_outcome(&ablation(AblationClassification::Supported)),
            RefutationArtifactOutcome::Passed
        ));
        assert!(matches!(
            ablation_refutation_outcome(&ablation(AblationClassification::Contradicted)),
            RefutationArtifactOutcome::Failed { .. }
        ));
        assert!(matches!(
            ablation_refutation_outcome(&ablation(AblationClassification::Inconclusive)),
            RefutationArtifactOutcome::Inconclusive { .. }
        ));
    }
}
