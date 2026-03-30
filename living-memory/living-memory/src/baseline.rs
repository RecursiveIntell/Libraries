//! Baseline provenance and workspace policy for experiment comparability.

use serde::{Deserialize, Serialize};

/// How a baseline was obtained.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BaselineSourceKind {
    /// Clean checkout of a specific commit.
    GitCommit,
    /// Working tree snapshot (may be dirty).
    WorkingTree,
    /// Cached result from a previous run.
    CachedRun,
    /// Explicitly provided by the caller.
    Explicit,
}

/// Full provenance record for a baseline execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaselineDescriptor {
    pub source_kind: BaselineSourceKind,
    /// Git commit SHA, if available.
    pub commit_sha: Option<String>,
    /// Whether the working tree was dirty when the baseline was taken.
    pub dirty: bool,
    /// Number of untracked files present.
    pub untracked_count: u32,
    /// blake3 hash of Cargo.lock, if present.
    pub lockfile_hash: Option<String>,
    /// `rustc --version` output.
    pub rustc_version: String,
    /// `cargo --version` output.
    pub cargo_version: String,
    /// Target triple (e.g. x86_64-unknown-linux-gnu).
    pub target_triple: String,
    /// blake3 hash of sorted, filtered environment variables.
    pub env_fingerprint: String,
    /// Submodule state: list of (path, commit_sha) pairs.
    pub submodule_state: Vec<(String, String)>,
}

impl BaselineDescriptor {
    /// Compute a deterministic fingerprint for comparability checks.
    pub fn fingerprint(&self) -> String {
        let content = serde_json::to_string(self).unwrap_or_default();
        blake3::hash(content.as_bytes()).to_hex().to_string()
    }
}

/// Policy for workspace isolation during experiments.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspacePolicy {
    /// Maximum bytes per workspace.
    #[serde(default = "default_max_workspace_bytes")]
    pub max_workspace_bytes: u64,
    /// Maximum number of retained workspaces.
    #[serde(default = "default_max_retained")]
    pub max_retained_workspaces: u32,
    /// Whether to retain workspaces from failed runs.
    #[serde(default)]
    pub retain_failed_workspaces: bool,
    /// TTL in seconds for workspace cleanup.
    #[serde(default = "default_cleanup_ttl")]
    pub workspace_cleanup_ttl_secs: u64,
}

fn default_max_workspace_bytes() -> u64 {
    2_000_000_000 // 2 GB
}
fn default_max_retained() -> u32 {
    8
}
fn default_cleanup_ttl() -> u64 {
    3600 // 1 hour
}

impl Default for WorkspacePolicy {
    fn default() -> Self {
        Self {
            max_workspace_bytes: default_max_workspace_bytes(),
            max_retained_workspaces: default_max_retained(),
            retain_failed_workspaces: false,
            workspace_cleanup_ttl_secs: default_cleanup_ttl(),
        }
    }
}

/// Policy for experiment comparability.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparabilityPolicy {
    /// Whether baseline fingerprints must match between trials.
    #[serde(default = "default_true")]
    pub require_fingerprint_match: bool,
    /// Whether timing measurements are admissible as evidence.
    #[serde(default)]
    pub timing_admissible: bool,
    /// Minimum number of trials for statistical claims.
    #[serde(default = "default_min_trials")]
    pub min_trials: u32,
}

fn default_true() -> bool {
    true
}
fn default_min_trials() -> u32 {
    3
}

impl Default for ComparabilityPolicy {
    fn default() -> Self {
        Self {
            require_fingerprint_match: true,
            timing_admissible: false,
            min_trials: default_min_trials(),
        }
    }
}

/// Capture baseline provenance from the current environment.
///
/// This is a best-effort snapshot: git info may be unavailable.
pub async fn capture_baseline_provenance(
    workspace_dir: &std::path::Path,
) -> crate::error::ForgeResult<BaselineDescriptor> {
    use tokio::process::Command;

    let commit_sha = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(workspace_dir)
        .output()
        .await
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string());

    let dirty = Command::new("git")
        .args(["diff", "--quiet", "HEAD"])
        .current_dir(workspace_dir)
        .output()
        .await
        .map(|o| !o.status.success())
        .unwrap_or(true);

    let untracked_count = Command::new("git")
        .args(["ls-files", "--others", "--exclude-standard"])
        .current_dir(workspace_dir)
        .output()
        .await
        .ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).lines().count() as u32)
        .unwrap_or(0);

    let lockfile_hash = {
        let lockfile = workspace_dir.join("Cargo.lock");
        if lockfile.exists() {
            let content = tokio::fs::read(&lockfile).await.unwrap_or_default();
            Some(blake3::hash(&content).to_hex().to_string())
        } else {
            None
        }
    };

    let rustc_version = Command::new("rustc")
        .arg("--version")
        .output()
        .await
        .ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    let cargo_version = Command::new("cargo")
        .arg("--version")
        .output()
        .await
        .ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    let target_triple = Command::new("rustc")
        .args(["-vV"])
        .output()
        .await
        .ok()
        .and_then(|o| {
            let text = String::from_utf8_lossy(&o.stdout);
            text.lines()
                .find(|l| l.starts_with("host:"))
                .map(|l| l.trim_start_matches("host:").trim().to_string())
        })
        .unwrap_or_else(|| "unknown".to_string());

    // Compute env fingerprint from sorted, filtered env vars
    let env_fingerprint = {
        let mut relevant: Vec<String> = std::env::vars()
            .filter(|(k, _)| crate::exec::is_env_allowed(k))
            .map(|(k, v)| format!("{k}={v}"))
            .collect();
        relevant.sort();
        blake3::hash(relevant.join("\n").as_bytes())
            .to_hex()
            .to_string()
    };

    let submodule_state = Command::new("git")
        .args(["submodule", "status"])
        .current_dir(workspace_dir)
        .output()
        .await
        .ok()
        .map(|o| {
            String::from_utf8_lossy(&o.stdout)
                .lines()
                .filter_map(|line| {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 2 {
                        Some((
                            parts[1].to_string(),
                            parts[0].trim_start_matches(['+', '-', 'U']).to_string(),
                        ))
                    } else {
                        None
                    }
                })
                .collect()
        })
        .unwrap_or_default();

    let source_kind = if commit_sha.is_some() && !dirty {
        BaselineSourceKind::GitCommit
    } else {
        BaselineSourceKind::WorkingTree
    };

    Ok(BaselineDescriptor {
        source_kind,
        commit_sha,
        dirty,
        untracked_count,
        lockfile_hash,
        rustc_version,
        cargo_version,
        target_triple,
        env_fingerprint,
        submodule_state,
    })
}
