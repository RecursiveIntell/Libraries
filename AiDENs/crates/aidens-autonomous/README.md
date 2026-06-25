# aidens-autonomous

Autonomous gap detection and task generation for the closed-loop self-learning AI.

## Overview

This crate implements the full autonomous learning loop for AiDENs:

```
detect → enqueue → execute → capture → evaluate
```

The loop scans the semantic-memory knowledge base for structural gaps, generates
remediation jobs, executes them through the plan-act-verify cycle, captures
results as new facts, and evaluates whether those facts should be promoted,
quarantined, or rejected.

## Modules

| Module | Purpose |
|---|---|
| `gap_detector` | Scans the semantic-memory KB for missing context, missing links, and stale facts via HTTP calls to the warm semantic-memory server. |
| `task_generator` | Converts detected gaps into `JobV1` entries and enqueues them via a `DaemonControllerV1` for the runner to pick up. |
| `executor` | Executes queued jobs through the plan-act-verify loop and returns `ExecutionResult` values. |
| `capture` | Stores execution outputs as facts in semantic memory with deduplication and graph-edge linkage. |
| `evaluation` | Evaluates captured facts for promotion, quarantine, or rejection. |
| `loop_driver` | Ties everything together into a continuous detect → enqueue → execute → capture → evaluate loop. |

## Key Types

- `AutonomousLoop` — the main loop driver; build with `from_config()` and call `run().await`
- `LoopConfig` — configuration (paths, provider URL, model, iteration limits, sleep intervals)
- `LoopState` — live state snapshot (iteration count, gaps detected, tasks completed/failed, facts captured/rejected, safe mode)

## Usage

```rust
use aidens_autonomous::{AutonomousLoop, LoopConfig};
use std::path::PathBuf;

let config = LoopConfig {
    memory_dir: PathBuf::from("~/.hermes/semantic-memory.db"),
    queue_dir: PathBuf::from("/tmp/aidens-queue"),
    ollama_url: "http://127.0.0.1:11434".to_string(),
    ollama_model: "granite4.1:3b".to_string(),
    http_base_url: "http://127.0.0.1:1738".to_string(),
    max_iterations: 0,        // 0 = infinite
    sleep_between_iterations_ms: 1000,
    ..Default::default()
};

let mut loop_driver = AutonomousLoop::from_config(config)?;
loop_driver.run().await?;
```

## CLI

The `aidens autonomous` subcommand wraps this crate in a headless runner:

```bash
cargo run -p aidens-cli -- autonomous \
    --memory-dir ~/.hermes/semantic-memory.db \
    --queue-dir /tmp/aidens-queue \
    --ollama-url http://127.0.0.1:11434 \
    --ollama-model granite4.1:3b \
    --http-base-url http://127.0.0.1:1738 \
    --max-iterations 0 \
    --sleep-ms 60
```

State is printed to stderr every 5 seconds. For a full terminal UI, use
`aidens-tui` instead.

## Dependencies

- `aidens-daemon-kit` — queue controller for job lifecycle
- `aidens-queue-kit` — queue data structures
- `aidens-contracts` — `JobV1` and other typed contracts
- `aidens-memory-kit` — canonical memory adapter
- `aidens-runner` — plan-act-verify execution
- `aidens-governance-kit` — governance checks
- `semantic-memory` — knowledge base access
- `reqwest` — HTTP client for semantic-memory server
- `tokio` — async runtime

## License

MIT OR Apache-2.0