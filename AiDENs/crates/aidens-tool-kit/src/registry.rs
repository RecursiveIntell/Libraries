use crate::descriptors::safe_coding_tool_plan;
use crate::sandbox::canonical_sandbox_root;
use crate::{canonical_stack, ToolExposurePolicyV1};
use aidens_contracts::{
    generated_artifact_id_from_material, ApprovalRequestV1, ArtifactId, CanonicalBackpointerV1,
    CapabilityGateDecisionDraftV1, CapabilityGateDecisionV1, CapabilityGateOutcomeV1,
    PermitUseReportV1, SchemaValidationReportV1, ToolDescriptorV1, ToolExposureSetV1,
    ToolLifecycleStateV1, ToolProviderSchemaV1,
};
use aidens_permit_kit::requires_permit;
use serde_json::Value;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Default)]
pub struct ToolRegistryV1 {
    pub(crate) tools: BTreeMap<String, ToolDescriptorV1>,
    pub(crate) executors: BTreeMap<String, ToolExecutorV1>,
    pub(crate) sandbox_root: Option<PathBuf>,
    pub(crate) construction_degradation_reasons: Vec<String>,
}

#[derive(Debug, Clone)]
pub(crate) enum ToolExecutorV1 {
    RepoRead { sandbox_root: PathBuf },
    RepoList { sandbox_root: PathBuf },
    FileStat { sandbox_root: PathBuf },
    RepoSearch { sandbox_root: PathBuf },
    PatchPropose { sandbox_root: PathBuf },
    PatchApply { sandbox_root: PathBuf },
    RunChecks { sandbox_root: PathBuf },
    Custom(crate::custom::CustomExecutorHandle),
}

#[derive(Debug, Clone)]
pub(crate) struct GateDescriptorOutcome {
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

pub(crate) fn canonical_descriptor_from_aidens(
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

pub(crate) fn validate_tool_input_with_canonical_runtime(
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

    /// Register a tool with a custom async executor.
    /// The executor implements `CustomToolExecutor` and will be called
    /// when the tool is dispatched during the agent loop.
    pub fn register_enabled_with_custom_executor(
        &mut self,
        descriptor: ToolDescriptorV1,
        enabled: bool,
        executor: std::sync::Arc<dyn crate::custom::CustomToolExecutor>,
    ) -> bool {
        let tool_id = descriptor.tool_id();
        if !self.register_enabled(descriptor, enabled) {
            return false;
        }
        self.executors.insert(
            tool_id,
            ToolExecutorV1::Custom(crate::custom::CustomExecutorHandle::new(executor)),
        );
        true
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

    pub(crate) fn sandbox_root_display(&self) -> Option<String> {
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
        if requires_permit(&descriptor.risk_class) {
            let Some(permit_scope) = sandbox_root else {
                return GateDescriptorOutcome::blocked(vec![
                    "permit-scope-missing-sandbox-root".into(),
                    format!("permit-required:{}", descriptor.risk_class),
                ]);
            };
            let approval_request = ApprovalRequestV1::scoped(
                tool_id.clone(),
                descriptor.risk_class.clone(),
                permit_scope,
                "pre-run exposure cannot authorize a side-effect tool without exact run and attempt identifiers",
            );
            return GateDescriptorOutcome::blocked_with_approval(
                vec![
                    "permit-context-missing-run-attempt".into(),
                    format!("permit-required:{}", descriptor.risk_class),
                ],
                approval_request,
            );
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
        GateDescriptorOutcome::exposed()
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

pub fn safe_coding_registry_for_sandbox_root(root: impl AsRef<Path>) -> ToolRegistryV1 {
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
