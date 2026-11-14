//! Python adapter development conformance; no claim of evaluator isolation.
use forge_engine::{
    EditOp, ExperimentConfig, FileEdit, FileMode, ForgeConfig, HostBackend, LineRange,
    PairedExperimentRunner, ProjectAdapter, PythonAdapter, StructuredPatch,
};
use std::path::PathBuf;

async fn adapter() -> PythonAdapter {
    PythonAdapter::new(
        PathBuf::from("/usr/bin/python3"),
        PathBuf::from("/home/sikmindz/.local/bin/ruff"),
    )
    .await
    .unwrap()
}
fn fixture(with_tests: bool) -> (tempfile::TempDir, StructuredPatch) {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("pyproject.toml"),
        "[project]\nname = 'forge-python-fixture'\nversion = '0.0.0'\nrequires-python = '>=3.14'\n",
    )
    .unwrap();
    std::fs::write(
        dir.path().join("helper.py"),
        "def add(a, b):\n    return a - b\n",
    )
    .unwrap();
    if with_tests {
        std::fs::write(dir.path().join("test_helper.py"), "import unittest\n\nfrom helper import add\n\n\nclass AddTest(unittest.TestCase):\n    def test_add(self):\n        self.assertEqual(add(2, 3), 5)\n").unwrap();
    }
    let patch = StructuredPatch {
        patch_id: uuid::Uuid::from_u128(314),
        summary: "fix arithmetic development control".into(),
        notes: vec![],
        edits: vec![FileEdit {
            path: "helper.py".into(),
            mode: Some(FileMode::Modify),
            ops: vec![EditOp::Replace {
                range: LineRange {
                    start: 2,
                    end_exclusive: 3,
                },
                lines: vec!["    return a + b".into()],
            }],
        }],
    };
    (dir, patch)
}
#[tokio::test]
async fn real_python_pair_corrects_failure_without_installing_dependencies() {
    let adapter = adapter().await;
    let (fixture, patch) = fixture(true);
    let cfg = ForgeConfig::default();
    let backend = HostBackend::new(&cfg);
    let result = PairedExperimentRunner::new(&backend, &adapter, &cfg)
        .run(fixture.path(), &patch, &ExperimentConfig::default())
        .await
        .unwrap();
    assert!(!result.baseline_result.test_pass);
    assert!(
        result.patched_result.test_pass,
        "{:?}",
        result.patched_result.test_output
    );
    assert!(result.patched_result.fmt_pass);
    assert!(result.patched_result.clippy_pass);
    assert!(result.trials[1].raw_command_outputs[2]
        .as_ref()
        .unwrap()
        .stdout
        .contains("forge_unittest_v1"));
    assert!(adapter.python_version.starts_with("Python 3.14"));
    assert!(adapter.ruff_version.starts_with("ruff "));
}
#[tokio::test]
async fn empty_python_suite_cannot_be_a_passing_experiment() {
    let adapter = adapter().await;
    let (fixture, patch) = fixture(false);
    let cfg = ForgeConfig::default();
    let backend = HostBackend::new(&cfg);
    let result = PairedExperimentRunner::new(&backend, &adapter, &cfg)
        .run(fixture.path(), &patch, &ExperimentConfig::default())
        .await
        .unwrap();
    assert!(!result.baseline_result.test_pass && !result.patched_result.test_pass);
    assert_eq!(result.diff.improvements, 0);
}
#[tokio::test]
async fn python_tools_are_explicit_and_bad_summaries_refuse() {
    assert!(
        PythonAdapter::new("python3".into(), "/home/sikmindz/.local/bin/ruff".into())
            .await
            .is_err()
    );
    assert!(
        PythonAdapter::new("/bin/false".into(), "/home/sikmindz/.local/bin/ruff".into())
            .await
            .is_err()
    );
    let adapter = adapter().await;
    let commands = adapter.check_commands(&ForgeConfig::default());
    let good = r#"{"forge_unittest_v1":true,"tests_run":1,"skipped":0,"failures":0,"errors":0}"#;
    for bad in [
        "".to_string(),
        "{}".into(),
        good.replace("\"tests_run\":1", "\"tests_run\":0"),
        format!("{good}\n{good}"),
    ] {
        assert_ne!(
            adapter
                .parse_check_output(&commands[2], &bad, "", 0)
                .exit_code,
            0
        );
    }
    assert_eq!(
        adapter
            .parse_check_output(&commands[2], good, "", 0)
            .exit_code,
        0
    );
}
#[test]
fn candidate_cannot_rewrite_python_evaluator_or_build_manifests() {
    let cfg = ForgeConfig::default();
    let (_, mut patch) = fixture(true);
    for path in [
        "test_helper.py",
        "nested/test_helper.py",
        "nested/conftest.py",
        "pyproject.toml",
        "Cargo.toml",
        "build.rs",
    ] {
        patch.edits[0].path = path.into();
        assert!(
            !forge_engine::validate_patch(&patch, &cfg).ok,
            "unprotected evaluator/build path: {path}"
        );
    }
}
