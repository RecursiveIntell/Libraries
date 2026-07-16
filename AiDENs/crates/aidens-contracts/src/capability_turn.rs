//! Capability, permit, turn, stop-rule, and run-report DTOs.
//!
//! These are orchestration receipts and display reports, not canonical tool/runtime truth.

use super::*;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct PoisonReportEntryV1 {
    pub poison_id: ArtifactId,
    pub source_path: String,
    pub line_number: u64,
    pub raw_digest: String,
    pub raw_line: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reason_codes: Vec<String>,
    pub recorded_at: DateTime<Utc>,
}

impl PoisonReportEntryV1 {
    pub fn new(
        source_path: impl Into<String>,
        line_number: u64,
        raw_line: impl Into<String>,
        reason: impl Into<String>,
    ) -> Self {
        let raw_line = raw_line.into();
        Self {
            poison_id: display_only_unstable_id("poison-receipt"),
            source_path: source_path.into(),
            line_number,
            raw_digest: non_authoritative_json_display_digest(&serde_json::Value::String(
                raw_line.clone(),
            )),
            raw_line,
            reason_codes: vec![reason.into()],
            recorded_at: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ExecutionLineageGraphV1 {
    pub graph_id: ArtifactId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub run_id: Option<ArtifactId>,
    pub nodes: Vec<ExecutionLineageNodeV1>,
    pub edges: Vec<ExecutionLineageEdgeV1>,
    pub generated_at: DateTime<Utc>,
}

impl ExecutionLineageGraphV1 {
    pub fn new(
        run_id: Option<ArtifactId>,
        nodes: Vec<ExecutionLineageNodeV1>,
        edges: Vec<ExecutionLineageEdgeV1>,
    ) -> Self {
        Self {
            graph_id: display_only_unstable_id("execution-lineage-graph"),
            run_id,
            nodes,
            edges,
            generated_at: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ExecutionLineageNodeV1 {
    pub receipt_id: ArtifactId,
    pub kind: ArtifactKindV1,
    pub content_digest: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ExecutionLineageEdgeV1 {
    pub parent_receipt_id: ArtifactId,
    pub child_receipt_id: ArtifactId,
    pub relation: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum ToolLifecycleStateV1 {
    Declared,
    Registered,
    Executable,
    Exposed,
    ExposedThisTurn,
    Invoked,
    Succeeded,
    Failed,
    Hidden,
    Blocked,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum CapabilityGateOutcomeV1 {
    Exposed,
    Hidden,
    Blocked,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct CapabilityGateDecisionV1 {
    pub decision_id: ArtifactId,
    pub kind: ArtifactKindV1,
    pub capability_id: String,
    pub outcome: CapabilityGateOutcomeV1,
    pub lifecycle: Vec<ToolLifecycleStateV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub risk_class: Option<CanonicalToolSideEffectClass>,
    #[serde(default)]
    pub permit_required: bool,
    pub executable_this_turn: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sandbox_root: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub approval_request: Option<ApprovalRequestV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub permit_grant_id: Option<ArtifactId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub permit_use_receipt_id: Option<ArtifactId>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reason_codes: Vec<String>,
}

impl CapabilityGateDecisionV1 {
    pub fn new(
        capability_id: impl Into<String>,
        outcome: CapabilityGateOutcomeV1,
        lifecycle: Vec<ToolLifecycleStateV1>,
        executable_this_turn: bool,
        reason_codes: Vec<String>,
    ) -> Self {
        Self {
            decision_id: display_only_unstable_id("capability-gate"),
            kind: ArtifactKindV1::ToolExposure,
            capability_id: capability_id.into(),
            outcome,
            lifecycle,
            risk_class: None,
            permit_required: false,
            executable_this_turn,
            sandbox_root: None,
            approval_request: None,
            permit_grant_id: None,
            permit_use_receipt_id: None,
            reason_codes,
        }
    }

    pub fn for_tool(draft: CapabilityGateDecisionDraftV1) -> Self {
        Self {
            decision_id: display_only_unstable_id("capability-gate"),
            kind: ArtifactKindV1::ToolExposure,
            capability_id: draft.tool_id,
            outcome: draft.outcome,
            lifecycle: draft.lifecycle,
            risk_class: Some(draft.risk_class),
            permit_required: draft.permit_required,
            executable_this_turn: draft.executable_this_turn,
            sandbox_root: draft.sandbox_root,
            approval_request: draft.approval_request,
            permit_grant_id: draft.permit_grant_id,
            permit_use_receipt_id: draft.permit_use_receipt_id,
            reason_codes: draft.reason_codes,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct CapabilityGateDecisionDraftV1 {
    pub tool_id: String,
    pub outcome: CapabilityGateOutcomeV1,
    pub lifecycle: Vec<ToolLifecycleStateV1>,
    pub risk_class: CanonicalToolSideEffectClass,
    pub permit_required: bool,
    pub executable_this_turn: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sandbox_root: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub approval_request: Option<ApprovalRequestV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub permit_grant_id: Option<ArtifactId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub permit_use_receipt_id: Option<ArtifactId>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reason_codes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct PermitGrantV1 {
    pub permit_id: ArtifactId,
    pub risk_class: CanonicalToolSideEffectClass,
    pub tool_id: String,
    pub sandbox_root: String,
    pub scope: String,
    pub granted_by: String,
    pub granted_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub run_id: Option<ArtifactId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attempt_id: Option<ArtifactId>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reason_codes: Vec<String>,
}

impl PermitGrantV1 {
    pub fn new(
        risk_class: CanonicalToolSideEffectClass,
        scope: impl Into<String>,
        granted_by: impl Into<String>,
    ) -> Self {
        let scope = scope.into();
        Self {
            permit_id: display_only_unstable_id("permit"),
            risk_class,
            tool_id: "*".into(),
            sandbox_root: scope.clone(),
            scope,
            granted_by: granted_by.into(),
            granted_at: Utc::now(),
            expires_at: None,
            run_id: None,
            attempt_id: None,
            reason_codes: Vec::new(),
        }
    }

    pub fn scoped(
        risk_class: CanonicalToolSideEffectClass,
        tool_id: impl Into<String>,
        sandbox_root: impl Into<String>,
        granted_by: impl Into<String>,
    ) -> Self {
        let tool_id = tool_id.into();
        let sandbox_root = sandbox_root.into();
        Self {
            permit_id: display_only_unstable_id("permit"),
            risk_class,
            tool_id: tool_id.clone(),
            sandbox_root: sandbox_root.clone(),
            scope: format!("tool={tool_id};sandbox={sandbox_root}"),
            granted_by: granted_by.into(),
            granted_at: Utc::now(),
            expires_at: None,
            run_id: None,
            attempt_id: None,
            reason_codes: Vec::new(),
        }
    }

    pub fn for_execution_context(mut self, context: &AidensRunContextV1) -> Self {
        self.run_id = Some(context.run_id.clone());
        self.attempt_id = Some(context.attempt_id.clone());
        self.scope = format!(
            "{};run={};attempt={}",
            self.scope,
            context.run_id.as_str(),
            context.attempt_id.as_str()
        );
        self
    }

    pub fn matches_scope(
        &self,
        risk_class: &CanonicalToolSideEffectClass,
        tool_id: &str,
        sandbox_root: &str,
        run_id: Option<&ArtifactId>,
        attempt_id: Option<&ArtifactId>,
    ) -> bool {
        if &self.risk_class != risk_class {
            return false;
        }
        if self.tool_id != tool_id {
            return false;
        }
        if self.sandbox_root != sandbox_root {
            return false;
        }
        if self
            .run_id
            .as_ref()
            .is_some_and(|grant_run_id| Some(grant_run_id) != run_id)
        {
            return false;
        }
        if self
            .attempt_id
            .as_ref()
            .is_some_and(|grant_attempt_id| Some(grant_attempt_id) != attempt_id)
        {
            return false;
        }
        self.expires_at
            .map(|expires_at| expires_at > Utc::now())
            .unwrap_or(true)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ApprovalRequestV1 {
    pub request_id: ArtifactId,
    pub tool_id: String,
    pub risk_class: CanonicalToolSideEffectClass,
    pub scope: String,
    pub sandbox_root: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub run_id: Option<ArtifactId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attempt_id: Option<ArtifactId>,
    pub reason: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reason_codes: Vec<String>,
    pub requested_at: DateTime<Utc>,
}

impl ApprovalRequestV1 {
    pub fn new(
        tool_id: impl Into<String>,
        risk_class: CanonicalToolSideEffectClass,
        scope: impl Into<String>,
        reason: impl Into<String>,
    ) -> Self {
        let scope = scope.into();
        Self {
            request_id: display_only_unstable_id("approval-request"),
            tool_id: tool_id.into(),
            risk_class,
            sandbox_root: scope.clone(),
            scope,
            run_id: None,
            attempt_id: None,
            reason: reason.into(),
            reason_codes: vec!["approval-required".into()],
            requested_at: Utc::now(),
        }
    }

    pub fn scoped(
        tool_id: impl Into<String>,
        risk_class: CanonicalToolSideEffectClass,
        sandbox_root: impl Into<String>,
        reason: impl Into<String>,
    ) -> Self {
        let tool_id = tool_id.into();
        let sandbox_root = sandbox_root.into();
        Self {
            request_id: display_only_unstable_id("approval-request"),
            tool_id: tool_id.clone(),
            risk_class,
            scope: format!("tool={tool_id};sandbox={sandbox_root}"),
            sandbox_root,
            run_id: None,
            attempt_id: None,
            reason: reason.into(),
            reason_codes: vec!["approval-required".into()],
            requested_at: Utc::now(),
        }
    }

    pub fn for_execution_context(mut self, context: &AidensRunContextV1) -> Self {
        self.run_id = Some(context.run_id.clone());
        self.attempt_id = Some(context.attempt_id.clone());
        self.scope = format!(
            "{};run={};attempt={}",
            self.scope,
            context.run_id.as_str(),
            context.attempt_id.as_str()
        );
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ApprovalDecisionV1 {
    pub decision_id: ArtifactId,
    pub request_id: ArtifactId,
    pub approved: bool,
    pub permit_grant: Option<PermitGrantV1>,
    pub decided_by: String,
    pub decided_at: DateTime<Utc>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reason_codes: Vec<String>,
}

impl ApprovalDecisionV1 {
    pub fn denied(
        request_id: ArtifactId,
        decided_by: impl Into<String>,
        reason: impl Into<String>,
    ) -> Self {
        Self {
            decision_id: display_only_unstable_id("approval-decision"),
            request_id,
            approved: false,
            permit_grant: None,
            decided_by: decided_by.into(),
            decided_at: Utc::now(),
            reason_codes: vec![reason.into()],
        }
    }

    pub fn approved(
        request_id: ArtifactId,
        permit_grant: PermitGrantV1,
        decided_by: impl Into<String>,
    ) -> Self {
        Self {
            decision_id: display_only_unstable_id("approval-decision"),
            request_id,
            approved: true,
            permit_grant: Some(permit_grant),
            decided_by: decided_by.into(),
            decided_at: Utc::now(),
            reason_codes: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct PermitUseReportV1 {
    pub receipt_id: ArtifactId,
    pub kind: ArtifactKindV1,
    pub permit_id: ArtifactId,
    pub tool_id: String,
    pub risk_class: CanonicalToolSideEffectClass,
    pub sandbox_root: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub run_id: Option<ArtifactId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attempt_id: Option<ArtifactId>,
    pub allowed: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reason_codes: Vec<String>,
    pub used_at: DateTime<Utc>,
}

impl PermitUseReportV1 {
    pub fn allowed(
        grant: &PermitGrantV1,
        tool_id: impl Into<String>,
        sandbox_root: impl Into<String>,
        run_id: Option<ArtifactId>,
        attempt_id: Option<ArtifactId>,
    ) -> Self {
        Self {
            receipt_id: display_only_unstable_id("permit-use"),
            kind: ArtifactKindV1::PermitUse,
            permit_id: grant.permit_id.clone(),
            tool_id: tool_id.into(),
            risk_class: grant.risk_class.clone(),
            sandbox_root: sandbox_root.into(),
            run_id,
            attempt_id,
            allowed: true,
            reason_codes: vec!["permit-scope-matched".into()],
            used_at: Utc::now(),
        }
    }

    pub fn denied(
        permit_id: ArtifactId,
        tool_id: impl Into<String>,
        risk_class: CanonicalToolSideEffectClass,
        sandbox_root: impl Into<String>,
        reason: impl Into<String>,
    ) -> Self {
        Self {
            receipt_id: display_only_unstable_id("permit-use"),
            kind: ArtifactKindV1::PermitUse,
            permit_id,
            tool_id: tool_id.into(),
            risk_class,
            sandbox_root: sandbox_root.into(),
            run_id: None,
            attempt_id: None,
            allowed: false,
            reason_codes: vec![reason.into()],
            used_at: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum TurnModeV1 {
    NoTools,
    NativeToolLoop,
    ParserFallback,
    ProviderUnavailable,
    BudgetExhausted,
}

impl fmt::Display for TurnModeV1 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::NoTools => "no-tools",
            Self::NativeToolLoop => "native-tool-loop",
            Self::ParserFallback => "parser-fallback",
            Self::ProviderUnavailable => "provider-unavailable",
            Self::BudgetExhausted => "budget-exhausted",
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum ToolCallSourceV1 {
    NativeProvider,
    ParserFallback,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ToolCallRequestV1 {
    pub call_id: ArtifactId,
    pub source: ToolCallSourceV1,
    pub tool_id: String,
    pub input: serde_json::Value,
    pub input_digest: String,
    #[serde(default)]
    pub degraded: bool,
    pub raw_provider_text: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reason_codes: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub canonical_backpointers: Vec<CanonicalBackpointerV1>,
    pub requested_at: DateTime<Utc>,
}

impl ToolCallRequestV1 {
    pub fn new(
        source: ToolCallSourceV1,
        tool_id: impl Into<String>,
        input: serde_json::Value,
        raw_provider_text: Option<String>,
        reason_codes: Vec<String>,
    ) -> Self {
        Self {
            call_id: display_only_unstable_id("tool-call"),
            source,
            tool_id: tool_id.into(),
            input_digest: json_digest(&input),
            input,
            degraded: source == ToolCallSourceV1::ParserFallback,
            raw_provider_text,
            reason_codes,
            canonical_backpointers: canonical_owner_backpointer(
                "llm-tool-runtime",
                "ToolCall",
                "canonical-tool-call-owner",
            ),
            requested_at: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ToolCallResultV1 {
    pub result_id: ArtifactId,
    pub call_id: ArtifactId,
    pub tool_id: String,
    pub input_digest: String,
    pub output: Option<serde_json::Value>,
    pub output_digest: Option<String>,
    pub succeeded: bool,
    pub invocation_receipt_id: ArtifactId,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reason_codes: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub canonical_backpointers: Vec<CanonicalBackpointerV1>,
    pub completed_at: DateTime<Utc>,
}

impl ToolCallResultV1 {
    pub fn from_invocation(
        request: &ToolCallRequestV1,
        invocation_receipt: &ToolInvocationReportV1,
    ) -> Self {
        Self {
            result_id: display_only_unstable_id("tool-call-result"),
            call_id: request.call_id.clone(),
            tool_id: request.tool_id.clone(),
            input_digest: request.input_digest.clone(),
            output: invocation_receipt.output.clone(),
            output_digest: invocation_receipt.output_digest.clone(),
            succeeded: invocation_receipt.succeeded,
            invocation_receipt_id: invocation_receipt.receipt_id.clone(),
            reason_codes: invocation_receipt.reason_codes.clone(),
            canonical_backpointers: canonical_owner_backpointer(
                "llm-tool-runtime",
                "ToolResult",
                "canonical-tool-result-owner",
            ),
            completed_at: Utc::now(),
        }
    }

    pub fn output_text(&self) -> String {
        self.output
            .as_ref()
            .and_then(|output| output.get("content"))
            .and_then(serde_json::Value::as_str)
            .map(str::to_string)
            .or_else(|| self.output.as_ref().map(serde_json::Value::to_string))
            .unwrap_or_default()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct TurnExecutionPlanV1 {
    pub plan_id: ArtifactId,
    pub mode: TurnModeV1,
    pub provider_route: ProviderRouteReportV1,
    pub tool_exposure_id: ArtifactId,
    pub exposed_tool_ids: Vec<String>,
    pub max_tool_calls: u32,
    pub max_retries: u32,
    pub max_turn_millis: u64,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reason_codes: Vec<String>,
    pub created_at: DateTime<Utc>,
}

impl TurnExecutionPlanV1 {
    pub fn new(
        mode: TurnModeV1,
        provider_route: ProviderRouteReportV1,
        tool_exposure: &ToolExposureSetV1,
        max_tool_calls: u32,
        max_retries: u32,
        max_turn_millis: u64,
        reason_codes: Vec<String>,
    ) -> Self {
        Self {
            plan_id: display_only_unstable_id("turn-plan"),
            mode,
            provider_route,
            tool_exposure_id: tool_exposure.exposure_id.clone(),
            exposed_tool_ids: tool_exposure.exposed_tool_ids.clone(),
            max_tool_calls,
            max_retries,
            max_turn_millis,
            reason_codes,
            created_at: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum TurnFinalStateV1 {
    FinalOutput,
    ProviderUnavailable,
    ToolBlocked,
    ToolFailed,
    BudgetExhausted,
    StopRuleTriggered,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct TurnReportV1 {
    pub receipt_id: ArtifactId,
    pub kind: ArtifactKindV1,
    pub run_id: ArtifactId,
    pub attempt_id: ArtifactId,
    pub plan_id: ArtifactId,
    pub mode: TurnModeV1,
    pub final_state: TurnFinalStateV1,
    #[serde(default)]
    pub degraded: bool,
    #[serde(default)]
    pub blocked: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tool_call_ids: Vec<ArtifactId>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tool_invocation_receipt_ids: Vec<ArtifactId>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub stop_rule_receipt_ids: Vec<ArtifactId>,
    pub budget_exhaustion_receipt_id: Option<ArtifactId>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reason_codes: Vec<String>,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

impl TurnReportV1 {
    pub fn started(context: &AidensRunContextV1, plan: &TurnExecutionPlanV1) -> Self {
        Self {
            receipt_id: display_only_unstable_id("turn"),
            kind: ArtifactKindV1::Turn,
            run_id: context.run_id.clone(),
            attempt_id: context.attempt_id.clone(),
            plan_id: plan.plan_id.clone(),
            mode: plan.mode,
            final_state: TurnFinalStateV1::StopRuleTriggered,
            degraded: false,
            blocked: false,
            tool_call_ids: Vec::new(),
            tool_invocation_receipt_ids: Vec::new(),
            stop_rule_receipt_ids: Vec::new(),
            budget_exhaustion_receipt_id: None,
            reason_codes: plan.reason_codes.clone(),
            started_at: Utc::now(),
            completed_at: None,
        }
    }

    pub fn record_tool_call(
        &mut self,
        request: &ToolCallRequestV1,
        invocation_receipt: &ToolInvocationReportV1,
    ) {
        self.tool_call_ids.push(request.call_id.clone());
        self.tool_invocation_receipt_ids
            .push(invocation_receipt.receipt_id.clone());
        if request.degraded {
            self.degraded = true;
        }
    }

    pub fn record_stop_rule(&mut self, stop_rule: &StopRuleReportV1) {
        self.stop_rule_receipt_ids
            .push(stop_rule.receipt_id.clone());
    }

    pub fn record_budget_exhaustion(&mut self, budget: &BudgetExhaustionReportV1) {
        self.budget_exhaustion_receipt_id = Some(budget.receipt_id.clone());
        self.degraded = true;
        self.blocked = true;
    }

    pub fn complete(mut self, final_state: TurnFinalStateV1) -> Self {
        self.final_state = final_state;
        self.completed_at = Some(Utc::now());
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum StopRuleV1 {
    FinalOutput,
    ProviderUnavailable,
    AgencyPolicy,
    ToolNotExposed,
    ToolInvocationFailed,
    RecursiveToolCall,
    MaxToolCalls,
    MaxRetries,
    DeadlineExceeded,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct StopRuleReportV1 {
    pub receipt_id: ArtifactId,
    pub kind: ArtifactKindV1,
    pub run_id: ArtifactId,
    pub attempt_id: ArtifactId,
    pub rule: StopRuleV1,
    pub triggered: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reason_codes: Vec<String>,
    pub checked_at: DateTime<Utc>,
}

impl StopRuleReportV1 {
    pub fn triggered(
        context: &AidensRunContextV1,
        rule: StopRuleV1,
        reason_codes: Vec<String>,
    ) -> Self {
        Self {
            receipt_id: display_only_unstable_id("stop-rule"),
            kind: ArtifactKindV1::StopRule,
            run_id: context.run_id.clone(),
            attempt_id: context.attempt_id.clone(),
            rule,
            triggered: true,
            reason_codes,
            checked_at: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct BudgetExhaustionReportV1 {
    pub receipt_id: ArtifactId,
    pub kind: ArtifactKindV1,
    pub run_id: ArtifactId,
    pub attempt_id: ArtifactId,
    pub max_tool_calls: u32,
    pub attempted_tool_calls: u32,
    pub max_retries: u32,
    pub retries: u32,
    pub max_turn_millis: u64,
    pub elapsed_millis: u64,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reason_codes: Vec<String>,
    pub exhausted_at: DateTime<Utc>,
}

impl BudgetExhaustionReportV1 {
    pub fn new(draft: BudgetExhaustionReportDraftV1) -> Self {
        Self {
            receipt_id: display_only_unstable_id("budget-exhaustion"),
            kind: ArtifactKindV1::BudgetExhaustion,
            run_id: draft.run_id,
            attempt_id: draft.attempt_id,
            max_tool_calls: draft.max_tool_calls,
            attempted_tool_calls: draft.attempted_tool_calls,
            max_retries: draft.max_retries,
            retries: draft.retries,
            max_turn_millis: draft.max_turn_millis,
            elapsed_millis: draft.elapsed_millis,
            reason_codes: draft.reason_codes,
            exhausted_at: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct BudgetExhaustionReportDraftV1 {
    pub run_id: ArtifactId,
    pub attempt_id: ArtifactId,
    pub max_tool_calls: u32,
    pub attempted_tool_calls: u32,
    pub max_retries: u32,
    pub retries: u32,
    pub max_turn_millis: u64,
    pub elapsed_millis: u64,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reason_codes: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum ToolInvocationOutcomeV1 {
    Started,
    Succeeded,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ToolInvocationReportV1 {
    pub receipt_id: ArtifactId,
    pub kind: ArtifactKindV1,
    pub run_id: Option<ArtifactId>,
    pub attempt_id: Option<ArtifactId>,
    pub tool_id: String,
    pub input: serde_json::Value,
    pub input_digest: String,
    pub output: Option<serde_json::Value>,
    pub output_digest: Option<String>,
    pub succeeded: bool,
    pub outcome: ToolInvocationOutcomeV1,
    pub permit_grant_id: Option<ArtifactId>,
    pub permit_use_receipt_id: Option<ArtifactId>,
    pub approval_decision_id: Option<ArtifactId>,
    pub approval_request_id: Option<ArtifactId>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub schema_validation_receipt_ids: Vec<ArtifactId>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub canonical_backpointers: Vec<CanonicalBackpointerV1>,
    pub lifecycle: Vec<ToolLifecycleStateV1>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reason_codes: Vec<String>,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

pub type ToolAttemptReportV1 = ToolInvocationReportV1;

impl ToolInvocationReportV1 {
    pub fn started(tool_id: impl Into<String>, input: serde_json::Value) -> Self {
        Self {
            receipt_id: display_only_unstable_id("tool-invocation"),
            kind: ArtifactKindV1::ToolInvocation,
            run_id: None,
            attempt_id: None,
            tool_id: tool_id.into(),
            input_digest: json_digest(&input),
            input,
            output: None,
            output_digest: None,
            succeeded: false,
            outcome: ToolInvocationOutcomeV1::Started,
            permit_grant_id: None,
            permit_use_receipt_id: None,
            approval_decision_id: None,
            approval_request_id: None,
            schema_validation_receipt_ids: Vec::new(),
            canonical_backpointers: canonical_owner_backpointer(
                "llm-tool-runtime",
                "ToolReceipt",
                "canonical-tool-receipt-owner",
            ),
            lifecycle: vec![ToolLifecycleStateV1::Invoked],
            reason_codes: Vec::new(),
            started_at: Utc::now(),
            completed_at: None,
        }
    }

    pub fn with_execution_context(mut self, context: &AidensRunContextV1) -> Self {
        self.run_id = Some(context.run_id.clone());
        self.attempt_id = Some(context.attempt_id.clone());
        self
    }

    pub fn with_permit_grant(mut self, permit_grant_id: ArtifactId) -> Self {
        self.permit_grant_id = Some(permit_grant_id);
        self
    }

    pub fn with_permit_use(mut self, permit_use_receipt_id: ArtifactId) -> Self {
        self.permit_use_receipt_id = Some(permit_use_receipt_id);
        self
    }

    pub fn with_approval_request(mut self, approval_request_id: ArtifactId) -> Self {
        self.approval_request_id = Some(approval_request_id);
        self
    }

    pub fn with_approval_decision(mut self, approval_decision_id: ArtifactId) -> Self {
        self.approval_decision_id = Some(approval_decision_id);
        self
    }

    pub fn with_schema_validation(mut self, schema_validation_receipt_id: ArtifactId) -> Self {
        self.schema_validation_receipt_ids
            .push(schema_validation_receipt_id);
        self
    }

    pub fn with_canonical_tool_receipt(mut self, receipt_id: impl Into<String>) -> Self {
        self.canonical_backpointers
            .push(CanonicalBackpointerV1::external(
                "llm-tool-runtime",
                "ToolReceipt",
                "canonical-tool-receipt-id",
                receipt_id,
            ));
        self
    }

    pub fn complete_success(mut self, output: serde_json::Value) -> Self {
        self.output_digest = Some(json_digest(&output));
        self.output = Some(output);
        self.succeeded = true;
        self.outcome = ToolInvocationOutcomeV1::Succeeded;
        self.lifecycle.push(ToolLifecycleStateV1::Succeeded);
        self.completed_at = Some(Utc::now());
        self
    }

    pub fn complete_failure(mut self, reason: impl Into<String>) -> Self {
        self.reason_codes.push(reason.into());
        self.succeeded = false;
        self.outcome = ToolInvocationOutcomeV1::Failed;
        self.lifecycle.push(ToolLifecycleStateV1::Failed);
        self.completed_at = Some(Utc::now());
        self
    }

    pub fn complete_failure_with_output(
        mut self,
        reason: impl Into<String>,
        output: serde_json::Value,
    ) -> Self {
        self.output_digest = Some(json_digest(&output));
        self.output = Some(output);
        self.complete_failure(reason)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct RunReportV1 {
    pub receipt_id: ArtifactId,
    pub kind: ArtifactKindV1,
    pub context: AidensRunContextV1,
    pub provider_route: Option<ProviderRouteReportV1>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tool_exposure_ids: Vec<ArtifactId>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub turn_receipts: Vec<TurnReportV1>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tool_call_requests: Vec<ToolCallRequestV1>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tool_call_results: Vec<ToolCallResultV1>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tool_invocation_receipts: Vec<ToolInvocationReportV1>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub approval_requests: Vec<ApprovalRequestV1>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub approval_decisions: Vec<ApprovalDecisionV1>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub permit_use_receipts: Vec<PermitUseReportV1>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub boundary_repair_receipts: Vec<BoundaryRepairReportV1>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub json_repair_receipts: Vec<JsonBoundaryRepairDisplayReportV1>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub schema_validation_receipts: Vec<SchemaValidationReportV1>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub stop_rule_receipts: Vec<StopRuleReportV1>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub budget_exhaustion_receipts: Vec<BudgetExhaustionReportV1>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub agency_receipt_ids: Vec<String>,
    pub warnings: Vec<String>,
    pub completed_at: Option<DateTime<Utc>>,
}

impl RunReportV1 {
    pub fn started(context: AidensRunContextV1) -> Self {
        Self {
            receipt_id: display_only_unstable_id("receipt"),
            kind: ArtifactKindV1::Run,
            context,
            provider_route: None,
            tool_exposure_ids: Vec::new(),
            turn_receipts: Vec::new(),
            tool_call_requests: Vec::new(),
            tool_call_results: Vec::new(),
            tool_invocation_receipts: Vec::new(),
            approval_requests: Vec::new(),
            approval_decisions: Vec::new(),
            permit_use_receipts: Vec::new(),
            boundary_repair_receipts: Vec::new(),
            json_repair_receipts: Vec::new(),
            schema_validation_receipts: Vec::new(),
            stop_rule_receipts: Vec::new(),
            budget_exhaustion_receipts: Vec::new(),
            agency_receipt_ids: Vec::new(),
            warnings: Vec::new(),
            completed_at: None,
        }
    }

    pub fn complete(mut self) -> Self {
        self.completed_at = Some(Utc::now());
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum AiDENsRunFailureClassV1 {
    None,
    ProviderUnavailable,
    ToolBlocked,
    ToolFailed,
    BudgetExhausted,
    VerificationUnavailable,
    ReplayMismatch,
    OperatorAbstained,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct AiDENsRunEventLogDigestV1 {
    pub event_log_path: String,
    pub digest: StackContentDigest,
    pub replay_normalized_digest: StackContentDigest,
    pub canonical_record_count: usize,
    pub event_count: usize,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reason_codes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct AiDENsRunBudgetDeadlineV1 {
    pub max_steps: u32,
    pub max_tool_calls: u32,
    pub max_retries: u32,
    pub max_turn_millis: u64,
    pub elapsed_ms: i64,
    pub deadline: Option<String>,
    pub cost_budget_units: Option<u64>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub degradation_markers: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct AiDENsRunReplayNormalizationV1 {
    pub replay_command: String,
    pub fixture_path: Option<String>,
    pub normalized_fields: Vec<String>,
    pub deterministic_compare: bool,
    pub normalized_digest: StackContentDigest,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reason_codes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct AiDENsRunFailureTaxonomyV1 {
    pub class: AiDENsRunFailureClassV1,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reason_codes: Vec<String>,
    pub degraded: bool,
    pub blocked: bool,
}
