mod common;

use common::{
    base_loop_config, latest_import_batch, open_memory_store, point_config_at_dir, tempdir,
    write_source_file,
};
use forge_pilot::bootstrap_source_workspace;
use knowledge_runtime::Scope;

#[tokio::test]
async fn bootstrap_source_delta_reimports_only_changed_local_subtree() {
    let dir = tempdir();
    write_source_file(
        dir.path(),
        "src/lib.rs",
        "pub fn alpha() -> bool { true }\n",
    );
    write_source_file(
        dir.path(),
        "src/other.rs",
        "pub fn beta() -> bool { true }\n",
    );

    let memory_store = open_memory_store(dir.path());
    let scope = Scope::new("bootstrap-delta");
    let mut config = base_loop_config(scope.clone());
    point_config_at_dir(&mut config, dir.path());

    bootstrap_source_workspace(&memory_store, &config)
        .await
        .unwrap();
    write_source_file(
        dir.path(),
        "src/lib.rs",
        "pub fn alpha() -> bool { false }\n",
    );

    let report = bootstrap_source_workspace(&memory_store, &config)
        .await
        .unwrap();
    assert_eq!(report.source_delta.changed_file_count, 1);
    assert_eq!(report.source_delta.unchanged_file_count, 1);
    assert_eq!(report.source_delta.new_file_count, 0);
    assert_eq!(report.source_delta.deleted_file_count, 0);

    let latest = latest_import_batch(&memory_store, &scope.namespace).await;
    let batch = latest.rebuildable_kernel_batch_v3().unwrap().unwrap();
    let changed_file_records = batch
        .records
        .iter()
        .filter_map(|record| match &record.record {
            forge_memory_bridge::ImportProjectionRecord::ClaimVersion(claim) => claim
                .metadata
                .as_ref()
                .and_then(|meta| meta.get("workspace_source_v2"))
                .and_then(|root| root.get("path"))
                .and_then(|value| value.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert!(changed_file_records
        .iter()
        .all(|path| *path == "src/lib.rs"));
}
