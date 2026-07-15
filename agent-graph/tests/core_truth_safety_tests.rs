#![allow(deprecated)]

use agent_graph::prelude::*;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};

struct ContextPayload {
    seen: Arc<Mutex<Vec<(String, String)>>>,
    token: &'static str,
}

impl Payload for ContextPayload {
    fn invoke(
        &self,
        _input: Value,
        ctx: &PayloadContext,
    ) -> Pin<Box<dyn Future<Output = std::result::Result<PayloadOutput, PayloadError>> + Send + '_>>
    {
        let run_id = ctx.run_id.clone();
        let node_id = ctx.node_id.clone();
        let on_token = ctx.on_token.clone();
        let token = self.token;
        let seen = self.seen.clone();
        Box::pin(async move {
            seen.lock().unwrap().push((run_id, node_id));
            if let Some(callback) = on_token {
                callback(token);
            }
            Ok(PayloadOutput {
                value: json!({}),
                meta: HashMap::new(),
            })
        })
    }
}

#[tokio::test]
async fn payload_context_has_real_ids_and_tokens_reach_sink_in_sequential_execution() {
    let seen = Arc::new(Mutex::new(Vec::new()));
    let events = Arc::new(Mutex::new(Vec::new()));
    let events_for_sink = events.clone();
    let graph = AgentGraph::builder()
        .with_event_sink(Arc::new(CallbackEventSink::new(move |event| {
            events_for_sink.lock().unwrap().push(event);
        })))
        .add_node(
            "payload",
            Box::new(PayloadNode::new(Box::new(ContextPayload {
                seen: seen.clone(),
                token: "hello",
            }))),
        )
        .build()
        .unwrap();

    graph.execute("payload", AgentState::new()).await.unwrap();

    let seen = seen.lock().unwrap();
    assert_eq!(seen.len(), 1);
    assert!(!seen[0].0.is_empty());
    assert_eq!(seen[0].1, "payload");
    assert!(events.lock().unwrap().iter().any(|event| matches!(
        event,
        GraphEvent::Token { run_id, node_id, token, .. }
            if run_id == &seen[0].0 && node_id == "payload" && token == "hello"
    )));
}

#[tokio::test]
async fn payload_context_is_passed_to_parallel_in_process_nodes() {
    let seen = Arc::new(Mutex::new(Vec::new()));
    let graph = AgentGraph::builder()
        .add_node("start", node!(|_state| async move { Ok(()) }))
        .add_node(
            "left",
            Box::new(PayloadNode::new(Box::new(ContextPayload {
                seen: seen.clone(),
                token: "left-token",
            }))),
        )
        .add_node(
            "right",
            Box::new(PayloadNode::new(Box::new(ContextPayload {
                seen: seen.clone(),
                token: "right-token",
            }))),
        )
        .add_edge("start", "left")
        .add_edge("start", "right")
        .build()
        .unwrap();

    graph.execute("start", AgentState::new()).await.unwrap();
    let mut seen = seen.lock().unwrap().clone();
    seen.sort_by(|a, b| a.1.cmp(&b.1));
    assert_eq!(seen.len(), 2);
    assert_eq!(seen[0].1, "left");
    assert_eq!(seen[1].1, "right");
    assert!(!seen[0].0.is_empty());
    assert_eq!(seen[0].0, seen[1].0);
}

#[tokio::test]
async fn send_op_preserves_distinct_state_for_duplicate_target_and_merges_in_send_order() {
    let graph = AgentGraph::builder()
        .add_node(
            "dispatch",
            node!(|_state| async move {
                Ok(NodeOutput::Command(Command {
                    update: None,
                    goto: Navigation::Send(vec![
                        SendOp {
                            node: "worker".into(),
                            state: HashMap::from([("item".into(), json!(2))]),
                        },
                        SendOp {
                            node: "worker".into(),
                            state: HashMap::from([("item".into(), json!(7))]),
                        },
                    ]),
                }))
            }),
        )
        .add_node(
            "worker",
            node!(|state| async move {
                let item: i64 = state.get("item").await?;
                state.set("results", vec![item * 10]).await?;
                Ok(())
            }),
        )
        .with_reducer("results", AppendReducer)
        .build()
        .unwrap();

    let result = graph
        .execute_with_config(
            "dispatch",
            AgentState::new(),
            GraphConfig::default().with_max_parallelism(1),
        )
        .await
        .unwrap();
    let results: Vec<i64> = result.get("results").await.unwrap();
    assert_eq!(results, vec![20, 70]);
}
