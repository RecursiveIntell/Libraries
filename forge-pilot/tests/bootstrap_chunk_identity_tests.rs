mod common;

use common::{
    base_loop_config, latest_bootstrap_manifest, open_memory_store, point_config_at_dir, tempdir,
    write_source_file,
};
use forge_pilot::bootstrap_source_workspace;
use knowledge_runtime::Scope;

#[tokio::test]
async fn local_edit_preserves_unaffected_chunk_ids_when_practical() {
    let dir = tempdir();
    let content = (1..=140)
        .map(|index| format!("fn line_{index}() {{}}\n"))
        .collect::<String>();
    write_source_file(dir.path(), "src/lib.rs", &content);

    let memory_store = open_memory_store(dir.path());
    let scope = Scope::new("bootstrap-chunk-stability");
    let mut config = base_loop_config(scope.clone());
    point_config_at_dir(&mut config, dir.path());

    bootstrap_source_workspace(&memory_store, &config)
        .await
        .unwrap();
    let first = latest_bootstrap_manifest(&memory_store, &scope.namespace).await;
    let original = first
        .files
        .iter()
        .find(|file| file.path == "src/lib.rs")
        .unwrap()
        .chunk_ids
        .clone();

    let edited = content.replacen(
        "fn line_40() {}\n",
        "fn line_40() { let changed = true; }\n",
        1,
    );
    write_source_file(dir.path(), "src/lib.rs", &edited);
    bootstrap_source_workspace(&memory_store, &config)
        .await
        .unwrap();
    let second = latest_bootstrap_manifest(&memory_store, &scope.namespace).await;
    let updated = second
        .files
        .iter()
        .find(|file| file.path == "src/lib.rs")
        .unwrap()
        .chunk_ids
        .clone();

    let overlap = original
        .iter()
        .filter(|chunk_id| updated.contains(chunk_id))
        .count();
    assert!(
        overlap >= 1,
        "expected at least one unchanged chunk id across a local edit"
    );
}
