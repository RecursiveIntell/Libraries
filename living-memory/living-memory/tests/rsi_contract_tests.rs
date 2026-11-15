//! Behavioral regression witnesses for the paired execution ingress.
use async_trait::async_trait;
use forge_engine::{
    CargoAdapter, CheckCommand, CommandOutput, EditOp, ExecutionBackend, ExecutionBackendKind,
    ExperimentConfig, ExperimentMode, FileEdit, FileMode, ForgeConfig, ForgeError, ForgeResult,
    HostBackend, LineRange, LogBundle, PairedExperimentRunner, ParsedCheckOutput, ProjectAdapter,
    StructuredPatch, Workspace,
};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

fn fixture() -> (tempfile::TempDir, StructuredPatch) {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir(dir.path().join("src")).unwrap();
    std::fs::write(
        dir.path().join("Cargo.toml"),
        "[package]\nname = \"rsi-admission-fixture\"\nversion = \"0.0.0\"\nedition = \"2021\"\n",
    )
    .unwrap();
    std::fs::write(dir.path().join("src/lib.rs"), "pub fn add(a: i32, b: i32) -> i32 {\n    a - b\n}\n\n#[cfg(test)]\nmod tests {\n    #[test]\n    fn adds() {\n        assert_eq!(super::add(2, 3), 5);\n    }\n}\n").unwrap();
    let patch = StructuredPatch {
        patch_id: uuid::Uuid::from_u128(1),
        summary: "fix arithmetic".into(),
        edits: vec![FileEdit {
            path: PathBuf::from("src/lib.rs"),
            ops: vec![EditOp::Replace {
                range: LineRange {
                    start: 2,
                    end_exclusive: 3,
                },
                lines: vec!["    a + b".into()],
            }],
            mode: Some(FileMode::Modify),
        }],
        notes: Vec::new(),
    };
    (dir, patch)
}

fn config() -> ForgeConfig {
    ForgeConfig {
        execution_backend_preference: "host".into(),
        container: forge_engine::config::ContainerConfig {
            command_timeout_secs: 20,
            ..Default::default()
        },
        ..ForgeConfig::default()
    }
}

// Instrument the actual host backend. Commands are really executed, not mocked.
struct CountedHost {
    host: HostBackend,
    prepared: AtomicUsize,
    commands: AtomicUsize,
    timeout_at: Option<usize>,
}
impl CountedHost {
    fn new(cfg: &ForgeConfig) -> Self {
        Self {
            host: HostBackend::new(cfg),
            prepared: AtomicUsize::new(0),
            commands: AtomicUsize::new(0),
            timeout_at: None,
        }
    }
}
#[async_trait]
impl ExecutionBackend for CountedHost {
    fn kind(&self) -> ExecutionBackendKind {
        self.host.kind()
    }
    async fn prepare_workspace(&self, fixture: &Path) -> ForgeResult<Workspace> {
        self.prepared.fetch_add(1, Ordering::SeqCst);
        self.host.prepare_workspace(fixture).await
    }
    async fn run_command(
        &self,
        workspace: &Path,
        program: &str,
        args: &[&str],
        env: &[(&str, &str)],
        timeout: u64,
    ) -> ForgeResult<CommandOutput> {
        let attempt = self.commands.fetch_add(1, Ordering::SeqCst) + 1;
        if self.timeout_at == Some(attempt) {
            return self
                .host
                .run_command(workspace, "sh", &["-c", "sleep 3"], &[], 1)
                .await;
        }
        self.host
            .run_command(workspace, program, args, env, timeout)
            .await
    }
    async fn collect_logs(
        &self,
        fmt: &CommandOutput,
        clippy: &CommandOutput,
        test: &CommandOutput,
    ) -> ForgeResult<LogBundle> {
        self.host.collect_logs(fmt, clippy, test).await
    }
}

#[tokio::test]
async fn patch_policy_refuses_before_workspace_or_commands() {
    for violation in [
        "cap",
        "forbidden",
        "test_file",
        "parent",
        "absolute",
        "empty_patch",
    ] {
        let (fixture, mut patch) = fixture();
        let mut cfg = config();
        match violation {
            "cap" => cfg.caps.max_files_changed = 0,
            "forbidden" => cfg.forbidden_paths = vec!["src/lib.rs".into()],
            "test_file" => patch.edits[0].path = "tests/check.rs".into(),
            "parent" => patch.edits[0].path = "../escape.rs".into(),
            "absolute" => patch.edits[0].path = "/tmp/escape.rs".into(),
            "empty_patch" => patch.edits.clear(),
            _ => unreachable!(),
        }
        assert!(
            !forge_engine::validate_patch(&patch, &cfg).ok,
            "invalid fixture: {violation}"
        );
        let backend = CountedHost::new(&cfg);
        let runner = PairedExperimentRunner::new(&backend, &CargoAdapter, &cfg);
        let result = runner
            .run(fixture.path(), &patch, &ExperimentConfig::default())
            .await;
        assert!(
            matches!(result, Err(ForgeError::PatchValidation { .. })),
            "wrong rejection for {violation}: {result:?}"
        );
        assert_eq!(
            backend.prepared.load(Ordering::SeqCst),
            0,
            "prepared invalid {violation}"
        );
        assert_eq!(
            backend.commands.load(Ordering::SeqCst),
            0,
            "executed invalid {violation}"
        );
    }
}

#[tokio::test]
async fn within_policy_patch_runs_real_baseline_and_candidate() {
    let (fixture, patch) = fixture();
    let cfg = config();
    let backend = CountedHost::new(&cfg);
    let runner = PairedExperimentRunner::new(&backend, &CargoAdapter, &cfg);
    let result = runner
        .run(fixture.path(), &patch, &ExperimentConfig::default())
        .await
        .unwrap();
    assert!(!result.baseline_result.test_pass);
    assert!(result.patched_result.test_pass);
    assert_eq!(result.trials.len(), 2);
    assert_eq!(backend.commands.load(Ordering::SeqCst), 6);
    assert_eq!(backend.prepared.load(Ordering::SeqCst), 2);
}

#[tokio::test]
async fn repeated_pairs_execute_every_side_in_fresh_workspaces() {
    let (fixture, patch) = fixture();
    let cfg = config();
    let backend = CountedHost::new(&cfg);
    let runner = PairedExperimentRunner::new(&backend, &CargoAdapter, &cfg);
    let experiment = ExperimentConfig {
        mode: ExperimentMode::RepeatedPaired,
        trial_count: 3,
        ..ExperimentConfig::default()
    };
    let result = runner
        .run(fixture.path(), &patch, &experiment)
        .await
        .unwrap();
    assert_eq!(result.trials.len(), 6);
    assert_eq!(backend.prepared.load(Ordering::SeqCst), 6);
    assert_eq!(backend.commands.load(Ordering::SeqCst), 18);
    assert!(
        !result.diff.statistically_meaningful,
        "build repetitions are not independent task trials"
    );
    for trial in &result.trials {
        assert_eq!(
            trial.test_pass,
            trial.side == forge_engine::experiment::TrialSide::Patched
        );
        assert!(trial.completed);
        assert_eq!(trial.check_outputs.len(), 3);
        assert!(!trial.check_outputs[2].raw_stdout.is_empty());
        assert!(trial.seed.is_none());
        assert!(trial.network_available.is_none());
    }
    let ids: std::collections::BTreeSet<_> = result.trials.iter().map(|t| &t.trial_id).collect();
    let workspaces: std::collections::BTreeSet<_> =
        result.trials.iter().map(|t| &t.workspace_path).collect();
    assert_eq!(ids.len(), 6);
    assert_eq!(workspaces.len(), 6);
    assert!(
        result.pair_comparability.valid,
        "{:?}",
        result.pair_comparability
    );
    let encoded = serde_json::to_string(&result.trials).unwrap();
    let decoded: Vec<forge_engine::TrialRecord> = serde_json::from_str(&encoded).unwrap();
    assert_eq!(decoded.len(), 6);
    assert_eq!(decoded[5].pair_index, 3);
    assert_eq!(decoded[5].trial_id, result.trials[5].trial_id);
}

#[tokio::test]
async fn invalid_modes_counts_and_unqualified_timing_refuse_before_execution() {
    for invalid in ["zero", "paired_count", "followup", "timing", "too_many"] {
        let (fixture, patch) = fixture();
        let cfg = config();
        let backend = CountedHost::new(&cfg);
        let runner = PairedExperimentRunner::new(&backend, &CargoAdapter, &cfg);
        let mut experiment = ExperimentConfig::default();
        match invalid {
            "zero" => experiment.trial_count = 0,
            "paired_count" => experiment.trial_count = 3,
            "followup" => experiment.mode = ExperimentMode::VerificationFollowup,
            "timing" => experiment.comparability.timing_admissible = true,
            "too_many" => {
                experiment.mode = ExperimentMode::RepeatedPaired;
                experiment.trial_count = 5;
            }
            _ => unreachable!(),
        }
        assert!(
            matches!(
                runner.run(fixture.path(), &patch, &experiment).await,
                Err(ForgeError::Config(_))
            ),
            "accepted {invalid}"
        );
        assert_eq!(backend.prepared.load(Ordering::SeqCst), 0);
        assert_eq!(backend.commands.load(Ordering::SeqCst), 0);
    }
}

struct EmptyAdapter;
impl ProjectAdapter for EmptyAdapter {
    fn detect(_: &Path) -> bool {
        true
    }
    fn name(&self) -> &str {
        "empty-negative-control"
    }
    fn check_commands(&self, _: &ForgeConfig) -> Vec<CheckCommand> {
        Vec::new()
    }
    fn parse_check_output(&self, _: &CheckCommand, _: &str, _: &str, _: i32) -> ParsedCheckOutput {
        unreachable!("empty adapter cannot execute")
    }
}
#[tokio::test]
async fn empty_check_plan_cannot_be_a_passing_experiment() {
    let (fixture, patch) = fixture();
    let cfg = config();
    let backend = CountedHost::new(&cfg);
    let runner = PairedExperimentRunner::new(&backend, &EmptyAdapter, &cfg);
    assert!(matches!(
        runner
            .run(fixture.path(), &patch, &ExperimentConfig::default())
            .await,
        Err(ForgeError::Config(_))
    ));
    assert_eq!(backend.prepared.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn actual_timeout_retains_completed_and_interrupted_sides_only() {
    let (fixture, patch) = fixture();
    let cfg = config();
    let mut backend = CountedHost::new(&cfg);
    backend.timeout_at = Some(9);
    let experiment = ExperimentConfig {
        mode: ExperimentMode::RepeatedPaired,
        trial_count: 3,
        ..Default::default()
    };
    let runner = PairedExperimentRunner::new(&backend, &CargoAdapter, &cfg);
    let error = runner
        .run(fixture.path(), &patch, &experiment)
        .await
        .unwrap_err();
    match error {
        ForgeError::ExperimentIncomplete {
            completed_sides,
            reason,
            trials,
        } => {
            assert_eq!(completed_sides, 2);
            assert_eq!(trials.len(), 3);
            assert!(trials[0].completed && trials[1].completed);
            assert!(!trials[2].completed);
            assert_eq!(trials[2].pair_index, 2);
            assert_eq!(trials[2].check_outputs.len(), 3);
            assert!(reason.contains("timeout"), "{reason}");
            assert!(trials[2].check_outputs[2].raw_stderr.contains("timeout"));
        }
        other => panic!("wrong failure: {other:?}"),
    }
    assert_eq!(backend.commands.load(Ordering::SeqCst), 9);
}

#[cfg(unix)]
#[tokio::test]
async fn symlink_source_refuses_before_backend_dispatch() {
    let (fixture, patch) = fixture();
    std::os::unix::fs::symlink("lib.rs", fixture.path().join("src/link.rs")).unwrap();
    let cfg = config();
    let backend = CountedHost::new(&cfg);
    let runner = PairedExperimentRunner::new(&backend, &CargoAdapter, &cfg);
    assert!(matches!(
        runner
            .run(fixture.path(), &patch, &ExperimentConfig::default())
            .await,
        Err(ForgeError::Fixture(_))
    ));
    assert_eq!(backend.prepared.load(Ordering::SeqCst), 0);
    assert_eq!(backend.commands.load(Ordering::SeqCst), 0);
}

#[test]
fn empty_or_ignored_only_test_execution_is_not_a_passing_check() {
    let adapter = CargoAdapter;
    let cfg = config();
    let commands = adapter.check_commands(&cfg);
    let test = &commands[2];
    for output in [
        "running 0 tests\ntest result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s\n",
        "running 1 test\ntest skip ... ignored\ntest result: ok. 0 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.00s\n",
        "",
    ] {
        let parsed = adapter.parse_check_output(test, output, "", 0);
        assert_ne!(parsed.exit_code, 0, "accepted no executed test: {output:?}");
    }
    let actual = "test adds ... ok\ntest result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s\n";
    assert_eq!(adapter.parse_check_output(test, actual, "", 0).exit_code, 0);
}
