# Libraries Reference

A comprehensive reference for every library and crate in this repository.

---

## Table of Contents

1. [AI-Batch-Queue](#ai-batch-queue) - Model-aware batch processing with ETA estimation
2. [ComfyUI-RS](#comfyui-rs) - Async Rust client for ComfyUI
3. [LLM-Pipeline](#llm-pipeline) - Production-grade LLM execution for workflow nodes
4. [Ollama-Vision-RS](#ollama-vision-rs) - Vision model toolkit for tagging and captioning
5. [agent-graph](#agent-graph) - Graph-based agent orchestration (LangGraph for Rust)
6. [job-queue](#job-queue) - Production-grade background job queue
7. [semantic-memory](#semantic-memory) - Hybrid semantic search with SQLite, FTS5, and HNSW
8. [Tauri-Queue](#tauri-queue) - Tauri integration for job-queue
9. [Tauri-React-Hooks](#tauri-react-hooks) - React hooks for Tauri 2 apps
10. [Primitives](#primitives) - Workspace of 10 crates for patch validation, execution, and causal attribution
11. [living-memory](#living-memory) - Causal edit attribution and structured patch evaluation engine

---

## AI-Batch-Queue

**Crate:** `ai-batch-queue` | **Version:** 0.2.0 | **License:** MIT | **Edition:** 2021

Model-aware batch processing queue with ETA estimation for Tauri applications. Automatically groups jobs by resource key to minimize expensive resource swaps (e.g., GPU model loads) and provides size-bucketed time estimates that improve as work completes.

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

### Architecture

```
ai-batch-queue/
├── lib.rs        -- Traits (BatchItemHandler, BatchStore) + build_job helper
├── types.rs      -- All enum and struct types
├── queue.rs      -- BatchQueue (Mutex-based thread-safe in-memory queue)
├── eta.rs        -- EtaTracker (bucketed timing statistics)
└── executor.rs   -- Tauri integration (background task spawning, event emission)
```

### Public API

#### Traits

**`BatchItemHandler<D>`** -- Main trait for implementing processing logic. Generic over item data type `D: Clone + Send + Sync + Serialize`.

| Method | Description |
|--------|-------------|
| `async fn process(&self, data: &D, resource_key: &str, operation: &str) -> Result<ItemResult>` | Process a single item |
| `fn should_skip(&self, data: &D, operation: &str) -> bool` | Check if item should be skipped (default: false) |

**`BatchStore<D>`** -- Optional persistence abstraction.

| Method | Description |
|--------|-------------|
| `fn save_job(&self, job: &BatchJob<D>) -> Result<()>` | Save/update a job |
| `fn load_all(&self) -> Result<Vec<BatchJob<D>>>` | Load all jobs for startup recovery |
| `fn delete_job(&self, job_id: &str) -> Result<()>` | Delete a completed/cancelled job |

#### Enums

| Enum | Variants | Description |
|------|----------|-------------|
| `BatchItemStatus` | `Pending`, `Running`, `Completed`, `Failed`, `Skipped`, `Cancelled` | Per-item lifecycle status |
| `BatchJobStatus` | `Queued`, `Running`, `Completed`, `CompletedWithErrors`, `Cancelled` | Overall job status |
| `OverwritePolicy` | `Skip`, `Overwrite` | Whether to skip or overwrite existing results |
| `SizeBucket` | `Small`, `Medium`, `Large`, `Unknown` | Item size classification for ETA estimation |
| `EtaConfidence` | `Low`, `Medium`, `High` | Confidence level of ETA estimate (based on sample count) |

#### Structs

**`BatchQueue<D>`** -- In-memory batch queue with reordering and ETA tracking.

| Method | Description |
|--------|-------------|
| `new()` | Create empty queue with default scheduling |
| `with_scheduling(config)` | Create with custom scheduling config |
| `enqueue(job)` | Add job, returns job ID |
| `next_queued()` | Get next queued job without removing |
| `mark_running(job_id)` | Mark job as running |
| `update_item(job_id, item_id, status, error, duration_ms)` | Update single item status |
| `mark_completed(job_id)` | Mark job complete, returns `BatchCompletionSummary` |
| `cancel_item(job_id, item_id)` / `cancel_job(job_id)` | Cancel item or job |
| `retry_failed(job_id)` | Reset failed items to Pending |
| `list_jobs()` / `get_job(job_id)` | Query jobs |
| `estimate_remaining(job_id)` | ETA with confidence and metadata |
| `has_running_job()` / `queued_count()` | Status checks |

**`BatchJob<D>`** -- A batch job containing multiple items sharing one resource. Fields: `id`, `resource_key`, `operation`, `overwrite_policy`, `items: Vec<BatchItem<D>>`, `status`, `created_at`, `started_at`, `completed_at`, `reordered`, `reorder_note`.

**`BatchItem<D>`** -- Single item within a batch job. Fields: `id`, `data: D`, `status`, `error`, `duration_ms`, `size_bucket`.

**`EtaEstimate`** -- Structured ETA. Fields: `remaining_ms`, `items_remaining`, `avg_item_ms`, `confidence`, `sample_count`.

**`BatchCompletionSummary`** -- Summary of completed job. Fields: `job_id`, `operation`, `resource_key`, `total`, `succeeded`, `failed`, `skipped`, `total_duration_ms`, `avg_duration_ms`.

**`ItemResult`** -- Result of processing a single item. Methods: `success()`, `success_with_output(String)`, `failure(String)`.

**`SchedulingConfig`** -- Controls: `max_consecutive_same_key` (default: 3), `resource_switch_cooldown` (default: 0), `enable_reordering` (default: true).

**`EtaTracker`** -- Internal ETA estimation engine. Records durations bucketed by (resource, operation, size). Confidence: 0-2 samples = Low, 3-9 = Medium, 10+ = High.

#### Helper Function

```rust
fn build_job<D>(resource_key: &str, operation: &str, overwrite_policy: OverwritePolicy,
    items: Vec<(String, D, SizeBucket)>) -> BatchJob<D>
```

### Key Features

- **Resource-aware reordering** -- Groups jobs by `resource_key` to minimize expensive resource swaps (GPU model loads)
- **Size-bucketed ETA estimation** -- Tracks processing durations by (resource, operation, size) for accurate predictions
- **Item-level status tracking** -- Each item has its own lifecycle with detailed status
- **Progressive completion with retry** -- Failed items can be retried without re-processing successful ones
- **Tauri event emission** -- `ai_batch:job_started`, `ai_batch:item_progress`, `ai_batch:job_completed`
- **Optional persistence** -- `BatchStore` trait for custom persistence adapters
- **Fairness controls** -- Prevents starvation with `max_consecutive_same_key`

### Example

```rust
let queue: BatchQueue<String> = BatchQueue::new();

let job = build_job(
    "llava:13b",
    "tag",
    OverwritePolicy::Skip,
    vec![
        ("img-1".into(), "/photos/cat.jpg".into(), SizeBucket::Medium),
        ("img-2".into(), "/photos/dog.jpg".into(), SizeBucket::Medium),
        ("img-3".into(), "/photos/sunset.jpg".into(), SizeBucket::Large),
    ],
);

let job_id = queue.enqueue(job)?;
```

---

## ComfyUI-RS

**Crate:** `comfyui-rs` | **Version:** 0.2.0 | **License:** MIT | **Edition:** 2021

Async Rust client for ComfyUI -- REST, WebSocket progress, and workflow building. Provides a strongly-typed API for interacting with ComfyUI's Stable Diffusion backend including real-time progress tracking via WebSocket with automatic polling fallback.

### Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| `reqwest` | 0.12 (json, multipart) | HTTP client |
| `tokio` | 1 (time) | Async runtime |
| `tokio-tungstenite` | 0.24 | WebSocket client |
| `futures-util` | 0.3 | Async stream handling |
| `serde` | 1 (derive) | Serialization |
| `serde_json` | 1 | JSON support |
| `thiserror` | 2 | Error derivation |
| `rand` | 0.9 | Seed generation |
| `tracing` | 0.1 | Structured logging |

### Architecture

```
comfyui-rs/
├── lib.rs       -- Public API exports
├── client.rs    -- REST + WebSocket client implementation
├── workflow.rs  -- Txt2ImgRequest builder (7-node ComfyUI pipeline)
├── types.rs     -- Data structures (ProgressUpdate, ImageRef, etc.)
└── error.rs     -- ComfyError enum
```

### Public API

#### ComfyClient

The main client struct. Stateless, Clone-able, holds no mutable state.

| Method | Description |
|--------|-------------|
| `new(endpoint)` | Create new client |
| `with_http_client(client)` | Use custom reqwest::Client |
| `with_client_id(id)` | Set client ID |
| `health()` | Check ComfyUI reachability |
| `queue_prompt(workflow)` | Submit workflow, get prompt_id |
| `history(prompt_id)` | Fetch history entry |
| `image(img)` | Download output image bytes |
| `queue_status()` | Get running/pending counts |
| `free_memory(unload_models)` | Free VRAM |
| `interrupt()` | Stop running generation |
| `upload_image(bytes, filename, overwrite)` | Upload image to server |
| `checkpoints()` / `samplers()` / `schedulers()` | List available resources |
| `wait_for_completion(prompt_id, timeout)` | Poll until done |
| `wait_for_completion_ws(prompt_id, timeout, on_progress)` | WebSocket with polling fallback |

#### Txt2ImgRequest

Builder for text-to-image workflows. Generates a 7-node ComfyUI pipeline.

| Method | Description |
|--------|-------------|
| `new(prompt, checkpoint)` | Create with defaults (512x768, 25 steps, cfg 7.5) |
| `negative(prompt)` | Set negative prompt |
| `size(width, height)` | Set output dimensions |
| `steps(n)` | Set sampling steps |
| `cfg_scale(cfg)` | Set classifier-free guidance |
| `sampler(name)` / `scheduler(name)` | Set sampler/scheduler |
| `seed(seed)` | Set specific seed (-1 for random) |
| `batch_size(n)` | Number of images per generation |
| `filename_prefix(prefix)` | Output filename prefix |
| `build()` | Build workflow JSON, returns `(Value, i64)` |

**7-Node Pipeline:** CheckpointLoaderSimple -> EmptyLatentImage -> CLIPTextEncode (positive) -> CLIPTextEncode (negative) -> KSampler -> VAEDecode -> SaveImage

#### Data Types

| Type | Description |
|------|-------------|
| `ProgressUpdate` | Real-time progress: `current_step`, `total_steps` |
| `ComfyProgress` | Richer progress with `node_id` and `prompt_id` |
| `ComfyStatus` | Enum: `Queued`, `Running`, `Progress`, `Completed`, `Failed` |
| `ImageRef` | Reference to output image: `filename`, `subfolder`, `img_type` |
| `PromptHistory` | Parsed history entry: `status`, `completed`, `images` |
| `QueueStatus` | Snapshot: `running`, `pending` |
| `GenerationOutcome` | Enum: `Completed { images }`, `Failed { error }`, `TimedOut` |
| `WsConfig` | WebSocket config: reconnect attempts, delays, message limits |
| `DownloadLimits` | Image download config: max bytes (100MB), timeout (60s) |

#### Error Type

`ComfyError` variants: `Http`, `InvalidResponse`, `NodeErrors`, `Timeout`, `GenerationFailed`, `OutputTooLarge`, `Network`, `Json`. Method: `kind() -> &'static str`.

### Example

```rust
use comfyui_rs::{ComfyClient, GenerationOutcome, Txt2ImgRequest};
use std::time::Duration;

let client = ComfyClient::new("http://127.0.0.1:8188");
let checkpoints = client.checkpoints().await?;

let (workflow, seed) = Txt2ImgRequest::new("a sunset over mountains", &checkpoints[0])
    .negative("lowres, blurry")
    .size(768, 1024)
    .steps(25)
    .build();

let prompt_id = client.queue_prompt(&workflow).await?;
let result = client
    .wait_for_completion_ws(&prompt_id, Duration::from_secs(120), |p| {
        println!("step {}/{}", p.current_step, p.total_steps);
    })
    .await?;

if let GenerationOutcome::Completed { images } = result {
    for img in &images {
        let bytes = client.image(img).await?;
        println!("downloaded {} bytes", bytes.len());
    }
}
```

---

## LLM-Pipeline

**Crate:** `llm-pipeline` | **Version:** 0.2.0 | **License:** MIT | **Edition:** 2021

Production-grade LLM execution for Rust workflow nodes. Reusable node payloads for LLM workflows with prompt templating, multi-backend support (Ollama, OpenAI), defensive parsing, streaming, transport and semantic retry, and sequential chaining.

### Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| `tokio` | 1 (full) | Async runtime |
| `reqwest` | 0.12 (json, stream) | HTTP client |
| `serde` | 1 (derive) | Serialization |
| `serde_json` | 1 | JSON support |
| `anyhow` | 1 | Error handling |
| `thiserror` | 2 | Error derivation |
| `futures` | 0.3 | Async utilities |
| `async-trait` | 0.1 | Async trait support |
| `fastrand` | 2 | Fast random numbers |
| `uuid` | 1 (v4) | Trace IDs |
| `tracing` | 0.1 | Structured logging |
| `llm-output-parser` | path | Shared output parsing library |

### Feature Flags

| Feature | Default | Description |
|---------|---------|-------------|
| `yaml` | No | YAML parsing via `llm-output-parser/yaml` |
| `openai` | No | OpenAI-compatible backend support |

### Architecture

```
llm-pipeline/
├── lib.rs              -- Crate root, public API exports
├── error.rs            -- PipelineError, Result<T>
├── exec_ctx.rs         -- ExecCtx, ExecCtxBuilder
├── llm_call.rs         -- LlmCall (primary payload)
├── payload.rs          -- Payload trait, PayloadOutput, BoxFut
├── chain.rs            -- Chain (sequential composition)
├── backend/
│   ├── mod.rs          -- Backend trait, LlmRequest, LlmResponse
│   ├── ollama.rs       -- OllamaBackend (/api/generate, /api/chat)
│   ├── openai.rs       -- OpenAiBackend (/v1/chat/completions, SSE)
│   ├── mock.rs         -- MockBackend (canned responses)
│   ├── recording.rs    -- RecordingBackend (capture/replay)
│   ├── backoff.rs      -- BackoffConfig, JitterStrategy
│   └── sse.rs          -- SSE decoder (openai feature)
├── output_parser.rs    -- Re-exports from llm-output-parser
├── diagnostics.rs      -- ParseDiagnostics
├── retry.rs            -- RetryConfig, ValidatorFn
├── retry_policy.rs     -- TransportRetryPolicy, SemanticRetryPolicy
├── prompt.rs           -- Template rendering: render(), numbered_list()
├── events.rs           -- Event enum, EventHandler trait
├── streaming.rs        -- StreamingDecoder (NDJSON)
├── parsing.rs          -- extract_thinking(), extract_json_block()
├── limits.rs           -- PipelineLimits
├── trace.rs            -- TraceId
├── pipeline.rs         -- Legacy Pipeline<T> API
├── stage.rs            -- Legacy Stage API
└── client.rs           -- Legacy LlmConfig, deprecated helpers
```

### Public API

#### Core Traits

**`Payload`** -- Object-safe trait for executable units.

| Method | Description |
|--------|-------------|
| `fn kind(&self) -> &'static str` | Stable identifier (e.g., "llm-call", "chain") |
| `fn name(&self) -> &str` | Instance name for logging |
| `fn invoke(&self, ctx: &ExecCtx, input: Value) -> BoxFut<Result<PayloadOutput>>` | Execute the payload |

**`Backend`** -- Abstraction over LLM providers.

| Method | Description |
|--------|-------------|
| `async fn complete(...)` | Non-streaming LLM call |
| `async fn complete_streaming(...)` | Streaming LLM call with token callbacks |
| `fn name(&self) -> &'static str` | Provider name |

**`EventHandler`** -- Handler for payload lifecycle events.

#### Primary Types

**`ExecCtx`** -- Shared execution context. Fields: `client`, `base_url`, `backend`, `backoff`, `vars`, `cancellation`, `event_handler`, `trace_id`, `limits`. Built via `ExecCtx::builder(base_url)`.

**`LlmCall`** -- Primary payload for LLM execution.

| Method | Description |
|--------|-------------|
| `new(name, prompt_template)` | Create new LLM call |
| `with_model(model)` | Set model name |
| `with_config(config)` | Set LLM config |
| `with_system_prompt(prompt)` | Set system prompt |
| `with_streaming(enabled)` | Enable streaming |
| `expecting_json()` | Parse output as JSON |
| `expecting_text()` | Parse as clean text |
| `expecting_choice(options)` | Match one of valid options |
| `expecting_string_list()` | Extract list of strings |
| `expecting_xml_tag(tag)` | Extract from XML tag |
| `expecting_number()` | Extract numeric value |
| `expecting_number_in_range(min, max)` | Extract bounded number |
| `with_retry(config)` | Set semantic retry config |
| `with_custom_parser(fn)` | Custom parse function |

**`PayloadOutput`** -- Output from payload invocation. Fields: `value`, `raw_response`, `thinking`, `model`, `diagnostics`, `trace_id`, `transport_retries_used`, `semantic_retries_used`, `response_bytes`, `wall_time_ms`. Methods: `from_value()`, `parse_as::<T>()`.

**`Chain`** -- Sequential composition of payloads. Each output's `value` is passed as the next payload's input.

#### Output Strategies

| Strategy | Description |
|----------|-------------|
| `Lossy` | Always succeeds; tries JSON then falls back to string |
| `Json` | Strict JSON with multi-strategy extraction |
| `StringList` | Extracts list of strings |
| `XmlTag(name)` | Extracts from named XML tag |
| `Choice(options)` | Matches one of valid options |
| `Number` | Extracts numeric value |
| `NumberInRange(min, max)` | Extracts number in bounded range |
| `Text` | Clean text with boilerplate stripping |
| `Custom(fn)` | Caller-provided parse function |

#### Retry Configuration

**`RetryConfig`** -- Semantic retry on parse failure. Fields: `max_retries` (capped at 5), `validator`, `cool_down`. Methods: `new()`, `with_validator()`, `requiring_keys()`, `no_cool_down()`.

**`BackoffConfig`** -- Transport-level retry with exponential backoff. Fields: `max_retries`, `initial_delay`, `multiplier`, `max_delay`, `jitter` (None/Full/Equal/Decorrelated), `retryable_statuses`, `respect_retry_after`. Presets: `none()`, `standard()`.

#### Backends

| Backend | Description |
|---------|-------------|
| `OllamaBackend` | Ollama native API (`/api/generate`, `/api/chat`), NDJSON streaming |
| `OpenAiBackend` | OpenAI-compatible (`/v1/chat/completions`), SSE streaming (feature: `openai`) |
| `MockBackend` | Canned responses for testing; cycles through responses |
| `RecordingBackend` | Capture/replay for testing |

#### Event System

`Event` enum: `PayloadStart`, `Token`, `PayloadEnd`, `RetryStart`, `RetryEnd`, `PartialParse`, `TransportRetry`.

#### Configuration

**`LlmConfig`** -- Fields: `temperature` (0.7), `max_tokens` (2048), `thinking`, `json_mode`, `options`.

**`PipelineLimits`** -- Fields: `max_response_bytes` (2MB), `request_timeout` (120s), `stream_idle_timeout` (30s).

### Example

```rust
use llm_pipeline::{ExecCtx, LlmCall, MockBackend, Chain, LlmConfig};
use serde_json::json;
use std::sync::Arc;

// Basic LlmCall with MockBackend
let mock = MockBackend::fixed(r#"{"title":"Inception","rating":9.2}"#);
let ctx = ExecCtx::builder("http://unused")
    .backend(Arc::new(mock))
    .build();

let call = LlmCall::new("review", "Review the movie: {input}")
    .expecting_json();

let output = call.invoke(&ctx, json!("Inception")).await?;
let review: Review = output.parse_as()?;

// Chain of LlmCalls
let chain = Chain::new("analyze")
    .push(Box::new(LlmCall::new("draft", "Analyze: {input}")))
    .push(Box::new(LlmCall::new("refine", "Refine: {input}")));

let output = chain.execute(&ctx, json!("Your text")).await?;

// With semantic retry
let call = LlmCall::new("extract", "Extract JSON: {input}")
    .expecting_json()
    .with_retry(RetryConfig::new(2).requiring_keys(&["title", "year"]));

// OpenAI backend (feature: openai)
let ctx = ExecCtx::builder("https://api.openai.com")
    .openai_with_key("sk-...")
    .build();
```

---

## Ollama-Vision-RS

**Crate:** `ollama-vision` | **Version:** 0.2.0 | **License:** MIT | **Edition:** 2021

Robust Ollama vision model toolkit for image tagging and captioning. Features a 7-strategy response parser that handles inconsistent LLM output formats reliably.

### Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| `reqwest` | 0.12 (json) | HTTP client for Ollama API |
| `serde` | 1 (derive) | Serialization |
| `serde_json` | 1 | JSON parsing |
| `tokio` | 1 (full) | Async runtime |
| `thiserror` | 2 | Error derivation |
| `base64` | 0.22 | Base64 encoding for in-memory images |
| `llm-output-parser` | path | Shared LLM output parsing library |

### Architecture

```
ollama-vision/
├── lib.rs        -- Public API root, re-exports
├── types.rs      -- Configuration & option types
├── tagger.rs     -- Image tagging logic
├── captioner.rs  -- Image captioning logic
└── parser.rs     -- Re-exports from llm-output-parser
```

### Public API

#### Core Functions

| Function | Description |
|----------|-------------|
| `tag_image(client, config, path, options)` | Tag an image from file path -> `Vec<String>` |
| `tag_image_base64(client, config, b64, options)` | Tag from base64-encoded bytes |
| `caption_image(client, config, path, options)` | Generate caption from file path -> `String` |
| `caption_image_base64(client, config, b64, options)` | Caption from base64-encoded bytes |
| `parse_tags(response)` | Parse tags from raw LLM response |
| `strip_think_tags(response)` | Remove `<think>` blocks from text |

#### Configuration Types

**`OllamaVisionConfig`** -- Client configuration. Fields: `endpoint`, `model`, `timeout` (120s), `connect_timeout` (10s), `options: GenerateOptions`. Builder: `with_model(model).endpoint(url).timeout(dur).options(opts)`.

**`GenerateOptions`** -- Ollama generation parameters. Fields: `num_predict`, `repeat_penalty` (1.2), `repeat_last_n` (128), `temperature`, `top_p`.

**`TagOptions`** -- Tag extraction config. Fields: `prompt`, `request_json_format` (true), `max_tags` (30), `max_tag_length` (50), `max_retries` (2).

**`CaptionOptions`** -- Caption generation config. Fields: `prompt`, `max_caption_length` (500), `max_retries` (2).

#### Error Types

**`TagError`** -- Variants: `Connection`, `OllamaError`, `InvalidResponse`, `ImageRead`, `Parse`.

**`CaptionError`** -- Variants: `Connection`, `OllamaError`, `InvalidResponse`, `ImageRead`, `EmptyCaption`.

Both provide `kind() -> &'static str` for stable discriminants.

### Key Features

- **7-Strategy Parser** -- Handles JSON arrays, prose, code blocks, thinking blocks, JSON objects, numbered lists, comma-separated output
- **Think Block Support** -- Works with reasoning models (deepseek-r1) that wrap output in `<think>` tags
- **Base64 APIs** -- In-memory image handling without file I/O
- **Tag Safety** -- Automatic lowercasing, trimming, deduplication, UTF-8 boundary awareness
- **Caption Truncation** -- Truncates at word boundaries, respects UTF-8

### Example

```rust
use ollama_vision::{OllamaVisionConfig, TagOptions, CaptionOptions};
use std::path::Path;

let config = OllamaVisionConfig::with_model("llava");
let client = reqwest::Client::new();

let tags = ollama_vision::tag_image(
    &client, &config, Path::new("photo.jpg"), &TagOptions::default(),
).await?;

let caption = ollama_vision::caption_image(
    &client, &config, Path::new("photo.jpg"), &CaptionOptions::default(),
).await?;
```

---

## agent-graph

**Crate:** `agent-graph` | **Version:** 0.2.0 | **License:** MIT | **Edition:** 2021

Graph-based agent orchestration for Rust -- LangGraph for the Rust ecosystem. Supports directed graph execution with conditional routing, parallel branches (fan-out/fan-in), loops, human-in-the-loop interrupts, streaming events, and checkpoint-backed recovery.

### Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| `tokio` | 1 (full) | Async runtime |
| `futures` | 0.3 | Async utilities |
| `async-trait` | 0.1 | Async trait support |
| `serde` | 1 (derive) | Serialization |
| `serde_json` | 1 | JSON support |
| `thiserror` | 1 | Error derivation |
| `anyhow` | 1 | Error handling |
| `chrono` | 0.4 (serde) | Timestamps |
| `uuid` | 1 (v4, serde) | Unique identifiers |
| `tracing` | 0.1 | Structured logging |
| `rusqlite` | 0.32 (bundled) | SQLite (optional, checkpointing feature) |

### Feature Flags

| Feature | Default | Description |
|---------|---------|-------------|
| `checkpointing` | Yes | SQLite-backed checkpoint persistence |

### Architecture

```
agent-graph/
├── lib.rs              -- Library root
├── graph.rs            -- AgentGraph + AgentGraphBuilder
├── state.rs            -- AgentState, StateTransaction, StateLimits
├── node.rs             -- Node trait, FnNode, node! macro
├── router.rs           -- RoutingFunction trait, FnRouter, router! macro
├── reducer.rs          -- Reducer trait + built-in reducers
├── event_sink.rs       -- EventSink trait + implementations
├── executor.rs         -- Executor trait, InProcessExecutor
├── checkpoint_store.rs -- Granular per-attempt checkpointing
├── checkpointer.rs     -- Legacy superstep checkpointing
├── checkpoint.rs       -- Checkpoint data structures
├── interrupt.rs        -- InterruptConfig, ExecutionResult
├── outcome.rs          -- NodeOutcome, Interrupt, InterruptKind
├── command.rs          -- Command, Navigation, SendOp
├── payload.rs          -- Payload trait, PayloadNode
├── join.rs             -- JoinNode for fan-in merging
├── config.rs           -- GraphConfig
├── retry.rs            -- RetryPolicy
├── edge.rs             -- EdgeType
├── error.rs            -- AgentGraphError
├── stream.rs           -- StreamEvent, StreamMode
└── prelude.rs          -- Convenience re-exports
```

### Public API

#### Graph Construction

**`AgentGraph`** -- Main graph orchestrator.

| Method | Description |
|--------|-------------|
| `builder()` | Create `AgentGraphBuilder` |
| `execute(start_node, state)` | Execute graph from start node |
| `execute_with_summary()` | Execute with run counts and trace_id |
| `execute_with_interrupt()` | Human-in-the-loop execution |
| `execute_cancellable()` | With cooperative cancellation |
| `stream()` | Streamed execution events |
| `resume()` / `resume_force()` | Checkpoint-backed recovery |

**`AgentGraphBuilder`** -- Builder methods: `add_node()`, `add_edge()`, `add_conditional_edge()`, `with_reducer()`, `build()`.

**Constants:** `START = "__start__"`, `END = "__end__"`.

#### State Management

**`AgentState`** -- Thread-safe async state store.

| Method | Description |
|--------|-------------|
| `new()` / `with_limits(limits)` | Create state |
| `get::<T>(key)` / `get_opt::<T>(key)` | Type-safe read |
| `set::<T>(key, value)` / `set_raw(key, value)` | Type-safe write |
| `update::<T, F>(key, f)` | Closure-based update |
| `contains(key)` / `remove(key)` / `keys()` | Key management |
| `snapshot()` / `restore(snapshot)` | State snapshots |
| `transaction()` | Begin atomic transaction |
| `fork()` | Deep copy for parallel branches |
| `register_reducer(key, reducer)` | Register reducer for key |

**`StateLimits`** -- Fields: `max_keys` (10,000), `max_value_bytes` (1 MiB), `max_history_len` (100), `lock_timeout` (5s).

#### Node Types

| Type | Description |
|------|-------------|
| `Node` (trait) | Base: `execute(state, config) -> Result<NodeOutput>` |
| `FnNode<F>` | Node from async function |
| `node!` macro | Node from async closure |
| `PayloadNode` | Node wrapping a `Payload` for external work |
| `JoinNode` | Merges results from parallel branches |

#### Navigation & Commands

**`NodeOutput`** -- Enum: `Done`, `Command(Command)`.

**`Command`** -- Methods: `goto(node)`, `end()`, `update(updates)`.

**`Navigation`** -- Enum: `Node(String)`, `Nodes(Vec<String>)`, `End`, `Send(Vec<SendOp>)`, `Default`.

#### Routing

**`RoutingFunction`** (trait) -- `route(state, config) -> Result<RouterOutput>`.

**`RouterOutput`** -- Enum: `Next(Option<String>)`, `FanOut(Vec<String>)`.

**`router!` macro** -- Create router from async closure.

#### Reducers

| Reducer | Description |
|---------|-------------|
| `LastWriteWins` | New value replaces old |
| `AppendReducer` | Appends items from update array |
| `AddReducer` | Adds numeric values |
| `MergeReducer` | Deep-merges JSON objects |
| `FnReducer<F>` | Custom closure-based reducer |

#### Events

**`EventSink`** (trait) -- Non-blocking event emission.

**`GraphEvent`** -- Variants: `RunStart`, `RunEnd`, `NodeStart`, `NodeEnd`, `Token`, `CheckpointWritten`, `InterruptRaised`, `StateUpdate`, `SuperstepStart`, `SuperstepEnd`.

Built-in sinks: `NoopEventSink`, `ChannelEventSink`, `CallbackEventSink`, `CompositeEventSink`.

#### Interrupts

**`InterruptConfig`** -- `before(node)`, `after(node)` for human-in-the-loop flows.

**`ExecutionResult`** -- Enum: `Complete(AgentState)`, `Interrupted { state, node, interrupt_value, checkpoint_data }`.

**`Interrupt`** -- Methods: `await_input(payload)`, `await_approval(payload)`, `custom(kind, payload)`.

#### Checkpointing

**`CheckpointStore`** (trait) -- Granular per-attempt recording: `create_run()`, `record_attempt()`, `complete_attempt()`, `fail_attempt()`, `record_interrupt()`, `save_state_snapshot()`, `load_run()`.

**`InMemoryCheckpointStore`** -- In-memory implementation.

**`SqliteSaver`** / **`CheckpointManager`** -- SQLite persistence (feature: checkpointing).

#### Configuration

**`GraphConfig`** -- Fields: `thread_id`, `trace_id`, `recursion_limit` (100), `max_parallelism` (8, max: 32), `tags`, `metadata`, `configurable`.

**`RetryPolicy`** -- Fields: `max_attempts` (3), `initial_interval` (1s), `backoff_factor` (2.0), `max_interval` (60s), `jitter` (true), `retry_on`.

#### Errors

`AgentGraphError` variants: `NodeNotFound`, `RoutingError`, `StateError`, `MaxIterationsExceeded`, `CycleDetected`, `CheckpointError`, `CheckpointMismatch`, `ExecutionError`, `InterruptError`, `PayloadError`, `Cancelled`, `SerializationError`, `DatabaseError`, `Other`.

### Example

```rust
use agent_graph::prelude::*;

let graph = AgentGraph::builder()
    .add_node("classify", node!(|state| async move {
        let input: String = state.get("input").await?;
        let category = if input.contains("urgent") { "high" } else { "low" };
        state.set("priority", category).await?;
        Ok(())
    }))
    .add_node("process", node!(|state| async move {
        let priority: String = state.get("priority").await?;
        state.set("result", format!("Processed with {} priority", priority)).await?;
        Ok(())
    }))
    .add_edge(START, "classify")
    .add_conditional_edge("classify", router!(|state| async move {
        let p: String = state.get("priority").await?;
        Ok(Some("process".to_string()))
    }))
    .add_edge("process", END)
    .build()?;

let mut state = AgentState::new();
state.set("input", "urgent task").await?;
let result = graph.execute("classify", state).await?;
```

---

## job-queue

**Crate:** `job-queue` | **Version:** 0.2.0 | **License:** MIT | **Edition:** 2021

Production-grade background job queue system with SQLite persistence, priority scheduling, crash recovery, exponential backoff retry, and real-time progress tracking.

### Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| `tokio` | 1 (full) | Async runtime |
| `rusqlite` | 0.32 (bundled) | SQLite persistence |
| `serde` | 1 (derive) | Serialization |
| `serde_json` | 1 | JSON support |
| `anyhow` | 1 | Error handling |
| `chrono` | 0.4 (serde) | Timestamps |
| `uuid` | 1 (v4, serde) | Job ID generation |
| `thiserror` | 2 | Error derivation |
| `tracing` | 0.1 | Structured logging |

### Architecture

```
job-queue/
├── lib.rs       -- Crate root, public API exports
├── types.rs     -- QueueJob, JobResult, QueueStats, etc.
├── config.rs    -- QueueConfig, QueueConfigBuilder
├── queue.rs     -- QueueManager (high-level API)
├── executor.rs  -- QueueExecutor (background processor)
├── db.rs        -- SQLite schema, migrations, queries
├── events.rs    -- Event types and QueueEventEmitter trait
└── error.rs     -- QueueError enum
```

### Public API

#### Core Trait

**`JobHandler`** -- Trait that job types must implement. Requires `Send + Sync + Serialize + DeserializeOwned + Clone`.

| Method | Description |
|--------|-------------|
| `async fn execute(&self, ctx: &JobContext) -> Result<JobResult, QueueError>` | Main execution method |
| `fn job_type(&self) -> &str` | Human-readable job type name (defaults to type_name) |

#### Enums

| Enum | Variants | Description |
|------|----------|-------------|
| `QueuePriority` | `Low` (3), `Normal` (2), `High` (1) | Job priority (lower number = higher priority) |
| `QueueJobStatus` | `Pending`, `Processing`, `Completed`, `Failed`, `Cancelled` | Job lifecycle status |
| `FailureClass` | `Transient`, `Permanent`, `RateLimited { retry_after_secs }` | Failure classification for retry logic |

#### Structs

**`QueueManager`** -- High-level queue API.

| Method | Description |
|--------|-------------|
| `new(config)` | Create manager, auto-recover crashed jobs |
| `add(job)` | Enqueue job, returns job ID |
| `cancel(job_id)` | Cancel pending/processing job |
| `reorder(job_id, new_priority)` | Change priority of pending job |
| `pause()` / `resume()` | Pause/resume executor |
| `list_jobs()` / `list_jobs_with_data()` | Query all jobs |
| `get_job_details(job_id)` | Detailed job info |
| `prune(days)` | Delete old jobs |
| `count_by_status()` | Stats by status |
| `shutdown()` | Graceful shutdown |
| `spawn::<H>(emitter)` | Start background executor |
| `process_one::<H>(emitter)` | Process one job synchronously |

**`QueueJob<T>`** -- Serializable job payload. Fields: `id`, `trace_id`, `priority`, `status`, `data: T`, timestamps, `error_message`. Builder: `new(data).with_priority(p).with_id(id).with_trace_id(tid)`.

**`JobContext`** -- Execution context provided to handlers. Fields: `job_id`, `trace_id`, `worker_id`, `attempt_count`. Methods: `emit_progress(current, total)`, `is_cancelled()`.

**`JobResult`** -- Execution result. Methods: `success()`, `success_with_output(s)`, `failure(s)`, `transient_failure(s)`, `rate_limited(s, retry_after)`.

**`QueueConfig`** -- Configuration via builder. Fields: `db_path`, `worker_id`, `cooldown`, `max_consecutive`, `poll_interval` (3s), `heartbeat_interval` (10s), `stale_after` (300s), `max_retries`.

**`QueueStats`** -- Aggregate counts: `pending`, `processing`, `completed`, `failed`, `cancelled`.

#### Events

**`QueueEventEmitter`** (trait) -- Receives lifecycle events.

| Method | Description |
|--------|-------------|
| `emit_job_started(event)` | Job started processing |
| `emit_job_completed(event)` | Job completed successfully |
| `emit_job_failed(event)` | Job failed (includes failure_class, next_retry_at) |
| `emit_job_progress(event)` | Progress update (current_step, total_steps) |
| `emit_job_cancelled(event)` | Job was cancelled |

Built-in: `NoopEventEmitter`, `LoggingEventEmitter`.

#### Errors

`QueueError` variants: `Database`, `Serialization`, `Execution`, `NotFound`, `InvalidTransition`, `Paused`, `Cancelled`, `Other`.

### Key Features

- **Priority scheduling** -- High > Normal > Low, FIFO within same priority
- **SQLite persistence** -- WAL mode, crash recovery on startup
- **Heartbeat leasing** -- Prevents stale job accumulation
- **Failure classification** -- Transient (retried with backoff), Permanent (not retried), RateLimited (delayed retry)
- **Exponential backoff** -- `2^(attempt-1) * 5s`, capped at 5 minutes
- **Cooperative cancellation** -- Jobs check `is_cancelled()` during execution
- **Progress tracking** -- Real-time progress via event emitter
- **Multi-worker support** -- Worker leases and ownership validation

### Example

```rust
use job_queue::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct EmailJob { to: String, subject: String }

impl JobHandler for EmailJob {
    async fn execute(&self, ctx: &JobContext) -> Result<JobResult, QueueError> {
        for i in 0..10 {
            if ctx.is_cancelled() { return Err(QueueError::Cancelled); }
            ctx.emit_progress(i + 1, 10);
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        Ok(JobResult::success_with_output("Sent".into()))
    }
}

let config = QueueConfig::builder()
    .with_db_path(PathBuf::from("queue.db"))
    .with_max_retries(3)
    .build();

let manager = QueueManager::new(config)?;
let job = QueueJob::new(EmailJob { to: "a@b.com".into(), subject: "Hi".into() })
    .with_priority(QueuePriority::High);
manager.add(job)?;

let manager = manager.spawn::<EmailJob>(Arc::new(LoggingEventEmitter));
```

---

## semantic-memory

**Crate:** `semantic-memory` | **Version:** 0.4.0 | **License:** MIT | **Edition:** 2021 | **MSRV:** 1.75+

Hybrid semantic search with SQLite, FTS5, and HNSW -- built for AI agents. Combines BM25 full-text search with vector similarity via Reciprocal Rank Fusion, all stored locally in SQLite with no external vector database required.

### Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| `bytemuck` | 1 (derive) | Memory layout manipulation |
| `rusqlite` | 0.32 (bundled, blob) | SQLite bindings |
| `reqwest` | 0.12 (json, rustls-tls) | HTTP client for embeddings |
| `serde` | 1 (derive) | Serialization |
| `serde_json` | 1 | JSON support |
| `tokio` | 1 (rt, macros) | Async runtime |
| `thiserror` | 2 | Error derivation |
| `tracing` | 0.1 | Structured logging |
| `uuid` | 1 (v4) | ID generation |
| `chrono` | 0.4 (serde) | Timestamps |
| `hnsw_rs` | 0.3 | HNSW ANN index (optional) |

### Feature Flags

| Feature | Default | Description |
|---------|---------|-------------|
| `hnsw` | Yes | HNSW approximate nearest-neighbor search |
| `brute-force` | No | Brute-force vector search (no external deps) |
| `testing` | No | Exposes `raw_execute()` for test harnesses |

At least one of `hnsw` or `brute-force` must be enabled.

### Architecture

```
semantic-memory/
├── lib.rs           -- MemoryStore implementation (main API)
├── types.rs         -- All public types
├── error.rs         -- MemoryError enum
├── config.rs        -- MemoryConfig, EmbeddingConfig, SearchConfig, etc.
├── db.rs            -- Database init, migrations, queries
├── search.rs        -- Hybrid search engine, RRF fusion
├── knowledge.rs     -- Fact CRUD with FTS5 sync
├── conversation.rs  -- Session/message management
├── documents.rs     -- Document chunking/ingestion
├── chunker.rs       -- Text splitting with overlap
├── hnsw.rs          -- HNSW wrapper
├── storage.rs       -- Storage path management
├── quantize.rs      -- SQ8 quantization (f32 -> int8)
└── tokenizer.rs     -- TokenCounter trait

Storage layout:
  base_dir/
  ├── memory.db           (SQLite database)
  ├── memory.hnsw.graph   (HNSW topology)
  └── memory.hnsw.data    (HNSW vector data)
```

### Public API

#### MemoryStore

**Initialization:**

| Method | Description |
|--------|-------------|
| `open(config)` | Open/create memory store |
| `open_with_embedder(config, embedder)` | Open with custom embedder |

**Facts (Namespaced Knowledge):**

| Method | Description |
|--------|-------------|
| `add_fact(namespace, content, source, metadata)` | Add fact with auto-embedding |
| `add_fact_with_embedding(ns, content, embedding, ...)` | Add with pre-computed embedding |
| `update_fact(fact_id, content)` | Update fact content |
| `delete_fact(fact_id)` / `delete_namespace(ns)` | Delete fact(s) |
| `get_fact(fact_id)` / `list_facts(ns, limit, offset)` | Query facts |

**Documents (Chunked Content):**

| Method | Description |
|--------|-------------|
| `ingest_document(title, content, ns, path, metadata)` | Chunk and embed document |
| `delete_document(document_id)` | Delete document and chunks |
| `list_documents(ns, limit, offset)` | Query documents |
| `chunk_text(text)` | Split text into chunks |

**Conversations:**

| Method | Description |
|--------|-------------|
| `create_session(channel)` | Create conversation session |
| `add_message(session_id, role, content, token_count, metadata)` | Add message |
| `get_recent_messages(session_id, limit)` | Get recent messages |
| `get_messages_within_budget(session_id, max_tokens)` | Token-budget retrieval |
| `session_token_count(session_id)` | Total tokens in session |

**Search:**

| Method | Description |
|--------|-------------|
| `search(query, top_k, namespaces, source_types)` | Hybrid BM25 + vector search |
| `search_fts_only(...)` | BM25 only |
| `search_vector_only(...)` | Vector only |
| `search_conversations(query, top_k, session_ids)` | Search chat history |
| `search_explained(...)` | Search with full score breakdown |

**Episodes (Causal Records):**

| Method | Description |
|--------|-------------|
| `ingest_episode(document_id, meta)` | Record causal episode |
| `update_episode_outcome(doc_id, outcome, confidence, exp_id)` | Update outcome |
| `search_episodes(effect_type, outcome, limit)` | Query episodes |

**Embedding & Analysis:**

| Method | Description |
|--------|-------------|
| `embed(text)` / `embed_batch(texts)` | Generate embeddings |
| `embedding_displacement(text_a, text_b)` | Cosine similarity + euclidean distance |

**Integrity & Maintenance:**

| Method | Description |
|--------|-------------|
| `verify_integrity(mode)` | Health check (Quick or Full) |
| `reconcile(action)` | Repair: ReportOnly, RebuildFts, or ReEmbed |
| `reembed_all()` | Re-embed all content |
| `rebuild_hnsw_index()` / `compact_hnsw()` | Index maintenance |
| `stats()` / `vacuum()` | Database stats and optimization |

#### Key Types

| Type | Description |
|------|-------------|
| `SearchResult` | Hit with score, content, source info |
| `ExplainedResult` | SearchResult + `ScoreBreakdown` (RRF, BM25, vector, recency) |
| `SearchSource` | Enum: Fact, Chunk, Message, Episode |
| `Fact` | Namespaced fact with source and metadata |
| `Document` / `TextChunk` | Document and its chunks |
| `Session` / `Message` / `Role` | Conversation types |
| `EpisodeMeta` | Causal metadata (cause_ids, effect_type, outcome, confidence) |
| `EpisodeOutcome` | Confirmed, Refuted, Inconclusive, Pending |
| `QuantizedVector` | SQ8 quantized vector (4x memory reduction) |
| `HnswConfig` | Index params: m (16), ef_construction (200), ef_search (50), dimensions (768) |
| `IntegrityReport` / `VerifyMode` / `ReconcileAction` | Health check types |
| `Embedder` (trait) | Text-to-vector conversion |
| `OllamaEmbedder` / `MockEmbedder` | Production and test embedders |

#### Configuration Defaults

```
base_dir: "memory"
embedding: ollama @ localhost:11434, model "nomic-embed-text", 768 dims
search: bm25_weight 1.0, vector_weight 1.0, rrf_k 60, top_k 5, min_similarity 0.3
chunking: target 1000 chars, min 100, max 2000, overlap 200
pool: WAL mode, busy_timeout 5s
limits: 100k facts/ns, 1k chunks/doc, 1MB content, 8 concurrent embeddings
hnsw: m=16, ef_construction=200, ef_search=50, max_elements=100k
```

### Search Pipeline

1. **Sanitize** -- Remove FTS5 operators from raw query
2. **BM25** -- Full-text search via FTS5 (Porter stemming, Unicode tokenization)
3. **Vector** -- Embedding similarity via HNSW or brute-force
4. **RRF Fusion** -- `bm25_score / (k + bm25_rank) + vector_score / (k + vector_rank)`
5. **Recency Boost** -- Optional exponential decay: `2^(-age_days / half_life)`
6. **Filter** -- By namespace, source type
7. **Rerank** -- Optional exact f32 cosine similarity from SQLite

### Example

```rust
use semantic_memory::{MemoryStore, MemoryConfig, Role};

let store = MemoryStore::open(MemoryConfig::default())?;

// Facts
store.add_fact("general", "Rust was first released in 2015", None, None).await?;

// Documents
store.ingest_document("guide.md", "# Rust Guide\n...", "docs", None, None).await?;

// Conversations
let session = store.create_session("chat").await?;
store.add_message(&session, Role::User, "What is Rust?", Some(10), None).await?;

// Hybrid search
let results = store.search("systems programming", Some(5), None, None).await?;

// Explained search
let explained = store.search_explained("systems programming", Some(5), None, None).await?;
for r in &explained {
    println!("{:.4} (bm25={}, vec={}, recency={:.2})",
        r.result.score, r.breakdown.bm25_rank, r.breakdown.vector_rank,
        r.breakdown.recency_boost);
}
```

---

## Tauri-Queue

**Crate:** `tauri-queue` | **Version:** 0.3.0 | **License:** MIT | **Edition:** 2021

Tauri integration for job-queue background job processing. Bridges job-queue events to the Tauri frontend event system with optional event coalescing to prevent UI event storms from high-frequency progress updates.

### Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| `job-queue` | path | Background job queue |
| `tauri` | 2 | Desktop framework |
| `tokio` | 1 (full) | Async runtime |
| `serde` | 1 (derive) | Serialization |
| `serde_json` | 1 | JSON support |

### Feature Flags

| Feature | Default | Description |
|---------|---------|-------------|
| `sqlite` | Yes | SQLite persistence for job-queue |

### Public API

**`TauriEventEmitter`** -- Bridges job-queue events to Tauri frontend.

| Method | Description |
|--------|-------------|
| `new(app_handle)` | Create emitter from Tauri AppHandle |
| `arc(app_handle)` | Create as `Arc<dyn QueueEventEmitter>` |

**`CoalescingEmitter`** -- Throttles rapid-fire events.

| Method | Description |
|--------|-------------|
| `new(inner, config)` | Wrap an emitter with coalescing |
| `arc(inner, config)` | Create as `Arc<dyn QueueEventEmitter>` |

**`EmitterConfig`** -- Coalescing configuration. Fields: `buffer_size` (256), `drop_policy` (DropNewest), `coalesce_interval_ms` (50), `include_trace_id` (true).

**`DropPolicy`** -- Enum: `DropOldest`, `DropNewest`, `Block`.

**Re-exports:** All public types from `job-queue` (QueueManager, QueueConfig, QueueJob, JobHandler, JobContext, JobResult, etc.).

### Frontend Events

| Event | Payload |
|-------|---------|
| `queue:job_started` | job_id, trace_id, worker_id, attempt_count, status |
| `queue:job_progress` | job_id, current_step, total_steps, progress percentage |
| `queue:job_completed` | job_id, output |
| `queue:job_failed` | job_id, error message |
| `queue:job_cancelled` | job_id |

### Example

```rust
use tauri_queue::*;

// In Tauri setup
let emitter = CoalescingEmitter::arc(
    TauriEventEmitter::arc(app.handle().clone()),
    EmitterConfig { coalesce_interval_ms: 50, ..Default::default() },
);

let manager = QueueManager::new(QueueConfig::default())?;
let manager = manager.spawn::<MyJob>(emitter);
app.manage(manager);
```

---

## Tauri-React-Hooks

**Package:** `@tauri-hooks/core` | **Version:** 0.1.0 | **License:** MIT | **Type:** ESM

React hooks for Tauri 2 apps -- async-safe event listeners, command invocation with state, config management, and stream buffering.

### Dependencies

| Package | Version | Scope |
|---------|---------|-------|
| `react` | >=18 | Peer |
| `@tauri-apps/api` | >=2 | Peer |

### Hooks

#### `useTauriEvent<T>(event, handler, deps?)`

Subscribe to a single Tauri event with async-safe cleanup. Uses `useRef` to keep handler fresh without re-subscribing.

```tsx
useTauriEvent<{ jobId: string }>("queue:job_completed", (payload) => {
    console.log("Done:", payload.jobId);
});
```

#### `useTauriEvents(bindings, deps?)`

Subscribe to multiple Tauri events atomically. All subscriptions established in parallel.

```tsx
useTauriEvents({
    "queue:job_started": (p) => setStatus("running"),
    "queue:job_completed": (p) => setStatus("done"),
});
```

#### `useTauriQuery<T>(command, args?, options?, deps?)`

Run a Tauri command and manage data/loading/error state. Auto-executes on mount, re-fetches on arg changes.

```tsx
const { data, loading, error, refresh } = useTauriQuery<string[]>(
    "list_images",
    { folder: "/tmp/gallery" },
    { refreshOn: ["queue:job_completed"] },
);
```

**Returns:** `{ data: T | null, loading: boolean, error: string | null, refresh: () => Promise<void> }`

**Options:** `enabled` (default: true), `refreshOn` (event names triggering auto-refresh).

#### `useTauriMutation<TArgs, TResult>(command, argsFn?, options?)`

Wrap a Tauri command as an explicit mutation (manual trigger). Does NOT auto-execute.

```tsx
const { mutate, loading, error, reset } = useTauriMutation<[string], void>(
    "delete_image",
    (path) => ({ path }),
    { onSuccess: () => refresh() },
);

await mutate("/tmp/photo.jpg");
```

#### `useTauriConfig<T>(loadCmd, saveCmd, saveArgName?)`

Load and save config object via Tauri commands. Supports optimistic local updates.

```tsx
const { config, update, save, reload, saving } = useTauriConfig<AppConfig>(
    "get_config", "save_config",
);

update({ theme: "dark" });  // Optimistic local merge
await save(config);          // Persist to backend
```

#### `useBufferedStream<K>(options?)`

High-frequency data batching with two-layer buffering. Layer 1: sync writes (no re-renders). Layer 2: flushed at interval (default: 33ms ~30fps).

```tsx
const stream = useBufferedStream({ interval: 33 });
useTauriEvent<{ id: string; token: string }>("llm:token", (p) => {
    stream.push(p.id, p.token);
});

// stream.buffers contains accumulated text per key
return <pre>{stream.buffers["main"]}</pre>;
```

### Key Design Decisions

- **Async-safe cleanup** -- Handles race conditions between listen promise resolution and unmount
- **Fresh handler pattern** -- `useRef` avoids dependency array overhead
- **No heavy dependencies** -- Only React and Tauri API
- **Query auto-refresh** -- Listens to Tauri events for automatic data refetching

---

## Primitives

**Location:** `Primitives/` | **Type:** Cargo workspace (10 crates) | **Version:** 0.1.0 each | **License:** MIT | **MSRV:** 1.75+

Foundation stack for patch generation, validation, execution, and causal edit attribution. All crates are modular with minimal coupling.

### Crate Dependency Graph

```
typed-patch (structured patch model)
    |
    +-- forge-policy (path/env/DB guardrails)
    +-- sandbox-workspace (safe workspace staging)
    |       +-- forge-policy
    |
effect-signature (stable effect identifiers)
    |
check-runner (host/container execution)
    +-- effect-signature
    +-- forge-policy
    +-- sandbox-workspace
    |
cea-core (causal edit attribution)
    +-- check-runner
    +-- typed-patch
    |
cea-store (persistence interface)
    +-- cea-core
    |
cea-sqlite (SQLite implementation)
    +-- cea-store
    +-- forge-policy
    |
mindstate-core (deterministic state hashing)
    |
stabilizer-core (attempt phase progression)
    +-- typed-patch
```

### 1. forge-policy

Path, environment, and DB guardrails.

| Function | Description |
|----------|-------------|
| `verify_sqlite_db_identity(path, spec)` | Validates SQLite DB file, version, schema |
| `ensure_relative_path(path)` | Ensures path is relative |
| `reject_symlinks(path)` | Rejects symlinks |
| `resolve_workspace_path(root, relative)` | Resolves with bounds checking |
| `validate_forbidden_paths(patches)` | Checks against forbidden patterns |
| `validate_patch_caps(patch)` | Enforces file/line count limits |
| `is_env_allowed(key)` | Allowlist for environment variables |

Types: `PolicyError`, `Violation`, `ViolationKind`, `DbIdentitySpec`.

### 2. sandbox-workspace

Safe workspace staging and patch filesystem helpers.

| Type/Function | Description |
|---------------|-------------|
| `Workspace` | Host path + optional temp dir |
| `PatchFs` (trait) | Path-safe filesystem abstraction: `root()`, `exists()`, `read_lines()`, `write_lines()`, etc. |
| `LocalPatchFs` | Concrete local filesystem implementation |
| `prepare_workspace(src, use_temp?)` | Copy workspace to temp dir |

### 3. effect-signature

Stable identifiers for observed validation effects.

| Type | Description |
|------|-------------|
| `EffectSignature` | Effect from a check: `check_kind`, `outcome`, `severity`, `message_class`, `line_offset_from_edit` |
| `LocatedEffect` | Effect with source location: `file`, `line`, `col`, `message`, `sig` |
| `effect_signature_hash(sig)` | Blake3 hash for stable identity |

### 4. check-runner

Host/container execution for patch verification.

| Type | Description |
|------|-------------|
| `ExecutionBackend` (trait) | `run_command(cmd, args, workspace, timeout)` |
| `HostBackend` | Local execution with env sanitization and timeouts |
| `ContainerBackend` | Docker/Podman/Nerdctl with sealed mode (`--network=none`) |
| `CheckResult` | Result of fmt + clippy + test checks |
| `ParsedCheckOutput` | Parsed output with `effects: Vec<LocatedEffect>` |

Features: `container` (optional).

### 5. mindstate-core

Deterministic rendering and hashing for agent mindstate.

| Type | Description |
|------|-------------|
| `MindState` | Serializable payload: request, repo context, evidence, traces, config |
| `EvidenceItem` | Evidence entry with stable ordering |
| `TraceRef` | Reference to previous answer trace |
| `OrderedFloat` | f64 wrapper with total ordering |

Methods: `render()` (deterministic JSON), `hash()` (Blake3). Functions: `compute_question_sig()`, `budget_evidence()`.

### 6. stabilizer-core

Attempt-phase and novelty helpers for iterative patch generation.

| Type | Description |
|------|-------------|
| `Stabilizer` | Manages progression through attempt phases |
| `DeltaPolicy` | Novelty amplitude per phase |
| `AttemptPhase` | `Innovative` -> `Stabilize1` -> `Stabilize2` -> `Clamp` |
| `AttemptOverrides` | Per-attempt configuration |

Functions: `extract_strategy_tags(patch)`, `compute_tag_novelty(tags, prev)`, `determine_approach_family(patch)`.

### 7. cea-core

Causal edit attribution for structured code patches.

| Type | Description |
|------|-------------|
| `AttributionTriple` | (cause: EditOpSignature, effect: EffectSignature, score) |
| `AttributedRunResult` | Instrumented run result with triples and coverage |
| `EditOpSignature` | Signature of edit op (file, op index, anchor kind, context hash) |
| `CausalGraph` | In-memory petgraph DiGraph |
| `CausalPrediction` | Risk flags, correctness prediction, coverage |

Functions: `attribute_effects(patch, result)`, `predict(graph, patch)`, `compute_run_hash(patch, results)`.

Key properties: deterministic hashing, proportional multi-cause attribution, beta-distribution confidence, raw source never stored (only blake3 hashes).

### 8. cea-store

Persistence interface for causal edit attribution graphs.

| Type | Description |
|------|-------------|
| `CeaStore` (trait) | `has_run()`, `upsert_node()`, `upsert_edge()`, `insert_run_log()`, `load_nodes()`, `load_edges()` |
| `UpdateResult` | `Applied { edges_added, edges_updated }` or `AlreadyProcessed` |
| `update_graph(store, result, eval_id, version_id, decay)` | Idempotent graph update |

### 9. cea-sqlite

SQLite implementation of `CeaStore`.

| Type | Description |
|------|-------------|
| `SqliteCeaStore` | Concrete implementation with `open(path)` |

Tables: `cea_nodes`, `cea_edges`, `cea_run_log`. Uses upsert semantics and beta-distribution confidence updates.

### 10. typed-patch

Structured patch model plus validation and application.

| Type | Description |
|------|-------------|
| `StructuredPatch` | Top-level: `patch_id`, `summary`, `edits: Vec<FileEdit>`, `notes` |
| `FileEdit` | Changes to a single file: `path`, `ops: Vec<EditOp>`, `mode` |
| `EditOp` | `Insert { anchor, lines }`, `Delete { range }`, `Replace { range, lines }` |
| `Anchor` | `AfterLine`, `BeforeLine`, `AfterMatch`, `BeforeMatch` |
| `LineRange` | `start`, `end_exclusive` |
| `LineAttributionMap` | Line mapping for CEA position attribution |
| `PatchPolicy` | Policy constraints (forbidden paths, max files, max lines) |

Functions: `validate_patch(patch, policy)`, `apply_patch(patch, workspace_fs)` (atomic), `render_diff(original, patched)`.

---

## living-memory

**Crate:** `semantic-memory-forge` | **Version:** 0.2.0 | **License:** MIT | **Edition:** 2021 | **MSRV:** 1.75+

Causal edit attribution and structured patch evaluation engine. Integrates all Primitives crates with semantic-memory (read-only) to provide a complete patch generation, validation, execution, and evolutionary search pipeline.

### Dependencies

All Primitives crates (path dependencies) plus:

| Crate | Version | Purpose |
|-------|---------|---------|
| `semantic-memory` | path | Read-only hybrid search |
| `tokio` | 1 (full) | Async runtime |
| `serde` / `serde_json` | 1 | Serialization |
| `thiserror` | 2 | Error handling |
| `anyhow` | 1 | Error context |
| `uuid` | 1 | Unique IDs |
| `blake3` | 1 | Content hashing |
| `rusqlite` | 0.32 (bundled) | SQLite |
| `petgraph` | 0.6 | Graph algorithms |
| `regex` | 1 | Pattern matching |
| `chrono` | 0.4 | Timestamps |
| `similar` | 2 | Diff algorithm |

### Feature Flags

| Feature | Default | Description |
|---------|---------|-------------|
| `danger-sm-write` | No | Allow writes to semantic-memory (normally read-only) |

### Module Structure

```
living-memory/
├── adapters    -- ProjectAdapter trait + CargoAdapter
├── cea         -- Causal edit attribution (re-exports from cea-core/store)
├── config      -- ForgeConfig + ForgeLimits
├── error       -- Forge-specific errors
├── exec        -- Execution backends (host/container)
├── invariants  -- Database safety checks
├── lab         -- Evaluation suite, MAP-Elites archive, promotion
├── runtime     -- MindState compilation, patch validation/apply, stabilization
└── store       -- ForgeStore trait + SQLite implementation
```

### Core Capabilities

#### Runtime (MindState + Patches)

| Function | Description |
|----------|-------------|
| `compile_mindstate(request, repo_ctx, evidence, traces, basis)` | Compile deterministic MindState |
| `validate_patch(patch, policy)` | Validate patch against policy |
| `apply_patch(patch, workspace_fs)` | Atomic patch application |
| `render_diff(original, patched)` | Generate unified diff |

#### Execution

| Type | Description |
|------|-------------|
| `ExecutionBackend` (trait) | Abstract execution interface |
| `HostBackend` / `ContainerBackend` | Concrete backends |
| `CheckResult` | Combined fmt + clippy + test results |
| `select_backend(config)` | Auto-select based on availability |

#### CEA (Causal Edit Attribution)

| Function | Description |
|----------|-------------|
| `attribute_effects(patch, result)` | Map edits to check effects |
| `load_graph(store)` | Load causal graph from storage |
| `predict(graph, patch)` | Zero-shot correctness prediction |
| `update_graph(store, result, eval_id, version_id, decay)` | Idempotent graph update |

#### Lab (Evolutionary Search)

| Type/Function | Description |
|---------------|-------------|
| `AlgebraSpec` | Candidate representation |
| `EvalSuite` / `EvalTask` | Test fixtures |
| `ScoreVector` | Multi-objective scores |
| `BasisVersion` | Promoted immutable version |
| `archive_insert(...)` | MAP-Elites cell insertion |
| `promote(...)` | Graduation to BasisVersion |
| `ExperimentRunner` | Lab orchestration |
| `VerificationPlan` | Verification contracts |

### Pipeline Flow

```
User Request + Repo Context + Evidence
    |
    v
MindState Compiler (+ semantic-memory search)
    |
    v
StructuredPatch (from external LLM/agent)
    |
    v
Validation (forge-policy) + Application (atomic)
    |
    v
Execution (host/container: fmt, clippy, test)
    |
    v
CEA Attribution (edit -> effect mapping)
    |
    v
Scoring (correctness + novelty)
    |
    v
Lab Archive (MAP-Elites) -> Promotion (BasisVersion)
```

### Key Design Principles

- **Read-only semantic-memory** -- Never modifies the external memory store
- **Deterministic** -- MindState rendering, patch hashing, run hashing all reproducible
- **Local-only** -- Container sealed mode (`--network=none`), no remote calls by default
- **Atomic patches** -- All-or-nothing application
- **Idempotent CEA** -- Same run data always produces same graph updates
- **Privacy** -- Context lines stored only as blake3 hashes, never raw source

---

## Cross-Crate Relationships

```
                    Tauri-React-Hooks (TypeScript/React)
                            |
                            | (Tauri events)
                            v
                       Tauri-Queue
                            |
                            | (re-exports)
                            v
                        job-queue
                            |
                            |
    ComfyUI-RS    Ollama-Vision-RS    LLM-Pipeline
         \              |              /
          \             |             /
           v            v            v
              AI-Batch-Queue (orchestrates batch processing)

    agent-graph (graph-based orchestration)
         |
         | (Payload trait)
         v
    LLM-Pipeline (execution inside graph nodes)

    semantic-memory (hybrid search, local vector DB)
         |
         | (read-only)
         v
    living-memory / semantic-memory-forge
         |
         | (uses all)
         v
    Primitives (10 crates: typed-patch, cea-core, check-runner, etc.)
```

| Relationship | Description |
|--------------|-------------|
| ComfyUI-RS -> AI-Batch-Queue | Image generation can be batched |
| Ollama-Vision-RS -> AI-Batch-Queue | Vision tagging/captioning can be batched |
| LLM-Pipeline -> agent-graph | Payloads execute inside graph nodes |
| job-queue -> Tauri-Queue | Thin Tauri bridge layer |
| Tauri-Queue -> Tauri-React-Hooks | Frontend hooks consume queue events |
| semantic-memory -> living-memory | Evidence retrieval for mindstate compilation |
| Primitives -> living-memory | Foundation for patch lifecycle |
| llm-output-parser | Shared parsing library used by LLM-Pipeline and Ollama-Vision-RS |
