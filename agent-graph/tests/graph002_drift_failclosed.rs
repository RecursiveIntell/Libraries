// GRAPH-002: Fail closed on graph drift and deterministic joins.
//
// RED:
//   - `resume()` with a checkpoint carrying no graph hash silently proceeds
//     (unverified resume is advertised as available).
//   - Fan-in (parallel join) determinism is unproven by any test.
//
// GREEN:
//   - Resume without a graph hash returns a typed error (unsupported resume
//     stays typed/degraded); `resume_force` remains the explicit bypass.
//   - Identical parallel graphs produce identical final state across runs.
use agent_graph::error::AgentGraphError;
use agent_graph::interrupt::InterruptCheckpoint;
use agent_graph::prelude::*;

fn no_hash_checkpoint() -> InterruptCheckpoint {
    InterruptCheckpoint {
        resume_node: "step".to_string(),
        resume_before: true,
        iteration: 1,
        active_nodes: vec!["step".to_string()],
        graph_hash: None,
    }
}

#[tokio::test]
async fn resume_without_graph_hash_fails_closed() {
    let graph = AgentGraph::builder()
        .add_node(
            "step",
            node!(|state| async move {
                state.set("resumed", true).await?;
                Ok(())
            }),
        )
        .build()
        .unwrap();

    // A checkpoint without a graph hash must NOT silently resume: the
    // topology cannot be verified, so the resume is refused with a typed
    // error rather than being advertised as available.
    let err = graph
        .resume(
            AgentState::new(),
            GraphConfig::default(),
            no_hash_checkpoint(),
        )
        .await
        .unwrap_err();

    assert!(
        matches!(err, AgentGraphError::UnverifiedCheckpoint),
        "expected typed unverified-checkpoint error, got: {err:?}"
    );
}

#[tokio::test]
async fn resume_force_is_the_explicit_bypass_for_missing_hash() {
    let graph = AgentGraph::builder()
        .add_node(
            "step",
            node!(|state| async move {
                state.set("resumed", true).await?;
                Ok(())
            }),
        )
        .build()
        .unwrap();

    let resumed = graph
        .resume_force(
            AgentState::new(),
            GraphConfig::default(),
            no_hash_checkpoint(),
        )
        .await
        .unwrap();

    let resumed_flag: bool = resumed.get("resumed").await.unwrap();
    assert!(resumed_flag);
}

#[tokio::test]
async fn parallel_fan_in_merge_is_deterministic_across_runs() {
    // Two parallel branches write distinct keys; the join must produce the
    // same final state on every run regardless of branch completion order.
    let graph = AgentGraph::builder()
        .add_node(
            "start",
            node!(|state| async move {
                state.set("phase", "start").await?;
                Ok(())
            }),
        )
        .add_node(
            "branch-a",
            node!(|state| async move {
                state.set("a", "alpha").await?;
                Ok(())
            }),
        )
        .add_node(
            "branch-b",
            node!(|state| async move {
                state.set("b", "beta").await?;
                Ok(())
            }),
        )
        .add_edge("start", "branch-a")
        .add_edge("start", "branch-b")
        .build()
        .unwrap();

    let mut first_state: Option<serde_json::Value> = None;
    for _ in 0..5 {
        let state = graph.execute("start", AgentState::new()).await.unwrap();
        // Compare parsed values, not strings: HashMap serialization order is
        // intentionally nondeterministic (random hasher seed) and must not
        // masquerade as merge nondeterminism.
        let snapshot = serde_json::to_value(state.export().await).unwrap();
        match &first_state {
            None => first_state = Some(snapshot),
            Some(expected) => assert_eq!(
                expected, &snapshot,
                "fan-in merge must be deterministic across runs"
            ),
        }
    }

    let snap = first_state.expect("at least one run produced state");
    assert_eq!(snap["a"], "alpha");
    assert_eq!(snap["b"], "beta");
}
