use crate::act::ActionFamily;
use crate::observe::{ObservationPaths, ObservationStatus};
use crate::types::{
    DecisionAudit, ExecutionLineageReceipt, ExportActionTraceV1, RepairRecordV1,
    StopRuleEvaluation, TargetNormalization, VerificationPlanArtifact,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::json;
use stack_ids::{AttemptId, TraceCtx};
use verification_adjudication::AdjudicationResult;
use verification_calibration::CalibrationSnapshot;
use verification_control::{
    BoundaryArtifactKind, BoundaryRepairRecord, CheckPlan, ControlReceipt, SchedulerDecision,
    VerificationAttempt, VerificationCase,
};
use verification_policy::{ApprovalRecord, PolicyDecision};

pub const PILOT_LOOP_RECEIPT_V1_SCHEMA: &str = "forge_pilot_loop_receipt_v1";
pub const LOOP_ITERATION_REPORT_V1_SCHEMA: &str = "forge_pilot_loop_iteration_report_v1";
pub const LOOP_REPORT_V1_SCHEMA: &str = "forge_pilot_loop_report_v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HaltReason {
    MaxIterationsReached,
    TimeBudgetExhausted,
    NothingWorthInvestigating,
    ImportRequired,
    NamespaceMismatch,
    StorageUnavailable,
    StorageCorrupt,
    AllTargetsExhausted,
    MissingKernelPayload,
    ThinExportBlockedExecution,
    ExternalHalt,
    RetryCapExceeded,
    AdvisoryOnlyFallback,
    #[cfg(feature = "governance")]
    GovernanceIncidentHalt,
    #[cfg(feature = "governance")]
    GovernanceAuthorityInsufficient,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct LoopBudgetReceipt {
    pub workload_class: String,
    pub time_budget_secs: u64,
    pub max_iterations: u32,
    pub cooldown_secs: u64,
    pub max_retries_per_target: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct LoopReceipt {
    pub schema_version: String,
    pub receipt_id: String,
    pub trace_ctx: TraceCtx,
    pub attempt_id: AttemptId,
    pub namespace: String,
    pub workspace_path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub observation_paths: Option<ObservationPaths>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub observation_status: Option<ObservationStatus>,
    pub non_authoritative: bool,
    pub advisory_only: bool,
    #[serde(default)]
    pub consumer_only_truth_access: bool,
    pub budget: LoopBudgetReceipt,
    pub halt_reason: HaltReason,
    pub iterations_completed: u32,
    pub actions_executed: u32,
    pub exports_completed: u32,
    pub imports_completed: u32,
    pub degraded_iterations: u32,
    pub targets_investigated: Vec<String>,
    pub started_at: String,
    pub finished_at: String,
    pub elapsed_seconds: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct LoopIterationReport {
    pub schema_version: String,
    pub iteration: u32,
    pub selected_target_key: Option<String>,
    pub action_family: Option<ActionFamily>,
    pub export_completed: bool,
    pub import_completed: bool,
    pub degraded: bool,
    pub advisory_only: bool,
    pub outcome_signature: Option<String>,
    pub verification_case: Option<VerificationCase>,
    pub check_plan: Option<CheckPlan>,
    pub verification_attempt: Option<VerificationAttempt>,
    pub scheduler_decision: Option<SchedulerDecision>,
    pub policy_decision: Option<PolicyDecision>,
    pub approval_records: Vec<ApprovalRecord>,
    pub calibration_snapshot: Option<CalibrationSnapshot>,
    pub adjudication: Option<AdjudicationResult>,
    pub control_receipt: Option<ControlReceipt>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_normalization: Option<TargetNormalization>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub decision_audit: Option<DecisionAudit>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lineage_receipt: Option<ExecutionLineageReceipt>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub export_trace: Option<ExportActionTraceV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stop_rule_evaluation: Option<StopRuleEvaluation>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub verification_plan_artifact: Option<VerificationPlanArtifact>,
    #[serde(default)]
    pub boundary_repairs: Vec<BoundaryRepairRecord>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub repair_records: Vec<RepairRecordV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rollback_invalidation_count: Option<usize>,
    #[cfg(feature = "governance")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub governance_receipt: Option<crate::governance_gate::GovernanceReceiptV1>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct LoopReport {
    pub schema_version: String,
    pub receipt: LoopReceipt,
    pub iterations_completed: u32,
    pub actions_executed: u32,
    pub exports_completed: u32,
    pub imports_completed: u32,
    pub halt_reason: HaltReason,
    pub targets_investigated: Vec<String>,
    pub degraded_iterations: u32,
    pub elapsed_seconds: f64,
    pub advisory_only_steps: bool,
    pub iterations: Vec<LoopIterationReport>,
}

pub(crate) fn apply_loop_report_boundary_repairs(report: &mut LoopReport) {
    for iteration in &mut report.iterations {
        if iteration.boundary_repairs.is_empty() && iteration.approval_records.is_empty() {
            iteration.boundary_repairs.push(BoundaryRepairRecord::new(
                BoundaryArtifactKind::LoopIterationReport,
                LOOP_ITERATION_REPORT_V1_SCHEMA,
                "default_empty_array",
                "$.approval_records",
                None,
                json!([]),
                "approval records default to an empty array at the loop report boundary",
            ));
        }
    }
}

/// Parses a loop report from a boundary artifact payload.
pub fn parse_loop_report_boundary(mut value: serde_json::Value) -> Result<LoopReport, String> {
    let iterations = value
        .get_mut("iterations")
        .and_then(|iterations| iterations.as_array_mut())
        .ok_or_else(|| {
            "loop report boundary validation failed: missing iterations array".to_string()
        })?;

    for iteration in iterations {
        let Some(iteration_object) = iteration.as_object_mut() else {
            return Err(
                "loop report boundary validation failed: iteration entry is not an object".into(),
            );
        };
        if !iteration_object.contains_key("approval_records") {
            iteration_object.insert("approval_records".into(), json!([]));
        }
        if !iteration_object.contains_key("boundary_repairs") {
            iteration_object.insert("boundary_repairs".into(), json!([]));
        }
    }

    let mut report: LoopReport = serde_json::from_value(value)
        .map_err(|error| format!("loop report boundary validation failed: {error}"))?;
    apply_loop_report_boundary_repairs(&mut report);
    Ok(report)
}
