//! Receipt and output DTOs for runner material operations.
//!
//! These are AiDENs-local execution receipts; canonical receipt truth remains
//! with `aidens-receipts` and sibling verification crates.

use super::*;

fn material_receipt_id(prefix: &str, material: serde_json::Value) -> ArtifactId {
    generated_artifact_id_from_material(prefix, &material.to_string())
}

pub(super) fn material_bound_run_context(
    app_id: &str,
    prompt: &str,
    provider_route: &ProviderRouteReportV1,
    tool_exposure: &ToolExposureSetV1,
    budget: &BudgetV1,
) -> AidensRunContextV1 {
    let material = serde_json::json!({
        "app_id": app_id,
        "prompt": prompt,
        "provider_kind": &provider_route.provider_kind,
        "provider_model": &provider_route.model,
        "provider_route": &provider_route.route,
        "exposed_tool_ids": &tool_exposure.exposed_tool_ids,
        "blocked_tool_ids": &tool_exposure.blocked_tool_ids,
        "sandbox_root": &tool_exposure.sandbox_root,
        "max_tool_calls": budget.max_tool_calls,
        "max_retries": budget.max_retries,
        "max_turn_millis": budget.max_turn_millis,
    });
    let material = material.to_string();
    let mut context = AidensRunContextV1::new(app_id);
    context.run_id = generated_artifact_id_from_material("run", &material);
    context.trace_id = generated_artifact_id_from_material("trace", &material);
    context.attempt_family_id = generated_artifact_id_from_material("attempt-family", &material);
    context.attempt_id = generated_artifact_id_from_material("attempt", &material);
    context
}

pub(super) fn material_bind_tool_exposure(
    exposure: &mut ToolExposureSetV1,
    context: &AidensRunContextV1,
) {
    for (index, request) in exposure.approval_requests.iter_mut().enumerate() {
        material_bind_approval_request(request, context, index);
    }
    for decision in &mut exposure.decisions {
        if let Some(request) = &mut decision.approval_request {
            if let Some(bound) = exposure
                .approval_requests
                .iter()
                .find(|candidate| candidate.tool_id == request.tool_id)
            {
                *request = bound.clone();
            }
        }
    }
    for (index, receipt) in exposure.permit_use_receipts.iter_mut().enumerate() {
        material_bind_permit_use_receipt(receipt, context, index);
    }
    for decision in &mut exposure.decisions {
        if let Some(old_receipt_id) = &decision.permit_use_receipt_id {
            if let Some(bound) = exposure.permit_use_receipts.iter().find(|receipt| {
                receipt.permit_id
                    == decision
                        .permit_grant_id
                        .clone()
                        .unwrap_or_else(|| receipt.permit_id.clone())
                    || &receipt.receipt_id == old_receipt_id
            }) {
                decision.permit_use_receipt_id = Some(bound.receipt_id.clone());
            }
        }
    }
}

pub(super) fn material_bind_approval_request(
    request: &mut ApprovalRequestV1,
    context: &AidensRunContextV1,
    call_index: usize,
) {
    if request.run_id.as_ref() != Some(&context.run_id) {
        *request = request.clone().for_execution_context(context);
    }
    request.request_id = material_receipt_id(
        "approval-request",
        serde_json::json!({
            "run_id": &context.run_id,
            "attempt_id": &context.attempt_id,
            "call_index": call_index,
            "tool_id": &request.tool_id,
            "risk_class": &request.risk_class,
            "scope": &request.scope,
            "sandbox_root": &request.sandbox_root,
            "reason": &request.reason,
            "reason_codes": &request.reason_codes,
        }),
    );
}

pub(super) fn material_bind_permit_use_receipt(
    receipt: &mut PermitUseReportV1,
    context: &AidensRunContextV1,
    call_index: usize,
) {
    receipt.run_id = Some(context.run_id.clone());
    receipt.attempt_id = Some(context.attempt_id.clone());
    receipt.receipt_id = material_receipt_id(
        "permit-use",
        serde_json::json!({
            "run_id": &context.run_id,
            "attempt_id": &context.attempt_id,
            "call_index": call_index,
            "permit_id": &receipt.permit_id,
            "tool_id": &receipt.tool_id,
            "risk_class": &receipt.risk_class,
            "sandbox_root": &receipt.sandbox_root,
            "allowed": receipt.allowed,
            "reason_codes": &receipt.reason_codes,
        }),
    );
}

pub(super) fn material_bind_tool_invocation_receipt(
    receipt: &mut ToolInvocationReportV1,
    request: &ToolCallRequestV1,
    context: &AidensRunContextV1,
    call_index: usize,
) {
    receipt.receipt_id = material_receipt_id(
        "tool-invocation",
        serde_json::json!({
            "run_id": &context.run_id,
            "attempt_id": &context.attempt_id,
            "call_index": call_index,
            "tool_id": &receipt.tool_id,
            "request_source": &request.source,
            "input_digest": &receipt.input_digest,
            "output_digest": &receipt.output_digest,
            "outcome": &receipt.outcome,
            "permit_grant_id": &receipt.permit_grant_id,
            "permit_use_receipt_id": &receipt.permit_use_receipt_id,
            "approval_request_id": &receipt.approval_request_id,
            "reason_codes": &receipt.reason_codes,
        }),
    );
}

pub(super) fn material_bind_run_report_receipt(report: &mut RunReportV1) {
    report.receipt_id = material_receipt_id(
        "run-report",
        serde_json::json!({
            "run_id": &report.context.run_id,
            "trace_id": &report.context.trace_id,
            "attempt_family_id": &report.context.attempt_family_id,
            "attempt_id": &report.context.attempt_id,
            "app_id": &report.context.app_id,
            "provider_route": &report.provider_route,
            "tool_calls": report.tool_call_requests.iter().map(|request| serde_json::json!({
                "source": request.source,
                "tool_id": request.tool_id,
                "input_digest": request.input_digest,
            })).collect::<Vec<_>>(),
            "tool_invocation_receipts": report.tool_invocation_receipts.iter().map(|receipt| serde_json::json!({
                "receipt_id": receipt.receipt_id,
                "tool_id": receipt.tool_id,
                "input_digest": receipt.input_digest,
                "output_digest": receipt.output_digest,
                "outcome": receipt.outcome,
                "reason_codes": receipt.reason_codes,
            })).collect::<Vec<_>>(),
            "approval_request_ids": report.approval_requests.iter().map(|request| &request.request_id).collect::<Vec<_>>(),
            "permit_use_receipt_ids": report.permit_use_receipts.iter().map(|receipt| &receipt.receipt_id).collect::<Vec<_>>(),
            "turn_outcomes": report.turn_receipts.iter().map(|turn| serde_json::json!({
                "mode": turn.mode,
                "final_state": turn.final_state,
                "degraded": turn.degraded,
                "blocked": turn.blocked,
                "reason_codes": turn.reason_codes,
            })).collect::<Vec<_>>(),
            "warnings": &report.warnings,
        }),
    );
}

#[derive(Debug, Clone)]
pub struct PlanReceiptV1 {
    pub receipt_id: ArtifactId,
    pub plan_id: ArtifactId,
    pub step: u32,
    pub action: String,
    pub reason_codes: Vec<String>,
}

impl PlanReceiptV1 {
    pub(crate) fn material_bound(
        agent_id: &str,
        prompt: &str,
        step: u32,
        action: impl Into<String>,
        reason_codes: Vec<String>,
    ) -> Self {
        let action = action.into();
        let plan_material = serde_json::json!({
            "agent_id": agent_id,
            "prompt": prompt,
            "step": step,
            "action": &action,
        });
        let plan_id = material_receipt_id("agent-plan", plan_material);
        let receipt_id = material_receipt_id(
            "agent-plan-receipt",
            serde_json::json!({
                "plan_id": &plan_id,
                "step": step,
                "action": &action,
                "reason_codes": &reason_codes,
            }),
        );
        Self {
            receipt_id,
            plan_id,
            step,
            action,
            reason_codes,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ToolRouteReceiptV1 {
    pub receipt_id: ArtifactId,
    pub plan_id: ArtifactId,
    pub step: u32,
    pub requested_tool_ids: Vec<String>,
    pub exposed_tool_ids: Vec<String>,
    pub reason_codes: Vec<String>,
}

impl ToolRouteReceiptV1 {
    pub(crate) fn material_bound(
        plan_id: ArtifactId,
        step: u32,
        requested_tool_ids: Vec<String>,
        exposed_tool_ids: Vec<String>,
        reason_codes: Vec<String>,
    ) -> Self {
        let receipt_id = material_receipt_id(
            "tool-route",
            serde_json::json!({
                "plan_id": &plan_id,
                "step": step,
                "requested_tool_ids": &requested_tool_ids,
                "exposed_tool_ids": &exposed_tool_ids,
                "reason_codes": &reason_codes,
            }),
        );
        Self {
            receipt_id,
            plan_id,
            step,
            requested_tool_ids,
            exposed_tool_ids,
            reason_codes,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ToolCallReceiptV1 {
    pub receipt_id: ArtifactId,
    pub plan_id: ArtifactId,
    pub step: u32,
    pub run_id: String,
    pub permitted_tool_calls: usize,
    pub blocked_tool_calls: usize,
    pub succeeded_tool_calls: usize,
    pub failed_tool_calls: usize,
    pub reason_codes: Vec<String>,
}

impl ToolCallReceiptV1 {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn material_bound(
        plan_id: ArtifactId,
        step: u32,
        run_id: String,
        permitted_tool_calls: usize,
        blocked_tool_calls: usize,
        succeeded_tool_calls: usize,
        failed_tool_calls: usize,
        reason_codes: Vec<String>,
    ) -> Self {
        let receipt_id = material_receipt_id(
            "agent-tool-call",
            serde_json::json!({
                "plan_id": &plan_id,
                "step": step,
                "run_id": &run_id,
                "permitted_tool_calls": permitted_tool_calls,
                "blocked_tool_calls": blocked_tool_calls,
                "succeeded_tool_calls": succeeded_tool_calls,
                "failed_tool_calls": failed_tool_calls,
                "reason_codes": &reason_codes,
            }),
        );
        Self {
            receipt_id,
            plan_id,
            step,
            run_id,
            permitted_tool_calls,
            blocked_tool_calls,
            succeeded_tool_calls,
            failed_tool_calls,
            reason_codes,
        }
    }
}

#[derive(Debug, Clone)]
pub struct VerificationReceiptV1 {
    pub receipt_id: ArtifactId,
    pub step: u32,
    pub check: String,
    pub passed: bool,
    pub reason_codes: Vec<String>,
}

impl VerificationReceiptV1 {
    pub(crate) fn material_bound(
        owner_material: &str,
        step: u32,
        check: impl Into<String>,
        passed: bool,
        reason_codes: Vec<String>,
    ) -> Self {
        let check = check.into();
        let receipt_id = material_receipt_id(
            "agent-verification",
            serde_json::json!({
                "owner_material": owner_material,
                "step": step,
                "check": &check,
                "passed": passed,
                "reason_codes": &reason_codes,
            }),
        );
        Self {
            receipt_id,
            step,
            check,
            passed,
            reason_codes,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AbstentionReceiptV1 {
    pub receipt_id: ArtifactId,
    pub step: u32,
    pub reason_code: String,
    pub blocked_action: String,
    pub evidence: Vec<String>,
    pub required_permits: Vec<String>,
    pub can_resume: bool,
    pub support_impact: String,
}

impl AbstentionReceiptV1 {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn material_bound(
        owner_material: &str,
        step: u32,
        reason_code: impl Into<String>,
        blocked_action: impl Into<String>,
        evidence: Vec<String>,
        required_permits: Vec<String>,
        can_resume: bool,
        support_impact: impl Into<String>,
    ) -> Self {
        let reason_code = reason_code.into();
        let blocked_action = blocked_action.into();
        let support_impact = support_impact.into();
        let receipt_id = material_receipt_id(
            "agent-abstention",
            serde_json::json!({
                "owner_material": owner_material,
                "step": step,
                "reason_code": &reason_code,
                "blocked_action": &blocked_action,
                "evidence": &evidence,
                "required_permits": &required_permits,
                "can_resume": can_resume,
                "support_impact": &support_impact,
            }),
        );
        Self {
            receipt_id,
            step,
            reason_code,
            blocked_action,
            evidence,
            required_permits,
            can_resume,
            support_impact,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RepairPlanDisplayReceiptV1 {
    pub repair_id: ArtifactId,
    pub source_run_id: Option<String>,
    pub failure_kind: String,
    pub candidate_repair_actions: Vec<String>,
    pub required_verification: Vec<String>,
    pub required_permits: Vec<String>,
    pub risk_level: String,
    pub canonical_owner: Option<String>,
}

impl RepairPlanDisplayReceiptV1 {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn material_bound(
        owner_material: &str,
        source_run_id: Option<String>,
        failure_kind: impl Into<String>,
        candidate_repair_actions: Vec<String>,
        required_verification: Vec<String>,
        required_permits: Vec<String>,
        risk_level: impl Into<String>,
        canonical_owner: Option<String>,
    ) -> Self {
        let failure_kind = failure_kind.into();
        let risk_level = risk_level.into();
        let repair_id = material_receipt_id(
            "agent-repair",
            serde_json::json!({
                "owner_material": owner_material,
                "source_run_id": &source_run_id,
                "failure_kind": &failure_kind,
                "candidate_repair_actions": &candidate_repair_actions,
                "required_verification": &required_verification,
                "required_permits": &required_permits,
                "risk_level": &risk_level,
                "canonical_owner": &canonical_owner,
            }),
        );
        Self {
            repair_id,
            source_run_id,
            failure_kind,
            candidate_repair_actions,
            required_verification,
            required_permits,
            risk_level,
            canonical_owner,
        }
    }
}

#[derive(Debug, Clone)]
pub struct FinalizationReceiptV1 {
    pub receipt_id: ArtifactId,
    pub step: u32,
    pub outcome: String,
    pub final_state: String,
    pub blocked: bool,
    pub support_label: String,
    pub reason_codes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlanActVerifyOutcomeV1 {
    Success,
    Abstained,
    RepairNeeded,
    Failed,
}

#[derive(Debug, Clone)]
pub struct PlanActVerifyLoopV1Output {
    pub agent_id: String,
    pub app_id: String,
    pub profile: String,
    pub support_label: String,
    pub outcome: PlanActVerifyOutcomeV1,
    pub turns_used: u32,
    pub max_turns: u32,
    pub plan_receipts: Vec<PlanReceiptV1>,
    pub tool_route_receipts: Vec<ToolRouteReceiptV1>,
    pub tool_call_receipts: Vec<ToolCallReceiptV1>,
    pub verification_receipts: Vec<VerificationReceiptV1>,
    pub memory_grounding_receipts: Vec<String>,
    pub abstention_receipt: Option<AbstentionReceiptV1>,
    pub repair_plan: Option<RepairPlanDisplayReceiptV1>,
    pub finalization: Option<FinalizationReceiptV1>,
    pub run_output: Option<AiDENsRunOutput>,
}

impl PlanActVerifyLoopV1Output {
    pub(crate) fn abstention(
        agent: &AgentSpecV1,
        step: u32,
        reason_code: impl Into<String>,
    ) -> Self {
        Self {
            agent_id: agent.agent_id.clone(),
            app_id: "aidens-plan-act-verify-loop".into(),
            profile: agent.profile.clone(),
            support_label: agent.support_label.to_string(),
            outcome: PlanActVerifyOutcomeV1::Abstained,
            turns_used: 0,
            max_turns: agent.budget_policy.max_turns,
            plan_receipts: Vec::new(),
            tool_route_receipts: Vec::new(),
            tool_call_receipts: Vec::new(),
            verification_receipts: Vec::new(),
            memory_grounding_receipts: Vec::new(),
            abstention_receipt: Some(AbstentionReceiptV1::material_bound(
                &agent.agent_id,
                step,
                reason_code,
                "execution-blocked-before-run",
                Vec::new(),
                Vec::new(),
                true,
                "partial",
            )),
            repair_plan: None,
            finalization: None,
            run_output: None,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn assemble_v3_bundle(
        &self,
        identity_material: &str,
        run_id: impl Into<String>,
        profile: impl Into<String>,
        canonical_execution_context: aidens_contracts::canonical_stack::ForgeExecutionContextV1,
        event_log: AiDENsRunEventLogDigestV1,
        budget: AiDENsRunBudgetDeadlineV1,
        support: AiDENsRunSupportTierEvidenceV1,
        support_labels: Vec<String>,
        replay: AiDENsRunReplayNormalizationV1,
        failure: AiDENsRunFailureTaxonomyV1,
        attempt_family_id: ArtifactId,
        attempt_id: StackAttemptId,
        trial_id: StackTrialId,
        agent_spec_digest: DisplayDigestV1,
        provider_receipts: Vec<String>,
        tool_receipts: Vec<String>,
        permit_receipts: Vec<String>,
        memory_grounding_receipts: Vec<String>,
        outputs: Vec<String>,
        replay_instructions: Vec<String>,
        blocked_checks: Vec<String>,
    ) -> Result<AiDENsRunBundleV3, Vec<String>> {
        if identity_material.trim().is_empty() {
            return Err(vec!["bundle-identity-material-required".into()]);
        }
        let mut bundle = AiDENsRunBundleV3::new_projection(
            run_id,
            profile,
            canonical_execution_context,
            event_log,
            budget,
            support,
            support_labels,
            replay,
            failure,
            attempt_family_id,
            attempt_id,
            trial_id,
            agent_spec_digest,
        );
        bundle.provider_receipts = provider_receipts;
        bundle.tool_receipts = tool_receipts;
        bundle.permit_receipts = permit_receipts;
        bundle.memory_grounding_receipts = memory_grounding_receipts;
        bundle.verification_receipts = self
            .verification_receipts
            .iter()
            .map(|receipt| receipt.receipt_id.to_string())
            .collect();
        bundle.abstention_receipts = self
            .abstention_receipt
            .as_ref()
            .map(|receipt| vec![receipt.receipt_id.to_string()])
            .unwrap_or_default();
        bundle.repair_plan_receipts = self
            .repair_plan
            .as_ref()
            .map(|receipt| vec![receipt.repair_id.to_string()])
            .unwrap_or_default();
        bundle.outputs = outputs;
        bundle.replay_instructions = replay_instructions;
        bundle.blocked_checks = blocked_checks;
        bundle.support_labels.sort();
        bundle.support_labels.dedup();
        let bound_material = serde_json::json!({
            "caller_material": identity_material,
            "run_id": &bundle.run_id,
            "profile": &bundle.profile,
            "trace_ctx": &bundle.trace_ctx,
            "attempt_family_id": &bundle.attempt_family_id,
            "attempt_id": &bundle.attempt_id,
            "trial_id": &bundle.trial_id,
            "event_log_replay_normalized_digest": &bundle.event_log.replay_normalized_digest,
            "provider_receipts": &bundle.provider_receipts,
            "tool_receipts": &bundle.tool_receipts,
            "permit_receipts": &bundle.permit_receipts,
            "verification_receipts": &bundle.verification_receipts,
            "abstention_receipts": &bundle.abstention_receipts,
            "repair_plan_receipts": &bundle.repair_plan_receipts,
            "agent_spec_digest": &bundle.agent_spec_digest,
            "support_labels": &bundle.support_labels,
            "failure": &bundle.failure,
            "blocked_checks": &bundle.blocked_checks,
        });
        bundle.bundle_id = material_receipt_id("aidens-run-bundle-v3", bound_material);
        bundle.validate_durable_identity()?;
        Ok(bundle)
    }
}
