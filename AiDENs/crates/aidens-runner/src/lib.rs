//! Reusable run/turn executor. Coordinates but does not own domain truth.

use aidens_agency_kit::{
    AgencyPolicyEngineV1, AgencyPolicyInputV1, AgencyPolicyOutcomeV1, AgencyPolicyReportV1,
    NudgeLedgerV1,
};
use aidens_boundary_kit::{parse_json_boundary, BoundaryRepairPolicyV1};
use aidens_budget_kit::BudgetV1;
use aidens_contracts::{
    display_only_unstable_id, AgentMemoryModeV1, AgentProviderModeV1, AgentSpecSupportLabelV1,
    AgentSpecV1, AgentVerificationCheckV1, AiDENsRunBudgetDeadlineV1, AiDENsRunBundleV3,
    AiDENsRunEventLogDigestV1, AiDENsRunFailureTaxonomyV1, AiDENsRunReplayNormalizationV1,
    AiDENsRunSupportTierEvidenceV1, AidensRunContextV1, ArtifactId, BoundaryRepairReportV1,
    BudgetExhaustionReportDraftV1, BudgetExhaustionReportV1, DegradationEventV1, DisplayDigestV1,
    JsonBoundaryRepairDisplayReportV1, ProjectionDigestV1, ProviderRouteKindV1,
    ProviderRouteReportV1, QueryWideningReportV1, ReportLevelV1, RunReportV1, RuntimeViewRequestV1,
    StackAttemptId, StackTrialId, StopRuleReportV1, StopRuleV1, ToolCallRequestV1,
    ToolCallResultV1, ToolCallSourceV1, ToolExposureSetV1, ToolInvocationReportV1,
    TurnExecutionPlanV1, TurnFinalStateV1, TurnModeV1, TurnReportV1, ViewDisclosureReportV1,
};
use aidens_governance_kit::{canonical_stack as governance_stack, CanonicalGovernanceAdapter};
use aidens_memory_kit::{
    canonical_stack, memory_config_for_root, runtime_config_for_namespace, CanonicalMemoryAdapter,
    MemoryGroundingEvidenceV1,
};
use aidens_permit_kit::PermitPolicyV1;
use aidens_provider_kit::{
    build_provider, route_receipt, AiDENsChatMessageV1, AiDENsCompletionRequestV1,
    AiDENsCompletionResponseV1, AiDENsProvider, ProviderSpecV1,
};
use aidens_receipts::{CanonicalEventLog, CanonicalEventLogConfig, CanonicalEventLogEntry};
use aidens_tool_kit::{
    safe_coding_registry_for_current_dir, ToolDispatcher, ToolExposurePolicyV1,
    ToolInvocationError, ToolRegistryV1,
};
use anyhow::anyhow;
use std::collections::BTreeSet;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Instant;
use std::time::{SystemTime, UNIX_EPOCH};

mod execution;
mod finalization;
mod provider_tool;
mod receipts;
mod replay;

use execution::*;
use finalization::*;
use provider_tool::*;
pub use receipts::*;
use replay::*;

#[derive(Debug, Clone)]
pub struct PlanActVerifyLoopV1 {
    app_id: String,
    spec: AgentSpecV1,
    tools: ToolRegistryV1,
    permit_policy: PermitPolicyV1,
    provider_mock_response: Option<String>,
    canonical_receipt_log_config: Option<CanonicalEventLogConfig>,
    budget_retries: u32,
    run_reports: RunReportLedger,
    receipt_level: ReportLevelV1,
    agency_policy: AgencyPolicyEngineV1,
    agency_nudges: Arc<Mutex<NudgeLedgerV1>>,
}

#[derive(Debug, Clone)]
pub struct AiDENsRunInput {
    pub prompt: String,
}

impl AiDENsRunInput {
    pub fn new(prompt: impl Into<String>) -> Self {
        Self {
            prompt: prompt.into(),
        }
    }
}

impl PlanActVerifyLoopV1 {
    pub fn new(spec: AgentSpecV1) -> Self {
        Self {
            app_id: "aidens-plan-act-loop".into(),
            spec,
            tools: safe_coding_registry_for_current_dir(),
            permit_policy: PermitPolicyV1::default(),
            provider_mock_response: None,
            canonical_receipt_log_config: Some(default_plan_act_verify_receipt_log_config()),
            budget_retries: 2,
            run_reports: RunReportLedger::default(),
            receipt_level: ReportLevelV1::Full,
            agency_policy: AgencyPolicyEngineV1::default(),
            agency_nudges: Arc::new(Mutex::new(NudgeLedgerV1::default())),
        }
    }

    pub fn app_id(mut self, app_id: impl Into<String>) -> Self {
        self.app_id = app_id.into();
        self
    }

    pub fn tools(mut self, tools: ToolRegistryV1) -> Self {
        self.tools = tools;
        self
    }

    pub fn permit_policy(mut self, permit_policy: PermitPolicyV1) -> Self {
        self.permit_policy = permit_policy;
        self
    }

    pub fn provider_mock_response(mut self, response: impl Into<String>) -> Self {
        self.provider_mock_response = Some(response.into());
        self
    }

    pub fn canonical_receipt_log_config(mut self, config: CanonicalEventLogConfig) -> Self {
        self.canonical_receipt_log_config = Some(config);
        self
    }

    pub fn with_agency_policy(mut self, policy: AgencyPolicyEngineV1) -> Self {
        self.agency_policy = policy;
        self
    }

    pub fn with_run_reports(mut self, reports: RunReportLedger) -> Self {
        self.run_reports = reports;
        self
    }

    pub fn max_retries(mut self, retries: u32) -> Self {
        self.budget_retries = retries;
        self
    }

    pub async fn execute(
        &self,
        input: impl Into<String>,
    ) -> anyhow::Result<PlanActVerifyLoopV1Output> {
        let prompt = input.into();
        if let Err(reason_codes) = self.spec.validate() {
            let mut output = PlanActVerifyLoopV1Output::abstention(
                &self.spec,
                1,
                "agent-spec-validation-failed",
            );
            output.verification_receipts.push(VerificationReceiptV1 {
                receipt_id: display_only_unstable_id("agent-verification"),
                step: 1,
                check: "agent-spec".into(),
                passed: false,
                reason_codes,
            });
            return Ok(output);
        }

        let mut memory_grounding_receipts = Vec::new();
        if self.spec.memory_policy.enabled
            && self.spec.memory_policy.mode == AgentMemoryModeV1::CanonicalSeam
        {
            match run_canonical_memory_grounding(&self.spec, &prompt).await {
                Ok(receipts) => {
                    memory_grounding_receipts = receipts;
                }
                Err(error) => {
                    let mut output = PlanActVerifyLoopV1Output::abstention(
                        &self.spec,
                        1,
                        "memory-grounding-failed",
                    );
                    output.memory_grounding_receipts = vec![error.to_string()];
                    output.abstention_receipt = Some(AbstentionReceiptV1 {
                        receipt_id: display_only_unstable_id("agent-abstention"),
                        step: 1,
                        reason_code: "memory-grounding-failed".into(),
                        blocked_action: "memory-grounding-query".into(),
                        evidence: vec![error.to_string()],
                        required_permits: Vec::new(),
                        can_resume: true,
                        support_impact: "degraded".into(),
                    });
                    output.finalization = Some(FinalizationReceiptV1 {
                        receipt_id: display_only_unstable_id("agent-finalization"),
                        step: 1,
                        outcome: "abstained-memory-grounding".into(),
                        final_state: "memory-grounding-error".into(),
                        blocked: true,
                        support_label: self.spec.support_label.to_string(),
                        reason_codes: vec!["memory-grounding-failed".into()],
                    });
                    output.repair_plan = Some(RepairPlanDisplayReceiptV1 {
                        repair_id: display_only_unstable_id("agent-repair"),
                        source_run_id: None,
                        failure_kind: "memory-grounding".into(),
                        candidate_repair_actions: vec![
                            "re-check memory seam fixture availability".into(),
                            "adjust query for grounded scope".into(),
                        ],
                        required_verification: vec!["memory-grounding".into()],
                        required_permits: Vec::new(),
                        risk_level: "moderate".into(),
                        canonical_owner: Some("knowledge-runtime".into()),
                    });
                    return Ok(output);
                }
            }
        }

        let mapped_tools = map_agent_tools_to_ids(&self.spec.tool_policy.allowed_tools);
        if !mapped_tools.unsupported.is_empty() {
            let mut output = PlanActVerifyLoopV1Output::abstention(
                &self.spec,
                1,
                "tool-policy-contains-unsupported-alias",
            );
            output.memory_grounding_receipts = memory_grounding_receipts;
            output.verification_receipts.push(VerificationReceiptV1 {
                receipt_id: display_only_unstable_id("agent-verification"),
                step: 1,
                check: "tool-policy".into(),
                passed: false,
                reason_codes: mapped_tools
                    .unsupported
                    .iter()
                    .map(|alias| format!("unsupported-tool-alias:{alias}"))
                    .collect(),
            });
            output.repair_plan = Some(RepairPlanDisplayReceiptV1 {
                repair_id: display_only_unstable_id("agent-repair"),
                source_run_id: None,
                failure_kind: "unsupported-tool-alias".into(),
                candidate_repair_actions: vec![
                    "remove unsupported tool aliases from allowed tool list".into(),
                    "replace run.replay requests with a supported alias".into(),
                ],
                required_verification: vec!["tool-policy".into()],
                required_permits: Vec::new(),
                risk_level: "low".into(),
                canonical_owner: Some("aidens-tool-kit".into()),
            });
            output.finalization = Some(FinalizationReceiptV1 {
                receipt_id: display_only_unstable_id("agent-finalization"),
                step: 1,
                outcome: "abstained-unsupported-tools".into(),
                final_state: "tool-policy-check".into(),
                blocked: true,
                support_label: self.spec.support_label.to_string(),
                reason_codes: vec!["tool-policy-contains-unsupported-alias".into()],
            });
            return Ok(output);
        }

        let plan = PlanReceiptV1 {
            receipt_id: display_only_unstable_id("agent-plan"),
            plan_id: display_only_unstable_id("agent-plan"),
            step: 1,
            action: "bounded plan-act-verify".into(),
            reason_codes: vec!["agent-spec-valid".into(), "tool-policy-authorized".into()],
        };
        let local_policy_uses_mock_fixture =
            self.spec.provider_policy.provider == AgentProviderModeV1::Local;
        let route_receipt = ToolRouteReceiptV1 {
            receipt_id: display_only_unstable_id("tool-route"),
            plan_id: plan.plan_id.clone(),
            step: 1,
            requested_tool_ids: mapped_tools.canonical.clone(),
            exposed_tool_ids: mapped_tools.canonical.clone(),
            reason_codes: if local_policy_uses_mock_fixture {
                vec!["provider-policy-local-routed-to-explicit-mock-fixture".into()]
            } else {
                Vec::new()
            },
        };

        let provider_spec = ProviderSpecV1 {
            kind: match self.spec.provider_policy.provider {
                AgentProviderModeV1::Mock | AgentProviderModeV1::Local => "mock".into(),
            },
            model: None,
            api_key: None,
            base_url: None,
            mock_response: self.provider_mock_response.clone(),
        };
        if provider_spec.mock_response.as_deref().is_none() {
            let mut output = PlanActVerifyLoopV1Output::abstention(
                &self.spec,
                1,
                "provider-mock-response-missing",
            );
            output.memory_grounding_receipts = memory_grounding_receipts;
            output.repair_plan = Some(RepairPlanDisplayReceiptV1 {
                repair_id: display_only_unstable_id("agent-repair"),
                source_run_id: None,
                failure_kind: "provider-config".into(),
                candidate_repair_actions: vec![
                    "configure a local provider mock response before execution".into(),
                    "validate local provider route and retry".into(),
                ],
                required_verification: vec!["provider-routing".into()],
                required_permits: Vec::new(),
                risk_level: "high".into(),
                canonical_owner: Some("aidens-provider-kit".into()),
            });
            output.finalization = Some(FinalizationReceiptV1 {
                receipt_id: display_only_unstable_id("agent-finalization"),
                step: 1,
                outcome: "abstained-setup".into(),
                final_state: "provider-mock-missing".into(),
                blocked: true,
                support_label: self.spec.support_label.to_string(),
                reason_codes: vec!["provider-mock-response-missing".into()],
            });
            return Ok(output);
        }

        let runner = AiDENsRunner::builder()
            .app_id(self.app_id.clone())
            .provider_spec(provider_spec)
            .tools(self.tools.clone())
            .permit_policy(self.permit_policy.clone())
            .budget(BudgetV1 {
                max_tool_calls: self.spec.budget_policy.max_tool_calls,
                max_retries: self.budget_retries,
                max_turn_millis: self
                    .spec
                    .budget_policy
                    .deadline_seconds
                    .saturating_mul(1000),
            })
            .run_reports(self.run_reports.clone())
            .receipt_level(self.receipt_level.clone())
            .agency_policy(self.agency_policy.clone())
            .agency_nudge_ledger(self.agency_nudges.clone())
            .canonical_receipt_log_config(
                self.canonical_receipt_log_config
                    .clone()
                    .unwrap_or_else(default_plan_act_verify_receipt_log_config),
            )
            .build()?;
        let mut policy = ToolExposurePolicyV1::coding_agent_default();
        if let Some(root) = self.tools.sandbox_root() {
            policy = policy.with_sandbox_root(root.to_string_lossy().to_string());
        }
        policy.allowed_tool_ids = Some(mapped_tools.canonical.iter().cloned().collect());

        let mut output = PlanActVerifyLoopV1Output {
            agent_id: self.spec.agent_id.clone(),
            app_id: self.app_id.clone(),
            profile: self.spec.profile.clone(),
            support_label: self.spec.support_label.to_string(),
            outcome: PlanActVerifyOutcomeV1::Failed,
            turns_used: 0,
            max_turns: self.spec.budget_policy.max_turns,
            plan_receipts: vec![plan],
            tool_route_receipts: vec![route_receipt],
            tool_call_receipts: Vec::new(),
            verification_receipts: Vec::new(),
            memory_grounding_receipts,
            abstention_receipt: None,
            repair_plan: None,
            finalization: None,
            run_output: None,
        };

        let max_turns = self.spec.budget_policy.max_turns.max(1);
        for step in 1..=max_turns {
            output.turns_used = step;
            let turn_output = match runner
                .run_with_tool_policy(AiDENsRunInput::new(prompt.clone()), policy.clone())
                .await
            {
                Ok(run_output) => run_output,
                Err(error) => {
                    output.outcome = PlanActVerifyOutcomeV1::Abstained;
                    output.abstention_receipt = Some(AbstentionReceiptV1 {
                        receipt_id: display_only_unstable_id("agent-abstention"),
                        step,
                        reason_code: format!("runner-execution-error:{error}"),
                        blocked_action: "provider/completion".into(),
                        evidence: vec![error.to_string()],
                        required_permits: Vec::new(),
                        can_resume: false,
                        support_impact: "degraded".into(),
                    });
                    output.repair_plan = Some(RepairPlanDisplayReceiptV1 {
                        repair_id: display_only_unstable_id("agent-repair"),
                        source_run_id: None,
                        failure_kind: "runner-execution".into(),
                        candidate_repair_actions: vec![
                            "collect provider and tool-chain evidence for the error".into(),
                            "retry with corrected local provider configuration".into(),
                        ],
                        required_verification: vec!["runner".into()],
                        required_permits: Vec::new(),
                        risk_level: "high".into(),
                        canonical_owner: Some("aidens-provider-kit".into()),
                    });
                    output.finalization = Some(FinalizationReceiptV1 {
                        receipt_id: display_only_unstable_id("agent-finalization"),
                        step,
                        outcome: "failed-runner-error".into(),
                        final_state: "failed".into(),
                        blocked: true,
                        support_label: self.spec.support_label.to_string(),
                        reason_codes: vec![error.to_string()],
                    });
                    return Ok(output);
                }
            };
            let plan_id = output
                .plan_receipts
                .first()
                .map(|p| p.plan_id.clone())
                .unwrap_or_else(|| display_only_unstable_id("agent-plan"));
            output.run_output = Some(turn_output.clone());
            output.tool_call_receipts.push(ToolCallReceiptV1 {
                receipt_id: display_only_unstable_id("agent-tool-call"),
                plan_id,
                step,
                run_id: turn_output.receipt.context.run_id.to_string(),
                permitted_tool_calls: turn_output.receipt.permit_use_receipts.len(),
                blocked_tool_calls: turn_output.receipt.approval_requests.len(),
                succeeded_tool_calls: turn_output
                    .receipt
                    .tool_invocation_receipts
                    .iter()
                    .filter(|receipt| receipt.succeeded)
                    .count(),
                failed_tool_calls: turn_output
                    .receipt
                    .tool_invocation_receipts
                    .iter()
                    .filter(|receipt| !receipt.succeeded)
                    .count(),
                reason_codes: turn_output
                    .receipt
                    .tool_invocation_receipts
                    .iter()
                    .flat_map(|receipt| receipt.reason_codes.iter().cloned())
                    .collect(),
            });

            output
                .verification_receipts
                .extend(verification_checks_for_loop(
                    &self.spec,
                    &turn_output,
                    &mapped_tools.canonical,
                ));
            let failed_checks = output
                .verification_receipts
                .iter()
                .any(|receipt| !receipt.passed);

            if failed_checks && self.spec.verification_policy.fail_closed {
                output.outcome = PlanActVerifyOutcomeV1::Abstained;
                let failed_checks = failed_verification_checks(&output.verification_receipts);
                output.abstention_receipt = Some(AbstentionReceiptV1 {
                    receipt_id: display_only_unstable_id("agent-abstention"),
                    step,
                    reason_code: "verification-failed".into(),
                    blocked_action: "execution".into(),
                    evidence: vec![turn_output.receipt.receipt_id.to_string()],
                    required_permits: Vec::new(),
                    can_resume: false,
                    support_impact: "degraded".into(),
                });
                output.repair_plan = Some(RepairPlanDisplayReceiptV1 {
                    repair_id: display_only_unstable_id("agent-repair"),
                    source_run_id: Some(turn_output.receipt.context.run_id.to_string()),
                    failure_kind: "verification-failed".into(),
                    candidate_repair_actions: vec![
                        "re-run with fixed verification-policy inputs".into(),
                        "remove or correct unsupported tool-call evidence".into(),
                    ],
                    required_verification: failed_checks,
                    required_permits: Vec::new(),
                    risk_level: "moderate".into(),
                    canonical_owner: Some("aidens-verification-kit".into()),
                });
                output.finalization = Some(FinalizationReceiptV1 {
                    receipt_id: display_only_unstable_id("agent-finalization"),
                    step,
                    outcome: "abstained-verification".into(),
                    final_state: format!("{:?}", turn_output.turn_receipt.final_state),
                    blocked: true,
                    support_label: self.spec.support_label.to_string(),
                    reason_codes: vec!["verification-failed".into()],
                });
                return Ok(output);
            }

            if turn_output.turn_receipt.final_state == TurnFinalStateV1::FinalOutput {
                output.outcome = PlanActVerifyOutcomeV1::Success;
                output.finalization = Some(FinalizationReceiptV1 {
                    receipt_id: display_only_unstable_id("agent-finalization"),
                    step,
                    outcome: "success".into(),
                    final_state: "FinalOutput".into(),
                    blocked: false,
                    support_label: self.spec.support_label.to_string(),
                    reason_codes: if local_policy_uses_mock_fixture {
                        vec!["provider-policy-local-routed-to-explicit-mock-fixture".into()]
                    } else {
                        Vec::new()
                    },
                });
                return Ok(output);
            }

            if turn_output.turn_receipt.blocked || turn_output.turn_receipt.degraded {
                output.outcome = PlanActVerifyOutcomeV1::Abstained;
                let stop_reasons = turn_output_block_reason_codes(&turn_output);
                let blocked_action = blocked_turn_action(turn_output.turn_receipt.final_state);
                output.abstention_receipt = Some(AbstentionReceiptV1 {
                    receipt_id: display_only_unstable_id("agent-abstention"),
                    step,
                    reason_code: format!("turn-blocked:{}", stop_reasons.join("|")),
                    blocked_action: blocked_action.into(),
                    evidence: {
                        let mut evidence = vec![turn_output.receipt.receipt_id.to_string()];
                        evidence.extend(stop_reasons);
                        evidence
                    },
                    required_permits: turn_output
                        .receipt
                        .approval_requests
                        .iter()
                        .map(|request| request.tool_id.clone())
                        .collect(),
                    can_resume: !turn_output.receipt.approval_requests.is_empty(),
                    support_impact: "deferred".into(),
                });
                output.repair_plan = Some(RepairPlanDisplayReceiptV1 {
                    repair_id: display_only_unstable_id("agent-repair"),
                    source_run_id: Some(turn_output.receipt.context.run_id.to_string()),
                    failure_kind: "blocked".into(),
                    candidate_repair_actions: vec![
                        "request permits for blocked tools".into(),
                        "inspect stop rule codes and adjust tool authority".into(),
                        "tighten tool policy to explicit writable tools".into(),
                    ],
                    required_verification: turn_output
                        .receipt
                        .stop_rule_receipts
                        .iter()
                        .flat_map(|receipt| receipt.reason_codes.iter().cloned())
                        .collect(),
                    required_permits: turn_output
                        .receipt
                        .approval_requests
                        .iter()
                        .map(|request| request.tool_id.clone())
                        .collect(),
                    risk_level: "moderate".into(),
                    canonical_owner: Some("aidens-tool-kit".into()),
                });
                output.finalization = Some(FinalizationReceiptV1 {
                    receipt_id: display_only_unstable_id("agent-finalization"),
                    step,
                    outcome: "abstained-blocked".into(),
                    final_state: format!("{:?}", turn_output.turn_receipt.final_state),
                    blocked: true,
                    support_label: self.spec.support_label.to_string(),
                    reason_codes: vec!["turn-blocked".into()],
                });
                return Ok(output);
            }

            if step >= max_turns {
                output.outcome = PlanActVerifyOutcomeV1::Abstained;
                output.abstention_receipt = Some(AbstentionReceiptV1 {
                    receipt_id: display_only_unstable_id("agent-abstention"),
                    step,
                    reason_code: "max-turns-exhausted".into(),
                    blocked_action: "repeat".into(),
                    evidence: vec![turn_output.receipt.receipt_id.to_string()],
                    required_permits: Vec::new(),
                    can_resume: true,
                    support_impact: "degraded".into(),
                });
                output.finalization = Some(FinalizationReceiptV1 {
                    receipt_id: display_only_unstable_id("agent-finalization"),
                    step,
                    outcome: "abstained-bounds-exhausted".into(),
                    final_state: format!("{:?}", turn_output.turn_receipt.final_state),
                    blocked: true,
                    support_label: self.spec.support_label.to_string(),
                    reason_codes: vec!["max-turns-exhausted".into()],
                });
                return Ok(output);
            }
        }

        output.outcome = PlanActVerifyOutcomeV1::RepairNeeded;
        output.finalization = Some(FinalizationReceiptV1 {
            receipt_id: display_only_unstable_id("agent-finalization"),
            step: output.turns_used,
            outcome: "repair-needed".into(),
            final_state: "repair".into(),
            blocked: true,
            support_label: self.spec.support_label.to_string(),
            reason_codes: vec!["loop-exited-without-success".into()],
        });
        Ok(output)
    }
}

#[derive(Debug, Clone)]
pub struct AiDENsRunOutput {
    pub text: String,
    pub receipt: RunReportV1,
    pub turn_receipt: TurnReportV1,
    pub tool_exposure: ToolExposureSetV1,
    pub agency_policy_reports: Vec<AgencyPolicyReportV1>,
    pub durable_receipt_records: Vec<CanonicalEventLogEntry>,
}

#[derive(Debug, Clone, Default)]
pub struct RunReportLedger {
    reports: Arc<Mutex<Vec<RunReportV1>>>,
}

impl RunReportLedger {
    pub fn append(&self, report: RunReportV1) -> usize {
        let mut reports = self.lock_reports();
        reports.push(report);
        reports.len()
    }

    pub fn list(&self) -> Vec<RunReportV1> {
        self.lock_reports().clone()
    }

    pub fn len(&self) -> usize {
        self.lock_reports().len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn lock_reports(&self) -> MutexGuard<'_, Vec<RunReportV1>> {
        self.reports
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

pub fn disclose_runtime_view(
    request: &RuntimeViewRequestV1,
    projection_digest: ProjectionDigestV1,
    matched_claim_ids: Vec<ArtifactId>,
    query_widenings: &[QueryWideningReportV1],
    degradations: &[DegradationEventV1],
) -> ViewDisclosureReportV1 {
    ViewDisclosureReportV1::new(
        request,
        projection_digest,
        matched_claim_ids,
        query_widenings
            .iter()
            .map(|receipt| receipt.receipt_id.clone())
            .collect(),
        degradations
            .iter()
            .map(|event| event.event_id.clone())
            .collect(),
    )
}

#[derive(Clone)]
pub struct AiDENsRunner {
    app_id: String,
    provider: Arc<dyn AiDENsProvider>,
    tools: ToolRegistryV1,
    permit_policy: PermitPolicyV1,
    budget: BudgetV1,
    run_reports: RunReportLedger,
    receipt_level: ReportLevelV1,
    canonical_receipts: Option<CanonicalEventLog>,
    agency_policy: AgencyPolicyEngineV1,
    agency_nudges: Arc<Mutex<NudgeLedgerV1>>,
}

impl std::fmt::Debug for AiDENsRunner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AiDENsRunner")
            .field("app_id", &self.app_id)
            .field("provider_kind", &self.provider.provider_kind())
            .field("model", &self.provider.model())
            .field("tool_count", &self.tools.len())
            .field("permit_policy", &"scoped")
            .field("budget", &self.budget)
            .field("receipt_level", &self.receipt_level)
            .field("agency_policy", &"enabled")
            .field(
                "durable_receipts",
                &self
                    .canonical_receipts
                    .as_ref()
                    .map(|store| store.config().root_path.display().to_string()),
            )
            .finish()
    }
}

impl AiDENsRunner {
    pub fn builder() -> AiDENsRunnerBuilder {
        AiDENsRunnerBuilder::default()
    }

    pub async fn run(&self, input: AiDENsRunInput) -> anyhow::Result<AiDENsRunOutput> {
        self.run_with_tool_policy(input, ToolExposurePolicyV1::coding_agent_default())
            .await
    }

    pub async fn run_with_tool_policy(
        &self,
        input: AiDENsRunInput,
        tool_exposure_policy: ToolExposurePolicyV1,
    ) -> anyhow::Result<AiDENsRunOutput> {
        TurnExecutorV1::new(TurnExecutorConfigV1 {
            app_id: self.app_id.clone(),
            provider: self.provider.clone(),
            tools: self.tools.clone(),
            permit_policy: self.permit_policy.clone(),
            budget: self.budget.clone(),
            run_reports: self.run_reports.clone(),
            receipt_level: self.receipt_level.clone(),
            canonical_receipts: self.canonical_receipts.clone(),
            agency_policy: self.agency_policy.clone(),
            agency_nudges: self.agency_nudges.clone(),
        })
        .execute_with_tool_policy(input, tool_exposure_policy)
        .await
    }

    pub fn run_reports(&self) -> RunReportLedger {
        self.run_reports.clone()
    }

    pub fn receipt_level(&self) -> &ReportLevelV1 {
        &self.receipt_level
    }

    pub fn canonical_receipt_log_config(&self) -> Option<&CanonicalEventLogConfig> {
        self.canonical_receipts
            .as_ref()
            .map(CanonicalEventLog::config)
    }

    pub fn provider_route(&self) -> ProviderRouteReportV1 {
        route_receipt(
            self.provider.provider_kind(),
            self.provider.model().map(str::to_string),
            &self.provider.capabilities(),
        )
    }

    pub fn tool_ids(&self) -> Vec<String> {
        self.tools.tool_ids()
    }

    pub fn executable_tool_ids(&self) -> Vec<String> {
        self.tools.executable_tool_ids()
    }

    pub fn tool_exposure(&self) -> ToolExposureSetV1 {
        let provider_route = self.provider_route();
        let tool_policy = ToolExposurePolicyV1::coding_agent_default()
            .with_permit_policy(self.permit_policy.clone())
            .for_provider_route(&provider_route);
        self.tools.plan_exposure(&tool_policy)
    }
}

#[derive(Clone)]
pub struct TurnExecutorV1 {
    app_id: String,
    provider: Arc<dyn AiDENsProvider>,
    tools: ToolRegistryV1,
    permit_policy: PermitPolicyV1,
    budget: BudgetV1,
    run_reports: RunReportLedger,
    receipt_level: ReportLevelV1,
    canonical_receipts: Option<CanonicalEventLog>,
    agency_policy: AgencyPolicyEngineV1,
    agency_nudges: Arc<Mutex<NudgeLedgerV1>>,
}

#[derive(Clone)]
pub struct TurnExecutorConfigV1 {
    pub app_id: String,
    pub provider: Arc<dyn AiDENsProvider>,
    pub tools: ToolRegistryV1,
    pub permit_policy: PermitPolicyV1,
    pub budget: BudgetV1,
    pub run_reports: RunReportLedger,
    pub receipt_level: ReportLevelV1,
    pub canonical_receipts: Option<CanonicalEventLog>,
    pub agency_policy: AgencyPolicyEngineV1,
    pub agency_nudges: Arc<Mutex<NudgeLedgerV1>>,
}

impl std::fmt::Debug for TurnExecutorV1 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TurnExecutorV1")
            .field("app_id", &self.app_id)
            .field("provider_kind", &self.provider.provider_kind())
            .field("model", &self.provider.model())
            .field("tool_count", &self.tools.len())
            .field("permit_policy", &"scoped")
            .field("budget", &self.budget)
            .field("receipt_level", &self.receipt_level)
            .finish()
    }
}

impl TurnExecutorV1 {
    pub fn new(config: TurnExecutorConfigV1) -> Self {
        Self {
            app_id: config.app_id,
            provider: config.provider,
            tools: config.tools,
            permit_policy: config.permit_policy,
            budget: config.budget,
            run_reports: config.run_reports,
            receipt_level: config.receipt_level,
            canonical_receipts: config.canonical_receipts,
            agency_policy: config.agency_policy,
            agency_nudges: config.agency_nudges,
        }
    }

    pub async fn execute(&self, input: AiDENsRunInput) -> anyhow::Result<AiDENsRunOutput> {
        self.execute_with_tool_policy(input, ToolExposurePolicyV1::coding_agent_default())
            .await
    }

    pub async fn execute_with_tool_policy(
        &self,
        input: AiDENsRunInput,
        mut tool_policy: ToolExposurePolicyV1,
    ) -> anyhow::Result<AiDENsRunOutput> {
        let ctx = AidensRunContextV1::new(&self.app_id);
        let provider_route = route_receipt(
            self.provider.provider_kind(),
            self.provider.model().map(str::to_string),
            &self.provider.capabilities(),
        );
        tool_policy = tool_policy
            .with_permit_policy(self.permit_policy.clone())
            .for_provider_route(&provider_route);
        let tool_exposure = self.tools.plan_exposure(&tool_policy);
        let mode = turn_mode_for(&provider_route, &tool_exposure);
        let plan = TurnExecutionPlanV1::new(
            mode,
            provider_route.clone(),
            &tool_exposure,
            self.budget.max_tool_calls,
            self.budget.max_retries,
            self.budget.max_turn_millis,
            turn_plan_reasons(mode),
        );

        let mut run_receipt = RunReportV1::started(ctx.clone());
        run_receipt.provider_route = Some(provider_route.clone());
        run_receipt
            .tool_exposure_ids
            .push(tool_exposure.exposure_id.clone());
        run_receipt
            .approval_requests
            .extend(tool_exposure.approval_requests.clone());
        run_receipt
            .permit_use_receipts
            .extend(tool_exposure.permit_use_receipts.clone());
        if provider_route.degraded {
            run_receipt.warnings.push(format!(
                "provider-route-degraded:{}",
                provider_route.route_label
            ));
        }
        for reason in &provider_route.reason_codes {
            if reason.contains("fallback") || reason.contains("unknown-native-provider") {
                run_receipt
                    .warnings
                    .push(format!("provider-route:{reason}"));
            }
        }
        for reason in &tool_exposure.reason_codes {
            run_receipt.warnings.push(format!("tool-exposure:{reason}"));
        }

        let mut turn_receipt = TurnReportV1::started(&ctx, &plan);
        if mode == TurnModeV1::ProviderUnavailable {
            let stop = StopRuleReportV1::triggered(
                &ctx,
                StopRuleV1::ProviderUnavailable,
                provider_route.reason_codes.clone(),
            );
            turn_receipt.record_stop_rule(&stop);
            let completed_turn = turn_receipt.complete(TurnFinalStateV1::ProviderUnavailable);
            run_receipt.stop_rule_receipts.push(stop);
            run_receipt.turn_receipts.push(completed_turn);
            run_receipt.warnings.push("provider-unavailable".into());
            let completed = run_receipt.complete();
            let control_records = self.failure_control_records(
                &ctx,
                "provider-route",
                &provider_route.reason_codes,
                serde_json::json!({
                    "provider_kind": provider_route.provider_kind.clone(),
                    "route_label": provider_route.route_label.clone(),
                    "route": provider_route.route,
                    "degraded": provider_route.degraded,
                }),
            )?;
            self.persist_completed_with_records(completed, &tool_exposure, control_records)?;
            return Err(anyhow!(
                "provider unavailable: {}",
                provider_route.reason_codes.join(",")
            ));
        }

        let mut tool_results = Vec::<ToolCallResultV1>::new();
        let mut agency_policy_reports = Vec::<AgencyPolicyReportV1>::new();
        let mut tool_calls_so_far = 0u32;
        let mut retries = 0u32;
        let mut seen_tool_calls = BTreeSet::<String>::new();
        let started_at = Instant::now();
        let exposed_tool_ids = tool_exposure
            .exposed_tool_ids
            .iter()
            .cloned()
            .collect::<BTreeSet<_>>();

        loop {
            let elapsed = elapsed_millis(started_at);
            if self.budget.deadline_exceeded(elapsed) {
                return self.finish_budget_exhausted(BudgetStopInput {
                    ctx,
                    run_receipt,
                    turn_receipt,
                    tool_exposure,
                    attempted_tool_calls: tool_calls_so_far,
                    retries,
                    elapsed_millis: elapsed,
                    reason: "turn-deadline-exceeded".into(),
                    agency_policy_reports,
                });
            }

            let request = match completion_request(
                input.prompt.clone(),
                &tool_exposure,
                tool_results.clone(),
            ) {
                Ok(request) => request,
                Err(error) => {
                    let reason_codes = vec!["tool-result-serialization-failed".into()];
                    run_receipt
                        .warnings
                        .push(format!("provider-request-error:{error}"));
                    let stop = StopRuleReportV1::triggered(
                        &ctx,
                        StopRuleV1::ToolInvocationFailed,
                        reason_codes.clone(),
                    );
                    turn_receipt.record_stop_rule(&stop);
                    turn_receipt.degraded = true;
                    turn_receipt.blocked = true;
                    run_receipt.stop_rule_receipts.push(stop);
                    let completed_turn = turn_receipt.complete(TurnFinalStateV1::ToolFailed);
                    run_receipt.turn_receipts.push(completed_turn);
                    let completed = run_receipt.complete();
                    let control_records = self.failure_control_records(
                        &ctx,
                        "provider-request",
                        &reason_codes,
                        serde_json::json!({
                            "provider_kind": provider_route.provider_kind.clone(),
                            "route_label": provider_route.route_label.clone(),
                            "error": error.to_string(),
                        }),
                    )?;
                    self.persist_completed_with_records(
                        completed,
                        &tool_exposure,
                        control_records,
                    )?;
                    return Err(error);
                }
            };
            let completion = match self.provider.complete(request).await {
                Ok(completion) => completion,
                Err(error) if self.budget.allows_retry(retries) => {
                    retries += 1;
                    run_receipt.warnings.push(format!("provider-retry:{error}"));
                    continue;
                }
                Err(error) => {
                    run_receipt.warnings.push(format!("provider-error:{error}"));
                    let stop = StopRuleReportV1::triggered(
                        &ctx,
                        StopRuleV1::MaxRetries,
                        vec!["provider-error-max-retries".into()],
                    );
                    turn_receipt.record_stop_rule(&stop);
                    let completed_turn = turn_receipt.complete(TurnFinalStateV1::StopRuleTriggered);
                    run_receipt.stop_rule_receipts.push(stop);
                    run_receipt.turn_receipts.push(completed_turn);
                    let completed = run_receipt.complete();
                    let control_records = self.failure_control_records(
                        &ctx,
                        "provider-error",
                        &["provider-error-max-retries".into()],
                        serde_json::json!({
                            "provider_kind": provider_route.provider_kind.clone(),
                            "route_label": provider_route.route_label.clone(),
                            "error": error.to_string(),
                        }),
                    )?;
                    self.persist_completed_with_records(
                        completed,
                        &tool_exposure,
                        control_records,
                    )?;
                    return Err(error);
                }
            };

            let completion_tool_calls = tool_calls_for_completion(&completion, mode);
            run_receipt
                .boundary_repair_receipts
                .extend(completion_tool_calls.boundary_repair_receipts);
            run_receipt
                .json_repair_receipts
                .extend(completion_tool_calls.json_repair_receipts);
            if !completion_tool_calls.degradation_reason_codes.is_empty() {
                let reason_codes = completion_tool_calls.degradation_reason_codes;
                let stop = StopRuleReportV1::triggered(
                    &ctx,
                    StopRuleV1::ToolInvocationFailed,
                    reason_codes.clone(),
                );
                turn_receipt.record_stop_rule(&stop);
                turn_receipt.degraded = true;
                turn_receipt.blocked = true;
                run_receipt.stop_rule_receipts.push(stop);
                run_receipt.warnings.extend(
                    reason_codes
                        .iter()
                        .map(|reason| format!("tool-call-degraded:{reason}")),
                );
                let completed_turn = turn_receipt.complete(TurnFinalStateV1::ToolFailed);
                run_receipt.turn_receipts.push(completed_turn.clone());
                let completed = run_receipt.complete();
                let control_records = self.failure_control_records(
                    &ctx,
                    "parser-fallback-tool-call",
                    &reason_codes,
                    serde_json::json!({
                        "provider_kind": provider_route.provider_kind.clone(),
                        "route_label": provider_route.route_label.clone(),
                        "provider_text": completion.text.clone(),
                    }),
                )?;
                let (completed, durable_receipt_records) = self.persist_completed_with_records(
                    completed,
                    &tool_exposure,
                    control_records,
                )?;
                return Ok(AiDENsRunOutput {
                    text: "Turn stopped: malformed parser-fallback tool call.".into(),
                    receipt: completed,
                    turn_receipt: completed_turn,
                    tool_exposure,
                    agency_policy_reports,
                    durable_receipt_records,
                });
            }
            let tool_calls = completion_tool_calls.calls;
            if tool_calls.is_empty() {
                let tool_result_texts = tool_results
                    .iter()
                    .map(ToolCallResultV1::output_text)
                    .filter(|text| !text.trim().is_empty())
                    .collect::<Vec<_>>();
                let agency_input = AgencyPolicyInputV1::for_runner_final_output(
                    input.prompt.clone(),
                    completion.text.clone(),
                    &tool_result_texts,
                );
                let agency_report = self.evaluate_agency_policy(&agency_input);
                run_receipt
                    .agency_receipt_ids
                    .extend(agency_report.receipt_ids());
                run_receipt.warnings.extend(
                    agency_report
                        .reason_codes()
                        .iter()
                        .map(|reason| format!("agency-policy:{reason}")),
                );
                let final_text = final_text_after_agency_gate(&completion.text, &agency_report);
                let agency_outcome = agency_report.outcome;
                agency_policy_reports.push(agency_report);

                if !agency_outcome.allows_direct_output() {
                    let reason_codes = vec![format!(
                        "agency-policy:{}",
                        agency_outcome.as_policy_label()
                    )];
                    let stop = StopRuleReportV1::triggered(
                        &ctx,
                        StopRuleV1::AgencyPolicy,
                        reason_codes.clone(),
                    );
                    turn_receipt.record_stop_rule(&stop);
                    turn_receipt.degraded = true;
                    turn_receipt.blocked = true;
                    run_receipt.stop_rule_receipts.push(stop);
                    run_receipt.warnings.extend(reason_codes.clone());
                    let completed_turn = turn_receipt.complete(TurnFinalStateV1::StopRuleTriggered);
                    run_receipt.turn_receipts.push(completed_turn.clone());
                    let completed = run_receipt.complete();
                    let mut control_records = self.turn_control_records(
                        &ctx,
                        "agency-policy",
                        &reason_codes,
                        serde_json::json!({
                            "control_decision": agency_outcome.as_policy_label(),
                            "agency_reports": &agency_policy_reports,
                            "provider_kind": provider_route.provider_kind.clone(),
                            "route_label": provider_route.route_label.clone(),
                            "turn_receipt_id": completed_turn.receipt_id.clone(),
                            "turn_final_state": completed_turn.final_state,
                            "tool_exposure_id": tool_exposure.exposure_id.clone(),
                        }),
                        true,
                        governance_stack::VerificationAttemptState::Blocked,
                    )?;
                    control_records.extend(self.agency_policy_records(&agency_policy_reports)?);
                    let (completed, durable_receipt_records) = self
                        .persist_completed_with_records(
                            completed,
                            &tool_exposure,
                            control_records,
                        )?;
                    return Ok(AiDENsRunOutput {
                        text: final_text,
                        receipt: completed,
                        turn_receipt: completed_turn,
                        tool_exposure,
                        agency_policy_reports,
                        durable_receipt_records,
                    });
                }

                let reason_codes = vec!["final-output-produced".into()];
                let stop = StopRuleReportV1::triggered(
                    &ctx,
                    StopRuleV1::FinalOutput,
                    reason_codes.clone(),
                );
                turn_receipt.record_stop_rule(&stop);
                let completed_turn = turn_receipt.complete(TurnFinalStateV1::FinalOutput);
                run_receipt.stop_rule_receipts.push(stop);
                run_receipt.turn_receipts.push(completed_turn.clone());
                let completed = run_receipt.complete();
                let mut control_records = self.turn_control_records(
                    &ctx,
                    "final-output",
                    &reason_codes,
                    serde_json::json!({
                        "control_decision": "final-output-produced",
                        "provider_kind": provider_route.provider_kind.clone(),
                        "route_label": provider_route.route_label.clone(),
                        "turn_receipt_id": completed_turn.receipt_id.clone(),
                        "turn_final_state": completed_turn.final_state,
                        "tool_exposure_id": tool_exposure.exposure_id.clone(),
                        "exposed_tool_ids": tool_exposure.exposed_tool_ids.clone(),
                    }),
                    completed_turn.degraded,
                    governance_stack::VerificationAttemptState::AdvisoryOnly,
                )?;
                control_records.extend(self.agency_policy_records(&agency_policy_reports)?);
                let (completed, durable_receipt_records) = self.persist_completed_with_records(
                    completed,
                    &tool_exposure,
                    control_records,
                )?;
                return Ok(AiDENsRunOutput {
                    text: final_text,
                    receipt: completed,
                    turn_receipt: completed_turn,
                    tool_exposure,
                    agency_policy_reports,
                    durable_receipt_records,
                });
            }

            let requested_tool_calls = tool_calls.len() as u32;
            if !self
                .budget
                .allows_tool_call(tool_calls_so_far, requested_tool_calls)
            {
                for tool_call in tool_calls {
                    let invocation = ToolInvocationReportV1::started(
                        tool_call.tool_id.clone(),
                        tool_call.input.clone(),
                    )
                    .with_execution_context(&ctx)
                    .complete_failure("budget-exhausted-before-dispatch");
                    let result = ToolCallResultV1::from_invocation(&tool_call, &invocation);
                    turn_receipt.record_tool_call(&tool_call, &invocation);
                    run_receipt.tool_call_requests.push(tool_call);
                    run_receipt.tool_call_results.push(result);
                    run_receipt.tool_invocation_receipts.push(invocation);
                }
                return self.finish_budget_exhausted(BudgetStopInput {
                    ctx,
                    run_receipt,
                    turn_receipt,
                    tool_exposure,
                    attempted_tool_calls: tool_calls_so_far.saturating_add(requested_tool_calls),
                    retries,
                    elapsed_millis: elapsed_millis(started_at),
                    reason: "max-tool-calls-exhausted".into(),
                    agency_policy_reports,
                });
            }

            for tool_call in tool_calls {
                let call_signature = format!("{}:{}", tool_call.tool_id, tool_call.input_digest);
                if !seen_tool_calls.insert(call_signature) {
                    let invocation = ToolInvocationReportV1::started(
                        tool_call.tool_id.clone(),
                        tool_call.input.clone(),
                    )
                    .with_execution_context(&ctx)
                    .complete_failure("recursive-tool-call");
                    let result = ToolCallResultV1::from_invocation(&tool_call, &invocation);
                    turn_receipt.record_tool_call(&tool_call, &invocation);
                    let stop = StopRuleReportV1::triggered(
                        &ctx,
                        StopRuleV1::RecursiveToolCall,
                        vec!["recursive-tool-call".into()],
                    );
                    turn_receipt.record_stop_rule(&stop);
                    return self.finish_blocked_turn(BlockedTurnInput {
                        run_receipt,
                        turn_receipt,
                        tool_exposure,
                        tool_call,
                        invocation,
                        result,
                        stop_rule: stop,
                        text: "Turn stopped: recursive tool call was blocked.".into(),
                        final_state: TurnFinalStateV1::ToolBlocked,
                        agency_policy_reports,
                    });
                }

                if !exposed_tool_ids.contains(&tool_call.tool_id) {
                    let invocation = ToolInvocationReportV1::started(
                        tool_call.tool_id.clone(),
                        tool_call.input.clone(),
                    )
                    .with_execution_context(&ctx)
                    .complete_failure("tool-not-exposed-this-turn");
                    let result = ToolCallResultV1::from_invocation(&tool_call, &invocation);
                    turn_receipt.record_tool_call(&tool_call, &invocation);
                    let stop = StopRuleReportV1::triggered(
                        &ctx,
                        StopRuleV1::ToolNotExposed,
                        vec!["tool-not-exposed-this-turn".into()],
                    );
                    turn_receipt.record_stop_rule(&stop);
                    return self.finish_blocked_turn(BlockedTurnInput {
                        run_receipt,
                        turn_receipt,
                        tool_exposure,
                        tool_call,
                        invocation,
                        result,
                        stop_rule: stop,
                        text: "Turn stopped: provider requested a tool that was not exposed."
                            .into(),
                        final_state: TurnFinalStateV1::ToolBlocked,
                        agency_policy_reports,
                    });
                }

                let dispatcher = ToolDispatcher::new(self.tools.clone())
                    .with_permit_policy(self.permit_policy.clone());
                let invocation = match dispatcher
                    .invoke(&tool_call.tool_id, tool_call.input.clone())
                    .await
                {
                    Ok(outcome) => {
                        if let Some(mut permit_use_receipt) = outcome.permit_use_receipt {
                            permit_use_receipt.run_id = Some(ctx.run_id.clone());
                            permit_use_receipt.attempt_id = Some(ctx.attempt_id.clone());
                            run_receipt.permit_use_receipts.push(permit_use_receipt);
                        }
                        outcome.receipt.with_execution_context(&ctx)
                    }
                    Err(error) => {
                        if let Some(invocation_error) = error.downcast_ref::<ToolInvocationError>()
                        {
                            if let Some(approval_request) = invocation_error.approval_request() {
                                run_receipt.approval_requests.push(approval_request.clone());
                            }
                            if let Some(schema_validation) =
                                invocation_error.schema_validation_receipt()
                            {
                                run_receipt
                                    .schema_validation_receipts
                                    .push(schema_validation.clone().with_execution_context(&ctx));
                            }
                        }
                        tool_invocation_receipt_from_error(
                            &tool_call.tool_id,
                            tool_call.input.clone(),
                            &error,
                            &ctx,
                        )
                    }
                };
                let result = ToolCallResultV1::from_invocation(&tool_call, &invocation);
                turn_receipt.record_tool_call(&tool_call, &invocation);
                run_receipt.tool_call_requests.push(tool_call.clone());
                run_receipt.tool_call_results.push(result.clone());
                run_receipt
                    .tool_invocation_receipts
                    .push(invocation.clone());
                tool_results.push(result.clone());
                tool_calls_so_far += 1;

                if result.succeeded {
                    let tool_output = result.output_text();
                    if !tool_output.trim().is_empty() {
                        let agency_input = AgencyPolicyInputV1::for_runner_tool_output(
                            input.prompt.clone(),
                            tool_output,
                        );
                        let agency_report = self.evaluate_agency_policy(&agency_input);
                        let agency_outcome = agency_report.outcome;
                        run_receipt
                            .agency_receipt_ids
                            .extend(agency_report.receipt_ids());
                        run_receipt.warnings.extend(
                            agency_report
                                .reason_codes()
                                .iter()
                                .map(|reason| format!("agency-policy:{reason}")),
                        );
                        agency_policy_reports.push(agency_report);
                        if !agency_outcome.allows_direct_output() {
                            let stop = StopRuleReportV1::triggered(
                                &ctx,
                                StopRuleV1::AgencyPolicy,
                                vec![format!(
                                    "agency-policy:{}",
                                    agency_outcome.as_policy_label()
                                )],
                            );
                            turn_receipt.record_stop_rule(&stop);
                            run_receipt.stop_rule_receipts.push(stop);
                            turn_receipt.degraded = true;
                            turn_receipt.blocked = true;
                            let completed_turn =
                                turn_receipt.complete(TurnFinalStateV1::StopRuleTriggered);
                            run_receipt.turn_receipts.push(completed_turn.clone());
                            let completed = run_receipt.complete();
                            let mut control_records = self.failure_control_records(
                                &ctx,
                                "agency-tool-output-policy",
                                &[format!(
                                    "agency-policy:{}",
                                    agency_outcome.as_policy_label()
                                )],
                                serde_json::json!({
                                    "agency_reports": &agency_policy_reports,
                                    "tool_result": result,
                                }),
                            )?;
                            control_records
                                .extend(self.agency_policy_records(&agency_policy_reports)?);
                            let (completed, durable_receipt_records) = self
                                .persist_completed_with_records(
                                    completed,
                                    &tool_exposure,
                                    control_records,
                                )?;
                            return Ok(AiDENsRunOutput {
                                text: final_text_for_agency_block(agency_outcome),
                                receipt: completed,
                                turn_receipt: completed_turn,
                                tool_exposure,
                                agency_policy_reports,
                                durable_receipt_records,
                            });
                        }
                    }
                }

                if !invocation.succeeded {
                    let stop = StopRuleReportV1::triggered(
                        &ctx,
                        StopRuleV1::ToolInvocationFailed,
                        invocation.reason_codes.clone(),
                    );
                    turn_receipt.record_stop_rule(&stop);
                    run_receipt.stop_rule_receipts.push(stop);
                    turn_receipt.degraded = true;
                    let completed_turn = turn_receipt.complete(TurnFinalStateV1::ToolFailed);
                    run_receipt.turn_receipts.push(completed_turn.clone());
                    run_receipt.warnings.push("tool-invocation-failed".into());
                    let completed = run_receipt.complete();
                    let control_records = self.failure_control_records(
                        &ctx,
                        &invocation.tool_id,
                        &invocation.reason_codes,
                        serde_json::json!({
                            "tool_invocation": invocation,
                            "tool_result": result,
                        }),
                    )?;
                    let (completed, durable_receipt_records) = self
                        .persist_completed_with_records(
                            completed,
                            &tool_exposure,
                            control_records,
                        )?;
                    return Ok(AiDENsRunOutput {
                        text: "Turn stopped: tool invocation failed.".into(),
                        receipt: completed,
                        turn_receipt: completed_turn,
                        tool_exposure,
                        agency_policy_reports,
                        durable_receipt_records,
                    });
                }
            }
        }
    }

    fn persist_completed(
        &self,
        completed: RunReportV1,
        tool_exposure: &ToolExposureSetV1,
    ) -> anyhow::Result<(RunReportV1, Vec<CanonicalEventLogEntry>)> {
        self.persist_completed_with_records(completed, tool_exposure, Vec::new())
    }

    fn evaluate_agency_policy(&self, input: &AgencyPolicyInputV1) -> AgencyPolicyReportV1 {
        let (mut ledger, recovered_from_poison) = match self.agency_nudges.lock() {
            Ok(ledger) => (ledger, false),
            Err(poisoned) => (poisoned.into_inner(), true),
        };
        let mut report = self.agency_policy.evaluate(input, &mut ledger);
        if recovered_from_poison {
            report
                .decision
                .reason_codes
                .push("agency-nudge-ledger-poison-recovered".into());
            report.decision.reason_codes.sort();
            report.decision.reason_codes.dedup();
            report.receipts.decision.reason_codes = report.decision.reason_codes.clone();
        }
        report
    }

    fn agency_policy_records(
        &self,
        reports: &[AgencyPolicyReportV1],
    ) -> anyhow::Result<Vec<CanonicalEventLogEntry>> {
        let Some(store) = &self.canonical_receipts else {
            return Ok(Vec::new());
        };
        let mut records = Vec::new();
        for report in reports {
            records.push(store.append_json(
                "aidens-agency-kit",
                "agency-policy-report-v1",
                report.report_id.clone(),
                serde_json::to_value(report)?,
            )?);
        }
        Ok(records)
    }

    fn persist_completed_with_records(
        &self,
        completed: RunReportV1,
        tool_exposure: &ToolExposureSetV1,
        mut durable_receipt_records: Vec<CanonicalEventLogEntry>,
    ) -> anyhow::Result<(RunReportV1, Vec<CanonicalEventLogEntry>)> {
        self.run_reports.append(completed.clone());
        if let Some(store) = &self.canonical_receipts {
            let tool_exposure_record_id =
                format!("{}:{}", tool_exposure.exposure_id, completed.receipt_id);
            durable_receipt_records.push(store.append_orchestration_report(
                "tool-exposure-plan-v1",
                tool_exposure_record_id,
                serde_json::to_value(tool_exposure)?,
            )?);
            durable_receipt_records.push(store.append_orchestration_report(
                "run-report-v1",
                completed.receipt_id.to_string(),
                serde_json::to_value(&completed)?,
            )?);
        }
        Ok((completed, durable_receipt_records))
    }

    fn turn_control_records(
        &self,
        context: &AidensRunContextV1,
        target_key: &str,
        reason_codes: &[String],
        details: serde_json::Value,
        degraded: bool,
        state: governance_stack::VerificationAttemptState,
    ) -> anyhow::Result<Vec<CanonicalEventLogEntry>> {
        let Some(store) = &self.canonical_receipts else {
            return Ok(Vec::new());
        };
        let adapter = CanonicalGovernanceAdapter;
        let recorded_at = context.started_at.to_rfc3339();
        let reason_summary = if reason_codes.is_empty() {
            "turn-control-decision".to_string()
        } else {
            reason_codes.join(",")
        };
        let receipt_details = serde_json::json!({
            "control_surface": "runner-turn",
            "reason_codes": reason_codes,
            "verification_attempt_state": state,
            "details": details,
        });
        let case = adapter.verification_case(
            governance_stack::VerificationCaseClass::QueryTurn,
            self.app_id.clone(),
            target_key.to_string(),
            context.stack_trace_ctx(),
            context.stack_attempt_id(),
            recorded_at.clone(),
            degraded,
            true,
        );
        let plan = adapter.check_plan(
            &case,
            governance_stack::CheckMethod::AdvisoryOnly,
            reason_codes.to_vec(),
            governance_stack::PromotionClass::P3,
            governance_stack::ReversibilityClass::ReversibleLocal,
            false,
            true,
            degraded,
            format!("runner turn control decision: {reason_summary}"),
            receipt_details.clone(),
        );
        let attempt = adapter.completed_attempt(
            &case,
            &plan,
            state,
            recorded_at.clone(),
            recorded_at,
            Some(reason_summary),
        );
        let receipt =
            adapter.control_receipt_for_attempt(&case, &plan, &attempt, false, receipt_details);
        Ok(vec![store.append_control_receipt(&receipt)?])
    }

    fn failure_control_records(
        &self,
        context: &AidensRunContextV1,
        target_key: &str,
        reason_codes: &[String],
        details: serde_json::Value,
    ) -> anyhow::Result<Vec<CanonicalEventLogEntry>> {
        let Some(store) = &self.canonical_receipts else {
            let recorded_at = context.started_at.to_rfc3339();
            let reason_summary = if reason_codes.is_empty() {
                "failure-honesty-degraded".to_string()
            } else {
                reason_codes.join(",")
            };
            let non_durable = serde_json::json!({
                "durable": false,
                "phase": "04-failure-honesty",
                "reason_summary": reason_summary,
                "reason_codes": reason_codes,
                "details": details,
                "target_key": target_key,
                "app_id": self.app_id.clone(),
            });
            let receipt = CanonicalEventLogEntry::new(
                "aidens-runner",
                "in-memory-control-receipt",
                format!("failure-control:{target_key}:{recorded_at}"),
                non_durable,
            )?;
            return Ok(vec![receipt]);
        };
        let adapter = CanonicalGovernanceAdapter;
        let recorded_at = context.started_at.to_rfc3339();
        let reason_summary = if reason_codes.is_empty() {
            "failure-honesty-degraded".to_string()
        } else {
            reason_codes.join(",")
        };
        let receipt_details = serde_json::json!({
            "phase": "04-failure-honesty",
            "reason_codes": reason_codes,
            "details": details,
        });
        let case = adapter.verification_case(
            governance_stack::VerificationCaseClass::QueryTurn,
            self.app_id.clone(),
            target_key.to_string(),
            context.stack_trace_ctx(),
            context.stack_attempt_id(),
            recorded_at.clone(),
            true,
            true,
        );
        let plan = adapter.check_plan(
            &case,
            governance_stack::CheckMethod::AdvisoryOnly,
            reason_codes.to_vec(),
            governance_stack::PromotionClass::P3,
            governance_stack::ReversibilityClass::ReversibleLocal,
            false,
            true,
            true,
            format!("failure honesty degradation: {reason_summary}"),
            receipt_details.clone(),
        );
        let attempt = adapter.completed_attempt(
            &case,
            &plan,
            governance_stack::VerificationAttemptState::Blocked,
            recorded_at.clone(),
            recorded_at,
            Some(reason_summary),
        );
        let receipt =
            adapter.control_receipt_for_attempt(&case, &plan, &attempt, false, receipt_details);
        Ok(vec![store.append_control_receipt(&receipt)?])
    }

    fn finish_budget_exhausted(&self, input: BudgetStopInput) -> anyhow::Result<AiDENsRunOutput> {
        let BudgetStopInput {
            ctx,
            mut run_receipt,
            mut turn_receipt,
            tool_exposure,
            attempted_tool_calls,
            retries,
            elapsed_millis,
            reason,
            agency_policy_reports,
        } = input;
        let budget_receipt = BudgetExhaustionReportV1::new(BudgetExhaustionReportDraftV1 {
            run_id: ctx.run_id.clone(),
            attempt_id: ctx.attempt_id.clone(),
            max_tool_calls: self.budget.max_tool_calls,
            attempted_tool_calls,
            max_retries: self.budget.max_retries,
            retries,
            max_turn_millis: self.budget.max_turn_millis,
            elapsed_millis,
            reason_codes: vec![reason.clone()],
        });
        let stop_rule = StopRuleReportV1::triggered(
            &ctx,
            if reason == "turn-deadline-exceeded" {
                StopRuleV1::DeadlineExceeded
            } else {
                StopRuleV1::MaxToolCalls
            },
            vec![reason.clone()],
        );
        turn_receipt.record_budget_exhaustion(&budget_receipt);
        turn_receipt.record_stop_rule(&stop_rule);
        let completed_turn = turn_receipt.complete(TurnFinalStateV1::BudgetExhausted);
        let control_records = self.failure_control_records(
            &ctx,
            "budget-exhaustion",
            std::slice::from_ref(&reason),
            serde_json::json!({
                "budget_exhaustion": budget_receipt.clone(),
            }),
        )?;
        run_receipt.budget_exhaustion_receipts.push(budget_receipt);
        run_receipt.stop_rule_receipts.push(stop_rule);
        run_receipt.turn_receipts.push(completed_turn.clone());
        run_receipt
            .warnings
            .push(format!("budget-exhausted:{reason}"));
        let completed = run_receipt.complete();
        let (completed, durable_receipt_records) =
            self.persist_completed_with_records(completed, &tool_exposure, control_records)?;
        Ok(AiDENsRunOutput {
            text: format!("Turn stopped: budget exhausted ({reason})."),
            receipt: completed,
            turn_receipt: completed_turn,
            tool_exposure,
            agency_policy_reports,
            durable_receipt_records,
        })
    }

    fn finish_blocked_turn(&self, input: BlockedTurnInput) -> anyhow::Result<AiDENsRunOutput> {
        let BlockedTurnInput {
            mut run_receipt,
            mut turn_receipt,
            tool_exposure,
            tool_call,
            invocation,
            result,
            stop_rule,
            text,
            final_state,
            agency_policy_reports,
        } = input;
        run_receipt.tool_call_requests.push(tool_call);
        run_receipt.tool_call_results.push(result);
        run_receipt.tool_invocation_receipts.push(invocation);
        run_receipt.stop_rule_receipts.push(stop_rule);
        turn_receipt.degraded = true;
        turn_receipt.blocked = true;
        let completed_turn = turn_receipt.complete(final_state);
        run_receipt.turn_receipts.push(completed_turn.clone());
        run_receipt.warnings.push("turn-blocked".into());
        let completed = run_receipt.complete();
        let (completed, durable_receipt_records) =
            self.persist_completed(completed, &tool_exposure)?;
        Ok(AiDENsRunOutput {
            text,
            receipt: completed,
            turn_receipt: completed_turn,
            tool_exposure,
            agency_policy_reports,
            durable_receipt_records,
        })
    }
}

struct BudgetStopInput {
    ctx: AidensRunContextV1,
    run_receipt: RunReportV1,
    turn_receipt: TurnReportV1,
    tool_exposure: ToolExposureSetV1,
    attempted_tool_calls: u32,
    retries: u32,
    elapsed_millis: u64,
    reason: String,
    agency_policy_reports: Vec<AgencyPolicyReportV1>,
}

struct BlockedTurnInput {
    run_receipt: RunReportV1,
    turn_receipt: TurnReportV1,
    tool_exposure: ToolExposureSetV1,
    tool_call: ToolCallRequestV1,
    invocation: ToolInvocationReportV1,
    result: ToolCallResultV1,
    stop_rule: StopRuleReportV1,
    text: String,
    final_state: TurnFinalStateV1,
    agency_policy_reports: Vec<AgencyPolicyReportV1>,
}

#[derive(Clone)]
pub struct AiDENsRunnerBuilder {
    app_id: String,
    provider: Option<Arc<dyn AiDENsProvider>>,
    provider_spec: ProviderSpecV1,
    tools: ToolRegistryV1,
    permit_policy: PermitPolicyV1,
    budget: BudgetV1,
    run_reports: RunReportLedger,
    receipt_level: ReportLevelV1,
    canonical_receipt_log_config: Option<CanonicalEventLogConfig>,
    agency_policy: AgencyPolicyEngineV1,
    agency_nudges: Arc<Mutex<NudgeLedgerV1>>,
}

impl Default for AiDENsRunnerBuilder {
    fn default() -> Self {
        Self {
            app_id: "aidens-app".into(),
            provider: None,
            provider_spec: ProviderSpecV1::new("disabled"),
            tools: safe_coding_registry_for_current_dir(),
            permit_policy: PermitPolicyV1::default(),
            budget: BudgetV1::default(),
            run_reports: RunReportLedger::default(),
            receipt_level: ReportLevelV1::Full,
            canonical_receipt_log_config: None,
            agency_policy: AgencyPolicyEngineV1::default(),
            agency_nudges: Arc::new(Mutex::new(NudgeLedgerV1::default())),
        }
    }
}

impl AiDENsRunnerBuilder {
    pub fn app_id(mut self, app_id: impl Into<String>) -> Self {
        self.app_id = app_id.into();
        self
    }

    pub fn provider(mut self, provider: Arc<dyn AiDENsProvider>) -> Self {
        self.provider = Some(provider);
        self
    }

    pub fn provider_spec(mut self, spec: ProviderSpecV1) -> Self {
        self.provider_spec = spec;
        self.provider = None;
        self
    }

    pub fn provider_kind(mut self, kind: impl Into<String>) -> Self {
        self.provider_spec.kind = kind.into();
        self.provider = None;
        self
    }

    pub fn mock_provider(mut self, response: impl Into<String>) -> Self {
        self.provider_spec.kind = "mock".into();
        self.provider_spec.mock_response = Some(response.into());
        self.provider = None;
        self
    }

    pub fn model(mut self, model: impl Into<String>) -> Self {
        self.provider_spec.model = Some(model.into());
        self.provider = None;
        self
    }

    pub fn base_url(mut self, base_url: impl Into<String>) -> Self {
        self.provider_spec.base_url = Some(base_url.into());
        self.provider = None;
        self
    }

    pub fn api_key(mut self, api_key: impl Into<String>) -> Self {
        self.provider_spec.api_key = Some(api_key.into());
        self.provider = None;
        self
    }

    pub fn tools(mut self, tools: ToolRegistryV1) -> Self {
        self.tools = tools;
        self
    }

    pub fn permit_policy(mut self, permit_policy: PermitPolicyV1) -> Self {
        self.permit_policy = permit_policy;
        self
    }

    pub fn budget(mut self, budget: BudgetV1) -> Self {
        self.budget = budget;
        self
    }

    pub fn run_reports(mut self, reports: RunReportLedger) -> Self {
        self.run_reports = reports;
        self
    }

    pub fn receipt_level(mut self, receipt_level: ReportLevelV1) -> Self {
        self.receipt_level = receipt_level;
        self
    }

    pub fn canonical_receipt_log_config(mut self, config: CanonicalEventLogConfig) -> Self {
        self.canonical_receipt_log_config = Some(config);
        self
    }

    pub fn agency_policy(mut self, policy: AgencyPolicyEngineV1) -> Self {
        self.agency_policy = policy;
        self
    }

    pub fn agency_nudge_ledger(mut self, ledger: Arc<Mutex<NudgeLedgerV1>>) -> Self {
        self.agency_nudges = ledger;
        self
    }

    pub fn build(self) -> anyhow::Result<AiDENsRunner> {
        let provider = match self.provider {
            Some(provider) => provider,
            None => build_provider(self.provider_spec)?,
        };
        let canonical_receipt_log_config = self
            .canonical_receipt_log_config
            .unwrap_or_else(|| default_runner_receipt_log_config(&self.app_id));
        let canonical_receipts = Some(CanonicalEventLog::open(canonical_receipt_log_config)?);
        Ok(AiDENsRunner {
            app_id: self.app_id,
            provider,
            tools: self.tools,
            permit_policy: self.permit_policy,
            budget: self.budget,
            run_reports: self.run_reports,
            receipt_level: self.receipt_level,
            canonical_receipts,
            agency_policy: self.agency_policy,
            agency_nudges: self.agency_nudges,
        })
    }
}

#[cfg(test)]
mod tests;
