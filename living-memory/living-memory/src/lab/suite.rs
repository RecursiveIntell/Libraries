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

/// Generates an EvalTask from a kernel syndrome.
///
/// All tasks produced this way carry a prompt prefix marking them as
/// `NonAuthoritativeDerived` — they originate from the kernel's advisory
/// inference graph and require human review before promotion to
/// authoritative evidence.
///
/// Defaults: correctness=0.8, fmt/clippy/tests all required, CEA
/// instrumentation enabled, no test modification allowed, max 2 files
/// changed.
pub fn eval_task_from_syndrome(
    syndrome_signature: &str,
    syndrome_id: &str,
    fixture_path: &std::path::Path,
    belief_micros: u64,
) -> EvalTask {
    let fragility = 1.0 - (belief_micros as f64 / 1_000_000.0);
    EvalTask {
        task_id: format!("kernel-{}", syndrome_id),
        prompt: format!(
            "KERNEL-GENERATED (NonAuthoritativeDerived): syndrome '{}' detected in crate at {}. \
             Belief: {}/1000000. Verify correctness by running fmt, clippy, and test suite \
             on the crate as-is (baseline only — the syndrome is an inference signal, \
             not a prescribed patch).",
            syndrome_signature,
            fixture_path.display(),
            belief_micros
        ),
        constraints: TaskConstraints {
            allow_test_modifications: false,
            max_files_changed: Some(2),
        },
        weights: TaskWeights {
            correctness: 0.8,
            novelty: fragility.min(0.3),
            stability: 0.1,
        },
        expected: TaskExpected {
            require_fmt: true,
            require_clippy: true,
            require_tests: true,
        },
        cea: TaskCea {
            instrument: true,
            risk_threshold_override: None,
        },
    }
}
