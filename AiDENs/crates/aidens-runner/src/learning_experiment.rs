//! Thin AiDENs composition over Forge's canonical repeated-paired experiment owner.
//!
//! This module does not compute statistics, mint promotion evidence, or persist a
//! second experiment truth. It exposes the owner-native `ExperimentResult` values
//! for later owner-side adjudication.

use crate::learning_corpus::{validate_and_load_v2_evaluator_tasks, CorpusError, V2EvaluatorTask};
use aidens_receipts::{CanonicalEventLog, CanonicalEventLogConfig};
use forge_engine::lab::evaluate::ScoreVector;
use forge_engine::ExperimentEvidenceBundle;
use forge_engine::{
    select_backend, CargoAdapter, ExecutionBackend, ExecutionBackendKind, ExperimentConfig,
    ExperimentMode, ExperimentResult, ForgeConfig, ForgeError, PairedExperimentRunner,
    ProjectAdapter, StatisticsPolicy,
};
use forge_memory_bridge::ForgeAdjudicationStore;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use stack_ids::ContentDigest;
use std::collections::BTreeMap;
use std::path::PathBuf;
use verification_adjudication::{
    adjudicate_candidate, CandidatePromotionInput, FamilyGateV1, FrozenPromotionThresholdsV1,
    HoldoutGateV1, IdentityDigest, ReceiptRef, UncertaintyV1,
};

const MIN_TOTAL_PAIRS: u32 = 20;

#[derive(Debug, Clone)]
pub struct V2RepeatedPairedConfig {
    pub corpus_root: PathBuf,
    pub pairs_per_task: u32,
    pub trial_order_seed: u64,
    pub forge: ForgeConfig,
}

#[derive(Debug)]
pub struct V2TaskExperimentRun {
    pub task_id: String,
    pub family: String,
    pub split: String,
    pub fixture_tree_digest: String,
    pub oracle_digest: String,
    pub experiment: ExperimentResult,
}

#[derive(Debug)]
pub struct V2CorpusExperimentRun {
    pub corpus_digest: String,
    pub total_pairs: u32,
    pub total_sides: u32,
    pub task_runs: Vec<V2TaskExperimentRun>,
}

#[derive(Debug, Clone)]
pub struct V2QualificationPersistenceConfig {
    pub experiment: V2RepeatedPairedConfig,
    pub canonical_receipt_log: CanonicalEventLogConfig,
    pub forge_store: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct V2QualificationAggregationV1 {
    pub key: String,
    pub denominator_pairs: u64,
    pub admissible_pairs: u64,
    pub excluded_pairs: u64,
    pub successful_pairs: u64,
    pub score: f64,
    pub uncertainty: V2QualificationUncertaintyV1,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct V2QualificationUncertaintyV1 {
    pub estimate: f64,
    pub lower_bound: f64,
    pub upper_bound: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct V2CorpusQualificationOutcomeV1 {
    pub schema: String,
    pub corpus_digest: String,
    pub experiment_id: String,
    pub assignment_id: String,
    pub assignment_digest: String,
    pub task_receipt_ids: Vec<String>,
    pub task_receipt_digests: Vec<String>,
    pub denominator_pairs: u64,
    pub admissible_pairs: u64,
    pub excluded_pairs: u64,
    pub successful_pairs: u64,
    pub uncertainty: V2QualificationUncertaintyV1,
    pub family_results: Vec<V2QualificationAggregationV1>,
    pub split_results: Vec<V2QualificationAggregationV1>,
    pub forge_bundle_id: String,
    pub forge_bundle_digest: String,
    pub qualification_adjudication_id: String,
    pub qualification_adjudication_digest: String,
    /// Qualification is owner evidence only; it is never candidate evidence.
    pub candidate_evidence: bool,
}

#[derive(Debug, thiserror::Error)]
pub enum CorpusExperimentError {
    #[error(transparent)]
    Corpus(#[from] CorpusError),
    #[error(transparent)]
    Forge(#[from] ForgeError),
    #[error("invalid sealed Forge configuration: {0}")]
    InvalidSealedConfig(String),
    #[error("repeated-paired evaluation requires a container backend")]
    NonContainerBackend,
    #[error("task {task_id} patch violates Forge policy ({violations} violations)")]
    InvalidPatch { task_id: String, violations: usize },
    #[error("total paired denominator {observed} is below required minimum {required}")]
    InsufficientTotalPairs { observed: u32, required: u32 },
    #[error("paired denominator arithmetic overflow")]
    DenominatorOverflow,
    #[error("qualification persistence failed: {0}")]
    Persistence(String),
}

/// Execute the validated v2 corpus and persist qualification receipts in every
/// owner store. Fixture/oracle execution is deliberately represented only as
/// qualification evidence and is never bound as candidate evidence.
pub async fn run_v2_qualification_persisted(
    config: V2QualificationPersistenceConfig,
) -> Result<V2CorpusQualificationOutcomeV1, CorpusExperimentError> {
    let validated =
        crate::learning_corpus::validate_and_consume_v2(&config.experiment.corpus_root)?;
    let run = run_v2_repeated_paired(config.experiment.clone()).await?;
    if run.corpus_digest != validated.corpus_digest || run.task_runs.len() != validated.tasks.len()
    {
        return Err(CorpusExperimentError::Persistence(
            "validated corpus changed during qualification".into(),
        ));
    }

    let event_log = CanonicalEventLog::open(config.canonical_receipt_log)
        .map_err(|error| CorpusExperimentError::Persistence(error.to_string()))?;
    let forge = forge_engine::ForgeStore::open(&config.forge_store)
        .map_err(|error| CorpusExperimentError::Persistence(error.to_string()))?;

    let mut task_receipt_ids = Vec::with_capacity(run.task_runs.len());
    let mut task_receipt_digests = Vec::with_capacity(run.task_runs.len());
    let mut task_records = Vec::with_capacity(run.task_runs.len());
    for task in &run.task_runs {
        let body = task_receipt_body(&run.corpus_digest, task)?;
        let receipt_id = material_id(
            "v2-qualification-task",
            &json!({
                "corpus_digest": run.corpus_digest,
                "task_id": task.task_id,
                "experiment_run_id": task.experiment.run_id,
            }),
        )?;
        let record = append_idempotent(&event_log, &receipt_id, body.clone())?;
        let receipt_digest = record.content_digest.to_string();
        forge
            .insert_experiment_run(
                &task.experiment.run_id,
                &material_id(
                    "v2-qualification-candidate",
                    &json!({
                        "corpus_digest": run.corpus_digest,
                        "task_id": task.task_id,
                    }),
                )?,
                &task.task_id,
                &receipt_id,
                "repeated_paired",
                &serde_json::to_string(&body["baseline_result"])
                    .map_err(|error| CorpusExperimentError::Persistence(error.to_string()))?,
                &serde_json::to_string(&body["patched_result"])
                    .map_err(|error| CorpusExperimentError::Persistence(error.to_string()))?,
                &serde_json::to_string(&body["diff"])
                    .map_err(|error| CorpusExperimentError::Persistence(error.to_string()))?,
                &serde_json::to_string(&body["baseline_descriptor"])
                    .map_err(|error| CorpusExperimentError::Persistence(error.to_string()))?,
                &serde_json::to_string(&body["trials"])
                    .map_err(|error| CorpusExperimentError::Persistence(error.to_string()))?,
            )
            .map_err(|error| CorpusExperimentError::Persistence(error.to_string()))?;
        task_receipt_ids.push(receipt_id);
        task_receipt_digests.push(receipt_digest);
        task_records.push((task, body));
    }

    let denominator_pairs = u64::from(run.total_pairs);
    let mut all = Aggregate::default();
    let mut families = BTreeMap::<String, Aggregate>::new();
    let mut splits = BTreeMap::<String, Aggregate>::new();
    for (task, _) in &task_records {
        let aggregate = aggregate_task(task)?;
        all.add(&aggregate);
        families
            .entry(task.family.clone())
            .or_default()
            .add(&aggregate);
        splits
            .entry(task.split.clone())
            .or_default()
            .add(&aggregate);
    }
    let uncertainty = wilson(all.successful_pairs, all.admissible_pairs);
    let assignment_id = material_id(
        "v2-qualification-assignment",
        &json!({"corpus_digest": run.corpus_digest, "trial_order_seed": config.experiment.trial_order_seed, "pairs_per_task": config.experiment.pairs_per_task}),
    )?;
    let experiment_id = material_id(
        "v2-qualification-experiment",
        &json!({"corpus_digest": run.corpus_digest, "assignment_id": assignment_id, "task_run_ids": run.task_runs.iter().map(|task| task.experiment.run_id.clone()).collect::<Vec<_>>()}),
    )?;
    let family_results = families
        .iter()
        .map(|(key, value)| value.to_public(key.clone()))
        .collect::<Vec<_>>();
    let split_results = splits
        .iter()
        .map(|(key, value)| value.to_public(key.clone()))
        .collect::<Vec<_>>();

    let bundle_id = material_id(
        "v2-qualification-forge-bundle",
        &json!({"experiment_id": experiment_id, "task_receipt_digests": task_receipt_digests, "family_results": family_results, "split_results": split_results}),
    )?;
    let bundle = qualification_bundle(
        &bundle_id,
        &experiment_id,
        &run.corpus_digest,
        &uncertainty,
        all.admissible_pairs,
        &task_receipt_ids,
        &task_receipt_digests,
    )?;
    let bundle_digest = ContentDigest::compute_json(&bundle.to_canonical_evidence_bundle())
        .map_err(|error| CorpusExperimentError::Persistence(error.to_string()))?
        .to_string();
    forge
        .insert_canonical_evidence_bundle(&bundle)
        .map_err(|error| CorpusExperimentError::Persistence(error.to_string()))?;

    let adjudication_id = material_id(
        "v2-qualification-adjudication",
        &json!({"experiment_id": experiment_id, "bundle_id": bundle_id, "assignment_id": assignment_id}),
    )?;
    let mut source_receipt_refs = Vec::with_capacity(task_receipt_ids.len());
    for (receipt_id, digest) in task_receipt_ids.iter().zip(&task_receipt_digests) {
        source_receipt_refs.push(ReceiptRef {
            receipt_id: receipt_id.clone(),
            receipt_digest: IdentityDigest::new(digest.clone())
                .map_err(CorpusExperimentError::Persistence)?,
        });
    }
    let adjudication = adjudicate_candidate(CandidatePromotionInput {
        adjudication_id: adjudication_id.clone(),
        candidate_id: material_id("v2-qualification-candidate", &run.corpus_digest)?,
        candidate_digest: IdentityDigest::of(run.corpus_digest.as_bytes()),
        patch_digest: IdentityDigest::of(task_receipt_digests.join("|").as_bytes()),
        source_tree_digest: IdentityDigest::of(run.corpus_digest.as_bytes()),
        verifier_digest: IdentityDigest::of("v2-qualification-verifier"),
        check_policy_digest: IdentityDigest::of(
            serde_json::to_vec(&config.experiment.forge)
                .map_err(|error| CorpusExperimentError::Persistence(error.to_string()))?,
        ),
        environment_digest: IdentityDigest::of("sealed-local-podman"),
        image_digest: IdentityDigest::of(config.experiment.forge.container.rust_image.as_bytes()),
        experiment_id: experiment_id.clone(),
        evidence_bundle_id: bundle_id.clone(),
        evidence_bundle_digest: IdentityDigest::new(bundle_digest.clone())
            .map_err(CorpusExperimentError::Persistence)?,
        assignment_digest: IdentityDigest::of(assignment_id.as_bytes()),
        paired_denominator: denominator_pairs,
        admissible_pairs: all.admissible_pairs,
        excluded_pairs: all.excluded_pairs,
        uncertainty: UncertaintyV1 {
            estimate: uncertainty.estimate,
            lower_bound: uncertainty.lower_bound,
            upper_bound: uncertainty.upper_bound,
        },
        family_results: family_results
            .iter()
            .map(|result| FamilyGateV1 {
                family: result.key.clone(),
                score: result.score,
                passed: result.excluded_pairs == 0 && result.score >= 1.0,
                admissible_pairs: result.admissible_pairs,
            })
            .collect(),
        holdout_result: {
            let result = split_results
                .iter()
                .find(|result| result.key == "holdout")
                .ok_or_else(|| {
                    CorpusExperimentError::Persistence("holdout aggregation missing".into())
                })?;
            HoldoutGateV1 {
                score: result.score,
                passed: result.excluded_pairs == 0 && result.score >= 1.0,
                admissible_pairs: result.admissible_pairs,
            }
        },
        thresholds: FrozenPromotionThresholdsV1 {
            minimum_admissible_pairs: 1,
            minimum_family_score: 1.0,
            minimum_holdout_score: 1.0,
            maximum_uncertainty: 1.0,
        },
        source_receipt_refs,
        created_at: "v2-qualification".into(),
    })
    .map_err(CorpusExperimentError::Persistence)?;
    forge
        .persist_adjudication(&adjudication)
        .map_err(|error| CorpusExperimentError::Persistence(error.to_string()))?;

    Ok(V2CorpusQualificationOutcomeV1 {
        schema: "V2CorpusQualificationOutcomeV1".into(),
        corpus_digest: run.corpus_digest,
        experiment_id,
        assignment_id: assignment_id.clone(),
        assignment_digest: IdentityDigest::of(assignment_id.as_bytes()).as_str().into(),
        task_receipt_ids,
        task_receipt_digests,
        denominator_pairs,
        admissible_pairs: all.admissible_pairs,
        excluded_pairs: all.excluded_pairs,
        successful_pairs: all.successful_pairs,
        uncertainty,
        family_results,
        split_results,
        forge_bundle_id: bundle_id,
        forge_bundle_digest: bundle_digest,
        qualification_adjudication_id: adjudication.adjudication_id,
        qualification_adjudication_digest: adjudication.adjudication_digest.as_str().into(),
        candidate_evidence: false,
    })
}

#[derive(Default)]
struct Aggregate {
    denominator_pairs: u64,
    admissible_pairs: u64,
    excluded_pairs: u64,
    successful_pairs: u64,
}

impl Aggregate {
    fn add(&mut self, other: &Self) {
        self.denominator_pairs += other.denominator_pairs;
        self.admissible_pairs += other.admissible_pairs;
        self.excluded_pairs += other.excluded_pairs;
        self.successful_pairs += other.successful_pairs;
    }

    fn to_public(&self, key: String) -> V2QualificationAggregationV1 {
        V2QualificationAggregationV1 {
            key,
            denominator_pairs: self.denominator_pairs,
            admissible_pairs: self.admissible_pairs,
            excluded_pairs: self.excluded_pairs,
            successful_pairs: self.successful_pairs,
            score: if self.admissible_pairs == 0 {
                0.0
            } else {
                self.successful_pairs as f64 / self.admissible_pairs as f64
            },
            uncertainty: wilson(self.successful_pairs, self.admissible_pairs),
        }
    }
}

fn aggregate_task(task: &V2TaskExperimentRun) -> Result<Aggregate, CorpusExperimentError> {
    let mut aggregate = Aggregate {
        denominator_pairs: task.experiment.trials.len() as u64 / 2,
        ..Default::default()
    };
    for pair_index in 0..aggregate.denominator_pairs {
        let pair = task
            .experiment
            .trials
            .iter()
            .filter(|trial| u64::from(trial.pair_index) == pair_index)
            .collect::<Vec<_>>();
        if pair.len() != 2 || pair.iter().any(|trial| !trial.timing_admissible) {
            aggregate.excluded_pairs += 1;
            continue;
        }
        aggregate.admissible_pairs += 1;
        let baseline_ok = !task.experiment.baseline_result.test_pass;
        let patched_ok = task.experiment.patched_result.test_pass;
        if baseline_ok && patched_ok {
            aggregate.successful_pairs += 1;
        }
    }
    Ok(aggregate)
}

fn wilson(successes: u64, trials: u64) -> V2QualificationUncertaintyV1 {
    if trials == 0 {
        return V2QualificationUncertaintyV1 {
            estimate: 0.0,
            lower_bound: 0.0,
            upper_bound: 0.0,
        };
    }
    let n = trials as f64;
    let estimate = successes as f64 / n;
    let z = 1.959_963_984_540_054;
    let denominator = 1.0 + z * z / n;
    let centre = (estimate + z * z / (2.0 * n)) / denominator;
    let margin =
        z * ((estimate * (1.0 - estimate) / n + z * z / (4.0 * n * n)).sqrt()) / denominator;
    V2QualificationUncertaintyV1 {
        estimate,
        lower_bound: (centre - margin).max(0.0),
        upper_bound: (centre + margin).min(1.0),
    }
}

fn material_id(prefix: &str, material: &impl Serialize) -> Result<String, CorpusExperimentError> {
    let digest = ContentDigest::compute_json(material)
        .map_err(|error| CorpusExperimentError::Persistence(error.to_string()))?;
    Ok(format!("{prefix}:{}", digest.hex()))
}

fn task_receipt_body(
    corpus_digest: &str,
    task: &V2TaskExperimentRun,
) -> Result<Value, CorpusExperimentError> {
    Ok(json!({
        "schema": "V2QualificationTaskReceiptV1",
        "evidence_scope": "qualification-only; not-candidate-evidence",
        "corpus_digest": corpus_digest,
        "task_id": task.task_id,
        "family": task.family,
        "split": task.split,
        "fixture_tree_digest": task.fixture_tree_digest,
        "experiment_run_id": task.experiment.run_id,
        "mode": task.experiment.mode,
        "baseline_descriptor": task.experiment.baseline_descriptor,
        "baseline_result": check_result_summary(&task.experiment.baseline_result),
        "patched_result": check_result_summary(&task.experiment.patched_result),
        "diff": task.experiment.diff,
        "trials": task.experiment.trials,
        "completed_at": "",
    }))
}

fn check_result_summary(result: &forge_engine::CheckResult) -> Value {
    json!({
        "fmt_pass": result.fmt_pass,
        "clippy_pass": result.clippy_pass,
        "test_pass": result.test_pass,
        "total_duration_ms": result.total_duration_ms,
    })
}

fn append_idempotent(
    log: &CanonicalEventLog,
    receipt_id: &str,
    body: Value,
) -> Result<aidens_receipts::CanonicalEventLogEntry, CorpusExperimentError> {
    match log.append_json(
        "aidens-runner",
        "v2-qualification-task-receipt-v1",
        receipt_id,
        body.clone(),
    ) {
        Ok(record) => Ok(record),
        Err(aidens_receipts::CanonicalEventLogError::DuplicateReceiptId(_)) => {
            let record = log
                .inspect(receipt_id)
                .map_err(|error| CorpusExperimentError::Persistence(error.to_string()))?;
            if record.body == body && record.verify_digest() && record.verify_record_digest() {
                Ok(record)
            } else {
                Err(CorpusExperimentError::Persistence(format!(
                    "qualification receipt conflict: {receipt_id}"
                )))
            }
        }
        Err(error) => Err(CorpusExperimentError::Persistence(error.to_string())),
    }
}

fn qualification_bundle(
    bundle_id: &str,
    experiment_id: &str,
    corpus_digest: &str,
    uncertainty: &V2QualificationUncertaintyV1,
    admissible_pairs: u64,
    receipt_ids: &[String],
    receipt_digests: &[String],
) -> Result<ExperimentEvidenceBundle, CorpusExperimentError> {
    serde_json::from_value(json!({
        "bundle_id": bundle_id,
        "candidate_id": format!("v2-qualification:{corpus_digest}"),
        "eval_id": experiment_id,
        "version_id": "v2-qualification-policy-v1",
        "scores": ScoreVector { correctness: uncertainty.estimate, novelty: 0.0, stability: 1.0 - (uncertainty.upper_bound - uncertainty.lower_bound), weighted_total: uncertainty.estimate, cea_confidence: None, cea_predicted_correctness: None },
        "hypotheses": [], "verification": null, "trace_id": experiment_id,
        "experiment_diff": null, "attribution_json": null, "assessment": null,
        "warnings": ["qualification-only; fixture/oracle execution is not candidate evidence"],
        "created_at": "v2-qualification",
        "outcome": format!("admissible_pairs={admissible_pairs}; estimate={}", uncertainty.estimate),
        "receipts": receipt_ids.iter().zip(receipt_digests).map(|(id, digest)| json!({"receipt_id": id, "kind": "check_result", "storage": {"store_row": {"table": "canonical_event_log", "key": id}}, "content_hash": digest})).collect::<Vec<_>>(),
        "sealed": true,
    })).map_err(|error| CorpusExperimentError::Persistence(format!("build Forge bundle: {error}")))
}

pub async fn run_v2_repeated_paired(
    config: V2RepeatedPairedConfig,
) -> Result<V2CorpusExperimentRun, CorpusExperimentError> {
    validate_sealed_config(&config.forge)?;
    let backend = select_backend(&config.forge)?;
    run_v2_repeated_paired_with_backend(config, backend.as_ref()).await
}

async fn run_v2_repeated_paired_with_backend(
    config: V2RepeatedPairedConfig,
    backend: &dyn ExecutionBackend,
) -> Result<V2CorpusExperimentRun, CorpusExperimentError> {
    validate_sealed_config(&config.forge)?;
    if backend.kind() != ExecutionBackendKind::Container {
        return Err(CorpusExperimentError::NonContainerBackend);
    }

    let (corpus_digest, tasks) = validate_and_load_v2_evaluator_tasks(&config.corpus_root)?;
    let task_count =
        u32::try_from(tasks.len()).map_err(|_| CorpusExperimentError::DenominatorOverflow)?;
    let total_pairs = task_count
        .checked_mul(config.pairs_per_task)
        .ok_or(CorpusExperimentError::DenominatorOverflow)?;
    if total_pairs < MIN_TOTAL_PAIRS {
        return Err(CorpusExperimentError::InsufficientTotalPairs {
            observed: total_pairs,
            required: MIN_TOTAL_PAIRS,
        });
    }
    let total_sides = total_pairs
        .checked_mul(2)
        .ok_or(CorpusExperimentError::DenominatorOverflow)?;

    for task in &tasks {
        if !CargoAdapter::detect(&task.learner.fixture_path) {
            return Err(CorpusExperimentError::InvalidSealedConfig(format!(
                "task fixture is not a Cargo project: {}",
                task.learner.task_id
            )));
        }
        let validation = forge_engine::validate_patch(&task.patch, &config.forge);
        if !validation.ok {
            return Err(CorpusExperimentError::InvalidPatch {
                task_id: task.learner.task_id.clone(),
                violations: validation.violations.len(),
            });
        }
    }

    let adapter = CargoAdapter;
    let mut task_runs = Vec::with_capacity(tasks.len());
    for (task_index, task) in tasks.into_iter().enumerate() {
        task_runs.push(
            run_task(
                task,
                task_index,
                config.pairs_per_task,
                config.trial_order_seed,
                backend,
                &adapter,
                &config.forge,
            )
            .await?,
        );
    }

    Ok(V2CorpusExperimentRun {
        corpus_digest,
        total_pairs,
        total_sides,
        task_runs,
    })
}

async fn run_task(
    task: V2EvaluatorTask,
    task_index: usize,
    pairs_per_task: u32,
    base_seed: u64,
    backend: &dyn ExecutionBackend,
    adapter: &CargoAdapter,
    forge: &ForgeConfig,
) -> Result<V2TaskExperimentRun, CorpusExperimentError> {
    let experiment_config = ExperimentConfig {
        mode: ExperimentMode::RepeatedPaired,
        trial_count: pairs_per_task,
        statistics_policy: StatisticsPolicy {
            min_paired_trials: pairs_per_task,
            require_timing_admissibility: false,
        },
        trial_order_seed: base_seed.wrapping_add(task_index as u64),
        ..ExperimentConfig::default()
    };
    let experiment = PairedExperimentRunner::new(backend, adapter, forge)
        .run(&task.learner.fixture_path, &task.patch, &experiment_config)
        .await?;

    Ok(V2TaskExperimentRun {
        task_id: task.learner.task_id,
        family: task.learner.family,
        split: task.learner.split,
        fixture_tree_digest: task.learner.fixture_tree_digest,
        oracle_digest: task.learner.oracle_digest,
        experiment,
    })
}

fn validate_sealed_config(config: &ForgeConfig) -> Result<(), CorpusExperimentError> {
    if config.mode != "sealed_local"
        || config.execution_backend_preference != "container"
        || config.container_runtime_preference != "podman"
        || config.sealed_allow_host_backend
    {
        return Err(CorpusExperimentError::InvalidSealedConfig(
            "requires mode=sealed_local, backend=container, runtime=podman, and no host fallback"
                .into(),
        ));
    }
    let Some((name, digest)) = config.container.rust_image.rsplit_once("@sha256:") else {
        return Err(CorpusExperimentError::InvalidSealedConfig(
            "container image must be digest-pinned".into(),
        ));
    };
    if name.trim().is_empty()
        || digest.len() != 64
        || !digest.bytes().all(|byte| byte.is_ascii_hexdigit())
    {
        return Err(CorpusExperimentError::InvalidSealedConfig(
            "container image must contain a 64-character SHA-256 digest".into(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use forge_engine::{CommandOutput, CommandTimings, ForgeResult, LogBundle, Workspace};
    use std::collections::BTreeSet;
    use std::path::Path;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[derive(Default)]
    struct RecordingBackend {
        prepared: AtomicUsize,
    }

    #[async_trait]
    impl ExecutionBackend for RecordingBackend {
        fn kind(&self) -> ExecutionBackendKind {
            ExecutionBackendKind::Container
        }

        async fn prepare_workspace(&self, fixture: &Path) -> ForgeResult<Workspace> {
            self.prepared.fetch_add(1, Ordering::SeqCst);
            Ok(sandbox_workspace::prepare_workspace(fixture)?)
        }

        async fn run_command(
            &self,
            _workspace: &Path,
            _program: &str,
            _args: &[&str],
            _env: &[(&str, &str)],
            _timeout_secs: u64,
        ) -> ForgeResult<CommandOutput> {
            Ok(CommandOutput {
                stdout: String::new(),
                stderr: String::new(),
                exit_code: 0,
                duration_ms: 1,
            })
        }

        async fn collect_logs(
            &self,
            _fmt: &CommandOutput,
            _clippy: &CommandOutput,
            _test: &CommandOutput,
        ) -> ForgeResult<LogBundle> {
            Ok(LogBundle {
                fmt_stdout: String::new(),
                fmt_stderr: String::new(),
                clippy_stdout: String::new(),
                clippy_stderr: String::new(),
                test_stdout: String::new(),
                test_stderr: String::new(),
                timings: CommandTimings {
                    fmt_ms: 0,
                    clippy_ms: 0,
                    test_ms: 0,
                },
            })
        }
    }

    fn config(pairs_per_task: u32) -> V2RepeatedPairedConfig {
        V2RepeatedPairedConfig {
            corpus_root: Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../fixtures/learning-coding-agent/v2"),
            pairs_per_task,
            trial_order_seed: 7,
            forge: ForgeConfig {
                mode: "sealed_local".into(),
                execution_backend_preference: "container".into(),
                container_runtime_preference: "podman".into(),
                sealed_allow_host_backend: false,
                container: forge_engine::config::ContainerConfig {
                    rust_image: format!("localhost/aidens-rust-checks@sha256:{}", "0".repeat(64)),
                    ..Default::default()
                },
                ..Default::default()
            },
        }
    }

    #[tokio::test]
    async fn runs_all_frozen_tasks_through_canonical_repeated_owner() {
        let backend = RecordingBackend::default();
        let result = run_v2_repeated_paired_with_backend(config(4), &backend)
            .await
            .unwrap();

        assert_eq!(result.task_runs.len(), 15);
        assert_eq!(result.total_pairs, 60);
        assert_eq!(result.total_sides, 120);
        assert_eq!(backend.prepared.load(Ordering::SeqCst), 120);
        assert_eq!(
            result
                .task_runs
                .iter()
                .map(|run| run.family.as_str())
                .collect::<BTreeSet<_>>()
                .len(),
            5
        );
        assert!(result.task_runs.iter().all(|run| {
            run.experiment.trials.len() == 8
                && (0..4).all(|pair_index| {
                    run.experiment
                        .trials
                        .iter()
                        .filter(|trial| trial.pair_index == pair_index)
                        .count()
                        == 2
                })
        }));
    }

    #[tokio::test]
    async fn rejects_underpowered_denominator_before_backend_effects() {
        let backend = RecordingBackend::default();
        let error = run_v2_repeated_paired_with_backend(config(1), &backend)
            .await
            .unwrap_err();

        assert!(matches!(
            error,
            CorpusExperimentError::InsufficientTotalPairs {
                observed: 15,
                required: 20
            }
        ));
        assert_eq!(backend.prepared.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn rejects_patch_policy_violations_before_backend_effects() {
        let backend = RecordingBackend::default();
        let mut config = config(4);
        config.forge.caps.max_files_changed = 0;

        let error = run_v2_repeated_paired_with_backend(config, &backend)
            .await
            .unwrap_err();

        assert!(matches!(error, CorpusExperimentError::InvalidPatch { .. }));
        assert_eq!(backend.prepared.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn rejects_digest_without_image_name_before_backend_effects() {
        let backend = RecordingBackend::default();
        let mut config = config(4);
        config.forge.container.rust_image = format!("@sha256:{}", "0".repeat(64));

        let error = run_v2_repeated_paired_with_backend(config, &backend)
            .await
            .unwrap_err();

        assert!(matches!(
            error,
            CorpusExperimentError::InvalidSealedConfig(_)
        ));
        assert_eq!(backend.prepared.load(Ordering::SeqCst), 0);
    }
}
