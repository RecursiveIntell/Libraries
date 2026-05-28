//! Tool registry, exposure planning, and safe read-only dispatch.

use aidens_contracts::{
    generated_artifact_id_from_material, ApprovalRequestV1, ArtifactId, CanonicalBackpointerV1,
    CanonicalToolSideEffectClass, CapabilityGateDecisionDraftV1, CapabilityGateDecisionV1,
    CapabilityGateOutcomeV1, CommandRunReportV1, DisplayDigestV1, PatchApplyReportV1,
    PatchProposalV1, PermitUseReportV1, RepoListEntryV1, RepoListReportV1, RepoReadReportV1,
    SchemaValidationReportV1, ToolDescriptorV1, ToolExposureSetV1, ToolInvocationReportV1,
    ToolLifecycleStateV1, ToolProviderSchemaV1, ToolSchemaV1,
};
use aidens_permit_kit::{requires_permit, PermitCheckContextV1, PermitDecisionV1, PermitPolicyV1};
use aidens_security_kit::{validate_sandbox_path, PathSafetyError};
use anyhow::{anyhow, bail, Context};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::fs::{File, OpenOptions};
use std::io::Write;
#[cfg(unix)]
use std::os::unix::process::CommandExt;
use std::path::{Component, Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

pub mod canonical_stack;
mod exposure;

pub use exposure::ToolExposurePolicyV1;

#[derive(Debug, Clone, Default)]
pub struct ToolRegistryV1 {
    tools: BTreeMap<String, ToolDescriptorV1>,
    executors: BTreeMap<String, ToolExecutorV1>,
    sandbox_root: Option<PathBuf>,
    construction_degradation_reasons: Vec<String>,
}

#[derive(Debug, Clone)]
enum ToolExecutorV1 {
    RepoRead { sandbox_root: PathBuf },
    RepoList { sandbox_root: PathBuf },
    FileStat { sandbox_root: PathBuf },
    RepoSearch { sandbox_root: PathBuf },
    PatchPropose { sandbox_root: PathBuf },
    PatchApply { sandbox_root: PathBuf },
    RunChecks { sandbox_root: PathBuf },
}

#[derive(Debug, Clone)]
struct GateDescriptorOutcome {
    outcome: CapabilityGateOutcomeV1,
    reason_codes: Vec<String>,
    approval_request: Option<ApprovalRequestV1>,
    permit_grant_id: Option<ArtifactId>,
    permit_use_receipt: Option<PermitUseReportV1>,
}

impl GateDescriptorOutcome {
    fn exposed() -> Self {
        Self {
            outcome: CapabilityGateOutcomeV1::Exposed,
            reason_codes: Vec::new(),
            approval_request: None,
            permit_grant_id: None,
            permit_use_receipt: None,
        }
    }

    fn exposed_with_permit(
        permit_use_receipt: PermitUseReportV1,
        permit_grant_id: ArtifactId,
    ) -> Self {
        Self {
            outcome: CapabilityGateOutcomeV1::Exposed,
            reason_codes: vec!["permit-scope-matched".into()],
            approval_request: None,
            permit_grant_id: Some(permit_grant_id),
            permit_use_receipt: Some(permit_use_receipt),
        }
    }

    fn hidden(reason_codes: Vec<String>) -> Self {
        Self {
            outcome: CapabilityGateOutcomeV1::Hidden,
            reason_codes,
            approval_request: None,
            permit_grant_id: None,
            permit_use_receipt: None,
        }
    }

    fn blocked(reason_codes: Vec<String>) -> Self {
        Self {
            outcome: CapabilityGateOutcomeV1::Blocked,
            reason_codes,
            approval_request: None,
            permit_grant_id: None,
            permit_use_receipt: None,
        }
    }

    fn blocked_with_approval(
        reason_codes: Vec<String>,
        approval_request: ApprovalRequestV1,
    ) -> Self {
        Self {
            outcome: CapabilityGateOutcomeV1::Blocked,
            reason_codes,
            approval_request: Some(approval_request),
            permit_grant_id: None,
            permit_use_receipt: None,
        }
    }
}

fn canonical_descriptor_from_aidens(
    descriptor: &ToolDescriptorV1,
) -> canonical_stack::CanonicalToolDescriptor {
    canonical_stack::CanonicalToolDescriptor {
        name: descriptor.tool_id(),
        version: descriptor.version.clone(),
        description: Some(descriptor.description.clone()),
        backend_kind: canonical_stack::ToolBackendKind::LocalFunction,
        input_schema: descriptor.schema.input_schema.clone(),
        output_mode: canonical_stack::ToolOutputMode::StructuredJson,
        read_only: descriptor.read_only,
        side_effect_class: descriptor.risk_class.clone(),
        idempotency_class: if descriptor.read_only {
            canonical_stack::ToolIdempotencyClass::Idempotent
        } else {
            canonical_stack::ToolIdempotencyClass::BestEffort
        },
        approval_kind: if requires_permit(&descriptor.risk_class) {
            canonical_stack::ToolApprovalKind::PolicyRequired
        } else {
            canonical_stack::ToolApprovalKind::None
        },
        timeout_ms: 30_000,
        concurrency_key: None,
        cache_ttl_ms: None,
        exposure_mode: if descriptor.hidden {
            canonical_stack::ToolExposureMode::Hidden
        } else {
            canonical_stack::ToolExposureMode::Auto
        },
        mcp_surface_kind: canonical_stack::McpSurfaceKind::Tool,
        exposure_policy: Default::default(),
        receipt_persistence: if descriptor.read_only || requires_permit(&descriptor.risk_class) {
            canonical_stack::ToolReceiptPersistence::ForgeRaw
        } else {
            canonical_stack::ToolReceiptPersistence::Ephemeral
        },
        output_size_limit_bytes: None,
        provider_payload: Some(serde_json::json!({
            "aidens_namespace": descriptor.namespace,
            "aidens_name": descriptor.name,
            "canonical_owner": "llm-tool-runtime",
        })),
    }
}

fn validate_tool_input_with_canonical_runtime(
    descriptor: &ToolDescriptorV1,
    input: &Value,
) -> SchemaValidationReportV1 {
    let canonical_descriptor = canonical_descriptor_from_aidens(descriptor);
    let errors = canonical_stack::validate_canonical_arguments(&canonical_descriptor, input)
        .err()
        .map(|error| vec![error.message])
        .unwrap_or_default();
    SchemaValidationReportV1::new(Some(&descriptor.schema.input_schema), input, errors)
        .with_tool_id(descriptor.tool_id())
}

impl ToolRegistryV1 {
    pub fn register_enabled(&mut self, descriptor: ToolDescriptorV1, enabled: bool) -> bool {
        if enabled {
            self.tools.insert(descriptor.tool_id(), descriptor);
            true
        } else {
            false
        }
    }

    pub fn register_enabled_with_repo_read_dispatcher(
        &mut self,
        descriptor: ToolDescriptorV1,
        enabled: bool,
        sandbox_root: impl AsRef<Path>,
    ) -> anyhow::Result<bool> {
        let tool_id = descriptor.tool_id();
        if !self.register_enabled(descriptor, enabled) {
            return Ok(false);
        }
        let sandbox_root = canonical_sandbox_root(sandbox_root.as_ref())?;
        self.sandbox_root = Some(sandbox_root.clone());
        self.executors
            .insert(tool_id, ToolExecutorV1::RepoRead { sandbox_root });
        Ok(true)
    }

    fn register_enabled_with_executor(
        &mut self,
        descriptor: ToolDescriptorV1,
        enabled: bool,
        sandbox_root: impl AsRef<Path>,
        executor: fn(PathBuf) -> ToolExecutorV1,
    ) -> anyhow::Result<bool> {
        let tool_id = descriptor.tool_id();
        if !self.register_enabled(descriptor, enabled) {
            return Ok(false);
        }
        let sandbox_root = canonical_sandbox_root(sandbox_root.as_ref())?;
        self.sandbox_root = Some(sandbox_root.clone());
        self.executors.insert(tool_id, executor(sandbox_root));
        Ok(true)
    }

    pub fn with_sandbox_root(mut self, sandbox_root: impl AsRef<Path>) -> anyhow::Result<Self> {
        self.sandbox_root = Some(canonical_sandbox_root(sandbox_root.as_ref())?);
        Ok(self)
    }

    pub fn sandbox_root(&self) -> Option<&Path> {
        self.sandbox_root.as_deref()
    }

    pub fn contains_tool_id(&self, tool_id: &str) -> bool {
        self.tools.contains_key(tool_id)
    }

    pub fn descriptor(&self, tool_id: &str) -> Option<&ToolDescriptorV1> {
        self.tools.get(tool_id)
    }

    pub fn descriptors(&self) -> Vec<ToolDescriptorV1> {
        self.tools.values().cloned().collect()
    }

    pub fn tool_ids(&self) -> Vec<String> {
        self.tools.keys().cloned().collect()
    }

    pub fn executable_tool_ids(&self) -> Vec<String> {
        self.tools
            .keys()
            .filter(|tool_id| self.can_execute(tool_id))
            .cloned()
            .collect()
    }

    pub fn len(&self) -> usize {
        self.tools.len()
    }

    pub fn is_empty(&self) -> bool {
        self.tools.is_empty()
    }

    pub fn expose_read_only(&self) -> ToolExposureSetV1 {
        self.plan_exposure(&ToolExposurePolicyV1::read_only_default())
    }

    pub fn can_execute(&self, tool_id: &str) -> bool {
        self.executors.contains_key(tool_id)
    }

    pub fn plan_exposure(&self, policy: &ToolExposurePolicyV1) -> ToolExposureSetV1 {
        self.plan_exposure_with_declarations(policy, self.descriptors())
    }

    pub fn plan_exposure_with_declarations(
        &self,
        policy: &ToolExposurePolicyV1,
        declarations: Vec<ToolDescriptorV1>,
    ) -> ToolExposureSetV1 {
        let mut declarations = declarations;
        declarations.sort_by_key(ToolDescriptorV1::tool_id);
        let mut exposed = Vec::new();
        let mut hidden = Vec::new();
        let mut blocked = Vec::new();
        let mut decisions = Vec::new();
        let mut approval_requests = Vec::new();
        let mut permit_use_receipts = Vec::new();
        let mut provider_tool_schemas = Vec::new();
        let mut reason_codes = self.construction_degradation_reasons.clone();
        let declared_tool_ids = declarations
            .iter()
            .map(ToolDescriptorV1::tool_id)
            .collect::<Vec<_>>();
        let registered_tool_ids = self.tool_ids();
        let executable_tool_ids = self.executable_tool_ids();
        let sandbox_root = policy
            .sandbox_root
            .clone()
            .or_else(|| self.sandbox_root_display());

        for descriptor in declarations {
            let tool_id = descriptor.tool_id();
            let registered = self.contains_tool_id(&tool_id);
            let executable = self.can_execute(&tool_id);
            let mut lifecycle = vec![ToolLifecycleStateV1::Declared];
            if registered {
                lifecycle.push(ToolLifecycleStateV1::Registered);
            }
            if executable {
                lifecycle.push(ToolLifecycleStateV1::Executable);
            }

            let gate = self.gate_descriptor(
                &descriptor,
                policy,
                exposed.len(),
                registered,
                executable,
                sandbox_root.as_deref(),
            );

            match gate.outcome {
                CapabilityGateOutcomeV1::Exposed => {
                    lifecycle.push(ToolLifecycleStateV1::ExposedThisTurn);
                    exposed.push(tool_id.clone());
                    provider_tool_schemas.push(ToolProviderSchemaV1::from_descriptor(&descriptor));
                }
                CapabilityGateOutcomeV1::Hidden => {
                    lifecycle.push(ToolLifecycleStateV1::Hidden);
                    hidden.push(tool_id.clone());
                }
                CapabilityGateOutcomeV1::Blocked => {
                    lifecycle.push(ToolLifecycleStateV1::Blocked);
                    blocked.push(tool_id.clone());
                }
            }

            if let Some(request) = gate.approval_request.clone() {
                approval_requests.push(request);
            }
            if let Some(permit_use_receipt) = gate.permit_use_receipt.clone() {
                permit_use_receipts.push(permit_use_receipt);
            }
            reason_codes.extend(gate.reason_codes.iter().cloned());
            decisions.push(CapabilityGateDecisionV1::for_tool(
                CapabilityGateDecisionDraftV1 {
                    tool_id,
                    outcome: gate.outcome,
                    lifecycle,
                    risk_class: descriptor.risk_class.clone(),
                    permit_required: requires_permit(&descriptor.risk_class),
                    executable_this_turn: executable,
                    sandbox_root: sandbox_root.clone(),
                    approval_request: gate.approval_request,
                    permit_grant_id: gate.permit_grant_id,
                    permit_use_receipt_id: gate
                        .permit_use_receipt
                        .map(|receipt| receipt.receipt_id),
                    reason_codes: gate.reason_codes,
                },
            ));
        }

        reason_codes.sort();
        reason_codes.dedup();
        let exposure_material = format!(
            "declared={declared:?}|registered={registered:?}|executable={executable_ids:?}|exposed={exposed:?}|hidden={hidden:?}|blocked={blocked:?}|sandbox={sandbox}",
            declared = &declared_tool_ids,
            registered = &registered_tool_ids,
            executable_ids = &executable_tool_ids,
            sandbox = sandbox_root
                .as_ref()
                .cloned()
                .unwrap_or_else(|| "<none>".into()),
        );

        ToolExposureSetV1 {
            exposure_id: generated_artifact_id_from_material("tool-exposure", &exposure_material),
            declared_tool_ids,
            registered_tool_ids,
            executable_tool_ids,
            exposed_tool_ids: exposed,
            hidden_tool_ids: hidden,
            blocked_tool_ids: blocked,
            decisions,
            approval_requests,
            permit_use_receipts,
            provider_tool_schemas,
            sandbox_root,
            degraded: !self.construction_degradation_reasons.is_empty(),
            reason_codes,
            canonical_backpointers: vec![CanonicalBackpointerV1::owner_type(
                "llm-tool-runtime",
                "ToolExposurePlan",
                "canonical-tool-exposure-owner",
            )],
            reason: Some("policy-filtered exposure".into()),
        }
    }

    pub fn declared_not_registered_tool_ids(
        &self,
        declarations: &[ToolDescriptorV1],
    ) -> Vec<String> {
        declarations
            .iter()
            .map(ToolDescriptorV1::tool_id)
            .filter(|tool_id| !self.contains_tool_id(tool_id))
            .collect()
    }

    fn sandbox_root_display(&self) -> Option<String> {
        self.sandbox_root
            .as_ref()
            .map(|path| path.display().to_string())
    }

    fn gate_descriptor(
        &self,
        descriptor: &ToolDescriptorV1,
        policy: &ToolExposurePolicyV1,
        exposed_count: usize,
        registered: bool,
        executable: bool,
        sandbox_root: Option<&str>,
    ) -> GateDescriptorOutcome {
        let tool_id = descriptor.tool_id();
        if !registered {
            return GateDescriptorOutcome::hidden(vec![format!(
                "tool-declared-not-registered:{tool_id}"
            )]);
        }
        if descriptor.hidden {
            return GateDescriptorOutcome::hidden(vec![format!("tool-hidden:{tool_id}")]);
        }
        if policy
            .allowed_tool_ids
            .as_ref()
            .is_some_and(|allowed| !allowed.contains(&tool_id))
        {
            return GateDescriptorOutcome::hidden(vec![format!(
                "tool-not-allowed-this-turn:{tool_id}"
            )]);
        }
        if !policy.allowed_risk_classes.contains(&descriptor.risk_class) {
            return GateDescriptorOutcome::blocked(vec![format!(
                "risk-blocked:{}",
                descriptor.risk_class
            )]);
        }
        let mut permit_grant_id = None;
        let mut permit_use_receipt = None;
        if requires_permit(&descriptor.risk_class) {
            let Some(permit_scope) = sandbox_root else {
                return GateDescriptorOutcome::blocked(vec![
                    "permit-scope-missing-sandbox-root".into(),
                    format!("permit-required:{}", descriptor.risk_class),
                ]);
            };
            let context = PermitCheckContextV1::new(
                tool_id.clone(),
                descriptor.risk_class.clone(),
                permit_scope,
            );
            match policy.permit_policy.decision_for_context(&context) {
                PermitDecisionV1::Allow => {
                    if let Some(receipt) = policy
                        .permit_policy
                        .permit_use_receipt_for_context(&context)
                    {
                        permit_grant_id = Some(receipt.permit_id.clone());
                        permit_use_receipt = Some(receipt);
                    }
                }
                PermitDecisionV1::RequiresApproval => {
                    let approval_request = policy
                        .permit_policy
                        .approval_request_for_context(&context)
                        .unwrap_or_else(|| {
                            ApprovalRequestV1::scoped(
                                tool_id.clone(),
                                descriptor.risk_class.clone(),
                                permit_scope,
                                "side-effect tool requires explicit scoped permit",
                            )
                        });
                    return GateDescriptorOutcome::blocked_with_approval(
                        vec![format!("permit-required:{}", descriptor.risk_class)],
                        approval_request,
                    );
                }
                PermitDecisionV1::Deny(reason) => {
                    return GateDescriptorOutcome::blocked(vec![format!("permit-denied:{reason}")]);
                }
            }
        }
        if descriptor.requires_native_tool_loop && !policy.native_tool_loop_available {
            return GateDescriptorOutcome::blocked(vec![format!(
                "route-requires-native-tool-loop:{tool_id}"
            )]);
        }
        if !executable {
            return GateDescriptorOutcome::hidden(vec![format!("tool-executor-missing:{tool_id}")]);
        }
        if policy.max_tools.is_some_and(|max| exposed_count >= max) {
            return GateDescriptorOutcome::hidden(vec!["max-tools-reached".into()]);
        }
        match (permit_use_receipt, permit_grant_id) {
            (Some(receipt), Some(grant_id)) => {
                GateDescriptorOutcome::exposed_with_permit(receipt, grant_id)
            }
            _ => GateDescriptorOutcome::exposed(),
        }
    }

    pub fn safe_coding_with_dispatchers(sandbox_root: impl AsRef<Path>) -> anyhow::Result<Self> {
        let mut registry = ToolRegistryV1::default();
        for (descriptor, enabled) in safe_coding_tool_plan() {
            match descriptor.tool_id().as_str() {
                "aidens:repo-read:1" => {
                    registry.register_enabled_with_repo_read_dispatcher(
                        descriptor,
                        enabled,
                        sandbox_root.as_ref(),
                    )?;
                }
                "aidens:repo-list:1" => {
                    registry.register_enabled_with_executor(
                        descriptor,
                        enabled,
                        sandbox_root.as_ref(),
                        |sandbox_root| ToolExecutorV1::RepoList { sandbox_root },
                    )?;
                }
                "aidens:file-stat:1" => {
                    registry.register_enabled_with_executor(
                        descriptor,
                        enabled,
                        sandbox_root.as_ref(),
                        |sandbox_root| ToolExecutorV1::FileStat { sandbox_root },
                    )?;
                }
                "aidens:repo-search:1" => {
                    registry.register_enabled_with_executor(
                        descriptor,
                        enabled,
                        sandbox_root.as_ref(),
                        |sandbox_root| ToolExecutorV1::RepoSearch { sandbox_root },
                    )?;
                }
                "aidens:patch-propose:1" => {
                    registry.register_enabled_with_executor(
                        descriptor,
                        enabled,
                        sandbox_root.as_ref(),
                        |sandbox_root| ToolExecutorV1::PatchPropose { sandbox_root },
                    )?;
                }
                "aidens:patch-apply:1" => {
                    registry.register_enabled_with_executor(
                        descriptor,
                        enabled,
                        sandbox_root.as_ref(),
                        |sandbox_root| ToolExecutorV1::PatchApply { sandbox_root },
                    )?;
                }
                "aidens:run-checks:1" => {
                    registry.register_enabled_with_executor(
                        descriptor,
                        enabled,
                        sandbox_root.as_ref(),
                        |sandbox_root| ToolExecutorV1::RunChecks { sandbox_root },
                    )?;
                }
                _ => {
                    registry.register_enabled(descriptor, enabled);
                }
            }
        }
        Ok(registry)
    }
}

pub fn safe_coding_registry() -> ToolRegistryV1 {
    let mut registry = ToolRegistryV1::default();
    for (descriptor, enabled) in safe_coding_tool_plan() {
        registry.register_enabled(descriptor, enabled);
    }
    registry
}

pub fn safe_coding_registry_for_current_dir() -> ToolRegistryV1 {
    safe_coding_registry_for_sandbox_root(".")
}

fn safe_coding_registry_for_sandbox_root(root: impl AsRef<Path>) -> ToolRegistryV1 {
    ToolRegistryV1::safe_coding_with_dispatchers(root.as_ref()).unwrap_or_else(|error| {
        let mut registry = safe_coding_registry();
        registry.construction_degradation_reasons.push(format!(
            "safe-coding dispatcher construction failed for sandbox root {}: {error}",
            root.as_ref().display()
        ));
        registry
    })
}

pub fn registry_from_enabled_bundles(
    enabled_bundles: &[String],
    sandbox_root: Option<&str>,
) -> ToolRegistryV1 {
    if !enabled_bundles.iter().any(|bundle| {
        matches!(
            bundle.as_str(),
            "safe-coding"
                | "repo-read"
                | "repo-list"
                | "file-stat"
                | "repo-search"
                | "patch-propose"
                | "patch-apply"
                | "run-checks"
        )
    }) {
        return ToolRegistryV1::default();
    }

    sandbox_root
        .and_then(|root| ToolRegistryV1::safe_coding_with_dispatchers(root).ok())
        .unwrap_or_else(safe_coding_registry)
}

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
struct ReceiptBearingToolFailure {
    message: String,
    reason_code: String,
    output: Value,
}

impl fmt::Display for ReceiptBearingToolFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for ReceiptBearingToolFailure {}

fn executor_reason_code(error: String) -> String {
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

fn repo_read(sandbox_root: &Path, input: &Value) -> anyhow::Result<Value> {
    let relative = input
        .get("path")
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("repo-read input requires string field 'path'"))?;
    let resolved = resolve_existing_sandboxed_path(sandbox_root, relative)?;
    let metadata = std::fs::metadata(&resolved)
        .with_context(|| format!("repo-read cannot stat {}", resolved.display()))?;
    if !metadata.is_file() {
        bail!("repo-read path is not a file: {}", relative)
    }
    reject_hardlinked_file(&resolved, &metadata)?;
    if metadata.len() > 1_048_576 {
        bail!(
            "repo-read refuses files larger than 1048576 bytes: {}",
            relative
        )
    }
    let content = std::fs::read_to_string(&resolved)
        .with_context(|| format!("repo-read failed to read {}", relative))?;
    let display_path = display_sandbox_path(sandbox_root, &resolved);
    let read_receipt = RepoReadReportV1::allowed(
        sandbox_root.display().to_string(),
        relative,
        display_path.clone(),
        metadata.len(),
        &content,
    );
    Ok(serde_json::json!({
        "tool_id": "aidens:repo-read:1",
        "path": display_path,
        "bytes": metadata.len(),
        "content_digest": read_receipt.content_digest,
        "receipt": read_receipt,
        "content": content,
    }))
}

fn repo_list(sandbox_root: &Path, input: &Value) -> anyhow::Result<Value> {
    let relative = input.get("path").and_then(Value::as_str).unwrap_or(".");
    let max_entries = input
        .get("max_entries")
        .and_then(Value::as_u64)
        .unwrap_or(200)
        .min(1000) as usize;
    let resolved = resolve_existing_sandboxed_path(sandbox_root, relative)?;
    if !resolved.is_dir() {
        bail!("repo-list path is not a directory: {relative}")
    }
    let mut entries = Vec::new();
    for entry in std::fs::read_dir(&resolved)
        .with_context(|| format!("repo-list cannot read {}", resolved.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        let metadata = std::fs::symlink_metadata(&path)?;
        if path_is_denied_by_prefix(sandbox_root, &path) {
            continue;
        }
        let file_type = metadata.file_type();
        let entry_kind = if file_type.is_symlink() {
            "symlink"
        } else if metadata.is_dir() {
            "dir"
        } else if metadata.is_file() {
            "file"
        } else {
            "other"
        };
        entries.push(RepoListEntryV1 {
            path: display_sandbox_path(sandbox_root, &path),
            entry_kind: entry_kind.into(),
            bytes: metadata.is_file().then_some(metadata.len()),
        });
    }
    entries.sort_by(|left, right| left.path.cmp(&right.path));
    let total_entries = entries.len();
    let full_listing = serde_json::to_value(&entries).unwrap_or(serde_json::Value::Null);
    let full_listing_digest = DisplayDigestV1::for_json_value(&full_listing);
    entries.truncate(max_entries);
    let receipt = RepoListReportV1::allowed_with_full_listing(
        sandbox_root.display().to_string(),
        relative,
        entries.clone(),
        total_entries,
        full_listing_digest,
    );
    Ok(serde_json::json!({
        "tool_id": "aidens:repo-list:1",
        "path": display_sandbox_path(sandbox_root, &resolved),
        "entries": entries,
        "total_entries": total_entries,
        "returned_entries": receipt.returned_entries,
        "truncated": receipt.truncated,
        "full_listing_digest": receipt.full_listing_digest,
        "receipt": receipt,
    }))
}

fn file_stat(sandbox_root: &Path, input: &Value) -> anyhow::Result<Value> {
    let relative = input
        .get("path")
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("file-stat input requires string field 'path'"))?;
    let resolved = resolve_existing_sandboxed_path(sandbox_root, relative)?;
    let metadata = std::fs::metadata(&resolved)
        .with_context(|| format!("file-stat cannot stat {}", resolved.display()))?;
    reject_hardlinked_file(&resolved, &metadata)?;
    let content_digest = if metadata.is_file() && metadata.len() <= 1_048_576 {
        let content = std::fs::read_to_string(&resolved)
            .with_context(|| format!("file-stat cannot read {}", resolved.display()))?;
        Some(DisplayDigestV1::for_text(&content))
    } else {
        None
    };
    Ok(serde_json::json!({
        "tool_id": "aidens:file-stat:1",
        "path": display_sandbox_path(sandbox_root, &resolved),
        "is_file": metadata.is_file(),
        "is_dir": metadata.is_dir(),
        "bytes": metadata.len(),
        "content_digest": content_digest,
    }))
}

fn repo_search(sandbox_root: &Path, input: &Value) -> anyhow::Result<Value> {
    let query = input
        .get("query")
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("repo-search input requires string field 'query'"))?;
    if query.is_empty() {
        bail!("repo-search query must not be empty")
    }
    let relative = input.get("path").and_then(Value::as_str).unwrap_or(".");
    let max_matches = input
        .get("max_matches")
        .and_then(Value::as_u64)
        .unwrap_or(50)
        .min(200) as usize;
    let resolved = resolve_existing_sandboxed_path(sandbox_root, relative)?;
    let mut matches = Vec::new();
    collect_search_matches(sandbox_root, &resolved, query, max_matches, &mut matches)?;
    Ok(serde_json::json!({
        "tool_id": "aidens:repo-search:1",
        "query": query,
        "path": display_sandbox_path(sandbox_root, &resolved),
        "matches": matches,
    }))
}

fn patch_propose(sandbox_root: &Path, input: &Value) -> anyhow::Result<Value> {
    let summary = input
        .get("summary")
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("patch-propose input requires string field 'summary'"))?;
    let diff = input
        .get("diff")
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("patch-propose input requires string field 'diff'"))?;
    let touched_paths = touched_paths_from_diff(diff)?;
    for path in &touched_paths {
        let _ = resolve_target_sandboxed_path(sandbox_root, path)?;
    }
    let proposal = PatchProposalV1::new(summary, diff, touched_paths);
    Ok(serde_json::json!({
        "tool_id": "aidens:patch-propose:1",
        "proposal": proposal,
        "mutates_files": false,
    }))
}

fn patch_apply(
    sandbox_root: &Path,
    input: &Value,
    permit_grant_id: Option<ArtifactId>,
    permit_use_receipt_id: Option<ArtifactId>,
) -> anyhow::Result<Value> {
    let diff = input
        .get("diff")
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("patch-apply input requires string field 'diff'"))?;
    let check_only = input
        .get("check_only")
        .or_else(|| input.get("dry_run"))
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let replacements = parse_simple_unified_diff(diff).map_err(|error| {
        patch_apply_failure(
            sandbox_root,
            input,
            error.to_string(),
            "invalid-patch",
            permit_grant_id.clone(),
            permit_use_receipt_id.clone(),
            Vec::new(),
        )
    })?;
    let mut before_digests = BTreeMap::new();
    let mut after_digests = BTreeMap::new();
    let mut touched_paths = Vec::new();
    let mut prepared = Vec::new();

    for replacement in replacements {
        let path = resolve_target_sandboxed_path(sandbox_root, &replacement.path)?;
        let before = std::fs::read_to_string(&path).map_err(|error| {
            patch_apply_failure(
                sandbox_root,
                input,
                format!(
                    "failed to read patch target {} before applying: {error}",
                    replacement.path
                ),
                "read-patch",
                permit_grant_id.clone(),
                permit_use_receipt_id.clone(),
                vec![replacement.path.clone()],
            )
        })?;
        let after = apply_single_replacement(&before, &replacement).map_err(|error| {
            let failure_kind = if error.to_string().to_ascii_lowercase().contains("ambiguous") {
                "ambiguous-patch"
            } else {
                "invalid-patch"
            };
            patch_apply_failure(
                sandbox_root,
                input,
                error.to_string(),
                failure_kind,
                permit_grant_id.clone(),
                permit_use_receipt_id.clone(),
                vec![replacement.path.clone()],
            )
        })?;
        let display_path = display_sandbox_path(sandbox_root, &path);
        before_digests.insert(display_path.clone(), DisplayDigestV1::for_text(&before));
        after_digests.insert(display_path.clone(), DisplayDigestV1::for_text(&after));
        touched_paths.push(display_path.clone());
        prepared.push((path, before, after, display_path));
    }

    if check_only {
        let receipt = PatchApplyReportV1::checked(
            sandbox_root.display().to_string(),
            input,
            touched_paths.clone(),
            before_digests,
            after_digests,
            permit_grant_id,
            permit_use_receipt_id,
        );
        return Ok(serde_json::json!({
            "tool_id": "aidens:patch-apply:1",
            "applied": false,
            "dry_run_checked": true,
            "changed_files": touched_paths,
            "semantic_status": "exact_check",
            "receipt": receipt,
        }));
    }

    let mut written = Vec::new();
    for (path, before, after, display_path) in &prepared {
        if let Err(error) = write_file_atomically(path, after) {
            let rollback_error = rollback_written_files(&written).err();
            return Err(patch_apply_failure(
                sandbox_root,
                input,
                format_rollback_failure(
                    format!("failed to write patched file {display_path}: {error}"),
                    rollback_error,
                ),
                "rollback-failed",
                permit_grant_id.clone(),
                permit_use_receipt_id.clone(),
                touched_paths.clone(),
            )
            .into());
        }
        written.push((path.clone(), before.clone()));
    }

    for (path, _before, after, display_path) in &prepared {
        match std::fs::read_to_string(path) {
            Ok(actual) if actual == *after => {}
            Ok(_) => {
                let rollback_error = rollback_written_files(&written).err();
                return Err(patch_apply_failure(
                    sandbox_root,
                    input,
                    format_rollback_failure(
                        format!("post-write verification failed for {display_path}"),
                        rollback_error,
                    ),
                    "rollback-failed",
                    permit_grant_id.clone(),
                    permit_use_receipt_id.clone(),
                    touched_paths.clone(),
                )
                .into());
            }
            Err(error) => {
                let rollback_error = rollback_written_files(&written).err();
                return Err(patch_apply_failure(
                    sandbox_root,
                    input,
                    format_rollback_failure(
                        format!("post-write verification could not read {display_path}: {error}"),
                        rollback_error,
                    ),
                    "rollback-failed",
                    permit_grant_id.clone(),
                    permit_use_receipt_id.clone(),
                    touched_paths.clone(),
                )
                .into());
            }
        }
    }

    let receipt = PatchApplyReportV1::new(
        sandbox_root.display().to_string(),
        input,
        touched_paths.clone(),
        before_digests,
        after_digests,
        permit_grant_id,
        permit_use_receipt_id,
    );
    Ok(serde_json::json!({
        "tool_id": "aidens:patch-apply:1",
        "applied": true,
        "dry_run_checked": true,
        "changed_files": touched_paths,
        "semantic_status": "exact_check",
        "touched_paths": touched_paths,
        "receipt": receipt,
    }))
}

fn patch_apply_failure(
    sandbox_root: &Path,
    input: &Value,
    message: String,
    failure_kind: &str,
    permit_grant_id: Option<ArtifactId>,
    permit_use_receipt_id: Option<ArtifactId>,
    touched_paths: Vec<String>,
) -> ReceiptBearingToolFailure {
    let reason_code = match failure_kind {
        "ambiguous-patch" => "patch-ambiguous-failed-closed",
        "read-patch" => "patch-target-read-failed-closed",
        "rollback-failed" => "patch-rollback-failed-quarantined",
        "rollback-patch" => "patch-rollback-quarantined",
        _ => "patch-invalid-failed-closed",
    };
    let receipt = PatchApplyReportV1::denied_with_details(
        sandbox_root.display().to_string(),
        input,
        reason_code,
        failure_kind,
        touched_paths.clone(),
        permit_grant_id,
        permit_use_receipt_id,
    );
    ReceiptBearingToolFailure {
        message,
        reason_code: reason_code.into(),
        output: serde_json::json!({
            "tool_id": "aidens:patch-apply:1",
            "applied": false,
            "dry_run_checked": true,
            "changed_files": touched_paths,
            "semantic_status": "failed_exact_check",
            "failure_kind": failure_kind,
            "rollback_advice": [
                "No files were written by this failed-closed patch attempt.",
                "Regenerate a single-file unified diff with unique removal context before retrying."
            ],
            "receipt": receipt,
        }),
    }
}

fn write_file_atomically(path: &Path, body: &str) -> std::io::Result<()> {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0);
    let tmp_path = parent.join(format!(
        ".{}.patch-tmp-{}-{suffix}",
        path.file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("target"),
        std::process::id()
    ));
    {
        let mut file = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&tmp_path)?;
        file.write_all(body.as_bytes())?;
        file.flush()?;
        file.sync_all()?;
    }
    std::fs::rename(&tmp_path, path)?;
    let _ = File::open(parent).and_then(|dir| dir.sync_all());
    Ok(())
}

fn rollback_written_files(written: &[(PathBuf, String)]) -> Result<(), String> {
    let mut failures = Vec::new();
    for (path, before) in written.iter().rev() {
        if let Err(error) = write_file_atomically(path, before) {
            failures.push(format!("{}: {error}", path.display()));
        }
    }
    if failures.is_empty() {
        Ok(())
    } else {
        Err(failures.join("; "))
    }
}

fn format_rollback_failure(primary: String, rollback_error: Option<String>) -> String {
    match rollback_error {
        Some(rollback_error) => format!("{primary}; rollback failed: {rollback_error}"),
        None => primary,
    }
}

fn run_checks(
    sandbox_root: &Path,
    input: &Value,
    permit_grant_id: Option<ArtifactId>,
    permit_use_receipt_id: Option<ArtifactId>,
) -> anyhow::Result<Value> {
    let command = command_args_from_input(input)?;
    if !command_is_allowed_check(&command) {
        let receipt = CommandRunReportV1::blocked(
            sandbox_root.display().to_string(),
            command,
            "command-not-allowed-by-policy",
        );
        return Ok(serde_json::json!({
            "tool_id": "aidens:run-checks:1",
            "succeeded": false,
            "receipt": receipt,
        }));
    }
    let timed_output =
        run_command_with_timeout(sandbox_root, &command, Duration::from_secs(120))
            .with_context(|| format!("failed to run check command: {}", command.join(" ")))?;
    let stdout_truncated = timed_output.output.stdout.len() > MAX_COMMAND_OUTPUT_BYTES;
    let stderr_truncated = timed_output.output.stderr.len() > MAX_COMMAND_OUTPUT_BYTES;
    let stdout = capped_utf8_lossy(&timed_output.output.stdout, MAX_COMMAND_OUTPUT_BYTES);
    let stderr = capped_utf8_lossy(&timed_output.output.stderr, MAX_COMMAND_OUTPUT_BYTES);
    let mut receipt = CommandRunReportV1::completed(
        sandbox_root.display().to_string(),
        command.clone(),
        permit_grant_id,
        permit_use_receipt_id,
        timed_output.output.status.code(),
        &stdout,
        &stderr,
    );
    if timed_output.timed_out {
        receipt.timed_out = true;
        receipt.succeeded = false;
        receipt.reason_codes = vec![
            "check-command-timeout".into(),
            "command-output-partial-after-timeout".into(),
        ];
    }
    if stdout_truncated {
        receipt.reason_codes.push("stdout-truncated".into());
    }
    if stderr_truncated {
        receipt.reason_codes.push("stderr-truncated".into());
    }
    receipt.reason_codes.sort();
    receipt.reason_codes.dedup();
    let semantic_status = if timed_output.timed_out {
        "partial_timeout"
    } else if stdout_truncated || stderr_truncated {
        "partial_output_capped"
    } else {
        "exact_check"
    };
    Ok(serde_json::json!({
        "tool_id": "aidens:run-checks:1",
        "command": command,
        "succeeded": receipt.succeeded,
        "exit_code": receipt.exit_code,
        "stdout": stdout,
        "stderr": stderr,
        "semantic_status": semantic_status,
        "receipt": receipt,
    }))
}

const MAX_COMMAND_OUTPUT_BYTES: usize = 65_536;

struct TimedCommandOutput {
    output: std::process::Output,
    timed_out: bool,
}

fn capped_utf8_lossy(bytes: &[u8], cap: usize) -> String {
    String::from_utf8_lossy(&bytes[..bytes.len().min(cap)]).to_string()
}

fn run_command_with_timeout(
    sandbox_root: &Path,
    command: &[String],
    timeout: Duration,
) -> anyhow::Result<TimedCommandOutput> {
    let executable = resolve_allowed_command_executable(&command[0])?;
    let mut command_proc = Command::new(executable);
    command_proc
        .args(&command[1..])
        .current_dir(sandbox_root)
        .env_clear()
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    #[cfg(unix)]
    {
        command_proc.process_group(0);
    }
    let mut child = command_proc.spawn()?;
    let started = Instant::now();
    let mut wait_interval = Duration::from_millis(5);
    loop {
        if child.try_wait()?.is_some() {
            return Ok(TimedCommandOutput {
                output: child.wait_with_output()?,
                timed_out: false,
            });
        }
        if started.elapsed() >= timeout {
            terminate_timed_out_command(&mut child, &command[0])?;
            return Ok(TimedCommandOutput {
                output: child.wait_with_output()?,
                timed_out: true,
            });
        }
        let elapsed = started.elapsed();
        let remaining = timeout.saturating_sub(elapsed);
        std::thread::sleep(wait_interval.min(remaining).min(Duration::from_millis(250)));
        wait_interval = (wait_interval * 2).min(Duration::from_millis(250));
    }
}

fn terminate_timed_out_command(child: &mut Child, command_label: &str) -> anyhow::Result<()> {
    #[cfg(unix)]
    {
        if terminate_unix_process_group(child.id()).is_ok() {
            return Ok(());
        }
    }
    match child.kill() {
        Ok(()) => Ok(()),
        Err(_) if child.try_wait()?.is_some() => Ok(()),
        Err(error) => {
            Err(error).with_context(|| format!("failed to kill timed-out command {command_label}"))
        }
    }
}

#[cfg(unix)]
fn terminate_unix_process_group(child_pid: u32) -> anyhow::Result<()> {
    let process_group = format!("-{child_pid}");
    for kill_path in ["/bin/kill", "/usr/bin/kill"] {
        if !Path::new(kill_path).exists() {
            continue;
        }
        let status = Command::new(kill_path)
            .args(["-KILL", &process_group])
            .env_clear()
            .status()
            .with_context(|| format!("failed to invoke fixed kill executable {kill_path}"))?;
        if status.success() {
            return Ok(());
        }
    }
    bail!("no fixed kill executable could terminate process group {process_group}")
}

fn resolve_allowed_command_executable(command: &str) -> anyhow::Result<PathBuf> {
    let candidates: &[&str] = match command {
        "cargo" => &[
            "/usr/bin/cargo",
            "/usr/local/bin/cargo",
            "/root/.cargo/bin/cargo",
        ],
        "bash" => &["/usr/bin/bash", "/bin/bash"],
        other => bail!("command executable is not in the fixed allowlist: {other}"),
    };
    candidates
        .iter()
        .map(PathBuf::from)
        .find(|path| path.is_file())
        .ok_or_else(|| anyhow!("allowed command executable not found in fixed paths: {command}"))
}

fn canonical_sandbox_root(path: &Path) -> anyhow::Result<PathBuf> {
    let canonical = path
        .canonicalize()
        .with_context(|| format!("sandbox root does not exist: {}", path.display()))?;
    if !canonical.is_dir() {
        bail!("sandbox root is not a directory: {}", canonical.display())
    }
    Ok(canonical)
}

fn resolve_existing_sandboxed_path(
    sandbox_root: &Path,
    requested: &str,
) -> anyhow::Result<PathBuf> {
    let requested_path = Path::new(requested);
    if requested_path.is_absolute() {
        bail!("absolute path escape rejected by sandbox")
    }
    if requested_path.components().any(|component| {
        matches!(
            component,
            Component::ParentDir | Component::RootDir | Component::Prefix(_)
        )
    }) {
        bail!("path traversal rejected by sandbox: {requested}")
    }

    let joined = sandbox_root.join(requested_path);
    let canonical = joined.canonicalize().with_context(|| {
        format!("repo-read path cannot be resolved inside sandbox: {requested}")
    })?;
    validate_sandbox_path(&canonical, sandbox_root)
        .map_err(|error| anyhow!(path_safety_message(error, requested)))?;
    Ok(canonical)
}

fn resolve_target_sandboxed_path(sandbox_root: &Path, requested: &str) -> anyhow::Result<PathBuf> {
    let requested_path = Path::new(requested);
    if requested_path.is_absolute() {
        bail!("absolute path escape rejected by sandbox")
    }
    if requested_path.components().any(|component| {
        matches!(
            component,
            Component::ParentDir | Component::RootDir | Component::Prefix(_)
        )
    }) {
        bail!("path traversal rejected by sandbox: {requested}")
    }
    let joined = sandbox_root.join(requested_path);
    let parent = joined.parent().unwrap_or(sandbox_root);
    let canonical_parent = parent.canonicalize().with_context(|| {
        format!("patch target parent cannot be resolved inside sandbox: {requested}")
    })?;
    validate_sandbox_path(&canonical_parent, sandbox_root)
        .map_err(|error| anyhow!(path_safety_message(error, requested)))?;
    let file_name = joined
        .file_name()
        .ok_or_else(|| anyhow!("patch target must name a file: {requested}"))?;
    if let Ok(metadata) = std::fs::symlink_metadata(&joined) {
        if metadata.file_type().is_symlink() {
            bail!("symlink write target rejected by sandbox: {requested}");
        }
        reject_hardlinked_file(&joined, &metadata)?;
        let canonical_target = joined.canonicalize().with_context(|| {
            format!("patch target cannot be resolved inside sandbox: {requested}")
        })?;
        validate_sandbox_path(&canonical_target, sandbox_root)
            .map_err(|error| anyhow!(path_safety_message(error, requested)))?;
    }
    Ok(canonical_parent.join(file_name))
}

fn path_safety_message(error: PathSafetyError, requested: &str) -> String {
    match error {
        PathSafetyError::TraversalNotAllowed => {
            format!("path traversal rejected by sandbox: {requested}")
        }
        PathSafetyError::OutsideSandbox { root } => {
            let _ = root;
            format!("path escape rejected by sandbox: {requested}; outside declared sandbox root")
        }
        PathSafetyError::SensitivePrefix { prefix } => {
            format!("sensitive prefix rejected by sandbox: {requested}; prefix {prefix}")
        }
        PathSafetyError::HiddenOrSensitiveComponent { component } => {
            format!("hidden or sensitive component rejected by sandbox: {requested}; component {component}")
        }
    }
}

#[cfg(unix)]
fn reject_hardlinked_file(path: &Path, metadata: &std::fs::Metadata) -> anyhow::Result<()> {
    use std::os::unix::fs::MetadataExt;
    if metadata.is_file() && metadata.nlink() > 1 {
        bail!(
            "hardlink read target rejected by sandbox: {}",
            display_redacted_path(path)
        );
    }
    Ok(())
}

#[cfg(not(unix))]
fn reject_hardlinked_file(_path: &Path, _metadata: &std::fs::Metadata) -> anyhow::Result<()> {
    Ok(())
}

fn display_redacted_path(path: &Path) -> String {
    path.file_name()
        .and_then(|name| name.to_str())
        .map(|name| format!("<sandbox>/{name}"))
        .unwrap_or_else(|| "<sandbox>/<unknown>".into())
}

fn path_is_denied_by_prefix(sandbox_root: &Path, path: &Path) -> bool {
    validate_sandbox_path(path, sandbox_root).is_err()
}

fn display_sandbox_path(sandbox_root: &Path, path: &Path) -> String {
    path.strip_prefix(sandbox_root)
        .map(|relative| relative.display().to_string().replace('\\', "/"))
        .unwrap_or_else(|_| display_redacted_path(path))
}

fn collect_search_matches(
    sandbox_root: &Path,
    current: &Path,
    query: &str,
    max_matches: usize,
    matches: &mut Vec<Value>,
) -> anyhow::Result<()> {
    if matches.len() >= max_matches || path_is_denied_by_prefix(sandbox_root, current) {
        return Ok(());
    }
    let metadata = std::fs::symlink_metadata(current)?;
    if metadata.file_type().is_symlink() {
        return Ok(());
    }
    if metadata.is_dir() {
        for entry in std::fs::read_dir(current)? {
            let entry = entry?;
            let path = entry.path();
            let display = display_sandbox_path(sandbox_root, &path);
            if path_has_denied_component(Path::new(&display)) {
                continue;
            }
            collect_search_matches(sandbox_root, &path, query, max_matches, matches)?;
            if matches.len() >= max_matches {
                break;
            }
        }
    } else if metadata.is_file() && metadata.len() <= 1_048_576 {
        let Ok(content) = std::fs::read_to_string(current) else {
            return Ok(());
        };
        for (line_index, line) in content.lines().enumerate() {
            if line.contains(query) {
                matches.push(serde_json::json!({
                    "path": display_sandbox_path(sandbox_root, current),
                    "line": line_index + 1,
                    "text": line,
                }));
                if matches.len() >= max_matches {
                    break;
                }
            }
        }
    }
    Ok(())
}

fn path_has_denied_component(path: &Path) -> bool {
    path.components().any(|component| {
        matches!(
            component,
            Component::Normal(name) if name == ".git" || name == "target"
        )
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FileReplacement {
    path: String,
    removed: Vec<String>,
    added: Vec<String>,
}

fn touched_paths_from_diff(diff: &str) -> anyhow::Result<Vec<String>> {
    let paths = parse_simple_unified_diff(diff)?
        .into_iter()
        .map(|replacement| replacement.path)
        .collect::<Vec<_>>();
    if paths.is_empty() {
        bail!("patch proposal contains no file paths")
    }
    Ok(paths)
}

fn parse_simple_unified_diff(diff: &str) -> anyhow::Result<Vec<FileReplacement>> {
    let mut replacements = Vec::new();
    let mut current: Option<FileReplacement> = None;
    let mut seen_paths = BTreeSet::new();
    let mut saw_old_path = false;
    for line in diff.lines() {
        if line.starts_with("--- ") {
            saw_old_path = true;
            continue;
        }
        if let Some(path) = line.strip_prefix("+++ ") {
            if !saw_old_path {
                bail!("unsupported unified diff: missing old-path header before new-path header")
            }
            if let Some(replacement) = current.take() {
                replacements.push(replacement);
            }
            let path = normalize_diff_path(path)?;
            if !seen_paths.insert(path.clone()) {
                bail!("ambiguous patch targets the same file more than once: {path}")
            }
            current = Some(FileReplacement {
                path,
                removed: Vec::new(),
                added: Vec::new(),
            });
            saw_old_path = false;
            continue;
        }
        if line.starts_with("@@") {
            continue;
        }
        if let Some(replacement) = current.as_mut() {
            if let Some(removed) = line.strip_prefix('-') {
                replacement.removed.push(removed.to_string());
            } else if let Some(added) = line.strip_prefix('+') {
                replacement.added.push(added.to_string());
            }
        }
    }
    if let Some(replacement) = current.take() {
        replacements.push(replacement);
    }
    replacements.retain(|replacement| {
        !replacement.path.is_empty()
            && (!replacement.removed.is_empty() || !replacement.added.is_empty())
    });
    if replacements.is_empty() {
        bail!("unsupported or empty unified diff")
    }
    if replacements
        .iter()
        .any(|replacement| replacement.removed.is_empty())
    {
        bail!("ambiguous patch lacks removal context")
    }
    Ok(replacements)
}

fn normalize_diff_path(path: &str) -> anyhow::Result<String> {
    let path = path.trim();
    let path = path.strip_prefix("b/").unwrap_or(path);
    if path == "/dev/null" {
        bail!("delete-only patch targets are not supported in P10")
    }
    if path.trim().is_empty() {
        bail!("diff path must not be empty")
    }
    Ok(path.to_string())
}

fn apply_single_replacement(before: &str, replacement: &FileReplacement) -> anyhow::Result<String> {
    let removed = replacement.removed.join("\n");
    let added = replacement.added.join("\n");
    if removed.is_empty() {
        let mut out = before.to_string();
        if !out.is_empty() && !out.ends_with('\n') {
            out.push('\n');
        }
        out.push_str(&added);
        out.push('\n');
        return Ok(out);
    }
    let with_newline = format!("{removed}\n");
    let added_with_newline = format!("{added}\n");
    let newline_matches = before.matches(&with_newline).count();
    if newline_matches == 1 {
        return Ok(before.replacen(&with_newline, &added_with_newline, 1));
    }
    if newline_matches > 1 {
        bail!(
            "ambiguous patch context appears multiple times in {}",
            replacement.path
        )
    }
    let raw_matches = before.matches(&removed).count();
    if raw_matches == 1 {
        return Ok(before.replacen(&removed, &added, 1));
    }
    if raw_matches > 1 {
        bail!(
            "ambiguous patch context appears multiple times in {}",
            replacement.path
        )
    }
    bail!(
        "patch context not found in {}; refused to guess",
        replacement.path
    )
}

fn command_args_from_input(input: &Value) -> anyhow::Result<Vec<String>> {
    if let Some(command) = input.get("command").and_then(Value::as_array) {
        let args = command
            .iter()
            .map(|value| {
                value
                    .as_str()
                    .map(str::to_string)
                    .ok_or_else(|| anyhow!("run-checks command entries must be strings"))
            })
            .collect::<anyhow::Result<Vec<_>>>()?;
        if args.is_empty() {
            bail!("run-checks command must not be empty")
        }
        return Ok(args);
    }
    if input.get("command").and_then(Value::as_str).is_some() {
        bail!("run-checks command must be structured argv array; shell/string command parsing is unsupported")
    }
    bail!("run-checks input requires field 'command'")
}

fn command_is_allowed_check(command: &[String]) -> bool {
    const ALLOWED: &[&[&str]] = &[
        &["cargo", "fmt", "--all", "--check"],
        &["cargo", "check", "--workspace"],
        &["cargo", "test", "--workspace"],
        &[
            "cargo",
            "clippy",
            "--workspace",
            "--all-targets",
            "--all-features",
            "--",
            "-D",
            "warnings",
        ],
        &["bash", "scripts/verify.sh"],
    ];
    ALLOWED.iter().any(|allowed| {
        command
            .iter()
            .map(String::as_str)
            .eq(allowed.iter().copied())
    })
}

pub fn safe_coding_tool_plan() -> Vec<(ToolDescriptorV1, bool)> {
    vec![
        (repo_read_descriptor(), true),
        (repo_list_descriptor(), true),
        (file_stat_descriptor(), true),
        (repo_search_descriptor(), true),
        (patch_propose_descriptor(), true),
        (patch_apply_descriptor(), true),
        (run_checks_descriptor(), true),
        (
            side_effect_descriptor("file-write", CanonicalToolSideEffectClass::Write),
            false,
        ),
        (
            side_effect_descriptor("shell", CanonicalToolSideEffectClass::Admin),
            false,
        ),
        (
            side_effect_descriptor("network", CanonicalToolSideEffectClass::Analysis),
            false,
        ),
        (
            side_effect_descriptor("memory-write", CanonicalToolSideEffectClass::Write),
            false,
        ),
        (
            side_effect_descriptor("schedule", CanonicalToolSideEffectClass::Admin),
            false,
        ),
    ]
}

pub fn safe_coding_tool_declarations() -> Vec<ToolDescriptorV1> {
    safe_coding_tool_plan()
        .into_iter()
        .map(|(descriptor, _enabled)| descriptor)
        .collect()
}

pub fn repo_read_descriptor() -> ToolDescriptorV1 {
    ToolDescriptorV1 {
        namespace: "aidens".into(),
        name: "repo-read".into(),
        version: "1".into(),
        description: "Read one UTF-8 file inside the configured repository sandbox.".into(),
        risk_class: CanonicalToolSideEffectClass::ReadOnly,
        read_only: true,
        hidden: false,
        requires_native_tool_loop: false,
        schema: ToolSchemaV1 {
            input_schema: serde_json::json!({
                "type": "object",
                "additionalProperties": false,
                "required": ["path"],
                "properties": {
                    "path": { "type": "string" }
                }
            }),
            output_schema: serde_json::json!({
                "type": "object",
                "required": ["tool_id", "path", "bytes", "content"],
                "properties": {
                    "tool_id": { "type": "string" },
                    "path": { "type": "string" },
                    "bytes": { "type": "integer", "minimum": 0 },
                    "content": { "type": "string" }
                }
            }),
            parser_fallback_hint: "Call aidens:repo-read:1 with JSON {\"path\":\"relative/file\"}."
                .into(),
        },
    }
}

pub fn patch_propose_descriptor() -> ToolDescriptorV1 {
    ToolDescriptorV1 {
        namespace: "aidens".into(),
        name: "patch-propose".into(),
        version: "1".into(),
        description: "Propose a patch without applying it; this tool never mutates files.".into(),
        risk_class: CanonicalToolSideEffectClass::ReadOnly,
        read_only: true,
        hidden: false,
        requires_native_tool_loop: false,
        schema: ToolSchemaV1 {
            input_schema: serde_json::json!({
                "type": "object",
                "additionalProperties": false,
                "required": ["summary", "diff"],
                "properties": {
                    "summary": { "type": "string" },
                    "diff": { "type": "string" }
                }
            }),
            output_schema: serde_json::json!({
                "type": "object",
                "required": ["tool_id", "proposal", "mutates_files"],
                "properties": {
                    "tool_id": { "type": "string" },
                    "proposal": { "type": "object" },
                    "mutates_files": { "type": "boolean" }
                }
            }),
            parser_fallback_hint: "Call aidens:patch-propose:1 with JSON {\"summary\":\"...\",\"diff\":\"unified diff\"}."
                .into(),
        },
    }
}

pub fn repo_list_descriptor() -> ToolDescriptorV1 {
    ToolDescriptorV1 {
        namespace: "aidens".into(),
        name: "repo-list".into(),
        version: "1".into(),
        description: "List entries inside one directory under the repository sandbox.".into(),
        risk_class: CanonicalToolSideEffectClass::ReadOnly,
        read_only: true,
        hidden: false,
        requires_native_tool_loop: false,
        schema: ToolSchemaV1 {
            input_schema: serde_json::json!({
                "type": "object",
                "additionalProperties": false,
                "properties": {
                    "path": { "type": "string" },
                    "max_entries": { "type": "integer", "minimum": 1, "maximum": 1000 }
                }
            }),
            output_schema: serde_json::json!({
                "type": "object",
                "required": ["tool_id", "path", "entries", "receipt"],
                "properties": {
                    "tool_id": { "type": "string" },
                    "path": { "type": "string" },
                    "entries": { "type": "array" },
                    "receipt": { "type": "object" }
                }
            }),
            parser_fallback_hint: "Call aidens:repo-list:1 with JSON {\"path\":\"relative/dir\"}."
                .into(),
        },
    }
}

pub fn file_stat_descriptor() -> ToolDescriptorV1 {
    ToolDescriptorV1 {
        namespace: "aidens".into(),
        name: "file-stat".into(),
        version: "1".into(),
        description: "Inspect metadata for one file or directory inside the sandbox.".into(),
        risk_class: CanonicalToolSideEffectClass::ReadOnly,
        read_only: true,
        hidden: false,
        requires_native_tool_loop: false,
        schema: ToolSchemaV1 {
            input_schema: serde_json::json!({
                "type": "object",
                "additionalProperties": false,
                "required": ["path"],
                "properties": {
                    "path": { "type": "string" }
                }
            }),
            output_schema: serde_json::json!({
                "type": "object",
                "required": ["tool_id", "path", "is_file", "is_dir", "bytes"],
                "properties": {
                    "tool_id": { "type": "string" },
                    "path": { "type": "string" },
                    "is_file": { "type": "boolean" },
                    "is_dir": { "type": "boolean" },
                    "bytes": { "type": "integer", "minimum": 0 },
                    "content_digest": { "type": ["object", "null"] }
                }
            }),
            parser_fallback_hint: "Call aidens:file-stat:1 with JSON {\"path\":\"relative/file\"}."
                .into(),
        },
    }
}

pub fn repo_search_descriptor() -> ToolDescriptorV1 {
    ToolDescriptorV1 {
        namespace: "aidens".into(),
        name: "repo-search".into(),
        version: "1".into(),
        description: "Search UTF-8 files inside the sandbox for a literal string.".into(),
        risk_class: CanonicalToolSideEffectClass::ReadOnly,
        read_only: true,
        hidden: false,
        requires_native_tool_loop: false,
        schema: ToolSchemaV1 {
            input_schema: serde_json::json!({
                "type": "object",
                "additionalProperties": false,
                "required": ["query"],
                "properties": {
                    "query": { "type": "string" },
                    "path": { "type": "string" },
                    "max_matches": { "type": "integer", "minimum": 1, "maximum": 200 }
                }
            }),
            output_schema: serde_json::json!({
                "type": "object",
                "required": ["tool_id", "query", "path", "matches"],
                "properties": {
                    "tool_id": { "type": "string" },
                    "query": { "type": "string" },
                    "path": { "type": "string" },
                    "matches": { "type": "array" }
                }
            }),
            parser_fallback_hint:
                "Call aidens:repo-search:1 with JSON {\"query\":\"needle\",\"path\":\".\"}.".into(),
        },
    }
}

pub fn patch_apply_descriptor() -> ToolDescriptorV1 {
    ToolDescriptorV1 {
        namespace: "aidens".into(),
        name: "patch-apply".into(),
        version: "1".into(),
        description: "Apply an approved unified patch inside the sandbox.".into(),
        risk_class: CanonicalToolSideEffectClass::Write,
        read_only: false,
        hidden: false,
        requires_native_tool_loop: false,
        schema: ToolSchemaV1 {
            input_schema: serde_json::json!({
                "type": "object",
                "additionalProperties": false,
                "required": ["diff"],
                "properties": {
                    "diff": { "type": "string" },
                    "check_only": { "type": "boolean" },
                    "dry_run": { "type": "boolean" }
                }
            }),
            output_schema: serde_json::json!({
                "type": "object",
                "required": ["tool_id", "applied", "dry_run_checked", "changed_files", "receipt"],
                "properties": {
                    "tool_id": { "type": "string" },
                    "applied": { "type": "boolean" },
                    "dry_run_checked": { "type": "boolean" },
                    "changed_files": { "type": "array", "items": { "type": "string" } },
                    "semantic_status": { "type": "string" },
                    "failure_kind": { "type": "string" },
                    "touched_paths": { "type": "array", "items": { "type": "string" } },
                    "receipt": { "type": "object" }
                }
            }),
            parser_fallback_hint:
                "aidens:patch-apply:1 requires explicit scoped file-write permit evidence.".into(),
        },
    }
}

pub fn run_checks_descriptor() -> ToolDescriptorV1 {
    ToolDescriptorV1 {
        namespace: "aidens".into(),
        name: "run-checks".into(),
        version: "1".into(),
        description: "Run an allowlisted local check command inside the sandbox.".into(),
        risk_class: CanonicalToolSideEffectClass::Admin,
        read_only: false,
        hidden: false,
        requires_native_tool_loop: false,
        schema: ToolSchemaV1 {
            input_schema: serde_json::json!({
                "type": "object",
                "additionalProperties": false,
                "required": ["command"],
                "properties": {
                    "command": {
                        "type": "array",
                        "items": { "type": "string" },
                        "minItems": 1
                    }
                }
            }),
            output_schema: serde_json::json!({
                "type": "object",
                "required": ["tool_id", "command", "succeeded", "receipt"],
                "properties": {
                    "tool_id": { "type": "string" },
                    "command": { "type": "array", "items": { "type": "string" } },
                    "succeeded": { "type": "boolean" },
                    "exit_code": { "type": ["integer", "null"] },
                    "stdout": { "type": "string" },
                    "stderr": { "type": "string" },
                    "receipt": { "type": "object" }
                }
            }),
            parser_fallback_hint:
                "aidens:run-checks:1 requires explicit scoped shell permit evidence.".into(),
        },
    }
}

pub fn side_effect_descriptor(
    name: &str,
    risk_class: CanonicalToolSideEffectClass,
) -> ToolDescriptorV1 {
    ToolDescriptorV1 {
        namespace: "aidens".into(),
        name: name.into(),
        version: "1".into(),
        description: format!("{name} is a side-effect tool and requires explicit permit evidence."),
        risk_class,
        read_only: false,
        hidden: false,
        requires_native_tool_loop: false,
        schema: ToolSchemaV1 {
            input_schema: serde_json::json!({
                "type": "object",
                "additionalProperties": true
            }),
            output_schema: serde_json::json!({
                "type": "object",
                "additionalProperties": true
            }),
            parser_fallback_hint: format!("{name} is not registered by the safe coding profile."),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn read_tool() -> ToolDescriptorV1 {
        ToolDescriptorV1 {
            namespace: "test".into(),
            name: "read".into(),
            version: "1".into(),
            description: "read test fixture".into(),
            risk_class: CanonicalToolSideEffectClass::ReadOnly,
            read_only: true,
            hidden: false,
            requires_native_tool_loop: false,
            schema: ToolSchemaV1 {
                input_schema: serde_json::json!({"type": "object"}),
                output_schema: serde_json::json!({"type": "object"}),
                parser_fallback_hint: "test read".into(),
            },
        }
    }

    fn normalized_exposure_json(exposure: &ToolExposureSetV1) -> serde_json::Value {
        let mut lifecycles = serde_json::Map::new();
        for decision in &exposure.decisions {
            lifecycles.insert(
                decision.capability_id.clone(),
                serde_json::Value::Array(
                    decision
                        .lifecycle
                        .iter()
                        .map(aidens_testkit::json_string)
                        .map(serde_json::Value::String)
                        .collect(),
                ),
            );
        }
        serde_json::json!({
            "declared_tool_ids": sorted(exposure.declared_tool_ids.clone()),
            "registered_tool_ids": sorted(exposure.registered_tool_ids.clone()),
            "executable_tool_ids": sorted(exposure.executable_tool_ids.clone()),
            "exposed_tool_ids": sorted(exposure.exposed_tool_ids.clone()),
            "hidden_tool_ids": sorted(exposure.hidden_tool_ids.clone()),
            "blocked_tool_ids": sorted(exposure.blocked_tool_ids.clone()),
            "lifecycles": serde_json::Value::Object(lifecycles),
            "reason_codes": sorted(exposure.reason_codes.clone())
        })
    }

    fn sorted(mut values: Vec<String>) -> Vec<String> {
        values.sort();
        values
    }

    #[test]
    fn disabled_tool_is_absent() {
        let mut registry = ToolRegistryV1::default();
        let tool = read_tool();
        let id = tool.tool_id();
        registry.register_enabled(tool, false);
        assert!(!registry.contains_tool_id(&id));
        assert!(registry.descriptor(&id).is_none());
        assert!(!registry.expose_read_only().exposed_tool_ids.contains(&id));
    }

    #[tokio::test]
    async fn disabled_means_absent_at_registration_exposure_and_invocation() {
        let mut registry = ToolRegistryV1::default();
        registry.register_enabled(
            side_effect_descriptor("shell", CanonicalToolSideEffectClass::Admin),
            false,
        );
        let dispatcher = ToolDispatcher::new(registry);

        let error = dispatcher
            .invoke("aidens:shell:1", serde_json::json!({}))
            .await
            .expect_err("disabled shell is not registered");
        assert!(error.to_string().contains("not registered"));
        let receipt = error
            .downcast_ref::<ToolInvocationError>()
            .expect("typed invocation error")
            .receipt();
        assert!(receipt.reason_codes.contains(&"tool-not-registered".into()));
    }

    #[test]
    fn read_only_default_exposes_only_read_only_executable_tools() {
        let dir = std::env::temp_dir();
        let registry = ToolRegistryV1::safe_coding_with_dispatchers(dir).unwrap();
        let exposure = registry.expose_read_only();

        assert!(exposure
            .exposed_tool_ids
            .contains(&"aidens:repo-read:1".into()));
        assert!(exposure
            .exposed_tool_ids
            .contains(&"aidens:patch-propose:1".into()));
        assert!(!exposure
            .exposed_tool_ids
            .contains(&"aidens:patch-apply:1".into()));
        assert!(exposure.exposed_tool_ids.iter().all(|tool_id| {
            let descriptor = registry.descriptor(tool_id).expect("descriptor");
            descriptor.read_only
                && descriptor.risk_class == CanonicalToolSideEffectClass::ReadOnly
                && registry.can_execute(tool_id)
        }));
    }

    #[test]
    fn exposure_policy_can_bound_tool_count() {
        let dir = std::env::temp_dir();
        let mut registry = ToolRegistryV1::default();
        registry
            .register_enabled_with_repo_read_dispatcher(read_tool(), true, &dir)
            .unwrap();
        registry
            .register_enabled_with_repo_read_dispatcher(
                ToolDescriptorV1 {
                    namespace: "test".into(),
                    name: "read2".into(),
                    version: "1".into(),
                    description: "second read test fixture".into(),
                    risk_class: CanonicalToolSideEffectClass::ReadOnly,
                    read_only: true,
                    hidden: false,
                    requires_native_tool_loop: false,
                    schema: ToolSchemaV1 {
                        input_schema: serde_json::json!({"type": "object"}),
                        output_schema: serde_json::json!({"type": "object"}),
                        parser_fallback_hint: "test read 2".into(),
                    },
                },
                true,
                &dir,
            )
            .unwrap();
        let exposure = registry.plan_exposure(&ToolExposurePolicyV1 {
            allowed_tool_ids: None,
            allowed_risk_classes: BTreeSet::from([CanonicalToolSideEffectClass::ReadOnly]),
            max_tools: Some(1),
            native_tool_loop_available: false,
            permit_policy: PermitPolicyV1::default(),
            sandbox_root: None,
        });
        assert_eq!(exposure.exposed_tool_ids.len(), 1);
        assert_eq!(exposure.hidden_tool_ids.len(), 1);
        assert!(exposure.reason_codes.contains(&"max-tools-reached".into()));
    }

    #[test]
    fn safe_coding_registry_does_not_register_dangerous_tools() {
        let registry = safe_coding_registry();

        assert!(registry.contains_tool_id("aidens:repo-read:1"));
        assert!(registry.contains_tool_id("aidens:patch-propose:1"));
        assert!(registry.contains_tool_id("aidens:patch-apply:1"));
        assert!(registry.contains_tool_id("aidens:run-checks:1"));
        assert!(!registry.contains_tool_id("aidens:file-write:1"));
        assert!(!registry.contains_tool_id("aidens:shell:1"));
        assert!(!registry.contains_tool_id("aidens:network:1"));
        assert!(!registry.contains_tool_id("aidens:memory-write:1"));
        assert!(!registry.contains_tool_id("aidens:schedule:1"));
    }

    #[test]
    fn patch_propose_is_executable_and_non_mutating() {
        let registry = safe_coding_registry_for_current_dir();
        let exposure = registry.expose_read_only();

        assert!(registry.contains_tool_id("aidens:patch-propose:1"));
        assert!(registry.can_execute("aidens:patch-propose:1"));
        assert!(exposure
            .exposed_tool_ids
            .contains(&"aidens:patch-propose:1".into()));
    }

    #[test]
    fn side_effect_tool_requires_permit() {
        let mut registry = ToolRegistryV1::default();
        registry.register_enabled(
            side_effect_descriptor("file-write", CanonicalToolSideEffectClass::Write),
            true,
        );
        let exposure = registry.plan_exposure(&ToolExposurePolicyV1 {
            allowed_tool_ids: None,
            allowed_risk_classes: BTreeSet::from([CanonicalToolSideEffectClass::Write]),
            max_tools: Some(16),
            native_tool_loop_available: false,
            permit_policy: PermitPolicyV1::default(),
            sandbox_root: Some(".".into()),
        });

        assert!(exposure
            .blocked_tool_ids
            .contains(&"aidens:file-write:1".into()));
        assert!(exposure
            .reason_codes
            .contains(&"permit-required:write".into()));
        assert_eq!(exposure.approval_requests.len(), 1);
        let decision = exposure
            .decisions
            .iter()
            .find(|decision| decision.capability_id == "aidens:file-write:1")
            .expect("file-write decision");
        assert_eq!(decision.outcome, CapabilityGateOutcomeV1::Blocked);
        assert!(decision.lifecycle.contains(&ToolLifecycleStateV1::Blocked));
        assert!(decision.approval_request.is_some());
    }

    #[test]
    fn side_effect_tool_without_sandbox_root_fails_closed_without_wildcard_scope() {
        let mut registry = ToolRegistryV1::default();
        registry.register_enabled(
            side_effect_descriptor("file-write", CanonicalToolSideEffectClass::Write),
            true,
        );

        let exposure = registry.plan_exposure(&ToolExposurePolicyV1 {
            allowed_tool_ids: None,
            allowed_risk_classes: BTreeSet::from([CanonicalToolSideEffectClass::Write]),
            max_tools: Some(16),
            native_tool_loop_available: false,
            permit_policy: PermitPolicyV1::default(),
            sandbox_root: None,
        });

        assert!(exposure
            .blocked_tool_ids
            .contains(&"aidens:file-write:1".into()));
        assert!(exposure
            .reason_codes
            .contains(&"permit-scope-missing-sandbox-root".into()));
        assert!(exposure.approval_requests.is_empty());
        let decision = exposure
            .decisions
            .iter()
            .find(|decision| decision.capability_id == "aidens:file-write:1")
            .expect("file-write decision");
        assert_eq!(decision.outcome, CapabilityGateOutcomeV1::Blocked);
        assert!(decision.approval_request.is_none());
        assert_eq!(decision.sandbox_root, None);
    }

    #[test]
    fn declared_registered_executable_hidden_and_blocked_are_distinct() {
        let registry = safe_coding_registry_for_current_dir();
        let declarations = safe_coding_tool_declarations();
        let exposure = registry.plan_exposure_with_declarations(
            &ToolExposurePolicyV1::read_only_default(),
            declarations.clone(),
        );

        assert!(exposure
            .declared_tool_ids
            .contains(&"aidens:file-write:1".into()));
        assert!(exposure
            .registered_tool_ids
            .contains(&"aidens:repo-read:1".into()));
        assert!(exposure
            .executable_tool_ids
            .contains(&"aidens:repo-read:1".into()));
        assert!(exposure
            .exposed_tool_ids
            .contains(&"aidens:repo-read:1".into()));
        assert!(exposure
            .exposed_tool_ids
            .contains(&"aidens:patch-propose:1".into()));
        assert!(exposure
            .blocked_tool_ids
            .contains(&"aidens:patch-apply:1".into()));
        assert!(registry
            .declared_not_registered_tool_ids(&declarations)
            .contains(&"aidens:file-write:1".into()));
    }

    #[test]
    fn safe_coding_exposure_matches_reference_interpreter() {
        let registry = safe_coding_registry_for_current_dir();
        let declarations = safe_coding_tool_declarations();
        let exposure = registry.plan_exposure_with_declarations(
            &ToolExposurePolicyV1::read_only_default(),
            declarations,
        );
        let case = aidens_testkit::reference_safe_coding_exposure_case();
        let report = aidens_testkit::compare_case_to_actual(
            &case,
            "aidens-tool-kit::plan_exposure_with_declarations",
            normalized_exposure_json(&exposure),
        );

        assert!(
            report.passed,
            "{}",
            report
                .findings
                .iter()
                .map(|finding| finding.human_diff.as_str())
                .collect::<Vec<_>>()
                .join("\n")
        );
    }

    #[tokio::test]
    async fn side_effect_invocation_without_permit_is_receipt_bearing_denial() {
        let mut registry = ToolRegistryV1::default();
        registry.register_enabled(
            side_effect_descriptor("file-write", CanonicalToolSideEffectClass::Write),
            true,
        );
        let dispatcher = ToolDispatcher::new(registry);

        let error = dispatcher
            .invoke("aidens:file-write:1", serde_json::json!({"path":"x"}))
            .await
            .expect_err("side-effect invocation requires permit");
        let invocation_error = error
            .downcast_ref::<ToolInvocationError>()
            .expect("typed invocation error");

        assert!(invocation_error
            .receipt()
            .reason_codes
            .contains(&"permit-required:write".into()));
        assert!(invocation_error.receipt().approval_request_id.is_some());
        assert!(invocation_error.approval_request().is_some());
    }

    #[test]
    fn exposure_receipt_matches_provider_tool_schema() {
        let dir = std::env::temp_dir();
        let registry = ToolRegistryV1::safe_coding_with_dispatchers(&dir).unwrap();
        let exposure = registry.expose_read_only();
        let schema_ids = exposure
            .provider_tool_schemas
            .iter()
            .map(|schema| schema.tool_id.clone())
            .collect::<BTreeSet<_>>();

        assert_eq!(
            exposure
                .exposed_tool_ids
                .iter()
                .cloned()
                .collect::<BTreeSet<_>>(),
            schema_ids
        );
        assert!(exposure
            .provider_tool_schemas
            .iter()
            .all(|schema| schema.input_schema["type"] == "object"));
    }

    #[test]
    fn side_effect_descriptors_require_durable_runtime_receipts() {
        let read_descriptor = canonical_descriptor_from_aidens(&repo_read_descriptor());
        assert_eq!(
            read_descriptor.receipt_persistence,
            canonical_stack::ToolReceiptPersistence::ForgeRaw
        );

        let patch_apply_descriptor = canonical_descriptor_from_aidens(&patch_apply_descriptor());
        assert_eq!(
            patch_apply_descriptor.receipt_persistence,
            canonical_stack::ToolReceiptPersistence::ForgeRaw
        );
    }

    #[test]
    fn safe_coding_registry_for_current_dir_fallback_is_degraded() {
        let missing_root = std::env::temp_dir().join(format!(
            "aidens-tool-kit-missing-root-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&missing_root);
        let registry = safe_coding_registry_for_sandbox_root(&missing_root);
        let exposure = registry.plan_exposure(&ToolExposurePolicyV1::read_only_default());

        assert!(
            exposure.degraded,
            "fallback path should emit a degraded tool registry exposure"
        );
        assert!(
            exposure
                .reason_codes
                .iter()
                .any(|reason| reason.contains("safe-coding dispatcher construction failed")),
            "fallback reasons should be surfaced through exposure reason codes"
        );
        assert!(
            !registry.tool_ids().is_empty(),
            "safe-registry fallback should still declare safe tools"
        );
        assert!(
            !registry.can_execute("aidens:repo-read:1"),
            "fallback tools should remain non-executable when dispatchers fail"
        );
    }

    #[tokio::test]
    async fn repo_read_rejects_traversal() {
        let dir = std::env::temp_dir().join(format!("aidens-tool-kit-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::create_dir_all(dir.join(".ssh")).unwrap();
        std::fs::create_dir_all(dir.join(".git")).unwrap();
        std::fs::create_dir_all(dir.join(".aws")).unwrap();
        std::fs::write(dir.join("README.md"), "hello").unwrap();
        std::fs::write(dir.join(".ssh").join("id_rsa"), "secret").unwrap();
        std::fs::write(dir.join(".git").join("config"), "secret").unwrap();
        std::fs::write(dir.join(".env"), "TOKEN=secret").unwrap();
        std::fs::write(dir.join(".npmrc"), "//registry/:_authToken=secret").unwrap();
        std::fs::write(dir.join(".aws").join("credentials"), "secret").unwrap();
        let registry = ToolRegistryV1::safe_coding_with_dispatchers(&dir).unwrap();
        let dispatcher = ToolDispatcher::new(registry);

        let output = dispatcher
            .invoke(
                "aidens:repo-read:1",
                serde_json::json!({ "path": "README.md" }),
            )
            .await
            .unwrap();
        assert!(output.output_text().contains("hello"));

        let error = dispatcher
            .invoke(
                "aidens:repo-read:1",
                serde_json::json!({ "path": "../secret" }),
            )
            .await
            .unwrap_err();
        assert!(error.to_string().contains("traversal"));
        let invocation_error = error
            .downcast_ref::<ToolInvocationError>()
            .expect("typed traversal receipt");
        assert!(invocation_error
            .receipt()
            .reason_codes
            .iter()
            .any(|reason| { reason.contains("sandbox") || reason.contains("traversal") }));
        let sensitive = dispatcher
            .invoke(
                "aidens:repo-read:1",
                serde_json::json!({ "path": ".ssh/id_rsa" }),
            )
            .await
            .unwrap_err();
        let sensitive = sensitive
            .downcast_ref::<ToolInvocationError>()
            .expect("typed sensitive-prefix receipt");
        assert!(sensitive
            .receipt()
            .reason_codes
            .contains(&"sandbox-sensitive-prefix-denied".into()));
        for denied_path in [".git/config", ".env", ".npmrc", ".aws/credentials"] {
            let error = dispatcher
                .invoke(
                    "aidens:repo-read:1",
                    serde_json::json!({ "path": denied_path }),
                )
                .await
                .unwrap_err();
            let invocation_error = error
                .downcast_ref::<ToolInvocationError>()
                .expect("typed sensitive fixture receipt");
            assert!(
                invocation_error
                    .receipt()
                    .reason_codes
                    .iter()
                    .any(|reason| reason == "sandbox-sensitive-prefix-denied"
                        || reason == "sandbox-hidden-component-denied"),
                "missing sensitive denial for {denied_path}: {:?}",
                invocation_error.receipt().reason_codes
            );
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::symlink;
            let outside = dir
                .parent()
                .unwrap_or_else(|| Path::new("/tmp"))
                .join(format!("aidens-outside-secret-{}", std::process::id()));
            std::fs::write(&outside, "outside secret").unwrap();
            symlink(&outside, dir.join("outside-link")).unwrap();
            std::fs::hard_link(&outside, dir.join("outside-hardlink")).unwrap();

            let symlink_error = dispatcher
                .invoke(
                    "aidens:repo-read:1",
                    serde_json::json!({ "path": "outside-link" }),
                )
                .await
                .unwrap_err();
            let symlink_text = symlink_error.to_string();
            assert!(symlink_text.contains("sandbox"));
            assert!(
                !symlink_text.contains(outside.parent().unwrap().to_string_lossy().as_ref()),
                "symlink denial leaked host path: {symlink_text}"
            );

            let hardlink_error = dispatcher
                .invoke(
                    "aidens:repo-read:1",
                    serde_json::json!({ "path": "outside-hardlink" }),
                )
                .await
                .unwrap_err();
            let hardlink_receipt = hardlink_error
                .downcast_ref::<ToolInvocationError>()
                .expect("typed hardlink receipt");
            assert!(hardlink_receipt
                .receipt()
                .reason_codes
                .contains(&"sandbox-hardlink-denied".into()));
            let _ = std::fs::remove_file(outside);
        }
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[tokio::test]
    async fn p10_read_inspect_search_and_patch_propose_are_read_only() {
        let dir = std::env::temp_dir().join(format!("aidens-p10-read-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("src")).unwrap();
        std::fs::write(dir.join("README.md"), "hello p10\n").unwrap();
        std::fs::write(dir.join("src").join("lib.rs"), "pub fn p10() {}\n").unwrap();
        let registry = ToolRegistryV1::safe_coding_with_dispatchers(&dir).unwrap();
        let dispatcher = ToolDispatcher::new(registry);

        let list = dispatcher
            .invoke("aidens:repo-list:1", serde_json::json!({"path":"."}))
            .await
            .unwrap();
        assert!(list.output["entries"]
            .as_array()
            .unwrap()
            .iter()
            .any(|entry| { entry["path"] == "README.md" }));

        let stat = dispatcher
            .invoke(
                "aidens:file-stat:1",
                serde_json::json!({"path":"README.md"}),
            )
            .await
            .unwrap();
        assert_eq!(stat.output["is_file"], true);

        let search = dispatcher
            .invoke(
                "aidens:repo-search:1",
                serde_json::json!({"query":"p10","path":"."}),
            )
            .await
            .unwrap();
        assert!(!search.output["matches"].as_array().unwrap().is_empty());

        let before = std::fs::read_to_string(dir.join("README.md")).unwrap();
        let proposal = dispatcher
            .invoke(
                "aidens:patch-propose:1",
                serde_json::json!({
                    "summary": "extend readme",
                    "diff": "--- a/README.md\n+++ b/README.md\n@@\n-hello p10\n+hello p10 patched\n"
                }),
            )
            .await
            .unwrap();
        let after = std::fs::read_to_string(dir.join("README.md")).unwrap();
        assert_eq!(before, after);
        assert_eq!(proposal.output["mutates_files"], false);
        assert_eq!(proposal.output["proposal"]["mutates_files"], false);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[tokio::test]
    async fn p10_patch_apply_requires_permit_and_then_writes_receipt() {
        let dir = std::env::temp_dir().join(format!("aidens-p10-apply-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("README.md"), "hello p10\n").unwrap();
        let registry = ToolRegistryV1::safe_coding_with_dispatchers(&dir).unwrap();
        let dispatcher = ToolDispatcher::new(registry.clone());
        let diff = "--- a/README.md\n+++ b/README.md\n@@\n-hello p10\n+hello p10 patched\n";

        let denied = dispatcher
            .invoke("aidens:patch-apply:1", serde_json::json!({"diff": diff}))
            .await
            .expect_err("patch apply requires file-write permit");
        let denied = denied
            .downcast_ref::<ToolInvocationError>()
            .expect("typed denial");
        assert!(denied.approval_request().is_some());
        assert!(denied.receipt().approval_request_id.is_some());

        let grant = aidens_contracts::PermitGrantV1::scoped(
            CanonicalToolSideEffectClass::Write,
            "aidens:patch-apply:1",
            dir.canonicalize().unwrap().display().to_string(),
            "test",
        );
        let permitted = ToolDispatcher::new(registry)
            .with_permit_policy(PermitPolicyV1::default().with_grant(grant));
        let applied = permitted
            .invoke("aidens:patch-apply:1", serde_json::json!({"diff": diff}))
            .await
            .unwrap();

        assert_eq!(
            std::fs::read_to_string(dir.join("README.md")).unwrap(),
            "hello p10 patched\n"
        );
        assert!(applied.receipt.succeeded);
        assert!(applied.receipt.permit_use_receipt_id.is_some());
        assert_eq!(applied.output["receipt"]["applied"], true);
        assert_eq!(applied.output["dry_run_checked"], true);
        assert_eq!(applied.output["semantic_status"], "exact_check");
        assert_eq!(applied.output["changed_files"][0], "README.md");
        assert_eq!(applied.output["receipt"]["changed_files"][0], "README.md");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[tokio::test]
    async fn p10_patch_apply_check_only_validates_without_mutation() {
        let dir =
            std::env::temp_dir().join(format!("aidens-p10-check-only-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("README.md"), "hello p10\n").unwrap();
        let registry = ToolRegistryV1::safe_coding_with_dispatchers(&dir).unwrap();
        let grant = aidens_contracts::PermitGrantV1::scoped(
            CanonicalToolSideEffectClass::Write,
            "aidens:patch-apply:1",
            dir.canonicalize().unwrap().display().to_string(),
            "test",
        );
        let dispatcher = ToolDispatcher::new(registry)
            .with_permit_policy(PermitPolicyV1::default().with_grant(grant));
        let diff = "--- a/README.md\n+++ b/README.md\n@@\n-hello p10\n+hello p10 checked\n";

        let checked = dispatcher
            .invoke(
                "aidens:patch-apply:1",
                serde_json::json!({"diff": diff, "check_only": true}),
            )
            .await
            .unwrap();

        assert_eq!(checked.output["applied"], false);
        assert_eq!(checked.output["dry_run_checked"], true);
        assert_eq!(
            checked.output["receipt"]["reason_codes"][0],
            "patch-validated-without-application"
        );
        assert_eq!(
            std::fs::read_to_string(dir.join("README.md")).unwrap(),
            "hello p10\n"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[tokio::test]
    async fn phase04_patch_apply_multifile_records_before_after_digests() {
        let dir =
            std::env::temp_dir().join(format!("aidens-phase04-multifile-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("a.txt"), "alpha\n").unwrap();
        std::fs::write(dir.join("b.txt"), "beta\n").unwrap();
        let registry = ToolRegistryV1::safe_coding_with_dispatchers(&dir).unwrap();
        let grant = aidens_contracts::PermitGrantV1::scoped(
            CanonicalToolSideEffectClass::Write,
            "aidens:patch-apply:1",
            dir.canonicalize().unwrap().display().to_string(),
            "test",
        );
        let dispatcher = ToolDispatcher::new(registry)
            .with_permit_policy(PermitPolicyV1::default().with_grant(grant));
        let diff = "--- a/a.txt\n+++ b/a.txt\n@@\n-alpha\n+alpha patched\n--- a/b.txt\n+++ b/b.txt\n@@\n-beta\n+beta patched\n";

        let applied = dispatcher
            .invoke("aidens:patch-apply:1", serde_json::json!({"diff": diff}))
            .await
            .unwrap();

        assert_eq!(
            std::fs::read_to_string(dir.join("a.txt")).unwrap(),
            "alpha patched\n"
        );
        assert_eq!(
            std::fs::read_to_string(dir.join("b.txt")).unwrap(),
            "beta patched\n"
        );
        assert_eq!(applied.output["changed_files"].as_array().unwrap().len(), 2);
        assert!(!applied.output["receipt"]["before_digests"]["a.txt"].is_null());
        assert!(!applied.output["receipt"]["after_digests"]["b.txt"].is_null());
        assert_eq!(applied.output["semantic_status"], "exact_check");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[tokio::test]
    async fn p10_patch_apply_ambiguous_diff_fails_closed_with_receipt() {
        let dir = std::env::temp_dir().join(format!("aidens-p10-ambiguous-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("README.md"), "repeat\nrepeat\n").unwrap();
        let registry = ToolRegistryV1::safe_coding_with_dispatchers(&dir).unwrap();
        let grant = aidens_contracts::PermitGrantV1::scoped(
            CanonicalToolSideEffectClass::Write,
            "aidens:patch-apply:1",
            dir.canonicalize().unwrap().display().to_string(),
            "test",
        );
        let dispatcher = ToolDispatcher::new(registry)
            .with_permit_policy(PermitPolicyV1::default().with_grant(grant));
        let diff = "--- a/README.md\n+++ b/README.md\n@@\n-repeat\n+patched\n";

        let denied = dispatcher
            .invoke("aidens:patch-apply:1", serde_json::json!({"diff": diff}))
            .await
            .expect_err("ambiguous diff must fail closed");
        let denied = denied
            .downcast_ref::<ToolInvocationError>()
            .expect("typed patch failure");

        assert!(!denied.receipt().succeeded);
        assert!(denied
            .receipt()
            .reason_codes
            .contains(&"patch-ambiguous-failed-closed".into()));
        let output = denied.receipt().output.as_ref().expect("failure output");
        assert_eq!(output["applied"], false);
        assert_eq!(output["dry_run_checked"], true);
        assert_eq!(output["semantic_status"], "failed_exact_check");
        assert_eq!(output["failure_kind"], "ambiguous-patch");
        assert_eq!(output["receipt"]["changed_files"][0], "README.md");
        assert_eq!(
            std::fs::read_to_string(dir.join("README.md")).unwrap(),
            "repeat\nrepeat\n"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[tokio::test]
    async fn p28_patch_apply_missing_parent_leaves_no_dirty_directory() {
        let dir =
            std::env::temp_dir().join(format!("aidens-p28-missing-parent-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let registry = ToolRegistryV1::safe_coding_with_dispatchers(&dir).unwrap();
        let grant = aidens_contracts::PermitGrantV1::scoped(
            CanonicalToolSideEffectClass::Write,
            "aidens:patch-apply:1",
            dir.canonicalize().unwrap().display().to_string(),
            "test",
        );
        let dispatcher = ToolDispatcher::new(registry)
            .with_permit_policy(PermitPolicyV1::default().with_grant(grant));
        let diff = "--- a/missing/child.txt\n+++ b/missing/child.txt\n@@\n-old\n+new\n";

        let error = dispatcher
            .invoke("aidens:patch-apply:1", serde_json::json!({"diff": diff}))
            .await
            .expect_err("missing parent must fail before filesystem mutation");

        assert!(error.to_string().contains("parent cannot be resolved"));
        assert!(!dir.join("missing").exists());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn p28_patch_apply_rejects_symlink_write_targets() {
        let dir = std::env::temp_dir().join(format!("aidens-p28-symlink-{}", std::process::id()));
        let outside =
            std::env::temp_dir().join(format!("aidens-p28-symlink-outside-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let _ = std::fs::remove_file(&outside);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(&outside, "secret\n").unwrap();
        std::os::unix::fs::symlink(&outside, dir.join("link.txt")).unwrap();
        let registry = ToolRegistryV1::safe_coding_with_dispatchers(&dir).unwrap();
        let grant = aidens_contracts::PermitGrantV1::scoped(
            CanonicalToolSideEffectClass::Write,
            "aidens:patch-apply:1",
            dir.canonicalize().unwrap().display().to_string(),
            "test",
        );
        let dispatcher = ToolDispatcher::new(registry)
            .with_permit_policy(PermitPolicyV1::default().with_grant(grant));
        let diff = "--- a/link.txt\n+++ b/link.txt\n@@\n-secret\n+patched\n";

        let error = dispatcher
            .invoke("aidens:patch-apply:1", serde_json::json!({"diff": diff}))
            .await
            .expect_err("symlink write target must be rejected");

        assert!(error.to_string().contains("symlink write target rejected"));
        assert_eq!(std::fs::read_to_string(&outside).unwrap(), "secret\n");
        let _ = std::fs::remove_dir_all(&dir);
        let _ = std::fs::remove_file(&outside);
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn p28_repo_list_reports_symlinks_without_following_targets() {
        let dir =
            std::env::temp_dir().join(format!("aidens-p28-list-symlink-{}", std::process::id()));
        let outside =
            std::env::temp_dir().join(format!("aidens-p28-list-outside-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let _ = std::fs::remove_dir_all(&outside);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::create_dir_all(&outside).unwrap();
        std::fs::write(outside.join("secret.txt"), "secret\n").unwrap();
        std::os::unix::fs::symlink(&outside, dir.join("outside-link")).unwrap();
        let registry = ToolRegistryV1::safe_coding_with_dispatchers(&dir).unwrap();
        let dispatcher = ToolDispatcher::new(registry);

        let list = dispatcher
            .invoke("aidens:repo-list:1", serde_json::json!({"path":"."}))
            .await
            .unwrap();
        let link = list.output["entries"]
            .as_array()
            .unwrap()
            .iter()
            .find(|entry| entry["path"] == "outside-link")
            .expect("symlink entry");
        assert_eq!(link["entry_kind"], "symlink");
        assert!(link.get("bytes").map_or(true, serde_json::Value::is_null));
        let _ = std::fs::remove_dir_all(&dir);
        let _ = std::fs::remove_dir_all(&outside);
    }

    #[tokio::test]
    async fn p28_repo_list_truncation_discloses_total_and_full_digest() {
        let dir =
            std::env::temp_dir().join(format!("aidens-p28-list-trunc-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        for index in 0..3 {
            std::fs::write(dir.join(format!("file-{index}.txt")), format!("{index}\n")).unwrap();
        }
        let registry = ToolRegistryV1::safe_coding_with_dispatchers(&dir).unwrap();
        let dispatcher = ToolDispatcher::new(registry);

        let list = dispatcher
            .invoke(
                "aidens:repo-list:1",
                serde_json::json!({"path":".","max_entries":1}),
            )
            .await
            .unwrap();

        assert_eq!(list.output["returned_entries"], 1);
        assert_eq!(list.output["total_entries"], 3);
        assert_eq!(list.output["truncated"], true);
        assert!(list.output["full_listing_digest"].is_object());
        assert_eq!(list.output["receipt"]["truncated"], true);
        assert_eq!(list.output["receipt"]["total_entries"], 3);
        assert!(list.output["receipt"]["reason_codes"]
            .as_array()
            .unwrap()
            .contains(&serde_json::json!("repo-list-truncated-with-full-digest")));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[tokio::test]
    async fn p28_file_stat_fails_on_unreadable_digest_input() {
        let dir = std::env::temp_dir().join(format!(
            "aidens-p28-file-stat-binary-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("binary.dat"), [0xff, 0xfe, 0xfd]).unwrap();
        let registry = ToolRegistryV1::safe_coding_with_dispatchers(&dir).unwrap();
        let dispatcher = ToolDispatcher::new(registry);

        let error = dispatcher
            .invoke(
                "aidens:file-stat:1",
                serde_json::json!({"path":"binary.dat"}),
            )
            .await
            .expect_err("invalid utf8 digest input must fail");
        assert!(error.to_string().contains("file-stat cannot read"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[tokio::test]
    async fn p28_repo_search_skips_denied_components_anywhere_in_path() {
        let dir = std::env::temp_dir().join(format!(
            "aidens-p28-search-components-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("nested").join("target")).unwrap();
        std::fs::create_dir_all(dir.join("src")).unwrap();
        std::fs::write(
            dir.join("nested").join("target").join("secret.txt"),
            "needle\n",
        )
        .unwrap();
        std::fs::write(dir.join("src").join("lib.rs"), "needle\n").unwrap();
        let registry = ToolRegistryV1::safe_coding_with_dispatchers(&dir).unwrap();
        let dispatcher = ToolDispatcher::new(registry);

        let search = dispatcher
            .invoke(
                "aidens:repo-search:1",
                serde_json::json!({"query":"needle","path":"."}),
            )
            .await
            .unwrap();
        let paths = search.output["matches"]
            .as_array()
            .unwrap()
            .iter()
            .map(|entry| entry["path"].as_str().unwrap().to_string())
            .collect::<Vec<_>>();
        assert!(paths.iter().any(|path| path == "src/lib.rs"));
        assert!(paths.iter().all(|path| !path.contains("/target/")));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn p28_run_command_timeout_marks_partial_execution() {
        let dir = std::env::temp_dir();
        let output = run_command_with_timeout(
            &dir,
            &["bash".into(), "-c".into(), "sleep 2".into()],
            Duration::from_millis(50),
        )
        .unwrap();
        assert!(output.timed_out);
    }

    #[test]
    fn p28_command_timeout_wait_uses_adaptive_backoff() {
        let dir = std::env::temp_dir();
        let started = Instant::now();
        let output = run_command_with_timeout(
            &dir,
            &["bash".into(), "-c".into(), "sleep 1".into()],
            Duration::from_millis(20),
        )
        .unwrap();
        assert!(output.timed_out);
        assert!(started.elapsed() < Duration::from_secs(1));
    }

    #[cfg(unix)]
    #[test]
    fn p30_command_timeout_terminates_process_group() {
        let dir = std::env::temp_dir();
        let started = Instant::now();
        let output = run_command_with_timeout(
            &dir,
            &["bash".into(), "-c".into(), "sleep 2 & wait".into()],
            Duration::from_millis(20),
        )
        .unwrap();
        assert!(output.timed_out);
        assert!(started.elapsed() < Duration::from_secs(1));
    }

    #[tokio::test]
    async fn p10_run_checks_requires_shell_permit_and_blocks_unallowed_commands() {
        let dir = std::env::temp_dir().join(format!("aidens-p10-checks-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let registry = ToolRegistryV1::safe_coding_with_dispatchers(&dir).unwrap();
        let dispatcher = ToolDispatcher::new(registry.clone());

        let denied = dispatcher
            .invoke(
                "aidens:run-checks:1",
                serde_json::json!({"command":["cargo","check","--workspace"]}),
            )
            .await
            .expect_err("run-checks requires shell permit");
        let denied = denied
            .downcast_ref::<ToolInvocationError>()
            .expect("typed denial");
        assert!(denied.approval_request().is_some());

        let grant = aidens_contracts::PermitGrantV1::scoped(
            CanonicalToolSideEffectClass::Admin,
            "aidens:run-checks:1",
            dir.canonicalize().unwrap().display().to_string(),
            "test",
        );
        let permitted = ToolDispatcher::new(registry)
            .with_permit_policy(PermitPolicyV1::default().with_grant(grant));
        let blocked = permitted
            .invoke(
                "aidens:run-checks:1",
                serde_json::json!({"command":["rm","-rf","."]}),
            )
            .await
            .unwrap();

        assert_eq!(blocked.output["succeeded"], false);
        assert_eq!(
            blocked.output["receipt"]["reason_codes"][0],
            "command-not-allowed-by-policy"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[tokio::test]
    async fn phase05_run_checks_rejects_string_commands_and_caps_output() {
        let dir =
            std::env::temp_dir().join(format!("aidens-phase05-checks-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let registry = ToolRegistryV1::safe_coding_with_dispatchers(&dir).unwrap();
        let grant = aidens_contracts::PermitGrantV1::scoped(
            CanonicalToolSideEffectClass::Admin,
            "aidens:run-checks:1",
            dir.canonicalize().unwrap().display().to_string(),
            "test",
        );
        let dispatcher = ToolDispatcher::new(registry)
            .with_permit_policy(PermitPolicyV1::default().with_grant(grant));

        let string_command = dispatcher
            .invoke(
                "aidens:run-checks:1",
                serde_json::json!({"command":"bash scripts/verify.sh"}),
            )
            .await
            .expect_err("string command must not be parsed");
        let string_command = string_command
            .downcast_ref::<ToolInvocationError>()
            .expect("typed schema rejection");
        assert!(string_command
            .receipt()
            .reason_codes
            .contains(&"schema-validation-failed".into()));

        let oversized = vec![b'x'; MAX_COMMAND_OUTPUT_BYTES + 8];
        assert_eq!(
            capped_utf8_lossy(&oversized, MAX_COMMAND_OUTPUT_BYTES).len(),
            MAX_COMMAND_OUTPUT_BYTES
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[tokio::test]
    async fn schema_invalid_tool_input_is_blocked_before_executor() {
        let dir = std::env::temp_dir().join(format!(
            "aidens-tool-kit-schema-test-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("README.md"), "hello").unwrap();
        let registry = ToolRegistryV1::safe_coding_with_dispatchers(&dir).unwrap();
        let dispatcher = ToolDispatcher::new(registry);

        let error = dispatcher
            .invoke("aidens:repo-read:1", serde_json::json!({ "path": 7 }))
            .await
            .expect_err("invalid tool input should be schema-blocked");
        let invocation_error = error
            .downcast_ref::<ToolInvocationError>()
            .expect("typed invocation error");
        let schema_validation = invocation_error
            .schema_validation_receipt()
            .expect("schema validation receipt");

        assert!(!schema_validation.valid);
        assert_eq!(
            schema_validation.tool_id.as_deref(),
            Some("aidens:repo-read:1")
        );
        assert!(invocation_error
            .receipt()
            .reason_codes
            .contains(&"schema-validation-failed".into()));
        assert_eq!(
            invocation_error.receipt().schema_validation_receipt_ids,
            vec![schema_validation.receipt_id.clone()]
        );
        assert!(!error.to_string().contains("requires string field"));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
