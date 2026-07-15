//! Tests that the `execute_with_receipt` API actually emits a real
//! `GraphExecutionReceiptV1` for a graph execution, not just declares
//! the type.
//!
//! Closes P1-3 from the V30 corrected audit: "agent-graph has no
//! execution receipt infrastructure / receipts are typed surfaces,
//! never emitted".

#![allow(clippy::expect_used)] // test code — expect() on Result/Option is the idiomatic pattern

use agent_graph::prelude::*;
use agent_graph::{ExecutionOutcome, GraphExecutionReceiptV1};

fn build_counting_graph() -> AgentGraph {
    AgentGraph::builder()
        .add_node(
            "step1",
            node!(|state| async move {
                state.set("count", 1).await?;
                Ok(())
            }),
        )
        .add_node(
            "step2",
            node!(|state| async move {
                let count: i32 = state.get("count").await?;
                state.set("count", count + 1).await?;
                Ok(())
            }),
        )
        .add_edge("step1", "step2")
        .build()
        .expect("graph must build")
}

#[tokio::test]
async fn execute_with_receipt_emits_completed_receipt_for_clean_run() {
    let graph = build_counting_graph();
    let state = AgentState::new();

    let (result, receipt) = graph
        .execute_with_receipt("step1", state, GraphConfig::default())
        .await;

    let final_state = result.expect("clean run must succeed");
    let final_count: i32 = final_state.get("count").await.expect("count");
    assert_eq!(final_count, 2, "step1 + step2 must both run");

    // The whole point: a real `GraphExecutionReceiptV1` came back, not
    // a typed surface, not a placeholder. The receipt describes the
    // actual run we just did.
    assert!(
        matches!(receipt.outcome, ExecutionOutcome::Completed),
        "clean run must produce a Completed receipt, got {:?}",
        receipt.outcome
    );
    assert!(!receipt.execution_id.is_empty(), "execution_id must be set");
    assert!(!receipt.graph_id.is_empty(), "graph_id must be set");
    assert!(receipt.started_at <= receipt.finished_at);
    assert_eq!(receipt.steps.len(), 2, "one receipt per executed node");
    assert_eq!(receipt.steps[0].agent_id, "step1");
    assert_eq!(receipt.steps[1].agent_id, "step2");
    for (index, step) in receipt.steps.iter().enumerate() {
        assert_eq!(step.step_index, index);
        assert!(step.tool_calls.is_empty());
        assert!(step.error.is_none(), "clean run must not carry an error");
    }
    assert_eq!(
        receipt.steps[0].output_digest,
        receipt.steps[1].input_digest
    );
}

#[tokio::test]
async fn execute_with_receipt_records_parallel_nodes_in_scheduling_order() {
    let graph = AgentGraph::builder()
        .add_node("start", node!(|_state| async move { Ok(()) }))
        .add_node(
            "second",
            node!(|state| async move {
                state.set("second", true).await?;
                Ok(())
            }),
        )
        .add_node(
            "first",
            node!(|state| async move {
                state.set("first", true).await?;
                Ok(())
            }),
        )
        .add_edge("start", "second")
        .add_edge("start", "first")
        .build()
        .expect("graph must build");

    let (result, receipt) = graph
        .execute_with_receipt("start", AgentState::new(), GraphConfig::default())
        .await;
    result.expect("parallel run");

    let names: Vec<_> = receipt
        .steps
        .iter()
        .map(|step| step.agent_id.as_str())
        .collect();
    assert_eq!(names, ["start", "second", "first"]);
    assert_eq!(
        receipt.steps[1].input_digest, receipt.steps[2].input_digest,
        "parallel branches receive the same superstep snapshot"
    );
    assert_ne!(
        receipt.steps[1].output_digest,
        receipt.steps[2].output_digest
    );
}

#[tokio::test]
async fn execute_with_receipt_records_failed_node_and_partial_state() {
    let graph = AgentGraph::builder()
        .add_node(
            "fail",
            node!(|state| async move {
                state.set("written_before_error", true).await?;
                Err::<(), _>(AgentGraphError::ExecutionError("expected failure".into()))
            }),
        )
        .build()
        .unwrap();

    let (result, receipt) = graph
        .execute_with_receipt("fail", AgentState::new(), GraphConfig::default())
        .await;

    assert!(result.is_err());
    assert!(matches!(
        receipt.outcome,
        ExecutionOutcome::InternalError { .. }
    ));
    assert_eq!(receipt.steps.len(), 1);
    assert_eq!(receipt.steps[0].agent_id, "fail");
    assert!(receipt.steps[0]
        .error
        .as_deref()
        .is_some_and(|error| error.contains("expected failure")));
    assert_ne!(
        receipt.steps[0].input_digest,
        receipt.steps[0].output_digest
    );
}

#[tokio::test]
async fn execute_with_receipt_receipt_serializes_to_json_round_trip() {
    let graph = build_counting_graph();
    let (result, receipt) = graph
        .execute_with_receipt("step1", AgentState::new(), GraphConfig::default())
        .await;
    result.expect("clean run");

    // The receipt is the audit handle. A consumer must be able to
    // persist it (serialize) and reload it (deserialize) for replay.
    let json = serde_json::to_string(&receipt).expect("receipt must serialize");
    let restored: GraphExecutionReceiptV1 =
        serde_json::from_str(&json).expect("receipt must deserialize");
    assert_eq!(restored.execution_id, receipt.execution_id);
    assert_eq!(restored.graph_id, receipt.graph_id);
    assert!(matches!(restored.outcome, ExecutionOutcome::Completed));
}

#[tokio::test]
async fn execute_with_receipt_uses_distinct_execution_ids_per_run() {
    let graph = build_counting_graph();

    let (_, r1) = graph
        .execute_with_receipt("step1", AgentState::new(), GraphConfig::default())
        .await;
    let (_, r2) = graph
        .execute_with_receipt("step1", AgentState::new(), GraphConfig::default())
        .await;

    // Two separate executions must produce two distinct execution_ids
    // (each run is its own audit event). graph_id stays the same.
    assert_ne!(r1.execution_id, r2.execution_id);
    assert_eq!(r1.graph_id, r2.graph_id);
}

/// GRAPH-001 fix: receipts must contain real digests, not placeholders.
#[tokio::test]
async fn execute_with_receipt_contains_real_input_digest_not_placeholder() {
    let graph = build_counting_graph();
    let (result, receipt) = graph
        .execute_with_receipt("step1", AgentState::new(), GraphConfig::default())
        .await;
    result.expect("clean run");

    let step = &receipt.steps[0];
    // GRAPH-001: input_digest must NOT be the literal "graph-root" placeholder
    assert_ne!(
        step.input_digest, "graph-root",
        "input_digest must be a real digest, not the 'graph-root' placeholder"
    );
    // output_digest must NOT be a node-count string
    assert!(
        !step.output_digest.starts_with("nodes_executed="),
        "output_digest must be a real digest, not a node-count string: {}",
        step.output_digest
    );
    // Input and output digests should be different (different state)
    assert_ne!(
        step.input_digest, step.output_digest,
        "input and output digests should differ (state changed)"
    );
}

/// GRAPH-001 fix: mutating input state changes the input digest.
#[tokio::test]
async fn execute_with_receipt_input_digest_changes_with_different_state() {
    let graph = build_counting_graph();

    let state1 = {
        let s = AgentState::new();
        // set requires &self, not &mut self, so no mut needed
        s
    };
    let _ = state1.set("seed", 1).await;

    let state2 = AgentState::new();
    let _ = state2.set("seed", 2).await;

    let (_, r1) = graph
        .execute_with_receipt("step1", state1, GraphConfig::default())
        .await;
    let (_, r2) = graph
        .execute_with_receipt("step1", state2, GraphConfig::default())
        .await;

    // Different input state must produce different input digests
    assert_ne!(
        r1.steps[0].input_digest, r2.steps[0].input_digest,
        "different input states must produce different input digests"
    );
}

/// GRAPH-002 fix: non-interrupt errors must not be reported as Complete.
#[tokio::test]
async fn execute_with_interrupt_reports_failure_not_complete() {
    let graph = AgentGraph::builder()
        .add_node(
            "fail_node",
            node!(|_state| async move {
                Err::<(), _>(AgentGraphError::ExecutionError(
                    "deliberate test failure".to_string(),
                ))
            }),
        )
        .build()
        .expect("graph must build");

    let result = graph
        .execute_with_interrupt("fail_node", AgentState::new(), GraphConfig::default())
        .await;

    // GRAPH-002: a failed execution must NOT be Complete
    assert!(
        !matches!(result, ExecutionResult::Complete(_)),
        "non-interrupt errors must not be reported as Complete"
    );
    // It should be Failed with the error message
    match &result {
        ExecutionResult::Failed { error, .. } => {
            assert!(
                error.contains("deliberate test failure"),
                "error message should contain the original error: {}",
                error
            );
        }
        other => panic!("expected Failed, got {:?}", other),
    }
}
