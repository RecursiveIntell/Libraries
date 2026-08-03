# agent-graph

A Rust runtime for composing agent workflows as explicit graphs.

`agent-graph` owns **control flow, shared state, bounded execution, retries, interrupts, checkpoints, reducers, event sinks, and execution summaries**. It does not call an LLM, browse the web, authenticate an MCP client, or decide whether a source is trustworthy. Those concerns belong in adjacent layers such as [`llm-pipeline`](../llm-pipeline) and [`agent-graph-mcp`](../agent-graph-mcp).

> Current crate version: `0.2.0` · Rust 2021 · MIT
>
> This README describes the checked-in source. It makes no claim about hosted availability, benchmark superiority, production maturity, or external service integrations.

## Why this crate exists

Agent workflows become difficult to reason about when control flow is hidden inside prompts or ad-hoc callbacks. This crate makes the workflow topology and runtime policies inspectable:

```text
AgentGraph
  ├── named Nodes             do work against AgentState
  ├── normal / conditional edges
  ├── fan-out and subgraphs   compose execution
  ├── Reducers                define concurrent state merge semantics
  ├── RetryPolicy             bounds transient node retries
  ├── CheckpointStore         records durable run identity / attempts
  ├── CheckpointSaver         legacy thread-level state persistence
  ├── EventSink / Executor    observe or customize execution
  └── Interrupt + resume      pause, inspect, and continue safely
```

## Ecosystem boundaries

| Crate | Owns | Does not own |
| --- | --- | --- |
| `agent-graph` | In-process graph execution and state | Provider HTTP, MCP transport, web research, credentials |
| `llm-pipeline` | LLM payloads, provider backends, parsing, retries, streaming | Graph registration, MCP lifecycle, durable graph catalog |
| `agent-graph-mcp` | MCP tools, graph specs, persistence, runs, receipts, approvals, witnesses | Human authority outside its configured approval contract |

A useful dependency direction is:

```text
MCP client
    │ stdio / MCP tools
    ▼
agent-graph-mcp ──► agent-graph
        │          └─► llm-pipeline
        └─► SQLite / integrity / receipt projections when durable mode is enabled
```

## Quick start

From the `Libraries` workspace:

```bash
cargo check -p agent-graph
cargo test -p agent-graph
```

The following is the smallest useful in-process graph. It uses the public builder, node macro, typed state API, and ordinary execution path:

```rust
use agent_graph::{node, AgentGraph, AgentState};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let graph = AgentGraph::builder()
        .add_node(
            "write_result",
            node!(|state| async move {
                state.set("answer", "done").await?;
                Ok(())
            }),
        )
        .set_entry_point("write_result")
        .set_finish_point("write_result")
        .build()?;

    let state = graph.execute("write_result", AgentState::new()).await?;
    let answer: String = state.get("answer").await?;
    assert_eq!(answer, "done");
    Ok(())
}
```

For a runnable source example with a real provider boundary, use `agent-graph-mcp`; for provider calls without MCP, use `llm-pipeline`.

## Core model

### Nodes

A node implements:

```rust
#[async_trait::async_trait]
pub trait Node: Send + Sync {
    async fn execute(
        &self,
        state: &AgentState,
        config: &GraphConfig,
    ) -> agent_graph::Result<NodeOutput>;
}
```

The `node!` macro provides named and unnamed forms, with or without access to `GraphConfig`. A node can mutate shared state and return `NodeOutput::Done`, or return a `NodeOutput` command that changes navigation.

```rust
use agent_graph::{node, NodeOutput};

let node = node!("route_or_finish", |state, config| async move {
    let should_continue: bool = state.get_opt("continue").await?.unwrap_or(false);
    let _ = config;
    if should_continue {
        Ok(NodeOutput::goto("next"))
    } else {
        Ok(NodeOutput::end())
    }
});
```

### Edges and routing

`AgentGraphBuilder` supports:

- `add_edge(from, to)` for normal edges;
- multiple normal edges from one node for parallel fan-out;
- `add_conditional_edge(from, router)` for state-dependent routing;
- `set_entry_point(node)` and `set_finish_point(node)` as start/end sugar;
- `add_subgraph(name, graph)` for nested graph composition.

A router returns `RouterOutput::Next(Some(node))`, `RouterOutput::Next(None)` to end, or `RouterOutput::FanOut(Vec<String>)` for parallel branches. The `router!` macro mirrors the node macro’s closure forms.

### State

`AgentState` is a cloneable, asynchronous, JSON-backed shared state container. The normal API is typed at the call site:

```rust
let state = AgentState::new();
state.set("count", 1_u64).await?;
let count: u64 = state.get("count").await?;
let maybe_value: Option<String> = state.get_opt("label").await?;
```

The state implementation also provides:

- `set_raw` for an already-built `serde_json::Value`;
- `update` for closure-based updates;
- bounded history snapshots;
- explicit `StateLimits` for key count, value size, history length, and lock timeout;
- `transaction()` with commit, rollback, and concurrent-version conflict detection;
- `fork`, `export`, and restore paths used by subgraphs and persistence;
- per-key reducers for parallel or repeated updates.

Default state limits are finite: 10,000 keys, 1 MiB per serialized value, 100 history snapshots, and a 5-second lock timeout. Tune them deliberately rather than treating state as an unbounded object store.

### Reducers

Without a registered reducer, a later write replaces the previous value. Built-in reducers include:

- `LastWriteWins`;
- `AppendReducer` for array accumulation;
- `AddReducer` for numeric accumulation;
- `MergeReducer` for recursive JSON-object merging;
- `FnReducer` for caller-defined merge semantics.

Reducers are correctness controls, not presentation helpers: concurrent branches must have an explicit merge policy when write order is not the business rule.

## Execution controls

The builder defaults to cycle detection and a maximum of 100 graph iterations. Configure limits explicitly when the workflow has a known bound:

```rust
let graph = AgentGraph::builder()
    .with_max_iterations(32)
    .with_cycle_detection(true)
    .with_name("bounded-workflow")
    // add nodes and edges...
    .build()?;
```

`GraphConfig` carries the runtime boundary:

- optional `thread_id` for legacy checkpointer lookup;
- canonical `stack_ids::TraceCtx` plus a legacy trace-ID compatibility field;
- recursion and parallelism limits;
- tags, metadata, and configurable values.

The runtime exposes three useful execution shapes:

- `execute(start, state)` for a final `Result<AgentState>`;
- `execute_with_summary(start, state, config)` for state plus run metrics and trace data;
- `execute_with_interrupt(start, state, config)` for `Complete`, `Interrupted`, or `Failed` outcomes.

Cancellation is available through the cancellable execution path, which returns a task handle and an atomic cancellation flag.

## Interrupts, resume, and checkpoints

Interrupts can be configured before or after named nodes:

```rust
let graph = AgentGraph::builder()
    .with_interrupt_before(vec!["approval".into()])
    .with_interrupt_after(vec!["draft".into()])
    // nodes and edges...
    .build()?;
```

An interrupt carries the current state, node, optional interrupt value, and an `InterruptCheckpoint`. Normal `resume` validates the saved graph topology hash before continuing. `resume_force` skips that topology check and is therefore an explicit escape hatch, not the default recovery path.

There are two persistence abstractions:

- `CheckpointSaver`: the older thread/superstep-oriented interface used by `get_state`, `get_state_history`, and `update_state`;
- `CheckpointStore`: the granular run/attempt-oriented interface used for durable run IDs and per-attempt recording.

If a configured durable store cannot create a run, the error is returned. The runtime does not silently replace a failed durable identity operation with a random local ID.

## Observability and extension points

The builder can attach:

- an `EventSink` for structured execution events;
- a custom `Executor` for execution policy or instrumentation;
- graph names for diagnostics and event metadata;
- a `CheckpointStore` or legacy `CheckpointSaver`;
- per-node `RetryPolicy` values.

`to_mermaid()` emits a deterministic graph diagram for topology inspection. `compute_graph_hash()` provides a stable topology digest used by interrupt resume validation.

## What this crate deliberately does not do

- It does not call Ollama, OpenAI, Anthropic, or another provider.
- It does not parse model output or implement tool/function calling.
- It does not expose MCP transport or a daemon.
- It does not fetch, verify, or cite web sources.
- It does not turn an `External`, `Tool`, or human-authority concept into an executable side effect by itself.
- It does not make a graph durable merely because a caller uses the word “checkpoint”. Configure and verify a persistence implementation.

Those boundaries are intentional. They make the graph runtime reusable and keep provider, transport, authority, and evidence concerns visible at their actual ownership layer.

## Validation

Run focused checks from the workspace root:

```bash
cargo fmt --check -p agent-graph
cargo check -p agent-graph --all-targets
cargo test -p agent-graph
```

The repository also contains a benchmark target:

```bash
cargo bench -p agent-graph --bench graph_bench
```

A benchmark command is an execution entry point, not a benchmark claim. Record the machine, compiler, feature set, workload, and raw output before publishing any number.

## Module map

| Module | Responsibility |
| --- | --- |
| `graph`, `builder`, `engine` | Graph definition and execution |
| `node`, `router`, `edge`, `command` | Work units and control-flow decisions |
| `state`, `reducer` | Shared JSON state and merge semantics |
| `config`, `retry` | Runtime and retry policy |
| `interrupt`, `checkpoint`, `checkpointer`, `checkpoint_store` | Pause/resume and persistence contracts |
| `event_sink`, `executor`, `stream` | Events, execution injection, and streaming |
| `receipt`, `run_summary`, `error` | Run evidence, metrics, and typed failures |

## License

MIT. See the repository license files for the governing text.
