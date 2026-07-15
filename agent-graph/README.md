# agent-graph

Graph-based agent orchestration for Rust — a LangGraph-inspired execution engine with checkpointing, branching, interrupt/resume, payload integration, token streaming, and execution receipts.

## What it gives you

- **Directed graph execution**: nodes, edges, conditional routing, parallel branches, cycles with bounded iterations
- **Checkpointing**: SQLite-backed state persistence for interrupt/resume workflows
- **Payload trait**: clean boundary between the graph orchestrator and external work units (e.g., LLM calls from `llm-pipeline`)
- **Token streaming**: `PayloadContext` provides a token sink so payloads can stream tokens to the `EventSink` during execution
- **Execution receipts**: `GraphExecutionReceiptV1` with per-step `StepExecutionReceiptV1` — input/output digests, tool call receipts, memory generation refs, and replay verification
- **Run bundles**: `RunBundleV1` for durable audit persistence with redacted digest-chain integrity checking
- **Event sink**: pluggable event system emitting node lifecycle, token, and completion events

## Quick start

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

## Payload integration

The `Payload` trait is the canonical interface between the orchestrator and external payload implementations. The graph runtime handles scheduling, checkpointing, and event emission — never payload logic.

```rust
use agent_graph::prelude::*;
use serde_json::Value;

struct MyPayload;

impl Payload for MyPayload {
    fn invoke(
        &self,
        input: Value,
        ctx: &PayloadContext,
    ) -> Pin<Box<dyn Future<Output = Result<PayloadOutput, PayloadError>> + Send + '_>> {
        let on_token = ctx.on_token.clone();
        Box::pin(async move {
            // Stream tokens if a sink is connected
            if let Some(callback) = on_token {
                callback("generating...");
            }
            Ok(PayloadOutput::new(input))
        })
    }
}
```

## Execution receipts

```rust
let (state, receipt) = graph.execute_with_receipt(json!({})).await?;

// receipt.steps: Vec<StepExecutionReceiptV1> — per-step input/output digests
// receipt.replay_verification: ReplayVerification — integrity check
// receipt.run_bundle: RunBundleV1 — durable audit bundle
```

## Architecture

```
AgentGraph
  ├── Builder → nodes, edges, conditional routing, config
  ├── Engine → execution loop, scheduled nodes, parallel branches
  ├── Payload → trait boundary for external work (LLM calls, tools, etc.)
  ├── PayloadContext → token streaming sink, run/node IDs
  ├── EventSink → pluggable event system (tokens, node lifecycle)
  ├── Receipt → GraphExecutionReceiptV1, StepExecutionReceiptV1, RunBundleV1
  ├── Checkpoint → SQLite state persistence (optional feature)
  └── Interrupt → InterruptCheckpoint, ExecutionResult for resume
```

See `ARCHITECTURE.md` for the full design document.

## Ecosystem

- **stack-ids**: `TraceCtx`, `AttemptId`, `TrialId` for execution tracing
- **llm-pipeline**: `LlmCall` implements `Payload` trait for use as graph nodes
- **agent-graph-mcp**: MCP server exposing graph-orchestrated LLM workflows

## stack-ids integration

Fully integrated. Graph executions carry `TraceCtx` for correlation, `AttemptId` per retry family, and `TrialId` per individual node trial.

## Verification

```bash
cargo test -p agent-graph
cargo clippy -p agent-graph --all-targets -- -D warnings
```

## License

MIT