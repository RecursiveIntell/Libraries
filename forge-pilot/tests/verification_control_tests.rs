mod common;

use common::{
    base_loop_config, import_promoted_hyperedge_batch, import_v3_bundle,
    install_permissive_governance, open_forge_store, open_memory_store, point_config_at_dir,
    resources, sample_bundle, tempdir, write_source_file,
};
use forge_pilot::{
    observe_scope, parse_loop_report_boundary, score_targets, LoopRunner, PilotHistory, PlanKind,
};
use knowledge_runtime::Scope;
use verification_adjudication::VerificationDisposition;
use verification_control::VerificationAttemptState;
use verification_policy::{
    ApprovalRecord, ApprovalRequirement, ApprovalScope, AutonomyCeiling, MethodPolicy,
    PolicySnapshot, POLICY_SNAPSHOT_V1_SCHEMA,
};

fn complete_citation() -> verification_policy::V25CitationContext {
    verification_policy::V25CitationContext {
        applicability_context_id: Some(stack_ids::ApplicabilityContextId::new(
            "pilot-applicability-context",
        )),
        profile_set_id: Some(stack_ids::ProfileSetId::new("pilot-profile-set")),
        composition_receipt_id: Some(stack_ids::CompositionReceiptId::new(
            "pilot-composition-receipt",
        )),
        effective_constitution_id: Some(stack_ids::EffectiveConstitutionId::new(
            "pilot-effective-constitution",
        )),
        compiled_obligation_set_id: Some(stack_ids::CompiledObligationSetId::new(
            "pilot-compiled-obligation-set",
        )),
        composition_conflict_set_id: None,
        profile_exception_bundle_ids: Vec::new(),
    }
}

fn complete_obligation_refs() -> verification_policy::V25ControlObligationRefs {
    verification_policy::V25ControlObligationRefs {
        required_obligation_refs: vec!["obligation:pilot-required".into()],
        blocking_obligation_refs: vec!["obligation:pilot-blocking".into()],
        monitoring_obligation_refs: vec!["obligation:pilot-monitoring".into()],
    }
}

fn plan_method(plan: &PlanKind) -> verification_control::CheckMethod {
    match plan {
        PlanKind::OracleExactBounded { .. } => {
            verification_control::CheckMethod::ExactBoundedOracle
        }
        PlanKind::OracleConservative => verification_control::CheckMethod::ConservativeOracle,
        PlanKind::OracleDeltaParity { .. } => verification_control::CheckMethod::DeltaParityOracle,
        PlanKind::OracleTemporalReplay { .. } => {
            verification_control::CheckMethod::TemporalReplayOracle
        }
        PlanKind::OracleCausalRefuter { .. } => verification_control::CheckMethod::CausalRefuter,
        PlanKind::OracleMinimalPerturbation { .. } => {
            verification_control::CheckMethod::MinimalPerturbationOracle
        }
        PlanKind::PairedPatch { .. } => verification_control::CheckMethod::PairedPatch,
        PlanKind::AdvisoryOnlyVerificationPlan(_) => {
            verification_control::CheckMethod::AdvisoryOnly
        }
    }
}

async fn preview_first_candidate() -> (
    tempfile::TempDir,
    forge_pilot::LoopConfig,
    forge_pilot::TargetCandidate,
) {
    let dir = tempdir();
    let memory_store = open_memory_store(dir.path());
    let forge_store = open_forge_store(dir.path());
    let scope = Scope::new("pilot-v8-control");
    let mut config = base_loop_config(scope.clone());
    point_config_at_dir(&mut config, dir.path());
    write_source_file(
        dir.path(),
        "src/lib.rs",
        "pub fn verification_control_fixture() -> bool { true }\n",
    );

    import_v3_bundle(
        &memory_store,
        &forge_store,
        &scope.namespace,
        &sample_bundle("pilot-v8-control"),
    )
    .await;
    install_permissive_governance(&memory_store).await;

    let resources = resources(memory_store, forge_store, &config);
    let observation = observe_scope(&resources.runtime, &resources.memory_store, &config)
        .await
        .unwrap();
    let candidate = score_targets(&observation, &PilotHistory::default(), &config)
        .into_iter()
        .next()
        .unwrap();

    (dir, config, candidate)
}

#[tokio::test]
async fn full_loop_policy_deny_blocks_execution_before_action() {
    let (_dir, mut config, candidate) = preview_first_candidate().await;
    let method = plan_method(&candidate.plan);
    config.policy_snapshots = vec![PolicySnapshot {
        schema_version: POLICY_SNAPSHOT_V1_SCHEMA.into(),
        policy_version: "policy-deny".into(),
        effective_from: "2026-03-01T00:00:00Z".into(),
        effective_to: None,
        autonomy_ceiling: AutonomyCeiling::PromotionEligible,
        method_rules: vec![MethodPolicy {
            method,
            allowed: false,
            max_autonomy: AutonomyCeiling::PromotionEligible,
            approval_requirement: ApprovalRequirement::None,
        }],
        blocked_promotions_when_degraded: true,
        blocked_promotions_when_budget_exhausted: true,
        citation: complete_citation(),
        obligation_refs: complete_obligation_refs(),
    }];

    let scope = config.scope.clone();
    let memory_store = open_memory_store(_dir.path());
    let forge_store = open_forge_store(_dir.path());
    import_v3_bundle(
        &memory_store,
        &forge_store,
        &scope.namespace,
        &sample_bundle("pilot-v8-control"),
    )
    .await;
    install_permissive_governance(&memory_store).await;
    let resources = resources(memory_store, forge_store, &config);
    let mut runner = LoopRunner::new(config, resources);
    let report = runner.run().await.unwrap();

    assert_eq!(report.actions_executed, 0);
    assert_eq!(
        report.iterations[0]
            .verification_attempt
            .as_ref()
            .unwrap()
            .state,
        VerificationAttemptState::Blocked
    );
    assert_eq!(
        report.iterations[0]
            .adjudication
            .as_ref()
            .unwrap()
            .disposition,
        VerificationDisposition::BlockedByPolicy
    );
    let artifact = report.iterations[0]
        .verification_plan_artifact
        .as_ref()
        .unwrap();
    assert!(!artifact.cheapest_checks.is_empty());
    assert!(artifact.proof_profile.is_some());
    assert!(artifact.promotion_blocked_on_missing_proof);
    assert!(!artifact.proof_obligations_remaining.is_empty());
    assert!(artifact
        .replay_recipe
        .iter()
        .any(|step| step.contains("replay_case")));
    assert!(!artifact.policy_blockers.is_empty());
    assert!(!artifact.blocked_checks.is_empty());
}

#[tokio::test]
async fn full_loop_requires_approval_when_policy_demands_it() {
    let (_dir, mut config, candidate) = preview_first_candidate().await;
    let method = plan_method(&candidate.plan);
    config.policy_snapshots = vec![PolicySnapshot {
        schema_version: POLICY_SNAPSHOT_V1_SCHEMA.into(),
        policy_version: "policy-approval".into(),
        effective_from: "2026-03-01T00:00:00Z".into(),
        effective_to: None,
        autonomy_ceiling: AutonomyCeiling::PromotionEligible,
        method_rules: vec![MethodPolicy {
            method,
            allowed: true,
            max_autonomy: AutonomyCeiling::PromotionEligible,
            approval_requirement: ApprovalRequirement::HumanReview,
        }],
        blocked_promotions_when_degraded: true,
        blocked_promotions_when_budget_exhausted: true,
        citation: complete_citation(),
        obligation_refs: complete_obligation_refs(),
    }];

    let scope = config.scope.clone();
    let memory_store = open_memory_store(_dir.path());
    let forge_store = open_forge_store(_dir.path());
    import_v3_bundle(
        &memory_store,
        &forge_store,
        &scope.namespace,
        &sample_bundle("pilot-v8-control"),
    )
    .await;
    install_permissive_governance(&memory_store).await;
    let resources = resources(memory_store, forge_store, &config);
    let mut runner = LoopRunner::new(config, resources);
    let report = runner.run().await.unwrap();

    let iteration = &report.iterations[0];
    assert!(
        iteration
            .policy_decision
            .as_ref()
            .unwrap()
            .approval_required
    );
    assert!(
        !iteration
            .policy_decision
            .as_ref()
            .unwrap()
            .approval_satisfied
    );
    assert_eq!(
        iteration.adjudication.as_ref().unwrap().disposition,
        VerificationDisposition::PendingApproval
    );
}

#[tokio::test]
async fn full_loop_accepts_scoped_approval_record() {
    let (_dir, mut config, candidate) = preview_first_candidate().await;
    let method = plan_method(&candidate.plan);
    let policy_version = "policy-approval-granted";
    config.policy_snapshots = vec![PolicySnapshot {
        schema_version: POLICY_SNAPSHOT_V1_SCHEMA.into(),
        policy_version: policy_version.into(),
        effective_from: "2026-03-01T00:00:00Z".into(),
        effective_to: None,
        autonomy_ceiling: AutonomyCeiling::PromotionEligible,
        method_rules: vec![MethodPolicy {
            method,
            allowed: true,
            max_autonomy: AutonomyCeiling::PromotionEligible,
            approval_requirement: ApprovalRequirement::HumanReview,
        }],
        blocked_promotions_when_degraded: true,
        blocked_promotions_when_budget_exhausted: true,
        citation: verification_policy::V25CitationContext::missing(),
        obligation_refs: verification_policy::V25ControlObligationRefs::missing(),
    }];
    config.approval_records = vec![ApprovalRecord::new(
        policy_version,
        None,
        None,
        ApprovalScope {
            namespace: config.scope.namespace.clone(),
            target_key: None,
            method: None,
            promotion_class: None,
            expires_at: None,
            reusable: true,
        },
        "operator",
        "2026-03-12T00:00:00Z",
        vec![candidate.stable_key.clone()],
        "reviewed planned verification action",
    )];

    let scope = config.scope.clone();
    let memory_store = open_memory_store(_dir.path());
    let forge_store = open_forge_store(_dir.path());
    import_v3_bundle(
        &memory_store,
        &forge_store,
        &scope.namespace,
        &sample_bundle("pilot-v8-control"),
    )
    .await;
    install_permissive_governance(&memory_store).await;
    let resources = resources(memory_store, forge_store, &config);
    let mut runner = LoopRunner::new(config, resources);
    let report = runner.run().await.unwrap();

    let iteration = &report.iterations[0];
    assert!(
        iteration
            .policy_decision
            .as_ref()
            .unwrap()
            .approval_satisfied
    );
    assert!(!matches!(
        iteration.verification_attempt.as_ref().unwrap().state,
        VerificationAttemptState::Blocked
    ));
}

#[tokio::test]
async fn full_loop_uncalibrated_path_forces_advisory_only() {
    let dir = tempdir();
    let memory_store = open_memory_store(dir.path());
    let forge_store = open_forge_store(dir.path());
    let scope = Scope::new("pilot-v8-uncalibrated");
    let mut config = base_loop_config(scope.clone());
    point_config_at_dir(&mut config, dir.path());
    write_source_file(
        dir.path(),
        "src/lib.rs",
        "pub fn uncalibrated_fixture() -> bool { true }\n",
    );
    config.calibration_abstention_threshold_micros = 1;

    import_v3_bundle(
        &memory_store,
        &forge_store,
        &scope.namespace,
        &sample_bundle("pilot-v8-uncalibrated"),
    )
    .await;
    install_permissive_governance(&memory_store).await;

    let resources = resources(memory_store, forge_store, &config);
    let mut runner = LoopRunner::new(config, resources);
    let report = runner.run().await.unwrap();

    assert_eq!(
        report.iterations[0]
            .adjudication
            .as_ref()
            .unwrap()
            .disposition,
        VerificationDisposition::AdvisoryOnly
    );
    let artifact = report.iterations[0]
        .verification_plan_artifact
        .as_ref()
        .unwrap();
    assert!(artifact
        .blocked_checks
        .iter()
        .any(|entry| entry.contains("advisory-only")));
    assert_eq!(
        artifact.target_exactness,
        semantic_memory_forge::ExactnessLevelV1::Conservative
    );
    assert!(artifact
        .degradation_flags
        .iter()
        .any(|flag| flag.contains("runtime degraded")));
}

#[tokio::test]
async fn full_loop_emits_v9_plan_and_consumer_only_proof_artifacts() {
    let (_dir, config, _candidate) = preview_first_candidate().await;
    let scope = config.scope.clone();
    let memory_store = open_memory_store(_dir.path());
    let forge_store = open_forge_store(_dir.path());
    import_v3_bundle(
        &memory_store,
        &forge_store,
        &scope.namespace,
        &sample_bundle("pilot-v8-control"),
    )
    .await;
    install_permissive_governance(&memory_store).await;
    let resources = resources(memory_store, forge_store, &config);
    let mut runner = LoopRunner::new(config, resources);
    let report = runner.run().await.unwrap();

    assert!(report.receipt.consumer_only_truth_access);
    let iteration = &report.iterations[0];
    assert!(iteration.verification_plan_artifact.is_some());
    assert!(
        iteration.repair_records.is_empty()
            || iteration
                .repair_records
                .iter()
                .all(|record| record.unchanged_statement.contains("authoritative truth"))
    );
}

#[tokio::test]
async fn loop_report_boundary_parser_repairs_missing_approval_records() {
    let dir = tempdir();
    let memory_store = open_memory_store(dir.path());
    let forge_store = open_forge_store(dir.path());
    let scope = Scope::new("pilot-v8-boundary");
    let mut config = base_loop_config(scope.clone());
    point_config_at_dir(&mut config, dir.path());
    write_source_file(
        dir.path(),
        "src/lib.rs",
        "pub fn boundary_fixture() -> bool { true }\n",
    );

    import_v3_bundle(
        &memory_store,
        &forge_store,
        &scope.namespace,
        &sample_bundle("pilot-v8-boundary"),
    )
    .await;
    install_permissive_governance(&memory_store).await;

    let resources = resources(memory_store, forge_store, &config);
    let mut runner = LoopRunner::new(config, resources);
    let report = runner.run().await.unwrap();

    let mut value = serde_json::to_value(report).unwrap();
    value["iterations"][0]
        .as_object_mut()
        .unwrap()
        .remove("approval_records");

    let repaired = parse_loop_report_boundary(value).unwrap();
    assert!(repaired.iterations[0].approval_records.is_empty());
    assert_eq!(
        repaired.iterations[0].boundary_repairs[0].field_path,
        "$.approval_records"
    );
}

#[tokio::test]
async fn full_loop_refuted_promoted_state_triggers_rollback_invalidation() {
    let dir = tempdir();
    let memory_store = open_memory_store(dir.path());
    let forge_store = open_forge_store(dir.path());
    let scope = Scope::new("pilot-v8-rollback");
    let mut config = base_loop_config(scope.clone());
    point_config_at_dir(&mut config, dir.path());
    write_source_file(
        dir.path(),
        "src/lib.rs",
        "pub fn rollback_fixture() -> bool { true }\n",
    );

    import_promoted_hyperedge_batch(&memory_store, &scope.namespace, "pilot-v8-rollback").await;
    install_permissive_governance(&memory_store).await;

    let resources = resources(memory_store, forge_store, &config);
    let observation = observe_scope(&resources.runtime, &resources.memory_store, &config)
        .await
        .unwrap();
    let candidates = score_targets(&observation, &PilotHistory::default(), &config);
    assert!(candidates
        .iter()
        .any(|candidate| matches!(candidate.plan, PlanKind::OracleCausalRefuter { .. })));

    let mut runner = LoopRunner::new(config, resources);
    let report = runner.run().await.unwrap();
    let iteration = &report.iterations[0];

    assert!(
        matches!(
            iteration.check_plan.as_ref().map(|plan| plan.method),
            Some(verification_control::CheckMethod::CausalRefuter)
        ),
        "expected causal refuter plan, got {:?}",
        iteration.check_plan
    );

    assert_eq!(
        iteration.adjudication.as_ref().unwrap().disposition,
        VerificationDisposition::Refuted
    );
    assert!(
        iteration
            .adjudication
            .as_ref()
            .unwrap()
            .rollback_plan
            .required
    );
    assert!(iteration.rollback_invalidation_count.is_some());
    assert!(iteration.repair_records.iter().any(|record| matches!(
        record.repair_class,
        forge_pilot::RepairClassV1::RollbackRepair
    )));
    assert!(iteration
        .repair_records
        .iter()
        .any(|record| record.blast_radius == "projection_scope"));
}
