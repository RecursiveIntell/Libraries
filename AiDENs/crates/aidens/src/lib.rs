//! Public umbrella crate for AiDENs.

pub mod prelude {
    pub use aidens_agency_kit::{
        AgencyPolicyEngineV1, AgencyPolicyInputV1, AgencyPolicyOutcomeV1, AgencyPolicyReportV1,
        InfluenceClassV1,
    };
    pub use aidens_app_kit::{AiDENsApp, AiDENsAppBuilder, AiDENsProfile};
    pub use aidens_contracts::{
        AiDENsAppPlanV1, AidensRunContextV1, CanonicalToolSideEffectClass, CompactionReportV1,
        CompiledRegionGraphV1, CompletionAuditReportV1, ConvergenceReportV1,
        CrossPassTraceabilityMatrixV1, ExampleAppManifestV1, HistoryPreservationReportV1,
        InstallSmokeReportV1, InvariantBudgetV1, KernelResidualReportV1, KernelRunDisplayReportV1,
        KernelSyndromeReportV1, KnownLimitationsRegisterV1, OperatorStatusReportV1,
        OracleSliceRequestV1, RegionContractV1, RegressionDebtLedgerV1, ReleaseArtifactManifestV1,
        ReleaseReadinessReportV1, RemovalFrontierV1, RiskDisclosureV1, RunReportV1,
        SubtractionPlanV1, SupportCoreV1,
    };
    pub use aidens_kernel_kit::{
        CanonicalKernelAdapter, CompileOutput, CompilerPolicy, ExecutionReport, OracleAssessment,
    };
    pub use aidens_profile_coding::coding_agent_plan;
    pub use aidens_runner::{AiDENsRunInput, AiDENsRunOutput, AiDENsRunner};
}
