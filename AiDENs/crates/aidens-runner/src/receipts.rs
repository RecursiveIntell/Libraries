//! Receipt and output DTOs for runner material operations.
//!
//! These are AiDENs-local execution receipts; canonical receipt truth remains
//! with `aidens-receipts` and sibling verification crates.

use super::*;

#[derive(Debug, Clone)]
pub struct PlanReceiptV1 {
    pub receipt_id: ArtifactId,
    pub plan_id: ArtifactId,
    pub step: u32,
    pub action: String,
    pub reason_codes: Vec<String>,
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

#[derive(Debug, Clone)]
pub struct VerificationReceiptV1 {
    pub receipt_id: ArtifactId,
    pub step: u32,
    pub check: String,
    pub passed: bool,
    pub reason_codes: Vec<String>,
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
            abstention_receipt: Some(AbstentionReceiptV1 {
                receipt_id: display_only_unstable_id("agent-abstention"),
                step,
                reason_code: reason_code.into(),
                blocked_action: "execution-blocked-before-run".into(),
                evidence: Vec::new(),
                required_permits: Vec::new(),
                can_resume: true,
                support_impact: "partial".into(),
            }),
            repair_plan: None,
            finalization: None,
            run_output: None,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn assemble_v3_bundle(
        &self,
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
        memory_grounding_receipts: Vec<String>,
        outputs: Vec<String>,
        replay_instructions: Vec<String>,
        blocked_checks: Vec<String>,
    ) -> AiDENsRunBundleV3 {
        let mut bundle = AiDENsRunBundleV3::new(
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
        bundle.provider_receipts = self
            .tool_route_receipts
            .iter()
            .map(|receipt| receipt.receipt_id.to_string())
            .collect();
        bundle.tool_receipts = self
            .tool_call_receipts
            .iter()
            .map(|receipt| receipt.receipt_id.to_string())
            .collect();
        bundle.permit_receipts = self
            .tool_call_receipts
            .iter()
            .map(|receipt| format!("permit-use:{}", receipt.receipt_id))
            .collect();
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
        bundle
    }
}
