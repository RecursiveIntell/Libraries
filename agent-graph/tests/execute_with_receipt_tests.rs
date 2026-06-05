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
    // The step we emitted is a run-level placeholder until per-step
    // instrumentation lands; verify it has the structural shape we
    // promise in the docstring.
    assert_eq!(receipt.steps.len(), 1, "one run-level step entry");
    let step = &receipt.steps[0];
    assert_eq!(step.step_index, 0);
    assert!(step.tool_calls.is_empty(), "no tool calls at the run level");
    assert!(step.error.is_none(), "clean run must not carry an error");
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
