mod common;

use common::{
    base_loop_config, import_v3_bundle, open_forge_store, open_memory_store, resources,
    sample_bundle, tempdir,
};
use forge_pilot::{
    canonical_roundtrip, execute_plan, observe_scope, score_targets, ActionFamily, PilotHistory,
};
use knowledge_runtime::Scope;
use verification_control::{
    CaseRegion, CheckMethod, CheckPlan, PromotionClass, ReversibilityClass, VerificationCase,
    VerificationCaseClass,
};
use verification_policy::{evaluate_policy, PolicySnapshot};

fn method_for_plan(plan: &forge_pilot::PlanKind) -> CheckMethod {
    match plan {
        forge_pilot::PlanKind::OracleExactBounded { .. } => CheckMethod::ExactBoundedOracle,
        forge_pilot::PlanKind::OracleConservative => CheckMethod::ConservativeOracle,
        forge_pilot::PlanKind::OracleDeltaParity { .. } => CheckMethod::DeltaParityOracle,
        forge_pilot::PlanKind::OracleTemporalReplay { .. } => CheckMethod::TemporalReplayOracle,
        forge_pilot::PlanKind::OracleCausalRefuter { .. } => CheckMethod::CausalRefuter,
        forge_pilot::PlanKind::OracleMinimalPerturbation { .. } => {
            CheckMethod::MinimalPerturbationOracle
        }
        forge_pilot::PlanKind::PairedPatch { .. } => CheckMethod::PairedPatch,
        forge_pilot::PlanKind::AdvisoryOnlyVerificationPlan(_) => CheckMethod::AdvisoryOnly,
    }
}

#[tokio::test]
async fn oracle_plan_executes_and_roundtrips_through_canonical_v3_lane() {
    let dir = tempdir();
    let memory_store = open_memory_store(dir.path());
    let forge_store = open_forge_store(dir.path());
    let scope = Scope::new("pilot-oracle");
    let config = base_loop_config(scope.clone());

    import_v3_bundle(
        &memory_store,
        &forge_store,
        &scope.namespace,
        &sample_bundle("oracle-1"),
    )
    .await;

    let resources = resources(memory_store, forge_store, &config);
    let observation = observe_scope(&resources.runtime, &resources.memory_store, &config)
        .await
        .unwrap();
    let candidates = score_targets(&observation, &PilotHistory::default(), &config);
    let candidate = candidates
        .into_iter()
        .find(|candidate| !matches!(candidate.plan, forge_pilot::PlanKind::PairedPatch { .. }))
        .unwrap();
    let verification_case = VerificationCase::new(
        VerificationCaseClass::ThinExport,
        CaseRegion {
            namespace: scope.namespace.clone(),
            scope_key: Some(config.scope.key()),
            target_key: candidate.stable_key.clone(),
            region_id: None,
            region_digest_id: None,
            claim_version_id: candidate.target.primary_claim_version_id(),
            as_of_recorded_at: None,
        },
        stack_ids::TraceCtx::generate(),
        stack_ids::AttemptId::generate(),
        "2026-03-24T00:00:00Z",
        false,
        false,
    );
    let check_plan = CheckPlan::new(
        verification_case.case_id.clone(),
        method_for_plan(&candidate.plan),
        candidate.plan.check_names(),
        PromotionClass::P2,
        ReversibilityClass::ReversibleScoped,
        true,
        false,
        false,
        "oracle plan test",
        serde_json::json!({"target_key": candidate.stable_key.clone()}),
    );
    let policy_snapshot = PolicySnapshot::permissive("oracle-plan-test", "2026-03-24T00:00:00Z");
    let policy_decision = evaluate_policy(
        &policy_snapshot,
        &verification_case,
        &check_plan,
        &[],
        false,
        false,
    );
    let permit = policy_decision
        .issue_execution_permit(&verification_case, &check_plan, &[])
        .unwrap();

    let action = execute_plan(
        &observation,
        &candidate.stable_key,
        &candidate.plan,
        &permit,
        &config,
        &resources.forge_store,
    )
    .await
    .unwrap();
    assert_eq!(action.family, ActionFamily::Oracle);

    let execution = action.oracle_execution.as_ref().unwrap();
    let observed_binary = execution
        .assessment
        .as_ref()
        .map(|assessment| assessment.supported)
        .or_else(|| {
            execution
                .delta_parity
                .as_ref()
                .map(|assessment| assessment.parity_match)
        })
        .or_else(|| {
            execution
                .temporal_replay
                .as_ref()
                .map(|assessment| assessment.matched_expected_hash)
        });
    let (expected_support, expected_contradictions) = if let Some(supported) = observed_binary {
        (u64::from(supported), u64::from(!supported))
    } else {
        match &execution.refutation.as_ref().unwrap().outcome {
            kernel_oracles::OracleRefutationOutcome::FlipWitness { .. } => (1, 0),
            kernel_oracles::OracleRefutationOutcome::NoFlipFound { .. } => (0, 1),
            kernel_oracles::OracleRefutationOutcome::NotApplicable { .. } => (0, 0),
        }
    };
    let bundle = action.bundle.as_ref().unwrap();
    let expected_correctness = if expected_support > 0 && expected_contradictions == 0 {
        1.0
    } else {
        0.0
    };
    assert_eq!(bundle.scores.correctness, expected_correctness);
    assert_eq!(bundle.scores.novelty, 0.0);
    assert_eq!(bundle.scores.stability, 0.0);
    assert_eq!(bundle.scores.weighted_total, expected_correctness);
    assert!(bundle.pair_comparability.is_none());
    assert_eq!(bundle.hypotheses[0].support_count, expected_support);
    assert_eq!(
        bundle.hypotheses[0].contradiction_count,
        expected_contradictions
    );
    assert_eq!(
        bundle.hypotheses[0].confidence,
        forge_engine::local_hypothesis_support_confidence(
            expected_support,
            expected_contradictions,
        )
    );

    let roundtrip = canonical_roundtrip(
        action.bundle.as_ref().unwrap(),
        &scope.namespace,
        &resources.forge_store,
        &resources.memory_store,
    )
    .await
    .unwrap();

    assert_eq!(roundtrip.import_result.status, "complete");
}
