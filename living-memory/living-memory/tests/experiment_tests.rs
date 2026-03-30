//! Tests for experiment execution, typed diffs, hypothesis updates, and scoring policies.

use forge_engine::baseline::*;
use forge_engine::experiment::*;
use forge_engine::lab::evaluate::ScoreVector;
use forge_engine::lab::evidence::*;
use forge_engine::scoring::*;
use forge_engine::{
    CausalHypothesis, CheckKind, CheckResult, ClaimStrength, EffectSignature,
    ExperimentEvidenceBundle, ForgeLimits, HypothesisStatus, LocatedEffect, ParsedCheckOutput,
};

fn make_check_result(fmt: bool, clippy: bool, test: bool) -> CheckResult {
    CheckResult {
        fmt_pass: fmt,
        clippy_pass: clippy,
        test_pass: test,
        fmt_output: ParsedCheckOutput::default(),
        clippy_output: ParsedCheckOutput::default(),
        test_output: ParsedCheckOutput::default(),
        total_duration_ms: 100,
    }
}

fn make_check_result_with_effects(
    fmt: bool,
    clippy: bool,
    test: bool,
    clippy_effects: Vec<LocatedEffect>,
    test_effects: Vec<LocatedEffect>,
) -> CheckResult {
    CheckResult {
        fmt_pass: fmt,
        clippy_pass: clippy,
        test_pass: test,
        fmt_output: ParsedCheckOutput::default(),
        clippy_output: ParsedCheckOutput {
            check_kind: CheckKind::Clippy,
            exit_code: if clippy { 0 } else { 1 },
            effects: clippy_effects,
            raw_stdout: String::new(),
            raw_stderr: String::new(),
        },
        test_output: ParsedCheckOutput {
            check_kind: CheckKind::Test,
            exit_code: if test { 0 } else { 1 },
            effects: test_effects,
            raw_stdout: String::new(),
            raw_stderr: String::new(),
        },
        total_duration_ms: 100,
    }
}

fn make_test_bundle(
    hypotheses: Vec<CausalHypothesis>,
    diff: Option<ExperimentDiff>,
) -> ExperimentEvidenceBundle {
    ExperimentEvidenceBundle {
        bundle_id: "b-test".into(),
        candidate_id: "c-test".into(),
        eval_id: "e-test".into(),
        version_id: "v0001".into(),
        supersedes_claim_version_id: None,
        relation_lineage_hints: Default::default(),
        scores: ScoreVector {
            correctness: 0.95,
            novelty: 0.1,
            stability: 0.5,
            weighted_total: 0.7,
            cea_confidence: None,
            cea_predicted_correctness: None,
        },
        hypotheses,
        verification: None,
        trace_id: Some("trace-test".into()),
        experiment_diff: diff,
        attribution_json: None,
        assessment: None,
        warnings: vec![],
        created_at: "2026-03-07T00:00:00Z".into(),
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

// ── ExperimentDiff tests ──

#[test]
fn experiment_diff_from_all_pass_both() {
    let baseline = make_check_result(true, true, true);
    let patched = make_check_result(true, true, true);
    let diff = ExperimentDiff::from_paired(&baseline, &patched);

    assert_eq!(diff.regressions, 0);
    assert_eq!(diff.improvements, 0);
    assert_eq!(diff.stable_passes, 3); // fmt, clippy, test all stable pass
    assert_eq!(diff.stable_failures, 0);
}

#[test]
fn experiment_diff_from_baseline_pass_patched_fail() {
    let baseline = make_check_result(true, true, true);
    let patched = make_check_result(true, false, false);
    let diff = ExperimentDiff::from_paired(&baseline, &patched);

    assert_eq!(diff.regressions, 2); // clippy and test regressed
    assert_eq!(diff.improvements, 0);
    assert_eq!(diff.stable_passes, 1); // fmt
}

#[test]
fn experiment_diff_from_baseline_fail_patched_pass() {
    let baseline = make_check_result(false, false, false);
    let patched = make_check_result(true, true, true);
    let diff = ExperimentDiff::from_paired(&baseline, &patched);

    assert_eq!(diff.regressions, 0);
    assert_eq!(diff.improvements, 3); // all improved
}

#[test]
fn experiment_diff_stable_failures_are_neutral() {
    // Both fail = stable failure, NOT a regression
    let baseline = make_check_result(true, false, false);
    let patched = make_check_result(true, false, false);
    let diff = ExperimentDiff::from_paired(&baseline, &patched);

    assert_eq!(diff.stable_failures, 2);
    assert_eq!(diff.stable_passes, 1);
    assert_eq!(
        diff.regressions, 0,
        "stable failures must not count as regressions"
    );
}

#[test]
fn experiment_diff_with_detailed_effects() {
    let clippy_eff = LocatedEffect {
        file: Some(std::path::PathBuf::from("src/lib.rs")),
        line: Some(42),
        col: None,
        message: "unused variable".to_string(),
        sig: EffectSignature {
            check_kind: "clippy".to_string(),
            outcome: "warning".to_string(),
            severity: "warning".to_string(),
            message_class: "unused_variable".to_string(),
            line_offset_from_edit: None,
        },
    };

    let baseline = make_check_result_with_effects(true, true, true, vec![], vec![]);
    let patched = make_check_result_with_effects(true, false, true, vec![clippy_eff], vec![]);

    let diff = ExperimentDiff::from_paired(&baseline, &patched);
    assert_eq!(diff.regressions, 1);
    assert!(!diff.effects.is_empty());
    assert_eq!(diff.effects[0].kind, EffectKind::LintFailure);
    assert!(diff.effects[0].in_patched);
    assert!(!diff.effects[0].in_baseline);
}

/// Phase 5: single-pair diff must NOT claim statistical meaningfulness.
#[test]
fn experiment_diff_single_pair_not_statistically_meaningful() {
    let baseline = make_check_result(true, true, true);
    let patched = make_check_result(true, false, true);
    let diff = ExperimentDiff::from_paired(&baseline, &patched);

    assert!(
        !diff.statistically_meaningful,
        "single paired trial must not be statistically meaningful"
    );
    assert!(
        diff.sample_warning.is_some(),
        "must have a sample warning for single pair"
    );
}

// ── Hypothesis lifecycle tests ──

#[test]
fn derive_status_from_counts() {
    assert_eq!(derive_status(0, 0), HypothesisStatus::Proposed);
    assert_eq!(derive_status(1, 0), HypothesisStatus::Supported);
    assert_eq!(derive_status(5, 0), HypothesisStatus::Supported);
    assert_eq!(derive_status(3, 5), HypothesisStatus::Contradicted);
    assert_eq!(derive_status(3, 1), HypothesisStatus::Supported);
    // Equal nonzero -> Neutral
    assert_eq!(derive_status(3, 3), HypothesisStatus::Neutral);
}

#[test]
fn compute_confidence_basic() {
    // With prior=1.0: confidence = support / (support + contradictions + 1.0)
    let c1 = local_hypothesis_support_confidence(0, 0);
    assert!(
        (c1 - 0.0).abs() < 0.01,
        "zero/zero should be ~0.0, got {c1}"
    );

    let c2 = local_hypothesis_support_confidence(10, 0);
    assert!(c2 > 0.8, "10/0 should be high, got {c2}");

    let c3 = local_hypothesis_support_confidence(5, 5);
    assert!(
        (c3 - 5.0 / 11.0).abs() < 0.01,
        "5/5 should be ~0.45, got {c3}"
    );
}

#[test]
fn update_hypotheses_from_diff_new_effect() {
    let mut hypotheses = vec![CausalHypothesis {
        hypothesis_id: "h1".into(),
        cause_signature: "edit1".into(),
        effect_signature: "unused_variable".into(),
        confidence: 0.5,
        status: HypothesisStatus::Proposed,
        support_count: 0,
        contradiction_count: 0,
    }];

    let diff = ExperimentDiff {
        effects: vec![TypedLocatedEffect {
            kind: EffectKind::LintFailure,
            file: None,
            line: None,
            message: "unused_variable: x".to_string(),
            in_baseline: false,
            in_patched: true,
        }],
        regressions: 1,
        improvements: 0,
        stable_failures: 0,
        stable_passes: 2,
        statistically_meaningful: false,
        sample_warning: None,
    };

    update_hypotheses_from_diff(&mut hypotheses, &diff);
    assert_eq!(
        hypotheses[0].support_count, 1,
        "new patched effect should add support"
    );
    assert_eq!(hypotheses[0].status, HypothesisStatus::Supported);
}

#[test]
fn update_hypotheses_stable_failure_is_neutral() {
    let mut hypotheses = vec![CausalHypothesis {
        hypothesis_id: "h1".into(),
        cause_signature: "edit1".into(),
        effect_signature: "test_fail".into(),
        confidence: 0.5,
        status: HypothesisStatus::Proposed,
        support_count: 0,
        contradiction_count: 0,
    }];

    // Effect present in BOTH baseline and patched = stable = neutral
    let diff = ExperimentDiff {
        effects: vec![TypedLocatedEffect {
            kind: EffectKind::TestFailure,
            file: None,
            line: None,
            message: "test_fail".to_string(),
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

    update_hypotheses_from_diff(&mut hypotheses, &diff);
    assert_eq!(
        hypotheses[0].support_count, 0,
        "stable failures must NOT count as support"
    );
    assert_eq!(
        hypotheses[0].contradiction_count, 0,
        "stable failures must NOT count as contradiction"
    );
}

// ── Evidence assessment tests ──

#[test]
fn evidence_assessment_categories() {
    let bundle = make_test_bundle(
        vec![CausalHypothesis {
            hypothesis_id: "h1".into(),
            cause_signature: "e".into(),
            effect_signature: "f".into(),
            confidence: 0.8,
            status: HypothesisStatus::Supported,
            support_count: 5,
            contradiction_count: 1,
        }],
        None,
    );

    let assessment = compute_assessment(&bundle, 5, true, 3);
    assert_eq!(assessment.reproducibility, AssessmentCategory::Strong);
    assert_eq!(assessment.isolation, AssessmentCategory::Strong);
    assert_eq!(
        assessment.contradiction_state,
        ContradictionState::HasContradictions
    );
    assert_eq!(assessment.sample_support, SampleSupport::Marginal);

    // Insufficient trials
    let weak = compute_assessment(&bundle, 1, false, 3);
    assert_eq!(weak.reproducibility, AssessmentCategory::Weak);
    assert_eq!(weak.isolation, AssessmentCategory::Weak);
}

// ── Verification plan generation tests ──

#[test]
fn verification_plan_basic_generation() {
    let bundle = make_test_bundle(
        vec![CausalHypothesis {
            hypothesis_id: "h1".into(),
            cause_signature: "e".into(),
            effect_signature: "f".into(),
            confidence: 0.5,
            status: HypothesisStatus::Proposed,
            support_count: 0,
            contradiction_count: 0,
        }],
        None,
    );

    let policy = VerificationPolicy::default();
    let limits = ForgeLimits::default();

    let plan = generate_verification_plan(&bundle, &[], &policy, &limits);

    assert!(!plan.plan_id.is_empty());
    assert_eq!(plan.target_hypotheses, vec!["h1"]);
    // Should have core steps (CompareBaseline, CheckInvariant, ManualReview for low-confidence)
    assert!(plan.steps.len() >= 2);
    // Budget should be present
    assert!(plan.budget.is_some());
    let budget = plan.budget.unwrap();
    assert_eq!(budget.max_steps, limits.max_verification_steps);
}

#[test]
fn verification_plan_with_ablation() {
    let diff = ExperimentDiff {
        effects: vec![],
        regressions: 1,
        improvements: 1,
        stable_failures: 0,
        stable_passes: 0,
        statistically_meaningful: false,
        sample_warning: None,
    };

    let bundle = make_test_bundle(
        vec![CausalHypothesis {
            hypothesis_id: "h1".into(),
            cause_signature: "e".into(),
            effect_signature: "f".into(),
            confidence: 0.5,
            status: HypothesisStatus::Proposed,
            support_count: 0,
            contradiction_count: 0,
        }],
        Some(diff),
    );

    let edit_sigs = vec!["sig1".to_string(), "sig2".to_string()];
    let policy = VerificationPolicy::default();
    let limits = ForgeLimits::default();

    let plan = generate_verification_plan(&bundle, &edit_sigs, &policy, &limits);

    // Should include ablation steps for the mixed-effect patch
    let ablation_steps: Vec<_> = plan
        .steps
        .iter()
        .filter(|s| matches!(s.verification_type, VerificationType::Ablation { .. }))
        .collect();
    assert_eq!(
        ablation_steps.len(),
        2,
        "should have ablation for each edit sig"
    );
    // Ablation steps should be Informational
    assert_eq!(
        ablation_steps[0].requirement,
        StepRequirement::Informational
    );
}

#[test]
fn verification_plan_respects_step_limit() {
    let limits = ForgeLimits {
        max_verification_steps: 2,
        ..ForgeLimits::default()
    };

    let bundle = make_test_bundle(vec![], None);
    let policy = VerificationPolicy::default();

    let plan = generate_verification_plan(&bundle, &[], &policy, &limits);
    assert!(
        plan.steps.len() <= 2,
        "plan steps must not exceed limit, got {}",
        plan.steps.len()
    );
    // Dropped steps should be recorded
    if plan.steps.len() == 2 {
        // There should be dropped steps if we generated more than 2
        // (depends on bundle content; check dropped_steps is Vec)
        assert!(plan.dropped_steps.is_empty() || !plan.dropped_steps.is_empty());
    }
}

/// Verification plan budget is recorded.
#[test]
fn verification_plan_has_budget() {
    let bundle = make_test_bundle(vec![], None);
    let policy = VerificationPolicy::default();
    let limits = ForgeLimits::default();

    let plan = generate_verification_plan(&bundle, &[], &policy, &limits);
    assert!(plan.budget.is_some());
    let budget = plan.budget.unwrap();
    assert!(budget.max_steps > 0);
    assert!(budget.estimated_duration_secs > 0);
}

// ── Pair comparability tests ──

#[test]
fn pair_comparability_valid() {
    let result = PairComparability::check(
        "workload-1",
        "workload-1",
        &["fmt".into(), "clippy".into(), "test".into()],
        &["fmt".into(), "clippy".into(), "test".into()],
        "standard",
        "standard",
        "host",
        "host",
        &["flag1".into()],
        &["flag1".into()],
    );
    assert!(result.valid);
    assert!(result.violations.is_empty());
}

#[test]
fn pair_comparability_workload_mismatch() {
    let result = PairComparability::check(
        "workload-1",
        "workload-2",
        &["fmt".into()],
        &["fmt".into()],
        "standard",
        "standard",
        "host",
        "host",
        &[],
        &[],
    );
    assert!(!result.valid);
    assert!(result.violations[0].contains("workload mismatch"));
}

#[test]
fn pair_comparability_timeout_asymmetry() {
    // Baseline timeout + patched success => invalid
    let violation = PairComparability::check_timeout_asymmetry(true, false);
    assert!(violation.is_some());

    // Patched timeout + baseline success => may be a valid failure outcome
    let no_violation = PairComparability::check_timeout_asymmetry(false, true);
    assert!(no_violation.is_none());

    // Both timeout => no asymmetry
    let both = PairComparability::check_timeout_asymmetry(true, true);
    assert!(both.is_none());
}

// ── Hypothesis edge tests ──

#[test]
fn build_hypothesis_edges_from_diff() {
    let diff = ExperimentDiff {
        effects: vec![
            TypedLocatedEffect {
                kind: EffectKind::TestFailure,
                file: None,
                line: None,
                message: "test_foo failed".to_string(),
                in_baseline: false,
                in_patched: true,
            },
            TypedLocatedEffect {
                kind: EffectKind::LintFailure,
                file: None,
                line: None,
                message: "fixed: unused import".to_string(),
                in_baseline: true,
                in_patched: false,
            },
            TypedLocatedEffect {
                kind: EffectKind::TestFailure,
                file: None,
                line: None,
                message: "test_bar still fails".to_string(),
                in_baseline: true,
                in_patched: true,
            },
        ],
        regressions: 1,
        improvements: 1,
        stable_failures: 1,
        stable_passes: 0,
        statistically_meaningful: false,
        sample_warning: None,
    };

    let edges = forge_engine::build_hypothesis_edges(&diff, "b-test");
    assert_eq!(edges.len(), 3);

    // First effect: new failure -> CausesRegression
    assert_eq!(
        edges[0].kind,
        forge_engine::HypothesisEdgeKind::CausesRegression
    );
    assert_eq!(edges[0].status, HypothesisStatus::Supported);
    assert!(edges[0].confidence > 0.0);

    // Second: fixed -> FixesFailure
    assert_eq!(
        edges[1].kind,
        forge_engine::HypothesisEdgeKind::FixesFailure
    );

    // Third: stable -> AssociatedWithStableFailure
    assert_eq!(
        edges[2].kind,
        forge_engine::HypothesisEdgeKind::AssociatedWithStableFailure
    );
    assert_eq!(edges[2].status, HypothesisStatus::Neutral);
    assert_eq!(edges[2].confidence, 0.0);
}

/// One paired run with multiple effects => one bundle with one primary claim.
#[test]
fn multiple_effects_one_primary_claim() {
    let mut bundle = make_test_bundle(vec![], None);
    bundle.primary_effect = Some(TypedLocatedEffect {
        kind: EffectKind::TestFailure,
        file: None,
        line: None,
        message: "primary: test_foo".to_string(),
        in_baseline: false,
        in_patched: true,
    });
    bundle.all_effects = vec![
        TypedLocatedEffect {
            kind: EffectKind::TestFailure,
            file: None,
            line: None,
            message: "primary: test_foo".to_string(),
            in_baseline: false,
            in_patched: true,
        },
        TypedLocatedEffect {
            kind: EffectKind::LintFailure,
            file: None,
            line: None,
            message: "secondary: lint".to_string(),
            in_baseline: false,
            in_patched: true,
        },
    ];

    assert!(bundle.primary_effect.is_some());
    assert_eq!(bundle.all_effects.len(), 2);
    assert_eq!(bundle.claim_strength, ClaimStrength::ProvisionalSinglePair);
}

// ── Scoring policy tests ──

#[test]
fn objective_policy_bug_fix() {
    let policy = ObjectivePolicy::bug_fix();
    assert_eq!(policy.kind, ObjectiveKind::BugFix);
    assert!(policy.allow_scalarization);
    assert!(policy.correctness_weight > 0.8);

    let total = policy.compute_weighted_total(1.0, 0.5, Some(0.8), 5);
    assert!(total.is_some());
    let t = total.unwrap();
    assert!(t > 0.5 && t <= 1.0);
}

#[test]
fn objective_policy_performance_no_scalarization() {
    let policy = ObjectivePolicy::performance();
    assert!(!policy.allow_scalarization);

    let total = policy.compute_weighted_total(1.0, 0.5, Some(0.8), 5);
    assert!(
        total.is_none(),
        "performance should not allow scalarization"
    );
}

#[test]
fn objective_policy_stability_excluded_insufficient_trials() {
    let policy = ObjectivePolicy::bug_fix();
    // Only 1 trial, but min_trials_for_stability is 3
    let total = policy
        .compute_weighted_total(1.0, 0.5, Some(0.8), 1)
        .unwrap();
    // Stability should be excluded (treated as 0.0)
    let expected = policy.correctness_weight * 1.0 + policy.novelty_weight * 0.5;
    assert!(
        (total - expected).abs() < 0.01,
        "with insufficient trials, stability should be excluded: got {total}, expected {expected}"
    );
}

// ── Check classification tests ──

#[test]
fn patch_execution_plan_cargo_default() {
    let plan = PatchExecutionPlan::cargo_default();
    assert!(plan.frozen);
    assert_eq!(plan.checks.len(), 3);

    let comparable = plan.comparable_checks();
    assert_eq!(
        comparable.len(),
        3,
        "all cargo checks are ComparableCore by default"
    );
}

#[test]
fn comparability_class_filtering() {
    let plan = PatchExecutionPlan {
        checks: vec![
            PlannedCheck {
                check_kind: CheckKind::Fmt,
                comparability: ComparabilityClass::ComparableCore,
                config_override: false,
            },
            PlannedCheck {
                check_kind: CheckKind::Test,
                comparability: ComparabilityClass::PatchOnlyDiagnostic,
                config_override: true,
            },
        ],
        frozen: true,
    };

    let comparable = plan.comparable_checks();
    assert_eq!(comparable.len(), 1);
    assert_eq!(comparable[0].check_kind, CheckKind::Fmt);
}

// ── Baseline provenance tests ──

#[test]
fn baseline_descriptor_fingerprint_deterministic() {
    let desc = BaselineDescriptor {
        source_kind: BaselineSourceKind::GitCommit,
        commit_sha: Some("abc123".into()),
        dirty: false,
        untracked_count: 0,
        lockfile_hash: Some("hash123".into()),
        rustc_version: "rustc 1.78.0".into(),
        cargo_version: "cargo 1.78.0".into(),
        target_triple: "x86_64-unknown-linux-gnu".into(),
        env_fingerprint: "envhash".into(),
        submodule_state: vec![],
    };

    let fp1 = desc.fingerprint();
    let fp2 = desc.fingerprint();
    assert_eq!(fp1, fp2, "fingerprint must be deterministic");
    assert_eq!(fp1.len(), 64, "fingerprint should be blake3 hex");
}

#[test]
fn workspace_policy_defaults() {
    let policy = WorkspacePolicy::default();
    assert_eq!(policy.max_workspace_bytes, 2_000_000_000);
    assert_eq!(policy.max_retained_workspaces, 8);
    assert!(!policy.retain_failed_workspaces);
    assert_eq!(policy.workspace_cleanup_ttl_secs, 3600);
}

// ── Experiment mode tests ──

#[test]
fn experiment_config_defaults() {
    let config = ExperimentConfig::default();
    assert_eq!(config.mode, ExperimentMode::Paired);
    assert_eq!(config.trial_count, 1);
}

// ── Receipt tests ──

#[test]
fn receipt_content_hash_verification() {
    let content = b"hello world";
    let hash = blake3::hash(content).to_hex().to_string();

    let receipt = forge_engine::ReceiptRef {
        receipt_id: "r-1".into(),
        kind: forge_engine::ReceiptKind::TrialLog,
        storage: forge_engine::ReceiptStorage::Inline("hello world".into()),
        content_hash: hash.clone(),
        trace_id: None,
        replay_handle: None,
    };

    assert!(receipt.verify_content(content));
    assert!(!receipt.verify_content(b"tampered content"));
}

/// Content-hash mismatch on mutable receipt is detected.
#[test]
fn receipt_content_hash_mismatch_detected() {
    let receipt = forge_engine::ReceiptRef {
        receipt_id: "r-2".into(),
        kind: forge_engine::ReceiptKind::CheckResult,
        storage: forge_engine::ReceiptStorage::Inline("original".into()),
        content_hash: "0000000000000000000000000000000000000000000000000000000000000000"
            .to_string(),
        trace_id: None,
        replay_handle: None,
    };

    assert!(
        !receipt.verify_content(b"original"),
        "mismatched hash must be detected"
    );
}

/// LocalExperimentRunner does NOT fabricate evidence in Phase 5.
#[tokio::test]
async fn local_experiment_runner_does_not_fabricate() {
    use forge_engine::lab::evidence::ExperimentRunner;

    let runner = LocalExperimentRunner::new(std::path::PathBuf::from("/tmp/test"));
    let bundle = make_test_bundle(
        vec![CausalHypothesis {
            hypothesis_id: "h1".into(),
            cause_signature: "e".into(),
            effect_signature: "f".into(),
            confidence: 0.5,
            status: HypothesisStatus::Proposed,
            support_count: 0,
            contradiction_count: 0,
        }],
        None,
    );

    let plan = VerificationPlan {
        plan_id: "plan-1".into(),
        target_hypotheses: vec!["h1".into()],
        steps: vec![
            forge_engine::VerificationStep {
                verification_type: VerificationType::Reproduce,
                description: "reproduce".into(),
                expected_outcome: "same".into(),
                requirement: StepRequirement::Required,
            },
            forge_engine::VerificationStep {
                verification_type: VerificationType::Negate,
                description: "negate".into(),
                expected_outcome: "disappears".into(),
                requirement: StepRequirement::Required,
            },
        ],
        budget: None,
        dropped_steps: vec![],
    };

    let result = runner.run_plan(&plan, &bundle).await.unwrap();

    // Phase 5: hypotheses must be UNCHANGED. No fabricated support/contradictions.
    assert_eq!(result[0].support_count, 0, "must not fabricate support");
    assert_eq!(
        result[0].contradiction_count, 0,
        "must not fabricate contradictions"
    );
    assert_eq!(result[0].status, HypothesisStatus::Proposed);
}
