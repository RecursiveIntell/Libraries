//! Phase 5 finish-gate tests.
//!
//! These tests verify the Phase 5 contracts from the spec:
//! - Pair comparability enforcement
//! - Timeout/retry asymmetry handling
//! - Hypothesis edge model
//! - Bundle claim strength honesty
//! - Receipt integrity
//! - Episode conversion shape
//! - Sealed bundle immutability
//! - Verification plan budget
//! - Limit enforcement

use forge_engine::experiment::*;
use forge_engine::lab::evaluate::ScoreVector;
use forge_engine::lab::evidence::*;
use forge_engine::{
    build_hypothesis_edges, ClaimStrength, ExperimentEvidenceBundle, ForgeLimits,
    HypothesisEdgeKind, HypothesisStatus, PairComparability, ReceiptKind, ReceiptRef,
    ReceiptStorage, Treatment, VerificationState,
};

fn make_bundle() -> ExperimentEvidenceBundle {
    ExperimentEvidenceBundle {
        bundle_id: "b-p5".into(),
        candidate_id: "c-p5".into(),
        eval_id: "e-p5".into(),
        version_id: "v0001".into(),
        supersedes_claim_version_id: None,
        relation_lineage_hints: Default::default(),
        scores: ScoreVector {
            correctness: 0.9,
            novelty: 0.2,
            stability: 0.6,
            weighted_total: 0.7,
            cea_confidence: None,
            cea_predicted_correctness: None,
        },
        hypotheses: vec![],
        verification: None,
        trace_id: Some("trace-p5".into()),
        experiment_diff: None,
        attribution_json: None,
        assessment: None,
        warnings: vec![],
        created_at: "2026-03-07T00:00:00Z".into(),
        run_id: Some("run-1".into()),
        attempt_id: Some("attempt-1".into()),
        causal_question: Some("Did patch P change outcome O on workload W under scope S?".into()),
        unit_definition: Some("one paired baseline/patched run on one fixed workload slice".into()),
        bundle_scope: Some(BundleScope {
            workload_id: "wl-1".into(),
            backend_family: "host".into(),
            selected_checks: vec!["fmt".into(), "clippy".into(), "test".into()],
            timeout_class: "standard".into(),
            config_flags: vec![],
        }),
        pair_comparability: None,
        claim_strength: ClaimStrength::ProvisionalSinglePair,
        identification_rationale: Some("same workload, same checks, same backend".into()),
        known_threats: vec!["single trial only".into()],
        patch_hash: Some("abc123".into()),
        treatment: Some(Treatment {
            kind: "patch_applied".into(),
            patch_hash: "abc123".into(),
            patch_summary: "2 files, 5 edits".into(),
        }),
        outcome: Some("1 regression, 1 improvement".into()),
        covariates: Some(Covariates {
            env_fingerprint: "env-hash".into(),
            dependency_fingerprint: Some("dep-hash".into()),
            config_flags: vec![],
            workload_id: "wl-1".into(),
            selected_checks: vec!["fmt".into(), "clippy".into(), "test".into()],
            adjacent_edits: false,
            adjacent_edit_signatures: vec![],
        }),
        promotion_state: None,
        primary_effect: Some(TypedLocatedEffect {
            kind: EffectKind::TestFailure,
            file: Some(std::path::PathBuf::from("src/lib.rs")),
            line: Some(42),
            message: "test_foo failed".into(),
            in_baseline: false,
            in_patched: true,
        }),
        all_effects: vec![
            TypedLocatedEffect {
                kind: EffectKind::TestFailure,
                file: Some(std::path::PathBuf::from("src/lib.rs")),
                line: Some(42),
                message: "test_foo failed".into(),
                in_baseline: false,
                in_patched: true,
            },
            TypedLocatedEffect {
                kind: EffectKind::LintFailure,
                file: None,
                line: None,
                message: "fixed: unused import".into(),
                in_baseline: true,
                in_patched: false,
            },
        ],
        hypothesis_edges: vec![],
        receipts: vec![],
        verification_trials: vec![],
        refutation_artifacts: vec![],
        sealed: false,
    }
}

// ── Pair comparability ──

#[test]
fn pair_comparability_checks_mismatch() {
    let result = PairComparability::check(
        "wl-1",
        "wl-1",
        &["fmt".into(), "clippy".into()],
        &["fmt".into(), "test".into()], // different check
        "standard",
        "standard",
        "host",
        "host",
        &[],
        &[],
    );
    assert!(!result.valid);
    assert!(result.violations[0].contains("selected_checks mismatch"));
}

#[test]
fn pair_comparability_backend_mismatch() {
    let result = PairComparability::check(
        "wl-1",
        "wl-1",
        &["fmt".into()],
        &["fmt".into()],
        "standard",
        "standard",
        "host",
        "container", // different backend
        &[],
        &[],
    );
    assert!(!result.valid);
    assert!(result.violations[0].contains("backend mismatch"));
}

#[test]
fn pair_comparability_config_flags_mismatch() {
    let result = PairComparability::check(
        "wl-1",
        "wl-1",
        &["fmt".into()],
        &["fmt".into()],
        "standard",
        "standard",
        "host",
        "host",
        &["flag-a".into()],
        &["flag-b".into()],
    );
    assert!(!result.valid);
    assert!(result.violations[0].contains("config_flags mismatch"));
}

/// Baseline timeout + patched success => invalid paired comparison.
#[test]
fn baseline_timeout_patched_success_invalid() {
    let violation = PairComparability::check_timeout_asymmetry(true, false);
    assert!(
        violation.is_some(),
        "baseline timeout + patched success must be invalid"
    );
}

/// Patched timeout + baseline success => treat as failure outcome.
#[test]
fn patched_timeout_baseline_success_is_failure_outcome() {
    let violation = PairComparability::check_timeout_asymmetry(false, true);
    assert!(
        violation.is_none(),
        "patched timeout is a valid failure outcome, not an asymmetry"
    );
}

// ── Hypothesis edge model ──

#[test]
fn hypothesis_edge_fields() {
    let edge = HypothesisEdge {
        edge_id: "edge-1".into(),
        source_edit: "edit_sig".into(),
        target_effect: "effect_sig".into(),
        kind: HypothesisEdgeKind::CausesRegression,
        status: HypothesisStatus::Supported,
        confidence: 0.5,
        evidence_ids: vec!["b-1".into()],
        contradiction_ids: vec![],
        verification_status: VerificationState::Unverified,
    };

    assert_eq!(edge.kind, HypothesisEdgeKind::CausesRegression);
    assert_eq!(edge.verification_status, VerificationState::Unverified);
    assert!(!edge.evidence_ids.is_empty());
}

#[test]
fn hypothesis_edge_kinds_complete() {
    // Verify all three edge kinds exist
    let _ = HypothesisEdgeKind::CausesRegression;
    let _ = HypothesisEdgeKind::FixesFailure;
    let _ = HypothesisEdgeKind::AssociatedWithStableFailure;
}

#[test]
fn stable_failure_edge_has_zero_confidence() {
    let diff = ExperimentDiff {
        effects: vec![TypedLocatedEffect {
            kind: EffectKind::TestFailure,
            file: None,
            line: None,
            message: "stable fail".into(),
            in_baseline: true,
            in_patched: true,
        }],
        regressions: 0,
        improvements: 0,
        stable_failures: 1,
        stable_passes: 0,
        statistically_meaningful: false,
        sample_warning: None,
    };

    let edges = build_hypothesis_edges(&diff, "b-stable");
    assert_eq!(edges.len(), 1);
    assert_eq!(
        edges[0].kind,
        HypothesisEdgeKind::AssociatedWithStableFailure
    );
    assert_eq!(edges[0].confidence, 0.0);
    assert_eq!(edges[0].status, HypothesisStatus::Neutral);
}

// ── Bundle shape ──

#[test]
fn bundle_has_all_required_fields() {
    let bundle = make_bundle();

    assert!(bundle.run_id.is_some());
    assert!(bundle.attempt_id.is_some());
    assert!(bundle.causal_question.is_some());
    assert!(bundle.unit_definition.is_some());
    assert!(bundle.bundle_scope.is_some());
    assert_eq!(bundle.claim_strength, ClaimStrength::ProvisionalSinglePair);
    assert!(bundle.identification_rationale.is_some());
    assert!(!bundle.known_threats.is_empty());
    assert!(bundle.patch_hash.is_some());
    assert!(bundle.treatment.is_some());
    assert!(bundle.outcome.is_some());
    assert!(bundle.covariates.is_some());
    assert!(bundle.primary_effect.is_some());
    assert!(!bundle.all_effects.is_empty());
}

#[test]
fn bundle_serialization_round_trip_with_phase5_fields() {
    let bundle = make_bundle();
    let json = serde_json::to_string(&bundle).unwrap();
    let deserialized: ExperimentEvidenceBundle = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.bundle_id, bundle.bundle_id);
    assert_eq!(deserialized.run_id, bundle.run_id);
    assert_eq!(deserialized.attempt_id, bundle.attempt_id);
    assert_eq!(deserialized.causal_question, bundle.causal_question);
    assert_eq!(deserialized.claim_strength, bundle.claim_strength);
    assert_eq!(deserialized.patch_hash, bundle.patch_hash);
    assert!(deserialized.primary_effect.is_some());
    assert_eq!(deserialized.all_effects.len(), 2);
    assert!(!deserialized.sealed);
}

// ── Sealed bundle ──

#[test]
fn sealed_bundle_blocks_reseal() {
    let mut bundle = make_bundle();
    bundle.seal().unwrap();
    assert!(bundle.sealed);

    let err = bundle.seal().unwrap_err();
    assert!(format!("{err}").contains("already sealed"));
}

#[test]
fn sealed_bundle_check_not_sealed() {
    let mut bundle = make_bundle();
    assert!(bundle.check_not_sealed().is_ok());

    bundle.seal().unwrap();
    assert!(bundle.check_not_sealed().is_err());
}

// ── Receipt integrity ──

#[test]
fn receipt_ref_has_required_fields() {
    let receipt = ReceiptRef {
        receipt_id: "r-1".into(),
        kind: ReceiptKind::TrialLog,
        storage: ReceiptStorage::Inline("log data".into()),
        content_hash: blake3::hash(b"log data").to_hex().to_string(),
        trace_id: Some("trace-1".into()),
        replay_handle: None,
    };

    assert!(receipt.verify_content(b"log data"));
    assert!(!receipt.verify_content(b"tampered"));
}

#[test]
fn receipt_kind_variants() {
    let _ = ReceiptKind::TrialLog;
    let _ = ReceiptKind::TrialMetrics;
    let _ = ReceiptKind::CheckResult;
    let _ = ReceiptKind::PatchApplicationRecord;
}

#[test]
fn receipt_storage_variants() {
    let _inline = ReceiptStorage::Inline("data".into());
    let _store = ReceiptStorage::StoreRow {
        table: "evidence_bundles".into(),
        key: "b-1".into(),
    };
    let _path = ReceiptStorage::ArtifactPath("/tmp/artifact.json".into());
}

// ── Episode conversion ──

#[test]
fn episode_content_includes_required_fields() {
    let bundle = make_bundle();
    let content = bundle.to_episode_content();

    // Must include
    assert!(content.contains("b-p5"), "bundle_id");
    assert!(content.contains("provisional"), "claim strength");
    assert!(content.contains("test_foo"), "primary effect");
    assert!(content.contains("wl-1"), "workload");
    assert!(content.contains("2 files"), "patch summary");

    // Must NOT include giant raw logs
    assert!(
        content.len() < 5000,
        "episode content should be concise, got {} bytes",
        content.len()
    );
}

#[test]
fn episode_meta_includes_phase5_fields() {
    let bundle = make_bundle();
    let meta = bundle.to_episode_meta();

    assert_eq!(meta["type"], "forge_evidence");
    assert_eq!(meta["bundle_id"], "b-p5");
    assert!(meta["claim_strength"].as_str().is_some());
    assert_eq!(meta["run_id"], "run-1");
    assert_eq!(meta["attempt_id"], "attempt-1");
    assert_eq!(meta["patch_hash"], "abc123");
}

// ── Causal question rendering ──

#[test]
fn causal_question_format() {
    let q = ExperimentEvidenceBundle::render_causal_question(
        "P (2 files, 5 edits)",
        "test_foo failure",
        "wl-1",
        "host/standard",
    );
    assert!(q.starts_with("Did patch"));
    assert!(q.contains("test_foo"));
    assert!(q.contains("wl-1"));
}

// ── Unit definition ──

#[test]
fn unit_definition_describes_paired_run() {
    let bundle = make_bundle();
    let unit = bundle.unit_definition.unwrap();
    assert!(unit.contains("paired"));
    assert!(unit.contains("baseline"));
    assert!(unit.contains("patched"));
}

// ── Derive status Phase 5 semantics ──

#[test]
fn derive_status_phase5_semantics() {
    // Equal nonzero -> Neutral (not Confirmed or Supported)
    assert_eq!(derive_status(5, 5), HypothesisStatus::Neutral);
    assert_eq!(derive_status(1, 1), HypothesisStatus::Neutral);

    // More contradictions -> Contradicted (not Refuted)
    assert_eq!(derive_status(2, 5), HypothesisStatus::Contradicted);
}

// ── Limits enforcement shapes ──

#[test]
fn forge_limits_has_patch_and_graph_limits() {
    let limits = ForgeLimits::default();
    assert!(limits.max_patch_files > 0);
    assert!(limits.max_patch_bytes > 0);
    assert!(limits.max_check_runtime_secs > 0);
    assert!(limits.max_check_output_bytes > 0);
    assert!(limits.max_graph_nodes > 0);
    assert!(limits.max_graph_edges > 0);
}

// ── Verification plan budget ──

#[test]
fn verification_plan_records_dropped_steps() {
    let limits = ForgeLimits {
        max_verification_steps: 1,
        ..ForgeLimits::default()
    };

    // Create a bundle with a diff that would generate many steps
    let diff = ExperimentDiff {
        effects: vec![
            TypedLocatedEffect {
                kind: EffectKind::TestFailure,
                file: None,
                line: None,
                message: "fail1".into(),
                in_baseline: false,
                in_patched: true,
            },
            TypedLocatedEffect {
                kind: EffectKind::TestFailure,
                file: None,
                line: None,
                message: "fail2".into(),
                in_baseline: false,
                in_patched: true,
            },
        ],
        regressions: 2,
        improvements: 0,
        stable_failures: 0,
        stable_passes: 1,
        statistically_meaningful: false,
        sample_warning: None,
    };

    let mut bundle = make_bundle();
    bundle.experiment_diff = Some(diff);

    let policy = VerificationPolicy::default();
    let plan = generate_verification_plan(&bundle, &[], &policy, &limits);

    assert_eq!(plan.steps.len(), 1, "must respect limit");
    assert!(!plan.dropped_steps.is_empty(), "must record dropped steps");
    assert!(plan.dropped_steps[0]
        .reason
        .contains("max_verification_steps"));
}

// ── ID model ──

#[test]
fn attempt_id_is_not_trace_id() {
    let bundle = make_bundle();
    assert_ne!(
        bundle.attempt_id.as_deref(),
        bundle.trace_id.as_deref(),
        "attempt_id must not be the same as trace_id"
    );
}

// ── Coverage: covariates shape ──

#[test]
fn covariates_has_all_required_fields() {
    let cov = Covariates {
        env_fingerprint: "env".into(),
        dependency_fingerprint: Some("dep".into()),
        config_flags: vec!["flag1".into()],
        workload_id: "wl-1".into(),
        selected_checks: vec!["fmt".into(), "test".into()],
        adjacent_edits: true,
        adjacent_edit_signatures: vec!["sig1".into()],
    };

    assert!(!cov.env_fingerprint.is_empty());
    assert!(cov.dependency_fingerprint.is_some());
    assert!(!cov.config_flags.is_empty());
    assert!(!cov.workload_id.is_empty());
    assert!(!cov.selected_checks.is_empty());
    assert!(cov.adjacent_edits);
    assert!(!cov.adjacent_edit_signatures.is_empty());
}
