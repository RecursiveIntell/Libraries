# Libraries2 — Comprehensive Ecosystem Snapshot

> Generated: 2026-04-04
> Workspace resolver: Cargo workspace v2

This document is a detailed snapshot of every library in the Libraries2 ecosystem — a collection of reusable Rust crates and one TypeScript package forming the utility layer for a local-first AI agent runtime.

---

## Table of Contents

1. [Workspace Overview](#workspace-overview)
2. [stack-ids](#stack-ids) — Identity foundation
3. [job-queue](#job-queue) — Background job processing
4. [Tauri-Queue](#tauri-queue) — Tauri event bridge for job-queue
5. [AI-Batch-Queue](#ai-batch-queue) — Model-aware batch processing
6. [LLM-Pipeline](#llm-pipeline) — LLM execution payloads
7. [agent-graph](#agent-graph) — Graph-based agent orchestration
8. [ComfyUI-RS](#comfyui-rs) — Async ComfyUI client
9. [Ollama-Vision-RS](#ollama-vision-rs) — Ollama vision model toolkit
10. [llm-output-parser](#llm-output-parser) — LLM response parsing
11. [Tauri-React-Hooks](#tauri-react-hooks) — React hooks for Tauri 2
12. [constraint-compiler](#constraint-compiler) — Projection-to-inference graph compiler
13. [discovery-portfolio](#discovery-portfolio) — Experiment discovery portfolios
14. [spec-execution](#spec-execution) — Spec and proof surface
15. [federated-settlement](#federated-settlement) — Cross-runtime treaty settlement
16. [attestation-exchange](#attestation-exchange) — Attestation envelope contracts
17. [profile-runtime](#profile-runtime) — Constitution and profile composition
18. [remote-oracle-admission](#remote-oracle-admission) — Remote oracle contracts
19. [Non-Crate Directories](#non-crate-directories)

---

## Workspace Overview

**Workspace members** (build together via `cargo test --workspace`):
- stack-ids, job-queue, agent-graph, ComfyUI-RS, Ollama-Vision-RS, discovery-portfolio, spec-execution, federated-settlement, attestation-exchange, remote-oracle-admission

**Excluded from workspace** (need external deps or Tauri build env):
- AI-Batch-Queue, Tauri-Queue, LLM-Pipeline, constraint-compiler, profile-runtime, demo-tauri-libraries, llm-tool-runtime (symlink)

**Shared workspace dependencies:**
- `async-trait 0.1`, `serde 1` (derive), `serde_json 1`, `thiserror 2`, `tokio 1` (full), `tracing 0.1`, `chrono 0.4` (serde), `uuid 1` (v4, serde), `anyhow 1`, `schemars 0.8`, `blake3 1`

---

## stack-ids

> Opaque ID newtypes and shared identity primitives for the local-first AI systems stack

| Attribute | Value |
|-----------|-------|
| **Version** | 0.2.0 |
| **MSRV** | 1.75.0 |
| **License** | MIT |
| **Lines of Code** | ~2,435 |
| **Dependencies** | blake3, uuid, serde, serde_json, schemars |
| **Feature Flags** | None |

### Purpose

stack-ids is the **identity foundation** for the entire ecosystem. Every crate that handles trace correlation, retry lineage, or cross-crate identity depends on it. It provides opaque string-wrapper ID types, trace context primitives, content digests, and governance types.

### Public API

**Macro:**
- `define_id!` — Generates opaque string-wrapper ID newtypes with auto-derived `Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, JsonSchema` and methods: `new()`, `generate()`, `as_str()`, `is_empty()`, `Display`, `From<String>`, `FromStr`

**Structs:**
- `TraceCtx` — Canonical trace context for end-to-end correlation. Methods: `generate()`, `from_legacy_trace_id()`, `to_legacy_trace_id()`
- `ScopeKey` — Namespace + optional sub-scope for scoped queries. Methods: `namespace_only()`, `as_str()`
- `ContentDigest` — BLAKE3-based content hash. Methods: `compute()`, `compute_str()`, `compute_json<T>()`, `from_hex()`, `hex()`
- `V25ConstitutionCitation` — Canonical v25 constitutional references (applicability_context_id, profile_set_id, composition_receipt_id, effective_constitution_id, compiled_obligation_set_id, plus optional conflict set and exception bundle IDs)
- `DigestError` — Error type for digest operations

**Enum:**
- `SurfaceStatus` — Publication status: `AdvisoryOnly`, `NonAdmitted`, `Degraded`, `HorizonOnly`

**ID Types:** 217 unique ID newtypes organized by domain — core identity (EnvelopeId, ClaimId, AttemptId, TrialId...), kernel/execution (ConstraintId, RegionId, OracleSliceId...), control plane (VerificationCaseId, PolicyDecisionId...), governance (TreatyBundleId, SettlementCaseId...), profiles (ApplicabilityContextId, EffectiveConstitutionId...), and many more.

### Tests
6 unit tests covering trace context generation/roundtrip, scope keys, content digest determinism, SurfaceStatus serde, and V25 citation roundtrip. Complex integration test structs validate ID composition across domains.

---

## job-queue

> Production-grade background job queue with SQLite persistence

| Attribute | Value |
|-----------|-------|
| **Version** | 0.2.0 |
| **License** | MIT |
| **Lines of Code** | ~3,496 |
| **Dependencies** | tokio, rusqlite (bundled), serde, serde_json, anyhow, chrono, uuid, thiserror, tracing, stack-ids |
| **Feature Flags** | None |

### Purpose

Framework-agnostic background job queue engine with SQLite persistence, priority scheduling, retry lineage tracking, lease-based claiming, heartbeats, stale job reclamation, and crash recovery. Designed for production use in Tauri apps but works standalone.

### Public API

**Traits:**
- `JobHandler` — User-defined job type with `async fn execute()` and optional `job_type()` name
- `QueueEventEmitter` — Pluggable event sink for lifecycle notifications

**Structs:**
- `QueueManager` — High-level API: `add()`, `cancel()`, `reorder()`, `pause()`/`resume()`, `list_jobs()`, `prune()`, `process_one()`, `count_by_status()`, `spawn()`, `shutdown()`
- `QueueConfig` / `QueueConfigBuilder` — Builder for DB path, worker ID, cooldown, max consecutive, poll/heartbeat intervals, stale timeout, max retries
- `JobContext` — Passed to handlers: `emit_progress()`, `is_cancelled()`, trace context fields
- `QueueJob<T>` — Builder-pattern job wrapper with priority, trace context, attempt/trial IDs
- `QueueJobDetails` — Queryable runtime details (18 fields) for debugging/UI
- `JobResult` — Handler result: `success()`, `failure()`, `transient_failure()`, `rate_limited()` with `FailureClass`
- `QueueStats` — Aggregate counts by status
- `NoopEventEmitter`, `LoggingEventEmitter` — Built-in emitters

**Enums:**
- `QueuePriority` — `High` (1), `Normal` (2), `Low` (3)
- `QueueJobStatus` — `Pending`, `Processing`, `Completed`, `Failed`, `Cancelled`
- `FailureClass` — `Transient`, `Permanent`, `RateLimited { retry_after_secs }`
- `QueueError` — `Database`, `Serialization`, `Execution`, `NotFound`, `InvalidTransition`, `Paused`, `Cancelled`, `Other` — all with `.kind()` method

**Event Types:** `JobStartedEvent`, `JobCompletedEvent`, `JobFailedEvent`, `JobProgressEvent`, `JobCancelledEvent`

**Database:** SQLite with WAL mode, 18-column `queue_jobs` table, atomic claim via SELECT+UPDATE, worker ownership validation, retry lineage persistence.

---

## Tauri-Queue

> Tauri integration for job-queue with frontend event bridging

| Attribute | Value |
|-----------|-------|
| **Version** | 0.3.0 |
| **License** | MIT |
| **Lines of Code** | ~1,464 |
| **Dependencies** | job-queue, tauri 2, tokio, serde, serde_json, stack-ids, tracing |
| **Feature Flags** | `sqlite` (default) |

### Purpose

Bridge layer between Tauri applications and the job-queue backend. Translates job-queue events into Tauri frontend events with `queue:` prefix, provides backpressure management via coalescing, and propagates trace context.

### Public API

**Structs:**
- `TauriEventEmitter` — Emits job-queue events to Tauri frontend. Methods: `new(app_handle)`, `arc(app_handle)`
- `CoalescingEmitter` — Throttled wrapper with configurable buffer, drop policy, and coalesce interval. Methods: `new(inner, config)`, `arc(inner, config)`
- `EmitterConfig` — Buffer size (256), drop policy (DropNewest), coalesce interval (50ms), trace context inclusion

**Enums:**
- `DropPolicy` — `DropOldest`, `DropNewest`, `Block`

**Functions:**
- `trace_ctx_from_event_trace_id()` — Extract TraceCtx from legacy trace_id

**Re-exports:** All job-queue types (`QueueConfig`, `QueueManager`, `QueueJob`, `QueueJobStatus`, `QueuePriority`, `JobResult`, all event types, `JobHandler`, `QueueEventEmitter`, `JobContext`) plus `stack_ids::TraceCtx`.

### Tests
Integration tests covering queue lifecycle, persistence, crash recovery, pause/resume, coalescing, trace context bridging, event serialization.

---

## AI-Batch-Queue

> Model-aware batch processing queue with ETA estimation for Tauri apps

| Attribute | Value |
|-----------|-------|
| **Version** | 0.2.0 |
| **License** | MIT |
| **Lines of Code** | ~1,819 |
| **Dependencies** | tauri 2, tokio, serde, serde_json, anyhow, chrono, uuid, thiserror, tracing, stack-ids |
| **Feature Flags** | None |

### Purpose

Resource-aware batch processing queue that minimizes expensive resource swaps (e.g., GPU model loads) by grouping jobs by resource key. Tracks processing durations for accurate ETA predictions. Designed for AI model inference workloads in Tauri apps.

### Public API

**Traits:**
- `BatchItemHandler<D>` — Process individual items: `async fn process()`, `fn should_skip()`
- `BatchStore<D>` — Optional persistence: `save_job()`, `load_all()`, `delete_job()`

**Structs:**
- `BatchQueue<D>` — In-memory queue with model-aware reordering: `enqueue()`, `next_queued()`, `mark_running()`, `update_item()`, `mark_completed()`, `cancel_item()`, `cancel_job()`, `retry_failed()`, `estimate_remaining()`, `list_jobs()`
- `BatchJob<D>` — Multi-item batch job with resource_key, operation, overwrite policy, status, trace context
- `BatchItem<D>` — Individual item with status, size bucket, TraceCtx, AttemptId, TrialId
- `BatchCompletionSummary` — Job completion stats (total, succeeded, failed, skipped, durations)
- `EtaEstimate` — Structured prediction: remaining_ms, items_remaining, avg_item_ms, confidence, sample_count
- `EtaTracker` — Duration tracker keyed by (resource, operation, size_bucket)
- `SchedulingConfig` — Fairness: max_consecutive_same_key (3), resource_switch_cooldown, enable_reordering
- `ItemResult` — Item result: `success()`, `success_with_output()`, `failure()`

**Enums:**
- `BatchItemStatus` — `Pending`, `Running`, `Completed`, `Failed`, `Skipped`, `Cancelled`
- `BatchJobStatus` — `Queued`, `Running`, `Completed`, `CompletedWithErrors`, `Cancelled`
- `OverwritePolicy` — `Skip`, `Overwrite`
- `SizeBucket` — `Small` (<500K px), `Medium` (<2M px), `Large` (>=2M px), `Unknown` — with `from_pixel_count()`, `from_dimensions()`
- `EtaConfidence` — `Low`, `Medium` (3-9 samples), `High` (>=10)

**Functions:**
- `build_job()` — Construct a BatchJob from items
- `build_job_traced()` — With trace context (assigns fresh AttemptIds)
- `spawn()` / `spawn_with_interval()` — Background batch processor emitting `ai_batch:` prefixed Tauri events

---

## LLM-Pipeline

> Reusable node payloads for LLM workflows

| Attribute | Value |
|-----------|-------|
| **Version** | 0.2.0 |
| **License** | MIT |
| **Lines of Code** | ~9,084 |
| **Dependencies** | tokio, reqwest, serde, serde_json, anyhow, thiserror, futures, async-trait, chrono, fastrand, uuid, tracing, llm-output-parser, llm-tool-runtime, stack-ids |
| **Feature Flags** | `yaml` (optional YAML parsing), `openai` (OpenAI backend) |

### Purpose

Production-ready execution layer for LLM workflows. Provides object-safe payload abstractions, multi-backend support (Ollama, OpenAI), defensive output parsing with automatic repair, semantic and transport retry separation, streaming, sequential chaining, and tool-use loops. Designed to run inside graph orchestration nodes (agent-graph) while keeping routing/checkpoint logic in the orchestrator.

### Public API

**Core Trait:**
- `Payload` — Object-safe: `kind()`, `name()`, `invoke(ctx, input)` async

**Core Structs:**
- `ExecCtx` / `ExecCtxBuilder` — Shared execution context: HTTP client, backend, endpoint, backoff, vars, cancellation, event handler, trace context, limits
- `LlmCall` — Primary LLM payload with prompt/system template, model, config, streaming, output strategy, retry, timeout. Builder: `new()`, `with_system()`, `with_model()`, `with_config()`, `with_streaming()`, `with_output_strategy()`, `with_retry()`, `expecting_json()`
- `PayloadOutput` — Output with parsed value, raw response, thinking, model, diagnostics, trace context, retry counts, timing
- `Chain` — Sequential payload composition: `push()`, `execute_all()`, `execute()`
- `LlmRequest` / `LlmResponse` — Normalized backend request/response
- `ParseDiagnostics` — Parse telemetry: strategy used, errors, retries, repairs, warnings, attempt/trial IDs
- `RetryConfig` — Semantic retry on parse failure with optional validator and cooldown
- `PipelineLimits` — Resource caps: 2MB response, 120s timeout, 30s stream idle
- `StreamingDecoder` — NDJSON buffering decoder
- `ToolLoopRunner` / `ToolLoopRequest` / `ToolLoopResponse` — Tool-use orchestration
- `LlmConfig` — Temperature, max_tokens, thinking, json_mode

**Output Strategy Enum:**
- `OutputStrategy` — `Lossy` (default), `Json`, `StringList`, `XmlTag(String)`, `Choice(Vec<String>)`, `Number`, `NumberInRange(f64, f64)`, `Text`, `Custom(fn)`

**Backend Types:**
- `Backend` trait — `complete()`, `complete_streaming()`, `name()`
- `OllamaBackend` — Default Ollama API
- `OpenAiBackend` — Feature-gated OpenAI-compatible API
- `MockBackend` — Canned responses for testing
- `RecordingBackend` — Record/replay decorator
- `BackoffConfig` — Transport retry: max_retries, delays, multiplier, jitter strategies (`None`, `Full`, `Equal`, `Decorrelated`)

**Events:** `Event` enum (`PayloadStart`, `Token`, `PayloadEnd`, `RetryStart`, `RetryEnd`, `PartialParse`, `TransportRetry`) + `EventHandler` trait

**Error:** `PipelineError` — `Request`, `Json`, `Parse`, `StageFailed`, `Cancelled`, `InvalidConfig`, `HttpError`, `ResponseTooLarge`, `StreamIdle`, `Timeout`, `Other` — with `.kind()`

**Legacy API (deprecated):** `Pipeline<T>`, `Stage`, `call_llm()`, `call_llm_chat()`

---

## agent-graph

> Graph-based agent orchestration — LangGraph for Rust

| Attribute | Value |
|-----------|-------|
| **Version** | 0.2.0 |
| **License** | MIT |
| **Lines of Code** | ~4,702 |
| **Dependencies** | tokio, futures, async-trait, serde, serde_json, thiserror, chrono, uuid, tracing, stack-ids, rusqlite (optional) |
| **Feature Flags** | `checkpointing` (default) — SQLite-backed checkpoint persistence |

### Purpose

Full-featured graph-based agent orchestration engine supporting deterministic DAG execution, parallel fan-out/fan-in, conditional routing, checkpointing/state persistence, interrupt/resume for human-in-the-loop workflows, retry policies, structured event streaming, and subgraph composition.

### Public API

**Core Structs:**
- `AgentGraph` — Main orchestrator: `execute()`, `execute_with_config()`, `execute_with_interrupt()`, `stream()`, `resume()`, `get_state()`, `get_state_history()`, `update_state()`, `to_mermaid()`, `compute_graph_hash()`
- `AgentGraphBuilder` — Fluent construction: `add_node()`, `add_node_with_retry()`, `add_subgraph()`, `add_edge()`, `add_conditional_edge()`, `set_entry_point()`, `set_finish_point()`, `with_reducer()`, `with_interrupt_before/after()`, `with_checkpointer()`, `with_event_sink()`, `build()`
- `AgentState` — Shared state with `tokio::sync::RwLock`, transactions, snapshots, history, resource limits: `get()`, `set()`, `apply_reducer()`, `transaction()`, `fork()`, `snapshot()`, `restore()`
- `StateTransaction` — MVCC-style: `get()`, `set()`, `commit()`, `rollback()`
- `GraphConfig` — Runtime config: thread_id, trace_ctx, recursion_limit, max_parallelism, tags, metadata
- `RetryPolicy` — Configurable: max_attempts, initial_interval, backoff_factor, max_interval, jitter
- `InterruptConfig` / `InterruptCheckpoint` — Interrupt/resume data

**Node Types:**
- `FnNode<F>` — Function-based node
- `JoinNode` — Fan-in merge: `collect_array()`, `merge_objects()`
- `PayloadNode` — Wraps a `Payload` for external work (LLM calls)

**Navigation:**
- `Command` — State updates + navigation: `goto()`, `end()`, `update()`
- `SendOp` — Dynamic fan-out to specific nodes
- `Interrupt` — Raised by nodes: `await_input()`, `await_approval()`, `custom()`

**Traits:**
- `Node` — `execute(state, config)` async, `name()`
- `RoutingFunction` — `route(state, config)` for conditional edges
- `Reducer` — `reduce(current, update)` for parallel merge
- `Payload` — External work integration
- `EventSink` — `emit(GraphEvent)` for monitoring
- `Executor` — Node execution strategy
- `CheckpointSaver` / `CheckpointStore` — Persistence abstractions

**Built-in Reducers:** `LastWriteWins`, `AppendReducer`, `AddReducer`, `MergeReducer`, `FnReducer<F>`

**Event Sinks:** `NoopEventSink`, `ChannelEventSink`, `CallbackEventSink<F>`, `CompositeEventSink`

**Checkpoint Stores:** `MemorySaver`, `SqliteSaver`, `InMemoryCheckpointStore`

**Macros:**
- `node!` — Inline function nodes: `node!(|state| async move { ... })` or `node!("name", |state, config| async move { ... })`
- `router!` — Inline routing functions

**Enums:** `NodeOutput` (Done/Command), `Navigation` (Node/Nodes/End/Send/Default), `ExecutionResult` (Complete/Interrupted), `AgentGraphError` (12 variants with `.kind()`), `GraphEvent` (10 variants), `StreamEvent`, `StreamMode`, `AttemptStatus`, `RunStatus`

**Constants:** `START = "__start__"`, `END = "__end__"`

### Tests
11 test files: checkpointer, execution, integration, interrupt, parallel, reducer, retry, routing, runtime, state, streaming. 15 examples including basic, checkpointing, human-in-loop, map-reduce, subgraph, visualization.

---

## ComfyUI-RS

> Async Rust client for ComfyUI — REST, WebSocket progress, workflow building

| Attribute | Value |
|-----------|-------|
| **Version** | 0.2.0 |
| **License** | MIT |
| **Lines of Code** | ~1,730 |
| **Dependencies** | reqwest, tokio, tokio-tungstenite, futures-util, serde, serde_json, thiserror, rand, tracing, stack-ids |
| **Feature Flags** | None |

### Purpose

Complete async Rust client for ComfyUI with REST API operations (queue prompts, fetch history, download images, model discovery), real-time WebSocket progress tracking with automatic polling fallback, and a fluent workflow builder for txt2img generation.

### Public API

**Structs:**
- `ComfyClient` — Main client: `health()`, `queue_prompt()`, `history()`, `image()`, `queue_status()`, `checkpoints()`, `samplers()`, `schedulers()`, `free_memory()`, `interrupt()`, `upload_image()`, `wait_for_completion()`, `wait_for_completion_ws()` — plus 10 traced variants with TraceCtx
- `Txt2ImgRequest` — Fluent workflow builder: `new(prompt, checkpoint)`, `.negative()`, `.size()`, `.steps()`, `.cfg_scale()`, `.sampler()`, `.scheduler()`, `.seed()`, `.batch_size()`, `.filename_prefix()`, `build()` → (workflow JSON, seed)
- `ComfyProgress` — Rich progress: current_step, total_steps, node_id, prompt_id
- `ProgressUpdate` — Simple progress: current_step, total_steps
- `WsConfig` — WebSocket settings: reconnect attempts (3), delay (1s), message timeout (30s), max messages per prompt (10K), max total (50K)
- `DownloadLimits` — Safety: max 100MB, 60s timeout
- `ImageRef`, `PromptHistory`, `QueueStatus`

**Enums:**
- `ComfyError` — `Http`, `InvalidResponse`, `NodeErrors`, `Timeout`, `GenerationFailed`, `OutputTooLarge`, `Network`, `Json` — with `.kind()`
- `ComfyStatus` — `Queued`, `Running`, `Progress`, `Completed`, `Failed`
- `GenerationOutcome` — `Completed { images }`, `Failed { error }`, `TimedOut`

### Tests
Inline unit tests in workflow.rs (12 tests) and client.rs (3 tests). 3 examples: simple_generation, progress_tracking, workflow_builder.

---

## Ollama-Vision-RS

> Robust Ollama vision model toolkit for image tagging and captioning

| Attribute | Value |
|-----------|-------|
| **Version** | 0.2.0 |
| **License** | MIT |
| **Lines of Code** | ~671 |
| **Dependencies** | reqwest, serde, serde_json, tokio, thiserror, base64, llm-output-parser |
| **Feature Flags** | None |

### Purpose

Production-ready library for image analysis using Ollama vision models (llava, minicpm-v, llama3.2-vision). Provides image tagging (extracts tags via 7-strategy parser) and captioning (with automatic think-block stripping). Emphasizes robustness: handles malformed LLM output, supports base64 in-memory images, configurable truncation/retry.

### Public API

**Structs:**
- `OllamaVisionConfig` — Builder: `with_model()`, `.endpoint()`, `.timeout()`, `.connect_timeout()`, `.options()`
- `GenerateOptions` — num_predict, repeat_penalty, temperature, top_p
- `TagOptions` — prompt, request_json_format, max_tags (30), max_tag_length (50), max_retries (2)
- `CaptionOptions` — prompt, max_caption_length (500), max_retries (2)

**Functions (async):**
- `tag_image()` / `tag_image_base64()` → `Result<Vec<String>, TagError>`
- `caption_image()` / `caption_image_base64()` → `Result<String, CaptionError>`
- `parse_tags()` — 7-strategy parser (re-exported from llm-output-parser)
- `strip_think_tags()` — Remove `<think>` blocks

**Enums:**
- `TagError` — `Connection`, `OllamaError`, `InvalidResponse`, `ImageRead`, `Parse` — with `.kind()`
- `CaptionError` — `Connection`, `OllamaError`, `InvalidResponse`, `ImageRead`, `EmptyCaption` — with `.kind()`

### Tests
5 tests covering truncation limits, parse integration, UTF-8 safety. 3 examples: tag_images, caption_images, thinking_mode.

---

## llm-output-parser

> Production-grade parser for extracting structured data from LLM responses

| Attribute | Value |
|-----------|-------|
| **Version** | 0.2.0 |
| **License** | MIT |
| **Lines of Code** | ~3,425 |
| **Dependencies** | serde, serde_json, thiserror, serde_yaml (optional) |
| **Feature Flags** | `yaml` — enables YAML parsing via serde_yaml |

### Purpose

Standalone parser library for extracting structured data from messy LLM output. Handles think blocks, markdown fences, malformed JSON, and real-world model output patterns without an additional LLM call. Used by Ollama-Vision-RS and LLM-Pipeline.

### Public API

**Standard Parsers:**
- `parse_json<T>()` — Extract typed JSON struct
- `parse_json_value()` — Extract untyped JSON
- `parse_string_list()` — Extract cleaned string lists (tags, items)
- `parse_string_list_raw()` — String lists without cleaning
- `parse_xml_tag()` / `parse_xml_tags()` — Extract content from XML tags
- `parse_choice()` — Extract a choice from valid options
- `parse_number()` / `parse_number_in_range()` — Extract numeric values
- `parse_text()` — Clean text extraction
- `parse_yaml()` (feature: yaml) — Extract typed YAML

**Traced Variants** (return `ParseTrace` diagnostics):
- `parse_json_with_trace`, `parse_json_value_with_trace`, `parse_string_list_with_trace`, `parse_xml_tag_with_trace`, `parse_xml_tags_with_trace`, `parse_choice_with_trace`, `parse_number_with_trace`, `parse_number_in_range_with_trace`, `parse_text_with_trace`

**Utilities:**
- `strip_think_tags()` — Remove `<think>...</think>` blocks
- `try_repair_json()` — Fix common LLM JSON errors (unclosed braces, trailing commas, etc.)
- `preprocess()` — Combined preprocessing pipeline

**Types:**
- `ParseError` — Error enum (thiserror v2)
- `ParseOptions` — Safety limits and behavior toggles
- `ParseTrace` — Diagnostic output from traced calls

**Modules:** choice, error, extract, json, list, number, repair, text, xml, yaml (feature-gated)

---

## Tauri-React-Hooks

> React hooks for Tauri 2 apps

| Attribute | Value |
|-----------|-------|
| **Package** | @tauri-hooks/core |
| **Version** | 0.1.0 |
| **License** | MIT |
| **Lines of Code** | ~518 (TypeScript) |
| **Peer Deps** | react >= 18, @tauri-apps/api >= 2 |
| **Build** | tsup → ESM + CJS with TypeScript declarations |

### Purpose

React hooks library abstracting Tauri 2 integration patterns — async-safe event listeners, command invocation with query/mutation state, config management, and high-frequency data stream buffering. Frontend counterpart to Tauri-Queue and AI-Batch-Queue.

### Public Exports

**Hooks (6):**
- `useTauriEvent<T>` — Subscribe to a single Tauri event with async-safe cleanup
- `useTauriEvents` — Subscribe to multiple Tauri events atomically
- `useTauriQuery<T>` — Command invocation with loading/error/data state, auto-refresh on events
- `useTauriMutation<TArgs, TResult>` — Explicit mutation wrapper without auto-execution
- `useTauriConfig<T>` — Load, update, save, reload config objects
- `useBufferedStream<K>` — Batch high-frequency text/data updates with controlled render intervals

**Types (10):**
- `TauriEventHandler<T>`, `EventBindings`, `TauriQueryOptions`, `TauriQueryState<T>`, `TauriMutationOptions<TResult>`, `TauriMutationState<TArgs, TResult>`, `TauriConfigState<T>`, `BufferedStreamOptions`, `BufferedStreamState<K>`, `DependencyList`

### Key Features
- Async-safe cleanup preventing unmounted component setState warnings
- Handler ref pattern (fresh handlers without re-subscribing)
- Event-driven auto-refresh for queries
- Two-layer buffering (pendingRef for sync writes + state for flushed output)
- Optimistic local updates in config hook
- 75-91% code reduction vs. custom hooks (per demo-usage examples)

---

## constraint-compiler

> Deterministic projection-to-inference graph compiler

| Attribute | Value |
|-----------|-------|
| **Version** | 0.1.0 |
| **MSRV** | 1.75.0 |
| **License** | MIT |
| **Lines of Code** | ~1,372 |
| **Dependencies** | blake3, serde, serde_json, schemars, stack-ids, forge-memory-bridge, recursive-kernel-core, semantic-memory-forge |
| **Feature Flags** | None |
| **Workspace** | Excluded (needs external sibling deps) |

### Purpose

Transforms canonical projection import batches into bounded inference graph artifacts. Produces nodes, hyperedges, constraints, invalidation cones, degradation markers, and oracle slice candidates. Deterministic: same input always produces identical output including content hash.

### Public API

**Function:**
- `compile_batch(batch, policy)` → `CompileOutput` — Main compiler entry point

**Structs:**
- `CompilerPolicy` — policy_version, include_hyperedges
- `CompileOutput` — graph_hash, scope_key, geometry_manifest, nodes, hyperedges, constraints, regions, invalidation_cones, degradations, oracle_candidates
- `InferenceNode` — node_id, kind
- `InferenceHyperedge` — edge_id, member_node_ids
- `InvalidationCone` — source node → affected nodes, hyperedges, constraints
- `OracleSliceCandidate` — oracle_slice_id, node_ids (eligible if <=8 non-nuisance nodes, non-degraded)
- `CompilationBoundary` — from_surface, to_surface, artifact_families, deterministic
- `GraphGeometryManifest` — surfaces, boundaries, no_silent_collapse
- `CompiledRegion` — BFS-clustered connected subgraph with region_id, digest, nodes, hyperedges, constraints

**Enums:**
- `DegradationMarker` — `MissingClaimFamily`, `MissingAssertionGroup`, `MissingRelationGroup`, `ThinExport`
- `GraphSurfaceKind` — `Storage`, `Retrieval`, `Inference`, `Repair`, `Control`

### Tests
15 tests: determinism (including under record reordering), hyperedge grouping, nuisance state, invalidation cones, degradation handling, bug regressions, kernel integration, geometry/regions.

---

## discovery-portfolio

> Typed discovery portfolio surface with bounded budget evaluators

| Attribute | Value |
|-----------|-------|
| **Version** | 0.1.0 |
| **MSRV** | 1.75.0 |
| **License** | MIT |
| **Lines of Code** | ~415 |
| **Dependencies** | schemars, serde, stack-ids |
| **Feature Flags** | None |

### Purpose

Manages experiment discovery portfolios with bounded budget constraints and value-aware campaign selection. Keeps discovery as governed planning rather than heuristic score optimization.

### Public API

**Function:**
- `evaluate_portfolio_plan()` → `CampaignDecisionTraceV1` — Ranks campaigns by information-value-per-review-cost ratio, respects budget, generates explicit rationales

**Structs:**
- `DiscoveryProgramV1` — Program definition with owner and publication status
- `ProgramHypothesisSetV1` — Hypothesis references
- `InformationValueEstimateV1` — Expected information gain + estimated review cost
- `ExperimentCampaignV1` — Campaign with utility case and review requirements
- `PortfolioPlanV1` — Campaign IDs with utility rationale
- `VerificationLoadBudgetV1` — Total/remaining review slots with exhaustion tracking
- `CampaignDecisionLineV1` — Per-campaign decision with gain, cost, budget pressure, rationale
- `CampaignDecisionTraceV1` — Complete evaluation trace

**Enum:**
- `CampaignDecision` — `Launch`, `Defer`, `PauseBudgetExhausted`

**Constants:** 7 schema version strings

### Tests
4 tests across 3 files: budget exhaustion produces explicit pause (not silent skip), happy path launches campaigns, selector prefers better information-value-per-slot.

---

## spec-execution

> Typed spec and proof surface with bounded artifact evaluators

| Attribute | Value |
|-----------|-------|
| **Version** | 0.1.0 |
| **MSRV** | 1.75.0 |
| **License** | MIT |
| **Lines of Code** | ~526 |
| **Dependencies** | schemars, serde, serde_json, stack-ids |
| **Feature Flags** | None |

### Purpose

Manages generated artifacts in a specification and proof workflow. Defines schema bundles, specification artifacts, and proof evaluation receipts. Maintains governance baselines with human veto and challenge mechanisms.

### Public API

**Functions:**
- `generate_schema_bundle()` → `(GeneratedSchemaBundleV1, ProofEvaluationReceiptV1)` — Generates schema bundle with proof evaluation
- `generate_companion_bundles()` → `GeneratedCompanionBundles` — All companion artifacts in one call
- `establish_veto_challenge_baseline()` → `SelfHostingBuildReceiptV1` — Final governance receipt with rollback tracking

**Structs (13):**
- `SpecBundleV1`, `NormativeAstNodeV1`, `NormativeASTV1`, `GeneratedSchemaFileV1`, `GeneratedSchemaBundleV1`, `GeneratedInterpreterBundleV1`, `GeneratedConformanceCorpusV1`, `GeneratedMigrationPlanV1`, `ProofObligationV1`, `ProofObligationSetV1`, `ProofEvaluationReceiptV1`, `HumanVetoBundleV1`, `SelfHostingBuildReceiptV1`, `GeneratedCompanionBundles`

**Enums:**
- `GeneratedAdmissionState` — `AdvisoryOnly`, `NonAdmitted`, `HumanVetoed`
- `GeneratedSurfaceGovernanceState` — `AdvisoryBaseline`, `ChallengePending`, `HumanVetoed`

**Constants:** 11 schema version strings

### Tests
4 tests: backpointer integrity, blocking obligation evaluation, companion generation with self-hosting receipt, governance baseline with veto/challenge rollback.

---

## federated-settlement

> Typed treaty and settlement surface with bounded shared-view evaluators

| Attribute | Value |
|-----------|-------|
| **Version** | 0.1.0 |
| **MSRV** | 1.75.0 |
| **License** | MIT |
| **Lines of Code** | ~617 |
| **Dependencies** | schemars, serde, stack-ids |
| **Feature Flags** | None |

### Purpose

Cross-runtime settlement and treaty evaluation with explicit handling of degraded states, replay requirements, dissent preservation, and suspension artifacts. Ensures no silent success under missing replay, no shared disposition that erases local dissent.

### Public API

**Functions:**
- `evaluate_shared_replay(case)` → `SharedReplaySliceV1` — Identifies missing required replays
- `evaluate_divergence_or_suspension(case, replay)` → `(SharedDivergenceReportV1, Option<TreatySuspensionV1>)` — Analyzes blocking dissent/missing replays, recommends suspension
- `evaluate_settlement(case)` → `SettlementReceiptV1` — Main evaluation: `AdvisoryOnly`, `DegradedSharedView`, or `SharedDispositionIssued`

**Structs (13):**
- `TreatyPartyV1`, `TreatyBundleV1`, `RuntimeIdentityRecordV1`, `RuntimeIdentitySetV1`, `EquivalenceEvidenceV1`, `CrossRuntimeEquivalenceBundleV1`, `SharedDispositionV1`, `LocalDissentRecordV1`, `ReplayRequirementV1`, `SharedViewDowngradeV1`, `SettlementCaseV1`, `SettlementReceiptV1`, `SharedReplaySliceV1`, `SharedDivergenceReportV1`, `TreatySuspensionV1`

**Enums:**
- `SettlementDisposition` — `SharedDispositionIssued`, `AdvisoryOnly`, `DegradedSharedView`
- `DowngradeReason` — `MissingRequiredReplay`, `NonAdmittedSurface`

**Re-exports:** `SurfaceStatus`, `V25ConstitutionCitation` from stack-ids

**Constants:** 11 schema version strings

### Tests
6 tests across 4 files: settlement happy/degraded paths, blocking runtime visibility, suspension resumption, replay completeness, constitutional citation preservation.

---

## attestation-exchange

> Typed attestation exchange contracts for envelope, trust-root, and transparency artifacts

| Attribute | Value |
|-----------|-------|
| **Version** | 0.1.0 |
| **MSRV** | 1.75.0 |
| **License** | MIT |
| **Lines of Code** | ~184 |
| **Dependencies** | schemars, serde, stack-ids |
| **Feature Flags** | None |

### Purpose

Defines typed contracts for attestation envelopes, trust root sets, transparency receipts, and vendor certification adapters. Used by profile-runtime for vendor attestation integration.

### Public API

**Structs:**
- `AttestationEnvelopeV1` — Envelope with content digest, signer identity, trust root set, provenance, disclosure policy, replayability class, revocation/supersession refs
- `TrustRootSetV1` — Trust root identities, allowed signer classes, expiration/rotation policies, allowed artifact families
- `TransparencyReceiptV1` — Registry inclusion material, recorded time, admissibility judgment

**Vendor Profile Structs (P6):**
- `VendorCertificationAdapterV1` — Vendor name, product surface, covered artifact families, translation mode
- `VendorEvidenceTranslationV1` — Source shapes, canonical targets, lossy fields, required caveats
- `VendorTrustRootBindingV1` — Trust root refs, signer classes, rotation/revocation channels
- `VendorRevocationHandlingV1` — Revocation inputs, local invalidation actions, replay/admission impact

**Constants:** 3 schema version strings

### Tests
2 test files: profile P6 roundtrip and fixture conformance.

---

## profile-runtime

> Canonical effective constitution and profile composition runtime

| Attribute | Value |
|-----------|-------|
| **Version** | 0.1.0 |
| **MSRV** | 1.75.0 |
| **License** | MIT |
| **Lines of Code** | ~3,601 |
| **Dependencies** | serde, serde_json, schemars, thiserror, stack-ids, assurance-runtime, attestation-exchange, authority-delegation, continuity-runtime, verification-policy |
| **Feature Flags** | None |
| **Workspace** | Excluded (needs external sibling deps) |

### Purpose

Composes overlays from typed profile surfaces into one replayable effective constitution answer with typed conflicts and explicit exceptions. Handles applicability context selection, profile-set normalization, fold/conflict rule sets, and obligation compilation.

### Public API

**Core Function:**
- `compose_profile_runtime()` → `Result<CompositionOutcomeV1, CompositionError>` — Folds profile contributions + exceptions into effective constitution
- `diff_policy_impact()` → `PolicyImpactDiffV1` — Computes differences between two constitutions

**30+ Profile Adapter Functions** (`from_*` pattern):
- Policy: `from_effect_policy_profile`, `from_delegation_policy_profile`, `from_release_policy_profile`, `from_continuity_policy_profile`
- Privacy: `from_privacy_retention_profile`, `from_redaction_rule_set`, `from_access_purpose_matrix`, `from_audit_extraction_policy`, `from_residency_policy_profile`, `from_tenant_boundary_profile`, `from_cross_boundary_transfer_class`
- Authority: `from_role_catalog`, `from_delegation_matrix`, `from_approval_matrix`, `from_conflict_class_catalog`
- Regulatory: `from_regulatory_regime_profile`, `from_requirement_control_map`, `from_evidence_collection_plan`, `from_recertification_schedule`
- Risk: `from_hazard_library`, `from_hazard_scenario`, `from_monitor_catalog`, `from_mitigation_playbook`, `from_incident_taxonomy`, `from_severity_matrix`
- Attestation: `from_vendor_certification_adapter`, `from_vendor_evidence_translation`, `from_vendor_trust_root_binding`, `from_vendor_revocation_handling`
- Continuity: `from_pager_route_profile`, `from_escalation_clock_policy`

**Key Structs:**
- `ApplicabilityContextV1` — Anchors profile selection for one evaluation scope
- `ProfileRefGroupV1` — Collection of 30+ optional profile IDs
- `ProfileSetV1` — Normalized profile set with ordering
- `CompositionRuleSetV1` — Complete fold/conflict/exception rules with `reference_v1()` defaults
- `EffectiveConstitutionV1` — Canonical answer: base laws, doctrines, admitted profiles/exceptions, mode, summaries for all obligation categories
- `CompiledObligationSetV1` — All obligations with blocks, required approvals/checks/monitors
- `CompositionConflictSetV1` — Explicit conflicts with admissible exceptions
- `ProfileExceptionBundleV1` — Time-bounded exceptions with residual obligations
- `PolicyImpactDiffV1` — Constitution diff with behavior changes and migration consequences

**Enums:**
- `FoldClassV1` — `Intersection`, `Union`, `MinOfMaxima`, `MaxOfMinima`, `EarliestExpiry`, `ConflictIfDifferent`, `BlockDominant`
- `CompiledObligationKindV1` — 17 kinds: `Approval`, `Check`, `Monitor`, `Evidence`, `Disclosure`, `Residency`, `Tenancy`, `Assurance`, `Continuity`, `Effect`, `Delegation`, `Replay`, `Rollback`, `Compensation`, `PostHocReview`, `Warning`, `Block`
- `ConstitutionModeV1` — `Normal`, `Incident`, `AdvisoryOnly`, `Blocked`
- `CompositionError` — `ApplicabilityMismatch`, `UnsupportedFold`, `DigestFailed`

**Constants:** 10 schema version strings

### Tests
4 test files: roundtrip serialization, reference composition with block-dominant fold, fixture conformance, fixture manifest.

---

## remote-oracle-admission

> Typed remote oracle admission contracts for lease, result, replay, and re-admission artifacts

| Attribute | Value |
|-----------|-------|
| **Version** | 0.1.0 |
| **MSRV** | 1.75.0 |
| **License** | MIT |
| **Lines of Code** | ~700 |
| **Dependencies** | schemars, serde, stack-ids, attestation-exchange |
| **Feature Flags** | None |

### Purpose

Strongly-typed contract structures for remote oracle admission workflows — leases, requests, results, replay tickets, and attestation management (revocation/supersession). All structures validate on construction and embed V25 constitutional citations.

### Public API

**Structs (6 main contracts):**
- `RemoteOracleLeaseV1` — Permission contract: oracle identity, allowed artifact families, exactness/disclosure ceilings, budget ceiling, replay obligation, lease expiry, policy owners
- `RemoteSliceRequestV1` — Slice request: definition, required artifacts, disclosure policy, exactness target, trust root set, citation, challenge expectations
- `RemoteSliceResultV1` — Response: returned artifacts, exactness class, execution evidence, disclosure markers, replay handle, local admission recommendation
- `CrossRuntimeReplayTicketV1` — Replay mechanism: artifact refs, time coordinates, trust roots, allowed disclosure, lease window, failure behavior
- `AttestationRevocationV1` — Revocation contract: affected refs, reason, effective time, blast radius, invalidation behavior, dispute linkage
- `AttestationSupersessionV1` — Supersession contract: prior/replacement refs, semantic delta, replay impact, re-admission requirement

All structs have `new()` → `Result<Self, &'static str>` constructors with validation and `validate()` methods.

**Enums (6):**
- `RemoteExactnessClassV1` — `Exact`, `BoundedExact`
- `RemoteDisclosureClassV1` — `NonSensitive`, `RedactedStructuredOnly`
- `RemoteReplayObligationV1` — `MustReturnReplayTicketOrNonreplayableReason`
- `AttestationReplayImpactV1` — `ReplayTicketUnchanged`
- `LocalAdmissionRecommendationV1` — `Eligible`, `AdmitIfTransparencyReceiptPresent`, `AdmitWithDisclosureConstraints`
- `ReplayFailureBehaviorV1` — `Retry`, `DowngradeToAdvisoryAndEmitDisputeIfMandatory`

**Constants:** 6 schema version strings

### Tests
6 tests across 2 files: validation (empty policy owners, empty artifact refs, self-replacement rejection) and V25 constitutional citation preservation across request/result/ticket lifecycle.

---

## Non-Crate Directories

| Directory | Type | Purpose |
|-----------|------|---------|
| `conformance/` | Test fixtures | Conformance test suites organized by version (p1-p7, v16-v25) |
| `contracts/` | Shared fixtures | Fixture directory referenced by spec-execution, discovery-portfolio, federated-settlement tests |
| `apply/` | Shell scripts | CI patch application scripts (v25) |
| `overlay/` | Documentation | Master issue matrix, agent docs, prompts |
| `demo-tauri-libraries/` | Demo app | Tauri + React demo app showcasing the library ecosystem |
| `docs/` | Documentation | Architecture docs and guides |
| `examples/` | Code examples | Cross-crate usage examples |
| `plans/` | Planning docs | Implementation plans |
| `prompts/` | LLM prompts | Prompt templates for AI workflows |
| `reference/` | Reference docs | API and architecture reference |
| `scaffolds/` | Code scaffolds | Template scaffolds for new crates |
| `schemas/` + `schemas.generated/` | JSON Schemas | Generated JSON schemas for all typed surfaces |
| `snippets/` | Code snippets | Reusable code snippets |
| `release/` | Release tooling | Release scripts and config |
| `repo_overlay/` | Repo config | Repository overlay files |
| `llm-tool-runtime` | Symlink | → `/home/sikmindz/Coding/Libraries/llm-tool-runtime` |
| `.parser-lib` | Symlink | → `/home/sikmindz/Coding/Gloss/src-tauri/vendor/llm-output-parser` |

---

## Dependency Graph (Simplified)

```
stack-ids (identity foundation)
├── job-queue
│   └── Tauri-Queue (+ tauri)
├── AI-Batch-Queue (+ tauri)
├── ComfyUI-RS (+ reqwest, tokio-tungstenite)
├── Ollama-Vision-RS (+ reqwest, llm-output-parser)
├── agent-graph (+ rusqlite optional)
├── LLM-Pipeline (+ reqwest, llm-output-parser, llm-tool-runtime)
├── constraint-compiler (+ forge-memory-bridge, recursive-kernel-core, semantic-memory-forge)
├── discovery-portfolio
├── spec-execution
├── federated-settlement
├── attestation-exchange
│   └── remote-oracle-admission
└── profile-runtime (+ assurance-runtime, authority-delegation, continuity-runtime, verification-policy)
        └── attestation-exchange

llm-output-parser (standalone)
├── Ollama-Vision-RS
└── LLM-Pipeline

Tauri-React-Hooks (TypeScript, standalone)
```

---

## Ecosystem Statistics

| Metric | Value |
|--------|-------|
| **Total Rust crates** | 17 |
| **TypeScript packages** | 1 |
| **Total lines of code (est.)** | ~34,000+ |
| **Workspace members** | 10 |
| **Excluded crates** | 7 |
| **Shared ID types** | 217 |
| **thiserror version** | v2 (ecosystem-wide) |
| **Error `.kind()` convention** | All error enums |
| **Async runtime** | tokio 1.x |
| **Serialization** | serde + serde_json |
| **Schema generation** | schemars 0.8 |
| **Hashing** | BLAKE3 |
| **Tracing** | tracing 0.1 |
