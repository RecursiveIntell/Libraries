use crate::error::RuntimeError;
use crate::ids::Scope;
use crate::runtime::KnowledgeRuntime;
use constraint_compiler::{compile_batch, CompilerPolicy, ConstraintDegradation};
use kernel_execution::{
    schedule_execution, ExecutionBudget, ExecutionMode, ExecutionReport, ExecutionStopReason,
    ScheduledExecution, SchedulerStageKind,
};
use kernel_oracles::{
    evaluate_causal_refuter, evaluate_conservative, evaluate_exact_bounded,
    minimal_perturbation_witness, OracleAssessment, OracleRefutationOutcome,
    OracleRefutationResult,
};
use schemars::JsonSchema;
use semantic_memory::ProjectionImportLogEntry;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "InferenceAdvisoryV1")]
pub struct InferenceAdvisory {
    pub batch_id: String,
    pub source_envelope_id: String,
    pub graph_hash: String,
    pub execution_mode: String,
    pub iteration_count: u32,
    pub message_count: usize,
    pub degradation_markers: Vec<String>,
    pub oracle_mode: String,
    pub oracle_supported: bool,
    pub satisfied_constraint_count: usize,
    pub advisory_only: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub degraded_reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "InferenceExplanationV1")]
pub struct InferenceExplanation {
    pub batch_id: String,
    pub source_envelope_id: String,
    pub graph_hash: String,
    pub execution_mode: String,
    pub stop_reason: String,
    pub stage_kinds: Vec<String>,
    pub degradation_markers: Vec<String>,
    pub witness_count: usize,
    pub witness_node_ids: Vec<String>,
    pub certificate_count: usize,
    pub certificate_node_ids: Vec<String>,
    pub residual_count: usize,
    pub residual_micros: Vec<u64>,
    pub syndrome_signatures: Vec<String>,
    pub calibration_caveats: Vec<String>,
    pub refutation_outcome: String,
    pub causal_refutation_outcome: String,
    pub minimal_perturbation_outcome: String,
    pub oracle_mode: String,
    pub oracle_supported: bool,
    pub advisory_only: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub degraded_reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "RiskGateDecisionV1")]
pub struct RiskGateDecision {
    pub batch_id: String,
    pub source_envelope_id: String,
    pub status: String,
    pub reasons: Vec<String>,
    pub advisory_only: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub degraded_reason: Option<String>,
}

struct KernelAdvisoryContext {
    import_log: ProjectionImportLogEntry,
    compiled: constraint_compiler::CompileOutput,
    oracle: OracleAssessment,
    scheduled: ScheduledExecution,
    causal_refutation: OracleRefutationResult,
    minimal_perturbation: OracleRefutationResult,
}

fn degradation_marker_name(marker: &ConstraintDegradation) -> &'static str {
    match marker {
        ConstraintDegradation::MissingClaimFamily => "missing_claim_family",
        ConstraintDegradation::MissingAssertionGroup => "missing_assertion_group",
        ConstraintDegradation::MissingRelationGroup => "missing_relation_group",
        ConstraintDegradation::ThinExport => "thin_export",
    }
}

fn oracle_mode_name(oracle: &OracleAssessment) -> &'static str {
    match oracle.mode {
        kernel_oracles::OracleMode::ExactBounded => "exact_bounded",
        kernel_oracles::OracleMode::ConservativeFallback => "conservative_fallback",
    }
}

fn execution_mode_name(report: &ExecutionReport) -> &'static str {
    match report.execution_mode {
        ExecutionMode::AcyclicBaseline => "acyclic_baseline",
        ExecutionMode::MessagePassingBaseline => "message_passing_baseline",
        ExecutionMode::DeltaPropagation => "delta_propagation",
        ExecutionMode::ResidualCorrection => "residual_correction",
        ExecutionMode::MultiscaleScheduler => "multiscale_scheduler",
    }
}

fn execution_stop_reason_name(reason: &ExecutionStopReason) -> &'static str {
    match reason {
        ExecutionStopReason::AcyclicCompletion => "acyclic_completion",
        ExecutionStopReason::FixedPoint => "fixed_point",
        ExecutionStopReason::MaxIterations => "max_iterations",
        ExecutionStopReason::BudgetExhausted => "budget_exhausted",
        ExecutionStopReason::DeltaWindowCompleted => "delta_window_completed",
    }
}

fn scheduler_stage_name(stage: &SchedulerStageKind) -> &'static str {
    match stage {
        SchedulerStageKind::AcyclicPass => "acyclic_pass",
        SchedulerStageKind::MessagePassing => "message_passing",
        SchedulerStageKind::DeltaPropagation => "delta_propagation",
        SchedulerStageKind::ResidualCorrection => "residual_correction",
    }
}

fn refutation_outcome_name(outcome: &OracleRefutationOutcome) -> &'static str {
    match outcome {
        OracleRefutationOutcome::FlipWitness { .. } => "flip_witness_found",
        OracleRefutationOutcome::NoFlipFound { .. } => "no_flip_found_within_budget",
        OracleRefutationOutcome::NotApplicable { .. } => "unsupported",
    }
}

impl KnowledgeRuntime {
    async fn latest_kernel_advisory_context(
        &self,
        scope: &Scope,
    ) -> Result<Option<KernelAdvisoryContext>, RuntimeError> {
        let Some(import_log) = self
            .adapter_ref()
            .latest_kernel_projection_import(&scope.key())
            .await?
        else {
            return Ok(None);
        };
        let Some(batch) = import_log.rebuildable_kernel_batch_v3()? else {
            return Ok(None);
        };

        let compiled = compile_batch(
            &batch,
            &CompilerPolicy {
                policy_version: "knowledge-runtime.phase1.v1".into(),
                include_hyperedges: true,
            },
        );
        let oracle =
            evaluate_exact_bounded(&compiled).unwrap_or_else(|| evaluate_conservative(&compiled));
        let scheduled = schedule_execution(&compiled, &ExecutionBudget::default());
        let primary_node_id = scheduled
            .execution
            .witnesses
            .first()
            .map(|witness| witness.node_id.clone())
            .or_else(|| {
                compiled
                    .nodes
                    .iter()
                    .find(|node| node.kind != "nuisance_state")
                    .map(|node| node.node_id.clone())
            })
            .unwrap_or_else(|| "unsupported".into());
        let causal_refutation = evaluate_causal_refuter(&compiled, &primary_node_id, 1);
        let minimal_perturbation = minimal_perturbation_witness(&compiled, &primary_node_id, 1);

        Ok(Some(KernelAdvisoryContext {
            import_log,
            compiled,
            oracle,
            scheduled,
            causal_refutation,
            minimal_perturbation,
        }))
    }

    /// Compile the latest rebuildable kernel payload for a scope into a
    /// non-authoritative inference advisory.
    pub async fn latest_inference_advisory(
        &self,
        scope: Option<&Scope>,
    ) -> Result<Option<InferenceAdvisory>, RuntimeError> {
        let scope = scope.unwrap_or(self.default_scope());
        let Some(context) = self.latest_kernel_advisory_context(scope).await? else {
            return Ok(None);
        };

        Ok(Some(InferenceAdvisory {
            batch_id: context.import_log.batch_id,
            source_envelope_id: context.import_log.source_envelope_id,
            graph_hash: context.compiled.graph_hash.hex().to_string(),
            execution_mode: execution_mode_name(&context.scheduled.execution).to_string(),
            iteration_count: context.scheduled.execution.iteration_count,
            message_count: context.scheduled.execution.messages.len(),
            degradation_markers: context
                .compiled
                .degradations
                .iter()
                .map(degradation_marker_name)
                .map(str::to_string)
                .collect(),
            oracle_mode: oracle_mode_name(&context.oracle).to_string(),
            oracle_supported: context.oracle.supported,
            satisfied_constraint_count: context.oracle.satisfied_constraint_count,
            advisory_only: context.scheduled.execution.advisory_only,
            degraded_reason: context.scheduled.degraded_reason.clone(),
        }))
    }

    /// Expose witnesses, residuals, syndromes, calibration, and refutation
    /// state for the latest rebuildable kernel payload.
    pub async fn latest_inference_explanation(
        &self,
        scope: Option<&Scope>,
    ) -> Result<Option<InferenceExplanation>, RuntimeError> {
        let scope = scope.unwrap_or(self.default_scope());
        let Some(context) = self.latest_kernel_advisory_context(scope).await? else {
            return Ok(None);
        };

        Ok(Some(InferenceExplanation {
            batch_id: context.import_log.batch_id,
            source_envelope_id: context.import_log.source_envelope_id,
            graph_hash: context.compiled.graph_hash.hex().to_string(),
            execution_mode: execution_mode_name(&context.scheduled.execution).to_string(),
            stop_reason: execution_stop_reason_name(&context.scheduled.execution.stop_reason)
                .to_string(),
            stage_kinds: context
                .scheduled
                .stage_kinds
                .iter()
                .map(scheduler_stage_name)
                .map(str::to_string)
                .collect(),
            degradation_markers: context
                .compiled
                .degradations
                .iter()
                .map(degradation_marker_name)
                .map(str::to_string)
                .collect(),
            witness_count: context.scheduled.execution.witnesses.len(),
            witness_node_ids: context
                .scheduled
                .execution
                .witnesses
                .iter()
                .map(|witness| witness.node_id.clone())
                .collect(),
            certificate_count: context.scheduled.execution.certificates.len(),
            certificate_node_ids: context
                .scheduled
                .execution
                .certificates
                .iter()
                .map(|certificate| certificate.node_id.clone())
                .collect(),
            residual_count: context.scheduled.execution.residuals.len(),
            residual_micros: context
                .scheduled
                .execution
                .residuals
                .iter()
                .map(|sample| sample.total_residual_micros)
                .collect(),
            syndrome_signatures: context
                .scheduled
                .execution
                .syndromes
                .iter()
                .map(|syndrome| syndrome.signature.clone())
                .collect(),
            calibration_caveats: context
                .scheduled
                .execution
                .calibration_report
                .as_ref()
                .map(|report| {
                    report
                        .caution_markers
                        .iter()
                        .map(|marker| marker.replace('_', " "))
                        .collect()
                })
                .unwrap_or_default(),
            refutation_outcome: refutation_outcome_name(&context.causal_refutation.outcome)
                .to_string(),
            causal_refutation_outcome: refutation_outcome_name(&context.causal_refutation.outcome)
                .to_string(),
            minimal_perturbation_outcome: refutation_outcome_name(
                &context.minimal_perturbation.outcome,
            )
            .to_string(),
            oracle_mode: oracle_mode_name(&context.oracle).to_string(),
            oracle_supported: context.oracle.supported,
            advisory_only: context.scheduled.execution.advisory_only,
            degraded_reason: context.scheduled.degraded_reason.clone(),
        }))
    }

    /// Gate risk-influencing use of runtime kernel output on explicit artifacts.
    pub async fn latest_risk_gate_decision(
        &self,
        scope: Option<&Scope>,
    ) -> Result<Option<RiskGateDecision>, RuntimeError> {
        let scope = scope.unwrap_or(self.default_scope());
        let Some(context) = self.latest_kernel_advisory_context(scope).await? else {
            return Ok(None);
        };
        let mut reasons = Vec::new();
        if !context.oracle.supported {
            reasons.push("oracle_not_supported".to_string());
        }
        if !context.compiled.degradations.is_empty() {
            reasons.push("degradation_active".to_string());
        }
        if let Some(reason) = &context.scheduled.degraded_reason {
            reasons.push(format!("scheduled_degraded:{reason}"));
        }
        if context.scheduled.execution.witnesses.is_empty() {
            reasons.push("missing_witness".to_string());
        }
        if context.scheduled.execution.certificates.is_empty() {
            reasons.push("missing_certificate".to_string());
        }
        if context
            .scheduled
            .execution
            .calibration_report
            .as_ref()
            .is_some_and(|report| !report.caution_markers.is_empty())
        {
            reasons.push("calibration_caveat_active".to_string());
        }
        if matches!(
            context.causal_refutation.outcome,
            OracleRefutationOutcome::NotApplicable { .. }
        ) || matches!(
            context.minimal_perturbation.outcome,
            OracleRefutationOutcome::NotApplicable { .. }
        ) {
            reasons.push("refutation_harness_incomplete".to_string());
        }

        Ok(Some(RiskGateDecision {
            batch_id: context.import_log.batch_id,
            source_envelope_id: context.import_log.source_envelope_id,
            status: if reasons.is_empty() {
                "allowed".to_string()
            } else {
                "blocked".to_string()
            },
            reasons,
            advisory_only: context.scheduled.execution.advisory_only,
            degraded_reason: context.scheduled.degraded_reason.clone(),
        }))
    }

    /// Compatibility wrapper for callers using the earlier method name.
    pub async fn latest_risk_gate(
        &self,
        scope: Option<&Scope>,
    ) -> Result<Option<RiskGateDecision>, RuntimeError> {
        self.latest_risk_gate_decision(scope).await
    }
}
