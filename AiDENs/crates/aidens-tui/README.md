# aidens-tui

Terminal UI for observing the AiDENs autonomous loop.

## Overview

This crate provides a ratatui-based terminal user interface that renders the
live state of the [`aidens-autonomous`](../aidens-autonomous) loop in four
panels:

| Panel | Content |
|---|---|
| **Memory Stats** | Knowledge-base statistics (fact count, chunk count, edge count) polled via HTTP from the semantic-memory server. |
| **Loop State** | Live `LoopState` snapshot: iteration, gaps detected, tasks generated/completed/failed, facts captured/rejected, safe-mode status. |
| **Queue** | Read-only view of the daemon queue (pending, leased, completed, cancelled jobs). |
| **Activity Log** | Scrollable log of recent loop events. |

## Keyboard Controls

| Key | Action |
|---|---|
| `q` | Quit the TUI |
| `p` | Pause/resume HTTP polling |
| `s` | Toggle safe mode on the queue |

## Usage

```bash
cargo run -p aidens-tui -- \
    --memory-dir ~/.hermes/semantic-memory.db \
    --queue-dir /tmp/aidens-queue \
    --ollama-url http://127.0.0.1:11434 \
    --ollama-model granite4.1:3b \
    --http-base-url http://127.0.0.1:1738
```

All flags are optional; defaults match the `LoopConfig` defaults.

## Architecture

```
┌─────────────────────────────────────────────────┐
│                  aidens-tui                      │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐      │
│  │  Stats   │  │  Loop    │  │  Queue   │      │
│  │  Panel   │  │  Panel   │  │  Panel   │      │
│  └──────────┘  └──────────┘  └──────────┘      │
│  ┌─────────────────────────────────────────┐    │
│  │            Activity Log                  │    │
│  └─────────────────────────────────────────┘    │
└──────────────────┬──────────────────────────────┘
                   │ shares Arc<Mutex<LoopState>>
                   ▼
┌─────────────────────────────────────────────────┐
│              aidens-autonomous                   │
│   AutonomousLoop::run() (async, background)     │
└─────────────────────────────────────────────────┘
```

The TUI shares `Arc<Mutex<LoopState>>` with the `AutonomousLoop`, allowing
real-time observation without blocking the loop.

## Dependencies

- `aidens-autonomous` — loop driver and state types
- `aidens-daemon-kit` — queue controller for queue panel
- `aidens-queue-kit` — queue data structures
- `aidens-contracts` — typed contracts
- `ratatui` — terminal UI framework
- `crossterm` — terminal backend
- `tokio` — async runtime
- `reqwest` — HTTP client for memory stats polling

## License

MIT OR Apache-2.0