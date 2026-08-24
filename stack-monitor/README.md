# stack-monitor

Experimental local-first execution observability for programs using the RecursiveIntell stack.

This crate now contains the first executable vertical slice:

```text
stack adapters → versioned observation envelope → bounded try-send queue
             → collector worker → SQLite observations table
```

It is **not yet** a Tauri application or cross-platform transport. The current implementation
proves the in-process contract, Linux Unix-socket boundary, and launch-managed collector
lifecycle while keeping the older synchronous `ActivityStore` API as a compatibility path.

## Implemented pieces

- `stack-observation` — dependency-light, versioned `ObservationEnvelope` contract.
- `MonitorClient` — bounded, non-blocking in-process producer handle.
- `stack-observation` global sink — installing a collector enables automatic metadata-only emission from core owners that dispatch global events.
- `ipc` — Unix-domain-socket producer/collector transport with bounded length-delimited frames.
- `live_ipc` — read-only Unix live socket for cursor-bearing `LiveEvent` frames.
- `start_collector()` — collector worker that owns normalized observation writes.
- `start_unix_collector()` / `start_unix_client()` — cross-process Linux transport APIs.
- `stack-monitor-collector` — launch-managed daemon binary with SIGINT/SIGTERM shutdown,
  private socket/database directories, producer socket, and separate live-event socket.
- `ActivityStore` — legacy activity rows plus normalized `observations` rows, schema history,
  retention, privacy-aware persistence, streaming export, typed historical queries, and
  normalized observation counts.
- `LiveHub` / `LiveSubscription` — bounded cursor-based live fan-out with explicit lag signals.
- `ProjectionService` — serializable timeline and health projections for desktop/UI consumers.
- `stack-monitor-desktop` — Tauri v2 read-side shell with timeline, trace waterfall, and Agent Graph tabs,
  synchronized live health counters, privacy/cost inspector fields, JSONL export, polling fallback,
  and live `observation-live` event wiring.
- `LlmPipelineObservationHandler` — metadata-only, non-blocking pipeline adapter.
- `AgentGraphObservationSink` — metadata-only, non-blocking graph adapter behind
  `agent-graph-bridge`.
- `ToolObservationSink` — canonical `llm-tool-runtime::ToolReceiptSink` bridge behind
  `tool-runtime-bridge`.
- `EmbedderObservationWrapper` — public semantic-memory `Embedder` wrapper behind
  `semantic-memory-bridge`; records duration/model/dimensions without input/vector content.
- `SemanticMemoryReceiptObservationSink` — read-only public LLM receipt metadata adapter;
  excludes raw receipt bodies and performs no semantic-memory writes.
- `TracingActivityLayer` — deprecated compatibility fallback for selected tracing targets.
- Legacy synchronous `ActivityStore::record`, `record_batch`, and `export_jsonl` are deprecated;
  use collector-backed observation ingestion and `export_observations_jsonl_to` instead.

## Usage

```rust
use stack_monitor::{start_collector, ActivityStore};

let store = ActivityStore::open("activity.db")?;
let (monitor, collector) = start_collector(store.clone(), 1024);

// Pass `monitor` to a structured adapter. `try_emit` never waits for SQLite.
// The collector is the only writer for normalized observations.

// On shutdown, drain accepted events and join the collector.
let stats = collector.shutdown();
println!("persisted={}, dropped={}", stats.persisted, stats.dropped);
# Ok::<(), Box<dyn std::error::Error>>(())
```

For `llm-pipeline`:

```rust
use stack_monitor::LlmPipelineObservationHandler;

let handler = LlmPipelineObservationHandler::new(monitor.clone(), "my-process");
// Attach `handler` to the pipeline's EventHandler seam.
```

For Agent Graph, enable `agent-graph-bridge` and attach
`AgentGraphObservationSink::new(monitor.clone(), "my-process")` to the graph's
`EventSink` seam.

## Privacy boundary

Normalized observations default to metadata-only capture. Token text, prompts, responses,
tool payloads, and interrupt payloads are not emitted by the new adapters. Missing trace,
attempt, trial, and request identifiers remain missing; the monitor does not manufacture
correlation.

The product must not claim access to hidden model weights, hidden activations, private
reasoning, or chain-of-thought. It reports observable execution events from instrumented
stack components.

## Validation

```bash
cargo fmt --all -- --check
cargo test -p stack-observation
cargo test -p stack-monitor
cargo test -p stack-monitor --all-features
cargo clippy -p stack-observation --all-targets -- -D warnings
cargo clippy -p stack-monitor --all-targets --all-features -- -D warnings
```

The workspace may emit an unrelated warning when a non-root package declares ignored
Cargo profiles. That warning does not affect this crate's gates.

## Performance baseline

Run the release-mode producer/collector baseline with:

```bash
cargo run -p stack-monitor --example transport_benchmark --release
```

The measured result is preserved in `PERFORMANCE_BASELINE.md`. The three-run provisional budget proposal
is preserved in `PERFORMANCE_BUDGETS.md`; neither artifact makes a universal performance claim.

## Release validation

Run the non-activating release validator with:

```bash
stack-monitor-desktop/scripts/validate-release.sh
```

It verifies tests, strict Clippy, release binaries, Tauri build output, service syntax, and
required artifact paths. It does not install, enable, publish, or start the user service.

The staged user installer is:

```bash
stack-monitor-desktop/scripts/install-user.sh       # dry-run only
stack-monitor-desktop/scripts/install-user.sh --apply
stack-monitor-desktop/scripts/install-user.sh --activate
```

`--activate` was explicitly approved and run for this host. The collector service is enabled and active;
a real metadata-only health event traversed the 0600 producer socket and persisted to the installed SQLite database.

## Remaining implementation phases

1. User-service packaging, operational health/metrics, and cross-platform named-pipe abstraction.
2. Full transactional migration history, retention, redaction, and export policy enforcement.
3. Enriched upstream model/provider/request/timing metadata.
4. Tool-runtime, semantic-memory, embedding, MCP, and Python adapters.
5. Failure-injection/performance hardening, packaging, and release closure.
6. Legacy synchronous adapter retirement and remaining MCP/Python automatic integration wiring.
