# Libraries Reference (v3)

Complete architectural reference for every crate and package in this repository. Describes current state from source, not aspirational future state.

---

## Table of Contents

1. [semantic-memory](#semantic-memory) - Hybrid semantic search with SQLite, FTS5, and HNSW
2. [knowledge-runtime](#knowledge-runtime) - Bounded orchestration scaffold for semantic-memory
3. [agent-graph](#agent-graph) - Graph-based agent orchestration (LangGraph for Rust)
4. [LLM-Pipeline](#llm-pipeline) - LLM payload execution: prompts, backends, parsing, retry, streaming
5. [llm-output-parser](#llm-output-parser) - Standalone LLM response parser library
6. [job-queue](#job-queue) - SQLite-backed background job queue
7. [AI-Batch-Queue](#ai-batch-queue) - Model-aware batch processing with ETA estimation
8. [Ollama-Vision-RS](#ollama-vision-rs) - Vision model toolkit for tagging and captioning
9. [ComfyUI-RS](#comfyui-rs) - Async Rust client for ComfyUI
10. [Tauri-Queue](#tauri-queue) - Tauri integration bridge for job-queue
11. [Tauri-React-Hooks](#tauri-react-hooks) - React hooks for Tauri 2 apps
12. [Primitives](#primitives) - 10 crates for patch validation, execution, and causal attribution
13. [living-memory (semantic-memory-forge)](#living-memory) - Causal edit attribution and structured patch evaluation engine

---

## Dependency Graph

```
semantic-memory
    +-- knowledge-runtime (depends on semantic-memory)
    +-- living-memory/semantic-memory-forge (depends on semantic-memory + Primitives)

llm-output-parser (.parser-lib)
    +-- LLM-Pipeline (depends on llm-output-parser)
    +-- Ollama-Vision-RS (depends on llm-output-parser)

agent-graph (standalone, pairs with LLM-Pipeline via Payload trait)

job-queue (standalone)
    +-- Tauri-Queue (depends on job-queue + tauri)

AI-Batch-Queue (standalone, depends on tauri)

ComfyUI-RS (standalone)

Tauri-React-Hooks (npm package, standalone)

Primitives (10 internal crates, self-contained DAG)
    +-- living-memory/semantic-memory-forge (depends on all Primitives + semantic-memory)
```

---

## Package Inventory

| Directory | Package | Version | Language | Role |
|-----------|---------|---------|----------|------|
| `semantic-memory/` | `semantic-memory` | 0.5.0 | Rust | SQLite + FTS5 + HNSW memory and retrieval |
| `knowledge-runtime/` | `knowledge-runtime` | 0.1.0 | Rust | Orchestration scaffold for semantic-memory |
| `agent-graph/` | `agent-graph` | 0.2.0 | Rust | Graph orchestrator for agent workflows |
| `LLM-Pipeline/` | `llm-pipeline` | 0.2.0 | Rust | LLM payload execution, streaming, retry, parsing |
| `.parser-lib/` | `llm-output-parser` | 0.2.0 | Rust | Reusable parser for messy model output |
| `job-queue/` | `job-queue` | 0.2.0 | Rust | Framework-agnostic persistent background queue |
| `AI-Batch-Queue/` | `ai-batch-queue` | 0.2.0 | Rust | Model-aware in-memory batch queue for AI workloads |
| `Ollama-Vision-RS/` | `ollama-vision` | 0.2.0 | Rust | Vision tagging and captioning with robust parsing |
| `ComfyUI-RS/` | `comfyui-rs` | 0.2.0 | Rust | Typed ComfyUI REST/WebSocket client |
| `Tauri-Queue/` | `tauri-queue` | 0.3.0 | Rust | Tauri bridge for job-queue |
| `Tauri-React-Hooks/` | `@tauri-hooks/core` | 0.1.0 | TypeScript | React hooks for Tauri events, commands, streams |
| `Primitives/` (10 crates) | various | 0.1.0 | Rust | Patch validation, execution, causal attribution |
| `living-memory/living-memory/` | `semantic-memory-forge` | 0.2.0 | Rust | Causal edit attribution and patch evaluation engine |

---

## semantic-memory

**Crate:** `semantic-memory` | **Version:** 0.5.0 | **License:** MIT | **MSRV:** 1.75

Hybrid semantic search engine combining SQLite (FTS5 + f32 embeddings) with optional HNSW approximate nearest-neighbor acceleration. Designed for AI agents needing local, searchable, causal knowledge management without external vector databases.

### Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| `rusqlite` | 0.32 (bundled, blob) | SQLite database |
| `reqwest` | 0.12 (json, rustls-tls) | HTTP client for Ollama embedding |
| `tokio` | 1 (rt, macros, sync) | Async runtime |
| `serde` | 1 (derive) | Serialization |
| `serde_json` | 1 | JSON support |
| `thiserror` | 2 | Error handling |
| `tracing` | 0.1 | Structured logging |
| `uuid` | 1 (v4) | ID generation |
| `chrono` | 0.4 (serde) | Timestamps |
| `bytemuck` | 1 (derive) | Zero-copy vector conversion |
| `hnsw_rs` | 0.3 | HNSW ANN search (optional) |

### Features

- `hnsw` (default) - HNSW approximate nearest-neighbor search
- `brute-force` - Brute-force vector search (no external deps)
- `testing` - Exposes `raw_execute()` for test harnesses

### Source Map

```
semantic-memory/src/
  lib.rs          -- MemoryStore (70+ public methods), open/open_with_embedder
  types.rs        -- Fact, Document, TextChunk, Session, Message, SearchResult,
                     ExplainedResult, ScoreBreakdown, SearchSource, SearchSourceType,
                     EpisodeMeta, EpisodeOutcome, VerificationStatus, GraphView,
                     GraphEdge, GraphEdgeType, GraphDirection, TraceId, Role
  error.rs        -- MemoryError (18+ variants) with kind() discriminant
  config.rs       -- MemoryConfig, EmbeddingConfig, SearchConfig, ChunkingConfig,
                     PoolConfig, MemoryLimits, HnswConfig
  db.rs           -- SQLite init, migrations (V1-V5), schema management, integrity checks
  pool.rs         -- SqlitePool: 1 writer + N readers under WAL mode
  storage.rs      -- StoragePaths for .db, .hnsw.graph, .hnsw.data files
  embedder.rs     -- Embedder trait, OllamaEmbedder, MockEmbedder
  search.rs       -- Hybrid BM25 + vector search with RRF fusion, recency scoring
  chunker.rs      -- Recursive text splitting with overlap and UTF-8 safety
  knowledge.rs    -- Fact CRUD with FTS5 synchronization
  documents.rs    -- Document ingestion pipeline (chunk, embed, store)
  conversation.rs -- Session/message CRUD, token budgeting
  episodes.rs     -- Causal episode tracking with outcome/verification
  quantize.rs     -- SQ8 int8 scalar quantization (4x memory reduction)
  hnsw.rs         -- HNSW index wrapper with keymap persistence (feature-gated)
  graph.rs        -- Derived graph view: BFS neighbors, shortest path, edge types
  tokenizer.rs    -- TokenCounter trait, EstimateTokenCounter (chars/4)
```

### Search Pipeline

1. Sanitize query (strip FTS operators)
2. FTS5 MATCH for BM25 hits (default top 50 candidates)
3. Embed query, search vector backend (HNSW or brute-force)
4. Apply min_similarity cosine threshold (default 0.3)
5. Optional f32 reranking for exact cosine
6. Optional recency boost (exponential decay)
7. RRF fusion of BM25 + vector + recency scores
8. Return top K results with full score breakdown

### Data Primitives

- **Facts** - namespaced knowledge with embeddings and metadata
- **Documents** - ingested and chunked text with embeddings
- **Sessions/Messages** - conversation tracking with FTS and token counting
- **Episodes** - causal records with effect types, outcomes, verification status

### Persistence Model

- **SQLite** (authoritative): All records, embeddings, FTS5 indexes, HNSW keymaps
- **HNSW sidecar** (recoverable): Topology + vector data files; rebuilt from SQLite on corruption
- **WAL mode**: Concurrent readers, serialized writes, journal-based HNSW mutation tracking

---

## knowledge-runtime

**Crate:** `knowledge-runtime` | **Version:** 0.1.0 | **License:** MIT | **MSRV:** 1.75

Bounded orchestration scaffold for `semantic-memory`. Provides intent classification, route planning, scoped entity resolution, provenance-preserving merge, and projection status tracking. This crate never owns source truth -- all data lives in `semantic-memory`.

### Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| `semantic-memory` | path | Upstream memory store |
| `serde` | 1 (derive) | Serialization |
| `serde_json` | 1 | JSON support |
| `thiserror` | 2 | Error handling |
| `tracing` | 0.1 | Structured logging |
| `chrono` | 0.4 (serde) | Timestamps |
| `uuid` | 1 (v4) | Trace ID generation |

### Source Map

```
knowledge-runtime/src/
  lib.rs                    -- Crate docs ("Implemented Now" / "Not Real Yet"), re-exports
  runtime.rs                -- KnowledgeRuntime: query pipeline, projection maintenance
  config.rs                 -- RuntimeConfig, QueryConfig, EntityConfig, ProjectionConfig
  error.rs                  -- RuntimeError (6 variants, no catch-all)
  ids.rs                    -- Scope, ScopeKey, EntityId, ProjectionId, ProjectionKind
  query/
    classify.rs             -- Rule-based classifier: QueryMode (Semantic|Entity|Temporal|Mixed)
    route.rs                -- RoutePlan, RouteLeg, RetrievalStrategy (Hybrid|Entity|Temporal)
    merge.rs                -- Provenance-preserving merge: fuse duplicates, multi-leg boost
  entity/
    registry.rs             -- Scope-partitioned EntityRegistry with fallback resolution
    code_ids.rs             -- Scope-aware code entity IDs (repo_id in canonical ID)
  projection/
    lifecycle.rs            -- ProjectionTracker with StaleCause, version metadata, invalidation
  temporal/
    claims.rs               -- TemporalClaim, ContradictionStatus (structural only)
  evidence/
    support.rs              -- EvidenceBundle, EvidenceItem, EvidenceRelevance
  adapters/
    semantic_memory.rs      -- Read-only SemanticMemoryAdapter (namespace-only search)
  obs/
    trace.rs                -- QueryTrace with QueryWarning enum, ProjectionTrace
```

### What Is Implemented

| Capability | Detail |
|---|---|
| Rule-based classification | Heuristic: `@mention`, `"quoted"`, temporal keywords |
| Route planning | Translates classified modes to retrieval legs |
| Hybrid search execution | Delegates to semantic-memory BM25+vector |
| Entity search with scoped resolution | Scope-partitioned indexes; ExactCanonical/ExactAlias/ScopedFallback/Unresolved |
| Result merge with provenance fusion | Duplicates fused, union of source legs, per-leg scores, configurable boost |
| Scope-aware code identity | Code entity IDs include repo_id/workspace_id |
| Projection lifecycle tracking | Health, staleness, StaleCause, version metadata, invalidation by scope/kind |
| Query degradation warnings | TemporalDowngradedToHybrid, ScopePartiallyEnforced, EntityScopeFallback |

### What Is NOT Implemented (Deferred)

| Capability | Current Behavior |
|---|---|
| Temporal search execution | Falls back to hybrid; `QueryWarning::TemporalDowngradedToHybrid` emitted |
| Full scope enforcement in search | Only `namespace` passed to adapter; other dims enforced in runtime-owned logic only |
| Projection persistence | `persist` config flag accepted but ignored (in-memory only) |
| Projection rebuild execution | Tracker is status-only; callers drive rebuilds |
| LLM-based classification | Heuristic only |
| Forge causal projections | No adapter |
| Fuzzy entity resolution | Exact canonical/alias only |

---

## agent-graph

**Crate:** `agent-graph` | **Version:** 0.2.0 | **License:** MIT | **Edition:** 2021

Graph-based agent orchestration for Rust -- LangGraph for the Rust ecosystem. Owns control flow (routing, loops, parallel execution) and delegates actual work to a pluggable `Payload` trait boundary.

### Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| `tokio` | 1 (full) | Async runtime |
| `futures` | 0.3 | Futures utilities |
| `async-trait` | 0.1 | Async trait syntax |
| `serde` | 1 (derive) | Serialization |
| `serde_json` | 1 | JSON wire format |
| `thiserror` | 1 | Error handling |
| `anyhow` | 1 | Error context |
| `chrono` | 0.4 (serde) | Timestamps |
| `uuid` | 1 (v4, serde) | UUID generation |
| `tracing` | 0.1 | Structured logging |
| `rusqlite` | 0.32 (bundled, optional) | SQLite checkpointing |

### Features

- `checkpointing` (default) -- SQLite-backed checkpoint persistence via `rusqlite`

### Source Map

```
agent-graph/src/
  lib.rs              -- Crate entry + prelude re-exports
  graph.rs            -- AgentGraph, AgentGraphBuilder (core orchestrator)
  state.rs            -- AgentState: async-safe shared state, transactions, forks, snapshots
  node.rs             -- Node trait, FnNode, node! macro (4 forms)
  payload.rs          -- Payload trait (object-safe), PayloadNode, PayloadContext, PayloadOutput
  router.rs           -- RoutingFunction trait, FnRouter, router! macro, RouterOutput
  edge.rs             -- EdgeType (Normal|Conditional)
  command.rs          -- NodeOutput, Command, Navigation, SendOp (dynamic fan-out)
  reducer.rs          -- Reducer trait + 5 built-ins: LastWriteWins, Append, Add, Merge, Fn
  join.rs             -- JoinNode for deterministic fan-in merge
  interrupt.rs        -- InterruptConfig, InterruptCheckpoint, Interrupt, InterruptKind
  outcome.rs          -- ExecutionResult (Complete|Interrupted), NodeOutcome
  event_sink.rs       -- EventSink trait + Noop/Callback/Channel/Composite implementations
  stream.rs           -- StreamEvent, StreamMode (Values|Updates|Events)
  retry.rs            -- RetryPolicy (exponential backoff, jitter, error predicate)
  error.rs            -- AgentGraphError (14 variants) with kind() discriminant
  executor.rs         -- Executor trait, InProcessExecutor
  config.rs           -- GraphConfig (recursion limits, parallelism, trace IDs, metadata)
  checkpoint.rs       -- Legacy checkpoint structures (feature-gated)
  checkpointer.rs     -- CheckpointSaver trait, MemorySaver, SqliteSaver, CheckpointManager
  checkpoint_store.rs -- Granular CheckpointStore trait, AttemptRecord, RunState, RunStatus
  prelude.rs          -- All public re-exports
```

### Execution Modes

| Mode | Description |
|------|-------------|
| `execute()` | Normal execution |
| `execute_with_summary()` | With run summary and trace_id |
| `execute_with_interrupt()` | Human-in-the-loop pause/resume |
| `execute_cancellable()` | Cooperative cancellation |
| `stream()` | Streamed event emission |
| `resume()` | Resume from checkpoint with topology validation |
| `resume_force()` | Resume bypassing topology check |

### Core Primitives

- **Directed graph** with named nodes, START/END sentinels
- **Conditional routing** and fan-out via `RouterOutput::FanOut`
- **Parallel branches** with reducer-based state merges (supersteps)
- **Loops** with configurable iteration limits
- **Interrupt before/after** nodes for human-in-the-loop
- **Transactions** for multi-key atomic state updates
- **State snapshots** with history and per-key reducers
- **Two checkpoint systems**: granular (AttemptRecord-based) and legacy (superstep-based)
- **Topology hash validation** on resume

---

## LLM-Pipeline

**Crate:** `llm-pipeline` | **Version:** 0.2.0 | **License:** MIT | **Edition:** 2021

Reusable node payloads for LLM workflows: prompt templating, multi-backend calls (Ollama, OpenAI), defensive parsing, streaming, transport/semantic retry, and sequential chaining. Designed as the payload layer that pairs with `agent-graph`.

### Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| `llm-output-parser` | path (.parser-lib) | Extracted output parsing |
| `tokio` | 1 (full) | Async runtime |
| `reqwest` | 0.12 (json, stream) | HTTP client |
| `serde` | 1 (derive) | Serialization |
| `serde_json` | 1 | JSON support |
| `anyhow` | 1 | Error handling |
| `thiserror` | 2 | Error derivation |
| `futures` | 0.3 | Async utilities |
| `async-trait` | 0.1 | Async trait support |
| `fastrand` | 2 | Jitter for backoff |
| `uuid` | 1 (v4) | Trace ID generation |
| `tracing` | 0.1 | Observability |

### Features

- `yaml` -- YAML parser (gates `llm-output-parser/yaml`)
- `openai` -- OpenAI-compatible backend (SSE decoder)

### Source Map

```
LLM-Pipeline/src/
  lib.rs              -- Public API surface, all re-exports
  exec_ctx.rs         -- ExecCtx, ExecCtxBuilder (client, backend, vars, cancellation, limits, trace)
  payload.rs          -- Payload trait (object-safe: kind, name, invoke), PayloadOutput, BoxFut
  llm_call.rs         -- LlmCall: primary single-call payload with output strategy + retry
  chain.rs            -- Chain: sequential composition of Box<dyn Payload>
  pipeline.rs         -- Pipeline<T>: legacy typed multi-stage API (#[deprecated])
  stage.rs            -- Stage type for legacy Pipeline
  client.rs           -- Legacy direct call API (#[deprecated])
  backend/
    mod.rs            -- Backend trait (complete, complete_streaming), LlmRequest, LlmResponse,
                         ChatMessage, Role, LlmConfig, with_backoff() wrapper
    ollama.rs         -- OllamaBackend: /api/generate, /api/chat, NDJSON streaming
    openai.rs         -- OpenAiBackend: /v1/chat/completions, SSE streaming (feature-gated)
    mock.rs           -- MockBackend: canned responses for testing
    recording.rs      -- RecordingBackend: capture/replay for deterministic tests
    backoff.rs        -- BackoffConfig, JitterStrategy (None|Full|Equal|Decorrelated)
    sse.rs            -- SseDecoder for OpenAI SSE streams (feature-gated)
  output_parser.rs    -- Re-exports from llm-output-parser + streaming JSON utilities
  output_strategy.rs  -- OutputStrategy enum (Lossy|Json|StringList|XmlTag|Choice|Number|Text|Custom)
  diagnostics.rs      -- ParseDiagnostics: strategy, errors, retries, repairs, warnings
  retry_policy.rs     -- RetryConfig: semantic retry on parse failure, optional validator
  retry.rs            -- Legacy retry types
  parsing.rs          -- Defensive extraction: extract_thinking(), extract_json_block(), parse_value_lossy()
  streaming.rs        -- StreamingDecoder: buffered NDJSON decoder
  events.rs           -- Event enum (PayloadStart|Token|PayloadEnd|RetryStart|RetryEnd|TransportRetry)
  limits.rs           -- PipelineLimits: max_response_bytes, request_timeout, stream_idle_timeout
  trace.rs            -- TraceId for cross-crate correlation
  prompt.rs           -- render() template engine ({input}, {key} substitution), numbered_list(), section()
  error.rs            -- PipelineError (11 variants), Result<T>
  types.rs            -- Legacy types: PipelineInput, StageOutput<T>, PipelineContext
```

### Core Data Flow

1. `LlmCall::invoke()` renders prompt with context variables
2. Builds normalized `LlmRequest` (provider-agnostic)
3. `Backend::complete()` translates to provider-specific HTTP
4. Response becomes `LlmResponse { text, status, metadata }`
5. `OutputStrategy` parses text via `llm-output-parser`
6. On parse failure + `RetryConfig`: builds correction prompt, retries
7. Returns `PayloadOutput { value, raw_response, thinking, diagnostics, ... }`

### Retry Layers

| Layer | Trigger | Behavior |
|---|---|---|
| **Transport** | HTTP 429/5xx | Exponential backoff with jitter, respects Retry-After |
| **Semantic** | Parse failure or validator rejection | Correction prompt with original request + bad output + error |

---

## llm-output-parser

**Crate:** `llm-output-parser` | **Version:** 0.2.0 | **License:** MIT | **Path:** `.parser-lib/`

Production-grade parser for extracting structured data from LLM responses. Handles think blocks, markdown fences, malformed JSON, and real-world model output without an additional LLM call. Used by both `llm-pipeline` and `ollama-vision`.

### Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| `serde` | 1 (derive) | Serialization |
| `serde_json` | 1 | JSON support |
| `thiserror` | 2 | Error handling |
| `serde_yaml` | 0.9 (optional) | YAML parsing |

### Source Map

```
.parser-lib/src/
  lib.rs      -- Public re-exports for all parsers
  error.rs    -- ParseError, ParseOptions (safety limits), ParseTrace (diagnostics)
  extract.rs  -- preprocess(), strip_think_tags()
  json.rs     -- parse_json<T>(), parse_json_value(), traced variants
  list.rs     -- parse_string_list(), parse_string_list_raw(), traced variant
  xml.rs      -- parse_xml_tag(), parse_xml_tags(), traced variants
  choice.rs   -- parse_choice(), traced variant
  number.rs   -- parse_number(), parse_number_in_range(), traced variants
  text.rs     -- parse_text(), traced variant
  repair.rs   -- try_repair_json() (fix common LLM JSON errors)
  yaml.rs     -- parse_yaml() (feature-gated)
```

### Parser Functions

| Parser | Input | Output |
|--------|-------|--------|
| `parse_json<T>` | LLM text | Typed `T: DeserializeOwned` |
| `parse_json_value` | LLM text | `serde_json::Value` |
| `parse_string_list` | LLM text | `Vec<String>` (cleaned, deduped) |
| `parse_xml_tag` | LLM text + tag name | Tag content `String` |
| `parse_xml_tags` | LLM text + tag names | `HashMap<String, String>` |
| `parse_choice` | LLM text + valid options | Matched `String` |
| `parse_number` | LLM text | `f64` |
| `parse_number_in_range` | LLM text + min/max | Bounded `f64` |
| `parse_text` | LLM text | Cleaned `String` |
| `parse_yaml` | LLM text | Typed `T` (feature: yaml) |

All parsers have `_with_trace` variants returning `ParseTrace` diagnostics.

---

## job-queue

**Crate:** `job-queue` | **Version:** 0.2.0 | **License:** MIT | **Edition:** 2021

Production-grade background job queue with SQLite persistence, priority scheduling, lease-based claiming, heartbeat, retry with failure classification, and cooperative cancellation. Framework-agnostic (no Tauri dependency).

### Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| `tokio` | 1 (full) | Async runtime |
| `rusqlite` | 0.32 (bundled) | SQLite persistence |
| `serde` | 1 (derive) | Serialization |
| `serde_json` | 1 | JSON payloads |
| `anyhow` | 1 | Error handling |
| `chrono` | 0.4 (serde) | Timestamps |
| `uuid` | 1 (v4, serde) | Job ID generation |
| `thiserror` | 2 | Error derivation |
| `tracing` | 0.1 | Structured logging |

### Source Map

```
job-queue/src/
  lib.rs       -- Traits: JobHandler, QueueEventEmitter, JobContext; built-in emitters
  types.rs     -- QueueJob<T>, JobResult, QueuePriority (High|Normal|Low),
                  QueueJobStatus, QueueStats, QueueJobDetails, FailureClass
  config.rs    -- QueueConfig + QueueConfigBuilder (poll interval, heartbeat, stale timeout, retries)
  error.rs     -- QueueError (8 variants) with kind() discriminant
  queue.rs     -- QueueManager: public API (add, cancel, reorder, pause/resume, list, prune, spawn)
  executor.rs  -- QueueExecutor: background processing loop, lease claiming, heartbeat, retry
  db.rs        -- SQLite layer: schema V3, 24 functions, WAL mode, migrations
  events.rs    -- JobStartedEvent, JobCompletedEvent, JobFailedEvent, JobProgressEvent, JobCancelledEvent
```

### Key Features

- **Priority scheduling**: High(1) > Normal(2) > Low(3) with FIFO within priority
- **Lease-based claiming**: Worker ID + visibility timeout prevents duplicate processing
- **Crash recovery**: Requeues processing jobs from crashed workers on startup
- **Retry with failure classification**: Transient (retry), Permanent (no retry), RateLimited (delay)
- **Cooperative cancellation**: Jobs check `ctx.is_cancelled()` during execution
- **Heartbeat**: Running jobs renew leases periodically
- **Progress reporting**: Via `ctx.emit_progress(current, total)` through pluggable emitter
- **Trace ID**: End-to-end correlation with upstream orchestration
- **Hardware throttling**: `max_consecutive` + `cooldown` prevents resource exhaustion

---

## AI-Batch-Queue

**Crate:** `ai-batch-queue` | **Version:** 0.2.0 | **License:** MIT | **Edition:** 2021

Model-aware batch processing queue with ETA estimation for Tauri applications. Groups jobs by resource key to minimize expensive resource swaps (e.g., GPU model loads) and provides size-bucketed time estimates.

### Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| `tauri` | 2 | Desktop app framework |
| `tokio` | 1 (full) | Async runtime |
| `serde` | 1 (derive) | Serialization |
| `serde_json` | 1 | JSON support |
| `anyhow` | 1 | Error handling |
| `chrono` | 0.4 (serde) | Timestamps |
| `uuid` | 1 (v4, serde) | ID generation |
| `thiserror` | 2 | Error derivation |
| `tracing` | 0.1 | Structured logging |

### Source Map

```
AI-Batch-Queue/src/
  lib.rs       -- Traits: BatchItemHandler (process, should_skip), BatchStore; build_job()
  types.rs     -- BatchJob<D>, BatchItem<D>, BatchItemStatus, BatchJobStatus,
                  OverwritePolicy, SizeBucket, EtaEstimate, EtaConfidence, SchedulingConfig
  queue.rs     -- BatchQueue<D>: in-memory Mutex-based queue with resource-aware reordering
  eta.rs       -- EtaTracker: per-(resource, operation, size_bucket) timing statistics
  executor.rs  -- Tauri background executor: poll loop, process items, emit events
```

### Key Features

- **Resource-aware reordering**: Groups jobs by `resource_key` to minimize GPU model swaps
- **Size-bucketed ETA**: Small/Medium/Large buckets improve estimates as work completes
- **Fairness scheduling**: `max_consecutive_same_key` prevents resource starvation
- **Progressive completion**: Failed items retryable without re-processing successes
- **Tauri events**: `ai_batch:job_started`, `ai_batch:item_progress`, `ai_batch:job_completed`
- **Generic over item data**: Works with any `Clone + Serialize` type

---

## Ollama-Vision-RS

**Crate:** `ollama-vision` | **Version:** 0.2.0 | **License:** MIT | **Edition:** 2021

Robust Ollama vision model toolkit for image tagging and captioning. Core value is not just calling Ollama, but reliably recovering structured tags and captions from inconsistent model output using multi-strategy parsing via `llm-output-parser`.

### Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| `reqwest` | 0.12 (json) | HTTP client |
| `tokio` | 1 (full) | Async runtime |
| `serde` | 1 (derive) | Serialization |
| `serde_json` | 1 | JSON support |
| `thiserror` | 2 | Error handling |
| `base64` | 0.22 | Image encoding |
| `llm-output-parser` | path (.parser-lib) | Response parsing |

### Source Map

```
Ollama-Vision-RS/src/
  lib.rs       -- Public re-exports
  types.rs     -- OllamaVisionConfig (model, timeout, options), TagOptions, CaptionOptions,
                  GenerateOptions (num_predict, temperature, top_p)
  tagger.rs    -- tag_image(), tag_image_base64(): file/base64 -> Ollama -> parsed tags
  captioner.rs -- caption_image(), caption_image_base64(): file/base64 -> Ollama -> caption text
  parser.rs    -- Re-exports from llm-output-parser: parse_tags, strip_think_tags
```

### Public API

| Function | Input | Output |
|----------|-------|--------|
| `tag_image` | file path | `Vec<String>` tags |
| `tag_image_base64` | base64 string | `Vec<String>` tags |
| `caption_image` | file path | `String` caption |
| `caption_image_base64` | base64 string | `String` caption |
| `parse_tags` | raw LLM text | `Vec<String>` (7-strategy parser) |
| `strip_think_tags` | text with `<think>` | cleaned text |

---

## ComfyUI-RS

**Crate:** `comfyui-rs` | **Version:** 0.2.0 | **License:** MIT | **Edition:** 2021

Typed async client for ComfyUI (node-based Stable Diffusion GUI/backend). Covers queueing prompts, inspecting history, downloading images, discovering models, and waiting for completion with WebSocket progress plus polling fallback.

### Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| `reqwest` | 0.12 (json, multipart) | HTTP client |
| `tokio` | 1 (time) | Async runtime |
| `tokio-tungstenite` | 0.24 | WebSocket client |
| `futures-util` | 0.3 | StreamExt for WS |
| `serde` | 1 (derive) | Serialization |
| `serde_json` | 1 | JSON workflows |
| `thiserror` | 2 | Error handling |
| `rand` | 0.9 | Seed generation |
| `tracing` | 0.1 | Structured logging |

### Source Map

```
ComfyUI-RS/src/
  lib.rs       -- Public re-exports
  client.rs    -- ComfyClient: health, queue_prompt, history, image download,
                  model/sampler/scheduler discovery, upload, free_memory,
                  wait_for_completion (polling), wait_for_completion_ws (WebSocket + fallback)
  types.rs     -- ProgressUpdate, ComfyProgress, ComfyStatus, WsConfig, DownloadLimits,
                  ImageRef, PromptHistory, QueueStatus, GenerationOutcome
  error.rs     -- ComfyError (8 variants) with kind() discriminant
  workflow.rs  -- Txt2ImgRequest builder (7-node pipeline: checkpoint -> latent -> CLIP ->
                  KSampler -> VAE -> SaveImage)
```

### Key Features

- **WebSocket progress** with automatic polling fallback
- **Message count guards** to prevent memory exhaustion (10k per-prompt, 50k total)
- **Model/sampler/scheduler discovery** via `/object_info`
- **Image upload** with multipart form support
- **Fluent workflow builder** for txt2img with sensible defaults

---

## Tauri-Queue

**Crate:** `tauri-queue` | **Version:** 0.3.0 | **License:** MIT | **Edition:** 2021

Thin Tauri integration bridge for `job-queue`. Adds event emission to Tauri's frontend event system with configurable coalescing/throttling to prevent event storms.

### Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| `job-queue` | path | Core queue implementation |
| `tauri` | 2 | Desktop app framework |
| `tokio` | 1 (full) | Async runtime |
| `serde` | 1 (derive) | Serialization |
| `serde_json` | 1 | JSON support |

### Source Map

```
Tauri-Queue/src/
  lib.rs  -- Re-exports all job-queue types + adds:
              TauriEventEmitter (emits queue:* events to Tauri frontend)
              CoalescingEmitter (time-based throttling with buffer + drop policy)
              EmitterConfig (buffer_size, drop_policy, coalesce_interval_ms)
              DropPolicy (DropOldest | DropNewest | Block)
```

### Event Mapping

| Queue Event | Tauri Event Name |
|---|---|
| JobStarted | `queue:job_started` |
| JobCompleted | `queue:job_completed` |
| JobFailed | `queue:job_failed` |
| JobProgress | `queue:job_progress` |
| JobCancelled | `queue:job_cancelled` |

### Coalescing Behavior

- Progress events within `coalesce_interval_ms` (default 50ms) are suppressed
- Non-progress events (started, completed, failed, cancelled) always pass through
- Buffer overflow handled by configurable `DropPolicy`
- Per-job tracking with FIFO pending queue

---

## Tauri-React-Hooks

**Package:** `@tauri-hooks/core` | **Version:** 0.1.0 | **License:** MIT | **Type:** npm/TypeScript

React hooks for Tauri 2 applications. Removes repetitive wiring of React frontends to Tauri commands and events. Provides async-safe event listeners, command invocation with state management, config persistence, and high-frequency stream buffering.

### Peer Dependencies

- `react` >= 18
- `@tauri-apps/api` >= 2

### Build

- **Tool:** tsup (TypeScript bundler)
- **Formats:** ESM + CommonJS dual-publish
- **Target:** ES2020

### Source Map

```
Tauri-React-Hooks/src/
  index.ts              -- Barrel exports
  types.ts              -- Shared TypeScript types
  useTauriEvent.ts      -- Single Tauri event subscription
  useTauriEvents.ts     -- Multiple event subscription (atomic)
  useTauriQuery.ts      -- Command invocation with auto-fetch + event invalidation
  useTauriMutation.ts   -- Mutation wrapper with onSuccess/onError callbacks
  useTauriConfig.ts     -- Config load, optimistic-update, and save
  useBufferedStream.ts  -- Two-layer buffered high-frequency updates (~30fps)
```

### Exported Hooks

| Hook | Purpose |
|------|---------|
| `useTauriEvent` | Subscribe to a single Tauri event (async-safe cleanup, ref-based handler freshness) |
| `useTauriEvents` | Subscribe to multiple events atomically (parallel async cleanup) |
| `useTauriQuery` | Command invocation with auto-fetch, event-based invalidation, enable/disable |
| `useTauriMutation` | Explicit mutation wrapper with onSuccess/onError callbacks |
| `useTauriConfig` | Load, optimistic-update, and save config objects |
| `useBufferedStream` | Two-layer buffered high-frequency updates (default ~30fps flush) |

### Key Design Patterns

- **Async-safe cleanup**: `cancelled` flag + ref pattern for async `listen()` operations
- **Handler freshness**: `useRef` keeps handler identities fresh without re-subscriptions
- **Two-layer buffering**: `useBufferedStream` uses pending (sync writes) + flushed (state updates)
- **Query invalidation**: Built-in Tauri event-based auto-refresh

---

## Primitives

**Path:** `Primitives/` | **Crates:** 10 | **Version:** 0.1.0 (all) | **License:** MIT | **MSRV:** 1.75

A workspace of 10 complementary Rust crates forming a unified system for patch validation, execution, causal attribution, and intelligent retry policies. These are the low-level building blocks used by `semantic-memory-forge`.

### Crate Overview

| Crate | Purpose | Key Types |
|-------|---------|-----------|
| **effect-signature** | Stable identifiers for check effects | `EffectSignature`, `LocatedEffect`, BLAKE3 hashing |
| **forge-policy** | Path/env/DB guardrails | `Violation`, `ViolationKind` (7 kinds), `DbIdentitySpec`, forbidden path patterns |
| **sandbox-workspace** | Safe workspace staging | `Workspace`, `PatchedWorkspace`, `PatchFs` trait, `LocalPatchFs` |
| **check-runner** | Host/container check execution | `ExecutionBackend` trait, `HostBackend`, `ContainerBackend`, `CheckKind`, `CheckResult` |
| **typed-patch** | Structured patch model | `StructuredPatch`, `FileEdit`, `EditOp` (Insert/Delete/Replace), `Anchor`, `LineRange`, `LineAttributionMap` |
| **cea-core** | Causal edit attribution | `AttributionTriple`, `CausalGraph` (petgraph), `CausalPrediction`, `RiskFlag`, `EditOpSignature` |
| **cea-store** | CEA persistence interface | `CeaStore` trait, `UpdateResult`, idempotency via run hash |
| **cea-sqlite** | SQLite-backed CEA storage | `SqliteCeaStore`, upsert nodes/edges, Beta-like confidence |
| **mindstate-core** | Deterministic state rendering | `MindState`, `EvidenceItem`, BLAKE3 hash, `compute_question_sig()` |
| **stabilizer-core** | Attempt phase management | `Stabilizer` (Innovative->Stabilize1->Stabilize2->Clamp), `extract_strategy_tags()`, novelty scoring |

### Internal Dependency DAG

```
effect-signature --------------------------+
forge-policy ----------+---------+---------+
                        |         |         |
sandbox-workspace ------+         |         |
                        |         |         |
check-runner -----------+         |         |
                                  |         |
typed-patch ----------------------+         |
                                            |
cea-core -----------------------------------+
  |
cea-store
  |
cea-sqlite

mindstate-core (standalone)
stabilizer-core (standalone, uses typed-patch)
```

### CEA Attribution Pipeline

1. **Build signatures**: `build_edit_op_signature()` from `EditOp` + context (BLAKE3 hash)
2. **Attribute effects**: Match check effects to edit ops by line distance, severity weighting
3. **Score with softmax**: Normalize attribution weights across candidates
4. **Update graph**: `ingest_run()` adds edges, applies decay, records negative evidence
5. **Predict risk**: Fuzzy match new signatures against graph history for correctness/novelty/confidence

---

## living-memory

**Crate:** `semantic-memory-forge` | **Version:** 0.2.0 | **License:** MIT | **MSRV:** 1.75
**Path:** `living-memory/living-memory/`

Causal edit attribution and structured patch evaluation engine built on `semantic-memory` and all 10 Primitives crates. Turns lower-level libraries into an agent-runtime and evaluation system: compiles mindstate, validates structured patches, runs host/container checks, records evidence, and updates a causal graph for patch risk prediction.

### Dependencies

**Internal (11 crates):**
`semantic-memory`, `forge-policy`, `sandbox-workspace`, `typed-patch`, `effect-signature`, `check-runner` (with container feature), `mindstate-core`, `stabilizer-core`, `cea-core`, `cea-store`, `cea-sqlite`

**External (19 crates):**
`tokio`, `serde`, `serde_json`, `thiserror`, `anyhow`, `uuid`, `blake3`, `rusqlite`, `tempfile`, `walkdir`, `once_cell`, `petgraph`, `regex`, `async-trait`, `tracing`, `chrono`, `glob`, `similar`, `rand`

### Source Map

```
living-memory/living-memory/src/
  lib.rs              -- Public API surface (all re-exports)
  config.rs           -- ForgeConfig (master config for all subsystems)
  error.rs            -- ForgeError, ForgeResult, Violation, ViolationKind
  invariants.rs       -- DB identity verification, patch validation safety rails
  baseline.rs         -- BaselineDescriptor, BaselineSourceKind, capture_baseline_provenance()
  scoring.rs          -- ComparabilityClass, ObjectiveKind, PatchExecutionPlan, PlannedCheck
  failure.rs          -- Failure analysis types
  experiment.rs       -- PairedExperimentRunner, ExperimentConfig, ExperimentResult, TrialRecord
  export.rs           -- EpisodeExport, compute_export_key(), export_bundle()
  store/
    mod.rs            -- Store module entry
    schema.rs         -- 10 SQLite tables, indexes, migration framework
    db.rs             -- ForgeStore: CRUD for candidates, tasks, evals, archive, CEA
  runtime/
    mod.rs            -- compile_mindstate, extract_strategy_tags, AttemptPhase, DeltaPolicy
    mindstate.rs      -- MindState compilation from request + repo context + evidence
    compiler.rs       -- MindState compilation logic
    novelty.rs        -- Strategy tag extraction, novelty scoring, delta amplitude
    stabilizer.rs     -- Attempt loop phases with weight management
    patch/
      mod.rs          -- Patch module entry
      types.rs        -- StructuredPatch, FileEdit, EditOp, Anchor, LineRange, etc.
      validate.rs     -- validate_patch()
      apply.rs        -- apply_patch()
      render_diff.rs  -- render_diff()
  exec/
    mod.rs            -- ExecutionBackend selection, CheckCommand, CheckKind, CheckResult
    backend.rs        -- ExecutionBackend selection (host vs container)
    host.rs           -- Direct process execution with env sanitization
    container.rs      -- Docker/Podman/Nerdctl with auto-detection
  adapters/
    mod.rs            -- Adapter module entry
    cargo.rs          -- Rust/Cargo adapter (fmt, clippy, test)
  lab/
    mod.rs            -- Lab module entry
    suite.rs          -- EvalTask/EvalSuite loaded from fixtures
    evaluate.rs       -- Correctness/novelty/stability scoring
    archive.rs        -- MAP-Elites archive (novelty x stability x approach family)
    promote.rs        -- BasisVersion promotion with frozen parameters
    evidence.rs       -- Evidence bundles, causal hypotheses, verification plans
    emitters.rs       -- AlgebraSpec (parameter space), mutation, crossover
  cea/
    mod.rs            -- CEA module entry
    graph.rs          -- Re-exports CausalGraph
    instrumentation.rs -- EditOpSignature construction, effect attribution
    predictor.rs      -- Risk prediction from causal graph
    store.rs          -- Graph persistence (load/update from ForgeStore)
```

### Layered Architecture

| Layer | Responsibility |
|-------|---------------|
| **Config** | Single `ForgeConfig` controls all runtime behavior |
| **Invariants** | Safety rails validate DB identity before operations |
| **Store** | SQLite persistence with ACID + WAL mode (10 tables) |
| **Runtime** | Compiles MindState, generates patches, stabilizes attempts |
| **Exec** | Executes checks via host process or container with timeout/env controls |
| **Lab** | Scores runs, archives candidates (MAP-Elites), promotes basis versions |
| **CEA** | Attributes failures to edit ops, learns causal graph, predicts risk |
| **Baseline** | Captures baseline provenance and comparability policy |
| **Experiment** | Paired experiment runner with trial records and statistical analysis |
| **Export** | Serialized episode export with rendering versioning |

### Database Schema (10 tables)

`forge_meta`, `candidates`, `tasks`, `eval_runs`, `archive_cells`, `promotions`, `answer_traces`, `cea_nodes`, `cea_edges`, `cea_run_log`

### Features

- `danger-sm-write` (default: false) -- Permits write access to semantic-memory (normally read-only)

---

## Ghost Directories

| Directory | Status |
|---|---|
| `llm-pipeline/` (lowercase) | Orphaned artifact from deleted duplicate crate. Not the real crate. The real crate is `LLM-Pipeline/` (uppercase). |

---

## Composition Patterns

### 1. Local agent runtime
`agent-graph` + `llm-pipeline` + `semantic-memory`

Use when you want a LangGraph-style orchestrator with local retrieval and durable context.

### 2. Safe autonomous patching
`agent-graph` + `semantic-memory-forge` + `Primitives/*`

Use when you want an agent to generate, validate, score, and learn from code patches.

### 3. Desktop AI operations app
`job-queue` or `ai-batch-queue` + `tauri-queue` + `@tauri-hooks/core`

Use when you need long-running work, resumability, frontend progress, and high-frequency event handling.

### 4. Multimodal image workflow
`ollama-vision` + `comfyui-rs` + `ai-batch-queue`

Use for captioning, tagging, reranking, and image generation pipelines with model-aware batching.

### 5. Knowledge-augmented agents
`knowledge-runtime` + `semantic-memory` + `agent-graph` + `llm-pipeline`

Use when you need scoped entity resolution, intent classification, and provenance-preserving search on top of the base agent runtime.

### 6. Experiment and evaluation
`semantic-memory-forge` + `Primitives/*` + `semantic-memory`

Use for structured patch evaluation, paired experiments, MAP-Elites archiving, causal attribution, and basis version promotion.
