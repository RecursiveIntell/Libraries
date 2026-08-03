# agent-graph-mcp

A stdio Model Context Protocol (MCP) server that exposes declarative agent graphs, bounded runs, durable projections, receipts, approvals, and source-witness operations as typed tools.

`agent-graph-mcp` is the control-plane adapter around [`agent-graph`](../agent-graph) and [`llm-pipeline`](../llm-pipeline). It registers graphs, validates their specifications, executes them, and exposes lifecycle evidence. It is not a web-research engine, an HTTP MCP daemon, or an authority system by itself.

> Current crate version: `0.2.0` · Rust 2021 · minimum Rust `1.75` · MIT
>
> This README describes the checked-in implementation. Claims are limited to source-visible behavior and testable commands.

## What it provides

```text
MCP client
    │ stdio transport
    ▼
agent-graph-mcp
    ├── strict CLI parsing and startup validation
    ├── GraphSpec validation / normalization / version digest
    ├── in-memory graph registry
    ├── sync and async run lifecycle
    ├── bounded events, state, checkpoints, receipts
    ├── idempotency protection for repeated requests
    ├── approvals bound to deterministic-local checkpoints
    ├── caller-supplied source-witness capture and authenticated reads
    ├── policy preflight and Mermaid / JSON rendering
    └── optional SQLite-backed durable projections
         │
         ├── agent-graph: execution semantics
         └── llm-pipeline: provider calls and output handling
```

The binary uses `rmcp`’s stdio transport. Logs are written to stderr so the MCP stream remains the protocol channel. The current CLI has no `--http-port` or `--mcp-http-port` option; do not copy an HTTP-service unit file from another MCP server onto this binary.

## Workspace quick start

From `/home/.../Libraries` or another checkout of the workspace:

```bash
cargo check -p agent-graph-mcp --all-targets
cargo test -p agent-graph-mcp
cargo run -p agent-graph-mcp --bin agent-graph-mcp -- --help
```

Run explicitly in ephemeral mode:

```bash
cargo run -p agent-graph-mcp --bin agent-graph-mcp -- \
  --ephemeral \
  --base-url http://127.0.0.1:11434 \
  --model llama3.2:3b
```

The process waits for an MCP client on stdin/stdout. `--base-url` is the provider URL used by LLM nodes, not an MCP listener URL.

## CLI configuration

The parser is deliberately strict: unknown flags, missing values, unsupported URL schemes, empty model values, and invalid flag combinations fail before MCP transport starts.

| Flag | Meaning |
| --- | --- |
| `--base-url <url>` | Provider URL; only `http://` and `https://` are accepted. Default: `http://127.0.0.1:11434`. |
| `--model <name>` | Default model for LLM nodes. Default: `glm-5.2:cloud`. |
| `--api-key <key>` | API key passed to the configured provider path. Avoid shell history and process-list exposure when supplying secrets. |
| `--ephemeral` | Explicit in-memory mode. State and registry data are lost on restart. |
| `--data-dir <path>` | Enable persistent SQLite-backed storage in a private directory. Mutually exclusive with `--ephemeral`. |
| `--integrity-key <path>` | Integrity key file used for durable integrity-protected records. Keep it outside the repository and restrict its permissions. |
| `--checkpoint-db-path <path>` | SQLite checkpoint database path; can also be supplied through `AGENT_GRAPH_CHECKPOINT_DB_PATH`. |
| `--require-integrity-key` | Refuse startup when durable mode has no readable key of at least 32 bytes. Requires `--data-dir` plus a CLI or environment key path. |

Environment variables used by the binary:

- `AGENT_GRAPH_INTEGRITY_KEY_PATH` is the fallback integrity-key path;
- `AGENT_GRAPH_CHECKPOINT_DB_PATH` is the fallback checkpoint database path;
- `RUST_LOG` controls the `tracing_subscriber` filter.

### Durable-mode admission

Use durable mode only after deciding where the data directory, SQLite database, integrity key, and backups belong:

```bash
install -d -m 700 /var/lib/agent-graph-mcp
chmod 600 /secure/path/agent-graph-integrity.key

cargo run -p agent-graph-mcp --bin agent-graph-mcp -- \
  --data-dir /var/lib/agent-graph-mcp \
  --integrity-key /secure/path/agent-graph-integrity.key \
  --require-integrity-key \
  --checkpoint-db-path /var/lib/agent-graph-mcp/checkpoints.sqlite \
  --base-url http://127.0.0.1:11434 \
  --model llama3.2:3b
```

The example paths are illustrative. Do not put real keys, provider credentials, or private source content into README files, shell history, or issue reports.

## GraphSpec

Graphs are registered from JSON. The server normalizes accepted specifications to spec version `2`, computes a graph version/digest, and returns structured success or error output.

A minimal declarative graph:

```json
{
  "spec_version": "2",
  "name": "plan-and-refine",
  "entry": "plan",
  "output_key": "final",
  "max_iterations": 8,
  "nodes": [
    {
      "id": "plan",
      "type": "llm",
      "prompt": "Create a concise plan for: {input}",
      "config": {"output_key": "draft"}
    },
    {
      "id": "refine",
      "type": "llm",
      "prompt": "Refine this draft: {input}",
      "config": {"input_key": "draft", "output_key": "final"}
    }
  ],
  "edges": [
    {"from": "plan", "to": "refine"},
    {"from": "refine", "to": "END"}
  ]
}
```

Supported executable node classes are:

- `llm` — invoke the configured `llm-pipeline` path;
- `router` — select routes from state/configured rules;
- `passthrough` — preserve or forward state;
- `state_transform` — apply declared local state operations;
- `join` — combine branch values;
- `parallel` — execute declared branches with bounded parallelism;
- `subgraph` — invoke another registered graph;
- `human_approval` — represent an approval checkpoint under the server’s approval contract.

`external`, `tool`, and `loop` are reserved classifications but are rejected as unsupported executable node types by the local runtime. A node being representable in the enum is not evidence that it can run.

### Resource bounds

The specification validator enforces finite limits, including:

- 64 registered graphs;
- 64 KiB per graph specification and input;
- 128 nodes;
- 512 edges;
- 64 maximum iterations after normalization;
- 2 MiB maximum serialized state;
- 128 KiB maximum output;
- bounded parallelism, normalized to a finite default.

Treat these as runtime admission limits, not tuning suggestions.

## MCP tool surface

The server exposes typed JSON schemas generated from Rust parameter structs. The main tool groups are:

### Graph lifecycle

- `graph_create` — create, validate, or delete using a spec or template; supports overwrite and idempotency keys.
- `graph_list` — list registered graphs with optional query and limit.
- `graph_inspect` — retrieve a graph’s topology, version, nodes, edges, and Mermaid representation.
- `graph_delete` — delete a registered graph by name.
- `graph_render` — render Mermaid or JSON topology.
- `graph_policy_check` — preflight a graph against policy before execution.

### Runs and evidence

- `graph_execute` — execute synchronously or request async behavior.
- `graph_execute`: synchronous/default start or asynchronous start with `mode:"async"`.
- `graph_run_start` — start an async run with optional budgets and an intentional pre-execution checkpoint.
- `graph_run_wait` — wait for completion.
- `graph_run_get` — inspect status, budget use, and pending approvals.
- `graph_run_state` — read projected state or a JSON pointer.
- `graph_run_events` — read bounded events from a cursor.
- `graph_run_receipt` — fetch the canonical run receipt.
- `graph_run_cancel` — cancel a running execution.
- `graph_run_checkpoint` — read a durable deterministic-local checkpoint.
- `graph_run_resume` — consume a checkpoint and resume exactly once.

### Approvals and witnesses

- `graph_approval_request`, `graph_approval_list`, `graph_approval_get`, `graph_approval_decide` manage approvals bound to checkpoint material.
- `graph_source_witness_capture` stores caller-supplied source content with locator and authority metadata; it does not fetch the locator.
- `graph_source_witness_get` reads a witness after authenticating its local receipt.

Approval audience labels are metadata, not authority. A claimed actor label does not grant permission. Decisions are constrained by the stored approval and checkpoint contract. Durable approval is supported only as a SQLite-backed decision bound to a persisted checkpoint; ephemeral mode fails closed for approval operations.

All tool responses use a structured envelope with `ok`, optional `status`, `data`, `error`, `error_code`, and applicable graph/version/run identifiers.

## Templates

`graph_template_list` currently distinguishes executable templates from unavailable ones:

| Template | Version | State | Boundary |
| --- | ---: | --- | --- |
| `council_deliberation` | 2 | executable | Three parallel analyst branches plus join/synthesis |
| `parallel_council` | 1 | executable | Optimist/skeptic debate plus judge |
| `plan_critique_refine` | 1 | executable | Sequential plan → critique → refine |
| `analysis_pipeline` | 1 | executable | LLM knowledge synthesis with validation/correction loop; not web research |
| `classifier_router` | 2 | executable | Classify bug/feature/question while preserving original input |
| `approval_gated_action` | — | unavailable | Requires authenticated human approval subsystem |
| `research_pipeline` | — | unavailable alias | Renamed to `analysis_pipeline`; true web research needs source-witness and external-tool integration |
| `map_reduce` | — | unavailable | Dynamic branch count is not implemented by the current template catalog |

Instantiate, inspect, policy-check, and execute a template as separate gates. A template being listed as executable means it passed the crate’s declared semantic checks; it does not mean a provider, source, or external side effect was verified for your environment.

## Lifecycle pattern

For a durable, auditable run, use this order:

```text
1. graph_template_list / graph_create(action=validate)
2. graph_create(action=create, idempotency_key=...)
3. graph_policy_check
4. graph_run_start(checkpoint=true, budgets=...)
5. graph_run_checkpoint / graph_run_get
6. explicit approval if the graph requires it
7. graph_run_resume(checkpoint_id=...)
8. graph_run_wait
9. graph_run_events and graph_run_receipt
```

Use an idempotency key when the caller may retry a request. Reusing a key with different request material returns an idempotency conflict rather than silently executing a different operation.

A checkpoint contains graph/version identity, state and state digest, budget counters, dependency summaries, event cursors, and a checkpoint digest. Consumed checkpoints cannot be resumed again. Durable integrity operations require the configured integrity key; persistence failure is surfaced instead of advertising resumability that was not recorded.

## Persistence modes

| Mode | Registry / run data | Restart behavior | Use when |
| --- | --- | --- | --- |
| `--ephemeral` | Process memory | Data is lost | Local smoke tests and disposable experiments |
| `--data-dir <path>` | SQLite-backed projections plus private directory checks | Data can be recovered subject to the store/key contract | You need durable lifecycle evidence |

The server also accepts an explicit checkpoint database path. Keep graph data, checkpoint data, integrity keys, logs, and backups under separately governed retention policies.

## Security and trust boundaries

- CLI parsing fails closed on unknown flags and invalid URLs.
- Durable directories are checked for private ownership/permissions and use an owner lock.
- Integrity-protected records use digests/HMAC material from the configured key path.
- Witness capture stores caller-supplied bytes; it never dereferences a URL or filesystem locator.
- Prompt text is stored as a digest in approval records rather than returned as raw approval content.
- `--api-key` exists for provider authentication, but this README intentionally never contains a real secret.
- MCP clients and providers remain external trust boundaries. Tool schemas are not authorization.

This server does not provide a general-purpose sandbox for arbitrary tools or external effects. Unsupported node classes are rejected before registration/execution.

## Verification

Run checks from the workspace root:

```bash
cargo fmt --check -p agent-graph-mcp
cargo check -p agent-graph-mcp --all-targets
cargo test -p agent-graph-mcp
cargo run -p agent-graph-mcp --bin agent-graph-mcp -- --help
```

Focused integration targets present in this crate include:

```bash
cargo test -p agent-graph-mcp --test mcp_integration
cargo test -p agent-graph-mcp --test lifecycle
cargo test -p agent-graph-mcp --test process_boundary
cargo test -p agent-graph-mcp --test proxy_stdio
```

A successful build or test run proves the checked-in path under that command. It does not prove provider availability, MCP-client compatibility beyond the exercised transport, authority deployment, or production durability.

## Troubleshooting

### The process exits before MCP initialization

Run `cargo run -p agent-graph-mcp --bin agent-graph-mcp -- --help` and check every flag. The parser rejects unknown flags and missing values before transport starts.

### Durable operations return `INTEGRITY_KEY_REQUIRED`

Supply `--data-dir` plus a readable integrity key, or set `AGENT_GRAPH_INTEGRITY_KEY_PATH`. If the deployment requires the key, add `--require-integrity-key` so startup fails early rather than entering a weaker mode.

### A graph validates but does not execute

Validation checks the declarative contract. Execution additionally needs a reachable provider for `llm` nodes, a model available to that provider, compatible node configuration, and a budget that permits the run. Inspect the structured error, events, and receipt.

### A “research” template appears unavailable

That is intentional. The current executable template is `analysis_pipeline`, which is model-knowledge synthesis and explicitly does not perform web research or source verification.

## License

MIT. See the repository license files for the governing text.
