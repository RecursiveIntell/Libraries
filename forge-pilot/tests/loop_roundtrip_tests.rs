mod common;

use common::{
    base_loop_config, import_v3_bundle, open_forge_store, open_memory_store, point_config_at_dir,
    resources, sample_bundle, tempdir, write_patch_fixture, write_source_file,
};
use forge_pilot::{
    canonical_roundtrip, observe_scope, score_targets, ActionFamily, LoopRunner, PilotError,
    PilotHistory,
};
use knowledge_runtime::Scope;

#[tokio::test]
async fn loop_runner_executes_oracle_and_patch_families_with_real_roundtrips() {
    let oracle_dir = tempdir();
    let oracle_memory = open_memory_store(oracle_dir.path());
    let oracle_forge = open_forge_store(oracle_dir.path());
    let oracle_scope = Scope::new("pilot-loop-oracle");
    let mut oracle_config = base_loop_config(oracle_scope.clone());
    point_config_at_dir(&mut oracle_config, oracle_dir.path());
    write_source_file(
        oracle_dir.path(),
        "src/lib.rs",
        "pub fn oracle_loop_fixture() -> bool { true }\n",
    );

    import_v3_bundle(
        &oracle_memory,
        &oracle_forge,
        &oracle_scope.namespace,
        &sample_bundle("loop-oracle"),
    )
    .await;

    let oracle_resources = resources(oracle_memory, oracle_forge, &oracle_config);
    let mut oracle_runner = LoopRunner::new(oracle_config, oracle_resources);
    let oracle_report = oracle_runner.run().await.unwrap();
    assert!(oracle_report.imports_completed >= 1);
    assert_eq!(
        oracle_report.iterations[0].action_family,
        Some(ActionFamily::Oracle)
    );

    let patch_dir = tempdir();
    let patch_memory = open_memory_store(patch_dir.path());
    let patch_forge = open_forge_store(patch_dir.path());
    let patch_scope = Scope::new("pilot-loop-patch");
    let mut patch_config = base_loop_config(patch_scope.clone());
    point_config_at_dir(&mut patch_config, patch_dir.path());
    write_source_file(
        patch_dir.path(),
        "src/lib.rs",
        "pub fn patch_loop_fixture() -> bool { true }\n",
    );

    import_v3_bundle(
        &patch_memory,
        &patch_forge,
        &patch_scope.namespace,
        &sample_bundle("loop-patch"),
    )
    .await;

    let preview_resources = resources(patch_memory.clone(), patch_forge, &patch_config);
    let preview_observation = observe_scope(
        &preview_resources.runtime,
        &preview_resources.memory_store,
        &patch_config,
    )
    .await
    .unwrap();
    let preview_candidates = score_targets(
        &preview_observation,
        &PilotHistory::default(),
        &patch_config,
    );
    let seeded_target_key = preview_candidates.first().unwrap().stable_key.clone();
    let (fixture_path, patch) = write_patch_fixture(patch_dir.path());
    patch_config
        .patch_plan_seeds
        .push(forge_pilot::PatchPlanSeed {
            target_key: seeded_target_key,
            fixture_path: fixture_path.to_string_lossy().to_string(),
            patch,
            experiment_config: forge_engine::ExperimentConfig::default(),
            description: "patch fixture".into(),
        });

    let patch_forge = open_forge_store(&patch_dir.path().join("patch-loop-forge"));
    import_v3_bundle(
        &patch_memory,
        &patch_forge,
        &patch_scope.namespace,
        &sample_bundle("loop-patch-2"),
    )
    .await;
    let patch_resources = resources(patch_memory, patch_forge, &patch_config);
    let mut patch_runner = LoopRunner::new(patch_config, patch_resources);
    let patch_report = patch_runner.run().await.unwrap();
    assert!(patch_report.imports_completed >= 1);
    assert_eq!(
        patch_report.iterations[0].action_family,
        Some(ActionFamily::PairedPatch)
    );
}

#[tokio::test]
async fn canonical_roundtrip_records_durable_failure_receipt_when_import_breaks() {
    let dir = tempdir();
    let memory_store = open_memory_store(dir.path());
    let forge_store = open_forge_store(dir.path());
    let scope = Scope::new("pilot-roundtrip-failure");

    let conn = rusqlite::Connection::open(dir.path().join("memory.db")).unwrap();
    conn.execute_batch("DROP TABLE claim_versions;").unwrap();

    let err = canonical_roundtrip(
        &sample_bundle("roundtrip-failure"),
        &scope.namespace,
        &forge_store,
        &memory_store,
    )
    .await
    .unwrap_err();
    assert!(matches!(err, PilotError::Memory(_)));

    let failures = memory_store
        .query_projection_import_failures(Some(&scope.namespace), 8)
        .await
        .unwrap();
    assert_eq!(failures.len(), 1);
    assert_eq!(failures[0].scope_namespace, scope.namespace);
    assert_eq!(failures[0].source_authority, "forge");
    assert!(failures[0]
        .error_message
        .contains("no such table: claim_versions"));
}
