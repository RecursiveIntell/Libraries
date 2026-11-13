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

#[tokio::test]
async fn target_selection_is_compared_against_a_simple_deterministic_baseline() {
    // S3-03.06: evaluate target selection on frozen replay cases and compare with a
    // SIMPLER deterministic priority baseline. The baseline here is the naive
    // "first candidate by stable-key ascending" ordering; the pilot's ordering is by
    // urgency. Both must be deterministic, and the pilot must not be WORSE than the
    // naive baseline on its own declared criterion (urgency).
    let dir = tempdir();
    let memory_store = open_memory_store(dir.path());
    let forge_store = open_forge_store(dir.path());
    let scope = Scope::new("pilot-score-baseline");
    let config = base_loop_config(scope.clone());

    import_v3_bundle(
        &memory_store,
        &forge_store,
        &scope.namespace,
        &sample_bundle("score-baseline"),
    )
    .await;

    let resources = resources(memory_store, forge_store, &config);
    let observation = observe_scope(&resources.runtime, &resources.memory_store, &config)
        .await
        .unwrap();

    let history = PilotHistory::default();
    let pilot = score_targets(&observation, &history, &config);
    assert!(
        !pilot.is_empty(),
        "frozen case set must select at least one target"
    );

    // Simple deterministic baseline: stable-key ascending.
    let mut baseline_keys: Vec<String> = pilot.iter().map(|c| c.stable_key.clone()).collect();
    baseline_keys.sort();
    let baseline_top_key = baseline_keys[0].clone();

    let pilot_top_urgency = pilot[0].urgency;
    let baseline_top_urgency = pilot
        .iter()
        .find(|c| c.stable_key == baseline_top_key)
        .unwrap()
        .urgency;

    // The pilot's declared criterion must not be worse than the naive baseline's pick.
    assert!(
        pilot_top_urgency >= baseline_top_urgency,
        "pilot selection ({:?}, urgency {}) must not be worse than the simple baseline ({:?}, urgency {})",
        pilot[0].stable_key,
        pilot_top_urgency,
        baseline_top_key,
        baseline_top_urgency
    );

    // Determinism of the pilot ordering under replay.
    let replay = score_targets(&observation, &history, &config);
    assert_eq!(
        replay
            .iter()
            .map(|c| c.stable_key.clone())
            .collect::<Vec<_>>(),
        pilot
            .iter()
            .map(|c| c.stable_key.clone())
            .collect::<Vec<_>>(),
        "replaying the frozen cases must reproduce the same ordering"
    );
}
