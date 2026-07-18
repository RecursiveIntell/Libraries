//! Thin AiDENs composition over Forge's canonical repeated-paired experiment owner.
//!
//! This module does not compute statistics, mint promotion evidence, or persist a
//! second experiment truth. It exposes the owner-native `ExperimentResult` values
//! for later owner-side adjudication.

use crate::learning_corpus::{validate_and_load_v2_evaluator_tasks, CorpusError, V2EvaluatorTask};
use forge_engine::{
    select_backend, CargoAdapter, ExecutionBackend, ExecutionBackendKind, ExperimentConfig,
    ExperimentMode, ExperimentResult, ForgeConfig, ForgeError, PairedExperimentRunner,
    ProjectAdapter, StatisticsPolicy,
};
use std::path::PathBuf;

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
