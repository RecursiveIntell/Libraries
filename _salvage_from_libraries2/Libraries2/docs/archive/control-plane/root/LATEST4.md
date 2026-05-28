# LATEST4 — Full Ecosystem Reference

> Snapshot date: 2026-03-07
> Total: 10 primary Rust crates, 10 primitive Rust crates, 1 parser crate, 1 TypeScript package, 1 runtime orchestration crate
> Combined test count: 1,059

---

## Table of Contents

1. [Ecosystem Overview](#ecosystem-overview)
2. [semantic-memory](#semantic-memory)
3. [knowledge-runtime](#knowledge-runtime)
4. [semantic-memory-forge](#semantic-memory-forge)
5. [agent-graph](#agent-graph)
6. [llm-pipeline](#llm-pipeline)
7. [llm-output-parser](#llm-output-parser)
8. [job-queue](#job-queue)
9. [ai-batch-queue](#ai-batch-queue)
10. [tauri-queue](#tauri-queue)
11. [comfyui-rs](#comfyui-rs)
12. [ollama-vision](#ollama-vision)
13. [@tauri-hooks/core](#tauri-hooks-core)
14. [Primitive Crates](#primitive-crates)
15. [Composition Patterns](#composition-patterns)
16. [Cross-Crate Contracts](#cross-crate-contracts)
17. [Dependency Graph](#dependency-graph)

---

## Ecosystem Overview

```text
agent-graph ─────────────────────────────────────────── orchestration layer
  |
llm-pipeline + llm-output-parser ────────────────────── model execution layer
  |
semantic-memory ─────────────────────────────────────── retrieval + persistence layer
  |                    |
knowledge-runtime      |  ───────────────────────────── query planning + merge layer
  |                    |
semantic-memory-forge + Primitives/* ────────────────── verification + evaluation layer
  |
job-queue ──> tauri-queue ──> @tauri-hooks/core ─────── work management + frontend layer
ai-batch-queue ──────────^
  |
ollama-vision + comfyui-rs ──────────────────────────── multimodal layer
```

### Package Inventory

| Crate | Version | Tests | Role |
|---|---|---|---|
| `semantic-memory` | 0.5.0 | 224 | Hybrid retrieval, persistence, projection import |
| `knowledge-runtime` | 0.1.0 | 67 | Query planning, merge, projection lifecycle |
| `semantic-memory-forge` | 0.2.0 | 144 | Patch evaluation, causal attribution, verification |
| `agent-graph` | 0.2.0 | 133 | Graph-based agent orchestration |
| `llm-pipeline` | 0.2.0 | 206 | LLM call execution, streaming, retry, parsing |
| `llm-output-parser` | 0.2.0 | 144 | Defensive structured output parsing |
| `job-queue` | 0.2.0 | 35 | Persistent background job queue |
| `ai-batch-queue` | 0.2.0 | 52 | Model-aware batch processing with ETA |
| `tauri-queue` | 0.3.0 | 26 | Tauri bridge for job-queue |
| `comfyui-rs` | 0.2.0 | 22 | ComfyUI REST/WebSocket client |
| `ollama-vision` | 0.2.0 | 6 | Vision model tagging and captioning |
| `@tauri-hooks/core` | 0.1.0 | — | React hooks for Tauri events/commands |
| 10x Primitives | 0.1.0 each | — | Forge lower-layer contracts |

---

## semantic-memory

**Version:** 0.5.0 | **Tests:** 224 | **Schema:** V10 | **License:** MIT

Hybrid semantic search engine with SQLite, FTS5, and optional HNSW ANN — built for AI agents.

### Authority

Authoritative for **queryable knowledge state**: facts, documents/chunks, sessions/messages, episodes, imported claim/entity/relation projections, search indexes.

Does NOT own verification semantics, promotion policy, or Forge-specific logic.

### Features

- `default` = `["hnsw"]` — HNSW approximate nearest neighbor search
- `brute-force` — pure vector scanning (alternative to HNSW)
- `testing` — test utilities (MockEmbedder)

### Dependencies

tokio, rusqlite (bundled), reqwest, serde/serde_json, uuid, chrono, thiserror, tracing, bytemuck, hnsw_rs (optional)

### Storage Schema (19 tables, V10)

| Table | Purpose |
|---|---|
| `sessions` | Conversation sessions |
| `messages` | Chat messages with embeddings |
| `facts` | Discrete knowledge items |
| `facts_fts` / `facts_rowid_map` | FTS5 full-text index |
| `documents` | Chunked source documents |
| `chunks` | Document chunks with embeddings |
| `chunks_fts` / `chunks_rowid_map` | FTS5 full-text index |
| `messages_fts` / `messages_rowid_map` | FTS5 full-text index |
| `episodes` | Causal records with verification status |
| `episodes_fts` / `episodes_rowid_map` | FTS5 full-text index |
| `episode_causes` | Causal edge table |
| `embedding_metadata` | Embedding model tracking |
| `hnsw_metadata` / `hnsw_keymap` | HNSW sidecar state |
| `pending_index_ops` | HNSW journal for crash recovery |
| `import_log` | Projection import tracking (V10) |

### Public API (64 methods on MemoryStore)

**Lifecycle:** `open()`, `open_with_embedder()`, `config()`

**Sessions:** `create_session()`, `rename_session()`, `list_sessions()`, `delete_session()`

**Messages:** `add_message()`, `add_message_with_trace()`, `add_message_fts()`, `add_message_embedded()`, `get_recent_messages()`, `get_messages_within_budget()`, `session_token_count()`

**Facts:** `add_fact()`, `add_fact_with_trace()`, `add_fact_with_embedding()`, `update_fact()`, `delete_fact()`, `delete_namespace()`, `get_fact()`, `get_fact_embedding()`, `list_facts()`

**Documents:** `ingest_document()`, `delete_document()`, `list_documents()`, `count_chunks_for_document()`

**Search:** `search()` (hybrid BM25+vector RRF), `search_fts_only()`, `search_vector_only()`, `search_conversations()`, `search_explained()` (with score breakdown)

**Episodes:** `ingest_episode()`, `create_episode()`, `get_episode()`, `update_episode_outcome()`, `search_episodes()`

**Embeddings:** `embed()`, `embed_batch()`, `embedding_displacement()`

**Graph:** `graph_view()` — semantic/temporal/causal edge traversal

**HNSW:** `rebuild_hnsw_index()`, `flush_hnsw()`, `compact_hnsw()`

**Integrity:** `verify_integrity()`, `reconcile()`, `stats()`, `vacuum()`

**Projection Import:** `import_envelope()`, `import_status()`, `list_imports()`, `last_import_at()`

### Configuration

| Config | Key Fields | Defaults |
|---|---|---|
| `EmbeddingConfig` | ollama_url, model, dimensions, batch_size, timeout_secs | localhost:11434, nomic-embed-text, 768, 32, 30s |
| `SearchConfig` | bm25_weight, vector_weight, rrf_k, min_similarity, recency_half_life_days | 1.0, 1.0, 60.0, 0.3, None |
| `ChunkingConfig` | target_size, min_size, max_size, overlap | 1000, 100, 2000, 200 |
| `PoolConfig` | max_read_connections, busy_timeout_ms, enable_wal | 4, 5000, true |
| `MemoryLimits` | max_facts_per_namespace, max_content_bytes, max_embedding_concurrency | 100K, 1MB, 8 |
| `HnswConfig` | m, ef_construction, ef_search, max_elements, compaction_threshold | 16, 200, 50, 100K, 0.3 |

### Error Types (25 variants)

Database, EmbeddingRequest, DimensionMismatch, InvalidEmbedding, ModelMismatch, SessionNotFound, FactNotFound, DocumentNotFound, EpisodeNotFound, EmbedderUnavailable, MigrationFailed, HnswError, InvalidKey, QuantizationError, StorageError, IntegrityError, SchemaAhead, ContentTooLarge, NamespaceFull, DatabaseSizeLimitExceeded, PoolTimeout, InvalidConfig, CorruptData, ImportInvalid, ImportDuplicate, Other

All with `.kind()` returning stable `&'static str` discriminant.

### Import Boundary Types

- `EnvelopeId` — opaque import envelope identifier
- `ImportEnvelope` — envelope_id, schema_version, content_digest, source_authority, trace_id, namespace, records
- `ImportRecord` — Fact or Episode variant
- `ImportReceipt` — confirmation with status, record_count, imported_at, was_duplicate, trace_id
- `ImportStatus` — Complete, AlreadyImported, Aborted
- `ProjectionFreshness` — Current, Stale, Superseded, ImportFailed, NeverImported

### Key Patterns

- **SQLite is authoritative** — HNSW sidecar is recoverable acceleration only
- **Hybrid search** — BM25 + vector via Reciprocal Rank Fusion with configurable weights
- **Explainable ranking** — ScoreBreakdown per result with all scoring components
- **Atomic import** — all records in an envelope commit together or not at all
- **Idempotent ingest** — composite key (envelope_id, schema_version, content_digest)
- **Quantized storage** — f32 + i8 (SQ8) embeddings for 4x compression at <0.5% error
- **Provenance preservation** — `_import` metadata on every imported record
- **Connection pooling** — 1 writer + N WAL readers with semaphore-capped embedding concurrency

---

## knowledge-runtime

**Version:** 0.1.0 | **Tests:** 67 | **License:** MIT

Bounded orchestration scaffold for semantic-memory: classification, routing, scoped entity resolution, provenance-preserving merge, and projection status tracking.

### Authority

Owns **query planning, retrieval composition, merge policy, and projection lifecycle interpretation**. NEVER owns source truth — all data lives in semantic-memory. Deleting any projection forces recomputation but causes no data loss.

### Dependencies

semantic-memory (path), serde/serde_json, thiserror, tracing, chrono, uuid

### Query Pipeline (5 stages)

```text
query text
  -> classify (QueryMode: semantic | entity | temporal | mixed)
  -> plan (RoutePlan with retrieval legs)
  -> execute (per-leg retrieval via semantic-memory adapter)
  -> merge (fuse duplicates, normalize, boost, rank, truncate)
  -> results + QueryTrace (with degradation warnings)
```

**Classification:** Rule-based heuristic — `@mentions`, `"quoted strings"`, temporal keywords. LLM-based classification deferred.

**Route Planning:** Maps QueryMode to RetrievalStrategy legs (HybridSearch, EntitySearch, TemporalSearch).

**Merge Pipeline:** 6-phase — collect, fuse duplicates by identity key, normalize (MinMax or None), boost multi-leg support, rank with 3-tier deterministic tie-breaking (score > leg count > identity key), truncate.

### Public API

**Query Execution:**
- `query(query, scope)` — full pipeline, returns `(Vec<SearchResult>, QueryTrace)`
- `classify(query)` — classification only
- `plan(query, scope)` — planning only

**Entity Registry:** `entity_registry()`, `entity_registry_mut()` — scope-partitioned entity resolution with exact canonical/alias matching and namespace fallback

**Projection Management:** `projection_health()`, `projection_status()`, `record_projection_build()`, `record_projection_failure()`, `invalidate_projections()`, `invalidate_projections_by_kind()`, `invalidate_scope()`, `clear_projection_scope()`

### Configuration

| Config | Key Fields | Defaults |
|---|---|---|
| `QueryConfig` | max_results_per_leg, max_route_legs, default_limit | 20, 4, 10 |
| `EntityConfig` | max_aliases, max_entities | 16, 10K |
| `ProjectionConfig` | staleness_threshold_secs, import_staleness_threshold_secs, persist | 3600, 3600, false |

### Projection Health (6 states)

Healthy, Stale, Missing, Rebuilding, ImportLagging, ImportFailed

### Stale Causes (6 variants)

TimeThreshold, ExplicitInvalidation, SourceChanged, VersionMismatch, ImportLag, ImportFailure

### Query Warnings (4 variants)

TemporalDowngradedToHybrid, ScopePartiallyEnforced, EntityScopeFallback, ProjectionImportStale

### QueryTrace Helpers

`is_degraded()`, `has_temporal_downgrade()`, `has_scope_enforcement_warning()`, `has_import_staleness_warning()`

### Error Types (6 variants)

Memory, InvalidConfig, EntityNotFound, RegistryFull, ProjectionUnavailable, Adapter — all with `.kind()` discriminant.

### Not Yet Implemented

- Temporal search execution (falls back to hybrid with warning)
- Full scope enforcement upstream (namespace only pushed to semantic-memory)
- Projection persistence (in-memory only)
- Projection rebuild execution (tracker only, callers drive rebuilds)
- Forge causal projection adapter
- Fuzzy entity resolution

---

## semantic-memory-forge

**Version:** 0.2.0 | **Tests:** 144 | **License:** MIT

Causal edit attribution and structured patch evaluation engine. The verification, attribution, and evaluation authority in the stack.

### Authority

Authoritative for **raw verification state**: attempts, trials, eval runs, raw receipts, causal/experimental provenance, promotion/archive state, export envelopes.

Does NOT offload raw-truth interpretation into semantic-memory.

### Dependencies

semantic-memory, forge-policy, sandbox-workspace, typed-patch, effect-signature, check-runner, mindstate-core, stabilizer-core, cea-core, cea-store, cea-sqlite, tokio, serde/serde_json, async-trait, uuid, blake3, rusqlite, thiserror, anyhow, chrono, petgraph, walkdir, glob, regex, similar, rand, tracing, once_cell

### Features

- `default` = [] (no default features)
- `danger-sm-write` — enables direct write-through to semantic-memory (opt-in)

### Module Structure

| Module | Purpose |
|---|---|
| `config` | ForgeConfig, ForgeLimits, 50+ configuration options |
| `experiment` | Paired/repeated experiment runner, TypedLocatedEffect, ExperimentDiff |
| `export` | Episode export bridge to semantic-memory (EpisodeExport, to_import_envelope) |
| `failure` | FailureClass (9 variants), retry semantics |
| `invariants` | DB identity checks, forbidden paths, patch caps enforcement |
| `scoring` | Objective policies (BugFix, Refactor, SafetyHardening, Performance, Exploration) |
| `baseline` | BaselineDescriptor, ComparabilityPolicy, WorkspacePolicy |
| `store/` | SQLite backend, 18 tables (v1 core + v2 additive), schema hash verification |
| `lab/` | EvalTask, EvalSuite, ScoreVector, EvidenceBundle, HypothesisEdge, VerificationPlan, MAP-Elites archive, BasisVersion promotion |
| `cea/` | CausalGraph, EditOpSignature, EffectSignature, CausalPrediction, attribution |
| `exec/` | ExecutionBackend trait, host and container backends |
| `runtime/` | MindState compilation, novelty/stabilization, patch apply/validate/render |
| `adapters/` | CargoAdapter for Rust projects |

### Export Bridge

`EpisodeExport::to_import_envelope()` translates a Forge-owned export into a semantic-memory `ImportEnvelope` without leaking Forge semantics. Uses deterministic blake3 export keys. Export receipts tracked for idempotency.

### Key Invariants Enforced

- **I1:** DB identity check (SQLite magic bytes, PRAGMA user_version, schema_hash match)
- **I2:** Forbidden paths (tests, fixtures, Cargo.lock, .github)
- **I3:** Patch caps (max 8 files, 400 total lines, 200 per-file)
- **I9:** CEA no raw source (EditOpSignature/EffectSignature never contain raw code)

### Experiment Modes

- `Paired` — single baseline + patched run
- `RepeatedPaired` — multiple trials for statistical robustness
- `VerificationFollowup` — targeting specific verification steps

### Evidence & Claims

Phase 5 evidence bundles carry explicit claim strength: `ProvisionalSinglePair` ("provisional local attribution from one paired intervention on one fixed workload slice"). No false confidence in single-run results.

### Promotion Pipeline

Candidates → MAP-Elites archive (novelty × stability × approach_family) → BasisVersion promotion with graduation contract (min 95% suite pass rate, min 5% weighted improvement, max 15% stability variance, max 25% causal drift).

### Storage Schema (18 tables)

**V1 Core:** forge_meta, candidates, tasks, eval_runs, archive_cells, promotions, answer_traces, cea_nodes, cea_edges, cea_run_log

**V2 Additive:** evidence_bundles, experiment_runs, export_receipts, run_failures, verification_plans

---

## agent-graph

**Version:** 0.2.0 | **Tests:** 133 | **License:** MIT

Graph-based agent orchestration for Rust — LangGraph for the Rust ecosystem.

### Execution Model

Superstep-based execution (BSP variant):
1. Execute all active nodes in parallel (capped at max_parallelism=32)
2. Collect outputs, apply routing to determine next superstep's nodes
3. State merges use registered Reducer implementations for concurrent writes
4. Terminates when no more nodes queued or max_iterations exceeded (default 100)

### Public API

**Core:** `AgentGraph`, `AgentGraphBuilder`, `AgentState`, `GraphConfig`

**Nodes:** `Node` trait, `FnNode`, `PayloadNode`, `JoinNode`, `node!` macro

**Routing:** `RouterOutput`, `RoutingFunction`, `FnRouter`, `EdgeType`, `router!` macro

**Reducers:** `Reducer` trait, `LastWriteWins`, `AppendReducer`, `AddReducer`, `MergeReducer`, `FnReducer`

**State:** RwLock-based with typed get/set, transactions (ACID-like), fork for parallel branches, snapshot/restore history, limits (10K keys, 1MB per value)

**Interrupts:** `InterruptConfig`, `InterruptCheckpoint`, `ExecutionResult` — supports human-in-the-loop with AwaitInput, AwaitApproval, Custom kinds

**Events:** `EventSink` trait with 10 event types (RunStart/End, NodeStart/End, Token, Checkpoint, Interrupt, StateUpdate, SuperstepStart/End). Implementations: NoopEventSink, ChannelEventSink, CallbackEventSink, CompositeEventSink.

**Checkpointing:** Two-layer — legacy superstep-level (MemorySaver, SqliteSaver) + granular per-attempt (CheckpointStore, InMemoryCheckpointStore with RunId/AttemptId tracking). Graph hash validates topology on resume.

### Configuration

- `recursion_limit` (default 100), `max_parallelism` (default 8, capped 32)
- `thread_id`, `trace_id`, `tags`, `metadata`, `configurable`

### Error Types (14 variants)

NodeNotFound, RoutingError, StateError, MaxIterationsExceeded, CycleDetected, CheckpointError, CheckpointMismatch, ExecutionError, InterruptError, PayloadError, Cancelled, SerializationError, DatabaseError, Other

---

## llm-pipeline

**Version:** 0.2.0 | **Tests:** 206 | **License:** MIT

Reusable node payloads for LLM workflows: prompt templating, provider backends, defensive parsing, streaming, and sequential chaining.

### Architecture

```text
ExecCtx (backend, client, vars, limits, events, tracing)
  -> LlmCall (prompt template + output strategy + retry config)
    -> Backend::complete() (Ollama / OpenAI / Mock / Recording)
      -> LlmResponse { text, status, metadata }
        -> OutputStrategy parsing -> PayloadOutput { value, diagnostics }
```

### Backends

| Backend | Provider Coverage | Features |
|---|---|---|
| `OllamaBackend` | Ollama native API | /api/generate, /api/chat, NDJSON streaming |
| `OpenAiBackend` | OpenAI, Anthropic compat, vLLM, llama.cpp, LM Studio, Together AI, Groq, Mistral, Fireworks, Ollama /v1/ | /v1/chat/completions, SSE streaming, optional API key |
| `MockBackend` | Testing | Pre-configured responses, cycles when exhausted |
| `RecordingBackend` | Testing/replay | Captures requests/responses |

### Output Strategies

Lossy (always succeeds), Json (extraction + repair), StringList, XmlTag, Choice, Number, NumberInRange, Text, Custom

### Retry System

- **Semantic retry:** On parse failure, constructs correction prompt with error context, retries with lower temperature (max 5)
- **Transport retry:** Exponential backoff for HTTP 429/5xx with configurable jitter (None, Full, Bounded)

### Resource Limits (PipelineLimits)

- max_response_bytes (2MB), request_timeout (120s), stream_idle_timeout (30s)

### Key Types

- `Payload` trait — object-safe, composable work unit
- `LlmCall` — primary payload with prompt template, output strategy, retry config
- `Chain` — sequential payload composition
- `PayloadOutput` — value, raw_response, thinking, diagnostics, trace_id
- `ParseDiagnostics` — strategy used, parse_error, retry counts, repair actions, warnings
- `TraceId` — correlation ID for cross-system tracing

### Prompt System

`render(template, input, context)` — substitutes `{key}` from context, `{input}` from input. Escape: `{{` → `{`.

---

## llm-output-parser

**Version:** 0.2.0 | **Tests:** 144 | **License:** MIT

Production-grade parser for extracting structured data from LLM responses. Handles think blocks, markdown fences, malformed JSON, and real-world model output without an additional LLM call.

### Parsing Strategies (7)

1. Pure JSON arrays
2. Markdown code blocks (`` ```json `` / `` ``` ``)
3. JSON objects with known keys
4. `<think>` block stripping (DeepSeek R1 style)
5. Numbered lists
6. Comma-separated text
7. Line-separated text

### Key Functions

- `parse_tags(text)` — 7-strategy extraction
- `strip_think_tags(text)` — removes reasoning blocks
- `auto_complete_json(input)` — closes unclosed delimiters for streaming
- `StreamingJsonParser` — incremental JSON construction

Shared by both `llm-pipeline` and `ollama-vision`.

---

## job-queue

**Version:** 0.2.0 | **Tests:** 35 | **License:** MIT

Production-grade background job queue with SQLite persistence, priority ordering, worker leasing, retry with backoff, and cooperative cancellation.

### State Machine

```text
Pending -> Processing -> Completed
                      -> Failed (permanent or exhausted retries)
                      -> Cancelled

Processing (stale heartbeat) -> Pending (reclaimed)
Processing (crash recovery)  -> Pending (requeued on restart)

Failed (transient, retries left) -> Pending (scheduled retry with backoff)
Failed (rate_limited)            -> Pending (scheduled after delay)
```

### Public API

**QueueManager:** `new()`, `add()`, `cancel()`, `reorder()`, `pause()`, `resume()`, `list_jobs()`, `get_job_details()`, `prune()`, `count_by_status()`, `shutdown()`, `spawn()`, `process_one()`

**JobHandler trait:** `execute(ctx) -> Result<JobResult>`, `job_type() -> &str`

**FailureClass:** Transient (exponential backoff), Permanent (no retry), RateLimited (specific delay)

### Configuration

| Option | Default | Purpose |
|---|---|---|
| `db_path` | None (in-memory) | SQLite persistence |
| `worker_id` | UUID v4 | Lease holder identity |
| `poll_interval` | 3s | Job check frequency |
| `heartbeat_interval` | 10s | Lease keepalive |
| `stale_after` | 300s | Lease expiry threshold |
| `max_retries` | 3 | Max transient retry attempts |
| `cooldown` | 0s | Between job executions |

### Error Types (8 variants)

Database, Serialization, Execution, NotFound, InvalidTransition, Paused, Cancelled, Other

---

## ai-batch-queue

**Version:** 0.2.0 | **Tests:** 52 | **License:** MIT

Model-aware batch processing queue with ETA estimation for Tauri applications. Groups work by resource key to minimize expensive model swaps.

### Public API

**BatchQueue\<D\>:** `enqueue()`, `next_queued()`, `mark_running()`, `update_item()`, `mark_completed()`, `cancel_job()`, `cancel_item()`, `retry_failed()`, `estimate_remaining()`, `has_running_job()`

**BatchItemHandler\<D\> trait:** `process(data, resource_key, operation)`, `should_skip()`

**EtaTracker:** Bucketed by (resource_key, operation, SizeBucket) with Low/Medium/High confidence levels

### Scheduling

- Resource-aware reordering groups same-model jobs together
- Fairness limits prevent resource monopolization (max_consecutive_same_key)
- Deterministic lexicographic sort order

### Item Lifecycle

Pending -> Running -> Completed | Failed | Skipped | Cancelled

### Job Lifecycle

Queued -> Running -> Completed | CompletedWithErrors | Cancelled

---

## tauri-queue

**Version:** 0.3.0 | **Tests:** 26 | **License:** MIT

Tauri integration bridge for job-queue. Emits queue events to the Tauri frontend with configurable event coalescing and backpressure.

### Key Types

- **TauriEventEmitter** — bridges job-queue events to Tauri's `queue:*` event system
- **CoalescingEmitter** — wraps any emitter to suppress duplicate progress events with configurable interval
- **EmitterConfig** — buffer_size (256), drop_policy (DropNewest), coalesce_interval_ms (50), include_trace_id (true)
- **DropPolicy** — DropOldest, DropNewest, Block

Re-exports all job-queue types for backward compatibility.

---

## comfyui-rs

**Version:** 0.2.0 | **Tests:** 22 | **License:** MIT

Async Rust client for ComfyUI — REST, WebSocket progress, and workflow building.

### Public API

**ComfyClient:** `queue_prompt()`, `history()`, `queue_status()`, `image()`, `upload_image()`, `checkpoints()`, `samplers()`, `schedulers()`, `health()`, `free_memory()`, `interrupt()`

**Completion:** `wait_for_completion()` (polling), `wait_for_completion_ws()` (WebSocket with automatic polling fallback)

**Workflow:** `Txt2ImgRequest` builder for text-to-image workflows

### Error Types (8 variants)

Http, InvalidResponse, NodeErrors, Timeout, GenerationFailed, OutputTooLarge, Network, Json

### Configuration

- WsConfig: reconnect_attempts (3), reconnect_delay (1s), message_timeout (30s), max_messages_per_prompt (10K)
- DownloadLimits: max_image_bytes (100MB), download_timeout (60s)

---

## ollama-vision

**Version:** 0.2.0 | **Tests:** 6 | **License:** MIT

Robust Ollama vision model toolkit for image tagging and captioning. Works with any Ollama vision model (llava, minicpm-v, llama3.2-vision, etc.).

### Public API

- `tag_image()` / `tag_image_base64()` — extract tags from images using 7-strategy parser
- `caption_image()` / `caption_image_base64()` — generate captions
- `parse_tags()` — re-exported from llm-output-parser
- `strip_think_tags()` — remove `<think>` blocks from reasoning models

### Configuration

- `OllamaVisionConfig`: endpoint, model, timeout, connect_timeout, GenerateOptions (num_predict, temperature, top_p, repeat_penalty)
- `TagOptions`: prompt override, json format flag, max_tags (30), max_tag_length (50), max_retries (2)
- `CaptionOptions`: prompt override, max_caption_length (500), max_retries (2)

### Error Types

TagError (5 variants), CaptionError (5 variants) — both with `.kind()` discriminant

---

## @tauri-hooks/core

**Version:** 0.1.0 | **Language:** TypeScript | **License:** MIT

React hooks for Tauri 2 applications. Removes repetitive wiring of React frontends to Tauri commands and events.

### Hooks

| Hook | Purpose |
|---|---|
| `useTauriEvent` | Subscribe to one event with fresh handlers and safe cleanup |
| `useTauriEvents` | Subscribe to multiple events at once |
| `useTauriQuery` | Run a command and manage data/loading/error/refresh |
| `useTauriMutation` | Wrap a command as an explicit mutation |
| `useTauriConfig` | Load, update, save, and reload a config object |
| `useBufferedStream` | Batch high-frequency text/data updates into controlled renders |

Peer dependencies: react >= 18, @tauri-apps/api >= 2

---

## Primitive Crates

All at v0.1.0. These form the lower layer under `semantic-memory-forge`.

| Crate | Purpose |
|---|---|
| `typed-patch` | Structured patch model, validation, apply/render helpers |
| `sandbox-workspace` | Workspace staging and controlled file operations |
| `check-runner` | Host/container check execution with command output parsing |
| `effect-signature` | Stable effect identity for validation outputs |
| `forge-policy` | Path, DB, and policy invariants |
| `mindstate-core` | Serialized reasoning/state representation for generation |
| `stabilizer-core` | Novelty/stabilization logic for retry passes |
| `cea-core` | Causal edit attribution graph and prediction |
| `cea-store` | Store abstractions for CEA persistence |
| `cea-sqlite` | SQLite-backed CEA storage |

---

## Composition Patterns

### 1. Local Agent Runtime

`agent-graph` + `llm-pipeline` + `semantic-memory`

LangGraph-style orchestrator with local retrieval and durable context. Agent nodes call LLM via payloads, read/write memory via semantic-memory, route via graph edges.

### 2. Safe Autonomous Patching

`agent-graph` + `semantic-memory-forge` + `Primitives/*`

Agent generates, validates, scores, and learns from code patches. Experiment runner compares baseline vs patched, CEA attributes effects to edits, promotion pipeline graduates successful strategies.

### 3. Desktop AI Operations App

`job-queue` or `ai-batch-queue` + `tauri-queue` + `@tauri-hooks/core`

Long-running work with resumability, frontend progress, and high-frequency event handling. Model-aware batch scheduling minimizes resource swaps.

### 4. Multimodal Image Workflow

`ollama-vision` + `comfyui-rs` + `ai-batch-queue`

Captioning, tagging, reranking, and image generation pipelines with model-aware batching and ETA estimation.

### 5. Invariant-Enforced Knowledge Pipeline

`semantic-memory` + `knowledge-runtime` + `semantic-memory-forge`

Forge produces verified evidence → exports `ExportEnvelopeV1` → `forge-memory-bridge` transforms to `ProjectionImportBatchV1` → `semantic-memory` atomically ingests via non-public integration boundary → `knowledge-runtime` queries with projection lifecycle awareness, deterministic merge, and degradation warnings.

---

## Cross-Crate Contracts

### Authority Boundaries

| Crate | Authoritative For | Must Not Do |
|---|---|---|
| `semantic-memory` | Queryable knowledge state, search indexes, imported projections | Own Forge policy, interpret raw verification, decide promotion |
| `knowledge-runtime` | Query planning, merge policy, projection lifecycle interpretation | Persist source truth, synthesize hidden cross-store truth |
| `semantic-memory-forge` | Raw verification truth, experiments, evaluation lineage, export envelopes | Offload raw-truth interpretation into semantic-memory |

### Shared Identity Types

| Type | Owner | Used By |
|---|---|---|
| `TraceId` | semantic-memory | All crates for cross-boundary correlation |
| `EnvelopeId` | semantic-memory | Forge export bridge, import boundary |
| `EntityId` | knowledge-runtime | Entity registry, scope-aware resolution |
| `ProjectionId` | knowledge-runtime | Projection lifecycle tracking |
| `ScopeKey` | knowledge-runtime | Partitioned entity/projection management |

### Import Flow

```text
Forge EvidenceBundle
  -> EpisodeExport::from_bundle()
  -> EpisodeExport::to_import_envelope()   [bridge adapter]
  -> MemoryStore::import_envelope()         [atomic, idempotent]
  -> ImportReceipt { status, record_count, trace_id }
```

- Atomic per envelope (all-or-nothing transaction)
- Idempotent (composite key: envelope_id + schema_version + content_digest)
- Provenance preserved (`_import` metadata on every record)
- No partial visibility on failure

### Consistency Model

- Eventually consistent across stores
- Monotonic within a claim lineage
- No silent history rewrite
- Import staleness surfaced as `ProjectionImportStale` warning

---

## Dependency Graph

```text
                   ┌──────────────────────┐
                   │    agent-graph        │
                   │  (orchestration)      │
                   └──────────┬───────────┘
                              │ payloads
                   ┌──────────▼───────────┐
                   │    llm-pipeline       │
                   │  (model execution)    │
                   └──────┬───────────────┘
                          │ depends on
              ┌───────────▼──────────┐
              │  llm-output-parser   │◄──── ollama-vision
              │  (defensive parsing) │
              └──────────────────────┘

┌────────────────────────────────────────────────────┐
│                semantic-memory                      │
│  (retrieval, persistence, projection import)        │
└────────────┬───────────────────────┬───────────────┘
             │ read-only adapter     │ import envelope
┌────────────▼────────────┐   ┌──────▼───────────────┐
│  knowledge-runtime      │   │ semantic-memory-forge │
│  (planning, merge)      │   │ (verification, CEA)   │
└─────────────────────────┘   └──────────┬────────────┘
                                         │ depends on
                              ┌──────────▼────────────┐
                              │    Primitives/* (10)    │
                              │  (typed-patch, cea-*,   │
                              │   check-runner, etc.)   │
                              └─────────────────────────┘

┌────────────────┐   ┌───────────────┐   ┌───────────────────┐
│   job-queue    │──►│  tauri-queue   │──►│ @tauri-hooks/core │
│ (persistence)  │   │ (Tauri bridge) │   │ (React hooks)     │
└────────────────┘   └───────────────┘   └───────────────────┘
┌────────────────┐            ▲
│ ai-batch-queue │────────────┘
│ (model-aware)  │
└────────────────┘

┌────────────────┐   ┌────────────────┐
│ ollama-vision  │   │  comfyui-rs    │
│ (tagging)      │   │ (generation)   │
└────────────────┘   └────────────────┘
```

---

*Total across all Rust crates: 1,059 tests. All green as of snapshot date.*
