# agent-graph

Graph-based agent orchestration for Rust -- a LangGraph-inspired execution engine with checkpointing, branching, and interrupt/resume.

## Example

```rust
use agent_graph::{AgentGraph, node, START, END};
use serde_json::{json, Value};

let graph = AgentGraph::builder()
    .with_name("my_graph")
    .add_node("greet", node!(|state: Value| async move {
        json!({"message": "hello"})
    }))
    .add_edge(START, "greet")
    .add_edge("greet", END)
    .build()?;

let result = graph.execute(json!({})).await?;
```

## Ecosystem

- **stack-ids**: `TraceCtx`, `AttemptId`, `TrialId` for execution tracing
- **LLM-Pipeline**: Provides `Payload` trait implementations that run as graph nodes

## stack-ids integration

Fully integrated. Graph executions carry `TraceCtx` for correlation, `AttemptId` per retry family, and `TrialId` per individual node trial.
