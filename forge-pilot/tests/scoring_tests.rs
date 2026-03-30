mod common;

use common::{
    base_loop_config, import_v3_bundle, open_forge_store, open_memory_store, resources,
    sample_bundle, tempdir,
};
use forge_pilot::{observe_scope, score_targets, PilotHistory};
use knowledge_runtime::Scope;

#[tokio::test]
async fn scoring_and_ordering_are_deterministic_and_retry_decay_applies() {
    let dir = tempdir();
    let memory_store = open_memory_store(dir.path());
    let forge_store = open_forge_store(dir.path());
    let scope = Scope::new("pilot-score");
    let config = base_loop_config(scope.clone());

    import_v3_bundle(
        &memory_store,
        &forge_store,
        &scope.namespace,
        &sample_bundle("score-1"),
    )
    .await;

    let resources = resources(memory_store, forge_store, &config);
    let observation = observe_scope(&resources.runtime, &resources.memory_store, &config)
        .await
        .unwrap();

    let history = PilotHistory::default();
    let first = score_targets(&observation, &history, &config);
    let second = score_targets(&observation, &history, &config);

    assert!(!first.is_empty());
    assert_eq!(
        first
            .iter()
            .map(|candidate| candidate.stable_key.clone())
            .collect::<Vec<_>>(),
        second
            .iter()
            .map(|candidate| candidate.stable_key.clone())
            .collect::<Vec<_>>()
    );

    let mut history = PilotHistory::default();
    history.mark_selected(&first[0].stable_key);
    let decayed = score_targets(&observation, &history, &config);
    let original = first
        .iter()
        .find(|candidate| candidate.stable_key == first[0].stable_key)
        .unwrap()
        .urgency;
    let decayed_urgency = decayed
        .iter()
        .find(|candidate| candidate.stable_key == first[0].stable_key)
        .unwrap()
        .urgency;

    assert!(decayed_urgency < original);
}
