use super::*;
use llm_tool_runtime::{
    ToolApprovalState, ToolBackendKind, ToolBudgetContext, ToolPlannerStage, ToolReceipt,
};
use semantic_memory_forge::{
    ForgeToolBudgetContext, ForgeToolReceiptV2, FORGE_TOOL_RECEIPT_V2_SCHEMA,
};
use stack_ids::{ContentDigest, DigestBuilder};

fn digest(value: &str) -> ContentDigest {
    let mut builder = DigestBuilder::new();
    builder.update(value.as_bytes());
    builder.finalize()
}

#[test]
fn tool_receipt_adapter_preserves_lineage() {
    let receipt = ToolReceipt {
        receipt_id: "tool-receipt-1".into(),
        tool_name: "shell".into(),
        tool_version: "1".into(),
        backend_kind: ToolBackendKind::LocalFunction,
        input_digest: digest("in"),
        output_digest_or_refs: json!({"ok": true}),
        policy_hash: digest("policy"),
        approval_state: ToolApprovalState::Approved,
        host_identity: "host".into(),
        started_at: "2026-03-12T00:00:00Z".into(),
        finished_at: "2026-03-12T00:00:01Z".into(),
        trace_ctx: TraceCtx::generate(),
        attempt_id: AttemptId::new("attempt-1"),
        trial_id: TrialId::new("trial-1"),
        planner_stage: ToolPlannerStage::Execution,
        deadline: Some("2026-03-12T00:01:00Z".into()),
        workload_class: Some("verification".into()),
        budget_context: Some(ToolBudgetContext {
            budget_kind: Some("time".into()),
            max_steps: Some(4),
            time_budget_ms: Some(1000),
            cost_budget_units: None,
        }),
        parent_receipt_id: Some("parent-1".into()),
        family_receipt_id: Some("family-1".into()),
        replay_parent_receipt_id: None,
        remote_oracle_lease_id: None,
        remote_slice_result_id: None,
        attestation_envelope_id: None,
        cross_runtime_replay_ticket_id: None,
        error_class: None,
        retry_owner: ToolRetryOwner::ForgeOrchestration,
        replay_link: None,
        tool_run_id: "run-1".into(),
        provider_call_id: None,
        phase: llm_tool_runtime::ToolReceiptPhase::default(),
        resolution: llm_tool_runtime::ToolReceiptResolution::default(),
        authority_lineage: vec![],
        preflight_receipt_id: None,
    };

    let control = ControlReceipt::from(&receipt);
    assert_eq!(control.attempt_id, receipt.attempt_id);
    assert_eq!(control.trial_id, Some(receipt.trial_id));
    assert_eq!(control.source_receipt_id.as_deref(), Some("tool-receipt-1"));
    assert_eq!(
        control.parent_receipt_id.as_ref().map(|id| id.as_str()),
        Some("control-receipt:parent-1")
    );
}

#[test]
fn forge_receipt_adapter_preserves_lineage() {
    let receipt = ForgeToolReceiptV2 {
        schema_version: FORGE_TOOL_RECEIPT_V2_SCHEMA.into(),
        receipt_id: "forge-receipt-1".into(),
        tool_run_id: "run-1".into(),
        tool_name: "shell".into(),
        tool_version: "1".into(),
        backend_kind: "local".into(),
        input_digest: digest("in"),
        output_digest_or_refs: json!({"ok": true}),
        policy_hash: digest("policy"),
        approval_state: "approved".into(),
        host_identity: "host".into(),
        started_at: "2026-03-12T00:00:00Z".into(),
        finished_at: "2026-03-12T00:00:01Z".into(),
        trace_ctx: TraceCtx::generate(),
        attempt_id: AttemptId::new("attempt-1"),
        trial_id: TrialId::new("trial-1"),
        planner_stage: "execution".into(),
        deadline: None,
        workload_class: Some("verification".into()),
        budget_context: Some(ForgeToolBudgetContext {
            budget_kind: Some("time".into()),
            max_steps: Some(2),
            time_budget_ms: Some(500),
            cost_budget_units: None,
        }),
        parent_receipt_id: None,
        family_receipt_id: Some("family-1".into()),
        replay_parent_receipt_id: None,
        error_class: None,
        retry_owner: "forge_orchestration".into(),
        replay_link: None,
        provider_call_id: None,
        raw_payload: json!({}),
    };

    let control = ControlReceipt::from(&receipt);
    assert_eq!(control.attempt_id, receipt.attempt_id);
    assert_eq!(
        control.family_receipt_id.as_ref().map(|id| id.as_str()),
        Some("control-receipt:family-1")
    );
    assert_eq!(
        control.source_receipt_id.as_deref(),
        Some("forge-receipt-1")
    );
}

#[test]
fn canonical_forge_receipt_matches_tool_receipt_lineage_in_control_adapter() {
    let tool_receipt = ToolReceipt {
        receipt_id: "tool-receipt-1".into(),
        tool_name: "shell".into(),
        tool_version: "1".into(),
        backend_kind: ToolBackendKind::LocalFunction,
        input_digest: digest("in"),
        output_digest_or_refs: json!({"ok": true}),
        policy_hash: digest("policy"),
        approval_state: ToolApprovalState::Approved,
        host_identity: "host".into(),
        started_at: "2026-03-12T00:00:00Z".into(),
        finished_at: "2026-03-12T00:00:01Z".into(),
        trace_ctx: TraceCtx::generate(),
        attempt_id: AttemptId::new("attempt-1"),
        trial_id: TrialId::new("trial-1"),
        planner_stage: ToolPlannerStage::Execution,
        deadline: Some("2026-03-12T00:01:00Z".into()),
        workload_class: Some("verification".into()),
        budget_context: Some(ToolBudgetContext {
            budget_kind: Some("time".into()),
            max_steps: Some(4),
            time_budget_ms: Some(1000),
            cost_budget_units: None,
        }),
        parent_receipt_id: Some("parent-1".into()),
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

    let canonical_forge = tool_receipt.to_forge_tool_receipt_v2(json!({
        "lineage_fixture": true,
    }));
    let tool_control = ControlReceipt::from(&tool_receipt);
    let forge_control = ControlReceipt::from(&canonical_forge);

    assert_eq!(tool_control.trace_ctx, forge_control.trace_ctx);
    assert_eq!(tool_control.attempt_id, forge_control.attempt_id);
    assert_eq!(tool_control.trial_id, forge_control.trial_id);
    assert_eq!(
        tool_control
            .parent_receipt_id
            .as_ref()
            .map(|id| id.as_str()),
        forge_control
            .parent_receipt_id
            .as_ref()
            .map(|id| id.as_str())
    );
    assert_eq!(
        tool_control
            .family_receipt_id
            .as_ref()
            .map(|id| id.as_str()),
        forge_control
            .family_receipt_id
            .as_ref()
            .map(|id| id.as_str())
    );
    assert_eq!(
        forge_control.source_receipt_id.as_deref(),
        Some("tool-receipt-1")
    );
    assert_eq!(
        forge_control.source_receipt_kind,
        "semantic_memory_forge.forge_tool_receipt_v2"
    );
    assert_eq!(
        forge_control.retry_owner,
        Some(ControlRetryOwnerV1::ForgeOrchestration)
    );
}

#[test]
fn ledger_replay_rebuilds_closed_case_state() {
    let case = VerificationCase::new(
        VerificationCaseClass::UnverifiedClaimVersion,
        CaseRegion {
            namespace: "demo".into(),
            scope_key: Some(ScopeKey::namespace_only("demo")),
            target_key: "unverified:claim-v1".into(),
            region_id: None,
            region_digest_id: None,
            claim_version_id: Some(ClaimVersionId::new("claim-v1")),
            as_of_recorded_at: None,
        },
        TraceCtx::generate(),
        AttemptId::new("attempt-1"),
        "2026-03-12T00:00:00Z",
        false,
        false,
    );
    let plan = CheckPlan::new(
        case.case_id.clone(),
        CheckMethod::ExactBoundedOracle,
        vec!["kernel_oracle".into()],
        PromotionClass::P2,
        ReversibilityClass::ReversibleScoped,
        true,
        false,
        false,
        "test plan",
        json!({"oracle_slice_id": "slice-1"}),
    );
    let attempt = VerificationAttempt::completed(
        case.case_id.clone(),
        plan.plan_id.clone(),
        case.attempt_id.clone(),
        Some(TrialId::new("trial-1")),
        VerificationAttemptState::Succeeded,
        false,
        false,
        "2026-03-12T00:00:00Z",
        "2026-03-12T00:00:01Z",
        Some("supported".into()),
    );
    let receipt = ControlReceipt::new_case_execution(
        &case,
        &plan,
        &attempt,
        true,
        json!({"family": "oracle"}),
    );
    assert_eq!(
        receipt.citation_status,
        ConstitutionalContextStatus::Missing
    );
    assert_eq!(
        receipt.obligation_refs_status,
        ConstitutionalContextStatus::Missing
    );
    assert!(!receipt.promotable);

    let ledger = vec![
        LedgerEntry::new(
            case.case_id.clone(),
            1,
            LedgerEvent::CaseOpened { case: case.clone() },
        ),
        LedgerEntry::new(
            case.case_id.clone(),
            2,
            LedgerEvent::PlanAdopted { plan: plan.clone() },
        ),
        LedgerEntry::new(
            case.case_id.clone(),
            3,
            LedgerEvent::AttemptRecorded {
                attempt: attempt.clone(),
            },
        ),
        LedgerEntry::new(
            case.case_id.clone(),
            4,
            LedgerEvent::ReceiptAppended {
                receipt: Box::new(receipt.clone()),
            },
        ),
        LedgerEntry::new(
            case.case_id.clone(),
            5,
            LedgerEvent::CaseClosed {
                case_id: case.case_id.clone(),
                disposition: TerminalDisposition::EligibleForPromotion,
            },
        ),
    ];

    let replayed = replay_case(&ledger).unwrap();
    assert_eq!(replayed.plan_count, 1);
    assert_eq!(replayed.attempt_count, 1);
    assert_eq!(replayed.receipt_count, 1);
    assert_eq!(
        replayed.terminal_disposition,
        Some(TerminalDisposition::EligibleForPromotion)
    );
    assert_eq!(
        replayed
            .case
            .as_ref()
            .and_then(|case| case.terminal_disposition.clone()),
        Some(TerminalDisposition::EligibleForPromotion)
    );
}

#[test]
fn scheduler_blocks_promotion_on_degraded_or_exhausted_paths() {
    let case = VerificationCase::new(
        VerificationCaseClass::ThinExport,
        CaseRegion {
            namespace: "demo".into(),
            scope_key: Some(ScopeKey::namespace_only("demo")),
            target_key: "thin_export".into(),
            region_id: None,
            region_digest_id: None,
            claim_version_id: None,
            as_of_recorded_at: None,
        },
        TraceCtx::generate(),
        AttemptId::new("attempt-1"),
        "2026-03-12T00:00:00Z",
        true,
        false,
    );
    let plan = CheckPlan::new(
        case.case_id.clone(),
        CheckMethod::ConservativeOracle,
        vec!["kernel_oracle".into()],
        PromotionClass::P0,
        ReversibilityClass::ReversibleLocal,
        true,
        false,
        true,
        "thin export fallback",
        json!({}),
    );
    let decision = schedule_check_plan(
        &case,
        &plan,
        BudgetLineage {
            budget_family: "orchestration".into(),
            retry_family: case.attempt_id.clone(),
            queue_hop_count: 1,
            max_time_budget_ms: Some(1_000),
            remaining_time_budget_ms: Some(0),
            max_cost_budget_units: None,
            remaining_cost_budget_units: None,
            exhausted: true,
        },
        vec![DegradationMarker {
            kind: "thin_export".into(),
            reason: "missing comparability payload".into(),
            blocks_promotion: true,
        }],
        vec![QueueHop {
            hop_index: 0,
            from_queue: "pilot.observe".into(),
            to_queue: "pilot.verify".into(),
            enqueued_at: "2026-03-12T00:00:00Z".into(),
            dequeued_at: Some("2026-03-12T00:00:01Z".into()),
        }],
    );

    assert!(decision.promotion_blocked);
    assert_eq!(decision.workload_class, VerificationWorkloadClass::Oracle);
}

#[test]
fn scheduler_blocks_promotion_when_proof_obligations_remain() {
    let case = VerificationCase::new(
        VerificationCaseClass::RefutationGap,
        CaseRegion {
            namespace: "demo".into(),
            scope_key: Some(ScopeKey::namespace_only("demo")),
            target_key: "claim:proof-gap".into(),
            region_id: None,
            region_digest_id: None,
            claim_version_id: Some(ClaimVersionId::new("claim-proof-gap-v1")),
            as_of_recorded_at: None,
        },
        TraceCtx::generate(),
        AttemptId::new("attempt-proof"),
        "2026-03-12T00:00:00Z",
        false,
        false,
    );
    let mut plan = CheckPlan::new(
        case.case_id.clone(),
        CheckMethod::TemporalReplayOracle,
        vec!["temporal_replay".into()],
        PromotionClass::P1,
        ReversibilityClass::ReversibleScoped,
        true,
        false,
        false,
        "proof obligations remain",
        json!({}),
    );
    plan.proof_obligations_remaining = vec!["bounded replay certificate missing".into()];

    let decision = schedule_check_plan(
        &case,
        &plan,
        BudgetLineage {
            budget_family: "verification".into(),
            retry_family: case.attempt_id.clone(),
            queue_hop_count: 0,
            max_time_budget_ms: Some(10_000),
            remaining_time_budget_ms: Some(9_500),
            max_cost_budget_units: Some(100),
            remaining_cost_budget_units: Some(90),
            exhausted: false,
        },
        Vec::new(),
        Vec::new(),
    );

    assert!(decision.promotion_blocked);
    let exactness_budget = decision.exactness_budget.expect("exactness budget");
    assert_eq!(exactness_budget.failure_artifact_refs.len(), 1);
    assert!(exactness_budget.failure_artifact_refs[0].contains("proof-obligation"));
}

#[test]
fn boundary_repair_record_is_schema_stable() {
    let record = BoundaryRepairRecord::new(
        BoundaryArtifactKind::LoopIterationReport,
        "loop_iteration_report_v1",
        "default_empty_array",
        "$.approval_records",
        None,
        json!([]),
        "approval records default to empty when absent at the boundary",
    );
    assert_eq!(record.schema_version, BOUNDARY_REPAIR_RECORD_V1_SCHEMA);
    assert_eq!(record.field_path, "$.approval_records");
}

#[test]
fn region_artifact_transport_is_digest_stable() {
    let transport = RegionArtifactTransport::new(
        RegionArtifactKind::Syndrome,
        "artifact:syndrome-1",
        br#"{"syndrome":"constraint_under_supported"}"#,
        RegionId::new("region:a"),
        RegionId::new("region:b"),
        "inference",
        "repair",
        "replay:delta-1",
    );

    assert_eq!(
        transport.schema_version,
        REGION_ARTIFACT_TRANSPORT_V1_SCHEMA
    );
    assert_eq!(transport.from_region_id, RegionId::new("region:a"));
    assert_eq!(transport.to_surface, "repair");
    assert!(!transport.artifact_digest.hex().is_empty());
}

#[test]
fn syndrome_route_record_is_local_and_pre_global() {
    let route = SyndromeRouteRecord::new(
        SyndromeId::new("syndrome:1"),
        RegionId::new("region:a"),
        vec!["claim-a".into(), "claim-b".into()],
        vec!["witness:claim-a".into()],
    );

    assert_eq!(route.schema_version, SYNDROME_ROUTE_RECORD_V1_SCHEMA);
    assert_eq!(route.repair_surface, "compiled_repair_graph");
    assert!(route.routed_before_global_invalidation);
    assert_eq!(route.blast_radius_node_ids.len(), 2);
}

#[test]
fn promotion_interpreter_requires_all_gates() {
    assert!(interpret_promotion_eligibility(
        true, false, false, false, true
    ));
    assert!(!interpret_promotion_eligibility(
        true, true, false, false, true
    ));
    assert!(interpret_rollback_requirement(true, true));
    assert!(!interpret_rollback_requirement(true, false));
}

#[test]
fn v21_v24_review_cases_serialize_cleanly() {
    let effect = EffectReviewCaseV1 {
        schema_version: EFFECT_REVIEW_CASE_V1_SCHEMA.into(),
        effect_review_case_id: stack_ids::EffectReviewCaseId::new("effect-review-case-1"),
        effect_intent_id: stack_ids::EffectIntentId::new("effect-intent-1"),
        effect_preflight_report_id: stack_ids::EffectPreflightReportId::new(
            "effect-preflight-report-1",
        ),
        effect_commit_decision_id: Some(stack_ids::EffectCommitDecisionId::new(
            "effect-commit-decision-1",
        )),
        citation: V25CitationContext {
            applicability_context_id: Some(stack_ids::ApplicabilityContextId::new(
                "applicability-context-1",
            )),
            profile_set_id: Some(stack_ids::ProfileSetId::new("profile-set-1")),
            composition_receipt_id: Some(stack_ids::CompositionReceiptId::new(
                "composition-receipt-1",
            )),
            effective_constitution_id: Some(stack_ids::EffectiveConstitutionId::new(
                "effective-constitution-1",
            )),
            compiled_obligation_set_id: Some(stack_ids::CompiledObligationSetId::new(
                "compiled-obligation-set-1",
            )),
            composition_conflict_set_id: Some(stack_ids::CompositionConflictSetId::new(
                "composition-conflict-set-1",
            )),
            profile_exception_bundle_ids: vec![stack_ids::ProfileExceptionBundleId::new(
                "profile-exception-bundle-1",
            )],
        },
        obligation_refs: V25ControlObligationRefs {
            required_obligation_refs: vec!["obligation:review-policy".into()],
            blocking_obligation_refs: vec!["obligation:blocked:mode".into()],
            monitoring_obligation_refs: vec!["obligation:monitoring:active".into()],
        },
        citation_status: ConstitutionalContextStatus::Complete,
        obligation_refs_status: ConstitutionalContextStatus::Complete,
        required_policy_refs: vec!["policy:effect".into()],
        decision_basis: "typed preflight satisfied".into(),
        final_state: EffectReviewFinalStateV1::Authorized,
        advisory_only: false,
        generated_at: "2026-03-15T12:00:00Z".into(),
    };
    let block = EffectBlockReceiptV1 {
        schema_version: EFFECT_BLOCK_RECEIPT_V1_SCHEMA.into(),
        effect_block_receipt_id: stack_ids::EffectBlockReceiptId::new("effect-block-receipt-1"),
        effect_review_case_id: effect.effect_review_case_id.clone(),
        citation: effect.citation.clone(),
        obligation_refs: effect.obligation_refs.clone(),
        citation_status: ConstitutionalContextStatus::Complete,
        obligation_refs_status: ConstitutionalContextStatus::Complete,
        block_reason: EffectBlockReasonV1::BudgetExhausted,
        generated_at: "2026-03-15T12:01:00Z".into(),
    };
    let delegation = DelegationReviewCaseV1 {
        schema_version: DELEGATION_REVIEW_CASE_V1_SCHEMA.into(),
        delegation_review_case_id: stack_ids::DelegationReviewCaseId::new(
            "delegation-review-case-1",
        ),
        authority_chain_id: stack_ids::AuthorityChainId::new("authority-chain-1"),
        separation_of_duties_policy_id: stack_ids::SeparationOfDutiesPolicyId::new("sod-policy-1"),
        citation: V25CitationContext {
            applicability_context_id: Some(stack_ids::ApplicabilityContextId::new(
                "applicability-context-2",
            )),
            profile_set_id: Some(stack_ids::ProfileSetId::new("profile-set-2")),
            composition_receipt_id: Some(stack_ids::CompositionReceiptId::new(
                "composition-receipt-2",
            )),
            effective_constitution_id: Some(stack_ids::EffectiveConstitutionId::new(
                "effective-constitution-2",
            )),
            compiled_obligation_set_id: Some(stack_ids::CompiledObligationSetId::new(
                "compiled-obligation-set-2",
            )),
            composition_conflict_set_id: None,
            profile_exception_bundle_ids: vec![],
        },
        obligation_refs: V25ControlObligationRefs {
            required_obligation_refs: vec!["obligation:delegation".into()],
            blocking_obligation_refs: vec![],
            monitoring_obligation_refs: vec![],
        },
        citation_status: ConstitutionalContextStatus::Complete,
        obligation_refs_status: ConstitutionalContextStatus::Complete,
        decision_state: DelegationReviewDecisionStateV1::Valid,
        generated_at: "2026-03-15T12:02:00Z".into(),
    };
    let release = ReleaseGateCaseV1 {
        schema_version: RELEASE_GATE_CASE_V1_SCHEMA.into(),
        release_gate_case_id: stack_ids::ReleaseGateCaseId::new("release-gate-case-1"),
        deployment_profile_id: stack_ids::DeploymentProfileId::new("deployment-profile-1"),
        assurance_case_id: stack_ids::AssuranceCaseId::new("assurance-case-1"),
        release_readiness_decision_id: stack_ids::ReleaseReadinessDecisionId::new(
            "release-readiness-decision-1",
        ),
        citation: V25CitationContext {
            applicability_context_id: Some(stack_ids::ApplicabilityContextId::new(
                "applicability-context-3",
            )),
            profile_set_id: Some(stack_ids::ProfileSetId::new("profile-set-3")),
            composition_receipt_id: Some(stack_ids::CompositionReceiptId::new(
                "composition-receipt-3",
            )),
            effective_constitution_id: Some(stack_ids::EffectiveConstitutionId::new(
                "effective-constitution-3",
            )),
            compiled_obligation_set_id: Some(stack_ids::CompiledObligationSetId::new(
                "compiled-obligation-set-3",
            )),
            composition_conflict_set_id: None,
            profile_exception_bundle_ids: vec![],
        },
        obligation_refs: V25ControlObligationRefs {
            required_obligation_refs: vec!["obligation:release".into()],
            blocking_obligation_refs: vec![],
            monitoring_obligation_refs: vec![],
        },
        citation_status: ConstitutionalContextStatus::Complete,
        obligation_refs_status: ConstitutionalContextStatus::Complete,
        score_only_gate: false,
        final_state: ReleaseGateFinalStateV1::ApprovedWithMonitoring,
    };
    let continuity = ContinuityReviewCaseV1 {
        schema_version: CONTINUITY_REVIEW_CASE_V1_SCHEMA.into(),
        continuity_review_case_id: stack_ids::ContinuityReviewCaseId::new(
            "continuity-review-case-1",
        ),
        incident_case_id: stack_ids::IncidentCaseId::new("incident-case-1"),
        continuity_exception_id: Some(stack_ids::ContinuityExceptionId::new(
            "continuity-exception-1",
        )),
        recovery_replay_slice_id: Some(stack_ids::RecoveryReplaySliceId::new(
            "recovery-replay-slice-1",
        )),
        citation: V25CitationContext {
            applicability_context_id: Some(stack_ids::ApplicabilityContextId::new(
                "applicability-context-4",
            )),
            profile_set_id: Some(stack_ids::ProfileSetId::new("profile-set-4")),
            composition_receipt_id: Some(stack_ids::CompositionReceiptId::new(
                "composition-receipt-4",
            )),
            effective_constitution_id: Some(stack_ids::EffectiveConstitutionId::new(
                "effective-constitution-4",
            )),
            compiled_obligation_set_id: Some(stack_ids::CompiledObligationSetId::new(
                "compiled-obligation-set-4",
            )),
            composition_conflict_set_id: None,
            profile_exception_bundle_ids: vec![],
        },
        obligation_refs: V25ControlObligationRefs {
            required_obligation_refs: vec!["obligation:continuity".into()],
            blocking_obligation_refs: vec![],
            monitoring_obligation_refs: vec![],
        },
        citation_status: ConstitutionalContextStatus::Complete,
        obligation_refs_status: ConstitutionalContextStatus::Complete,
        post_hoc_review_due: true,
        final_state: ContinuityReviewFinalStateV1::OpenReview,
    };

    let json = serde_json::to_string(&(effect, block, delegation, release, continuity))
        .expect("serialize review cases");
    let _: (
        EffectReviewCaseV1,
        EffectBlockReceiptV1,
        DelegationReviewCaseV1,
        ReleaseGateCaseV1,
        ContinuityReviewCaseV1,
    ) = serde_json::from_str(&json).expect("deserialize review cases");
}

#[test]
fn effect_review_builder_rejects_advisory_state_mismatch() {
    let result = EffectReviewCaseV1::new(
        stack_ids::EffectIntentId::new("effect-intent-2"),
        stack_ids::EffectPreflightReportId::new("effect-preflight-report-2"),
        None,
        V25CitationContext {
            applicability_context_id: Some(stack_ids::ApplicabilityContextId::new(
                "applicability-context-5",
            )),
            profile_set_id: Some(stack_ids::ProfileSetId::new("profile-set-5")),
            composition_receipt_id: Some(stack_ids::CompositionReceiptId::new(
                "composition-receipt-5",
            )),
            effective_constitution_id: Some(stack_ids::EffectiveConstitutionId::new(
                "effective-constitution-5",
            )),
            compiled_obligation_set_id: Some(stack_ids::CompiledObligationSetId::new(
                "compiled-obligation-set-5",
            )),
            composition_conflict_set_id: None,
            profile_exception_bundle_ids: vec![],
        },
        V25ControlObligationRefs {
            required_obligation_refs: vec!["obligation:effect".into()],
            blocking_obligation_refs: vec![],
            monitoring_obligation_refs: vec![],
        },
        vec!["policy:effect".into()],
        "preflight satisfied",
        EffectReviewFinalStateV1::AdvisoryOnly,
        false,
        "2026-03-15T12:03:00Z",
    );

    assert!(result.is_err());
}

#[test]
fn continuity_review_validation_rejects_closed_case_with_pending_post_hoc_work() {
    let review = ContinuityReviewCaseV1 {
        schema_version: CONTINUITY_REVIEW_CASE_V1_SCHEMA.into(),
        continuity_review_case_id: stack_ids::ContinuityReviewCaseId::new(
            "continuity-review-case-2",
        ),
        incident_case_id: stack_ids::IncidentCaseId::new("incident-case-2"),
        citation: V25CitationContext {
            applicability_context_id: Some(stack_ids::ApplicabilityContextId::new(
                "applicability-context-6",
            )),
            profile_set_id: Some(stack_ids::ProfileSetId::new("profile-set-6")),
            composition_receipt_id: Some(stack_ids::CompositionReceiptId::new(
                "composition-receipt-6",
            )),
            effective_constitution_id: Some(stack_ids::EffectiveConstitutionId::new(
                "effective-constitution-6",
            )),
            compiled_obligation_set_id: Some(stack_ids::CompiledObligationSetId::new(
                "compiled-obligation-set-6",
            )),
            composition_conflict_set_id: None,
            profile_exception_bundle_ids: vec![],
        },
        obligation_refs: V25ControlObligationRefs {
            required_obligation_refs: vec!["obligation:continuity".into()],
            blocking_obligation_refs: vec![],
            monitoring_obligation_refs: vec![],
        },
        citation_status: ConstitutionalContextStatus::Complete,
        obligation_refs_status: ConstitutionalContextStatus::Complete,
        continuity_exception_id: None,
        recovery_replay_slice_id: None,
        post_hoc_review_due: true,
        final_state: ContinuityReviewFinalStateV1::Closed,
    };

    assert!(review.validate().is_err());
}
