mod common;

use common::{
    base_loop_config, latest_bootstrap_manifest, open_forge_store, open_memory_store,
    point_config_at_dir, resources, tempdir, write_source_file,
};
use forge_pilot::{
    bootstrap_source_workspace, observe_scope, score_targets, HaltReason, LoopRunner,
    ObservationDisposition, PilotHistory,
};
use knowledge_runtime::Scope;
use std::fs;

#[tokio::test]
async fn bootstrap_source_reports_empty_scan_cleanly() {
    let dir = tempdir();
    let memory_store = open_memory_store(dir.path());
    let scope = Scope::new("bootstrap-empty");
    let mut config = base_loop_config(scope.clone());
    point_config_at_dir(&mut config, dir.path());

    let report = bootstrap_source_workspace(&memory_store, &config)
        .await
        .unwrap();
    assert_eq!(report.scanned_file_count, 0);
    assert_eq!(report.import_status, None);
    assert!(!report.was_duplicate);
}

#[tokio::test]
async fn bootstrap_source_imports_workspace_into_canonical_observation_state() {
    let dir = tempdir();
    write_source_file(
        dir.path(),
        "src/lib.rs",
        "pub fn bootstrap_source_ready() -> bool { true }\n",
    );
    write_source_file(dir.path(), "README.md", "# Bootstrap source\n");

    let memory_store = open_memory_store(dir.path());
    let forge_store = open_forge_store(dir.path());
    let scope = Scope::new("bootstrap-source");
    let mut config = base_loop_config(scope.clone());
    point_config_at_dir(&mut config, dir.path());

    let report = bootstrap_source_workspace(&memory_store, &config)
        .await
        .unwrap();
    assert_eq!(report.scanned_file_count, 2);
    assert_eq!(report.import_status.as_deref(), Some("complete"));

    let resources = resources(memory_store, forge_store, &config);
    let observation = observe_scope(&resources.runtime, &resources.memory_store, &config)
        .await
        .unwrap();
    assert_eq!(
        observation.status.disposition,
        ObservationDisposition::Ready
    );
    assert!(observation.status.import_records_found);
    assert_eq!(observation.status.imported_file_count, 2);
    assert!(observation.status.imported_chunk_count >= 2);
    assert_eq!(
        observation.status.bootstrap_richness,
        forge_pilot::BootstrapRichness::Symbolized
    );

    let manifest = latest_bootstrap_manifest(&resources.memory_store, &scope.namespace).await;
    assert_eq!(manifest.file_count, 2);
    assert!(manifest.chunk_count >= 2);

    let candidates = score_targets(&observation, &PilotHistory::default(), &config);
    assert!(!candidates.is_empty());

    let mut runner = LoopRunner::new(config, resources);
    let loop_report = runner.run().await.unwrap();
    assert_ne!(loop_report.halt_reason, HaltReason::ImportRequired);
    assert!(loop_report.iterations_completed > 0);
}

#[tokio::test]
async fn bootstrap_source_is_idempotent_for_unchanged_workspace() {
    let dir = tempdir();
    write_source_file(
        dir.path(),
        "src/lib.rs",
        "pub fn bootstrap_source_duplicate() -> bool { true }\n",
    );

    let memory_store = open_memory_store(dir.path());
    let scope = Scope::new("bootstrap-duplicate");
    let mut config = base_loop_config(scope.clone());
    point_config_at_dir(&mut config, dir.path());

    let first = bootstrap_source_workspace(&memory_store, &config)
        .await
        .unwrap();
    let second = bootstrap_source_workspace(&memory_store, &config)
        .await
        .unwrap();

    assert_eq!(first.import_status.as_deref(), Some("complete"));
    assert_eq!(second.import_status.as_deref(), Some("already_imported"));
    assert!(second.was_duplicate);
}

#[tokio::test]
async fn bootstrap_source_reports_invalid_utf8_skips() {
    let dir = tempdir();
    fs::create_dir_all(dir.path().join("src")).unwrap();
    fs::write(dir.path().join("src/bad.rs"), vec![0xff, 0xfe, 0xfd]).unwrap();

    let memory_store = open_memory_store(dir.path());
    let scope = Scope::new("bootstrap-invalid-utf8");
    let mut config = base_loop_config(scope.clone());
    point_config_at_dir(&mut config, dir.path());

    let report = bootstrap_source_workspace(&memory_store, &config)
        .await
        .unwrap();
    assert_eq!(report.scanned_file_count, 0);
    assert_eq!(report.skipped_file_count, 1);
    assert!(report.skipped_files[0].reason.contains("UTF-8"));
}
