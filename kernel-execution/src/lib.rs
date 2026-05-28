#![cfg_attr(test, allow(clippy::expect_used))]

use constraint_compiler::{CompileOutput, CompiledRegion, InvalidationCone};
use recursive_kernel_core::{ArtifactAuthorityClass, Syndrome};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use stack_ids::{ContentDigest, ConvergenceReportId, RegionId, SyndromeId};
use std::collections::{BTreeMap, BTreeSet};

const FULL_CONFIDENCE_MICROS: u64 = 1_000_000;
const DEFAULT_FIXED_POINT_TOLERANCE_MICROS: u64 = 1_000;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionMode {
    AcyclicBaseline,
    MessagePassingBaseline,
    DeltaPropagation,
    ResidualCorrection,
    MultiscaleScheduler,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionStopReason {
    AcyclicCompletion,
    FixedPoint,
    MaxIterations,
    BudgetExhausted,
    DeltaWindowCompleted,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SchedulerStageKind {
    AcyclicPass,
    MessagePassing,
    DeltaPropagation,
    ResidualCorrection,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum WorkloadClass {
    Interactive,
    Background,
    Heavy,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ExecutionBudget {
    pub max_iterations: u32,
    pub max_messages: usize,
    pub max_nodes: usize,
    pub allow_repair: bool,
}

impl Default for ExecutionBudget {
    fn default() -> Self {
        Self {
            max_iterations: 6,
            max_messages: 256,
            max_nodes: 32,
            allow_repair: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct NodeBelief {
    pub node_id: String,
    pub local_support_count: usize,
    pub belief_micros: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ConstraintMessage {
    pub iteration: u32,
    pub from_constraint_id: String,
    pub to_node_id: String,
    pub support_micros: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ResidualSample {
    pub iteration: u32,
    pub total_residual_micros: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ExecutionWitnessArtifact {
    pub node_id: String,
    pub supporting_constraint_ids: Vec<String>,
    pub belief_micros: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ExecutionCertificateArtifact {
    pub node_id: String,
    pub certificate_kind: String,
    pub supporting_constraint_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ExecutionCalibrationReport {
    pub nuisance_node_ids: Vec<String>,
    pub caution_markers: Vec<String>,
    pub degraded: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RecomputeTrigger {
    LineageDelta,
    ValidTimeSliceChange,
    RecordedTimeSliceChange,
    RepairDecision,
    RollbackQuarantine,
    ExplicitInvalidationManifest,
    NuisanceStateUpdate,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct InvalidationManifest {
    pub trigger: RecomputeTrigger,
    pub changed_node_ids: Vec<String>,
    pub changed_region_ids: Vec<RegionId>,
    pub explicit_global_rebuild: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct RegionExecutionTrace {
    pub region_id: RegionId,
    pub region_digest: String,
    pub executed_node_ids: Vec<String>,
    pub witness_node_ids: Vec<String>,
    pub syndrome_ids: Vec<SyndromeId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ConvergenceGovernance {
    pub damping_factor_micros: u64,
    pub residual_tolerance_micros: u64,
    pub max_iterations: u32,
    pub stop_rule: String,
    pub escalation_rule: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ConvergenceReport {
    pub convergence_report_id: ConvergenceReportId,
    pub governance: ConvergenceGovernance,
    pub residual_monotone_nonincreasing: bool,
    pub converged: bool,
    pub escalated: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct DeltaPropagationReport {
    pub changed_node_ids: Vec<String>,
    pub recomputed_node_ids: Vec<String>,
    pub recomputed_region_ids: Vec<RegionId>,
    pub invalidation_manifest: InvalidationManifest,
    pub region_traces: Vec<RegionExecutionTrace>,
    pub execution: ExecutionReport,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ScheduledExecution {
    pub workload_class: WorkloadClass,
    pub stage_kinds: Vec<SchedulerStageKind>,
    pub degraded_reason: Option<String>,
    pub execution: ExecutionReport,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ExecutionReport {
    pub execution_mode: ExecutionMode,
    pub stop_reason: ExecutionStopReason,
    pub iteration_count: u32,
    pub messages: Vec<ConstraintMessage>,
    pub node_beliefs: Vec<NodeBelief>,
    pub residuals: Vec<ResidualSample>,
    pub syndromes: Vec<Syndrome>,
    pub witnesses: Vec<ExecutionWitnessArtifact>,
    pub certificates: Vec<ExecutionCertificateArtifact>,
    pub region_traces: Vec<RegionExecutionTrace>,
    pub calibration_report: Option<ExecutionCalibrationReport>,
    pub convergence_report: ConvergenceReport,
    pub advisory_only: bool,
}

impl ExecutionReport {
    /// Returns the authority class for all execution reports emitted by this crate.
    pub fn authority_class(&self) -> ArtifactAuthorityClass {
        ArtifactAuthorityClass::NonAuthoritativeDerived
    }
}

/// Executes one acyclic belief pass over the compiled graph.
pub fn execute_acyclic_baseline(compiled: &CompileOutput) -> ExecutionReport {
    let initial = initial_beliefs(compiled);
    let step = message_iteration(compiled, &initial, None, 1);
    finalize_execution(
        compiled,
        ExecutionMode::AcyclicBaseline,
        ExecutionStopReason::AcyclicCompletion,
        1,
        step.messages,
        vec![ResidualSample {
            iteration: 1,
            total_residual_micros: total_residual(&initial, &step.beliefs),
        }],
        step.beliefs,
    )
}

/// Executes bounded message passing across the full compiled graph.
pub fn execute_message_passing_baseline(
    compiled: &CompileOutput,
    max_iterations: u32,
) -> ExecutionReport {
    execute_message_passing_internal(
        compiled,
        None,
        max_iterations,
        ExecutionMode::MessagePassingBaseline,
    )
}

/// Recomputes only the regions affected by a delta and reports parity metadata.
pub fn execute_delta_propagation(
    compiled: &CompileOutput,
    changed_node_ids: &[String],
    max_iterations: u32,
) -> DeltaPropagationReport {
    let recomputed_node_ids =
        affected_nodes_for_delta(&compiled.invalidation_cones, changed_node_ids);
    let execution = execute_message_passing_internal(
        compiled,
        Some(&recomputed_node_ids),
        max_iterations,
        ExecutionMode::DeltaPropagation,
    );
    let recomputed_region_ids = affected_regions_for_nodes(&compiled.regions, &recomputed_node_ids);
    let invalidation_manifest = InvalidationManifest {
        trigger: RecomputeTrigger::LineageDelta,
        changed_node_ids: changed_node_ids.to_vec(),
        changed_region_ids: recomputed_region_ids.clone(),
        explicit_global_rebuild: false,
    };
    let region_traces = build_region_traces(
        &compiled.regions,
        &recomputed_node_ids,
        &execution.witnesses,
        &execution.syndromes,
    );

    DeltaPropagationReport {
        changed_node_ids: changed_node_ids.to_vec(),
        recomputed_node_ids,
        recomputed_region_ids,
        invalidation_manifest,
        region_traces,
        execution: ExecutionReport {
            stop_reason: ExecutionStopReason::DeltaWindowCompleted,
            ..execution
        },
    }
}

/// Runs residual-correction passes on top of the message-passing baseline.
pub fn execute_residual_correction(
    compiled: &CompileOutput,
    max_iterations: u32,
) -> ExecutionReport {
    let baseline = execute_message_passing_baseline(compiled, max_iterations.max(1));
    let mut beliefs = belief_map(&baseline.node_beliefs);
    let mut residuals = baseline.residuals.clone();
    let mut stop_reason = ExecutionStopReason::FixedPoint;
    let max_iterations = max_iterations.max(1);
    let mut last_iteration = baseline.iteration_count;

    for iteration in 1..=max_iterations {
        let sample_iteration = baseline.iteration_count + iteration;
        let mut next = beliefs.clone();
        for hyperedge in &compiled.hyperedges {
            let active_members = hyperedge
                .member_node_ids
                .iter()
                .filter(|node_id| !node_id.starts_with("nuisance:"))
                .cloned()
                .collect::<Vec<_>>();
            if active_members.len() < 2 {
                continue;
            }
            let average = active_members
                .iter()
                .filter_map(|node_id| beliefs.get(node_id).copied())
                .sum::<u64>()
                / active_members.len() as u64;
            for node_id in active_members {
                next.insert(node_id, average);
            }
        }

        let residual = total_residual(&beliefs, &next);
        residuals.push(ResidualSample {
            iteration: sample_iteration,
            total_residual_micros: residual,
        });
        beliefs = next;
        last_iteration = sample_iteration;
        if iteration == max_iterations {
            stop_reason = ExecutionStopReason::MaxIterations;
            break;
        }
        if residual <= DEFAULT_FIXED_POINT_TOLERANCE_MICROS {
            stop_reason = ExecutionStopReason::FixedPoint;
            break;
        }
    }

    finalize_execution(
        compiled,
        ExecutionMode::ResidualCorrection,
        stop_reason,
        last_iteration,
        baseline.messages,
        residuals,
        beliefs,
    )
}

/// Chooses the execution lane that fits the supplied graph and budget.
pub fn schedule_execution(
    compiled: &CompileOutput,
    budget: &ExecutionBudget,
) -> ScheduledExecution {
    if compiled.nodes.len() > budget.max_nodes {
        let degraded = finalize_execution(
            compiled,
            ExecutionMode::MultiscaleScheduler,
            ExecutionStopReason::BudgetExhausted,
            0,
            Vec::new(),
            Vec::new(),
            initial_beliefs(compiled),
        );
        return ScheduledExecution {
            workload_class: WorkloadClass::Heavy,
            stage_kinds: vec![],
            degraded_reason: Some("budget_exhausted".into()),
            execution: degraded,
        };
    }

    if !compiled.invalidation_cones.is_empty() && compiled.nodes.len() > 8 {
        let execution = execute_message_passing_baseline(compiled, budget.max_iterations);
        return ScheduledExecution {
            workload_class: WorkloadClass::Background,
            stage_kinds: vec![SchedulerStageKind::MessagePassing],
            degraded_reason: Some("explicit_changed_nodes_required_for_delta".into()),
            execution,
        };
    }

    if compiled.hyperedges.is_empty() {
        let execution = execute_acyclic_baseline(compiled);
        return ScheduledExecution {
            workload_class: WorkloadClass::Interactive,
            stage_kinds: vec![SchedulerStageKind::AcyclicPass],
            degraded_reason: None,
            execution,
        };
    }

    let mut stage_kinds = vec![SchedulerStageKind::MessagePassing];
    let mut execution = execute_message_passing_baseline(compiled, budget.max_iterations);
    if budget.allow_repair && (compiled.hyperedges.len() > 1 || compiled.nodes.len() > 8) {
        stage_kinds.push(SchedulerStageKind::ResidualCorrection);
        execution = execute_residual_correction(compiled, budget.max_iterations);
    }
    ScheduledExecution {
        workload_class: WorkloadClass::Background,
        stage_kinds,
        degraded_reason: None,
        execution,
    }
}

fn execute_message_passing_internal(
    compiled: &CompileOutput,
    focus_nodes: Option<&[String]>,
    max_iterations: u32,
    mode: ExecutionMode,
) -> ExecutionReport {
    let mut beliefs = initial_beliefs(compiled);
    let mut messages = Vec::new();
    let mut residuals = Vec::new();
    let mut stop_reason = ExecutionStopReason::FixedPoint;
    let focused = focus_nodes.map(|nodes| nodes.iter().cloned().collect::<BTreeSet<_>>());
    let iterations = max_iterations.max(1);
    let mut performed = 0;

    for iteration in 1..=iterations {
        performed = iteration;
        let step = message_iteration(compiled, &beliefs, focused.as_ref(), iteration);
        let residual = total_residual(&beliefs, &step.beliefs);
        residuals.push(ResidualSample {
            iteration,
            total_residual_micros: residual,
        });
        messages.extend(step.messages);
        if iteration == iterations {
            beliefs = step.beliefs;
            stop_reason = ExecutionStopReason::MaxIterations;
            break;
        }
        if residual <= DEFAULT_FIXED_POINT_TOLERANCE_MICROS {
            beliefs = step.beliefs;
            break;
        }
        beliefs = step.beliefs;
    }

    finalize_execution(
        compiled,
        mode,
        stop_reason,
        performed,
        messages,
        residuals,
        beliefs,
    )
}

fn finalize_execution(
    compiled: &CompileOutput,
    execution_mode: ExecutionMode,
    stop_reason: ExecutionStopReason,
    iteration_count: u32,
    messages: Vec<ConstraintMessage>,
    residuals: Vec<ResidualSample>,
    beliefs: BTreeMap<String, u64>,
) -> ExecutionReport {
    let node_beliefs = belief_rows(compiled, &beliefs);
    let syndromes = emit_syndromes(compiled, &beliefs);
    let witnesses = emit_witnesses(compiled, &beliefs);
    let certificates = emit_certificates(&witnesses);
    let region_traces = build_region_traces(
        &compiled.regions,
        &compiled
            .nodes
            .iter()
            .map(|node| node.node_id.clone())
            .collect::<Vec<_>>(),
        &witnesses,
        &syndromes,
    );
    let calibration_report = emit_calibration_report(compiled);
    let convergence_report =
        emit_convergence_report(stop_reason.clone(), iteration_count, &residuals);

    ExecutionReport {
        execution_mode,
        stop_reason,
        iteration_count,
        messages,
        node_beliefs,
        residuals,
        syndromes,
        witnesses,
        certificates,
        region_traces,
        calibration_report,
        convergence_report,
        advisory_only: true,
    }
}

struct IterationStep {
    beliefs: BTreeMap<String, u64>,
    messages: Vec<ConstraintMessage>,
}

fn message_iteration(
    compiled: &CompileOutput,
    current: &BTreeMap<String, u64>,
    focus_nodes: Option<&BTreeSet<String>>,
    iteration: u32,
) -> IterationStep {
    let mut next = current.clone();
    let mut messages = Vec::new();
    let base = initial_beliefs(compiled);

    for node in &compiled.nodes {
        if let Some(focused) = focus_nodes {
            if !focused.contains(&node.node_id) {
                continue;
            }
        }

        let local = base.get(&node.node_id).copied().unwrap_or(0);
        let peers = peer_beliefs(compiled, current, &node.node_id);
        let peer_average = if peers.is_empty() {
            local
        } else {
            peers.iter().sum::<u64>() / peers.len() as u64
        };
        let next_value = if node.node_id.starts_with("nuisance:") {
            peer_average.max(local)
        } else {
            (local + peer_average) / 2
        };
        next.insert(node.node_id.clone(), next_value);
    }

    for constraint in &compiled.constraints {
        let signal = constraint
            .variable_ids
            .iter()
            .filter_map(|node_id| next.get(node_id).copied())
            .sum::<u64>()
            / constraint.variable_ids.len().max(1) as u64;
        for node_id in &constraint.variable_ids {
            messages.push(ConstraintMessage {
                iteration,
                from_constraint_id: constraint.constraint_id.as_str().to_string(),
                to_node_id: node_id.clone(),
                support_micros: signal,
            });
        }
    }

    IterationStep {
        beliefs: next,
        messages,
    }
}

fn initial_beliefs(compiled: &CompileOutput) -> BTreeMap<String, u64> {
    let support = local_support_by_node(compiled);
    let max_support = support.values().copied().max().unwrap_or(1).max(1);
    compiled
        .nodes
        .iter()
        .map(|node| {
            let belief = support
                .get(&node.node_id)
                .copied()
                .unwrap_or(0)
                .saturating_mul(FULL_CONFIDENCE_MICROS)
                / max_support;
            (node.node_id.clone(), belief)
        })
        .collect()
}

fn local_support_by_node(compiled: &CompileOutput) -> BTreeMap<String, u64> {
    let mut support = BTreeMap::new();
    for node in &compiled.nodes {
        let local_constraints = compiled
            .constraints
            .iter()
            .filter(|constraint| constraint.variable_ids.iter().any(|id| id == &node.node_id))
            .count() as u64;
        let local_hyperedges = compiled
            .hyperedges
            .iter()
            .filter(|hyperedge| {
                hyperedge
                    .member_node_ids
                    .iter()
                    .any(|id| id == &node.node_id)
            })
            .count() as u64;
        support.insert(
            node.node_id.clone(),
            (local_constraints + local_hyperedges).max(1),
        );
    }
    support
}

fn peer_beliefs(
    compiled: &CompileOutput,
    current: &BTreeMap<String, u64>,
    node_id: &str,
) -> Vec<u64> {
    let mut peers = Vec::new();
    for hyperedge in &compiled.hyperedges {
        if !hyperedge
            .member_node_ids
            .iter()
            .any(|member| member == node_id)
        {
            continue;
        }
        for member in &hyperedge.member_node_ids {
            if member == node_id {
                continue;
            }
            if let Some(value) = current.get(member) {
                peers.push(*value);
            }
        }
    }
    peers
}

fn total_residual(previous: &BTreeMap<String, u64>, next: &BTreeMap<String, u64>) -> u64 {
    next.iter()
        .map(|(node_id, next_value)| {
            previous
                .get(node_id)
                .copied()
                .unwrap_or(0)
                .abs_diff(*next_value)
        })
        .sum()
}

fn belief_rows(compiled: &CompileOutput, beliefs: &BTreeMap<String, u64>) -> Vec<NodeBelief> {
    let local_support = local_support_by_node(compiled);
    compiled
        .nodes
        .iter()
        .map(|node| NodeBelief {
            node_id: node.node_id.clone(),
            local_support_count: local_support.get(&node.node_id).copied().unwrap_or(0) as usize,
            belief_micros: beliefs.get(&node.node_id).copied().unwrap_or(0),
        })
        .collect()
}

fn belief_map(rows: &[NodeBelief]) -> BTreeMap<String, u64> {
    rows.iter()
        .map(|row| (row.node_id.clone(), row.belief_micros))
        .collect()
}

fn emit_syndromes(compiled: &CompileOutput, beliefs: &BTreeMap<String, u64>) -> Vec<Syndrome> {
    let mut syndromes = Vec::new();
    for degradation in &compiled.degradations {
        let signature = format!("degradation:{degradation:?}").to_lowercase();
        syndromes.push(Syndrome {
            syndrome_id: SyndromeId::new(format!("syndrome:{signature}")),
            signature,
            blocked_by_degradation: true,
        });
    }

    for constraint in &compiled.constraints {
        let active_non_nuisance = constraint
            .variable_ids
            .iter()
            .filter(|node_id| !node_id.starts_with("nuisance:"))
            .filter(|node_id| beliefs.get(*node_id).copied().unwrap_or(0) >= 250_000)
            .count();
        if active_non_nuisance < 2 && constraint.variable_ids.len() > 1 {
            let digest = ContentDigest::compute(
                format!("{}:{:?}", constraint.kind, constraint.variable_ids).as_bytes(),
            );
            syndromes.push(Syndrome {
                syndrome_id: SyndromeId::new(format!("syndrome:{}", digest.hex())),
                signature: format!("constraint_under_supported:{}", constraint.constraint_id),
                blocked_by_degradation: false,
            });
        }
    }

    syndromes.sort_by(|a, b| a.signature.cmp(&b.signature));
    syndromes
}

fn emit_witnesses(
    compiled: &CompileOutput,
    beliefs: &BTreeMap<String, u64>,
) -> Vec<ExecutionWitnessArtifact> {
    let mut witnesses = Vec::new();
    for node in &compiled.nodes {
        if node.node_id.starts_with("nuisance:") {
            continue;
        }
        let belief = beliefs.get(&node.node_id).copied().unwrap_or(0);
        if belief < 500_000 {
            continue;
        }
        let mut supporting_constraint_ids = compiled
            .constraints
            .iter()
            .filter(|constraint| constraint.variable_ids.iter().any(|id| id == &node.node_id))
            .map(|constraint| constraint.constraint_id.as_str().to_string())
            .collect::<Vec<_>>();
        supporting_constraint_ids.sort();
        witnesses.push(ExecutionWitnessArtifact {
            node_id: node.node_id.clone(),
            supporting_constraint_ids,
            belief_micros: belief,
        });
    }
    witnesses
}

fn emit_certificates(witnesses: &[ExecutionWitnessArtifact]) -> Vec<ExecutionCertificateArtifact> {
    witnesses
        .iter()
        .map(|witness| ExecutionCertificateArtifact {
            node_id: witness.node_id.clone(),
            certificate_kind: if witness.supporting_constraint_ids.len() > 1 {
                "multi_constraint_support".into()
            } else {
                "single_constraint_support".into()
            },
            supporting_constraint_count: witness.supporting_constraint_ids.len(),
        })
        .collect()
}

fn emit_calibration_report(compiled: &CompileOutput) -> Option<ExecutionCalibrationReport> {
    let nuisance_node_ids = compiled
        .nodes
        .iter()
        .filter(|node| node.kind == "nuisance_state")
        .map(|node| node.node_id.clone())
        .collect::<Vec<_>>();

    if nuisance_node_ids.is_empty() && compiled.degradations.is_empty() {
        return None;
    }

    let mut caution_markers = Vec::new();
    if !nuisance_node_ids.is_empty() {
        caution_markers.push("nuisance_state_present".into());
    }
    if !compiled.degradations.is_empty() {
        caution_markers.push("degraded_export".into());
    }
    Some(ExecutionCalibrationReport {
        nuisance_node_ids,
        caution_markers,
        degraded: !compiled.degradations.is_empty(),
    })
}

fn build_region_traces(
    regions: &[CompiledRegion],
    executed_node_ids: &[String],
    witnesses: &[ExecutionWitnessArtifact],
    syndromes: &[Syndrome],
) -> Vec<RegionExecutionTrace> {
    let executed = executed_node_ids.iter().cloned().collect::<BTreeSet<_>>();
    let syndrome_ids = syndromes
        .iter()
        .map(|syndrome| syndrome.syndrome_id.clone())
        .collect::<Vec<_>>();

    regions
        .iter()
        .filter_map(|region| {
            let executed_node_ids = region
                .node_ids
                .iter()
                .filter(|node_id| executed.contains(*node_id))
                .cloned()
                .collect::<Vec<_>>();
            if executed_node_ids.is_empty() {
                return None;
            }

            let witness_node_ids = witnesses
                .iter()
                .filter(|witness| {
                    region
                        .node_ids
                        .iter()
                        .any(|node_id| node_id == &witness.node_id)
                })
                .map(|witness| witness.node_id.clone())
                .collect::<Vec<_>>();

            Some(RegionExecutionTrace {
                region_id: region.region_id.clone(),
                region_digest: region.region_digest_id.to_string(),
                executed_node_ids,
                witness_node_ids,
                syndrome_ids: syndrome_ids.clone(),
            })
        })
        .collect()
}

fn emit_convergence_report(
    stop_reason: ExecutionStopReason,
    iteration_count: u32,
    residuals: &[ResidualSample],
) -> ConvergenceReport {
    let residual_monotone_nonincreasing = residuals
        .windows(2)
        .all(|window| window[1].total_residual_micros <= window[0].total_residual_micros);
    let converged = matches!(
        stop_reason,
        ExecutionStopReason::AcyclicCompletion
            | ExecutionStopReason::FixedPoint
            | ExecutionStopReason::DeltaWindowCompleted
    );
    let escalated = matches!(
        stop_reason,
        ExecutionStopReason::BudgetExhausted | ExecutionStopReason::MaxIterations
    ) && iteration_count > 0;

    let convergence_payload =
        serde_json::to_vec(&(stop_reason.clone(), iteration_count, residuals)).unwrap_or_else(
            |_| format!("{stop_reason:?}:{iteration_count}:{}", residuals.len()).into_bytes(),
        );
    let digest = ContentDigest::compute(&convergence_payload);

    ConvergenceReport {
        convergence_report_id: ConvergenceReportId::new(format!("convergence:{}", digest.hex())),
        governance: ConvergenceGovernance {
            damping_factor_micros: FULL_CONFIDENCE_MICROS,
            residual_tolerance_micros: DEFAULT_FIXED_POINT_TOLERANCE_MICROS,
            max_iterations: iteration_count.max(1),
            stop_rule: "fixed_point_or_explicit_stop".into(),
            escalation_rule: "emit_failure_artifact_on_nonconvergence".into(),
        },
        residual_monotone_nonincreasing,
        converged,
        escalated,
    }
}

fn affected_regions_for_nodes(regions: &[CompiledRegion], node_ids: &[String]) -> Vec<RegionId> {
    let changed = node_ids.iter().cloned().collect::<BTreeSet<_>>();
    regions
        .iter()
        .filter(|region| {
            region
                .node_ids
                .iter()
                .any(|node_id| changed.contains(node_id))
        })
        .map(|region| region.region_id.clone())
        .collect()
}

/// Computes the node set that must be recomputed for a changed-node delta.
pub fn affected_nodes_for_delta(
    invalidation_cones: &[InvalidationCone],
    changed_node_ids: &[String],
) -> Vec<String> {
    let mut affected = changed_node_ids.iter().cloned().collect::<BTreeSet<_>>();
    for changed in changed_node_ids {
        if let Some(cone) = invalidation_cones
            .iter()
            .find(|cone| cone.source_node_id == *changed)
        {
            affected.extend(cone.affected_node_ids.iter().cloned());
        }
    }
    affected.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use constraint_compiler::{
        CompilationBoundary, CompileOutput, CompiledRegion, ConstraintDegradation,
        GraphGeometryManifest, GraphSurfaceKind, InferenceHyperedge, InferenceNode,
        InvalidationCone, OracleSliceCandidate,
    };
    use recursive_kernel_core::{ConstraintUnit, CONSTRAINT_COMPILER_OPERATOR_ID};
    use stack_ids::{ConstraintId, OperatorId, OracleSliceId, RegionDigestId, RegionId, ScopeKey};

    fn compiled_fixture() -> CompileOutput {
        CompileOutput {
            graph_hash: ContentDigest::compute(b"kernel-execution-fixture"),
            scope_key: ScopeKey::namespace_only("kernel-execution"),
            geometry_manifest: GraphGeometryManifest {
                surfaces: vec![
                    GraphSurfaceKind::Storage,
                    GraphSurfaceKind::Retrieval,
                    GraphSurfaceKind::Inference,
                    GraphSurfaceKind::Repair,
                    GraphSurfaceKind::Control,
                ],
                compilation_boundaries: vec![CompilationBoundary {
                    from_surface: GraphSurfaceKind::Inference,
                    to_surface: GraphSurfaceKind::Repair,
                    artifact_families: vec!["syndrome".into()],
                    deterministic: true,
                }],
                no_silent_collapse: true,
            },
            nodes: vec![
                InferenceNode {
                    node_id: "node-a".into(),
                    kind: "claim_version".into(),
                },
                InferenceNode {
                    node_id: "node-b".into(),
                    kind: "claim_version".into(),
                },
                InferenceNode {
                    node_id: "nuisance:comparability:v1".into(),
                    kind: "nuisance_state".into(),
                },
            ],
            hyperedges: vec![
                InferenceHyperedge {
                    edge_id: "assertion_group:group-1".into(),
                    member_node_ids: vec!["node-a".into(), "node-b".into()],
                },
                InferenceHyperedge {
                    edge_id: "nuisance_edge:node-a:nuisance:comparability:v1".into(),
                    member_node_ids: vec!["node-a".into(), "nuisance:comparability:v1".into()],
                },
            ],
            constraints: vec![
                ConstraintUnit {
                    constraint_id: ConstraintId::new("constraint:assertion_group:group-1"),
                    kind: "hyperedge".into(),
                    variable_ids: vec!["node-a".into(), "node-b".into()],
                    operator_id: OperatorId::new(CONSTRAINT_COMPILER_OPERATOR_ID),
                },
                ConstraintUnit {
                    constraint_id: ConstraintId::new(
                        "constraint:nuisance_edge:node-a:nuisance:comparability:v1",
                    ),
                    kind: "nuisance_disclosure".into(),
                    variable_ids: vec!["node-a".into(), "nuisance:comparability:v1".into()],
                    operator_id: OperatorId::new(CONSTRAINT_COMPILER_OPERATOR_ID),
                },
            ],
            regions: vec![CompiledRegion {
                region_id: RegionId::new("region:fixture"),
                region_digest_id: RegionDigestId::new("region-digest:fixture"),
                node_ids: vec![
                    "node-a".into(),
                    "node-b".into(),
                    "nuisance:comparability:v1".into(),
                ],
                hyperedge_ids: vec![
                    "assertion_group:group-1".into(),
                    "nuisance_edge:node-a:nuisance:comparability:v1".into(),
                ],
                constraint_ids: vec![
                    ConstraintId::new("constraint:assertion_group:group-1"),
                    ConstraintId::new("constraint:nuisance_edge:node-a:nuisance:comparability:v1"),
                ],
                bounded_default_unit_of_work: true,
            }],
            invalidation_cones: vec![InvalidationCone {
                source_node_id: "node-a".into(),
                affected_node_ids: vec!["node-a".into(), "node-b".into()],
                affected_hyperedge_ids: vec!["assertion_group:group-1".into()],
                affected_constraint_ids: vec![ConstraintId::new(
                    "constraint:assertion_group:group-1",
                )],
            }],
            degradations: Vec::<ConstraintDegradation>::new(),
            oracle_candidates: vec![OracleSliceCandidate {
                oracle_slice_id: OracleSliceId::new("oracle:fixture"),
                node_ids: vec!["node-a".into(), "node-b".into()],
            }],
        }
    }

    fn degraded_fixture() -> CompileOutput {
        let mut compiled = compiled_fixture();
        compiled.constraints[0].variable_ids = vec!["node-a".into(), "node-b".into()];
        compiled.degradations = vec![ConstraintDegradation::ThinExport];
        compiled
    }

    fn unequal_supports_fixture() -> CompileOutput {
        let base = compiled_fixture();
        CompileOutput {
            graph_hash: base.graph_hash,
            scope_key: base.scope_key,
            geometry_manifest: base.geometry_manifest,
            nodes: vec![
                InferenceNode {
                    node_id: "node-a".into(),
                    kind: "claim_version".into(),
                },
                InferenceNode {
                    node_id: "node-b".into(),
                    kind: "claim_version".into(),
                },
            ],
            hyperedges: vec![InferenceHyperedge {
                edge_id: "edge:imbalance".into(),
                member_node_ids: vec!["node-a".into(), "node-b".into()],
            }],
            constraints: vec![
                ConstraintUnit {
                    constraint_id: ConstraintId::new("constraint:edge:imbalance"),
                    kind: "hyperedge".into(),
                    variable_ids: vec!["node-a".into(), "node-b".into()],
                    operator_id: OperatorId::new(CONSTRAINT_COMPILER_OPERATOR_ID),
                },
                ConstraintUnit {
                    constraint_id: ConstraintId::new("constraint:node-a-bias"),
                    kind: "node_bias".into(),
                    variable_ids: vec!["node-a".into()],
                    operator_id: OperatorId::new(CONSTRAINT_COMPILER_OPERATOR_ID),
                },
            ],
            regions: vec![CompiledRegion {
                region_id: RegionId::new("region:imbalance"),
                region_digest_id: RegionDigestId::new("region-digest:imbalance"),
                node_ids: vec!["node-a".into(), "node-b".into()],
                hyperedge_ids: vec!["edge:imbalance".into()],
                constraint_ids: vec![
                    ConstraintId::new("constraint:edge:imbalance"),
                    ConstraintId::new("constraint:node-a-bias"),
                ],
                bounded_default_unit_of_work: true,
            }],
            invalidation_cones: Vec::new(),
            degradations: Vec::<ConstraintDegradation>::new(),
            oracle_candidates: Vec::new(),
        }
    }

    #[test]
    fn acyclic_baseline_is_deterministic_and_non_authoritative() {
        let compiled = compiled_fixture();
        let a = execute_acyclic_baseline(&compiled);
        let b = execute_acyclic_baseline(&compiled);

        assert_eq!(a, b);
        assert_eq!(
            a.authority_class(),
            ArtifactAuthorityClass::NonAuthoritativeDerived
        );
        assert_eq!(a.stop_reason, ExecutionStopReason::AcyclicCompletion);
        assert!(!a.witnesses.is_empty());
    }

    #[test]
    fn message_passing_baseline_is_bounded_and_emits_messages() {
        let compiled = compiled_fixture();
        let report = execute_message_passing_baseline(&compiled, 3);

        assert_eq!(report.execution_mode, ExecutionMode::MessagePassingBaseline);
        assert!(report.iteration_count <= 3);
        assert!(!report.messages.is_empty());
        assert!(report
            .node_beliefs
            .iter()
            .all(|belief| belief.belief_micros > 0));
    }

    #[test]
    fn message_passing_respects_iteration_cap_as_stop_rule() {
        let compiled = compiled_fixture();
        let report = execute_message_passing_baseline(&compiled, 1);

        assert_eq!(report.execution_mode, ExecutionMode::MessagePassingBaseline);
        assert_eq!(report.stop_reason, ExecutionStopReason::MaxIterations);
        assert!(report.iteration_count <= 1);
    }

    #[test]
    fn delta_propagation_recomputes_only_affected_slice() {
        let compiled = compiled_fixture();
        let delta = execute_delta_propagation(&compiled, &["node-a".into()], 2);

        assert_eq!(
            delta.execution.execution_mode,
            ExecutionMode::DeltaPropagation
        );
        assert_eq!(
            delta.recomputed_node_ids,
            vec!["node-a".to_string(), "node-b".to_string()]
        );
        assert_eq!(
            affected_nodes_for_delta(&compiled.invalidation_cones, &["node-a".into()]),
            delta.recomputed_node_ids
        );
        assert_eq!(
            delta.recomputed_region_ids,
            vec![RegionId::new("region:fixture")]
        );
        assert!(!delta.region_traces.is_empty());
        assert!(!delta.invalidation_manifest.explicit_global_rebuild);
    }

    #[test]
    fn residual_correction_reduces_residuals_monotonically() {
        let compiled = compiled_fixture();
        let report = execute_residual_correction(&compiled, 3);

        let mut previous = u64::MAX;
        for sample in &report.residuals {
            assert!(sample.total_residual_micros <= previous);
            previous = sample.total_residual_micros;
        }
        assert!(report.convergence_report.residual_monotone_nonincreasing);
    }

    #[test]
    fn residual_correction_reports_max_iterations_stop_reason_when_not_converged() {
        let compiled = unequal_supports_fixture();
        let report = execute_residual_correction(&compiled, 1);

        assert_eq!(report.stop_reason, ExecutionStopReason::MaxIterations);
        assert_eq!(report.iteration_count, 2);
    }

    #[test]
    fn schedule_execution_does_not_fabricate_delta_changed_nodes() {
        let mut compiled = compiled_fixture();
        for index in 0..9 {
            compiled.nodes.push(InferenceNode {
                node_id: format!("node-extra-{index}"),
                kind: "claim_version".into(),
            });
        }

        let scheduled = schedule_execution(
            &compiled,
            &ExecutionBudget {
                max_nodes: 20,
                max_iterations: 2,
                allow_repair: true,
                max_messages: 256,
            },
        );

        assert_eq!(
            scheduled.degraded_reason.as_deref(),
            Some("explicit_changed_nodes_required_for_delta")
        );
        assert_eq!(
            scheduled.stage_kinds,
            vec![SchedulerStageKind::MessagePassing]
        );
        assert_eq!(
            scheduled.execution.execution_mode,
            ExecutionMode::MessagePassingBaseline
        );
    }

    #[test]
    fn scheduler_emits_budget_degradation_and_calibration() {
        let scheduled = schedule_execution(
            &degraded_fixture(),
            &ExecutionBudget {
                max_nodes: 2,
                ..ExecutionBudget::default()
            },
        );

        assert_eq!(
            scheduled.execution.stop_reason,
            ExecutionStopReason::BudgetExhausted
        );
        assert_eq!(
            scheduled.degraded_reason.as_deref(),
            Some("budget_exhausted")
        );
        assert!(scheduled.execution.calibration_report.is_some());
        assert!(scheduled
            .execution
            .syndromes
            .iter()
            .any(|syndrome| syndrome.blocked_by_degradation));
    }

    #[test]
    fn thin_export_execution_surfaces_degradation_artifacts() {
        let mut compiled = compiled_fixture();
        compiled.degradations = vec![ConstraintDegradation::ThinExport];

        let report = execute_message_passing_baseline(&compiled, 3);
        let calibration = report
            .calibration_report
            .expect("calibration report expected for degraded compilation");

        assert!(calibration.degraded);
        assert!(calibration
            .caution_markers
            .iter()
            .any(|marker| marker == "degraded_export"));
        assert!(report
            .syndromes
            .iter()
            .any(|syndrome| syndrome.blocked_by_degradation));
        assert!(report.advisory_only);
    }

    #[test]
    fn execution_emits_region_traces_and_convergence_governance() {
        let compiled = compiled_fixture();
        let report = execute_message_passing_baseline(&compiled, 3);

        assert!(!report.region_traces.is_empty());
        assert_eq!(
            report.region_traces[0].region_id,
            RegionId::new("region:fixture")
        );
        assert_eq!(
            report
                .convergence_report
                .governance
                .residual_tolerance_micros,
            DEFAULT_FIXED_POINT_TOLERANCE_MICROS
        );
        assert_eq!(
            report.convergence_report.governance.escalation_rule,
            "emit_failure_artifact_on_nonconvergence"
        );
    }
}
