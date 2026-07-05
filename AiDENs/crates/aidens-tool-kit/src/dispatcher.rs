use crate::executors::{
    file_stat, patch_apply, patch_propose, repo_list, repo_read, repo_search, run_checks,
};
use crate::registry::{validate_tool_input_with_canonical_runtime, ToolExecutorV1, ToolRegistryV1};
use aidens_contracts::{
    ApprovalRequestV1, CanonicalToolSideEffectClass, PermitUseReportV1, SchemaValidationReportV1,
    ToolDescriptorV1, ToolInvocationReportV1,
};
use aidens_permit_kit::{requires_permit, PermitCheckContextV1, PermitDecisionV1, PermitPolicyV1};
use serde_json::Value;
use std::fmt;

#[derive(Debug, Clone)]
pub struct ToolDispatcher {
    registry: ToolRegistryV1,
    permit_policy: PermitPolicyV1,
}

impl ToolDispatcher {
    pub fn new(registry: ToolRegistryV1) -> Self {
        Self {
            registry,
            permit_policy: PermitPolicyV1::default(),
        }
    }

    pub fn with_permit_policy(mut self, permit_policy: PermitPolicyV1) -> Self {
        self.permit_policy = permit_policy;
        self
    }

    pub async fn invoke(
        &self,
        tool_id: &str,
        input: Value,
    ) -> anyhow::Result<ToolInvocationOutcome> {
        let Some(descriptor) = self.registry.descriptor(tool_id) else {
            let receipt = ToolInvocationReportV1::started(tool_id, input)
                .complete_failure("tool-not-registered");
            return Err(ToolInvocationError::new(
                format!("tool '{tool_id}' is not registered"),
                receipt,
            )
            .into());
        };

        let mut receipt = ToolInvocationReportV1::started(tool_id, input.clone());
        let schema_validation = validate_tool_input_with_canonical_runtime(descriptor, &input);
        if !schema_validation.valid {
            let failed = receipt
                .with_schema_validation(schema_validation.receipt_id.clone())
                .complete_failure("schema-validation-failed");
            return Err(ToolInvocationError::new(
                format!("tool '{tool_id}' input failed schema validation"),
                failed,
            )
            .with_schema_validation_receipt(schema_validation)
            .into());
        }

        let sandbox_scope = self.sandbox_scope_for(tool_id);
        let permit_context = PermitCheckContextV1::new(
            tool_id.to_string(),
            descriptor.risk_class.clone(),
            sandbox_scope.clone(),
        );
        let mut permit_use_receipt = None;
        if let Some(grant) = self.permit_policy.grant_for_context(&permit_context) {
            receipt = receipt.with_permit_grant(grant.permit_id.clone());
            let use_receipt = PermitUseReportV1::allowed(
                grant,
                tool_id.to_string(),
                sandbox_scope.clone(),
                None,
                None,
            );
            receipt = receipt.with_permit_use(use_receipt.receipt_id.clone());
            permit_use_receipt = Some(use_receipt);
        }

        if requires_permit(&descriptor.risk_class) {
            match self.permit_policy.decision_for_context(&permit_context) {
                PermitDecisionV1::Allow => {}
                PermitDecisionV1::RequiresApproval => {
                    let approval_request = self
                        .permit_policy
                        .approval_request_for_context(&permit_context)
                        .unwrap_or_else(|| {
                            ApprovalRequestV1::scoped(
                                tool_id.to_string(),
                                descriptor.risk_class.clone(),
                                sandbox_scope.clone(),
                                "side-effect tool requires explicit scoped permit",
                            )
                        });
                    let failed = receipt
                        .with_approval_request(approval_request.request_id.clone())
                        .complete_failure(format!("permit-required:{}", descriptor.risk_class));
                    return Err(ToolInvocationError::new(
                        format!(
                            "tool '{tool_id}' requires explicit permit for {}",
                            descriptor.risk_class
                        ),
                        failed,
                    )
                    .with_approval_request(approval_request)
                    .into());
                }
                PermitDecisionV1::Deny(reason) => {
                    let failed = receipt.complete_failure(format!("permit-denied:{reason}"));
                    return Err(ToolInvocationError::new(
                        format!("tool '{tool_id}' permit denied: {reason}"),
                        failed,
                    )
                    .into());
                }
            }
        }

        let Some(executor) = self.registry.executors.get(tool_id) else {
            let failed = receipt.complete_failure("tool-executor-missing");
            return Err(ToolInvocationError::new(
                format!("tool '{tool_id}' is registered but not executable this turn"),
                failed,
            )
            .into());
        };

        match executor {
            ToolExecutorV1::RepoRead { sandbox_root } => {
                if !descriptor.read_only
                    || !matches!(
                        &descriptor.risk_class,
                        CanonicalToolSideEffectClass::ReadOnly
                    )
                {
                    let failed = receipt.complete_failure("read-only-executor-risk-mismatch");
                    return Err(ToolInvocationError::new(
                        format!("tool '{tool_id}' is blocked by read-only executor policy"),
                        failed,
                    )
                    .into());
                }
                self.invoke_executor(tool_id, receipt, permit_use_receipt, || {
                    repo_read(sandbox_root, &input)
                })
            }
            ToolExecutorV1::RepoList { sandbox_root } => self.invoke_read_only_executor(
                tool_id,
                descriptor,
                receipt,
                permit_use_receipt,
                || repo_list(sandbox_root, &input),
            ),
            ToolExecutorV1::FileStat { sandbox_root } => self.invoke_read_only_executor(
                tool_id,
                descriptor,
                receipt,
                permit_use_receipt,
                || file_stat(sandbox_root, &input),
            ),
            ToolExecutorV1::RepoSearch { sandbox_root } => self.invoke_read_only_executor(
                tool_id,
                descriptor,
                receipt,
                permit_use_receipt,
                || repo_search(sandbox_root, &input),
            ),
            ToolExecutorV1::PatchPropose { sandbox_root } => self.invoke_read_only_executor(
                tool_id,
                descriptor,
                receipt,
                permit_use_receipt,
                || patch_propose(sandbox_root, &input),
            ),
            ToolExecutorV1::PatchApply { sandbox_root } => {
                let permit_grant_id = receipt.permit_grant_id.clone();
                let permit_use_receipt_id = receipt.permit_use_receipt_id.clone();
                self.invoke_executor(tool_id, receipt, permit_use_receipt, || {
                    patch_apply(sandbox_root, &input, permit_grant_id, permit_use_receipt_id)
                })
            }
            ToolExecutorV1::RunChecks { sandbox_root } => {
                let permit_grant_id = receipt.permit_grant_id.clone();
                let permit_use_receipt_id = receipt.permit_use_receipt_id.clone();
                self.invoke_executor(tool_id, receipt, permit_use_receipt, || {
                    run_checks(sandbox_root, &input, permit_grant_id, permit_use_receipt_id)
                })
            }
            ToolExecutorV1::Custom(handle) => {
                let result = handle.inner.execute(tool_id, input.clone()).await;
                match result {
                    Ok(output_text) => {
                        let output_val = serde_json::json!({ "content": output_text });
                        receipt = receipt.complete_success(output_val.clone());
                        Ok(ToolInvocationOutcome {
                            output: output_val,
                            receipt,
                            permit_use_receipt,
                        })
                    }
                    Err(e) => {
                        let failed =
                            receipt.complete_failure(format!("custom-executor-failed: {e}"));
                        Err(ToolInvocationError::new(
                            format!("custom executor for '{tool_id}' failed: {e}"),
                            failed,
                        )
                        .into())
                    }
                }
            }
        }
    }

    fn invoke_read_only_executor(
        &self,
        tool_id: &str,
        descriptor: &ToolDescriptorV1,
        receipt: ToolInvocationReportV1,
        permit_use_receipt: Option<PermitUseReportV1>,
        execute: impl FnOnce() -> anyhow::Result<Value>,
    ) -> anyhow::Result<ToolInvocationOutcome> {
        if !descriptor.read_only
            || !matches!(
                &descriptor.risk_class,
                CanonicalToolSideEffectClass::ReadOnly
            )
        {
            let failed = receipt.complete_failure("read-only-executor-risk-mismatch");
            return Err(ToolInvocationError::new(
                format!("tool '{tool_id}' is blocked by read-only executor policy"),
                failed,
            )
            .into());
        }
        self.invoke_executor(tool_id, receipt, permit_use_receipt, execute)
    }

    fn invoke_executor(
        &self,
        tool_id: &str,
        receipt: ToolInvocationReportV1,
        permit_use_receipt: Option<PermitUseReportV1>,
        execute: impl FnOnce() -> anyhow::Result<Value>,
    ) -> anyhow::Result<ToolInvocationOutcome> {
        match execute() {
            Ok(output) => {
                let receipt = receipt.complete_success(output.clone());
                Ok(ToolInvocationOutcome {
                    output,
                    receipt,
                    permit_use_receipt,
                })
            }
            Err(error) => {
                let failed = if let Some(receipt_failure) =
                    error.downcast_ref::<ReceiptBearingToolFailure>()
                {
                    receipt.complete_failure_with_output(
                        receipt_failure.reason_code.clone(),
                        receipt_failure.output.clone(),
                    )
                } else {
                    let reason = executor_reason_code(error.to_string());
                    receipt.complete_failure(reason)
                };
                Err(ToolInvocationError::new(
                    format!("tool '{tool_id}' execution failed: {error}"),
                    failed,
                )
                .into())
            }
        }
    }

    fn sandbox_scope_for(&self, tool_id: &str) -> String {
        const UNKNOWN_SANDBOX_SCOPE: &str = "unknown-sandbox-root";
        self.registry
            .executors
            .get(tool_id)
            .map(|executor| match executor {
                ToolExecutorV1::RepoRead { sandbox_root }
                | ToolExecutorV1::RepoList { sandbox_root }
                | ToolExecutorV1::FileStat { sandbox_root }
                | ToolExecutorV1::RepoSearch { sandbox_root }
                | ToolExecutorV1::PatchPropose { sandbox_root }
                | ToolExecutorV1::PatchApply { sandbox_root }
                | ToolExecutorV1::RunChecks { sandbox_root } => sandbox_root.display().to_string(),
                ToolExecutorV1::Custom(_) => "custom-executor".to_string(),
            })
            .or_else(|| self.registry.sandbox_root_display())
            .unwrap_or_else(|| UNKNOWN_SANDBOX_SCOPE.into())
    }
}

#[derive(Debug, Clone)]
pub struct ToolInvocationOutcome {
    pub output: Value,
    pub receipt: ToolInvocationReportV1,
    pub permit_use_receipt: Option<PermitUseReportV1>,
}

impl ToolInvocationOutcome {
    pub fn output_text(&self) -> String {
        self.output
            .get("content")
            .and_then(Value::as_str)
            .map(str::to_string)
            .unwrap_or_else(|| self.output.to_string())
    }
}

#[derive(Debug, Clone)]
pub struct ToolInvocationError {
    message: String,
    receipt: ToolInvocationReportV1,
    approval_request: Option<ApprovalRequestV1>,
    schema_validation_receipt: Option<SchemaValidationReportV1>,
}

impl ToolInvocationError {
    pub fn new(message: String, receipt: ToolInvocationReportV1) -> Self {
        Self {
            message,
            receipt,
            approval_request: None,
            schema_validation_receipt: None,
        }
    }

    pub fn receipt(&self) -> &ToolInvocationReportV1 {
        &self.receipt
    }

    pub fn approval_request(&self) -> Option<&ApprovalRequestV1> {
        self.approval_request.as_ref()
    }

    pub fn schema_validation_receipt(&self) -> Option<&SchemaValidationReportV1> {
        self.schema_validation_receipt.as_ref()
    }

    pub fn with_approval_request(mut self, approval_request: ApprovalRequestV1) -> Self {
        self.approval_request = Some(approval_request);
        self
    }

    pub fn with_schema_validation_receipt(
        mut self,
        schema_validation_receipt: SchemaValidationReportV1,
    ) -> Self {
        self.schema_validation_receipt = Some(schema_validation_receipt);
        self
    }
}

impl fmt::Display for ToolInvocationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.receipt.reason_codes.is_empty() {
            f.write_str(&self.message)
        } else {
            write!(
                f,
                "{} [{}]",
                self.message,
                self.receipt.reason_codes.join(",")
            )
        }
    }
}

impl std::error::Error for ToolInvocationError {}

#[derive(Debug, Clone)]
pub(crate) struct ReceiptBearingToolFailure {
    pub(crate) message: String,
    pub(crate) reason_code: String,
    pub(crate) output: Value,
}

impl fmt::Display for ReceiptBearingToolFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for ReceiptBearingToolFailure {}

pub(crate) fn executor_reason_code(error: String) -> String {
    let lower = error.to_ascii_lowercase();
    if lower.contains("traversal") {
        "sandbox-path-traversal-denied".into()
    } else if lower.contains("hardlink") {
        "sandbox-hardlink-denied".into()
    } else if lower.contains("sensitive prefix") {
        "sandbox-sensitive-prefix-denied".into()
    } else if lower.contains("hidden or sensitive component") {
        "sandbox-hidden-component-denied".into()
    } else if lower.contains("escape") || lower.contains("outside sandbox") {
        "sandbox-escape-denied".into()
    } else if lower.contains("command-not-allowed") {
        "command-not-allowed-by-policy".into()
    } else if lower.contains("patch") {
        "patch-apply-failed".into()
    } else {
        "tool-executor-failed".into()
    }
}
