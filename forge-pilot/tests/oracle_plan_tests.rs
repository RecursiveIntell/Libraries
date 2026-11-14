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
        candidate
            .plan
            .execution_input(&config.forge_config, &observation)
            .unwrap(),
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

    // A permit for this action must not authorize substituted observed inputs,
    // even if the namespace and selected target string are unchanged.
    let mut substituted = observation.clone();
    substituted.compiled = None;
    let substitution = execute_plan(
        &substituted,
        &candidate.stable_key,
        &candidate.plan,
        &permit,
        &config,
    )
    .await;
    assert!(
        matches!(
            substitution,
            Err(forge_pilot::PilotError::ExecutionBindingMismatch)
        ),
        "observation substitution reached action execution: {substitution:?}"
    );

    let action = execute_plan(
        &observation,
        &candidate.stable_key,
        &candidate.plan,
        &permit,
        &config,
    )
    .await
    .unwrap();
    assert_eq!(action.family, ActionFamily::Oracle);

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

#[tokio::test]
async fn exact_oracle_slice_must_match_the_owner_selected_slice() {
    let dir = tempdir();
    let memory_store = open_memory_store(dir.path());
    let forge_store = open_forge_store(dir.path());
    let scope = Scope::new("pilot-slice-binding");
    let config = base_loop_config(scope.clone());
    import_v3_bundle(
        &memory_store,
        &forge_store,
        &scope.namespace,
        &sample_bundle("slice-binding"),
    )
    .await;
    let resources = resources(memory_store, forge_store, &config);
    let observation = observe_scope(&resources.runtime, &resources.memory_store, &config)
        .await
        .unwrap();
    let selected_id = observation
        .compiled
        .as_ref()
        .unwrap()
        .oracle_candidates
        .first()
        .unwrap()
        .oracle_slice_id
        .clone();

    for mode in ["matching", "wrong", "absent"] {
        let mut input = observation.clone();
        if mode == "absent" {
            input.compiled.as_mut().unwrap().oracle_candidates.clear();
        }
        let requested = if mode == "wrong" {
            stack_ids::OracleSliceId::new("oracle:not-the-owner-selected-slice")
        } else {
            selected_id.clone()
        };
        let plan = forge_pilot::PlanKind::OracleExactBounded {
            oracle_slice_id: requested.clone(),
        };
        let case = VerificationCase::new(
            VerificationCaseClass::ThinExport,
            CaseRegion {
                namespace: scope.namespace.clone(),
                scope_key: Some(config.scope.key()),
                target_key: "slice-binding".into(),
                region_id: None,
                region_digest_id: None,
                claim_version_id: None,
                as_of_recorded_at: None,
            },
            stack_ids::TraceCtx::generate(),
            stack_ids::AttemptId::generate(),
            "2026-03-24T00:00:00Z",
            false,
            false,
        );
        let check = CheckPlan::new(
            case.case_id.clone(),
            CheckMethod::ExactBoundedOracle,
            plan.check_names(),
            PromotionClass::P2,
            ReversibilityClass::ReversibleScoped,
            true,
            false,
            false,
            "exact slice binding witness",
            plan.execution_input(&config.forge_config, &input).unwrap(),
        );
        let policy = PolicySnapshot::permissive("slice-binding-test", "2026-03-24T00:00:00Z");
        let decision = evaluate_policy(&policy, &case, &check, &[], false, false);
        let permit = decision.issue_execution_permit(&case, &check, &[]).unwrap();
        let result = execute_plan(&input, "slice-binding", &plan, &permit, &config).await;
        if mode == "matching" {
            let action = result.unwrap();
            assert_eq!(
                action
                    .oracle_execution
                    .unwrap()
                    .assessment
                    .unwrap()
                    .slice_id,
                requested
            );
        } else {
            assert!(matches!(result, Err(forge_pilot::PilotError::ExecutionBindingMismatch)),
                "{mode} slice must refuse rather than evaluate or bundle another/no slice: {result:?}");
        }
    }
}
