# Candidate state — agent-graph-python — 2026-08-22

## Scope

This is a bounded candidate, not a full Libraries release admission.

Changed source/docs:

- `agent-graph-python/python/agent_graph/__init__.py`
- `agent-graph-python/README.md`
- `stack-monitor/src/live_ipc.rs`

The `agent-graph-python` Rust binding and optional observability source were inspected but not modified. The `stack-monitor` fix treats both `TimedOut` and `WouldBlock` as retryable socket reads; the prior full workspace run failed only `stack-monitor --lib` with three concurrency/timing-sensitive tests.

## Verified

- `stack-monitor` live IPC test: 5 repeated isolated passes.
- `stack-monitor` library suite: 23 passed.
- `stack-monitor` strict all-feature Clippy: pass.
- `agent-graph-python` Cargo test target: exit 0, but 0 tests discovered.
- `agent-graph-python` strict all-feature Clippy: pass.
- Isolated CPython 3.14 wheel build with `observability`: pass.
- Isolated import/state/graph/observability smoke: pass.
- `semantic-memory` now forwards `fib-quant-codec` to `poly-kv/fibquant-adapter`; Poly-KV, FibQuant, and combined offline feature checks pass.
- `llm-pipeline/examples/anthropic_budget.rs` now propagates a missing API key with `?`; its all-feature Clippy/test gate passes.

## Remaining proof debt

- Python test coverage is absent from the candidate; the wheel smoke is the current behavioral proof.
- The candidate admits the previously untracked observability source and monitoring crates as one owner-scoped slice; the optional semantic-memory bridge and root desktop workspace remain outside this candidate.
- Clean source-binding release evidence is still not admissible from the mixed parent worktree.

## Blocked

The root release recorder refuses the mixed dirty tree by default. The historical release receipt and dashboard remain stale for current HEAD. No release claim, install, commit, or deployment is admitted.
