# agent-graph-mcp

`agent-graph-mcp` is a three-tool MCP server that compiles bounded declarative workflow specs to the real `agent_graph::AgentGraph` runtime. It keeps the original `graph_create`, `graph_execute`, and `graph_status` names and accepts the original V1 graph shape.

## Capability boundary

- Storage is process-local and reported as `volatile`.
- Normal execution is synchronous. `graph_execute {"action":"start","wait":"accepted"}` starts a background run that can be inspected and cancellation-requested by run ID.
- Cancellation is checked at node boundaries. It does not interrupt an in-flight provider request.
- Receipts and bundles provide redacted digest-chain integrity checking only (`integrity_verified`). They do not prove an external model call occurred and are not complete replay.
- Resume, durable restart recovery, approval/HITL, arbitrary Hermes tools, shell, filesystem, user-selected provider URLs/headers, and secret/environment references are unsupported.

## Declarative runtime

Supported nodes are `llm`, `passthrough`, ordered `router`, `state_transform`, and `join`. Multiple outgoing edges use the core bounded parallel scheduler. Built-in reducers are `last_write_wins`, `append`, `add`, and `merge`. Cycles are allowed only within the graph's bounded `max_iterations`.

V1 router maps remain accepted. Because JSON object order was never a stable routing contract, they normalize in lexicographic pattern order. V2 routers use an explicit ordered `config.rules` array with first-match semantics and an explicit `default` target list.

Safe transform operations are `set`, `copy`, `delete`, `increment`, `append`, `merge_object`, `select`, `compare`, and bounded placeholder `format`. There is no eval, regex, code, shell, filesystem, or network transform.

## Actions and resources

- `graph_create`: default/create, `validate`, `delete`, or instantiate a built-in template.
- `graph_execute`: synchronous/default start, accepted start, `cancel`, and offline `verify_replay`.
- `graph_status`: legacy empty status or `server`, `graph`, `run`, `events`, `receipt`, `bundle`, and `templates` resources.

Executable built-ins are `plan_critique_refine@1` and `parallel_council@1`. Templates requiring external actions or correlation-bound resume are listed as unavailable with a reason.

## Verification

```bash
cargo fmt --check
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
```
