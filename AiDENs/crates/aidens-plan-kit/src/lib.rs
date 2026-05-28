//! Execution-plan assembly helpers for AiDENs product surfaces.
//!
//! This crate owns only construction of AiDENs app-layer plans. Canonical
//! memory, kernel, repair, verification, evidence, and federation semantics
//! remain delegated to their canonical crates.

use aidens_contracts::{AiDENsAppPlanV1, MemoryModeV1, ReportLevelV1, RiskDisclosureV1};
use thiserror::Error;

pub const PLAN_KIT_SCOPE: &str = "execution-plan-assembly-only";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionPlanAssemblyInputV1 {
    pub app_id: String,
    pub profile_id: String,
    pub provider_required: bool,
    pub memory_mode: MemoryModeV1,
    pub receipt_level: ReportLevelV1,
    pub dangerous_auto_approval: bool,
    pub risk_disclosures: Vec<RiskDisclosureV1>,
    pub enabled_tool_bundles: Vec<String>,
    pub disabled_tool_bundles: Vec<String>,
}

impl From<AiDENsAppPlanV1> for ExecutionPlanAssemblyInputV1 {
    fn from(plan: AiDENsAppPlanV1) -> Self {
        Self {
            app_id: plan.app_id,
            profile_id: plan.profile_id,
            provider_required: plan.provider_required,
            memory_mode: plan.memory_mode,
            receipt_level: plan.receipt_level,
            dangerous_auto_approval: plan.dangerous_auto_approval,
            risk_disclosures: plan.risk_disclosures,
            enabled_tool_bundles: plan.enabled_tool_bundles,
            disabled_tool_bundles: plan.disabled_tool_bundles,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionPlanAssemblyReportV1 {
    pub scope: &'static str,
    pub plan: AiDENsAppPlanV1,
    pub reason_codes: Vec<String>,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum PlanAssemblyError {
    #[error("app_id must not be empty")]
    EmptyAppId,
    #[error("profile_id must not be empty")]
    EmptyProfileId,
    #[error("invalid assembled plan: {0}")]
    InvalidPlan(String),
}

pub fn assemble_execution_plan(
    input: ExecutionPlanAssemblyInputV1,
) -> Result<ExecutionPlanAssemblyReportV1, PlanAssemblyError> {
    if input.app_id.trim().is_empty() {
        return Err(PlanAssemblyError::EmptyAppId);
    }
    if input.profile_id.trim().is_empty() {
        return Err(PlanAssemblyError::EmptyProfileId);
    }
    let plan = AiDENsAppPlanV1 {
        app_id: input.app_id,
        profile_id: input.profile_id,
        provider_required: input.provider_required,
        memory_mode: input.memory_mode,
        receipt_level: input.receipt_level,
        dangerous_auto_approval: input.dangerous_auto_approval,
        risk_disclosures: input.risk_disclosures,
        enabled_tool_bundles: input.enabled_tool_bundles,
        disabled_tool_bundles: input.disabled_tool_bundles,
    };
    plan.validate().map_err(PlanAssemblyError::InvalidPlan)?;
    let reason_codes = vec![
        format!("scope:{PLAN_KIT_SCOPE}"),
        format!("profile:{}", plan.profile_id),
        format!("memory-mode:{}", plan.memory_mode),
        format!("receipt-level:{}", plan.receipt_level),
        format!("enabled-tool-bundles:{}", plan.enabled_tool_bundles.len()),
        format!("disabled-tool-bundles:{}", plan.disabled_tool_bundles.len()),
    ];
    Ok(ExecutionPlanAssemblyReportV1 {
        scope: PLAN_KIT_SCOPE,
        plan,
        reason_codes,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input() -> ExecutionPlanAssemblyInputV1 {
        ExecutionPlanAssemblyInputV1 {
            app_id: "agent".into(),
            profile_id: "coding-agent".into(),
            provider_required: true,
            memory_mode: MemoryModeV1::Disabled,
            receipt_level: ReportLevelV1::Full,
            dangerous_auto_approval: false,
            risk_disclosures: Vec::new(),
            enabled_tool_bundles: vec!["repo-read".into()],
            disabled_tool_bundles: vec!["shell-auto".into()],
        }
    }

    #[test]
    fn assembles_app_layer_plan_only() {
        let report = assemble_execution_plan(input()).unwrap();

        assert_eq!(report.scope, PLAN_KIT_SCOPE);
        assert_eq!(report.plan.app_id, "agent");
        assert_eq!(report.plan.enabled_tool_bundles, ["repo-read"]);
        assert!(report
            .reason_codes
            .contains(&"scope:execution-plan-assembly-only".into()));
    }

    #[test]
    fn rejects_empty_identity() {
        let mut input = input();
        input.app_id.clear();

        assert_eq!(
            assemble_execution_plan(input).unwrap_err(),
            PlanAssemblyError::EmptyAppId
        );
    }

    #[test]
    fn rejects_dangerous_auto_approval() {
        let mut input = input();
        input.dangerous_auto_approval = true;

        let error = assemble_execution_plan(input).unwrap_err();

        assert!(matches!(error, PlanAssemblyError::InvalidPlan(_)));
    }
}
