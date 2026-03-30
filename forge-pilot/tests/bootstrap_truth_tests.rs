mod common;

use common::{
    base_loop_config, latest_bootstrap_manifest, open_memory_store, point_config_at_dir, tempdir,
    write_source_file,
};
use forge_pilot::{bootstrap_source_workspace, observe_scope};
use knowledge_runtime::Scope;

#[tokio::test]
async fn observation_current_state_comes_from_latest_manifest_only() {
    let dir = tempdir();
    write_source_file(
        dir.path(),
        "src/lib.rs",
        "pub fn first() -> bool { true }\n",
    );

    let memory_store = open_memory_store(dir.path());
    let scope = Scope::new("bootstrap-truth-latest");
    let mut config = base_loop_config(scope.clone());
    point_config_at_dir(&mut config, dir.path());

    bootstrap_source_workspace(&memory_store, &config)
        .await
        .unwrap();
    write_source_file(
        dir.path(),
        "src/lib.rs",
        "pub fn second() -> bool { true }\n",
    );
    write_source_file(
        dir.path(),
        "src/new.rs",
        "pub fn newer() -> bool { true }\n",
    );
    bootstrap_source_workspace(&memory_store, &config)
        .await
        .unwrap();

    let runtime =
        common::resources(memory_store, common::open_forge_store(dir.path()), &config).runtime;
    let memory_store = common::open_memory_store(dir.path());
    let observation = observe_scope(&runtime, &memory_store, &config)
        .await
        .unwrap();
    let manifest = latest_bootstrap_manifest(&memory_store, &scope.namespace).await;

    assert_eq!(
        observation.status.imported_file_count, manifest.file_count,
        "current-state view must follow the latest manifest snapshot, not historical imports",
    );
    assert!(manifest.files.iter().any(|file| file.path == "src/new.rs"));
}

#[tokio::test]
async fn derived_policy_changes_do_not_count_as_source_changes() {
    let dir = tempdir();
    let content = (1..=120)
        .map(|index| format!("fn line_{index}() {{}}\n"))
        .collect::<String>();
    write_source_file(dir.path(), "src/lib.rs", &content);

    let memory_store = open_memory_store(dir.path());
    let scope = Scope::new("bootstrap-derived-delta");
    let mut config = base_loop_config(scope.clone());
    point_config_at_dir(&mut config, dir.path());

    bootstrap_source_workspace(&memory_store, &config)
        .await
        .unwrap();
    let mut manifest = latest_bootstrap_manifest(&memory_store, &scope.namespace).await;
    manifest.files[0].chunk_ids.reverse();
    let delta = forge_pilot::compute_manifest_delta(
        Some(&manifest),
        &latest_bootstrap_manifest(&memory_store, &scope.namespace).await,
    );

    assert_eq!(delta.changed_files.len(), 0);
    assert_eq!(delta.source_unchanged_derived_changed_files.len(), 1);
}
