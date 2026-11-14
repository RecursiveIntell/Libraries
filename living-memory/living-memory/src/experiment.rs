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
    /// One-based pair index and unique physical side identity.
    pub pair_index: u32,
    pub trial_id: String,
    pub workspace_path: PathBuf,
    pub baseline_fingerprint: String,
    pub completed: bool,
    /// Actual output from this side, never duplicated from another trial.
    pub check_outputs: Vec<crate::exec::backend::ParsedCheckOutput>,
    /// Physical output, separate from parser-derived semantic outcomes. None
    /// means an attempted command failed without a completed output record.
    pub raw_command_outputs: Vec<Option<crate::exec::backend::CommandOutput>>,
    /// Summarized check pass/fail flags.
    pub fmt_pass: bool,
    pub clippy_pass: bool,
    pub test_pass: bool,
    /// Backend used.
    pub backend_kind: ExecutionBackendKind,
    /// Duration in milliseconds.
    pub duration_ms: u64,
    /// Seed used for reproducibility.
    pub seed: Option<u64>,
    /// Whether caches were warm.
    pub cache_mode: CacheMode,
    /// Whether network was available.
    pub network_available: Option<bool>,
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
    /// Exact first-pair outputs for existing single-pair callers. For repeated
    /// runs the complete per-side evidence is in `trials`; `diff` spans pairs.
    pub baseline_result: CheckResult,
    pub patched_result: CheckResult,
    pub diff: ExperimentDiff,
    pub patch_hash: String,
    pub source_hash: String,
    pub command_plan: serde_json::Value,
    pub pair_comparability: crate::lab::evidence::PairComparability,
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
        // Admission belongs at the execution ingress, including direct library
        // callers. Never stage or execute a patch the canonical owner rejects.
        let validation = crate::runtime::patch::validate::validate_patch(patch, self.config);
        if !validation.ok {
            return Err(ForgeError::PatchValidation {
                violations: validation.violations,
            });
        }
        // Replication/follow-up and caller-selected measurement policies do not
        // yet have a qualified execution/evidence contract. Refuse them rather
        // than label a single correctness pair as replicated or timing evidence.
        if experiment_config.mode == ExperimentMode::VerificationFollowup
            || experiment_config.trial_count == 0
            || (experiment_config.mode == ExperimentMode::RepeatedPaired
                && experiment_config.trial_count < 2)
            || experiment_config.trial_count
                > experiment_config.workspace_policy.max_retained_workspaces / 2
            || (experiment_config.mode == ExperimentMode::Paired
                && experiment_config.trial_count != 1)
            || experiment_config.comparability.timing_admissible
            || experiment_config.comparability.require_fingerprint_match
                != ComparabilityPolicy::default().require_fingerprint_match
            || experiment_config.comparability.min_trials
                != ComparabilityPolicy::default().min_trials
            || experiment_config.statistics_policy.min_paired_trials
                != StatisticsPolicy::default().min_paired_trials
            || experiment_config
                .statistics_policy
                .require_timing_admissibility
                != StatisticsPolicy::default().require_timing_admissibility
            || experiment_config.workspace_policy.max_workspace_bytes
                != WorkspacePolicy::default().max_workspace_bytes
            || experiment_config.workspace_policy.max_retained_workspaces
                != WorkspacePolicy::default().max_retained_workspaces
            || experiment_config.workspace_policy.retain_failed_workspaces
            || experiment_config
                .workspace_policy
                .workspace_cleanup_ttl_secs
                != WorkspacePolicy::default().workspace_cleanup_ttl_secs
        {
            return Err(ForgeError::Config(
                "correctness pairs must fit the retained workspace limit; follow-up, timing and custom experiment policies are unsupported".into(),
            ));
        }
        let commands = self.adapter.check_commands(self.config);
        for kind in [
            crate::exec::backend::CheckKind::Fmt,
            crate::exec::backend::CheckKind::Clippy,
            crate::exec::backend::CheckKind::Test,
        ] {
            if commands
                .iter()
                .filter(|command| command.kind == kind)
                .count()
                != 1
            {
                return Err(ForgeError::Config(
                    "paired experiments require exactly one fmt, clippy and test command".into(),
                ));
            }
        }

        let mut command_plan = Vec::new();
        for command in &commands {
            command_plan.push(serde_json::json!({
                "kind": command.kind, "program": command.program, "args": command.args,
                "environment_fingerprint": blake3::hash(&serde_json::to_vec(&command.env)?).to_hex().to_string(),
            }));
        }
        let command_plan = serde_json::Value::Array(command_plan);
        let run_id = uuid::Uuid::new_v4().to_string();
        let _started_at = chrono::Utc::now().to_rfc3339();

        tracing::info!(run_id = %run_id, mode = ?experiment_config.mode, "starting experiment");

        // Freeze the input once. Every physical side starts from the same
        // snapshot, never from a baseline build's mutated working directory.
        let source_hash = workspace_digest(fixture_path)?;
        let snapshot = sandbox_workspace::prepare_workspace(fixture_path)?;
        if workspace_digest(&snapshot.host_path)? != source_hash
            || workspace_digest(fixture_path)? != source_hash
        {
            return Err(ForgeError::Fixture(
                "source changed while staging snapshot".into(),
            ));
        }
        let patch_hash = blake3::hash(&serde_json::to_vec(patch)?)
            .to_hex()
            .to_string();
        let timeout = self.config.container.command_timeout_secs;
        if timeout == 0 {
            return Err(ForgeError::Config(
                "command timeout must be positive".into(),
            ));
        }
        let mut all_trials = Vec::new();
        let mut baseline_results = Vec::new();
        let mut patched_results = Vec::new();
        let mut descriptors = Vec::new();
        let mut diffs = Vec::new();
        let mut retained = Vec::new();

        for pair_index in 1..=experiment_config.trial_count {
            for side in [TrialSide::Baseline, TrialSide::Patched] {
                let side_result: ForgeResult<CheckResult> = async {
                    let workspace = self.backend.prepare_workspace(&snapshot.host_path).await?;
                    if workspace_digest(&workspace.host_path)? != source_hash {
                        return Err(ForgeError::Fixture(
                            "staged source differs from frozen input".into(),
                        ));
                    }
                    let descriptor =
                        crate::baseline::capture_baseline_provenance(&workspace.host_path).await?;
                    let fingerprint = descriptor.fingerprint();
                    descriptors.push(descriptor);
                    if side == TrialSide::Patched {
                        crate::runtime::patch::apply::apply_patch(patch, &workspace.host_path)?;
                    }
                    let result = self
                        .run_checks(
                            &workspace.host_path,
                            &commands,
                            timeout,
                            (side, pair_index, &fingerprint),
                            &mut all_trials,
                        )
                        .await;
                    retained.push(workspace);
                    result
                }
                .await;
                match side_result {
                    Ok(result) => match side {
                        TrialSide::Baseline => baseline_results.push(result),
                        TrialSide::Patched => patched_results.push(result),
                    },
                    Err(error) => {
                        return Err(ForgeError::ExperimentIncomplete {
                            completed_sides: all_trials
                                .iter()
                                .filter(|trial| trial.completed)
                                .count(),
                            reason: error.to_string(),
                            trials: all_trials,
                        })
                    }
                }
            }
            let baseline =
                baseline_results
                    .last()
                    .ok_or_else(|| ForgeError::ExperimentIncomplete {
                        completed_sides: all_trials.iter().filter(|trial| trial.completed).count(),
                        reason: "pair completed without a baseline result".into(),
                        trials: all_trials.clone(),
                    })?;
            let patched =
                patched_results
                    .last()
                    .ok_or_else(|| ForgeError::ExperimentIncomplete {
                        completed_sides: all_trials.iter().filter(|trial| trial.completed).count(),
                        reason: "pair completed without a patched result".into(),
                        trials: all_trials.clone(),
                    })?;
            diffs.push(ExperimentDiff::from_paired(baseline, patched));
        }
        let baseline_descriptor = descriptors[0].clone();
        let mut violations = Vec::new();
        if descriptors
            .iter()
            .any(|value| value.fingerprint() != baseline_descriptor.fingerprint())
        {
            violations.push("baseline environment/provenance fingerprint drift".into());
        }
        if baseline_descriptor.rustc_version.is_empty()
            || baseline_descriptor.cargo_version.is_empty()
            || baseline_descriptor.rustc_version == "unknown"
            || baseline_descriptor.cargo_version == "unknown"
        {
            violations.push("toolchain provenance unavailable".into());
        }
        let mut diff = diffs[0].clone();
        for next in diffs.iter().skip(1) {
            diff.effects.extend(next.effects.clone());
            diff.improvements += next.improvements;
            diff.regressions += next.regressions;
            diff.stable_passes += next.stable_passes;
            diff.stable_failures += next.stable_failures;
        }
        diff.sample_warning = Some(format!(
            "{} physical correctness pairs; no independent task-level or performance significance established",
            experiment_config.trial_count,
        ));
        let unstable = |results: &[CheckResult]| {
            results.iter().skip(1).any(|result| {
                (result.fmt_pass, result.clippy_pass, result.test_pass)
                    != (
                        results[0].fmt_pass,
                        results[0].clippy_pass,
                        results[0].test_pass,
                    )
            })
        };
        if unstable(&baseline_results) || unstable(&patched_results) {
            violations.push("check outcomes vary across pairs; inconclusive".into());
            diff.effects.push(TypedLocatedEffect {
                kind: EffectKind::FlakySignal,
                file: None,
                line: None,
                message: "check outcomes vary across pairs".into(),
                in_baseline: true,
                in_patched: true,
            });
        }
        Ok(ExperimentResult {
            run_id,
            mode: experiment_config.mode,
            baseline_descriptor,
            baseline_result: baseline_results.remove(0),
            patched_result: patched_results.remove(0),
            diff,
            patch_hash,
            source_hash,
            command_plan,
            pair_comparability: crate::lab::evidence::PairComparability {
                valid: violations.is_empty(),
                violations,
            },
            trials: all_trials,
            completed_at: chrono::Utc::now().to_rfc3339(),
        })
    }

    /// Run all check commands and aggregate into a CheckResult.
    async fn run_checks(
        &self,
        workspace: &Path,
        commands: &[crate::exec::backend::CheckCommand],
        timeout: u64,
        identity: (TrialSide, u32, &str),
        trials: &mut Vec<TrialRecord>,
    ) -> ForgeResult<CheckResult> {
        let start = std::time::Instant::now();
        let (side, pair_index, baseline_fingerprint) = identity;

        let unrun = crate::exec::backend::ParsedCheckOutput {
            exit_code: -2,
            ..Default::default()
        };
        let mut fmt_output = unrun.clone();
        let mut clippy_output = unrun.clone();
        let mut test_output = unrun;
        let mut terminal_error = None;
        let mut check_outputs = Vec::new();
        let mut raw_command_outputs = Vec::new();

        for cmd in commands {
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
                    raw_command_outputs.push(Some(output.clone()));
                    let parsed = self.adapter.parse_check_output(
                        cmd,
                        &output.stdout,
                        &output.stderr,
                        output.exit_code,
                    );
                    check_outputs.push(parsed.clone());
                    match kind {
                        crate::exec::backend::CheckKind::Fmt => fmt_output = parsed,
                        crate::exec::backend::CheckKind::Clippy => clippy_output = parsed,
                        crate::exec::backend::CheckKind::Test => test_output = parsed,
                    }
                }
                Err(error) => {
                    raw_command_outputs.push(None);
                    // Record timeout as failure
                    let parsed = crate::exec::backend::ParsedCheckOutput {
                        check_kind: kind.clone(),
                        exit_code: -1,
                        effects: vec![],
                        raw_stdout: String::new(),
                        raw_stderr: error.to_string(),
                    };
                    check_outputs.push(parsed.clone());
                    match kind {
                        crate::exec::backend::CheckKind::Fmt => fmt_output = parsed,
                        crate::exec::backend::CheckKind::Clippy => clippy_output = parsed,
                        crate::exec::backend::CheckKind::Test => test_output = parsed,
                    }
                    terminal_error = Some(error);
                    break;
                }
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
            pair_index,
            trial_id: uuid::Uuid::new_v4().to_string(),
            workspace_path: workspace.to_path_buf(),
            baseline_fingerprint: baseline_fingerprint.into(),
            completed: terminal_error.is_none() && check_outputs.len() == commands.len(),
            check_outputs,
            raw_command_outputs,
            fmt_pass: check_result.fmt_pass,
            clippy_pass: check_result.clippy_pass,
            test_pass: check_result.test_pass,
            backend_kind: self.backend.kind(),
            duration_ms,
            seed: None,
            cache_mode: CacheMode::Unknown,
            network_available: None,
            timing_admissible: false,
        });

        match terminal_error {
            Some(error) => Err(error),
            None => Ok(check_result),
        }
    }
}

/// Strict content identity for a frozen, regular-file source tree. Symlinks,
/// special files, unreadable entries and disappearing inputs are not skipped.
fn workspace_digest(root: &Path) -> ForgeResult<String> {
    if !root.is_dir() {
        return Err(ForgeError::Fixture("fixture is not a directory".into()));
    }
    let mut files = Vec::new();
    for entry in walkdir::WalkDir::new(root).follow_links(false) {
        let entry = entry.map_err(|error| ForgeError::Fixture(error.to_string()))?;
        if entry.file_type().is_dir() {
            continue;
        }
        if !entry.file_type().is_file() {
            return Err(ForgeError::Fixture(
                "non-regular source entry refused".into(),
            ));
        }
        files.push(
            entry
                .path()
                .strip_prefix(root)
                .map_err(|error| ForgeError::Fixture(error.to_string()))?
                .to_path_buf(),
        );
    }
    files.sort();
    let mut hasher = blake3::Hasher::new();
    for path in files {
        let name = path
            .to_str()
            .ok_or_else(|| ForgeError::Fixture("non-UTF8 source path refused".into()))?;
        let bytes = std::fs::read(root.join(&path))?;
        hasher.update(&(name.len() as u64).to_le_bytes());
        hasher.update(name.as_bytes());
        hasher.update(&(bytes.len() as u64).to_le_bytes());
        hasher.update(&bytes);
    }
    Ok(hasher.finalize().to_hex().to_string())
}
