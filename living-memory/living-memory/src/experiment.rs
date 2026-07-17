//! Experiment execution: baseline/patched paired trials with typed diffs.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::baseline::{BaselineDescriptor, ComparabilityPolicy, WorkspacePolicy};
use crate::error::{ForgeError, ForgeResult};
use crate::exec::backend::{CheckResult, ExecutionBackendKind};

// ── Experiment mode ──

/// How an experiment is structured.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExperimentMode {
    /// Single baseline + single patched execution.
    Paired,
    /// Multiple paired trials for statistical robustness.
    RepeatedPaired,
    /// Follow-up experiment targeting specific verification steps.
    VerificationFollowup,
}

// ── Typed effect kinds ──

/// Classification of an observed effect from experiment comparison.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EffectKind {
    CompileFailure,
    TestFailure,
    LintFailure,
    Timeout,
    PanicCrash,
    OutputMismatch,
    WarningRegression,
    WarningImprovement,
    PerformanceRegression,
    PerformanceImprovement,
    FlakySignal,
    InformationalOnly,
}

/// An effect located in the experiment diff.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypedLocatedEffect {
    pub kind: EffectKind,
    pub file: Option<PathBuf>,
    pub line: Option<u32>,
    pub message: String,
    /// Whether this effect appeared in the baseline.
    pub in_baseline: bool,
    /// Whether this effect appeared in the patched run.
    pub in_patched: bool,
}

// ── Trial record ──

/// Record of a single trial execution (baseline or patched).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrialRecord {
    /// Which side of the experiment.
    pub side: TrialSide,
    /// Summarized check pass/fail flags.
    pub fmt_pass: bool,
    pub clippy_pass: bool,
    pub test_pass: bool,
    /// Backend used.
    pub backend_kind: ExecutionBackendKind,
    /// Duration in milliseconds.
    pub duration_ms: u64,
    /// Seed used for reproducibility.
    pub seed: u64,
    /// Whether caches were warm.
    pub cache_mode: CacheMode,
    /// Whether network was available.
    pub network_available: bool,
    /// Whether timing data is admissible.
    pub timing_admissible: bool,
    /// Zero-based pair number; defaults preserve legacy records.
    #[serde(default)]
    pub pair_index: u32,
    /// Position within the pair (0 or 1); defaults preserve legacy records.
    #[serde(default)]
    pub order_index: u32,
}

/// Which side of a paired experiment a trial belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TrialSide {
    Baseline,
    Patched,
}

/// Whether build caches were warm or cold during a trial execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CacheMode {
    Cold,
    Warm,
    Unknown,
}

// ── Experiment diff ──

/// Typed diff between baseline and patched experiment results.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperimentDiff {
    /// Effects that differ between baseline and patched.
    pub effects: Vec<TypedLocatedEffect>,
    /// Summary counts.
    pub regressions: u32,
    pub improvements: u32,
    pub stable_failures: u32,
    pub stable_passes: u32,
    /// Whether the diff is statistically meaningful.
    pub statistically_meaningful: bool,
    /// Warning if sample size is insufficient.
    pub sample_warning: Option<String>,
}

impl ExperimentDiff {
    /// Derive a typed diff from baseline and patched check results.
    pub fn from_paired(baseline: &CheckResult, patched: &CheckResult) -> Self {
        let mut effects = Vec::new();
        let mut regressions = 0u32;
        let mut improvements = 0u32;
        let mut stable_failures = 0u32;
        let mut stable_passes = 0u32;

        // Compare fmt
        match (baseline.fmt_pass, patched.fmt_pass) {
            (true, false) => {
                regressions += 1;
                for eff in &patched.fmt_output.effects {
                    effects.push(TypedLocatedEffect {
                        kind: EffectKind::LintFailure,
                        file: eff.file.clone(),
                        line: eff.line,
                        message: eff.message.clone(),
                        in_baseline: false,
                        in_patched: true,
                    });
                }
            }
            (false, true) => {
                improvements += 1;
            }
            (false, false) => {
                stable_failures += 1;
            }
            (true, true) => {
                stable_passes += 1;
            }
        }

        // Compare clippy
        match (baseline.clippy_pass, patched.clippy_pass) {
            (true, false) => {
                regressions += 1;
                for eff in &patched.clippy_output.effects {
                    effects.push(TypedLocatedEffect {
                        kind: EffectKind::LintFailure,
                        file: eff.file.clone(),
                        line: eff.line,
                        message: eff.message.clone(),
                        in_baseline: false,
                        in_patched: true,
                    });
                }
            }
            (false, true) => {
                improvements += 1;
                for eff in &baseline.clippy_output.effects {
                    effects.push(TypedLocatedEffect {
                        kind: EffectKind::WarningImprovement,
                        file: eff.file.clone(),
                        line: eff.line,
                        message: format!("fixed: {}", eff.message),
                        in_baseline: true,
                        in_patched: false,
                    });
                }
            }
            (false, false) => {
                stable_failures += 1;
                // Classify effects that are stable across both
                diff_effects(
                    &baseline.clippy_output.effects,
                    &patched.clippy_output.effects,
                    &mut effects,
                    &mut regressions,
                    &mut improvements,
                );
            }
            (true, true) => {
                stable_passes += 1;
            }
        }

        // Compare tests
        match (baseline.test_pass, patched.test_pass) {
            (true, false) => {
                regressions += 1;
                for eff in &patched.test_output.effects {
                    effects.push(TypedLocatedEffect {
                        kind: EffectKind::TestFailure,
                        file: eff.file.clone(),
                        line: eff.line,
                        message: eff.message.clone(),
                        in_baseline: false,
                        in_patched: true,
                    });
                }
            }
            (false, true) => {
                improvements += 1;
            }
            (false, false) => {
                stable_failures += 1;
                diff_effects(
                    &baseline.test_output.effects,
                    &patched.test_output.effects,
                    &mut effects,
                    &mut regressions,
                    &mut improvements,
                );
            }
            (true, true) => {
                stable_passes += 1;
            }
        }

        ExperimentDiff {
            effects,
            regressions,
            improvements,
            stable_failures,
            stable_passes,
            // Single-pair is NOT statistically meaningful. It is a provisional
            // local observation only.
            statistically_meaningful: false,
            sample_warning: Some(
                "single paired trial: provisional local attribution only, not statistically meaningful".to_string(),
            ),
        }
    }
}

/// Diff individual effects between baseline and patched, classifying new/removed.
fn diff_effects(
    baseline_effects: &[crate::exec::backend::LocatedEffect],
    patched_effects: &[crate::exec::backend::LocatedEffect],
    out: &mut Vec<TypedLocatedEffect>,
    regressions: &mut u32,
    improvements: &mut u32,
) {
    let baseline_classes: std::collections::BTreeSet<_> = baseline_effects
        .iter()
        .map(|e| &e.sig.message_class)
        .collect();
    let patched_classes: std::collections::BTreeSet<_> = patched_effects
        .iter()
        .map(|e| &e.sig.message_class)
        .collect();

    // New in patched (regressions)
    for eff in patched_effects {
        if !baseline_classes.contains(&eff.sig.message_class) {
            *regressions += 1;
            out.push(TypedLocatedEffect {
                kind: EffectKind::WarningRegression,
                file: eff.file.clone(),
                line: eff.line,
                message: eff.message.clone(),
                in_baseline: false,
                in_patched: true,
            });
        }
    }

    // Gone from patched (improvements)
    for eff in baseline_effects {
        if !patched_classes.contains(&eff.sig.message_class) {
            *improvements += 1;
            out.push(TypedLocatedEffect {
                kind: EffectKind::WarningImprovement,
                file: eff.file.clone(),
                line: eff.line,
                message: format!("fixed: {}", eff.message),
                in_baseline: true,
                in_patched: false,
            });
        }
    }
}

/// Digest fixture contents and relative names, excluding volatile metadata.
fn fixture_tree_digest(root: &Path) -> ForgeResult<String> {
    fn walk(root: &Path, dir: &Path, out: &mut Vec<(String, Vec<u8>)>) -> std::io::Result<()> {
        for entry in std::fs::read_dir(dir)? {
            let path = entry?.path();
            if path.is_dir() {
                walk(root, &path, out)?;
            } else if path.is_file() {
                out.push((
                    path.strip_prefix(root)
                        .unwrap_or(&path)
                        .to_string_lossy()
                        .into_owned(),
                    std::fs::read(&path)?,
                ));
            }
        }
        Ok(())
    }
    let mut files = Vec::new();
    walk(root, root, &mut files)?;
    files.sort_by(|a, b| a.0.cmp(&b.0));
    let mut material = Vec::new();
    for (name, bytes) in files {
        material.extend_from_slice(name.as_bytes());
        material.push(0);
        material.extend_from_slice(&bytes);
        material.push(0xff);
    }
    Ok(blake3::hash(&material).to_hex().to_string())
}

// ── Statistics policy ──

/// Policy controlling statistical validity claims.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatisticsPolicy {
    /// Minimum paired trials for scalar stability claims.
    #[serde(default = "default_min_paired")]
    pub min_paired_trials: u32,
    /// Whether timing claims require timing_admissible = true.
    #[serde(default = "default_true")]
    pub require_timing_admissibility: bool,
}

fn default_min_paired() -> u32 {
    3
}
fn default_true() -> bool {
    true
}

impl Default for StatisticsPolicy {
    fn default() -> Self {
        Self {
            min_paired_trials: default_min_paired(),
            require_timing_admissibility: true,
        }
    }
}

// ── Run identity (split record model) ──

/// Identity of an experiment run (who/what/when).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunIdentity {
    pub run_id: String,
    pub candidate_id: String,
    pub task_id: String,
    pub trace_id: String,
    pub started_at: String,
}

/// Record of execution details (how it ran).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionRecord {
    pub run_id: String,
    pub mode: ExperimentMode,
    pub baseline: BaselineDescriptor,
    pub trials: Vec<TrialRecord>,
    pub backend_kind: ExecutionBackendKind,
    pub workspace_path: PathBuf,
    pub completed_at: String,
}

/// Record of analysis derived from execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisRecord {
    pub run_id: String,
    pub diff: ExperimentDiff,
    pub scores_json: String,
    pub attribution_json: Option<String>,
}

/// Record of evidence produced.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceRecord {
    pub run_id: String,
    pub bundle_id: String,
    pub hypothesis_ids: Vec<String>,
    pub verification_plan_id: Option<String>,
}

/// Record of export to semantic-memory.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperimentExportRecord {
    pub export_key: String,
    pub bundle_id: String,
    pub rendering_version: u32,
    pub namespace: String,
    pub exported_at: String,
    /// Whether the compatibility-only direct import escape hatch succeeded.
    pub write_through_ok: Option<bool>,
}

// ── Experiment execution ──

/// Configuration for a single experiment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperimentConfig {
    pub mode: ExperimentMode,
    pub trial_count: u32,
    pub statistics_policy: StatisticsPolicy,
    pub comparability: ComparabilityPolicy,
    pub workspace_policy: WorkspacePolicy,
    /// Stable seed controlling deterministic side order in repeated runs.
    pub trial_order_seed: u64,
}

impl Default for ExperimentConfig {
    fn default() -> Self {
        Self {
            mode: ExperimentMode::Paired,
            trial_count: 1,
            statistics_policy: StatisticsPolicy::default(),
            comparability: ComparabilityPolicy::default(),
            workspace_policy: WorkspacePolicy::default(),
            trial_order_seed: 0,
        }
    }
}

/// Result of running an experiment.
///
/// Note: CheckResult is not directly serializable (comes from check-runner primitive),
/// so we serialize the diff + trial summaries instead. The full CheckResults are
/// available in-memory for downstream processing.
#[derive(Debug, Clone)]
pub struct ExperimentResult {
    pub run_id: String,
    pub mode: ExperimentMode,
    pub baseline_descriptor: BaselineDescriptor,
    pub baseline_result: CheckResult,
    pub patched_result: CheckResult,
    pub diff: ExperimentDiff,
    pub trials: Vec<TrialRecord>,
    pub completed_at: String,
}

/// Real experiment runner that executes baseline and patched checks.
pub struct PairedExperimentRunner<'a> {
    backend: &'a dyn crate::exec::backend::ExecutionBackend,
    adapter: &'a dyn crate::adapters::ProjectAdapter,
    config: &'a crate::config::ForgeConfig,
}

impl<'a> PairedExperimentRunner<'a> {
    /// Create a new runner bound to the given backend, project adapter, and config.
    pub fn new(
        backend: &'a dyn crate::exec::backend::ExecutionBackend,
        adapter: &'a dyn crate::adapters::ProjectAdapter,
        config: &'a crate::config::ForgeConfig,
    ) -> Self {
        Self {
            backend,
            adapter,
            config,
        }
    }

    /// Run a paired experiment: baseline then patched.
    ///
    /// 1. Prepare workspace from fixture
    /// 2. Run baseline checks
    /// 3. Apply patch
    /// 4. Run patched checks
    /// 5. Compute typed diff
    pub async fn run(
        &self,
        fixture_path: &Path,
        patch: &crate::runtime::patch::types::StructuredPatch,
        experiment_config: &ExperimentConfig,
    ) -> ForgeResult<ExperimentResult> {
        if experiment_config.mode == ExperimentMode::VerificationFollowup {
            return Err(ForgeError::Config(
                "verification_followup is unsupported by the paired runner".into(),
            ));
        }
        let pair_count = match experiment_config.mode {
            ExperimentMode::RepeatedPaired => experiment_config.trial_count,
            ExperimentMode::Paired | ExperimentMode::VerificationFollowup => 1,
        };
        if pair_count == 0 {
            return Err(ForgeError::Config("trial_count must be nonzero".into()));
        }
        if experiment_config.mode == ExperimentMode::RepeatedPaired
            && pair_count < experiment_config.statistics_policy.min_paired_trials
        {
            return Err(ForgeError::Config(format!(
                "trial_count {pair_count} is below minimum paired trials {}",
                experiment_config.statistics_policy.min_paired_trials
            )));
        }
        let patch_bytes = serde_json::to_vec(patch)?;
        let fixture_digest = fixture_tree_digest(fixture_path)?;
        let config_bytes = serde_json::to_vec(experiment_config)?;
        let mut id_material = fixture_digest.into_bytes();
        id_material.extend_from_slice(&patch_bytes);
        id_material.extend_from_slice(&config_bytes);
        id_material.extend_from_slice(
            format!(
                "backend={:?};timeout={}",
                self.backend.kind(),
                self.config.container.command_timeout_secs
            )
            .as_bytes(),
        );
        let run_id = blake3::hash(&id_material).to_hex().to_string();
        let _started_at = chrono::Utc::now().to_rfc3339();

        tracing::info!(run_id = %run_id, mode = ?experiment_config.mode, "starting experiment");

        let timeout = self.config.container.command_timeout_secs;
        let mut all_trials = Vec::new();
        let mut baseline_result = None;
        let mut patched_result = None;
        let mut baseline_descriptor = None;
        for pair_index in 0..pair_count {
            let first_patched = ((experiment_config
                .trial_order_seed
                .wrapping_add(pair_index as u64 * 0x9e3779b97f4a7c15))
                & 1)
                == 1;
            for order_index in 0..2 {
                let side = if (order_index == 0) == first_patched {
                    TrialSide::Patched
                } else {
                    TrialSide::Baseline
                };
                let workspace = self.backend.prepare_workspace(fixture_path).await?;
                let ws_path = &workspace.host_path;
                if baseline_descriptor.is_none() {
                    baseline_descriptor =
                        Some(crate::baseline::capture_baseline_provenance(ws_path).await?);
                }
                if side == TrialSide::Patched {
                    crate::runtime::patch::apply::apply_patch(patch, ws_path)?;
                }
                let result = self
                    .run_checks(
                        ws_path,
                        timeout,
                        side,
                        pair_index,
                        order_index,
                        &mut all_trials,
                        experiment_config.trial_order_seed,
                    )
                    .await?;
                match side {
                    TrialSide::Baseline => baseline_result = Some(result),
                    TrialSide::Patched => patched_result = Some(result),
                }
            }
        }
        let baseline_result = baseline_result.ok_or_else(|| {
            ForgeError::ExperimentFailed("paired run produced no baseline result".into())
        })?;
        let patched_result = patched_result.ok_or_else(|| {
            ForgeError::ExperimentFailed("paired run produced no patched result".into())
        })?;
        let baseline_descriptor = baseline_descriptor.ok_or_else(|| {
            ForgeError::ExperimentFailed("paired run produced no baseline provenance".into())
        })?;

        // Compute typed diff
        let diff = ExperimentDiff::from_paired(&baseline_result, &patched_result);

        let completed_at = chrono::Utc::now().to_rfc3339();
        tracing::info!(
            run_id = %run_id,
            regressions = diff.regressions,
            improvements = diff.improvements,
            "experiment completed"
        );

        Ok(ExperimentResult {
            run_id,
            mode: experiment_config.mode,
            baseline_descriptor,
            baseline_result,
            patched_result,
            diff,
            trials: all_trials,
            completed_at,
        })
    }

    /// Run all check commands and aggregate into a CheckResult.
    async fn run_checks(
        &self,
        workspace: &Path,
        timeout: u64,
        side: TrialSide,
        pair_index: u32,
        order_index: u32,
        trials: &mut Vec<TrialRecord>,
        trial_order_seed: u64,
    ) -> ForgeResult<CheckResult> {
        let commands = self.adapter.check_commands(self.config);
        let start = std::time::Instant::now();

        let mut fmt_output = crate::exec::backend::ParsedCheckOutput::default();
        let mut clippy_output = crate::exec::backend::ParsedCheckOutput::default();
        let mut test_output = crate::exec::backend::ParsedCheckOutput::default();

        for cmd in &commands {
            let args: Vec<&str> = cmd.args.iter().map(|s| s.as_str()).collect();
            let env: Vec<(&str, &str)> = cmd
                .env
                .iter()
                .map(|(k, v)| (k.as_str(), v.as_str()))
                .collect();

            let output = self
                .backend
                .run_command(workspace, &cmd.program, &args, &env, timeout)
                .await;

            let kind = cmd.kind.clone();
            match output {
                Ok(output) => {
                    let parsed = self.adapter.parse_check_output(
                        cmd,
                        &output.stdout,
                        &output.stderr,
                        output.exit_code,
                    );
                    match kind {
                        crate::exec::backend::CheckKind::Fmt => fmt_output = parsed,
                        crate::exec::backend::CheckKind::Clippy => clippy_output = parsed,
                        crate::exec::backend::CheckKind::Test => test_output = parsed,
                    }
                }
                Err(ForgeError::CommandTimeout { .. }) => {
                    // Record timeout as failure
                    let parsed = crate::exec::backend::ParsedCheckOutput {
                        check_kind: kind.clone(),
                        exit_code: -1,
                        effects: vec![],
                        raw_stdout: String::new(),
                        raw_stderr: "timeout".to_string(),
                    };
                    match kind {
                        crate::exec::backend::CheckKind::Fmt => fmt_output = parsed,
                        crate::exec::backend::CheckKind::Clippy => clippy_output = parsed,
                        crate::exec::backend::CheckKind::Test => test_output = parsed,
                    }
                }
                Err(e) => return Err(e),
            }
        }

        let duration_ms = start.elapsed().as_millis() as u64;

        let check_result = CheckResult {
            fmt_pass: fmt_output.exit_code == 0,
            clippy_pass: clippy_output.exit_code == 0,
            test_pass: test_output.exit_code == 0,
            fmt_output,
            clippy_output,
            test_output,
            total_duration_ms: duration_ms,
        };

        trials.push(TrialRecord {
            side,
            fmt_pass: check_result.fmt_pass,
            clippy_pass: check_result.clippy_pass,
            test_pass: check_result.test_pass,
            backend_kind: self.backend.kind(),
            duration_ms,
            seed: trial_order_seed,
            cache_mode: CacheMode::Unknown,
            network_available: true,
            timing_admissible: false,
            pair_index,
            order_index,
        });

        Ok(check_result)
    }
}
