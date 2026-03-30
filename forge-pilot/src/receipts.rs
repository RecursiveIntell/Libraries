use crate::act::ActionFamily;
use crate::types::{
    ExecutionLineageReceipt, ExportActionTraceV1, RepairClassV1, RepairRecordV1,
    StopRuleEvaluation, VerificationPlanArtifact,
};
use semantic_memory_forge::{DispatchOutcomeV1, EvidenceAdmissibilityV1, ExecutionContextV1};
use verification_control::{
    CheapCheckKindV1, CheapCheckStatusV1, CheckPlan, ProofProfileV1, SchedulerDecision,
    VerificationAttempt, VerificationCase,
};

#[allow(clippy::too_many_arguments)]
/// Builds the lineage receipt that ties a loop iteration to its artifact family.
pub fn build_lineage_receipt(
    case: &VerificationCase,
    plan: &CheckPlan,
    attempt: &VerificationAttempt,
    target_key: &str,
    outcome_summary: impl Into<String>,
    consumed_artifact_refs: Vec<String>,
    produced_artifact_refs: Vec<String>,
    degradation_markers: Vec<String>,
) -> ExecutionLineageReceipt {
    ExecutionLineageReceipt {
        case_id: case.case_id.to_string(),
        plan_id: plan.plan_id.to_string(),
        attempt_id: attempt.attempt_id.to_string(),
        target_key: target_key.to_string(),
        execution_context_ref: format!("trace:{}", case.trace_ctx.trace_id),
        consumed_artifact_refs,
        produced_artifact_refs,
        degradation_markers,
        outcome_summary: outcome_summary.into(),
    }
}

/// Builds the export/import trace attached to a loop iteration.
pub fn build_export_trace(
    case: &VerificationCase,
    plan: &CheckPlan,
    attempt: &VerificationAttempt,
    action_family: Option<ActionFamily>,
    export_completed: bool,
    import_completed: bool,
    produced_artifact_refs: Vec<String>,
) -> ExportActionTraceV1 {
    ExportActionTraceV1 {
        case_id: case.case_id.to_string(),
        plan_id: plan.plan_id.to_string(),
        attempt_id: attempt.attempt_id.to_string(),
        action_family: action_family
            .map(|family| format!("{family:?}"))
            .unwrap_or_else(|| "none".into()),
        bridge_roundtrip_completed: export_completed,
        import_completed,
        produced_artifact_refs,
    }
}

/// Builds the stop-rule evaluation artifact for a halted loop iteration.
pub fn build_stop_rule_evaluation(
    halt_reason: impl Into<String>,
    retry_cap_reached: bool,
    cooldown_applied: bool,
    damping_applied: bool,
    degraded: bool,
    advisory_only: bool,
) -> StopRuleEvaluation {
    StopRuleEvaluation {
        halt_reason: halt_reason.into(),
        retry_cap_reached,
        cooldown_applied,
        damping_applied,
        degraded,
        advisory_only,
    }
}

/// Builds the execution-context artifact shared with canonical Forge bundles.
pub fn build_execution_context(
    case: &VerificationCase,
    plan: &CheckPlan,
    attempt: &VerificationAttempt,
    scheduler_decision: &SchedulerDecision,
    degraded: bool,
    advisory_only: bool,
) -> ExecutionContextV1 {
    let mut context = ExecutionContextV1::new(case.trace_ctx.clone());
    context.attempt_id = Some(attempt.attempt_id.clone());
    context.workload_class = Some(format!("{:?}", plan.method));
    context.queue_hops = scheduler_decision
        .queue_hops
        .iter()
        .map(|hop| format!("{}->{}", hop.from_queue, hop.to_queue))
        .collect();
    context.deadline = scheduler_decision
        .budget_lineage
        .remaining_time_budget_ms
        .map(|remaining| format!("remaining_ms:{remaining}"));
    context.cost_budget_units = scheduler_decision
        .budget_lineage
        .remaining_cost_budget_units;
    context.degradation_markers = if case.degraded {
        vec!["verification_case_marked_degraded".into()]
    } else {
        Vec::new()
    };
    if advisory_only || degraded {
        context.dispatch_outcome = DispatchOutcomeV1::Degraded;
    }
    context
}

/// Builds the verification-plan artifact emitted before or during execution.
pub fn build_verification_plan_artifact(
    plan: &CheckPlan,
    blocked_checks: Vec<String>,
    policy_blockers: Vec<String>,
    degradation_flags: Vec<String>,
) -> VerificationPlanArtifact {
    let cheapest_checks = if plan.check_names.is_empty() {
        vec![format!("{:?}", plan.method)]
    } else {
        plan.check_names.clone()
    };
    let cheap_check_ladder = if plan.cheap_check_ladder.is_empty() {
        vec![CheapCheckStatusV1 {
            check: CheapCheckKindV1::SchemaValidation,
            satisfied: true,
            detail: Some(
                "verification plan artifact synthesized default cheap-check ladder".into(),
            ),
        }]
    } else {
        plan.cheap_check_ladder.clone()
    };
    let proof_obligations_remaining = if plan.proof_obligations_remaining.is_empty()
        && (!plan.promotable_if_completed || plan.advisory_only || !policy_blockers.is_empty())
    {
        vec!["promotion path remains incomplete until a promotable proof is recorded".into()]
    } else {
        plan.proof_obligations_remaining.clone()
    };
    let promotion_blocked_on_missing_proof = !plan.promotable_if_completed
        || plan.advisory_only
        || !proof_obligations_remaining.is_empty()
        || !policy_blockers.is_empty();
    VerificationPlanArtifact {
        schema_version: "verification_plan_artifact_v1".into(),
        plan_id: plan.plan_id.to_string(),
        proof_profile: Some(plan.proof_profile.clone().unwrap_or(ProofProfileV1 {
            profile_name: "forge-pilot.default".into(),
            evidence_admissibility: plan.evidence_admissibility.clone(),
            cheap_checks: cheap_check_ladder.clone(),
            proof_obligations_remaining: proof_obligations_remaining.clone(),
            admissible_evidence: default_admissible_evidence(plan.evidence_admissibility.clone()),
        })),
        cheapest_checks,
        cheap_check_ladder,
        replay_recipe: vec![
            format!("replay_case:{}", plan.case_id),
            "rebuild_control_receipt_lineage".into(),
            format!("rerun_method:{:?}", plan.method),
        ],
        replay_preconditions: vec!["control receipt and ledger must remain queryable".into()],
        blocked_checks,
        proof_obligations_remaining,
        admissible_evidence: plan
            .proof_profile
            .as_ref()
            .map(|profile| profile.admissible_evidence.clone())
            .unwrap_or_else(|| default_admissible_evidence(plan.evidence_admissibility.clone())),
        refutation_suggestions: vec!["run minimal falsifier against the selected target".into()],
        degradation_flags,
        policy_blockers,
        promotion_blocked_on_missing_proof,
        target_exactness: plan.target_exactness.clone(),
        expires_at: None,
    }
}

fn default_admissible_evidence(admissibility: EvidenceAdmissibilityV1) -> Vec<String> {
    match admissibility {
        EvidenceAdmissibilityV1::Admissible => {
            vec![
                "control_receipt".into(),
                "replay_lineage".into(),
                "oracle_slice".into(),
            ]
        }
        EvidenceAdmissibilityV1::Restricted => {
            vec!["control_receipt".into(), "advisory_oracle_slice".into()]
        }
        EvidenceAdmissibilityV1::Inadmissible | EvidenceAdmissibilityV1::Unknown => Vec::new(),
    }
}

#[allow(clippy::too_many_arguments)]
/// Builds a repair record emitted for a bounded repair action.
pub fn build_repair_record(
    repair_record_id: String,
    affected_identities: Vec<String>,
    repair_class: RepairClassV1,
    trigger_artifacts: Vec<String>,
    blast_radius: String,
    reversibility: String,
    action: String,
    execution_context: ExecutionContextV1,
) -> RepairRecordV1 {
    RepairRecordV1 {
        schema_version: "repair_record_v1".into(),
        repair_record_id,
        affected_identities,
        repair_class,
        trigger_artifacts,
        blast_radius,
        reversibility,
        action,
        execution_context,
        opened_at: chrono::Utc::now().to_rfc3339(),
        resolved_at: None,
        unchanged_statement: "authoritative truth was not mutated directly by forge-pilot".into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use stack_ids::{AttemptId, ScopeKey, TraceCtx, TrialId};
    use verification_control::{
        BudgetLineage, CaseRegion, CheckMethod, CheckPlan, PromotionClass, QueueHop,
        ReversibilityClass, SchedulerDecision, VerificationAttempt, VerificationAttemptState,
        VerificationCase, VerificationCaseClass,
    };

    fn sample_case() -> VerificationCase {
        VerificationCase::new(
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
            TraceCtx::generate(),
            AttemptId::new("attempt-alpha"),
            "2026-03-12T00:00:00Z",
            false,
            false,
        )
    }

    fn sample_plan(case: &VerificationCase) -> CheckPlan {
        CheckPlan::new(
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
        )
    }

    fn sample_attempt(case: &VerificationCase, plan: &CheckPlan) -> VerificationAttempt {
        VerificationAttempt::completed(
            case.case_id.clone(),
            plan.plan_id.clone(),
            case.attempt_id.clone(),
            Some(TrialId::new("trial-alpha")),
            VerificationAttemptState::Succeeded,
            false,
            false,
            "2026-03-12T00:00:01Z",
            "2026-03-12T00:00:02Z",
            Some("match".into()),
        )
    }

    #[test]
    fn verification_plan_artifact_carries_replay_recipe_and_blockers() {
        let case = sample_case();
        let plan = sample_plan(&case);
        let artifact = build_verification_plan_artifact(
            &plan,
            vec!["ExactOracleSlice: degraded observation blocks exact path".into()],
            vec!["policy denied execution".into()],
            vec!["runtime degraded".into()],
        );

        assert_eq!(artifact.cheapest_checks, plan.check_names);
        assert!(artifact
            .replay_recipe
            .iter()
            .any(|step| step.contains("replay_case")));
        assert!(artifact
            .blocked_checks
            .iter()
            .any(|entry| entry.contains("degraded observation")));
        assert!(artifact.proof_profile.is_some());
        assert!(!artifact.cheap_check_ladder.is_empty());
        assert_eq!(
            artifact.refutation_suggestions,
            vec!["run minimal falsifier against the selected target"]
        );
        assert_eq!(artifact.policy_blockers, vec!["policy denied execution"]);
    }

    #[test]
    fn repair_record_preserves_variant_specific_semantics() {
        let case = sample_case();
        let plan = sample_plan(&case);
        let attempt = sample_attempt(&case, &plan);
        let scheduler = SchedulerDecision {
            schema_version: verification_control::SCHEDULER_DECISION_V1_SCHEMA.into(),
            case_id: case.case_id.clone(),
            plan_id: plan.plan_id.clone(),
            workload_class: verification_control::VerificationWorkloadClass::Oracle,
            cheapest_adequate: true,
            exactness_budget: None,
            queue_hops: vec![QueueHop {
                hop_index: 0,
                from_queue: "pilot".into(),
                to_queue: "verification".into(),
                enqueued_at: "2026-03-12T00:00:00Z".into(),
                dequeued_at: Some("2026-03-12T00:00:01Z".into()),
            }],
            budget_lineage: BudgetLineage {
                budget_family: "verification".into(),
                retry_family: case.attempt_id.clone(),
                queue_hop_count: 1,
                max_time_budget_ms: Some(30_000),
                remaining_time_budget_ms: Some(25_000),
                max_cost_budget_units: Some(100),
                remaining_cost_budget_units: Some(75),
                exhausted: false,
            },
            degradation_markers: Vec::new(),
            promotion_blocked: false,
        };
        let execution_context =
            build_execution_context(&case, &plan, &attempt, &scheduler, false, false);

        let variants = vec![
            (
                RepairClassV1::IdentityRepair,
                "identity_local".to_string(),
                "reversible_local".to_string(),
            ),
            (
                RepairClassV1::TemporalRepair,
                "time_slice".to_string(),
                "replayable".to_string(),
            ),
            (
                RepairClassV1::RollbackRepair,
                "projection_scope".to_string(),
                "requires_recompute".to_string(),
            ),
            (
                RepairClassV1::VerificationStateRepair,
                "verification_state".to_string(),
                "replayable".to_string(),
            ),
        ];

        for (class, blast_radius, reversibility) in variants {
            let record = build_repair_record(
                format!("repair:{class:?}"),
                vec!["claim:alpha".into()],
                class.clone(),
                vec!["receipt:alpha".into()],
                blast_radius.clone(),
                reversibility.clone(),
                "recorded bounded repair action".into(),
                execution_context.clone(),
            );

            assert_eq!(record.repair_class, class);
            assert_eq!(record.blast_radius, blast_radius);
            assert_eq!(record.reversibility, reversibility);
            assert!(record
                .unchanged_statement
                .contains("authoritative truth was not mutated directly"));
        }
    }
}
