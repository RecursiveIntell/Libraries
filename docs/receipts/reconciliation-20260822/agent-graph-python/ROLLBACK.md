# Rollback — agent-graph-python candidate

Revert only:

- `agent-graph-python/python/agent_graph/__init__.py`
- `agent-graph-python/README.md`
- `llm-pipeline-python/Cargo.toml`
- `stack-monitor/src/live_ipc.rs`
- `stack-monitor/src/transport.rs`
- `docs/receipts/reconciliation-20260822/agent-graph-python/`

Do not reset, clean, delete `activity.db*`, alter `agent-graph-python/src/observability.rs`, touch unrelated crate changes, regenerate the release receipt, or install the wheel into the active environment.

If full workspace tests or Clippy fail after the candidate edit, quarantine this candidate and retain the failure log; do not weaken the workspace gate or force a release claim.
