mod common;

use common::{
    base_loop_config, latest_import_batch, open_memory_store, point_config_at_dir, tempdir,
    write_source_file,
};
use forge_pilot::bootstrap_source_workspace;
use knowledge_runtime::Scope;

#[tokio::test]
async fn bootstrap_source_extracts_symbols_for_rust_typescript_python_and_generic() {
    let dir = tempdir();
    write_source_file(
        dir.path(),
        "src/lib.rs",
        "pub fn rust_ready() -> bool { true }\n",
    );
    write_source_file(
        dir.path(),
        "web/app.ts",
        "export function tsReady(): boolean { return true; }\n",
    );
    write_source_file(
        dir.path(),
        "tools/job.py",
        "def py_ready():\n    return True\n",
    );
    write_source_file(dir.path(), "docs/README.md", "# Heading\nbody\n");

    let memory_store = open_memory_store(dir.path());
    let scope = Scope::new("bootstrap-symbols");
    let mut config = base_loop_config(scope.clone());
    point_config_at_dir(&mut config, dir.path());

    let report = bootstrap_source_workspace(&memory_store, &config)
        .await
        .unwrap();
    assert_eq!(
        report.richness,
        forge_pilot::BootstrapSourceRichness::Symbolized
    );

    let latest = latest_import_batch(&memory_store, &scope.namespace).await;
    let batch = latest.rebuildable_kernel_batch_v3().unwrap().unwrap();
    let symbols = batch
        .records
        .iter()
        .filter_map(|record| match &record.record {
            forge_memory_bridge::ImportProjectionRecord::ClaimVersion(claim) => claim
                .metadata
                .as_ref()
                .and_then(|meta| meta.get("workspace_source_v2"))
                .filter(|root| {
                    root.get("record_kind").and_then(|value| value.as_str()) == Some("symbol")
                })
                .cloned(),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert!(symbols.iter().any(|symbol| symbol
        .get("symbol_name")
        .and_then(|value| value.as_str())
        == Some("rust_ready")));
    assert!(symbols
        .iter()
        .any(
            |symbol| symbol.get("symbol_name").and_then(|value| value.as_str()) == Some("tsReady")
        ));
    assert!(symbols
        .iter()
        .any(
            |symbol| symbol.get("symbol_name").and_then(|value| value.as_str()) == Some("py_ready")
        ));
    assert!(symbols
        .iter()
        .any(
            |symbol| symbol.get("symbol_kind").and_then(|value| value.as_str()) == Some("section")
        ));
}
