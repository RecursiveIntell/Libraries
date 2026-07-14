//! Act phase of the OODA loop.
//!
//! Executes the selected verification plan (oracle evaluation or
//! paired-patch experiment) and produces an evidence bundle for the
//! canonical Forge export path.

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
    select_backend, CargoAdapter, CausalAttributionEngine, ExperimentConfig,
    ExperimentEvidenceBundle, ForgeStore, ProjectAdapter, StructuredPatch,
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

/// An advisory-only plan that produces no promotable evidence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdvisoryPlan {
    pub description: String,
}

/// The concrete execution plan selected for a verification target.
///
/// Each variant determines which kernel oracle or paired-patch path
/// will be used during the act phase.
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

/// Broad classification of the action executed during the act phase.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ActionFamily {
    Oracle,
    PairedPatch,
    AdvisoryOnly,
}

/// Results from running kernel oracle evaluation against compiled constraints.
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

/// Results from running a paired-patch experiment through the Forge engine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatchExecution {
    pub run_id: String,
    pub improvements: u32,
    pub regressions: u32,
    pub cea_receipt_digests: Vec<String>,
    pub ablation_receipt_digests: Vec<String>,
    pub patch_digest: Option<String>,
    pub degradation_reasons: Vec<String>,
}

/// Full outcome of executing a plan, including the evidence bundle and execution details.
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

/// Executes a verification plan under an issued execution permit.
///
/// Dispatches to the appropriate oracle or paired-patch path based on
/// the plan variant. Returns an error if the permit scope does not
/// match `target_key`.
pub async fn execute_plan(
    observation: &Observation,
    target_key: &str,
    plan: &PlanKind,
    permit: &ExecutionPermit,
    config: &LoopConfig,
    forge_store: &ForgeStore,
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
                forge_store,
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
        // LIB-HIGH-001: replaced unreachable!() with recoverable error
        _ => {
            return Err(PilotError::Other(format!(
                "unsupported plan kind for oracle execution: {:?}",
                plan
            )))
        }
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

#[allow(clippy::too_many_arguments)]
async fn execute_patch_plan(
    observation: &Observation,
    target_key: &str,
    plan: &PlanKind,
    fixture_path: &str,
    patch: &StructuredPatch,
    experiment_config: &ExperimentConfig,
    config: &LoopConfig,
    forge_store: &ForgeStore,
) -> Result<ActionOutcome, PilotError> {
    let backend = select_backend(&config.forge_config)?;
    let fixture = Path::new(fixture_path);
    if !CargoAdapter::detect(fixture) {
        return Err(PilotError::UnsupportedPatchFixture {
            fixture_path: fixture_path.to_string(),
        });
    }
    let adapter = CargoAdapter;
    let engine = CausalAttributionEngine::new(
        forge_store,
        backend.as_ref(),
        &adapter,
        &config.forge_config,
        "forge-pilot.code-model.v1",
    );
    let eval_id = format!("forge-pilot:cea:{}", uuid::Uuid::new_v4());
    let causal_result = engine
        .run_and_observe(fixture, patch, experiment_config, &eval_id)
        .await?;
    let ablation_receipts = engine.run_singleton_ablations(fixture, patch).await?;
    let bundle = build_bundle_from_patch(PatchBundleInput {
        plan,
        target_key,
        trace_id: observation
            .batch
            .as_ref()
            .and_then(|batch| batch.trace_ctx.as_ref().map(|ctx| ctx.trace_id.clone())),
        scope_namespace: &observation.scope_key.namespace,
        causal_result: &causal_result,
        ablation_receipts: &ablation_receipts,
        known_threats: observation
            .degradations
            .iter()
            .map(|degradation| degradation.kind.clone())
            .collect(),
    })?;

    Ok(ActionOutcome {
        family: ActionFamily::PairedPatch,
        plan: plan.clone(),
        bundle: Some(bundle),
        oracle_execution: None,
        patch_execution: Some(PatchExecution {
            run_id: causal_result.experiment.run_id.clone(),
            improvements: causal_result.experiment.diff.improvements,
            regressions: causal_result.experiment.diff.regressions,
            cea_receipt_digests: causal_result
                .receipts
                .iter()
                .map(|receipt| receipt.receipt_digest.clone())
                .collect(),
            ablation_receipt_digests: ablation_receipts
                .iter()
                .map(|receipt| receipt.receipt_digest.clone())
                .collect(),
            patch_digest: causal_result
                .receipts
                .first()
                .map(|receipt| receipt.patch_digest.clone()),
            degradation_reasons: causal_result
                .receipts
                .iter()
                .flat_map(|receipt| receipt.degradation_reasons.clone())
                .chain(
                    ablation_receipts
                        .iter()
                        .flat_map(|receipt| receipt.degradation_reasons.clone()),
                )
                .collect(),
        }),
        advisory_only: false,
        outcome_signature: format!(
            "patch:improvements={} regressions={}",
            causal_result.experiment.diff.improvements, causal_result.experiment.diff.regressions
        ),
    })
}
