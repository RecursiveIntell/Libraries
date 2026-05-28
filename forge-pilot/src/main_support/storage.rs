use forge_engine::ForgeStore;
use forge_pilot::{LoopConfig, LoopRunnerResources};
use knowledge_runtime::{RuntimeConfig, Scope};
use semantic_memory::{MemoryConfig, MemoryStore, MockEmbedder};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

pub(super) fn open_resources(
    memory_dir: &Path,
    forge_db: &Path,
    config: &LoopConfig,
) -> Result<LoopRunnerResources, String> {
    let memory_store = open_memory_store(memory_dir)?;
    let forge_store = open_forge_store(forge_db)?;
    LoopRunnerResources::from_memory_store(memory_store, forge_store, config.runtime_config.clone())
        .map_err(|error| error.to_string())
}

fn open_memory_store(base_dir: &Path) -> Result<MemoryStore, String> {
    fs::create_dir_all(base_dir).map_err(|error| {
        format!(
            "memory folder {} could not be created: {}. Next step: point --memory-dir at a writable semantic-memory base directory or repair the parent path.",
            base_dir.display(),
            error
        )
    })?;
    let config = MemoryConfig {
        base_dir: base_dir.to_path_buf(),
        ..Default::default()
    };
    let embedder = Box::new(MockEmbedder::new(config.embedding.dimensions));
    MemoryStore::open_with_embedder(config, embedder).map_err(|error| {
        format!(
            "memory folder {} could not be opened: {}. Next step: point --memory-dir at a valid semantic-memory base directory or repair the existing store.",
            base_dir.display(),
            error
        )
    })
}

fn open_forge_store(path: &Path) -> Result<ForgeStore, String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    ForgeStore::open(path).map_err(|error| {
        format!(
            "forge db {} could not be opened: {}. Next step: repair or replace the sqlite file, or point --forge-db at a valid Forge database.",
            path.display(),
            error
        )
    })
}

pub(super) fn build_loop_config(
    scope: &Scope,
    workspace_path: String,
    memory_dir: &Path,
    forge_db: &Path,
    loop_interval_secs: u64,
    max_iterations: u32,
) -> LoopConfig {
    let mut config = LoopConfig::default_for_scope(scope.clone(), workspace_path);
    config.memory_dir = memory_dir.to_string_lossy().to_string();
    config.forge_db_path = forge_db.to_string_lossy().to_string();
    config.max_iterations = max_iterations;
    config.cooldown_secs = loop_interval_secs.min(60);
    config.runtime_config = RuntimeConfig {
        default_scope: scope.clone(),
        ..config.runtime_config.clone()
    };
    config
}

pub(super) fn normalize_user_path(path: &Path) -> PathBuf {
    let raw = path.to_string_lossy();
    let expanded = if raw == "~" {
        env::var_os("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| path.to_path_buf())
    } else if let Some(rest) = raw.strip_prefix("~/") {
        env::var_os("HOME")
            .map(|home| PathBuf::from(home).join(rest))
            .unwrap_or_else(|| path.to_path_buf())
    } else {
        path.to_path_buf()
    };

    if expanded.is_absolute() {
        expanded
    } else {
        env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(expanded)
    }
}

pub(super) fn managed_storage_root_for_workspace(workspace_path: &Path) -> PathBuf {
    detect_project_root(workspace_path).join(".forge-pilot")
}

pub(super) fn uses_managed_or_legacy_storage_defaults(
    workspace_path: &Path,
    memory_dir: &Path,
    forge_db: &Path,
) -> bool {
    let legacy_memory = workspace_path.join("memory");
    let legacy_forge_db = workspace_path.join("forge.db");
    let managed_root = managed_storage_root_for_workspace(workspace_path);
    let managed_memory = managed_root.join("memory");
    let managed_forge_db = managed_root.join("forge.db");

    (memory_dir == legacy_memory && forge_db == legacy_forge_db)
        || (memory_dir == managed_memory && forge_db == managed_forge_db)
}

pub(super) fn detect_project_root(workspace_path: &Path) -> PathBuf {
    let mut current = if workspace_path.is_file() {
        workspace_path
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| workspace_path.to_path_buf())
    } else {
        workspace_path.to_path_buf()
    };
    current = normalize_user_path(&current);

    let mut candidate = Some(current.as_path());
    while let Some(path) = candidate {
        if looks_like_project_root(path) {
            return path.to_path_buf();
        }
        candidate = path.parent();
    }

    current
}

fn looks_like_project_root(path: &Path) -> bool {
    [
        "Cargo.toml",
        "package.json",
        "pnpm-workspace.yaml",
        "pyproject.toml",
        ".git",
    ]
    .iter()
    .any(|marker| path.join(marker).exists())
}
