use aidens_contracts::{CanonicalToolSideEffectClass, ProviderRouteReportV1};
use aidens_permit_kit::PermitPolicyV1;
use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolExposurePolicyV1 {
    pub allowed_tool_ids: Option<BTreeSet<String>>,
    pub allowed_risk_classes: BTreeSet<CanonicalToolSideEffectClass>,
    pub max_tools: Option<usize>,
    pub native_tool_loop_available: bool,
    pub permit_policy: PermitPolicyV1,
    pub sandbox_root: Option<String>,
}

impl ToolExposurePolicyV1 {
    pub fn read_only_default() -> Self {
        Self {
            allowed_tool_ids: None,
            allowed_risk_classes: BTreeSet::from([CanonicalToolSideEffectClass::ReadOnly]),
            max_tools: Some(16),
            native_tool_loop_available: false,
            permit_policy: PermitPolicyV1::default(),
            sandbox_root: None,
        }
    }

    pub fn coding_agent_default() -> Self {
        Self {
            allowed_tool_ids: None,
            allowed_risk_classes: BTreeSet::from([
                CanonicalToolSideEffectClass::ReadOnly,
                CanonicalToolSideEffectClass::Write,
                CanonicalToolSideEffectClass::Admin,
            ]),
            max_tools: Some(16),
            native_tool_loop_available: false,
            permit_policy: PermitPolicyV1::default(),
            sandbox_root: None,
        }
    }

    pub fn for_provider_route(mut self, route: &ProviderRouteReportV1) -> Self {
        self.native_tool_loop_available = route.native_tool_loop;
        self
    }

    pub fn with_permit_policy(mut self, permit_policy: PermitPolicyV1) -> Self {
        self.permit_policy = permit_policy;
        self
    }

    pub fn with_sandbox_root(mut self, sandbox_root: impl Into<String>) -> Self {
        self.sandbox_root = Some(sandbox_root.into());
        self
    }
}
