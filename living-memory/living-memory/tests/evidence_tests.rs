#![allow(clippy::expect_used)]

//! Tests for VG-12: ExperimentEvidenceBundle, Experiments, ForgeLimits (Phase 5)

use forge_engine::lab::evaluate::ScoreVector;
use forge_engine::lab::evidence::{
    BaselineOrPatch, ReceiptKind, ReceiptRef, ReceiptStorage, RefutationArtifact,
    RefutationArtifactOutcome, RefutationArtifactType,
};
use forge_engine::{
    CausalHypothesis, ClaimStrength, ExperimentEvidenceBundle, ForgeLimits, HypothesisStatus,
    LocalExperimentRunner, VerificationPlan, VerificationType,
};
use stack_ids::{AttemptId, TrialId};

/// Helper to construct an ExperimentEvidenceBundle with all Phase 5 fields defaulted.
#[allow(clippy::too_many_arguments)]
fn test_bundle(
    bundle_id: &str,
    candidate_id: &str,
    eval_id: &str,
    version_id: &str,
    scores: ScoreVector,
    hypotheses: Vec<CausalHypothesis>,
    verification: Option<VerificationPlan>,
    trace_id: Option<String>,
) -> ExperimentEvidenceBundle {
    ExperimentEvidenceBundle {
        bundle_id: bundle_id.into(),
        candidate_id: candidate_id.into(),
        eval_id: eval_id.into(),
        version_id: version_id.into(),
        supersedes_claim_version_id: None,
        relation_lineage_hints: Default::default(),
        scores,
        hypotheses,
        verification,
        trace_id,
        experiment_diff: None,
        attribution_json: None,
        assessment: None,
        warnings: vec![],
        created_at: "2026-03-07T00:00:00Z".into(),
        // Phase 5 fields
        run_id: None,
        attempt_id: None,
        causal_question: None,
        unit_definition: None,
        bundle_scope: None,
        pair_comparability: None,
        claim_strength: ClaimStrength::ProvisionalSinglePair,
        identification_rationale: None,
        known_threats: vec![],
        patch_hash: None,
        treatment: None,
        outcome: None,
        covariates: None,
        promotion_state: None,
        primary_effect: None,
        all_effects: vec![],
        hypothesis_edges: vec![],
        receipts: vec![],
        verification_trials: vec![],
        refutation_artifacts: vec![],
        sealed: false,
    }
}

/// VG-12 Check 10: Hypothesis confidence bounds.
#[test]
fn confidence_bounds_on_hypothesis() {
    let h = CausalHypothesis {
        hypothesis_id: "h-001".into(),
        cause_signature: "edit_sig_abc".into(),
        effect_signature: "effect_sig_xyz".into(),
        confidence: 0.85,
        status: HypothesisStatus::Supported,
        support_count: 17,
        contradiction_count: 3,
    };
    assert!(h.confidence >= 0.0 && h.confidence <= 1.0);
    assert_eq!(h.status, HypothesisStatus::Supported);

    // Test all status variants
    assert_ne!(HypothesisStatus::Proposed, HypothesisStatus::Contradicted);
    assert_ne!(HypothesisStatus::Neutral, HypothesisStatus::Supported);
}

/// VG-12 Check 11: Serialization round-trip for ExperimentEvidenceBundle.
#[test]
fn serialization_round_trip_evidence_bundle() {
    let bundle = test_bundle(
        "b-001",
        "c-001",
        "e-001",
        "v0001",
        ScoreVector {
            correctness: 0.95,
            novelty: 0.3,
            stability: 0.8,
            weighted_total: 0.75,
            cea_confidence: Some(0.6),
            cea_predicted_correctness: Some(0.9),
        },
        vec![CausalHypothesis {
            hypothesis_id: "h-001".into(),
            cause_signature: "cause".into(),
            effect_signature: "effect".into(),
            confidence: 0.7,
            status: HypothesisStatus::Proposed,
            support_count: 5,
            contradiction_count: 1,
        }],
        Some(VerificationPlan {
            plan_id: "plan-001".into(),
            target_hypotheses: vec!["h-001".into()],
            steps: vec![],
            budget: None,
            dropped_steps: vec![],
        }),
        Some("trace-abc".into()),
    );

    let json = serde_json::to_string(&bundle).unwrap();
    let deserialized: ExperimentEvidenceBundle = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.bundle_id, "b-001");
    assert_eq!(deserialized.hypotheses.len(), 1);
    assert_eq!(deserialized.hypotheses[0].confidence, 0.7);
    assert_eq!(
        deserialized.hypotheses[0].status,
        HypothesisStatus::Proposed
    );
    assert!(deserialized.verification.is_some());
    assert_eq!(deserialized.trace_id.as_deref(), Some("trace-abc"));
    assert_eq!(
        deserialized.claim_strength,
        ClaimStrength::ProvisionalSinglePair
    );
    assert!(!deserialized.sealed);
}

#[test]
fn canonical_bundle_roundtrip_preserves_authoritative_contract() {
    let mut bundle = test_bundle(
        "b-canonical-roundtrip",
        "c-canonical",
        "e-canonical",
        "v-canonical",
        ScoreVector {
            correctness: 0.91,
            novelty: 0.25,
            stability: 0.73,
            weighted_total: 0.76,
            cea_confidence: Some(0.64),
            cea_predicted_correctness: Some(0.82),
        },
        vec![CausalHypothesis {
            hypothesis_id: "h-canonical".into(),
            cause_signature: "edit:canonical".into(),
            effect_signature: "effect:canonical".into(),
            confidence: 0.66,
            status: HypothesisStatus::Supported,
            support_count: 2,
            contradiction_count: 0,
        }],
        Some(VerificationPlan {
            plan_id: "plan-canonical".into(),
            target_hypotheses: vec!["h-canonical".into()],
            steps: vec![],
            budget: None,
            dropped_steps: vec![],
        }),
        Some("trace-canonical".into()),
    );
    bundle.run_id = Some("run-canonical".into());
    bundle.attempt_id = Some("attempt:attempt-canonical".into());
    bundle.causal_question = Some("Did the canonical adapter preserve bundle authority?".into());
    bundle.unit_definition = Some("one paired canonical adapter proof".into());
    bundle.identification_rationale = Some("same backend, same scope, same adapter".into());
    bundle.known_threats = vec!["selection_bias".into()];
    bundle.treatment = Some(forge_engine::Treatment {
        kind: "patch_applied".into(),
        patch_hash: "sha256:canonical".into(),
        patch_summary: "guard canonical adapter path".into(),
    });
    bundle.outcome = Some("verified".into());
    bundle.receipts.push(ReceiptRef {
        receipt_id: "receipt-canonical".into(),
        kind: ReceiptKind::CheckResult,
        storage: ReceiptStorage::StoreRow {
            table: "receipts".into(),
            key: "canonical".into(),
        },
        content_hash: "blake3:canonical".into(),
        trace_id: Some("trace-canonical".into()),
        replay_handle: Some("replay://canonical".into()),
    });
    bundle
        .verification_trials
        .push(forge_engine::lab::evidence::VerificationTrial {
            trial_id: TrialId::new("trial-canonical"),
            attempt_id: AttemptId::new("attempt-canonical"),
            baseline_or_patch: BaselineOrPatch::Patched,
            completed: true,
            receipts: vec!["receipt-canonical".into()],
        });
    bundle.refutation_artifacts.push(RefutationArtifact {
        artifact_id: "artifact-canonical".into(),
        artifact_type: RefutationArtifactType::Placebo,
        trial_id: Some(TrialId::new("trial-canonical")),
        attempt_id: Some(AttemptId::new("attempt-canonical")),
        outcome: RefutationArtifactOutcome::Passed,
        estimate_delta: Some(0.01),
        details: Some("stable under placebo".into()),
    });

    let canonical = bundle.to_canonical_evidence_bundle();
    let rebuilt = ExperimentEvidenceBundle::from_canonical_evidence_bundle(&canonical).unwrap();
    let canonical_again = rebuilt.to_canonical_evidence_bundle();

    assert_eq!(
        serde_json::to_value(&canonical).unwrap(),
        serde_json::to_value(&canonical_again).unwrap(),
        "local wrapper roundtrip must preserve the canonical evidence contract exactly"
    );
    let metadata = canonical
        .metadata
        .as_ref()
        .expect("canonical bundle metadata should persist");
    assert_eq!(
        metadata
            .get("canonical_owner")
            .and_then(|value| value.as_str()),
        Some("semantic_memory_forge::EvidenceBundle")
    );
    assert!(metadata.get("living_memory_authoring").is_some());
    assert_eq!(
        canonical
            .estimator_meta
            .as_ref()
            .and_then(|meta| meta.response_schema_version.as_deref()),
        Some("semantic_memory_forge.evidence_bundle.v3")
    );
    assert_eq!(
        canonical.question.description,
        "Did the canonical adapter preserve bundle authority?"
    );
    assert_eq!(
        canonical.treatment.description,
        "guard canonical adapter path [sha256:canonical]"
    );
    assert_eq!(canonical.outcome.description, "verified");
    assert!(canonical
        .covariates
        .iter()
        .any(|entry| entry.contains("known_threat:selection_bias")));
    assert_eq!(
        canonical.identification_rationale,
        "same backend, same scope, same adapter"
    );
    assert_eq!(canonical.verification_trials.len(), 1);
    assert_eq!(canonical.refutations.len(), 1);
    assert_eq!(canonical.refutation_artifacts.len(), 1);
    assert_eq!(
        canonical.raw_receipt_handle.as_deref(),
        Some("receipt:store:receipts:canonical")
    );
}

/// ForgeLimits defaults including new Phase 5 fields.
#[test]
fn forge_limits_defaults() {
    let limits = ForgeLimits::default();
    assert_eq!(limits.max_hypotheses, 100);
    assert_eq!(limits.max_concurrent_experiments, 4);
    assert_eq!(limits.max_bundles_per_candidate, 50);
    assert_eq!(limits.max_verification_steps, 20);
    assert_eq!(limits.max_retained_artifacts, 100);
    assert_eq!(limits.failed_run_retention_days, 30);
    // Phase 5 limits
    assert_eq!(limits.max_patch_files, 16);
    assert_eq!(limits.max_patch_bytes, 1_000_000);
    assert_eq!(limits.max_check_runtime_secs, 300);
    assert_eq!(limits.max_check_output_bytes, 10_000_000);
    assert_eq!(limits.max_graph_nodes, 10_000);
    assert_eq!(limits.max_graph_edges, 50_000);
}

/// LocalExperimentRunner construction.
#[test]
fn local_experiment_runner_construction() {
    let runner =
        LocalExperimentRunner::new(std::path::PathBuf::from("/tmp/exp")).with_max_parallelism(8);
    assert_eq!(runner.max_parallelism, 8);

    // Clamp to 32 max
    let runner2 =
        LocalExperimentRunner::new(std::path::PathBuf::from("/tmp")).with_max_parallelism(100);
    assert_eq!(runner2.max_parallelism, 32);
}

/// VerificationType variants.
#[test]
fn verification_type_variants() {
    assert_ne!(VerificationType::Reproduce, VerificationType::Generalize);
    assert_ne!(VerificationType::Negate, VerificationType::CrossProject);
    assert_ne!(VerificationType::RunTest, VerificationType::CheckInvariant);
    assert_ne!(
        VerificationType::CompareBaseline,
        VerificationType::ManualReview
    );
}

/// VG-13 Check 4: Episode conversion produces valid metadata and content.
#[test]
fn episode_conversion_from_evidence_bundle() {
    let bundle = test_bundle(
        "b-ep",
        "c-ep",
        "e-ep",
        "v0002",
        ScoreVector {
            correctness: 0.9,
            novelty: 0.4,
            stability: 0.7,
            weighted_total: 0.72,
            cea_confidence: None,
            cea_predicted_correctness: None,
        },
        vec![CausalHypothesis {
            hypothesis_id: "h-ep".into(),
            cause_signature: "cause_sig".into(),
            effect_signature: "effect_sig".into(),
            confidence: 0.6,
            status: HypothesisStatus::Proposed,
            support_count: 3,
            contradiction_count: 0,
        }],
        None,
        Some("trace-ep".into()),
    );

    let meta = bundle.to_episode_meta();
    assert_eq!(meta["type"], "forge_evidence");
    assert_eq!(meta["bundle_id"], "b-ep");
    assert_eq!(meta["candidate_id"], "c-ep");
    assert_eq!(meta["hypothesis_count"], 1);
    assert_eq!(meta["has_verification"], false);

    let content = bundle.to_episode_content();
    assert!(content.contains("b-ep"));
    assert!(content.contains("c-ep"));
    assert!(content.contains("correctness=0.90"));
    assert!(content.contains("Hypothesis h-ep"));
    assert!(content.contains("confidence=0.60"));
    // Phase 5: claim strength should be present
    assert!(content.contains("provisional local attribution"));
}

#[test]
fn episode_meta_and_content_capture_verification_artifacts() {
    let mut bundle = test_bundle(
        "b-verification-artifacts",
        "c-verification",
        "e-verification",
        "v-verify-1",
        ScoreVector {
            correctness: 0.88,
            novelty: 0.22,
            stability: 0.75,
            weighted_total: 0.7,
            cea_confidence: Some(0.66),
            cea_predicted_correctness: Some(0.81),
        },
        vec![],
        None,
        Some("trace-verification-1".into()),
    );

    bundle.run_id = Some("run-verification-1".into());
    bundle.attempt_id = Some("attempt-verification-1".into());
    bundle.causal_question =
        Some("Did patch fix the assertion while preserving behavior under workload X?".into());
    bundle.identification_rationale = Some("same workload, same checks, same backend".into());
    bundle.known_threats = vec!["selection_bias".into()];
    bundle.patch_hash = Some("sha256:deadbeef".into());
    bundle.treatment = Some(forge_engine::lab::evidence::Treatment {
        kind: "patch_applied".into(),
        patch_hash: "sha256:deadbeef".into(),
        patch_summary: "replace helper with guarded branch".into(),
    });
    bundle.outcome = Some("one regression removed".into());
    bundle.covariates = Some(forge_engine::lab::evidence::Covariates {
        env_fingerprint: "linux-x86_64".into(),
        dependency_fingerprint: Some("deps-sha-7".into()),
        config_flags: vec!["feature=true".into()],
        workload_id: "workload-42".into(),
        selected_checks: vec!["clippy".into(), "cargo-test".into()],
        adjacent_edits: false,
        adjacent_edit_signatures: vec![],
    });
    bundle.verification_trials = vec![
        forge_engine::lab::evidence::VerificationTrial {
            trial_id: TrialId::new("trial-baseline-1"),
            attempt_id: AttemptId::new("attempt-verification-1"),
            baseline_or_patch: BaselineOrPatch::Baseline,
            completed: true,
            receipts: vec!["baseline-run.log".into()],
        },
        forge_engine::lab::evidence::VerificationTrial {
            trial_id: TrialId::new("trial-patched-1"),
            attempt_id: AttemptId::new("attempt-verification-1"),
            baseline_or_patch: BaselineOrPatch::Patched,
            completed: true,
            receipts: vec!["patched-run.log".into()],
        },
    ];
    bundle.refutation_artifacts = vec![
        RefutationArtifact {
            artifact_id: "ref-placebo-1".into(),
            artifact_type: RefutationArtifactType::Placebo,
            trial_id: Some(TrialId::new("trial-baseline-1")),
            attempt_id: Some(AttemptId::new("attempt-verification-1")),
            outcome: RefutationArtifactOutcome::Passed,
            estimate_delta: Some(0.0),
            details: Some("no effect for placebo".into()),
        },
        RefutationArtifact {
            artifact_id: "ref-subsample-1".into(),
            artifact_type: RefutationArtifactType::SubsampleStability,
            trial_id: Some(TrialId::new("trial-patched-1")),
            attempt_id: Some(AttemptId::new("attempt-verification-1")),
            outcome: RefutationArtifactOutcome::Failed {
                reason: "high variance across folds".into(),
            },
            estimate_delta: Some(1.4),
            details: Some("fold 2 diverges".into()),
        },
        RefutationArtifact {
            artifact_id: "ref-dummy-outcome-1".into(),
            artifact_type: RefutationArtifactType::DummyOutcome,
            trial_id: Some(TrialId::new("trial-patched-1")),
            attempt_id: Some(AttemptId::new("attempt-verification-1")),
            outcome: RefutationArtifactOutcome::Inconclusive {
                reason: "insufficient signal for nullification check".into(),
            },
            estimate_delta: Some(0.2),
            details: Some("dummy outcome check did not reach threshold".into()),
        },
    ];
    bundle.warnings = vec!["sampling_warning".into()];

    let meta = bundle.to_episode_meta();

    assert_eq!(meta["type"], "forge_evidence");
    assert_eq!(meta["bundle_id"], "b-verification-artifacts");
    assert_eq!(meta["outcome"], "one regression removed");
    assert_eq!(
        meta["identification_rationale"],
        "same workload, same checks, same backend"
    );
    assert_eq!(meta["patch_hash"], "sha256:deadbeef");
    assert!(meta["confounders"].as_array().is_some());
    assert_eq!(
        meta["estimator_metadata"]["cea_confidence"],
        serde_json::json!(0.66)
    );
    assert_eq!(
        meta["estimator_metadata"]["cea_predicted_correctness"],
        serde_json::json!(0.81)
    );
    assert_eq!(meta["verification_trial_count"], 2);
    assert_eq!(meta["refutation_artifact_count"], 3);

    let content = bundle.to_episode_content();
    assert!(content.contains("Treatment: kind=patch_applied"));
    assert!(content.contains(
        "Trial trial:trial-baseline-1 / attempt attempt:attempt-verification-1: Baseline completed=true"
    ));
    assert!(content.contains(
        "Trial trial:trial-patched-1 / attempt attempt:attempt-verification-1: Patched completed=true"
    ));
    assert!(content.contains("Refutation Placebo"));
    assert!(content.contains("Refutation SubsampleStability failed: high variance across folds"));
    assert!(content.contains("Refutation DummyOutcome"));
    assert!(content.contains("Confounders / threats: selection_bias"));
}

/// Serialization backward compatibility: old JSON without new fields deserializes.
#[test]
fn backward_compat_old_json_without_new_fields() {
    // Simulate old JSON that doesn't have the v2/Phase 5 fields
    let old_json = r#"{
        "bundle_id": "b-old",
        "candidate_id": "c-old",
        "eval_id": "e-old",
        "version_id": "v0001",
        "scores": {"correctness": 0.9, "novelty": 0.1, "stability": 0.5, "weighted_total": 0.7, "cea_confidence": null, "cea_predicted_correctness": null},
        "hypotheses": [],
        "verification": null,
        "trace_id": null
    }"#;

    let bundle: ExperimentEvidenceBundle = serde_json::from_str(old_json).unwrap();
    assert_eq!(bundle.bundle_id, "b-old");
    assert!(bundle.experiment_diff.is_none());
    assert!(bundle.attribution_json.is_none());
    assert!(bundle.assessment.is_none());
    assert!(bundle.warnings.is_empty());
    // Phase 5 fields should have defaults
    assert!(bundle.run_id.is_none());
    assert!(bundle.attempt_id.is_none());
    assert_eq!(bundle.claim_strength, ClaimStrength::ProvisionalSinglePair);
    assert!(!bundle.sealed);
    assert!(bundle.hypothesis_edges.is_empty());
    assert!(bundle.receipts.is_empty());
}

/// Backward compat: old JSON with "Refuted"/"Confirmed" status deserializes via serde alias.
#[test]
fn backward_compat_old_status_names() {
    let json_refuted = r#"{"hypothesis_id":"h1","cause_signature":"c","effect_signature":"e","confidence":0.1,"status":"Refuted","support_count":0,"contradiction_count":5}"#;
    let h: CausalHypothesis = serde_json::from_str(json_refuted).unwrap();
    assert_eq!(h.status, HypothesisStatus::Contradicted);

    let json_confirmed = r#"{"hypothesis_id":"h2","cause_signature":"c","effect_signature":"e","confidence":0.9,"status":"Confirmed","support_count":10,"contradiction_count":0}"#;
    let h2: CausalHypothesis = serde_json::from_str(json_confirmed).unwrap();
    assert_eq!(h2.status, HypothesisStatus::Neutral);
}

/// Sealed bundle cannot be mutated.
#[test]
fn sealed_bundle_immutability() {
    let mut bundle = test_bundle(
        "b-seal",
        "c-seal",
        "e-seal",
        "v0001",
        ScoreVector {
            correctness: 0.9,
            novelty: 0.2,
            stability: 0.6,
            weighted_total: 0.7,
            cea_confidence: None,
            cea_predicted_correctness: None,
        },
        vec![],
        None,
        None,
    );

    // First seal succeeds
    bundle.seal().unwrap();
    assert!(bundle.sealed);

    // Second seal fails
    let result = bundle.seal();
    assert!(result.is_err());

    // check_not_sealed fails
    let result = bundle.check_not_sealed();
    assert!(result.is_err());
}

/// ClaimStrength display matches spec.
#[test]
fn claim_strength_display() {
    let strength = ClaimStrength::ProvisionalSinglePair;
    let display = format!("{strength}");
    assert!(display.contains("provisional"));
    assert!(display.contains("one paired intervention"));
}
