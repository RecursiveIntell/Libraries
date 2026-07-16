mod common;

use common::{
    base_loop_config, import_thin_v3_batch, import_v2_bundle_without_kernel_payload,
    open_forge_store, open_memory_store, point_config_at_dir, resources, sample_bundle, tempdir,
    write_source_file,
};
use forge_pilot::{extract_targets, observe_scope, HaltReason, LoopRunner, TargetKind};
use knowledge_runtime::Scope;

#[tokio::test]
async fn observation_degrades_honestly_when_kernel_payload_is_missing() {
    let dir = tempdir();
    let memory_store = open_memory_store(dir.path());
    let forge_store = open_forge_store(dir.path());
    let scope = Scope::new("pilot-degrade-missing");
    let mut config = base_loop_config(scope.clone());
    point_config_at_dir(&mut config, dir.path());
    write_source_file(
        dir.path(),
        "src/lib.rs",
        "pub fn missing_kernel_fixture() -> bool { true }\n",
    );

    import_v2_bundle_without_kernel_payload(
        &memory_store,
        &scope.namespace,
        &sample_bundle("missing-kernel"),
    )
    .await;

    let resources = resources(memory_store, forge_store, &config);
    let observation = observe_scope(&resources.runtime, &resources.memory_store, &config)
        .await
        .unwrap();

    assert!(observation.missing_kernel_payload());
    assert!(observation.batch.is_none());
    assert!(observation.compiled.is_none());
}

#[tokio::test]
async fn thin_export_yields_explicit_target_instead_of_invented_structure() {
    let dir = tempdir();
    let memory_store = open_memory_store(dir.path());
    let forge_store = open_forge_store(dir.path());
    let scope = Scope::new("pilot-degrade-thin");
    let mut config = base_loop_config(scope.clone());
    point_config_at_dir(&mut config, dir.path());
    write_source_file(
        dir.path(),
        "src/lib.rs",
        "pub fn thin_export_fixture() -> bool { true }\n",
    );

    import_thin_v3_batch(
        &memory_store,
        &forge_store,
        &scope.namespace,
        &sample_bundle("thin-export"),
    )
    .await;

    let resources = resources(memory_store, forge_store, &config);
    let observation = observe_scope(&resources.runtime, &resources.memory_store, &config)
        .await
        .unwrap();
    let targets = extract_targets(&observation, &config);

    assert!(observation.thin_export_active());
    assert!(targets
        .iter()
        .any(|target| matches!(target, TargetKind::ThinExport { .. })));
}

#[tokio::test]
async fn loop_runner_bounds_repeated_degraded_thin_export_cycles() {
    let dir = tempdir();
    let memory_store = open_memory_store(dir.path());
    let forge_store = open_forge_store(dir.path());
    let scope = Scope::new("pilot-degrade-loop");
    let mut config = base_loop_config(scope.clone());
    point_config_at_dir(&mut config, dir.path());
    write_source_file(
        dir.path(),
        "src/lib.rs",
        "pub fn thin_loop_fixture() -> bool { true }\n",
    );
    config.max_iterations = 3;
    config.max_retries_per_target = 2;
    let expected_iterations = config.max_iterations;

    import_thin_v3_batch(
        &memory_store,
        &forge_store,
        &scope.namespace,
        &sample_bundle("thin-loop"),
    )
    .await;

    // GOV-001 made governance absence fail closed. This fixture exercises
    // repeated thin-export degradation, so provide an observed, non-blocking
    // governance claim rather than implicitly relying on a missing state.
    memory_store
        .raw_execute(
            "INSERT INTO claim_versions (
                claim_version_id, claim_id, claim_state, projection_family,
                subject_entity_id, predicate, object_anchor,
                scope_namespace, scope_domain, scope_workspace_id, scope_repo_id,
                recorded_at, preferred_open,
                source_envelope_id, source_authority,
                freshness, contradiction_status, content, confidence
            ) VALUES (
                'gov-thin-loop-claim', 'gov-thin-loop', 'active', 'governance',
                'governance-entity', 'mechanism_fit_disposition', '\"fit\"',
                'governance', NULL, NULL, NULL,
                datetime('now'), 0,
                'gov-thin-loop-envelope', 'governance',
                'current', 'none', 'fit', 1.0
            )",
            vec![],
        )
        .await
        .unwrap();

    let resources = resources(memory_store, forge_store, &config);
    let mut runner = LoopRunner::new(config, resources);
    let report = runner.run().await.unwrap();

    assert_eq!(report.iterations_completed, expected_iterations);
    assert_eq!(report.receipt.iterations_completed, expected_iterations);
    assert_eq!(report.degraded_iterations, expected_iterations);
    assert_eq!(report.receipt.degraded_iterations, expected_iterations);
    assert_eq!(report.halt_reason, HaltReason::MaxIterationsReached);
    assert_eq!(
        report.targets_investigated.len(),
        expected_iterations as usize
    );
    assert!(report.iterations.iter().all(|iteration| iteration.degraded));
}
