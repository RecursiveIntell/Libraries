# pilot-bridge

JSON stdin/stdout bridge for **forge-pilot closed-loop evaluation**.

## Purpose

`pilot-bridge` exposes forge-pilot's OODA loop capabilities as a simple CLI
for Hermes integration. It runs REAL forge-engine evidence flows — this is
the real causal lane, distinct from `cea-bridge`'s synthetic telemetry.

## ⚠️ Scale Guard (read this first)

`bootstrap`, `observe`, and `evaluate` scan and chunk source files **in
memory**. A monorepo scan (Libraries workspace, ~8400 files) once exhausted
host RAM (5.7GB) and crashed the machine.

The bridge now refuses to run when the workspace has more than
`max_source_files` (default **2000**) source files, and it counts files
BEFORE any scanning begins. Use it on ONE small crate at a time.

## Commands

```
pilot-bridge status            < request.json
pilot-bridge observe           < request.json
pilot-bridge bootstrap         < request.json
pilot-bridge evaluate          < request.json
pilot-bridge receipt-verify    < request.json
```

Request fields (all optional):

| Field | Meaning | Default |
|---|---|---|
| `workspace_path` | Workspace/crate to scan | current dir |
| `memory_dir` | Pilot semantic-memory dir | `~/.recall/pilot-memory` |
| `forge_db` | Forge engine DB | `~/.recall/forge/forge.db` |
| `namespace` | Scope namespace | `default` |
| `max_iterations` | Loop iterations cap | `1` |
| `time_budget_secs` | Wall-clock budget | config default |
| `max_source_files` | Scale guard cap | `2000` |
| `receipt` | Receipt object for receipt-verify | — |

## Example (bounded evaluate)

```bash
echo '{"workspace_path":"/home/sikmindz/Coding/Libraries/cea-bridge","forge_db":"/home/sikmindz/.recall/forge/forge.db","memory_dir":"/tmp/pilot-memory-test","namespace":"my-ns","max_iterations":1,"time_budget_secs":30}' | pilot-bridge evaluate
```

The loop halts honestly under governance/authority gates
(`halt_reason: governance_authority_insufficient` when no governance claims
exist). That is correct behavior — it never fabricates actions.

## Receipt verification

Two modes, selected by the receipt id shape:

- **digest** — 64-hex id → recompute canonical BLAKE3 over the JSON and compare
- **structural** — UUID id (forge-pilot loop receipts) → required fields +
  RFC3339 timestamps

```bash
echo "{\"receipt\":$(cat /tmp/pilot-receipt.json)}" | pilot-bridge receipt-verify
```

## Build / install

```bash
cd ~/Coding/Libraries/pilot-bridge
cargo build
cargo test
cargo install --path . --locked --force
```

## MCP server

Wrapped for Hermes by `~/.local/lib/pilot-bridge-mcp/pilot-bridge-mcp.py`
(registered as MCP server `pilot_bridge`). Tools: `pilot_status`,
`pilot_observe`, `pilot_bootstrap`, `pilot_evaluate`,
`pilot_receipt_verify`. The MCP server passes `max_source_files: 2000` by
default on scan commands.
