mod common;

use common::{
    base_loop_config, import_v3_bundle, open_forge_store, open_memory_store, resources,
    sample_bundle, tempdir,
};
use forge_pilot::observe_scope;
use knowledge_runtime::Scope;

#[tokio::test]
async fn reconstructs_observation_from_public_runtime_and_import_surfaces() {
    let dir = tempdir();
    let memory_store = open_memory_store(dir.path());
    let forge_store = open_forge_store(dir.path());
    let scope = Scope::new("pilot-observe");
    let config = base_loop_config(scope.clone());

    import_v3_bundle(
        &memory_store,
        &forge_store,
        &scope.namespace,
        &sample_bundle("obs-1"),
    )
    .await;

    let resources = resources(memory_store, forge_store, &config);
    let observation = observe_scope(&resources.runtime, &resources.memory_store, &config)
        .await
        .unwrap();

    assert!(observation.advisory.is_some());
    assert!(observation.explanation.is_some());
    assert!(observation.risk_gate.is_some());
    assert!(observation.import_log.is_some());
    assert!(observation.batch.is_some());
    assert!(observation.compiled.is_some());
    assert!(observation.scheduled.is_some());
    assert!(observation.oracle.is_some());
    assert!(!observation.claim_versions.is_empty());
    assert_eq!(
        observation.scope_health.total_claim_versions,
        observation.claim_versions.len()
    );
    assert!(!observation.temporal_snapshots.is_empty());
}
