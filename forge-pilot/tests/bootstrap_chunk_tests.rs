mod common;

use common::{
    base_loop_config, latest_bootstrap_manifest, open_memory_store, point_config_at_dir, tempdir,
    write_source_file,
};
use forge_pilot::bootstrap_source_workspace;
use knowledge_runtime::Scope;

#[tokio::test]
async fn bootstrap_source_creates_deterministic_chunk_records() {
    let dir = tempdir();
    let content = (1..=120)
        .map(|index| format!("fn line_{index}() {{}}\n"))
        .collect::<String>();
    write_source_file(dir.path(), "src/lib.rs", &content);

    let memory_store = open_memory_store(dir.path());
    let scope = Scope::new("bootstrap-chunks");
    let mut config = base_loop_config(scope.clone());
    point_config_at_dir(&mut config, dir.path());

    let report = bootstrap_source_workspace(&memory_store, &config)
        .await
        .unwrap();
    assert_eq!(report.current_manifest_file_count, 1);
    assert!(report.current_manifest_chunk_count >= 2);
    assert_eq!(
        report.richness,
        forge_pilot::BootstrapSourceRichness::Symbolized
    );

    let manifest = latest_bootstrap_manifest(&memory_store, &scope.namespace).await;
    let file = manifest
        .files
        .iter()
        .find(|file| file.path == "src/lib.rs")
        .unwrap();
    assert!(file.chunk_count >= 2);
}
