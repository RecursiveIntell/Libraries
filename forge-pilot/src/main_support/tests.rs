use super::{
    detect_project_root, managed_storage_root_for_workspace, normalize_user_path, AppState,
};
use std::fs;
use std::path::Path;
use tempfile::TempDir;

#[test]
fn normalize_user_path_expands_tilde() {
    let home = std::env::var("HOME").unwrap();
    let path = normalize_user_path(Path::new("~/forge-pilot-path-test"));
    assert_eq!(path, Path::new(&home).join("forge-pilot-path-test"));
}

#[test]
fn normalize_user_path_makes_relative_paths_absolute() {
    let cwd = std::env::current_dir().unwrap();
    let path = normalize_user_path(Path::new("forge-pilot-relative-test"));
    assert_eq!(path, cwd.join("forge-pilot-relative-test"));
}

#[test]
fn detect_project_root_walks_up_from_nested_source_folder() {
    let dir = TempDir::new().unwrap();
    fs::write(
        dir.path().join("Cargo.toml"),
        "[package]\nname='x'\nversion='0.1.0'\n",
    )
    .unwrap();
    let nested = dir.path().join("src/stores");
    fs::create_dir_all(&nested).unwrap();

    assert_eq!(detect_project_root(&nested), dir.path());
}

#[test]
fn managed_storage_root_prefers_project_root_over_source_subdir() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("package.json"), "{ \"name\": \"x\" }").unwrap();
    let nested = dir.path().join("src/stores");
    fs::create_dir_all(&nested).unwrap();

    assert_eq!(
        managed_storage_root_for_workspace(&nested),
        dir.path().join(".forge-pilot")
    );
}

#[test]
fn prepare_runtime_layout_moves_managed_defaults_out_of_source_tree() {
    let dir = TempDir::new().unwrap();
    fs::write(
        dir.path().join("Cargo.toml"),
        "[package]\nname='x'\nversion='0.1.0'\n",
    )
    .unwrap();
    let nested = dir.path().join("src/stores");
    fs::create_dir_all(&nested).unwrap();

    let mut state = AppState {
        workspace_path: nested.clone(),
        namespace: "test".into(),
        memory_dir: nested.join("memory"),
        forge_db: nested.join("forge.db"),
        loop_interval_secs: 5,
        max_iterations: 4,
        provider: None,
    };

    state.prepare_runtime_layout().unwrap();

    assert_eq!(state.memory_dir, dir.path().join(".forge-pilot/memory"));
    assert_eq!(state.forge_db, dir.path().join(".forge-pilot/forge.db"));
    assert!(state.memory_dir.is_dir());
    assert!(state.forge_db.parent().expect("forge db parent").is_dir());
}
