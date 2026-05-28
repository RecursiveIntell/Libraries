//! Agent spec and run-bundle envelopes.
//!
//! These structures package AiDENs-local run evidence while preserving canonical execution and receipt backpointers.

use super::*;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum AgentSpecSupportLabelV1 {
    Supported,
    SupportedLocal,
    Partial,
    Deferred,
    Unsupported,
    Scaffold,
    Failed,
}

impl fmt::Display for AgentSpecSupportLabelV1 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Supported => "supported",
            Self::SupportedLocal => "supported-local",
            Self::Partial => "partial",
            Self::Deferred => "deferred",
            Self::Unsupported => "unsupported",
            Self::Scaffold => "scaffold",
            Self::Failed => "failed",
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum AgentProviderModeV1 {
    Mock,
    Local,
}

impl fmt::Display for AgentProviderModeV1 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Mock => "mock",
            Self::Local => "local",
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum AgentMemoryModeV1 {
    Fixture,
    CanonicalSeam,
}

impl fmt::Display for AgentMemoryModeV1 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Fixture => "fixture",
            Self::CanonicalSeam => "canonical-seam",
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum AgentPermitRuleV1 {
    OperatorApproved,
    Forbidden,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum AgentVerificationCheckV1 {
    Schema,
    Sandbox,
    Digest,
    SupportClaim,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct AgentSpecProviderPolicyV1 {
    pub provider: AgentProviderModeV1,
    pub cloud_allowed: bool,
    pub fallback_allowed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct AgentSpecMemoryPolicyV1 {
    pub enabled: bool,
    pub mode: AgentMemoryModeV1,
    pub requires_view_disclosure: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct AgentSpecToolPolicyV1 {
    pub allowed_tools: Vec<String>,
    pub write_tools_require_permit: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct AgentSpecPermitPolicyV1 {
    pub writes: AgentPermitRuleV1,
    pub commands: AgentPermitRuleV1,
    pub network: AgentPermitRuleV1,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct AgentSpecVerificationPolicyV1 {
    pub required_checks: Vec<AgentVerificationCheckV1>,
    pub fail_closed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct AgentSpecEvidencePolicyV1 {
    pub emit_run_bundle: bool,
    pub emit_tool_receipts: bool,
    pub emit_permit_receipts: bool,
    pub emit_abstention_receipts: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct AgentSpecBudgetPolicyV1 {
    pub max_turns: u32,
    pub max_tool_calls: u32,
    pub deadline_seconds: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct AgentSpecV1 {
    pub schema: String,
    pub agent_id: String,
    pub display_name: String,
    pub support_label: AgentSpecSupportLabelV1,
    pub profile: String,
    pub provider_policy: AgentSpecProviderPolicyV1,
    pub memory_policy: AgentSpecMemoryPolicyV1,
    pub tool_policy: AgentSpecToolPolicyV1,
    pub permit_policy: AgentSpecPermitPolicyV1,
    pub verification_policy: AgentSpecVerificationPolicyV1,
    pub evidence_policy: AgentSpecEvidencePolicyV1,
    pub budget_policy: AgentSpecBudgetPolicyV1,
}

impl AgentSpecV1 {
    pub const SCHEMA: &'static str = "AgentSpecV1";

    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut reason_codes = Vec::new();

        if self.schema.trim() != Self::SCHEMA {
            reason_codes.push("schema-must-equal-AgentSpecV1".into());
        }
        if self.agent_id.trim().is_empty() {
            reason_codes.push("agent-id-required".into());
        }
        if self.display_name.trim().is_empty() {
            reason_codes.push("display-name-required".into());
        }
        if self.profile.trim().is_empty() {
            reason_codes.push("profile-required".into());
        } else {
            let normalized = self.profile.to_ascii_lowercase();
            if !matches!(
                normalized.as_str(),
                "coding" | "memory" | "research" | "custom-local"
            ) {
                reason_codes.push("unsupported-profile".into());
            }
        }
        if self.provider_policy.cloud_allowed {
            reason_codes.push("cloud-providers-unsupported-for-p26-local-agent".into());
        }
        if self.tool_policy.allowed_tools.is_empty() {
            reason_codes.push("tool-policy-must-list-tools".into());
        } else {
            const ALLOWED_TOOLS: [&str; 8] = [
                "repo.read",
                "repo.list",
                "repo.search",
                "patch.propose",
                "patch.apply",
                "checks.run",
                "run.inspect",
                "run.replay",
            ];
            for tool in &self.tool_policy.allowed_tools {
                if !ALLOWED_TOOLS.contains(&tool.as_str()) {
                    reason_codes.push(format!("unsupported-tool:{tool}"));
                }
            }
        }

        const WRITE_TOOLS_REQUIRE_PERMIT: [&str; 3] = ["patch.apply", "checks.run", "repo.write"];
        let write_tools_present = self
            .tool_policy
            .allowed_tools
            .iter()
            .any(|tool| WRITE_TOOLS_REQUIRE_PERMIT.contains(&tool.as_str()));
        if write_tools_present && !self.tool_policy.write_tools_require_permit {
            reason_codes.push("write-tools-must-require-permit".into());
        }
        if write_tools_present
            && !matches!(
                self.permit_policy.writes,
                AgentPermitRuleV1::OperatorApproved
            )
        {
            reason_codes.push("write-permit-policy-must-be-operator-approved".into());
        }
        if matches!(
            self.permit_policy.network,
            AgentPermitRuleV1::OperatorApproved
        ) {
            reason_codes.push("network-access-must-be-forbidden-for-local-run".into());
        }
        if self.memory_policy.enabled {
            if !(self.memory_policy.requires_view_disclosure
                || self.memory_policy.mode == AgentMemoryModeV1::CanonicalSeam)
            {
                reason_codes.push("canonical-memory-grounding-must-disclose-or-be-disabled".into());
            }
        } else if self.memory_policy.requires_view_disclosure {
            reason_codes.push("requires_view_disclosure-when-memory-disabled".into());
        }
        if !self.evidence_policy.emit_tool_receipts {
            reason_codes.push("must-emit-tool-receipts".into());
        }
        if !self.evidence_policy.emit_permit_receipts {
            reason_codes.push("must-emit-permit-receipts".into());
        }
        if !self.evidence_policy.emit_abstention_receipts {
            reason_codes.push("must-emit-abstention-receipts".into());
        }
        if !self.evidence_policy.emit_run_bundle {
            reason_codes.push("must-emit-run-bundle".into());
        }
        if self.verification_policy.required_checks.is_empty() {
            reason_codes.push("verification-required-checks-missing".into());
        }
        if !self.verification_policy.fail_closed {
            reason_codes.push("verification-fail-closed-required".into());
        }
        if self.budget_policy.max_turns == 0 {
            reason_codes.push("max-turns-must-be-positive".into());
        }
        if self.budget_policy.max_tool_calls == 0 {
            reason_codes.push("max-tool-calls-must-be-positive".into());
        }
        if self.budget_policy.deadline_seconds == 0 {
            reason_codes.push("deadline-seconds-must-be-positive".into());
        }

        if reason_codes.is_empty() {
            Ok(())
        } else {
            Err(reason_codes)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct AiDENsRunSupportTierEvidenceV1 {
    pub support_tier: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub supported: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub partial: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub deferred: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reason_codes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct AiDENsRunBundleV2 {
    pub schema: String,
    pub bundle_id: ArtifactId,
    pub ownership: String,
    pub run_id: String,
    pub profile: String,
    pub created_at: DateTime<Utc>,
    pub canonical_execution_context: canonical_stack::ForgeExecutionContextV1,
    pub trace_ctx: StackTraceCtx,
    pub attempt_id: StackAttemptId,
    pub trial_id: StackTrialId,
    pub event_log: AiDENsRunEventLogDigestV1,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub provider_receipts: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tool_receipts: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub permit_receipts: Vec<String>,
    pub budget: AiDENsRunBudgetDeadlineV1,
    pub support: AiDENsRunSupportTierEvidenceV1,
    pub replay: AiDENsRunReplayNormalizationV1,
    pub failure: AiDENsRunFailureTaxonomyV1,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub outputs: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub canonical_backpointers: Vec<CanonicalBackpointerV1>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub blocked_checks: Vec<String>,
}

impl AiDENsRunBundleV2 {
    pub const SCHEMA: &'static str = "AiDENsRunBundleV2";

    #[allow(clippy::too_many_arguments)]
    pub fn new(
        run_id: impl Into<String>,
        profile: impl Into<String>,
        canonical_execution_context: canonical_stack::ForgeExecutionContextV1,
        event_log: AiDENsRunEventLogDigestV1,
        budget: AiDENsRunBudgetDeadlineV1,
        support: AiDENsRunSupportTierEvidenceV1,
        replay: AiDENsRunReplayNormalizationV1,
        failure: AiDENsRunFailureTaxonomyV1,
    ) -> Self {
        let trace_ctx = canonical_execution_context.trace_ctx.clone();
        let attempt_id = canonical_execution_context
            .attempt_id
            .clone()
            .unwrap_or_else(StackAttemptId::generate);
        let trial_id = canonical_execution_context
            .trial_id
            .clone()
            .unwrap_or_else(StackTrialId::generate);
        Self {
            schema: Self::SCHEMA.into(),
            bundle_id: display_only_unstable_id("aidens-run-bundle-v2"),
            ownership: "AiDENs-local operator run bundle; canonical execution, trace, identity, receipt, memory, and verification semantics remain in owner crates.".into(),
            run_id: run_id.into(),
            profile: profile.into(),
            created_at: Utc::now(),
            canonical_execution_context,
            trace_ctx,
            attempt_id,
            trial_id,
            event_log,
            provider_receipts: Vec::new(),
            tool_receipts: Vec::new(),
            permit_receipts: Vec::new(),
            budget,
            support,
            replay,
            failure,
            outputs: Vec::new(),
            canonical_backpointers: vec![
                CanonicalBackpointerV1::owner_type(
                    "semantic-memory-forge",
                    "ExecutionContextV1",
                    "canonical-execution-context-owner",
                ),
                CanonicalBackpointerV1::owner_type(
                    "stack-ids",
                    "TraceCtx",
                    "canonical-trace-owner",
                ),
                CanonicalBackpointerV1::owner_type(
                    "stack-ids",
                    "AttemptId",
                    "canonical-attempt-owner",
                ),
                CanonicalBackpointerV1::owner_type(
                    "stack-ids",
                    "TrialId",
                    "canonical-trial-owner",
                ),
                CanonicalBackpointerV1::owner_type(
                    "llm-tool-runtime",
                    "ToolReceipt",
                    "canonical-tool-receipt-owner",
                ),
            ],
            blocked_checks: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct AiDENsRunBundleV3 {
    pub schema: String,
    pub bundle_id: ArtifactId,
    pub ownership: String,
    pub run_id: String,
    pub profile: String,
    pub created_at: DateTime<Utc>,
    pub canonical_execution_context: canonical_stack::ForgeExecutionContextV1,
    pub trace_ctx: StackTraceCtx,
    pub attempt_family_id: ArtifactId,
    pub attempt_id: StackAttemptId,
    pub trial_id: StackTrialId,
    pub event_log: AiDENsRunEventLogDigestV1,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub provider_receipts: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tool_receipts: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub permit_receipts: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub memory_grounding_receipts: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub verification_receipts: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub abstention_receipts: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub repair_plan_receipts: Vec<String>,
    pub agent_spec_digest: DisplayDigestV1,
    pub budget: AiDENsRunBudgetDeadlineV1,
    pub support: AiDENsRunSupportTierEvidenceV1,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub support_labels: Vec<String>,
    pub replay: AiDENsRunReplayNormalizationV1,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub replay_instructions: Vec<String>,
    pub failure: AiDENsRunFailureTaxonomyV1,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub outputs: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub canonical_backpointers: Vec<CanonicalBackpointerV1>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub blocked_checks: Vec<String>,
}

impl AiDENsRunBundleV3 {
    pub const SCHEMA: &'static str = "AiDENsRunBundleV3";

    #[allow(clippy::too_many_arguments)]
    pub fn new(
        run_id: impl Into<String>,
        profile: impl Into<String>,
        canonical_execution_context: canonical_stack::ForgeExecutionContextV1,
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
    ) -> Self {
        let trace_ctx = canonical_execution_context.trace_ctx.clone();
        Self {
            schema: Self::SCHEMA.into(),
            bundle_id: display_only_unstable_id("aidens-run-bundle-v3"),
            ownership:
                "AiDENs-local operator run bundle; canonical execution, trace, identity, receipt, memory, and verification semantics remain in owner crates.".into(),
            run_id: run_id.into(),
            profile: profile.into(),
            created_at: Utc::now(),
            canonical_execution_context,
            trace_ctx,
            attempt_family_id,
            attempt_id,
            trial_id,
            event_log,
            provider_receipts: Vec::new(),
            tool_receipts: Vec::new(),
            permit_receipts: Vec::new(),
            memory_grounding_receipts: Vec::new(),
            verification_receipts: Vec::new(),
            abstention_receipts: Vec::new(),
            repair_plan_receipts: Vec::new(),
            agent_spec_digest,
            budget,
            support,
            support_labels,
            replay,
            replay_instructions: Vec::new(),
            failure,
            outputs: Vec::new(),
            canonical_backpointers: vec![
                CanonicalBackpointerV1::owner_type(
                    "semantic-memory-forge",
                    "ExecutionContextV1",
                    "canonical-execution-context-owner",
                ),
                CanonicalBackpointerV1::owner_type(
                    "stack-ids",
                    "TraceCtx",
                    "canonical-trace-owner",
                ),
                CanonicalBackpointerV1::owner_type(
                    "stack-ids",
                    "AttemptFamilyId",
                    "canonical-attempt-family-owner",
                ),
                CanonicalBackpointerV1::owner_type(
                    "stack-ids",
                    "AttemptId",
                    "canonical-attempt-owner",
                ),
                CanonicalBackpointerV1::owner_type(
                    "stack-ids",
                    "TrialId",
                    "canonical-trial-owner",
                ),
                CanonicalBackpointerV1::owner_type(
                    "llm-tool-runtime",
                    "ToolReceipt",
                    "canonical-tool-receipt-owner",
                ),
            ],
            blocked_checks: Vec::new(),
        }
    }
}
