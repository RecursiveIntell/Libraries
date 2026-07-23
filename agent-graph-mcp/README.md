# agent-graph-mcp

`agent-graph-mcp` is an MCP server that compiles bounded declarative workflow specs to the real `agent_graph::AgentGraph` runtime. It keeps the original `graph_create`, `graph_execute`, and `graph_status` names and accepts the original V1 graph shape.

## Capability boundary

- Runs are process-local and reported as `volatile` while active. With `--data-dir`, terminal projections and explicitly requested deterministic pre-execution checkpoints are persisted to SQLite; uncheckpointed active rows still become `interrupted_non_resumable` after restart.
- Normal execution is synchronous. `graph_execute {"mode":"async"}` starts a background run that can be inspected and cancellation-requested by run ID.
- Cancellation is observed while an LLM future is in flight by dropping the local provider future (best effort). The underlying provider request may still be in flight; terminal cancellation is recorded when the graph observes the interruption.
- With SQLite enabled, terminal write failures remain visible in the run record as `storage_class: "volatile"` with `persistence_error`; no failed write is reported as durable.
- Durable integrity-sensitive records require `AGENT_GRAPH_INTEGRITY_KEY_PATH` to name a readable external file containing at least 32 secret bytes. The key is never written to SQLite, receipts, or bundles. Without it, checkpoint/resume, durable approval, terminal receipt, and source-witness operations fail closed with `INTEGRITY_KEY_REQUIRED`.
- `graph_run_start {"checkpoint":true}` is an intentional pre-execution checkpoint. `graph_run_checkpoint` reads it and `graph_run_resume` reserves execution capacity before atomically consuming it once. Resume is available only for a linear chain of deterministic `passthrough` and local `state_transform` nodes, with SQLite-bound state, budget, graph-version, dependency, cursor, and HMAC-SHA256 checkpoint authentication. This is deterministic local resume, not generic replay.
- Unordered parallel writes to the same state key are rejected unless `GraphSpec.reducers` declares a reducer; sequential repeated writes remain allowed.
- `evidence_required` requires durable SQLite-backed local witness IDs and bounded UTF-8 spans. Witness capture stores caller-supplied content only; locators are never fetched, and source authority is never independently verified.
- `graph_source_witness_capture` and `graph_source_witness_get` provide exact-ID local capture receipts authenticated with HMAC-SHA256. Capture is unavailable without `--data-dir` and the external integrity key.
- Terminal receipts, checkpoints, approvals, and witnesses use HMAC-SHA256 authentication; their redacted bundles remain `integrity_only`. They do not prove an external model call occurred and are not complete replay.
- `graph_run_start` accepts optional positive-integer `max_wall_clock_ms` and `max_nodes` budgets. Requested budgets and observed counters are included in terminal projections and receipts. `max_llm_calls` is rejected with `INVALID_BUDGETS` because this permitted runtime path has no real LLM invocation hook.
- LLM, router, join, parallel, loop, subgraph, external/tool, provider, uncaptured source-witness, and generic replay behavior are excluded from resume. Durable approval is supported only as a SQLite-backed decision over an already-created deterministic-local checkpoint; it cannot execute HumanApproval nodes, arbitrary Hermes tools, shell, filesystem, provider actions, or secret/environment references.

## Declarative runtime

Supported nodes are `llm`, `passthrough`, ordered `router`, `state_transform`, and `join`. Multiple outgoing edges use the core bounded parallel scheduler. Built-in reducers are `last_write_wins`, `append`, `add`, and `merge`. Cycles are allowed only within the graph's bounded `max_iterations`.

V1 router maps remain accepted. Because JSON object order was never a stable routing contract, they normalize in lexicographic pattern order. V2 routers use an explicit ordered `config.rules` array with first-match semantics and an explicit `default` target list.

Safe transform operations are `set`, `copy`, `delete`, `increment`, `append`, `merge_object`, `select`, `compare`, and bounded placeholder `format`. There is no eval, regex, code, shell, filesystem, or network transform.

## Actions and resources

- `graph_create`: default/create, `validate`, `delete`, or instantiate a built-in template.
- `graph_execute`: synchronous/default start or asynchronous start with `mode:"async"`.
- `graph_status`: legacy empty status or `server`, `graph`, `run`, `events`, `receipt`, `bundle`, and `templates` resources.

Executable built-ins are `plan_critique_refine@1` and `parallel_council@1`. Templates requiring external actions or correlation-bound resume are listed as unavailable with a reason.

## Verification

```bash
cargo fmt --check
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
```
