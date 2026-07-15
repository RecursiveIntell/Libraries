use forge_pilot::receipts::{
    build_execution_context, build_lineage_receipt, build_verification_plan_artifact,
};
use forge_pilot::{LoopIterationReport, LOOP_ITERATION_REPORT_V1_SCHEMA};
use llm_tool_runtime::{
    ToolApprovalState, ToolBackendKind, ToolBudgetContext, ToolPlannerStage, ToolReceipt,
    ToolRetryOwner,
};
use semantic_memory_forge::DispatchOutcomeV1;
use serde_json::json;
use stack_ids::{AttemptId, ContentDigest, DigestBuilder, ScopeKey, TraceCtx, TrialId};
use verification_control::{
    BudgetLineage, CaseRegion, CheckMethod, CheckPlan, ControlReceipt, PromotionClass, QueueHop,
    ReversibilityClass, SchedulerDecision, VerificationAttempt, VerificationAttemptState,
    VerificationCase, VerificationCaseClass, VerificationWorkloadClass,
};

fn digest(value: &str) -> ContentDigest {
    let mut builder = DigestBuilder::new();
    builder.update(value.as_bytes());
    builder.finalize()
}

#[test]
fn tool_runtime_to_pilot_to_control_lineage_stays_coherent() {
    let trace_ctx = TraceCtx::generate();
    let attempt_id = AttemptId::new("attempt-exec-1");
    let trial_id = TrialId::new("trial-exec-1");
    let tool_receipt = ToolReceipt {
        receipt_id: "tool-receipt-1".into(),
        tool_name: "repo_scan".into(),
        tool_version: "1.0.0".into(),
        backend_kind: ToolBackendKind::LocalFunction,
        input_digest: digest("in"),
        output_digest_or_refs: json!({"digest": "deadbeef"}),
        policy_hash: digest("policy"),
        approval_state: ToolApprovalState::Approved,
        host_identity: "fixture-host".into(),
        started_at: "2026-03-12T00:00:00Z".into(),
        finished_at: "2026-03-12T00:00:01Z".into(),
        trace_ctx: trace_ctx.clone(),
        attempt_id: attempt_id.clone(),
        trial_id: trial_id.clone(),
        planner_stage: ToolPlannerStage::Execution,
        deadline: Some("2026-03-12T00:01:00Z".into()),
        workload_class: Some("verification".into()),
        budget_context: Some(ToolBudgetContext {
            budget_kind: Some("control_plane".into()),
            max_steps: Some(4),
            time_budget_ms: Some(1500),
            cost_budget_units: Some(8),
        }),
        parent_receipt_id: Some("dispatch-1".into()),
        family_receipt_id: Some("family-1".into()),
        replay_parent_receipt_id: Some("retry-0".into()),
        remote_oracle_lease_id: None,
        remote_slice_result_id: None,
        attestation_envelope_id: None,
        cross_runtime_replay_ticket_id: None,
        error_class: None,
        retry_owner: ToolRetryOwner::ForgeOrchestration,
        replay_link: Some("tool_run:run-1".into()),
        tool_run_id: "run-1".into(),
        provider_call_id: None,
        phase: llm_tool_runtime::ToolReceiptPhase::default(),
        resolution: llm_tool_runtime::ToolReceiptResolution::default(),
        authority_lineage: vec![],
        preflight_receipt_id: None,
    };
    let forge_receipt = tool_receipt.to_forge_tool_receipt_v2(json!({
        "fixture": "execution_evidence_lineage",
    }));
    let control_receipt = ControlReceipt::from(&forge_receipt);

    let case = VerificationCase::new(
        VerificationCaseClass::UnverifiedClaimVersion,
        CaseRegion {
            namespace: "demo".into(),
            scope_key: Some(ScopeKey::namespace_only("demo")),
            target_key: "claim:alpha".into(),
            region_id: None,
            region_digest_id: None,
            claim_version_id: Some(stack_ids::ClaimVersionId::new("claim-alpha-v1")),
            as_of_recorded_at: None,
        },
        trace_ctx.clone(),
        attempt_id.clone(),
        "2026-03-12T00:00:00Z",
        false,
        false,
    );
    let plan = CheckPlan::new(
        case.case_id.clone(),
        CheckMethod::ExactBoundedOracle,
        vec!["temporal_replay".into(), "bounded_oracle_parity".into()],
        PromotionClass::P2,
        ReversibilityClass::ReversibleScoped,
        true,
        false,
        false,
        "oracle",
        json!({}),
    );
    let attempt = VerificationAttempt::completed(
        case.case_id.clone(),
        plan.plan_id.clone(),
        attempt_id.clone(),
        Some(trial_id.clone()),
        VerificationAttemptState::Succeeded,
        false,
        false,
        "2026-03-12T00:00:00Z",
        "2026-03-12T00:00:01Z",
        Some("tool_runtime_receipt_promoted_to_control_receipt".into()),
    );
    let scheduler_decision = SchedulerDecision {
        schema_version: "scheduler_decision_v1".into(),
        case_id: case.case_id.clone(),
        plan_id: plan.plan_id.clone(),
        workload_class: VerificationWorkloadClass::Orchestration,
        cheapest_adequate: true,
        queue_hops: vec![QueueHop {
            hop_index: 0,
            from_queue: "observe".into(),
            to_queue: "verify".into(),
            enqueued_at: "2026-03-12T00:00:00Z".into(),
            dequeued_at: Some("2026-03-12T00:00:00Z".into()),
        }],
        budget_lineage: BudgetLineage {
            budget_family: "standard".into(),
            retry_family: attempt_id.clone(),
            queue_hop_count: 1,
            max_time_budget_ms: Some(1500),
            remaining_time_budget_ms: Some(900),
            max_cost_budget_units: Some(8),
            remaining_cost_budget_units: Some(6),
            exhausted: false,
        },
        exactness_budget: None,
        promotion_blocked: false,
        degradation_markers: Vec::new(),
    };

    let execution_context =
        build_execution_context(&case, &plan, &attempt, &scheduler_decision, false, false);
    let lineage_receipt = build_lineage_receipt(
        &case,
        &plan,
        &attempt,
        "claim:alpha",
        "execution_evidence_converged",
        vec![
            format!("tool_receipt:{}", forge_receipt.receipt_id),
            format!("control_receipt:{}", control_receipt.receipt_id),
        ],
        vec![
            format!("execution_context:{}", execution_context.trace_ctx.trace_id),
            format!("verification_plan:{}", plan.plan_id),
        ],
        Vec::new(),
    );
    let verification_plan_artifact =
        build_verification_plan_artifact(&plan, Vec::new(), Vec::new(), Vec::new());
    let loop_iteration = LoopIterationReport {
        schema_version: LOOP_ITERATION_REPORT_V1_SCHEMA.into(),
        iteration: 1,
        selected_target_key: Some("claim:alpha".into()),
        action_family: None,
        export_completed: false,
        import_completed: false,
        degraded: false,
        advisory_only: false,
        outcome_signature: Some("execution_evidence_converged".into()),
        verification_case: Some(case),
        check_plan: Some(plan.clone()),
        verification_attempt: Some(attempt),
        scheduler_decision: Some(scheduler_decision),
        policy_decision: None,
        approval_records: Vec::new(),
        calibration_snapshot: None,
        adjudication: None,
        control_receipt: Some(control_receipt.clone()),
        target_normalization: None,
        decision_audit: None,
        lineage_receipt: Some(lineage_receipt.clone()),
        export_trace: None,
        stop_rule_evaluation: None,
        verification_plan_artifact: Some(verification_plan_artifact),
        boundary_repairs: Vec::new(),
        repair_records: Vec::new(),
        rollback_invalidation_count: None,
        #[cfg(feature = "governance")]
        governance_receipt: None,
    };

    assert_eq!(forge_receipt.trace_ctx, trace_ctx);
    assert_eq!(forge_receipt.attempt_id, attempt_id);
    assert_eq!(forge_receipt.trial_id, trial_id);
    assert_eq!(forge_receipt.backend_kind, "local_function");
    assert_eq!(control_receipt.trace_ctx, trace_ctx);
    assert_eq!(control_receipt.attempt_id, attempt_id);
    assert_eq!(control_receipt.trial_id, Some(trial_id));
    assert_eq!(
        control_receipt.source_receipt_id.as_deref(),
        Some(forge_receipt.receipt_id.as_str())
    );
    assert_eq!(execution_context.trace_ctx, trace_ctx);
    assert_eq!(execution_context.attempt_id.as_ref(), Some(&attempt_id));
    assert_eq!(
        execution_context.dispatch_outcome,
        DispatchOutcomeV1::Succeeded
    );
    assert!(lineage_receipt
        .consumed_artifact_refs
        .iter()
        .any(|artifact| artifact == "tool_receipt:tool-receipt-1"));
    assert!(lineage_receipt
        .consumed_artifact_refs
        .iter()
        .any(|artifact| artifact.starts_with("control_receipt:")));
    assert_eq!(
        loop_iteration
            .control_receipt
            .as_ref()
            .and_then(|receipt| receipt.source_receipt_id.as_deref()),
        Some("tool-receipt-1")
    );

    let reparsed: LoopIterationReport =
        serde_json::from_str(&serde_json::to_string(&loop_iteration).unwrap()).unwrap();
    assert_eq!(reparsed.schema_version, LOOP_ITERATION_REPORT_V1_SCHEMA);
    assert_eq!(
        reparsed
            .control_receipt
            .as_ref()
            .and_then(|receipt| receipt.source_receipt_id.as_deref()),
        Some("tool-receipt-1")
    );
    assert_eq!(
        reparsed
            .verification_plan_artifact
            .as_ref()
            .map(|artifact| artifact.schema_version.as_str()),
        Some("verification_plan_artifact_v1")
    );
}
