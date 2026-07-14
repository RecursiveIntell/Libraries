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
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
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
    ///
    /// Retained for deserializing legacy records. New code must use
    /// `network_mode`; a `false` legacy value means only "not known available".
    #[serde(default)]
    pub network_available: bool,
    /// Explicit network observation. Unknown is deliberately not treated as
    /// unavailable, since that would manufacture a comparability claim.
    #[serde(default)]
    pub network_mode: NetworkMode,
    /// Whether timing data is admissible.
    pub timing_admissible: bool,
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

/// The observed availability of network access for a trial.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NetworkMode {
    Available,
    Unavailable,
    #[default]
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
    let baseline_identities: std::collections::BTreeSet<_> =
        baseline_effects.iter().map(effect_identity).collect();
    let patched_identities: std::collections::BTreeSet<_> =
        patched_effects.iter().map(effect_identity).collect();

    // New in patched (regressions)
    for eff in patched_effects {
        if !baseline_identities.contains(&effect_identity(eff)) {
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
        if !patched_identities.contains(&effect_identity(eff)) {
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

/// A checker effect identity intentionally includes location.  Message class
/// alone is an observational grouping, not enough to call an effect stable.
fn effect_identity(
    effect: &crate::exec::backend::LocatedEffect,
) -> (
    String,
    String,
    String,
    String,
    Option<std::path::PathBuf>,
    Option<u32>,
) {
    (
        effect.sig.check_kind.clone(),
        effect.sig.outcome.clone(),
        effect.sig.severity.clone(),
        effect.sig.message_class.clone(),
        effect.file.clone(),
        effect.line,
    )
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
#[derive(Debug, Clone)]
pub struct ExperimentConfig {
    pub mode: ExperimentMode,
    pub trial_count: u32,
    pub statistics_policy: StatisticsPolicy,
    pub comparability: ComparabilityPolicy,
    pub workspace_policy: WorkspacePolicy,
}

impl Default for ExperimentConfig {
    fn default() -> Self {
        Self {
            mode: ExperimentMode::Paired,
            trial_count: 1,
            statistics_policy: StatisticsPolicy::default(),
            comparability: ComparabilityPolicy::default(),
            workspace_policy: WorkspacePolicy::default(),
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
    /// Each independently prepared matched pair.  The legacy top-level fields
    /// remain the first pair for compatibility.
    pub pairs: Vec<PairedTrialResult>,
}

/// Complete record for one fresh baseline/patched pair.
#[derive(Debug, Clone)]
pub struct PairedTrialResult {
    pub pair_index: u32,
    pub baseline_descriptor: BaselineDescriptor,
    pub patched_descriptor: BaselineDescriptor,
    pub baseline_result: CheckResult,
    pub patched_result: CheckResult,
    pub line_map: crate::runtime::patch::apply::LineAttributionMap,
    pub diff: ExperimentDiff,
    pub comparable: bool,
    pub comparability_reasons: Vec<String>,
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
        let run_id = uuid::Uuid::new_v4().to_string();
        let _started_at = chrono::Utc::now().to_rfc3339();

        tracing::info!(run_id = %run_id, mode = ?experiment_config.mode, "starting experiment");

        let pair_count = match experiment_config.mode {
            ExperimentMode::RepeatedPaired => experiment_config.trial_count,
            _ => 1,
        };
        if pair_count == 0 {
            return Err(ForgeError::Config(
                "trial_count must be at least one".to_string(),
            ));
        }
        let mut all_trials = Vec::new();
        let mut pairs = Vec::with_capacity(pair_count as usize);
        for pair_index in 0..pair_count {
            pairs.push(
                self.run_pair(fixture_path, Some(patch), pair_index, &mut all_trials)
                    .await?,
            );
        }
        // All control arms must share provenance.  A fresh workspace is not
        // sufficient if its captured baseline has drifted between trials.
        if let Some(reference) = pairs
            .first()
            .map(|pair| pair.baseline_descriptor.fingerprint())
        {
            for pair in &mut pairs {
                if pair.baseline_descriptor.fingerprint() != reference {
                    pair.comparable = false;
                    pair.comparability_reasons
                        .push("baseline fingerprint drift across repeated pairs".to_string());
                }
            }
        }
        let first = pairs.first().cloned().ok_or_else(|| {
            ForgeError::Config("experiment produced no paired trials".to_string())
        })?;
        if experiment_config.comparability.require_fingerprint_match
            && pairs.iter().any(|pair| !pair.comparable)
        {
            let reasons = pairs
                .iter()
                .flat_map(|pair| pair.comparability_reasons.clone())
                .collect();
            return Err(ForgeError::PairIncomparable { reasons });
        }
        let diff = aggregate_diffs(
            &pairs,
            &experiment_config.statistics_policy,
            &experiment_config.comparability,
        );

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
            baseline_descriptor: first.baseline_descriptor,
            baseline_result: first.baseline_result,
            patched_result: first.patched_result,
            diff,
            trials: all_trials,
            completed_at,
            pairs,
        })
    }

    /// Execute one matched pair in two independently prepared workspaces.
    pub async fn run_pair(
        &self,
        fixture_path: &Path,
        patch: Option<&crate::runtime::patch::types::StructuredPatch>,
        pair_index: u32,
        trials: &mut Vec<TrialRecord>,
    ) -> ForgeResult<PairedTrialResult> {
        let timeout = self.config.container.command_timeout_secs;
        let baseline_workspace = self.backend.prepare_workspace(fixture_path).await?;
        let baseline_descriptor =
            crate::baseline::capture_baseline_provenance(&baseline_workspace.host_path).await?;
        let baseline_result = self
            .run_checks(
                &baseline_workspace.host_path,
                timeout,
                TrialSide::Baseline,
                trials,
            )
            .await?;

        let patched_workspace = self.backend.prepare_workspace(fixture_path).await?;
        let patched_descriptor =
            crate::baseline::capture_baseline_provenance(&patched_workspace.host_path).await?;
        let line_map = match patch {
            Some(patch) => {
                crate::runtime::patch::apply::apply_patch(patch, &patched_workspace.host_path)?
            }
            None => crate::runtime::patch::apply::LineAttributionMap::default(),
        };
        let patched_result = self
            .run_checks(
                &patched_workspace.host_path,
                timeout,
                TrialSide::Patched,
                trials,
            )
            .await?;
        let mut comparability_reasons = Vec::new();
        if baseline_descriptor.fingerprint() != patched_descriptor.fingerprint() {
            comparability_reasons
                .push("baseline fingerprint differs between matched workspaces".to_string());
        }
        let comparable = comparability_reasons.is_empty();
        Ok(PairedTrialResult {
            pair_index,
            baseline_descriptor,
            patched_descriptor,
            baseline_result: baseline_result.clone(),
            patched_result: patched_result.clone(),
            line_map,
            diff: ExperimentDiff::from_paired(&baseline_result, &patched_result),
            comparable,
            comparability_reasons,
        })
    }

    /// Run all check commands and aggregate into a CheckResult.
    async fn run_checks(
        &self,
        workspace: &Path,
        timeout: u64,
        side: TrialSide,
        trials: &mut Vec<TrialRecord>,
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
            seed: 0,
            cache_mode: CacheMode::Unknown,
            // ExecutionBackend does not expose network isolation state.  Keep
            // the legacy bool conservative and expose no known-good claim.
            network_available: false,
            network_mode: NetworkMode::Unknown,
            timing_admissible: false,
        });

        Ok(check_result)
    }
}

fn aggregate_diffs(
    pairs: &[PairedTrialResult],
    statistics: &StatisticsPolicy,
    comparability: &ComparabilityPolicy,
) -> ExperimentDiff {
    let Some(first) = pairs.first() else {
        return ExperimentDiff {
            effects: Vec::new(),
            regressions: 0,
            improvements: 0,
            stable_failures: 0,
            stable_passes: 0,
            statistically_meaningful: false,
            sample_warning: Some("no paired trials were available".to_string()),
        };
    };
    let mut aggregate = first.diff.clone();
    if pairs.len() <= 1 {
        return aggregate;
    }
    aggregate.regressions = pairs
        .iter()
        .map(|pair| pair.diff.regressions)
        .min()
        .unwrap_or(0);
    aggregate.improvements = pairs
        .iter()
        .map(|pair| pair.diff.improvements)
        .min()
        .unwrap_or(0);
    aggregate.stable_failures = pairs
        .iter()
        .map(|pair| pair.diff.stable_failures)
        .max()
        .unwrap_or(0);
    aggregate.stable_passes = pairs
        .iter()
        .map(|pair| pair.diff.stable_passes)
        .min()
        .unwrap_or(0);
    // Keep only effects whose full normalized identity is seen in every pair;
    // repeated trials are evidence of stability, not a way to multiply count.
    let common: std::collections::BTreeSet<_> = pairs.iter().skip(1).fold(
        first
            .diff
            .effects
            .iter()
            .map(typed_effect_identity)
            .collect(),
        |acc: std::collections::BTreeSet<_>, pair| {
            acc.intersection(
                &pair
                    .diff
                    .effects
                    .iter()
                    .map(typed_effect_identity)
                    .collect(),
            )
            .cloned()
            .collect()
        },
    );
    aggregate.effects = first
        .diff
        .effects
        .iter()
        .filter(|effect| common.contains(&typed_effect_identity(effect)))
        .cloned()
        .collect();
    let required = statistics.min_paired_trials.max(comparability.min_trials) as usize;
    aggregate.statistically_meaningful = pairs.len() >= required
        && pairs.iter().all(|pair| pair.comparable)
        && pairs.iter().all(|pair| {
            pair.diff.regressions == aggregate.regressions
                && pair.diff.improvements == aggregate.improvements
        });
    aggregate.sample_warning = if aggregate.statistically_meaningful {
        None
    } else {
        Some(format!(
            "{} comparable paired trials required for a meaningful claim; observed {}",
            required,
            pairs.len()
        ))
    };
    aggregate
}

fn typed_effect_identity(
    effect: &TypedLocatedEffect,
) -> (EffectKind, Option<PathBuf>, Option<u32>, String) {
    (
        effect.kind.clone(),
        effect.file.clone(),
        effect.line,
        effect.message.clone(),
    )
}
