use crate::act::{OracleExecution, PlanKind};
use forge_engine::lab::evidence::{BaselineOrPatch, PairComparability, VerificationTrial};
use forge_engine::{
    BundleScope, CausalHypothesis, ClaimStrength, EffectKind, ExperimentEvidenceBundle,
    ExperimentResult, HypothesisStatus, ReceiptKind, ReceiptRef, ReceiptStorage, ScoreVector,
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
    pub experiment_result: &'a ExperimentResult,
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

    ExperimentEvidenceBundle {
        bundle_id: bundle_id.clone(),
        candidate_id: input.target_key.to_string(),
        eval_id,
        version_id: "forge-pilot.v1".into(),
        supersedes_claim_version_id: input.plan.supersedes_claim_version_id(),
        relation_lineage_hints: Default::default(),
        scores: ScoreVector {
            correctness: 0.85,
            novelty: 0.10,
            stability: 0.80,
            weighted_total: 0.75,
            cea_confidence: None,
            cea_predicted_correctness: None,
        },
        hypotheses: vec![CausalHypothesis {
            hypothesis_id: format!("hypothesis:{bundle_id}"),
            cause_signature: input.target_key.to_string(),
            effect_signature: outcome.clone(),
            confidence: 0.60,
            status: HypothesisStatus::Supported,
            support_count: 1,
            contradiction_count: 0,
        }],
        verification: None,
        trace_id: input.trace_id,
        experiment_diff: None,
        attribution_json: Some(receipt_payload.clone()),
        assessment: None,
        warnings: input.known_threats.clone(),
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
        pair_comparability: Some(PairComparability {
            valid: true,
            violations: vec![],
        }),
        claim_strength: ClaimStrength::ProvisionalSinglePair,
        identification_rationale: Some(
            "bounded kernel-oracle evaluation over imported payload".into(),
        ),
        known_threats: input.known_threats,
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
        primary_effect: Some(TypedLocatedEffect {
            kind: EffectKind::TestFailure,
            file: None,
            line: None,
            message: input
                .oracle_execution
                .outcome_summary()
                .unwrap_or_else(|| "oracle evaluation completed".into()),
            in_baseline: true,
            in_patched: false,
        }),
        all_effects: vec![],
        hypothesis_edges: vec![],
        receipts: vec![inline_receipt("oracle-summary", &receipt_payload)],
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
pub fn build_bundle_from_patch(input: PatchBundleInput<'_>) -> ExperimentEvidenceBundle {
    let bundle_id = Uuid::new_v4().to_string();
    let eval_id = format!("patch-eval:{}", Uuid::new_v4());
    let attempt_id = AttemptId::generate();
    let receipt_payload = serde_json::json!({
        "run_id": input.experiment_result.run_id,
        "diff": input.experiment_result.diff,
    })
    .to_string();
    let patch_hash = ContentDigest::compute_str(&format!("{:?}", input.plan))
        .hex()
        .to_string();

    ExperimentEvidenceBundle {
        bundle_id: bundle_id.clone(),
        candidate_id: input.target_key.to_string(),
        eval_id,
        version_id: "forge-pilot.v1".into(),
        supersedes_claim_version_id: input.plan.supersedes_claim_version_id(),
        relation_lineage_hints: Default::default(),
        scores: ScoreVector {
            correctness: if input.experiment_result.diff.improvements
                >= input.experiment_result.diff.regressions
            {
                0.90
            } else {
                0.40
            },
            novelty: 0.25,
            stability: 0.70,
            weighted_total: 0.70,
            cea_confidence: None,
            cea_predicted_correctness: None,
        },
        hypotheses: vec![CausalHypothesis {
            hypothesis_id: format!("hypothesis:{bundle_id}"),
            cause_signature: input.target_key.to_string(),
            effect_signature: format!(
                "improvements={} regressions={}",
                input.experiment_result.diff.improvements, input.experiment_result.diff.regressions
            ),
            confidence: 0.65,
            status: HypothesisStatus::Supported,
            support_count: input.experiment_result.diff.improvements as u64,
            contradiction_count: input.experiment_result.diff.regressions as u64,
        }],
        verification: None,
        trace_id: input.trace_id,
        experiment_diff: Some(input.experiment_result.diff.clone()),
        attribution_json: None,
        assessment: None,
        warnings: {
            let mut warnings = input.known_threats.clone();
            // Authority gate: kernel-generated plans are NonAuthoritativeDerived
            if matches!(&input.plan, PlanKind::PairedPatch { description, .. }
                if description.contains("kernel-generated") || description.contains("NonAuthoritativeDerived"))
            {
                warnings.insert(
                    0,
                    "AUTHORITY: NonAuthoritativeDerived — this evidence originated from a kernel-inferred syndrome and requires human review before promotion to authoritative status.".into(),
                );
            }
            warnings
        },
        created_at: chrono::Utc::now().to_rfc3339(),
        run_id: Some(input.experiment_result.run_id.clone()),
        attempt_id: Some(attempt_id.to_string()),
        causal_question: Some(format!(
            "Does {} improve the selected fixture?",
            input.target_key
        )),
        unit_definition: Some("paired patch experiment".into()),
        bundle_scope: Some(BundleScope {
            workload_id: input.target_key.to_string(),
            backend_family: format!("{:?}", input.experiment_result.mode),
            selected_checks: vec!["fmt".into(), "clippy".into(), "test".into()],
            timeout_class: "forge".into(),
            config_flags: vec!["canonical_v3".into()],
        }),
        pair_comparability: Some(PairComparability {
            valid: true,
            violations: vec![],
        }),
        claim_strength: ClaimStrength::ProvisionalSinglePair,
        identification_rationale: Some(
            "paired baseline/patched experiment over the same fixture".into(),
        ),
        known_threats: input.known_threats,
        patch_hash: Some(patch_hash.clone()),
        treatment: Some(Treatment {
            kind: "patch_applied".into(),
            patch_hash,
            patch_summary: format!("{:?}", input.plan),
        }),
        outcome: Some(format!(
            "paired patch completed: improvements={} regressions={}",
            input.experiment_result.diff.improvements, input.experiment_result.diff.regressions
        )),
        covariates: None,
        promotion_state: None,
        primary_effect: input.experiment_result.diff.effects.first().map(|effect| {
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
            .experiment_result
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
        receipts: vec![inline_receipt("patch-summary", &receipt_payload)],
        verification_trials: vec![
            VerificationTrial {
                trial_id: TrialId::generate(),
                attempt_id: attempt_id.clone(),
                baseline_or_patch: BaselineOrPatch::Baseline,
                completed: true,
                receipts: vec!["patch-summary".into()],
            },
            VerificationTrial {
                trial_id: TrialId::generate(),
                attempt_id,
                baseline_or_patch: BaselineOrPatch::Patched,
                completed: true,
                receipts: vec!["patch-summary".into()],
            },
        ],
        refutation_artifacts: vec![],
        sealed: false,
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
