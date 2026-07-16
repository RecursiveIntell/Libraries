//! Execution context and invocation receipt envelopes for material operations.
//!
//! The owner plane is AiDENs orchestration evidence. Tool/runtime authority
//! remains delegated to the canonical runtime and tool crates.

use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum ExecutionCompletionStateV1 {
    Started,
    Succeeded,
    Failed,
    TimedOut,
    Cancelled,
    Partial,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ExecutionContextRefV1 {
    pub execution_id: ArtifactId,
    pub trace_id: ArtifactId,
    pub operation_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ExecutionContextEnvelopeV1 {
    pub execution_id: ArtifactId,
    pub trace_id: ArtifactId,
    pub span_id: String,
    pub operation_id: String,
    pub attempt_family_id: ArtifactId,
    pub retry_family_id: ArtifactId,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub queue_lineage: Vec<String>,
    pub provider_route: String,
    pub tool_route: String,
    pub environment_fingerprint: String,
    pub started_at: DateTime<Utc>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<DateTime<Utc>>,
    pub budget_millis_allocated: u64,
    pub budget_millis_consumed: u64,
    pub deadline_status: String,
    pub completion_state: ExecutionCompletionStateV1,
    pub replay_handle: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub non_replayability_reason: Option<String>,
    pub redaction_state: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub degradation_refs: Vec<ArtifactId>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reason_codes: Vec<String>,
}

impl ExecutionContextEnvelopeV1 {
    pub fn local_started(
        operation_id: impl Into<String>,
        attempt_family_id: ArtifactId,
        provider_route: impl Into<String>,
        tool_route: impl Into<String>,
    ) -> Self {
        let operation_id = operation_id.into();
        let started_at = Utc::now();
        let environment_fingerprint = format!(
            "crate:{}@{};os:{};arch:{};family:{}",
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_VERSION"),
            std::env::consts::OS,
            std::env::consts::ARCH,
            std::env::consts::FAMILY
        );
        let material = format!(
            "{}|{}|{}",
            operation_id,
            attempt_family_id.as_str(),
            started_at.to_rfc3339_opts(SecondsFormat::Nanos, true)
        );
        Self {
            execution_id: generated_artifact_id_from_material("execution-context", &material),
            trace_id: generated_artifact_id_from_material("trace", &material),
            span_id: generated_artifact_id_from_material("span", &material)
                .as_str()
                .to_string(),
            operation_id: operation_id.clone(),
            attempt_family_id: attempt_family_id.clone(),
            retry_family_id: generated_artifact_id_from_material(
                "retry-family",
                &format!("retry|{}|{}", operation_id, attempt_family_id.as_str()),
            ),
            queue_lineage: Vec::new(),
            provider_route: provider_route.into(),
            tool_route: tool_route.into(),
            environment_fingerprint,
            started_at,
            completed_at: None,
            budget_millis_allocated: 0,
            budget_millis_consumed: 0,
            deadline_status: "not-declared".into(),
            completion_state: ExecutionCompletionStateV1::Started,
            replay_handle: "local-replay-handle-deferred".into(),
            non_replayability_reason: None,
            redaction_state: "unredacted-local-fixture".into(),
            degradation_refs: Vec::new(),
            reason_codes: vec!["execution-context-started".into()],
        }
    }

    pub fn as_ref(&self) -> ExecutionContextRefV1 {
        ExecutionContextRefV1 {
            execution_id: self.execution_id.clone(),
            trace_id: self.trace_id.clone(),
            operation_id: self.operation_id.clone(),
        }
    }

    pub fn complete(mut self, state: ExecutionCompletionStateV1, consumed_millis: u64) -> Self {
        self.completed_at = Some(Utc::now());
        self.completion_state = state;
        self.budget_millis_consumed = consumed_millis;
        if matches!(
            state,
            ExecutionCompletionStateV1::TimedOut | ExecutionCompletionStateV1::Partial
        ) {
            self.deadline_status = "partial-or-timeout".into();
            self.reason_codes.push("execution-output-partial".into());
        } else {
            self.deadline_status = "completed".into();
        }
        self.reason_codes.push("execution-context-completed".into());
        self.reason_codes.sort();
        self.reason_codes.dedup();
        self
    }

    pub fn terminal_budget_is_enforced(&self) -> bool {
        if self.completed_at.is_none()
            || matches!(self.completion_state, ExecutionCompletionStateV1::Started)
        {
            return false;
        }
        if self.budget_millis_allocated == 0
            || self.budget_millis_consumed <= self.budget_millis_allocated
        {
            return true;
        }
        matches!(
            self.completion_state,
            ExecutionCompletionStateV1::TimedOut | ExecutionCompletionStateV1::Partial
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ToolCallReceiptV1 {
    pub receipt_id: ArtifactId,
    pub execution_context_ref: ExecutionContextRefV1,
    pub tool_id: String,
    pub input_digest: DisplayDigestV1,
    pub output_digest: DisplayDigestV1,
    pub completion_state: ExecutionCompletionStateV1,
    pub started_at: DateTime<Utc>,
    pub completed_at: DateTime<Utc>,
    pub partial_output: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub degradation_refs: Vec<ArtifactId>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reason_codes: Vec<String>,
}

impl ToolCallReceiptV1 {
    pub fn new(
        context: &ExecutionContextEnvelopeV1,
        tool_id: impl Into<String>,
        input: &serde_json::Value,
        output: &serde_json::Value,
        completion_state: ExecutionCompletionStateV1,
    ) -> Self {
        let tool_id = tool_id.into();
        let input_digest = DisplayDigestV1::for_json_value(input);
        let output_digest = DisplayDigestV1::for_json_value(output);
        let completed_at = Utc::now();
        let material = format!(
            "{}|{}|{}|{}|{}",
            context.execution_id.as_str(),
            tool_id,
            input_digest.digest,
            output_digest.digest,
            completed_at.to_rfc3339_opts(SecondsFormat::Nanos, true)
        );
        let partial_output = matches!(
            completion_state,
            ExecutionCompletionStateV1::TimedOut | ExecutionCompletionStateV1::Partial
        );
        Self {
            receipt_id: generated_artifact_id_from_material("tool-call-receipt", &material),
            execution_context_ref: context.as_ref(),
            tool_id,
            input_digest,
            output_digest,
            completion_state,
            started_at: context.started_at,
            completed_at,
            partial_output,
            degradation_refs: Vec::new(),
            reason_codes: if partial_output {
                vec!["tool-output-partial-or-timeout".into()]
            } else {
                vec!["tool-call-receipted".into()]
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct OperatorInvocationReceiptV1 {
    pub receipt_id: ArtifactId,
    pub operator_id: String,
    pub execution_context_ref: ExecutionContextRefV1,
    pub input_manifest: ArtifactManifestV1,
    pub output_manifest: ArtifactManifestV1,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tool_call_receipt_refs: Vec<ArtifactId>,
    pub completion_state: ExecutionCompletionStateV1,
    pub material_done: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub proof_debt_refs: Vec<ArtifactId>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub degradation_refs: Vec<ArtifactId>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reason_codes: Vec<String>,
    pub recorded_at: DateTime<Utc>,
}

impl OperatorInvocationReceiptV1 {
    pub fn material_done(
        operator_id: impl Into<String>,
        context: &ExecutionContextEnvelopeV1,
        input_manifest: ArtifactManifestV1,
        output_manifest: ArtifactManifestV1,
        tool_call_receipt_refs: Vec<ArtifactId>,
    ) -> Result<Self, String> {
        let operator_id = operator_id.into();
        if tool_call_receipt_refs.is_empty() {
            return Err("material operation done state requires at least one receipt ref".into());
        }
        if !input_manifest.complete() || !output_manifest.complete() {
            return Err(
                "material operation done state requires complete input/output manifests".into(),
            );
        }
        let material = format!(
            "{}|{}|{}|{}",
            operator_id,
            context.execution_id.as_str(),
            input_manifest.manifest_id.as_str(),
            output_manifest.manifest_id.as_str()
        );
        Ok(Self {
            receipt_id: generated_artifact_id_from_material("operator-invocation", &material),
            operator_id,
            execution_context_ref: context.as_ref(),
            input_manifest,
            output_manifest,
            tool_call_receipt_refs,
            completion_state: ExecutionCompletionStateV1::Succeeded,
            material_done: true,
            proof_debt_refs: Vec::new(),
            degradation_refs: Vec::new(),
            reason_codes: vec!["material-operation-done-with-receipts".into()],
            recorded_at: Utc::now(),
        })
    }
}
