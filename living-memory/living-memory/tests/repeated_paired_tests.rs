use async_trait::async_trait;
use forge_engine::adapters::ProjectAdapter;
use forge_engine::config::ForgeConfig;
use forge_engine::error::{ForgeError, ForgeResult};
use forge_engine::exec::backend::{
    CheckCommand, CheckKind, CommandOutput, CommandTimings, ExecutionBackend, ExecutionBackendKind,
    LogBundle, ParsedCheckOutput,
};
use forge_engine::{
    ExperimentConfig, ExperimentMode, PairedExperimentRunner, StatisticsPolicy, TrialSide,
};
use sandbox_workspace::Workspace;
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;
use typed_patch::StructuredPatch;

#[derive(Default)]
struct RecordingBackend {
    prepared: AtomicUsize,
    observed_patched: Mutex<Vec<bool>>,
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
        workspace: &Path,
        _program: &str,
        _args: &[&str],
        _env: &[(&str, &str)],
        _timeout_secs: u64,
    ) -> ForgeResult<CommandOutput> {
        let source = std::fs::read_to_string(workspace.join("src/lib.rs"))?;
        self.observed_patched
            .lock()
            .map_err(|_| ForgeError::ExperimentFailed("recording backend lock poisoned".into()))?
            .push(source.contains("42"));
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

struct OneCheckAdapter;

impl ProjectAdapter for OneCheckAdapter {
    fn detect(_workspace: &Path) -> bool {
        true
    }

    fn name(&self) -> &str {
        "repeated-paired-test"
    }

    fn check_commands(&self, _config: &ForgeConfig) -> Vec<CheckCommand> {
        vec![CheckCommand {
            kind: CheckKind::Fmt,
            program: "record".into(),
            args: Vec::new(),
            env: Vec::new(),
        }]
    }

    fn parse_check_output(
        &self,
        cmd: &CheckCommand,
        stdout: &str,
        stderr: &str,
        exit_code: i32,
    ) -> ParsedCheckOutput {
        ParsedCheckOutput {
            check_kind: cmd.kind.clone(),
            exit_code,
            effects: Vec::new(),
            raw_stdout: stdout.into(),
            raw_stderr: stderr.into(),
        }
    }
}

fn fixture() -> tempfile::TempDir {
    let root = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(root.path().join("src")).unwrap();
    std::fs::write(
        root.path().join("Cargo.toml"),
        "[package]\nname='paired-fixture'\nversion='0.1.0'\nedition='2021'\n",
    )
    .unwrap();
    std::fs::write(
        root.path().join("src/lib.rs"),
        "pub fn answer() -> u32 { 1 }\n",
    )
    .unwrap();
    root
}

fn patch() -> StructuredPatch {
    serde_json::from_value(serde_json::json!({
        "patch_id": "00000000-0000-4000-8000-000000000020",
        "summary": "change answer",
        "edits": [{
            "path": "src/lib.rs",
            "ops": [{"Replace": {
                "range": {"start": 1, "end_exclusive": 2},
                "lines": ["pub fn answer() -> u32 { 42 }"]
            }}],
            "mode": "Modify"
        }],
        "notes": []
    }))
    .unwrap()
}

fn repeated(seed: u64) -> ExperimentConfig {
    ExperimentConfig {
        mode: ExperimentMode::RepeatedPaired,
        trial_count: 20,
        statistics_policy: StatisticsPolicy {
            min_paired_trials: 20,
            require_timing_admissibility: true,
        },
        trial_order_seed: seed,
        ..ExperimentConfig::default()
    }
}

fn order(result: &forge_engine::ExperimentResult) -> Vec<(u32, u32, TrialSide)> {
    result
        .trials
        .iter()
        .map(|trial| (trial.pair_index, trial.order_index, trial.side))
        .collect()
}

#[tokio::test]
async fn repeated_paired_runs_twenty_fresh_isolated_pairs_deterministically() {
    let fixture = fixture();
    let patch = patch();
    let config = ForgeConfig::default();
    let adapter = OneCheckAdapter;

    let backend = RecordingBackend::default();
    let runner = PairedExperimentRunner::new(&backend, &adapter, &config);
    let first = runner
        .run(fixture.path(), &patch, &repeated(7))
        .await
        .unwrap();

    assert_eq!(first.trials.len(), 40);
    assert_eq!(backend.prepared.load(Ordering::SeqCst), 40);
    assert!(first.trials.iter().all(|trial| trial.seed == 7));
    for pair_index in 0..20 {
        let pair = first
            .trials
            .iter()
            .filter(|trial| trial.pair_index == pair_index)
            .collect::<Vec<_>>();
        assert_eq!(pair.len(), 2);
        assert!(pair.iter().any(|trial| trial.side == TrialSide::Baseline));
        assert!(pair.iter().any(|trial| trial.side == TrialSide::Patched));
    }
    let first_sides = first
        .trials
        .iter()
        .filter(|trial| trial.order_index == 0)
        .map(|trial| trial.side)
        .collect::<Vec<_>>();
    assert!(first_sides.contains(&TrialSide::Baseline));
    assert!(first_sides.contains(&TrialSide::Patched));

    let observations = backend.observed_patched.lock().unwrap().clone();
    assert_eq!(observations.len(), first.trials.len());
    for (trial, patched_tree) in first.trials.iter().zip(observations) {
        assert_eq!(patched_tree, trial.side == TrialSide::Patched);
    }

    let same_backend = RecordingBackend::default();
    let same = PairedExperimentRunner::new(&same_backend, &adapter, &config)
        .run(fixture.path(), &patch, &repeated(7))
        .await
        .unwrap();
    assert_eq!(first.run_id, same.run_id);
    assert_eq!(order(&first), order(&same));

    let changed_seed_backend = RecordingBackend::default();
    let changed_seed = PairedExperimentRunner::new(&changed_seed_backend, &adapter, &config)
        .run(fixture.path(), &patch, &repeated(8))
        .await
        .unwrap();
    assert_ne!(first.run_id, changed_seed.run_id);
    assert_ne!(order(&first), order(&changed_seed));

    std::fs::write(
        fixture.path().join("src/lib.rs"),
        "pub fn answer() -> u32 { 2 }\n",
    )
    .unwrap();
    let changed_fixture_backend = RecordingBackend::default();
    let changed_fixture = PairedExperimentRunner::new(&changed_fixture_backend, &adapter, &config)
        .run(fixture.path(), &patch, &repeated(7))
        .await
        .unwrap();
    assert_ne!(first.run_id, changed_fixture.run_id);
}

#[tokio::test]
async fn paired_and_invalid_modes_have_exact_fail_closed_denominators() {
    let fixture = fixture();
    let patch = patch();
    let config = ForgeConfig::default();
    let adapter = OneCheckAdapter;

    let paired_backend = RecordingBackend::default();
    let paired = PairedExperimentRunner::new(&paired_backend, &adapter, &config)
        .run(fixture.path(), &patch, &ExperimentConfig::default())
        .await
        .unwrap();
    assert_eq!(paired.trials.len(), 2);
    assert_eq!(paired_backend.prepared.load(Ordering::SeqCst), 2);

    for invalid in [
        ExperimentConfig {
            mode: ExperimentMode::RepeatedPaired,
            trial_count: 0,
            ..ExperimentConfig::default()
        },
        ExperimentConfig {
            mode: ExperimentMode::RepeatedPaired,
            trial_count: 2,
            statistics_policy: StatisticsPolicy {
                min_paired_trials: 3,
                require_timing_admissibility: true,
            },
            ..ExperimentConfig::default()
        },
        ExperimentConfig {
            mode: ExperimentMode::VerificationFollowup,
            ..ExperimentConfig::default()
        },
    ] {
        let backend = RecordingBackend::default();
        let error = PairedExperimentRunner::new(&backend, &adapter, &config)
            .run(fixture.path(), &patch, &invalid)
            .await
            .unwrap_err();
        assert!(matches!(error, ForgeError::Config(_)));
        assert_eq!(backend.prepared.load(Ordering::SeqCst), 0);
    }
}
