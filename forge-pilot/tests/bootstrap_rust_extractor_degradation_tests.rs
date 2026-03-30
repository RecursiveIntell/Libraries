mod common;

use common::{
    base_loop_config, latest_bootstrap_manifest, open_memory_store, point_config_at_dir, tempdir,
    write_source_file,
};
use forge_pilot::bootstrap_source_workspace;
use knowledge_runtime::Scope;

#[tokio::test]
async fn bootstrap_source_marks_rust_line_scanner_as_degraded_for_unsupported_surface() {
    let dir = tempdir();
    write_source_file(
        dir.path(),
        "src/lib.rs",
        r#"
#[cfg(feature = "demo")]
pub fn gated_symbol<T>
(
    value: T,
) -> T {
    value
}

#[derive(Clone)]
pub struct DerivedRecord<T> {
    value: T,
}
"#,
    );

    let memory_store = open_memory_store(dir.path());
    let scope = Scope::new("bootstrap-rust-degraded");
    let mut config = base_loop_config(scope.clone());
    point_config_at_dir(&mut config, dir.path());

    let report = bootstrap_source_workspace(&memory_store, &config)
        .await
        .unwrap();
    assert_eq!(report.degraded_symbol_file_count, 1);

    let manifest = latest_bootstrap_manifest(&memory_store, &scope.namespace).await;
    let file = manifest
        .files
        .iter()
        .find(|file| file.path == "src/lib.rs")
        .unwrap();
    assert_eq!(file.symbol_extraction_status, "degraded");
    let degradation = file.symbol_extraction_degradation.as_deref().unwrap();
    assert!(degradation.contains("cfg_gated_item"));
    assert!(degradation.contains("attribute_annotated_item"));
    assert!(degradation.contains("generic_header"));
    assert!(degradation.contains("multiline_signature"));
    assert_eq!(file.symbol_capability.extractor, "rust_line_scanner_v1");
}
