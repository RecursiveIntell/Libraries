use agent_graph::checkpointer::CheckpointSaver;
use agent_graph::prelude::*;
use agent_graph::receipt::ReplayError;
use agent_graph::{CheckpointPolicy, GraphSpecV1};
use async_trait::async_trait;
use serde_json::{json, Value};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

#[derive(Clone)]
struct FailingSaver;

#[async_trait]
impl CheckpointSaver for FailingSaver {
    async fn save(&self, _checkpoint: &Checkpoint) -> Result<()> {
        Err(AgentGraphError::CheckpointError("injected failure".into()))
    }

    async fn load(&self, _execution_id: &str) -> Result<Option<Checkpoint>> {
        Ok(None)
    }

    async fn load_history(&self, _execution_id: &str) -> Result<Vec<Checkpoint>> {
        Ok(Vec::new())
    }

    async fn clear(&self, _execution_id: &str) -> Result<()> {
        Ok(())
    }
}

fn checkpoint_graph(policy: CheckpointPolicy, calls: Arc<AtomicUsize>) -> AgentGraph {
    let first_calls = calls.clone();
    let second_calls = calls;
    AgentGraph::builder()
        .with_name("checkpoint-policy-test")
        .with_checkpoint_policy(policy)
        .with_checkpointer(FailingSaver)
        .add_node(
            "first",
            node!(|state| async move {
                first_calls.fetch_add(1, Ordering::SeqCst);
                state.set("first", true).await?;
                Ok(())
            }),
        )
        .add_node(
            "second",
            node!(|state| async move {
                second_calls.fetch_add(1, Ordering::SeqCst);
                state.set("second", true).await?;
                Ok(())
            }),
        )
        .add_edge("first", "second")
        .set_finish_point("second")
        .build()
        .unwrap()
}

#[tokio::test]
async fn required_checkpoint_failure_stops_before_next_node_with_partial_receipt() {
    let calls = Arc::new(AtomicUsize::new(0));
    let graph = checkpoint_graph(CheckpointPolicy::Required, calls.clone());
    let config = GraphConfig::default().with_thread_id("required-run");

    let (result, receipt) = graph
        .execute_with_receipt("first", AgentState::new(), config)
        .await;

    assert!(matches!(result, Err(AgentGraphError::CheckpointError(_))));
    assert_eq!(calls.load(Ordering::SeqCst), 1, "second node must not run");
    assert!(matches!(
        receipt.outcome,
        ExecutionOutcome::Partial { failed_step: 0 }
    ));
    assert_eq!(receipt.steps.len(), 1);
    assert!(receipt.steps[0]
        .error
        .as_deref()
        .unwrap_or_default()
        .contains("injected failure"));
}

#[tokio::test]
async fn best_effort_checkpoint_failure_continues() {
    let calls = Arc::new(AtomicUsize::new(0));
    let graph = checkpoint_graph(CheckpointPolicy::BestEffort, calls.clone());
    let config = GraphConfig::default().with_thread_id("best-effort-run");

    let result = graph
        .execute_with_config("first", AgentState::new(), config)
        .await;

    assert!(result.is_ok());
    assert_eq!(calls.load(Ordering::SeqCst), 2);
}

struct SchemaNode(&'static str);

#[async_trait]
impl Node for SchemaNode {
    async fn execute(&self, _state: &AgentState, _config: &GraphConfig) -> Result<NodeOutput> {
        Ok(NodeOutput::Done)
    }

    fn payload_schema(&self) -> Value {
        json!({"type": "object", "title": self.0})
    }
}

struct DescribedRouter(&'static str);

#[async_trait]
impl RoutingFunction for DescribedRouter {
    async fn route(&self, _state: &AgentState, _config: &GraphConfig) -> Result<RouterOutput> {
        Ok(RouterOutput::Next(None))
    }

    fn condition_spec(&self) -> Value {
        json!({"expression": self.0})
    }
}

fn semantic_graph(
    schema: &'static str,
    condition: &'static str,
    retry_attempts: usize,
    policy: CheckpointPolicy,
) -> AgentGraph {
    AgentGraph::builder()
        .with_name("semantic")
        .with_checkpoint_policy(policy)
        .add_node_with_retry(
            "a",
            Box::new(SchemaNode(schema)),
            RetryPolicy::new()
                .with_max_attempts(retry_attempts)
                .with_jitter(false),
        )
        .add_node("b", Box::new(SchemaNode("b-schema")))
        .add_conditional_edge("a", Box::new(DescribedRouter(condition)))
        .add_edge("a", "b")
        .set_finish_point("b")
        .build()
        .unwrap()
}

#[test]
fn graph_spec_and_digest_cover_nodes_edges_and_policies() {
    let base = semantic_graph("a-schema", "x > 0", 2, CheckpointPolicy::BestEffort);
    let spec: GraphSpecV1 = base.graph_spec_v1();
    assert_eq!(spec.checkpoint_policy, CheckpointPolicy::BestEffort);
    assert_eq!(base.compute_graph_hash(), base.compute_graph_hash());

    assert_ne!(
        base.compute_graph_hash(),
        semantic_graph("changed", "x > 0", 2, CheckpointPolicy::BestEffort).compute_graph_hash()
    );
    assert_ne!(
        base.compute_graph_hash(),
        semantic_graph("a-schema", "x >= 0", 2, CheckpointPolicy::BestEffort).compute_graph_hash()
    );
    assert_ne!(
        base.compute_graph_hash(),
        semantic_graph("a-schema", "x > 0", 3, CheckpointPolicy::BestEffort).compute_graph_hash()
    );
    assert_ne!(
        base.compute_graph_hash(),
        semantic_graph("a-schema", "x > 0", 2, CheckpointPolicy::Required).compute_graph_hash()
    );
}

#[test]
fn graph_digest_matches_cross_process_golden() {
    let digest =
        semantic_graph("a-schema", "x > 0", 2, CheckpointPolicy::BestEffort).compute_graph_hash();
    assert_eq!(
        digest,
        "blake3:8b38d7d6fa043f277d284be52d6afcc135371bd3e9dfe4f82924a6aafd24dbb9"
    );
}

#[tokio::test]
async fn recorded_run_replays_offline_and_localizes_mutation() {
    let graph = AgentGraph::builder()
        .with_name("replay-test")
        .add_node(
            "one",
            node!(|state| async move {
                state.set("value", 1).await?;
                Ok(())
            }),
        )
        .add_node(
            "two",
            node!(|state| async move {
                state.set("value", 2).await?;
                Ok(())
            }),
        )
        .add_edge("one", "two")
        .set_finish_point("two")
        .build()
        .unwrap();

    let bundle = graph
        .record_run_bundle("one", AgentState::new(), GraphConfig::default())
        .await
        .expect("record run");
    let verified = graph.verify_replay(&bundle).expect("offline replay");
    assert_eq!(verified.steps_verified, 2);

    let mut mutated = bundle.clone();
    mutated.step_state_deltas[1]
        .state_after
        .insert("value".into(), json!(999));
    let error = graph.verify_replay(&mutated).unwrap_err();
    assert!(matches!(
        error,
        ReplayError::StepDivergence { step_index: 1, .. }
    ));
}
