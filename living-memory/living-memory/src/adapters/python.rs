//! Explicit Python 3.14+ component checks through the existing ProjectAdapter.
//!
//! This runs development checks, NOT an independently custodied evaluator.
//! It does not install dependencies, infer package compatibility, select a
//! backend, or authorize Python targets in Pilot/bridge automatically.
use std::path::{Path, PathBuf};
use tokio::process::Command;

use super::ProjectAdapter;
use crate::config::ForgeConfig;
use crate::error::{ForgeError, ForgeResult};
use crate::exec::backend::{CheckCommand, CheckKind, ParsedCheckOutput};

pub struct PythonAdapter {
    python: PathBuf,
    ruff: PathBuf,
    pub python_version: String,
    pub ruff_version: String,
}

async fn probe(executable: &Path, args: &[&str]) -> ForgeResult<String> {
    if !executable.is_absolute() || !executable.is_file() {
        return Err(ForgeError::Config(
            "Python adapter requires explicit absolute tool paths".into(),
        ));
    }
    let output = tokio::time::timeout(
        std::time::Duration::from_secs(5),
        Command::new(executable)
            .args(args)
            .env_clear()
            .kill_on_drop(true)
            .output(),
    )
    .await
    .map_err(|_| ForgeError::Config("tool version probe timed out".into()))??;
    if !output.status.success() {
        return Err(ForgeError::Config(
            "Python adapter tool version probe failed".into(),
        ));
    }
    String::from_utf8(output.stdout)
        .map(|value| value.trim().to_string())
        .map_err(|_| ForgeError::Config("non-UTF8 tool version refused".into()))
}

impl PythonAdapter {
    /// Admit explicit tooling only. No PATH/default/older-interpreter fallback.
    pub async fn new(python: PathBuf, ruff: PathBuf) -> ForgeResult<Self> {
        let version = probe(&python, &["--version"]).await?;
        let numbers: Vec<_> = version
            .strip_prefix("Python ")
            .ok_or_else(|| ForgeError::Config("not a Python version response".into()))?
            .split('.')
            .take(2)
            .collect();
        if numbers.len() != 2
            || numbers[0] != "3"
            || match numbers[1].parse::<u32>() {
                Ok(minor) => minor < 14,
                Err(_) => true,
            }
        {
            return Err(ForgeError::Config("Python 3.14 or newer required".into()));
        }
        let ruff_version = probe(&ruff, &["--version"]).await?;
        if !ruff_version.starts_with("ruff ") {
            return Err(ForgeError::Config("not a Ruff version response".into()));
        }
        Ok(Self {
            python,
            ruff,
            python_version: version,
            ruff_version,
        })
    }
}

// Candidate imports occur only after the floor guard. unittest's discovered
// count is inspected by the trusted development wrapper, not inferred from a
// process exit code. A hostile module can still attack its process: acceptance
// labels/scorer therefore MUST live in a separate evaluator, outside this path.
const UNITTEST_RUNNER: &str = r#"
import sys
if sys.version_info < (3, 14):
    raise SystemExit('Python 3.14 or newer required')
import json
import unittest
suite = unittest.defaultTestLoader.discover('.')
result = unittest.TextTestRunner(verbosity=2).run(suite)
print(json.dumps({'forge_unittest_v1': True, 'tests_run': result.testsRun,
                  'skipped': len(result.skipped), 'failures': len(result.failures),
                  'errors': len(result.errors)}))
raise SystemExit(0 if result.wasSuccessful() and result.testsRun > len(result.skipped) else 1)
"#;

impl ProjectAdapter for PythonAdapter {
    fn detect(workspace: &Path) -> bool {
        workspace.join("pyproject.toml").is_file()
    }
    fn name(&self) -> &str {
        "python-3.14-development-checks"
    }
    fn check_commands(&self, _: &ForgeConfig) -> Vec<CheckCommand> {
        let ruff = self.ruff.to_string_lossy().into_owned();
        vec![
            CheckCommand {
                kind: CheckKind::Fmt,
                program: ruff.clone(),
                args: [
                    "format",
                    "--check",
                    "--no-cache",
                    "--isolated",
                    "--target-version",
                    "py314",
                    ".",
                ]
                .map(str::to_string)
                .to_vec(),
                env: vec![],
            },
            CheckCommand {
                kind: CheckKind::Clippy,
                program: ruff,
                args: [
                    "check",
                    "--no-cache",
                    "--isolated",
                    "--target-version",
                    "py314",
                    ".",
                ]
                .map(str::to_string)
                .to_vec(),
                env: vec![],
            },
            CheckCommand {
                kind: CheckKind::Test,
                program: self.python.to_string_lossy().into_owned(),
                args: vec![
                    "-I".into(),
                    "-S".into(),
                    "-c".into(),
                    UNITTEST_RUNNER.into(),
                ],
                env: vec![],
            },
        ]
    }
    fn parse_check_output(
        &self,
        cmd: &CheckCommand,
        stdout: &str,
        stderr: &str,
        exit_code: i32,
    ) -> ParsedCheckOutput {
        let mut effective_code = exit_code;
        if cmd.kind == CheckKind::Test && exit_code == 0 {
            let records: Vec<serde_json::Value> = stdout
                .lines()
                .filter_map(|line| serde_json::from_str::<serde_json::Value>(line).ok())
                .filter(|value| {
                    value.get("forge_unittest_v1").and_then(|v| v.as_bool()) == Some(true)
                })
                .collect();
            let valid = records.len() == 1
                && {
                    let value = &records[0];
                    matches!((value["tests_run"].as_u64(), value["skipped"].as_u64(), value["failures"].as_u64(), value["errors"].as_u64()),
                    (Some(run), Some(skipped), Some(0), Some(0)) if run > skipped)
                };
            if !valid {
                effective_code = -2;
            }
        }
        ParsedCheckOutput {
            check_kind: cmd.kind.clone(),
            exit_code: effective_code,
            effects: vec![],
            raw_stdout: stdout.into(),
            raw_stderr: stderr.into(),
        }
    }
}
