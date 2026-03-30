use crate::bundle_builder::{
    build_bundle_from_oracle, build_bundle_from_patch, OracleBundleInput, PatchBundleInput,
};
use crate::config::LoopConfig;
use crate::error::PilotError;
use crate::observe::Observation;
use forge_engine::lab::evidence::{
    RefutationArtifact, RefutationArtifactOutcome, RefutationArtifactType,
};
use forge_engine::{
    select_backend, CargoAdapter, ExperimentConfig, ExperimentEvidenceBundle,
    PairedExperimentRunner, ProjectAdapter, StructuredPatch,
};
use kernel_oracles::{
    evaluate_causal_refuter, evaluate_conservative, evaluate_delta_parity, evaluate_exact_bounded,
    evaluate_minimal_perturbation, evaluate_temporal_replay, DeltaParityAssessment,
    OracleAssessment, OracleRefutationOutcome, OracleRefutationResult, TemporalReplayAssessment,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use stack_ids::{ClaimVersionId, OracleSliceId};
use std::path::Path;
use verification_policy::ExecutionPermit;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdvisoryPlan {
    pub description: String,
}

#[derive(Debug, Clone)]
pub enum PlanKind {
    OracleExactBounded {
        oracle_slice_id: OracleSliceId,
    },
    OracleConservative,
    OracleDeltaParity {
        changed_node_ids: Vec<String>,
        max_iterations: u32,
    },
    OracleTemporalReplay {
        cutoff_recorded_at: String,
    },
    OracleCausalRefuter {
        target_node_id: String,
        max_removed_nodes: usize,
    },
    OracleMinimalPerturbation {
        target_node_id: String,
        max_removed_nodes: usize,
    },
    PairedPatch {
        fixture_path: String,
        patch: StructuredPatch,
        experiment_config: ExperimentConfig,
        description: String,
    },
    AdvisoryOnlyVerificationPlan(AdvisoryPlan),
}

impl PlanKind {
    /// Returns the superseded claim version targeted by this plan, if any.
    pub fn supersedes_claim_version_id(&self) -> Option<ClaimVersionId> {
        None
    }

    /// Returns the verification check names implied by this plan.
    pub fn check_names(&self) -> Vec<String> {
        match self {
            Self::PairedPatch { .. } => vec!["fmt".into(), "clippy".into(), "test".into()],
            _ => vec!["kernel_oracle".into()],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum ActionFamily {
    Oracle,
    PairedPatch,
    AdvisoryOnly,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OracleExecution {
    pub assessment: Option<OracleAssessment>,
    pub delta_parity: Option<DeltaParityAssessment>,
    pub temporal_replay: Option<TemporalReplayAssessment>,
    pub refutation: Option<OracleRefutationResult>,
}

impl OracleExecution {
    /// Returns a short textual summary of the oracle execution outcome.
    pub fn outcome_summary(&self) -> Option<String> {
        if let Some(assessment) = &self.assessment {
            return Some(format!(
                "oracle mode {:?} supported={} satisfied_constraints={}",
                assessment.mode, assessment.supported, assessment.satisfied_constraint_count
            ));
        }
        if let Some(delta) = &self.delta_parity {
            return Some(format!(
                "delta parity matched={} recomputed_nodes={}",
                delta.parity_match,
                delta.recomputed_node_ids.len()
            ));
        }
        if let Some(replay) = &self.temporal_replay {
            return Some(format!(
                "temporal replay matched_expected_hash={}",
                replay.matched_expected_hash
            ));
        }
        self.refutation
            .as_ref()
            .map(|refutation| format!("{:?}", refutation.outcome))
    }

    /// Returns a JSON summary of the oracle execution outcome.
    pub fn summary_json(&self) -> serde_json::Value {
        serde_json::json!({
            "assessment": self.assessment,
            "delta_parity": self.delta_parity,
            "temporal_replay": self.temporal_replay,
            "refutation": self.refutation,
        })
    }

    /// Returns the refutation artifacts emitted by this oracle execution.
    pub fn refutation_artifacts(&self) -> Vec<RefutationArtifact> {
        self.refutation
            .iter()
            .map(|refutation| RefutationArtifact {
                artifact_id: format!("refutation:{}", refutation.target_node_id),
                artifact_type: RefutationArtifactType::SubsampleStability,
                trial_id: None,
                attempt_id: None,
                outcome: match &refutation.outcome {
                    OracleRefutationOutcome::FlipWitness { .. } => {
                        RefutationArtifactOutcome::Passed
                    }
                    OracleRefutationOutcome::NoFlipFound { searched_budget } => {
                        RefutationArtifactOutcome::Failed {
                            reason: format!("no flip found in budget {searched_budget}"),
                        }
                    }
                    OracleRefutationOutcome::NotApplicable { reason } => {
                        RefutationArtifactOutcome::Inconclusive {
                            reason: reason.clone(),
                        }
                    }
                },
                estimate_delta: None,
                details: Some(format!("{:?}", refutation.outcome)),
            })
            .collect()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatchExecution {
    pub run_id: String,
    pub improvements: u32,
    pub regressions: u32,
}

#[derive(Debug, Clone)]
pub struct ActionOutcome {
    pub family: ActionFamily,
    pub plan: PlanKind,
    pub bundle: Option<ExperimentEvidenceBundle>,
    pub oracle_execution: Option<OracleExecution>,
    pub patch_execution: Option<PatchExecution>,
    pub advisory_only: bool,
    pub outcome_signature: String,
}

pub async fn execute_plan(
    observation: &Observation,
    target_key: &str,
    plan: &PlanKind,
    permit: &ExecutionPermit,
    config: &LoopConfig,
) -> Result<ActionOutcome, PilotError> {
    if permit.scope().target_key() != target_key {
        return Err(PilotError::Other(format!(
            "execution permit target {} does not match requested target {}",
            permit.scope().target_key(),
            target_key
        )));
    }
    match plan {
        PlanKind::OracleExactBounded { .. }
        | PlanKind::OracleConservative
        | PlanKind::OracleDeltaParity { .. }
        | PlanKind::OracleTemporalReplay { .. }
        | PlanKind::OracleCausalRefuter { .. }
        | PlanKind::OracleMinimalPerturbation { .. } => {
            execute_oracle_plan(observation, target_key, plan, config).await
        }
        PlanKind::PairedPatch {
            fixture_path,
            patch,
            experiment_config,
            ..
        } => {
            execute_patch_plan(
                observation,
                target_key,
                plan,
                fixture_path,
                patch,
                experiment_config,
                config,
            )
            .await
        }
        PlanKind::AdvisoryOnlyVerificationPlan(_) => Err(PilotError::Other(
            "advisory-only plans cannot consume execution permits".into(),
        )),
    }
}

async fn execute_oracle_plan(
    observation: &Observation,
    target_key: &str,
    plan: &PlanKind,
    _config: &LoopConfig,
) -> Result<ActionOutcome, PilotError> {
    let compiled = observation
        .compiled
        .as_ref()
        .ok_or(PilotError::MissingCompiledContext)?;

    let oracle_execution = match plan {
        PlanKind::OracleExactBounded { .. } => OracleExecution {
            assessment: evaluate_exact_bounded(compiled),
            delta_parity: None,
            temporal_replay: None,
            refutation: None,
        },
        PlanKind::OracleConservative => OracleExecution {
            assessment: Some(evaluate_conservative(compiled)),
            delta_parity: None,
            temporal_replay: None,
            refutation: None,
        },
        PlanKind::OracleDeltaParity {
            changed_node_ids,
            max_iterations,
        } => OracleExecution {
            assessment: None,
            delta_parity: Some(evaluate_delta_parity(
                compiled,
                changed_node_ids,
                *max_iterations,
            )),
            temporal_replay: None,
            refutation: None,
        },
        PlanKind::OracleTemporalReplay { cutoff_recorded_at } => OracleExecution {
            assessment: None,
            delta_parity: None,
            temporal_replay: Some(
                evaluate_temporal_replay(
                    &observation.temporal_snapshots,
                    cutoff_recorded_at,
                    &constraint_compiler::CompilerPolicy {
                        policy_version: "forge-pilot.v1".into(),
                        include_hyperedges: true,
                    },
                    &compiled.graph_hash,
                )
                .ok_or(PilotError::MissingTemporalSnapshots)?,
            ),
            refutation: None,
        },
        PlanKind::OracleCausalRefuter {
            target_node_id,
            max_removed_nodes,
        } => OracleExecution {
            assessment: None,
            delta_parity: None,
            temporal_replay: None,
            refutation: Some(evaluate_causal_refuter(
                compiled,
                target_node_id,
                *max_removed_nodes,
            )),
        },
        PlanKind::OracleMinimalPerturbation {
            target_node_id,
            max_removed_nodes,
        } => OracleExecution {
            assessment: None,
            delta_parity: None,
            temporal_replay: None,
            refutation: Some(evaluate_minimal_perturbation(
                compiled,
                target_node_id,
                *max_removed_nodes,
            )),
        },
        _ => unreachable!(),
    };

    let bundle = build_bundle_from_oracle(OracleBundleInput {
        plan,
        target_key,
        trace_id: observation
            .batch
            .as_ref()
            .and_then(|batch| batch.trace_ctx.as_ref().map(|ctx| ctx.trace_id.clone())),
        scope_namespace: &observation.scope_key.namespace,
        oracle_execution: &oracle_execution,
        known_threats: observation
            .degradations
            .iter()
            .map(|degradation| degradation.kind.clone())
            .collect(),
    });
    let outcome_signature = oracle_execution
        .outcome_summary()
        .unwrap_or_else(|| "oracle".into());

    Ok(ActionOutcome {
        family: ActionFamily::Oracle,
        plan: plan.clone(),
        bundle: Some(bundle),
        oracle_execution: Some(oracle_execution),
        patch_execution: None,
        advisory_only: false,
        outcome_signature,
    })
}

async fn execute_patch_plan(
    observation: &Observation,
    target_key: &str,
    plan: &PlanKind,
    fixture_path: &str,
    patch: &StructuredPatch,
    experiment_config: &ExperimentConfig,
    config: &LoopConfig,
) -> Result<ActionOutcome, PilotError> {
    let backend = select_backend(&config.forge_config)?;
    let fixture = Path::new(fixture_path);
    if !CargoAdapter::detect(fixture) {
        return Err(PilotError::UnsupportedPatchFixture {
            fixture_path: fixture_path.to_string(),
        });
    }
    let adapter = CargoAdapter;
    let runner = PairedExperimentRunner::new(backend.as_ref(), &adapter, &config.forge_config);
    let experiment_result = runner.run(fixture, patch, experiment_config).await?;
    let bundle = build_bundle_from_patch(PatchBundleInput {
        plan,
        target_key,
        trace_id: observation
            .batch
            .as_ref()
            .and_then(|batch| batch.trace_ctx.as_ref().map(|ctx| ctx.trace_id.clone())),
        scope_namespace: &observation.scope_key.namespace,
        experiment_result: &experiment_result,
        known_threats: observation
            .degradations
            .iter()
            .map(|degradation| degradation.kind.clone())
            .collect(),
    });

    Ok(ActionOutcome {
        family: ActionFamily::PairedPatch,
        plan: plan.clone(),
        bundle: Some(bundle),
        oracle_execution: None,
        patch_execution: Some(PatchExecution {
            run_id: experiment_result.run_id.clone(),
            improvements: experiment_result.diff.improvements,
            regressions: experiment_result.diff.regressions,
        }),
        advisory_only: false,
        outcome_signature: format!(
            "patch:improvements={} regressions={}",
            experiment_result.diff.improvements, experiment_result.diff.regressions
        ),
    })
}
