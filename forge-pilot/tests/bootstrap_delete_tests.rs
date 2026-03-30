mod common;

use common::{
    base_loop_config, latest_bootstrap_manifest, latest_import_batch, open_memory_store,
    point_config_at_dir, tempdir, write_source_file,
};
use forge_pilot::bootstrap_source_workspace;
use knowledge_runtime::Scope;
use std::fs;

#[tokio::test]
async fn bootstrap_source_deletions_are_manifest_driven_and_non_destructive() {
    let dir = tempdir();
    write_source_file(
        dir.path(),
        "src/lib.rs",
        "pub fn alive() -> bool { true }\n",
    );
    write_source_file(
        dir.path(),
        "src/dead.rs",
        "pub fn dead() -> bool { true }\n",
    );

    let memory_store = open_memory_store(dir.path());
    let scope = Scope::new("bootstrap-delete");
    let mut config = base_loop_config(scope.clone());
    point_config_at_dir(&mut config, dir.path());

    bootstrap_source_workspace(&memory_store, &config)
        .await
        .unwrap();
    fs::remove_file(dir.path().join("src/dead.rs")).unwrap();

    let report = bootstrap_source_workspace(&memory_store, &config)
        .await
        .unwrap();
    assert_eq!(report.source_delta.deleted_file_count, 1);

    let manifest = latest_bootstrap_manifest(&memory_store, &scope.namespace).await;
    assert!(manifest.files.iter().any(|file| file.path == "src/lib.rs"));
    assert!(!manifest.files.iter().any(|file| file.path == "src/dead.rs"));

    let latest = latest_import_batch(&memory_store, &scope.namespace).await;
    let batch = latest.rebuildable_kernel_batch_v3().unwrap().unwrap();
    assert!(batch.records.iter().any(|record| match &record.record {
        forge_memory_bridge::ImportProjectionRecord::ClaimVersion(claim) => {
            claim.predicate == "describes_workspace_source_deletion"
        }
        _ => false,
    }));
}
