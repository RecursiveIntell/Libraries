//! Public umbrella crate for AiDENs.
//!
//! ## Quickstart
//!
//! One-liner mock agent:
//! `let output = AiDENsApp::chat("Hello!").await?;`
//!
//! With a real provider and profile:
//! `AiDENsApp::run_with(AiDENsProfile::CodingAgent, ProviderSpecV1::new("ollama"), "Fix the bug").await?`
//!
//! ## Building an agent with memory + governance + kernel reasoning
//!
//! Use `AiDENsApp::from_plan(profile.expand("my-agent")?)` with `.mock_provider(...)`
//! or `.provider_spec(...)` then `.build().await?`. Profiles auto-configure
//! memory, governance, and kernel reasoning based on the selected profile.

pub mod prelude {
    pub use aidens_agency_kit::{
        AgencyPolicyEngineV1, AgencyPolicyInputV1, AgencyPolicyOutcomeV1, AgencyPolicyReportV1,
        InfluenceClassV1,
    };
    pub use aidens_app_kit::{AiDENsApp, AiDENsAppBuilder, AiDENsProfile};
    pub use aidens_contracts::{
        AiDENsAppPlanV1, AidensRunContextV1, CanonicalToolSideEffectClass, RunReportV1,
    };
    pub use aidens_governance_kit::{CanonicalGovernanceAdapter, GovernanceContext};
    pub use aidens_kernel_kit::{
        CanonicalKernelAdapter, CompileOutput, CompilerPolicy, ConformanceGateResult,
        ExecutionReport, OracleAssessment, ReasoningOutput,
    };
    pub use aidens_memory_kit::{
        CanonicalMemoryAdapter, MemoryGroundingEvidenceV1, memory_config_for_root,
        runtime_config_for_namespace,
    };
    pub use aidens_profile_coding::coding_agent_plan;
    pub use aidens_provider_kit::ProviderSpecV1;
    pub use aidens_runner::{AiDENsRunInput, AiDENsRunOutput, AiDENsRunner};
    pub use aidens_security_kit::{evaluate_mcp_tool_safety, McpTrustReportV1};
    pub use aidens_tool_kit::{ToolRegistryV1, ToolExposurePolicyV1};
}