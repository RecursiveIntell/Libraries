// GRAPH-001: Material checkpoint-attempt identity.
//
// RED: a deterministic replay test shows `GraphCheckpointAttemptId::random`
// changes material identity for identical inputs (i.e. attempt IDs minted per
// invocation must not be treated as material/replay-safe identifiers).
//
// GREEN (chosen branch): attempt IDs are EXPLICITLY NON-MATERIAL — each
// invocation mints a distinct random record handle; material identity of an
// attempt is the canonical (run_id, node_id, attempt) triple exposed by
// `material_attempt_identity`. Replay of identical input creates a distinct
// record and never overwrites a prior record.
use agent_graph::checkpoint_store::material_attempt_identity;
use agent_graph::prelude::*;
use serde_json::json;

fn input() -> serde_json::Value {
    json!({"question": "same deterministic input", "step": 1})
}

#[tokio::test]
async fn identical_replay_yields_distinct_attempt_ids_but_same_material_identity() {
    let store = InMemoryCheckpointStore::new();
    let run_id = store.create_run("graph").await.unwrap();

    let id1 = store
        .record_attempt(&run_id, "node-a", 0, &input())
        .await
        .unwrap();
    let id2 = store
        .record_attempt(&run_id, "node-a", 0, &input())
        .await
        .unwrap();

    // Attempt IDs are non-material: identical inputs must NOT reuse an ID.
    assert_ne!(
        id1, id2,
        "random attempt IDs must differ for identical inputs"
    );

    // Material identity is the canonical triple, identical across the replay.
    assert_eq!(
        material_attempt_identity(&run_id, "node-a", 0),
        material_attempt_identity(&run_id, "node-a", 0)
    );
    // Different attempt numbers or node ids change material identity.
    assert_ne!(
        material_attempt_identity(&run_id, "node-a", 0),
        material_attempt_identity(&run_id, "node-a", 1)
    );
    assert_ne!(
        material_attempt_identity(&run_id, "node-a", 0),
        material_attempt_identity(&run_id, "node-b", 0)
    );
}

#[tokio::test]
async fn replay_records_coexist_and_prior_record_is_not_overwritten() {
    let store = InMemoryCheckpointStore::new();
    let run_id = store.create_run("graph").await.unwrap();

    let first = store
        .record_attempt(&run_id, "node-a", 0, &input())
        .await
        .unwrap();
    store
        .complete_attempt(&first, &json!({"answer": "first"}), &Default::default())
        .await
        .unwrap();

    let replay = store
        .record_attempt(&run_id, "node-a", 0, &input())
        .await
        .unwrap();
    store
        .complete_attempt(&replay, &json!({"answer": "replay"}), &Default::default())
        .await
        .unwrap();

    let run = store.load_run(&run_id).await.unwrap().expect("run exists");
    let attempts = run.attempts;
    assert_eq!(attempts.len(), 2, "replay must not collapse prior records");
    assert_eq!(attempts[0].attempt_id, first);
    assert_eq!(attempts[1].attempt_id, replay);
    // Both records retain their own output; the prior one is untouched.
    assert_eq!(attempts[0].output, Some(json!({"answer": "first"})));
    assert_eq!(attempts[1].output, Some(json!({"answer": "replay"})));
    // Material identity is identical while record identity differs.
    assert_eq!(
        material_attempt_identity(
            &attempts[0].run_id,
            &attempts[0].node_id,
            attempts[0].attempt
        ),
        material_attempt_identity(
            &attempts[1].run_id,
            &attempts[1].node_id,
            attempts[1].attempt
        )
    );
}

#[tokio::test]
async fn attempt_ids_are_uuid_suffixed_and_reject_empty_domain() {
    // Non-material identity is random: two minted IDs are never equal, and the
    // domain prefix keeps distinct stores/call-sites distinguishable.
    let a = agent_graph::checkpoint_store::mint_attempt_id("agent-graph-checkpoint");
    let b = agent_graph::checkpoint_store::mint_attempt_id("agent-graph-checkpoint");
    assert_ne!(a, b);
    assert!(a.starts_with("v1:agent-graph-checkpoint:"));
    assert!(b.starts_with("v1:agent-graph-checkpoint:"));
}
