mod common;

use common::{
    base_loop_config, import_v3_bundle, open_forge_store, open_memory_store,
    persist_bundle_in_forge, point_config_at_dir, resources, sample_bundle, tempdir,
    write_source_file,
};
use forge_pilot::{
    import_recent_forge_bundles, inspect_observation_paths, observe_scope, ActionFamily,
    HaltReason, ImportRecordDisposition, LoopRunner, ObservationDisposition, PathAvailability,
};
use knowledge_runtime::Scope;

#[tokio::test]
async fn empty_workspace_stays_empty_instead_of_forcing_import() {
    let dir = tempdir();
    let memory_store = open_memory_store(dir.path());
    let forge_store = open_forge_store(dir.path());
    let scope = Scope::new("pilot-empty");
    let mut config = base_loop_config(scope.clone());
    point_config_at_dir(&mut config, dir.path());

    let resources = resources(memory_store, forge_store, &config);
    let observation = observe_scope(&resources.runtime, &resources.memory_store, &config)
        .await
        .unwrap();

    assert_eq!(
        observation.status.disposition,
        ObservationDisposition::EmptyWorkspace
    );
    assert_eq!(
        observation.status.import_record_disposition,
        ImportRecordDisposition::NeverImported
    );
    assert_eq!(observation.source_inventory.supported_file_count, 0);
    assert_eq!(observation.status.supported_file_count, 0);
    assert!(!observation.status.import_records_found);
    assert_eq!(observation.status.namespace_queried, "pilot-empty");
    assert_eq!(
        observation.status.resolved_workspace_path,
        observation.paths.workspace_path
    );
}

#[tokio::test]
async fn source_files_without_imports_surface_import_required() {
    let dir = tempdir();
    write_source_file(
        dir.path(),
        "src/lib.rs",
        "pub fn source_present_but_unimported() -> bool { true }\n",
    );
    let memory_store = open_memory_store(dir.path());
    let forge_store = open_forge_store(dir.path());
    let scope = Scope::new("pilot-import-required");
    let mut config = base_loop_config(scope.clone());
    point_config_at_dir(&mut config, dir.path());

    let resources = resources(memory_store, forge_store, &config);
    let observation = observe_scope(&resources.runtime, &resources.memory_store, &config)
        .await
        .unwrap();

    assert_eq!(
        observation.status.disposition,
        ObservationDisposition::ImportRequired
    );
    assert_eq!(
        observation.status.import_record_disposition,
        ImportRecordDisposition::NeverImported
    );
    assert_eq!(observation.source_inventory.supported_file_count, 1);
    assert_eq!(observation.status.supported_file_count, 1);
    assert!(!observation.status.import_records_found);
    assert!(observation
        .status
        .exact_next_step
        .contains("No completed canonical import or exportable Forge evidence bundle exists yet"));
    assert_eq!(observation.status.available_forge_bundle_count, 0);

    // GOV-001 requires observed governance before the loop can reach the
    // ImportRequired advisory path. This fixture validates that later path,
    // not the already-covered missing-governance block.
    resources
        .memory_store
        .raw_execute(
            "INSERT INTO claim_versions (
                claim_version_id, claim_id, claim_state, projection_family,
                subject_entity_id, predicate, object_anchor,
                scope_namespace, scope_domain, scope_workspace_id, scope_repo_id,
                recorded_at, preferred_open,
                source_envelope_id, source_authority,
                freshness, contradiction_status, content, confidence
            ) VALUES (
                'gov-import-required-claim', 'gov-import-required', 'active', 'governance',
                'governance-entity', 'mechanism_fit_disposition', '\"fit\"',
                'governance', NULL, NULL, NULL,
                datetime('now'), 0,
                'gov-import-required-envelope', 'governance',
                'current', 'none', 'fit', 1.0
            )",
            vec![],
        )
        .await
        .unwrap();

    let mut runner = LoopRunner::new(config, resources);
    let report = runner.run().await.unwrap();
    assert_eq!(report.halt_reason, HaltReason::AdvisoryOnlyFallback);
    assert_eq!(report.iterations_completed, 1);
    assert_eq!(
        report.iterations[0].action_family,
        Some(ActionFamily::AdvisoryOnly)
    );
    assert_eq!(
        report
            .receipt
            .observation_status
            .as_ref()
            .unwrap()
            .supported_file_count,
        1
    );
    assert_eq!(
        report
            .receipt
            .observation_paths
            .as_ref()
            .unwrap()
            .workspace_path,
        observation.paths.workspace_path
    );
}

#[tokio::test]
async fn forge_bundles_without_imports_surface_exact_import_command() {
    let dir = tempdir();
    write_source_file(
        dir.path(),
        "src/lib.rs",
        "pub fn source_present_and_bootstrap_ready() -> bool { true }\n",
    );
    let memory_store = open_memory_store(dir.path());
    let forge_store = open_forge_store(dir.path());
    let scope = Scope::new("pilot-bootstrap-ready");
    let mut config = base_loop_config(scope.clone());
    point_config_at_dir(&mut config, dir.path());

    let bundle = sample_bundle("bootstrap-ready");
    persist_bundle_in_forge(&forge_store, &bundle);

    let resources = resources(memory_store, forge_store, &config);
    let observation = observe_scope(&resources.runtime, &resources.memory_store, &config)
        .await
        .unwrap();

    assert_eq!(
        observation.status.disposition,
        ObservationDisposition::ImportRequired
    );
    assert_eq!(observation.status.available_forge_bundle_count, 1);
    assert!(observation
        .status
        .exact_next_step
        .contains("cargo run -p forge-pilot -- import --namespace 'pilot-bootstrap-ready'"));
    assert!(observation
        .status
        .exact_next_step
        .contains("and then rerun"));
}

#[tokio::test]
async fn imported_scope_remains_ready() {
    let dir = tempdir();
    write_source_file(
        dir.path(),
        "src/lib.rs",
        "pub fn imported_scope_ready() -> bool { true }\n",
    );
    let memory_store = open_memory_store(dir.path());
    let forge_store = open_forge_store(dir.path());
    let scope = Scope::new("pilot-imported");
    let mut config = base_loop_config(scope.clone());
    point_config_at_dir(&mut config, dir.path());

    import_v3_bundle(
        &memory_store,
        &forge_store,
        &scope.namespace,
        &sample_bundle("imported-scope"),
    )
    .await;

    let resources = resources(memory_store, forge_store, &config);
    let observation = observe_scope(&resources.runtime, &resources.memory_store, &config)
        .await
        .unwrap();

    assert_eq!(
        observation.status.disposition,
        ObservationDisposition::Ready
    );
    assert_eq!(
        observation.status.import_record_disposition,
        ImportRecordDisposition::Found
    );
    assert!(observation.status.import_records_found);
    assert!(observation.import_log.is_some());
}

#[tokio::test]
async fn pilot_can_bootstrap_memory_from_recent_forge_bundles() {
    let dir = tempdir();
    write_source_file(
        dir.path(),
        "src/lib.rs",
        "pub fn bootstrap_memory_from_forge() -> bool { true }\n",
    );
    let memory_store = open_memory_store(dir.path());
    let forge_store = open_forge_store(dir.path());
    let scope = Scope::new("pilot-bootstrap-import");
    let mut config = base_loop_config(scope.clone());
    point_config_at_dir(&mut config, dir.path());

    let bundle = sample_bundle("bootstrap-import");
    persist_bundle_in_forge(&forge_store, &bundle);

    let report = import_recent_forge_bundles(&scope.namespace, &forge_store, &memory_store, 16)
        .await
        .unwrap();
    assert_eq!(report.forge_bundle_count, 1);
    assert_eq!(
        report.imported_bundle_ids,
        vec!["bootstrap-import".to_string()]
    );

    let resources = resources(memory_store, forge_store, &config);
    let observation = observe_scope(&resources.runtime, &resources.memory_store, &config)
        .await
        .unwrap();
    assert_eq!(
        observation.status.disposition,
        ObservationDisposition::Ready
    );
    assert!(observation.status.import_records_found);
}

#[tokio::test]
async fn namespace_mismatch_reports_dedicated_status_for_requested_namespace() {
    let dir = tempdir();
    write_source_file(
        dir.path(),
        "src/lib.rs",
        "pub fn namespace_mismatch() -> bool { true }\n",
    );
    let memory_store = open_memory_store(dir.path());
    let forge_store = open_forge_store(dir.path());
    let scope = Scope::new("target-ns");
    let mut config = base_loop_config(scope.clone());
    point_config_at_dir(&mut config, dir.path());

    import_v3_bundle(
        &memory_store,
        &forge_store,
        "other-ns",
        &sample_bundle("namespace-mismatch"),
    )
    .await;

    let resources = resources(memory_store, forge_store, &config);
    let observation = observe_scope(&resources.runtime, &resources.memory_store, &config)
        .await
        .unwrap();

    assert_eq!(
        observation.status.disposition,
        ObservationDisposition::NamespaceMismatch
    );
    assert_eq!(
        observation.status.import_record_disposition,
        ImportRecordDisposition::NamespaceMismatch
    );
    assert!(!observation.status.import_records_found);
    assert_eq!(
        observation.status.other_import_namespaces,
        vec!["other-ns".to_string()]
    );
    assert!(observation
        .status
        .exact_next_step
        .contains("No completed canonical import was found for namespace target-ns"));

    let mut runner = LoopRunner::new(config, resources);
    let report = runner.run().await.unwrap();
    assert_eq!(report.halt_reason, HaltReason::NamespaceMismatch);
}

#[tokio::test]
async fn missing_storage_is_reported_explicitly() {
    let backing_dir = tempdir();
    let configured_dir = tempdir();
    write_source_file(
        configured_dir.path(),
        "src/lib.rs",
        "pub fn missing_storage() -> bool { true }\n",
    );

    let memory_store = open_memory_store(backing_dir.path());
    let forge_store = open_forge_store(backing_dir.path());
    let mut config = base_loop_config(Scope::new("pilot-missing-db"));
    point_config_at_dir(&mut config, configured_dir.path());

    let paths = inspect_observation_paths(&config);
    assert_eq!(paths.memory_dir_state, PathAvailability::Missing);
    assert_eq!(paths.forge_db_state, PathAvailability::Missing);

    let resources = resources(memory_store, forge_store, &config);
    let observation = observe_scope(&resources.runtime, &resources.memory_store, &config)
        .await
        .unwrap();

    assert_eq!(
        observation.status.disposition,
        ObservationDisposition::StorageUnavailable
    );
    assert_eq!(
        observation.status.import_record_disposition,
        ImportRecordDisposition::StorageUnavailable
    );
    assert_eq!(observation.status.supported_file_count, 1);

    let mut runner = LoopRunner::new(config, resources);
    let report = runner.run().await.unwrap();
    assert_eq!(report.halt_reason, HaltReason::StorageUnavailable);
}

#[tokio::test]
async fn corrupt_forge_db_is_reported_explicitly() {
    let backing_dir = tempdir();
    let configured_dir = tempdir();
    write_source_file(
        configured_dir.path(),
        "src/lib.rs",
        "pub fn corrupt_storage() -> bool { true }\n",
    );
    std::fs::create_dir_all(configured_dir.path().join("memory")).unwrap();
    std::fs::write(configured_dir.path().join("forge.db"), b"not-a-sqlite-file").unwrap();

    let memory_store = open_memory_store(backing_dir.path());
    let forge_store = open_forge_store(backing_dir.path());
    let mut config = base_loop_config(Scope::new("pilot-corrupt-db"));
    point_config_at_dir(&mut config, configured_dir.path());

    let paths = inspect_observation_paths(&config);
    assert_eq!(paths.forge_db_state, PathAvailability::Invalid);
    assert_eq!(paths.memory_dir_state, PathAvailability::Present);

    let resources = resources(memory_store, forge_store, &config);
    let observation = observe_scope(&resources.runtime, &resources.memory_store, &config)
        .await
        .unwrap();

    assert_eq!(
        observation.status.disposition,
        ObservationDisposition::StorageCorrupt
    );
    assert_eq!(
        observation.status.import_record_disposition,
        ImportRecordDisposition::StorageCorrupt
    );

    let mut runner = LoopRunner::new(config, resources);
    let report = runner.run().await.unwrap();
    assert_eq!(report.halt_reason, HaltReason::StorageCorrupt);
}
