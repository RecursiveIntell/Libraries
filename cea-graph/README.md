# cea-graph

Read-only JSON stdin/stdout bridge for the **real CEA causal graph**.

## Purpose

`cea-graph` exposes the canonical causal edit attribution graph (from
`cea-core` / `cea-store` / `cea-sqlite`, populated by forge-engine) as a
simple CLI. It is the **real causal lane** — distinct from `cea-bridge`,
which records synthetic Hermes tool telemetry and is deliberately
quarantined.

Every response carries:

```json
{
  "evidence_kind": "causal_graph",
  "causal_claim": true
}
```

## Commands

```
cea-graph predict            --db <path> [--version <id>] < signatures.json
cea-graph graph-stats        --db <path> [--version <id>]
cea-graph inspect-signature  --db <path> [--version <id>] < signature.json
```

- `predict` — predict edit risk for a list of `EditOpSignature`s
- `graph-stats` — coverage summary (nodes, edges, mean confidence)
- `inspect-signature` — a cause node's outgoing causal edges

## Database access

Reads forge-engine databases (e.g. `~/.recall/forge/forge.db`) **read-only**
via `SqliteCeaStoreConn` — the raw-connection pattern from forge-engine's own
`cea/store.rs`. This deliberately bypasses `SqliteCeaStore::open()`'s schema
gate because forge DBs carry `user_version = 5` (newer than cea-sqlite's
current 2). Read-only flag guarantees no mutation.

Missing DB → cold-start neutral prediction, never an error (fail open).

## Example

```bash
echo '{"signatures":[{"op_kind":"replace","anchor_kind":"range","lines_added":1,"lines_removed":0,"context_hash":"","file_extension":"rs","scope_tag":"unknown","op_index":0,"file_index":0}],"db_path":"~/.recall/forge/forge.db"}' | cea-graph predict
```

Cold graphs return neutral predictions (`predicted_correctness: 0.5`,
`confidence: 0.0`, no risk flags). Risk flags need ~40+ observations of a
signature before confidence crosses 0.65.

## Build / install

```bash
cd ~/Coding/Libraries/cea-graph
cargo build
cargo test
cargo install --path . --locked --force
```

## MCP server

Wrapped for Hermes by `~/.local/lib/cea-graph-mcp/cea-graph-mcp.py`
(registered as MCP server `cea_graph`). Tools: `cea_graph_predict`,
`cea_graph_stats`, `cea_graph_inspect_signature`.
