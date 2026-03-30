use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::{ForgeError, ForgeResult};

/// A task loaded from a fixture directory.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvalTask {
    pub task_id: String,
    pub prompt: String,
    pub constraints: TaskConstraints,
    pub weights: TaskWeights,
    pub expected: TaskExpected,
    pub cea: TaskCea,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskConstraints {
    #[serde(default)]
    pub allow_test_modifications: bool,
    pub max_files_changed: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskWeights {
    #[serde(default = "default_correctness_weight")]
    pub correctness: f64,
    #[serde(default = "default_novelty_weight")]
    pub novelty: f64,
    #[serde(default = "default_stability_weight")]
    pub stability: f64,
}

fn default_correctness_weight() -> f64 {
    0.7
}
fn default_novelty_weight() -> f64 {
    0.2
}
fn default_stability_weight() -> f64 {
    0.1
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskExpected {
    #[serde(default = "default_true")]
    pub require_fmt: bool,
    #[serde(default = "default_true")]
    pub require_clippy: bool,
    #[serde(default = "default_true")]
    pub require_tests: bool,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskCea {
    #[serde(default = "default_true")]
    pub instrument: bool,
    pub risk_threshold_override: Option<f64>,
}

/// A loaded fixture suite.
#[derive(Debug)]
pub struct EvalSuite {
    pub name: String,
    pub tasks: Vec<LoadedTask>,
}

/// A task with its fixture path.
#[derive(Debug)]
pub struct LoadedTask {
    pub task: EvalTask,
    pub fixture_path: PathBuf,
}

/// Load an evaluation suite from a fixtures directory.
pub fn load_suite(suite_dir: &Path) -> ForgeResult<EvalSuite> {
    let suite_name = suite_dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    let mut tasks = Vec::new();

    if !suite_dir.exists() {
        return Err(ForgeError::Fixture(format!(
            "suite directory does not exist: {}",
            suite_dir.display()
        )));
    }

    // Each subdirectory is a task
    let mut entries: Vec<_> = std::fs::read_dir(suite_dir)?
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
        .collect();
    entries.sort_by_key(|e| e.file_name());

    for entry in entries {
        let task_dir = entry.path();
        let task_json = task_dir.join("task.json");
        let repo_dir = task_dir.join("repo");

        if !task_json.exists() {
            tracing::warn!("skipping fixture {}: no task.json", task_dir.display());
            continue;
        }

        if !repo_dir.exists() {
            tracing::warn!(
                "skipping fixture {}: no repo/ directory",
                task_dir.display()
            );
            continue;
        }

        // Validate: repo must have Cargo.toml
        if !repo_dir.join("Cargo.toml").exists() {
            tracing::warn!(
                "skipping fixture {}: repo/ has no Cargo.toml",
                task_dir.display()
            );
            continue;
        }

        let task_content = std::fs::read_to_string(&task_json).map_err(|e| {
            ForgeError::Fixture(format!("cannot read {}: {e}", task_json.display()))
        })?;

        let task: EvalTask = serde_json::from_str(&task_content).map_err(|e| {
            ForgeError::Fixture(format!("cannot parse {}: {e}", task_json.display()))
        })?;

        tasks.push(LoadedTask {
            task,
            fixture_path: repo_dir,
        });
    }

    Ok(EvalSuite {
        name: suite_name,
        tasks,
    })
}
