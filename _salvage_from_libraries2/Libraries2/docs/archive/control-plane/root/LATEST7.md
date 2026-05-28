# LATEST7.md — Master Library Reference

> Generated 2026-03-08. Granular inventory of every library in the stack.

---

## Table of Contents

| # | Library | Version | Language | Description |
|---|---------|---------|----------|-------------|
| 1 | [stack-ids](#1-stack-ids) | 0.1.0 | Rust | Canonical cross-crate ID, scope, trace, and digest primitives |
| 2 | [agent-graph](#2-agent-graph) | 0.2.0 | Rust | Graph-based agent orchestration (LangGraph for Rust) |
| 3 | [llm-pipeline](#3-llm-pipeline) | 0.2.0 | Rust | LLM call payloads: prompt templating, backends, parsing, retry |
| 4 | [semantic-memory](#4-semantic-memory) | 0.5.0 | Rust | Hybrid semantic search with SQLite, FTS5, and HNSW |
| 5 | [semantic-memory-forge (schemas)](#5-semantic-memory-forge-schemas) | 0.1.0 | Rust | Evidence bundle, export envelope, and estimator schemas |
| 6 | [forge-memory-bridge](#6-forge-memory-bridge) | 0.1.0 | Rust | Transform export envelopes into projection import batches |
| 7 | [knowledge-runtime](#7-knowledge-runtime) | 0.1.0 | Rust | Query classification, routing, entity resolution, projection tracking |
| 8 | [job-queue](#8-job-queue) | 0.2.0 | Rust | Production-grade SQLite background job queue |
| 9 | [AI-Batch-Queue](#9-ai-batch-queue) | 0.2.0 | Rust | Model-aware batch processing with ETA estimation (Tauri) |
| 10 | [Tauri-Queue](#10-tauri-queue) | 0.3.0 | Rust | Tauri integration for job-queue with event coalescing |
| 11 | [Tauri-React-Hooks](#11-tauri-react-hooks) | 0.1.0 | TypeScript | React hooks for Tauri 2: events, queries, mutations, config, buffering |
| 12 | [ComfyUI-RS](#12-comfyui-rs) | 0.2.0 | Rust | Async client for ComfyUI (REST + WebSocket progress) |
| 13 | [Ollama-Vision-RS](#13-ollama-vision-rs) | 0.2.0 | Rust | Ollama vision model toolkit for image tagging and captioning |
| 14 | [Primitives](#14-primitives) | 0.1.0 | Rust | 10-crate toolkit: typed patches, check runners, CEA, policy |
| 15 | [living-memory](#15-living-memory) | 0.2.0 | Rust | Full forge engine: causal attribution, experiments, evidence, export |

---

## Dependency Graph (Simplified)

```
stack-ids (canonical IDs, trace, digest)
    |
    +---> agent-graph (orchestration)
    +---> llm-pipeline (LLM calls + parsing)
    +---> semantic-memory (storage + search)
    +---> semantic-memory-forge [schemas] (evidence types)
    +---> forge-memory-bridge (envelope transform)
    +---> knowledge-runtime (query pipeline)
    +---> job-queue (background jobs)
    +---> AI-Batch-Queue (batch processing)
    +---> Tauri-Queue --> job-queue

Primitives (10 crates: typed-patch, check-runner, cea-core, ...)
    |
    +---> living-memory (forge engine) --> semantic-memory

ComfyUI-RS (standalone)
Ollama-Vision-RS --> llm-output-parser
Tauri-React-Hooks (standalone TypeScript)
```

---

## 1. stack-ids

**Crate:** `stack-ids` | **Version:** 0.1.0 | **MSRV:** 1.75 | **License:** MIT
**Description:** Shared identity, scope, and trace primitives for the local-first AI systems stack

### Dependencies
`serde` 1, `serde_json` 1, `blake3` 1, `uuid` 1 (v4), `chrono` 0.4 (serde)

### Modules
- `ids` — 12 opaque ID newtypes via `define_id!` macro
- `scope` — Scope partitioning primitives
- `trace` — W3C-compatible trace context
- `digest` — BLAKE3 content digest computation

### ID Types (all: `new()`, `generate()`, `as_str()`, `is_empty()`, Display, Serialize, Deserialize, Hash, Ord)
`EnvelopeId`, `ClaimId`, `ClaimVersionId`, `EntityId`, `EpisodeId`, `AttemptId`, `TrialId`, `ArtifactId`, `ProjectionId`, `RelationId`, `RelationVersionId`, `ImportBatchId`

### Scope
```rust
pub struct Scope { pub namespace, pub domain?, pub workspace_id?, pub repo_id? }
pub struct ScopeKey { /* same fields, hashable/Ord */ }
  fn namespace_only(ns) -> Self
  fn from_legacy_namespace(ns) -> Self
  fn to_legacy_namespace(&self) -> &str
pub enum PhaseStatus { Current, Compatibility, PhaseGated }
```

### Trace
```rust
pub struct TraceCtx { pub trace_id, pub parent_id?, pub baggage: Vec<BaggageEntry> }
  fn generate() -> Self               // UUID v4 hex
  fn from_trace_id(id) -> Self
  fn from_legacy_trace_id(id) -> Self  // BLAKE3 hash for non-W3C IDs
  fn child(&self, span_id) -> Self
  fn add_baggage(&mut self, key, value) -> Result<(), TraceError>
  fn to_traceparent(&self) -> Result<String, TraceError>   // W3C header
  fn from_traceparent(header) -> Result<Self, TraceError>
pub struct BaggageEntry { pub key, pub value }
pub const MAX_BAGGAGE_ENTRIES: usize = 16;
pub const MAX_BAGGAGE_ITEM_BYTES: usize = 256;
```

### Digest
```rust
pub struct ContentDigest(pub String)  // 64-char hex BLAKE3
  fn compute(data: &[u8]) -> Self
  fn compute_str(data: &str) -> Self
  fn compute_json<T: Serialize>(value: &T) -> Result<Self, DigestError>
  fn from_hex(hex) -> Result<Self, DigestError>
pub struct DigestBuilder
  fn new() -> Self
  fn update(&mut self, data: &[u8]) -> &mut Self
  fn separator(&mut self) -> &mut Self
  fn finalize(self) -> ContentDigest
```

### Tests: ~60 (IDs, scope, trace W3C roundtrips, baggage limits, digest determinism)

---

## 2. agent-graph

**Crate:** `agent-graph` | **Version:** 0.2.0 | **License:** MIT
**Description:** Graph-based agent orchestration for Rust — LangGraph for the Rust ecosystem

### Dependencies
`tokio` 1, `futures` 0.3, `async-trait` 0.1, `serde` 1, `serde_json` 1, `thiserror` 1, `anyhow` 1, `chrono` 0.4, `uuid` 1, `tracing` 0.1, `stack-ids` (path), `rusqlite` 0.32 (optional)

### Features
- `checkpointing` (default) — SQLite-backed checkpoint persistence

### Core API

```rust
pub struct AgentGraph { /* ... */ }
  // Execution
  async fn execute(&self, start_node, state) -> Result<AgentState>
  async fn execute_with_config(&self, start_node, state, config) -> Result<AgentState>
  async fn execute_with_interrupt(&self, start_node, state) -> Result<ExecutionResult>
  fn execute_cancellable(&self, start_node, state) -> (JoinHandle, CancellationToken)
  fn stream(&self, start_node, state, config) -> (JoinHandle, Receiver<StreamEvent>)
  async fn resume(&self, run_id, checkpoint_store) -> Result<AgentState>
  // Introspection
  fn to_mermaid(&self) -> String
  fn node_names(&self) -> Vec<&String>
  fn compute_graph_hash(&self) -> String
  fn builder() -> AgentGraphBuilder
```

### Builder
```rust
pub struct AgentGraphBuilder
  fn add_node(self, name, node: Box<dyn Node>) -> Self
  fn add_node_with_retry(self, name, node, policy: RetryPolicy) -> Self
  fn add_subgraph(self, name, subgraph: AgentGraph) -> Self
  fn add_edge(self, from, to) -> Self
  fn add_conditional_edge(self, from, router: Box<dyn RoutingFunction>) -> Self
  fn with_max_iterations(self, max) -> Self
  fn with_reducer(self, key, reducer: impl Reducer) -> Self
  fn with_interrupt_before(self, nodes) -> Self
  fn with_interrupt_after(self, nodes) -> Self
  fn with_checkpoint_store(self, store: Arc<dyn CheckpointStore>) -> Self
  fn with_event_sink(self, sink: Arc<dyn EventSink>) -> Self
  fn build(self) -> Result<AgentGraph>
```

### State
```rust
pub struct AgentState  // Arc<RwLock> backed
  async fn get<T: DeserializeOwned>(&self, key) -> Result<T>
  async fn set<T: Serialize>(&self, key, value) -> Result<()>
  async fn fork(&self) -> AgentState
  async fn snapshot(&self) -> StateSnapshot
  async fn transaction(&self) -> StateTransaction
pub struct StateLimits { max_keys: 10_000, max_value_bytes: 1MiB, max_history_len: 100 }
```

### Node & Router
```rust
#[async_trait] pub trait Node: Send + Sync {
    async fn execute(&self, state: &AgentState, config: &GraphConfig) -> Result<NodeOutput>;
}
pub enum NodeOutput { Done, Command(Command) }
pub enum Navigation { Node(String), Nodes(Vec<String>), End, Send(Vec<SendOp>), Default }

#[async_trait] pub trait RoutingFunction: Send + Sync {
    async fn route(&self, state: &AgentState, config: &GraphConfig) -> Result<RouterOutput>;
}
pub enum RouterOutput { Next(Option<String>), FanOut(Vec<String>) }
```

### Reducers
`LastWriteWins`, `AppendReducer`, `AddReducer`, `MergeReducer`, `FnReducer<F>`

### Payload Integration
```rust
pub trait Payload: Send + Sync {
    fn invoke(&self, input: Value, ctx: &PayloadContext) -> Pin<Box<dyn Future<Output = Result<PayloadOutput>> + Send>>;
}
pub struct PayloadNode { fn new(payload) -> Self; fn with_input_selector(); fn with_output_mapper() }
```

### Event System
```rust
pub enum GraphEvent { RunStart, RunEnd, NodeStart, NodeEnd, Token, CheckpointWritten, InterruptRaised, StateUpdate, SuperstepStart, SuperstepEnd }
pub trait EventSink: Send + Sync { fn emit(&self, event: GraphEvent); }
// Impls: NoopEventSink, ChannelEventSink, CallbackEventSink, CompositeEventSink
```

### Checkpoint Store (Per-Attempt Granularity)
```rust
pub trait CheckpointStore: Send + Sync {
    fn create_run(&self, graph_name) -> Future<Result<RunId>>;
    fn record_attempt(&self, run_id, node_id, attempt, input) -> Future<Result<CheckpointAttemptId>>;
    fn complete_attempt(&self, attempt_id, output, meta) -> Future<Result<()>>;
    fn fail_attempt(&self, attempt_id, error) -> Future<Result<()>>;
    fn load_run(&self, run_id) -> Future<Result<Option<RunState>>>;
    // ...
}
pub struct InMemoryCheckpointStore
```

### Interrupt/Resume
```rust
pub enum ExecutionResult { Complete(AgentState), Interrupted { state, node, interrupt_value, checkpoint_data } }
pub struct Interrupt { pub kind: InterruptKind, pub payload: Value, pub correlation_id: String }
pub enum InterruptKind { AwaitInput, AwaitApproval, Custom(String) }
```

### Configuration
```rust
pub struct GraphConfig {
    pub thread_id?, pub trace_ctx?: TraceCtx, pub recursion_limit: 100,
    pub max_parallelism: 8 (max 32), pub tags, pub metadata, pub configurable
}
```

### Examples (14): basic, conditional, parallel, streaming, loop, research_agent, checkpointing, human_in_loop, map_reduce, pipeline_node, reducers, retry, subgraph, visualization
### Tests: 13 test files covering execution, parallel, interrupt, routing, reducers, state, checkpointing, subgraphs, retry, runtime

---

## 3. llm-pipeline

**Crate:** `llm-pipeline` | **Version:** 0.2.0 | **License:** MIT
**Description:** Reusable node payloads for LLM workflows: prompt templating, Ollama calls, defensive parsing, streaming, and sequential chaining

### Dependencies
`llm-output-parser` (path), `stack-ids` (path), `tokio` 1, `reqwest` 0.12 (json, stream), `serde` 1, `serde_json` 1, `anyhow` 1, `thiserror` 2, `futures` 0.3, `async-trait` 0.1, `fastrand` 2, `uuid` 1, `tracing` 0.1

### Features
- `yaml` — YAML output parsing
- `openai` — OpenAI-compatible backend

### Core Trait
```rust
pub trait Payload: Send + Sync {
    fn kind(&self) -> &'static str;
    fn name(&self) -> &str;
    fn invoke<'a>(&'a self, ctx: &'a ExecCtx, input: Value) -> BoxFut<'a, Result<PayloadOutput>>;
}
```

### LlmCall (Primary Payload)
```rust
pub struct LlmCall
  fn new(name, prompt_template) -> Self
  fn with_model(self, model) -> Self
  fn with_system(self, template) -> Self
  fn with_config(self, config: LlmConfig) -> Self
  fn with_output_strategy(self, strategy: OutputStrategy) -> Self
  fn with_retry(self, retry: RetryConfig) -> Self
  fn with_streaming(self, enabled: bool) -> Self
  // Shorthands
  fn expecting_json(self) -> Self
  fn expecting_list(self) -> Self
  fn expecting_choice(self, choices) -> Self
  fn expecting_number(self) -> Self
  fn expecting_text(self) -> Self
```

### Output Strategies
```rust
pub enum OutputStrategy {
    Lossy,                    // default, always succeeds
    Json,                     // strict JSON extraction + repair
    StringList,               // array of strings
    XmlTag(String),           // extract XML tag body
    Choice(Vec<String>),      // match one of N
    Number,                   // numeric extraction
    NumberInRange(f64, f64),  // bounded numeric
    Text,                     // cleaned prose
    Custom(CustomParseFn),    // caller-provided
}
```

### Execution Context
```rust
pub struct ExecCtx {
    pub client: Client, pub base_url: String, pub backend: Arc<dyn Backend>,
    pub backoff: BackoffConfig, pub vars: HashMap<String, String>,
    pub cancellation: Option<Arc<AtomicBool>>,
    pub event_handler: Option<Arc<dyn EventHandler>>,
    pub trace_ctx: TraceCtx, pub limits: PipelineLimits,
}
  fn builder(base_url) -> ExecCtxBuilder
  fn is_cancelled(&self) -> bool
```

### Backend Trait & Implementations
```rust
#[async_trait] pub trait Backend: Send + Sync {
    async fn complete(&self, client, base_url, request: &LlmRequest) -> Result<LlmResponse>;
    async fn complete_streaming(&self, client, base_url, request, on_token) -> Result<LlmResponse>;
    fn name(&self) -> &'static str;
}
pub struct OllamaBackend;                          // /api/generate or /api/chat
pub struct OpenAiBackend;                          // /v1/chat/completions (feature-gated)
pub struct MockBackend { fn fixed(response) -> Self }
pub struct RecordingBackend;                        // Record/replay
```

### Transport Retry
```rust
pub struct BackoffConfig {
    pub max_retries, pub initial_delay, pub multiplier, pub max_delay,
    pub jitter: JitterStrategy, pub retryable_statuses: Vec<u16>,
}
pub enum JitterStrategy { None, Full, Equal, Decorrelated }
  fn none() -> Self        // no retry
  fn standard() -> Self    // 3 retries, 2s base, jitter
```

### Semantic Retry
```rust
pub struct RetryConfig { pub max_retries (cap 5), pub validator?, pub cool_down: bool }
  fn requiring_keys(self, keys: &[&str]) -> Self  // validate JSON keys exist
```

### Resource Limits
```rust
pub struct PipelineLimits {
    pub max_response_bytes: 2MB, pub request_timeout: 120s, pub stream_idle_timeout: 30s
}
```

### Chain (Sequential Composition)
```rust
pub struct Chain
  fn new(name) -> Self
  fn push(self, payload: Box<dyn Payload>) -> Self
  async fn execute(&self, ctx, input) -> Result<PayloadOutput>
  async fn execute_all(&self, ctx, input) -> Result<Vec<PayloadOutput>>
```

### PayloadOutput
```rust
pub struct PayloadOutput {
    pub value: Value, pub raw_response: String, pub thinking: Option<String>,
    pub diagnostics: Option<ParseDiagnostics>, pub trace_ctx: Option<TraceCtx>,
    pub transport_retries_used: u32, pub semantic_retries_used: u32,
    pub response_bytes: usize, pub wall_time_ms: u64,
}
  fn parse_as<T: DeserializeOwned>(&self) -> Result<T>
```

### Events
```rust
pub enum Event {
    PayloadStart, Token, PayloadEnd, RetryStart, RetryEnd, PartialParse, TransportRetry
}
pub trait EventHandler: Send + Sync { fn on_event(&self, event: Event); }
```

### Error
```rust
pub enum PipelineError {
    Request, Json, Parse, StageFailed, Cancelled, InvalidConfig,
    HttpError { status, body, retry_after? }, ResponseTooLarge, StreamIdle, Timeout, Other
}
```

### Parsing Utilities
`extract_thinking()`, `extract_json_block()`, `extract_json_candidate()`, `parse_value_lossy()`, `parse_value_defensively()`, `parse_as::<T>()`

### Examples (6): mock_example, basic_pipeline, streaming_pipeline, context_injection, payload_chain, thinking_mode
### Tests: ~120+ unit + integration tests

---

## 4. semantic-memory

**Crate:** `semantic-memory` | **Version:** 0.5.0 | **MSRV:** 1.75 | **License:** MIT
**Description:** Hybrid semantic search with SQLite, FTS5, and HNSW — built for AI agents

### Dependencies
`rusqlite` 0.32 (bundled, blob), `reqwest` 0.12, `serde` 1, `serde_json` 1, `tokio` 1, `thiserror` 2, `tracing` 0.1, `uuid` 1, `chrono` 0.4, `bytemuck` 1, `stack-ids` (path), `hnsw_rs` 0.3 (optional)

### Features
- `hnsw` (default) — HNSW approximate nearest-neighbor
- `brute-force` — Brute-force vector search
- `testing` — Exposes `raw_execute()` for tests

### Configuration
```rust
pub struct MemoryConfig {
    pub base_dir: PathBuf,
    pub embedding: EmbeddingConfig,   // ollama_url, model (nomic-embed-text), dimensions (768)
    pub search: SearchConfig,         // bm25_weight, vector_weight, rrf_k (60), recency_half_life_days?
    pub chunking: ChunkingConfig,     // target_size (1000), overlap (200)
    pub pool: PoolConfig,             // busy_timeout, wal, max_read_connections (4)
    pub limits: MemoryLimits,         // max_facts_per_namespace (100k), max_content_bytes (1MB)
    pub hnsw: HnswConfig,            // m (16), ef_construction (200), ef_search (50)
}
```

### MemoryStore (Main API)
```rust
pub struct MemoryStore
  fn open(config) -> Result<Self>
  fn open_with_embedder(config, embedder: Box<dyn Embedder>) -> Result<Self>

  // Sessions & Messages
  async fn create_session(&self, channel) -> Result<String>
  async fn add_message(&self, session_id, role, content, token_count?, metadata?) -> Result<i64>
  async fn add_message_with_trace(&self, ..., trace_ctx?) -> Result<i64>
  async fn get_recent_messages(&self, session_id, limit) -> Result<Vec<Message>>
  async fn get_messages_within_budget(&self, session_id, max_tokens) -> Result<Vec<Message>>

  // Facts
  async fn add_fact(&self, namespace, content, source?, metadata?) -> Result<String>
  async fn add_fact_with_trace(&self, ..., trace_ctx?) -> Result<String>
  async fn add_fact_with_embedding(&self, namespace, content, embedding, source?, metadata?) -> Result<String>
  async fn update_fact(&self, fact_id, content) -> Result<()>
  async fn delete_fact(&self, fact_id) -> Result<()>
  async fn delete_namespace(&self, namespace) -> Result<usize>

  // Documents
  async fn ingest_document(&self, title, content, namespace, source_path?, metadata?) -> Result<String>
  async fn delete_document(&self, document_id) -> Result<()>

  // Search (Hybrid BM25 + Vector + RRF)
  async fn search(&self, query, top_k?, namespaces?, source_types?) -> Result<Vec<SearchResult>>
  async fn search_fts_only(&self, ...) -> Result<Vec<SearchResult>>
  async fn search_vector_only(&self, ...) -> Result<Vec<SearchResult>>
  async fn search_explained(&self, ...) -> Result<Vec<ExplainedResult>>
  async fn search_conversations(&self, query, top_k?, session_ids?) -> Result<Vec<SearchResult>>

  // Episodes
  async fn ingest_episode(&self, document_id, meta: &EpisodeMeta) -> Result<String>
  async fn update_episode_outcome_by_id(&self, episode_id, outcome, confidence, experiment_id?) -> Result<()>

  // Embedding
  async fn embed(&self, text) -> Result<Vec<f32>>
  async fn embedding_displacement(&self, text_a, text_b) -> Result<EmbeddingDisplacement>

  // Import
  async fn import_envelope(&self, envelope, records) -> Result<ImportReceipt>       // Legacy V10
  async fn import_projection_batch(&self, batch_json) -> Result<ImportReceipt>      // Canonical V11

  // Maintenance
  async fn verify_integrity(&self, mode) -> Result<IntegrityReport>
  async fn reconcile(&self, action) -> Result<IntegrityReport>
  async fn rebuild_hnsw_index(&self) -> Result<()>
  async fn stats(&self) -> Result<MemoryStats>
  fn graph_view(&self) -> Arc<dyn GraphView>
```

### Key Types
```rust
pub enum Role { System, User, Assistant, Tool }
pub struct SearchResult { pub content, pub source: SearchSource, pub score, pub bm25_rank, pub vector_rank, pub cosine_similarity }
pub enum SearchSource { Fact{..}, Chunk{..}, Message{..}, Episode{..} }
pub struct ExplainedResult { pub result: SearchResult, pub breakdown: ScoreBreakdown }
pub struct EpisodeMeta { pub cause_ids, pub effect_type, pub outcome, pub confidence, pub verification_status }
pub enum EpisodeOutcome { Confirmed, Refuted, Inconclusive, Pending }
pub trait GraphView { fn neighbors(); fn path() }
pub enum GraphEdgeType { Semantic, Temporal, Causal, Entity }
```

### Embedder Trait
```rust
pub trait Embedder: Send + Sync {
    fn embed(&self, text) -> Future<Result<Vec<f32>>>;
    fn embed_batch(&self, texts) -> Future<Result<Vec<Vec<f32>>>>;
    fn model_name(&self) -> &str;
    fn dimensions(&self) -> usize;
}
pub struct OllamaEmbedder;   // production
pub struct MockEmbedder;     // testing (deterministic)
```

### Quantization (SQ8)
```rust
pub struct Quantizer { fn quantize(&self, vector) -> Result<QuantizedVector> }
pub fn pack_quantized(qv) -> Vec<u8>       // [scale:f32][zero_point:i8][data:i8*dims]
pub fn unpack_quantized(bytes, dims) -> Result<QuantizedVector>
```

### Schema: V1-V11 progressive migrations (sessions, messages, facts, docs, chunks, FTS5, episodes, quantization, projections, entity aliases)
### Tests: 27 test files (concurrency, HNSW, search, quantization, import, integrity, migration)

---

## 5. semantic-memory-forge (schemas)

**Crate:** `semantic-memory-forge` | **Version:** 0.1.0 | **MSRV:** 1.75 | **License:** MIT
**Description:** Forge verification truth: evidence bundles, export envelopes, and causal estimation substrate

### Dependencies
`stack-ids` (path), `serde` 1, `serde_json` 1, `chrono` 0.4, `uuid` 1

### Evidence Bundle
```rust
pub struct EvidenceBundle {
    pub id: EvidenceBundleId, pub question: CausalQuestion, pub treatment: TreatmentSpec,
    pub outcome: OutcomeSpec, pub covariates: Vec<String>, pub estimator_kind: String,
    pub estimate: f64, pub confidence: f32, pub trial_count: u32,
    pub refutations: Vec<RefutationAttempt>,
    pub trace_ctx?: TraceCtx, pub attempt_id?: AttemptId, pub trial_id?: TrialId,
    pub source_envelope_id?: EnvelopeId, pub claim_ids: Vec<ClaimId>,
}
  fn all_refutations_passed(&self) -> bool
  fn has_failed_refutation(&self) -> bool
```

### Estimator Types
```rust
pub enum EstimatorKind { DiffInDiff, PropensityScore, InstrumentalVariables, OLS, Bayesian, BeforeAfter, Custom(String) }
pub struct EstimatorMeta { pub kind, pub version, pub parameters, pub environment?: EnvironmentFingerprint }
pub struct SidecarExecution { pub estimator, pub request, pub response?, pub duration_ms?, pub success }
```

### Export Authority
```rust
pub enum ExportAuthority { Forge, External { name } }
pub struct ForgeExportMeta { pub authority, pub run_id?, pub direct_write: bool }
```

### Tests: 7 (construction, refutation tracking, serde roundtrips)

---

## 6. forge-memory-bridge

**Crate:** `forge-memory-bridge` | **Version:** 0.1.0 | **MSRV:** 1.75 | **License:** MIT
**Description:** Transform Forge export envelopes into projection import batches for semantic-memory

### Dependencies
`stack-ids` (path), `serde` 1, `serde_json` 1, `thiserror` 2, `chrono` 0.4, `uuid` 1

### Export Side (envelope.rs)
```rust
pub struct ExportEnvelopeV1 {
    pub envelope_id: EnvelopeId, pub content_digest: ContentDigest,
    pub source_authority, pub scope_key: ScopeKey, pub trace_ctx?: TraceCtx,
    pub records: Vec<ExportRecord>,
}
  fn validate(&self) -> Result<(), BridgeError>
pub enum ExportRecord { Claim(..), Relation(..), Episode(..), EntityAlias(..), EvidenceRef(..) }
```

### Import Side (batch.rs)
```rust
pub struct ProjectionImportBatchV1 {
    pub source_envelope_id: EnvelopeId, pub content_digest: ContentDigest,
    pub scope_key: ScopeKey, pub trace_ctx?: TraceCtx,
    pub records: Vec<ImportProjectionRecord>,
}
pub enum ImportProjectionRecord { ClaimVersion(..), RelationVersion(..), Episode(..), EntityAlias(..), EvidenceRef(..) }
pub enum ClaimState { Active, Superseded, Retracted, Archived, PendingReview, Disputed }
pub enum ProjectionFreshness { Current, Stale, Superseded, ImportFailed, NeverImported, ImportLagging }
pub enum ContradictionStatus { None, PossibleContradiction{..}, Confirmed{..}, Resolved{..} }
pub enum MergeDecision { Automated{..}, HumanReviewed{..}, PendingReview, Rejected{..} }
pub enum ReviewState { Unreviewed, PendingReview, Approved{..}, Rejected{..} }
```

### Transform
```rust
pub fn transform_envelope(envelope: &ExportEnvelopeV1) -> Result<ProjectionImportBatchV1, BridgeError>
pub fn is_compatible_version(schema_version: &str) -> bool
pub fn bridge_trace_ctx(source: Option<&TraceCtx>) -> TraceCtx
```

### Legacy Compatibility
```rust
pub fn upgrade_legacy_envelope(legacy) -> Result<ExportEnvelopeV1, BridgeError>
pub fn transform_legacy_envelope(legacy) -> Result<ProjectionImportBatchV1, BridgeError>
```

### Key Invariants
- Content digest carries through unchanged
- Provenance (envelope_id, authority, trace_ctx) propagates unchanged
- Bridge preserves `transformed_at`; authoritative imported `recorded_at` is assigned by `semantic-memory` at import commit time
- No synthetic supersession synthesis (BRG-002)
- Entity alias: automated flows never set `is_human_confirmed_final=true`

### Tests: 38 (6 envelope, 14 transform, 7 legacy, 11 integration proofs)

---

## 7. knowledge-runtime

**Crate:** `knowledge-runtime` | **Version:** 0.1.0 | **MSRV:** 1.75 | **License:** MIT
**Description:** Bounded orchestration scaffold for semantic-memory: classification, routing, entity resolution, merge, projection tracking

### Dependencies
`semantic-memory` (path), `stack-ids` (path), `serde` 1, `serde_json` 1, `thiserror` 2, `tracing` 0.1, `chrono` 0.4, `uuid` 1

### Main Runtime
```rust
pub struct KnowledgeRuntime
  fn new(config: RuntimeConfig, adapter: SemanticMemoryAdapter) -> Result<Self>
  async fn query(&self, query, scope?) -> Result<(Vec<SearchResult>, QueryTrace)>
  async fn query_with_trace(&self, query, scope?, trace_ctx?) -> Result<(Vec<SearchResult>, QueryTrace)>
  fn classify(&self, query) -> ClassifyResult
  fn plan(&self, query, scope?) -> RoutePlan
  fn entity_registry(&self) -> &EntityRegistry
  fn projection_health(&self, id) -> ProjectionHealth
  fn record_projection_build(&mut self, id, source_count, build_duration_ms, version?)
  fn invalidate_projections(&mut self, event) -> ProjectionActionResult
```

### Query Pipeline
```rust
// 1. Classification
pub enum QueryMode { SemanticLookup, EntityLookup{mention}, TemporalLookup{temporal_expr}, Mixed{..} }
pub struct ClassifyResult { pub mode, pub confidence, pub reason? }

// 2. Routing
pub struct RoutePlan { pub query, pub scope, pub legs: Vec<RouteLeg> }
pub struct RouteLeg { pub strategy: RetrievalStrategy, pub limit, pub filter? }
pub enum RetrievalStrategy { HybridSearch, EntitySearch{mention}, TemporalSearch{temporal_expr} }

// 3. Merge
pub struct MergedResults { pub results: Vec<MergedItem>, pub duplicates_fused, pub total_raw }
pub struct MergedItem { pub result, pub final_score, pub source_legs, pub per_leg_scores }
```

### Entity Registry
```rust
pub struct EntityRegistry
  fn register(&mut self, entity: Entity) -> Result<()>
  fn resolve(&self, mention, scope: &ScopeKey) -> ResolveResult
pub enum MatchQuality { ExactCanonical, ExactAlias, ScopedFallback, Unresolved }
pub enum EntityKind { Person, Code, Project, Concept, Custom(String) }
pub fn code_entity_id(kind: &CodeEntityKind, qualified_path, scope) -> EntityId
```

### Projection Tracking (Observability-Only)
```rust
pub enum ProjectionHealth { Healthy, Stale, Missing, Rebuilding, ImportLagging, ImportFailed }
pub struct ProjectionTracker
pub trait RebuildDriver { async fn rebuild(&self, id) -> Result<RebuildOutcome>; fn can_rebuild(&self, id) -> bool; }
pub async fn rebuild_stale<D: RebuildDriver>(tracker, driver) -> Result<usize>
```

### Observability
```rust
pub struct QueryTrace { pub trace_ctx, pub scope, pub classification, pub plan, pub leg_timings_ms, pub warnings }
pub enum QueryWarning { TemporalDowngradedToHybrid, ScopePartiallyEnforced, EntityScopeFallback, ProjectionImportStale }
```

### Evidence & Temporal
```rust
pub struct EvidenceBundle { pub items: Vec<EvidenceItem>, pub aggregate_confidence }
pub struct TemporalClaim { pub entity_id, pub claim, pub valid_from?, pub valid_until?, pub confidence }
pub fn check_contradiction(a, b) -> ContradictionStatus
```

### Configuration
```rust
pub struct RuntimeConfig {
    pub default_scope: Scope,
    pub query: QueryConfig { max_results_per_leg: 20, max_route_legs: 4, default_limit: 10 },
    pub entity: EntityConfig { max_aliases: 16, max_entities: 10_000 },
    pub projection: ProjectionConfig { staleness_threshold_secs: 3600 },
    pub strict_temporal: bool, pub strict_scope: bool,
}
```

### Tests: 3 files (invariants, ugly cases, cross-crate proof with forge-memory-bridge)

---

## 8. job-queue

**Crate:** `job-queue` | **Version:** 0.2.0 | **License:** MIT
**Description:** Production-grade background job queue system

### Dependencies
`tokio` 1, `rusqlite` 0.32 (bundled), `serde` 1, `serde_json` 1, `anyhow` 1, `chrono` 0.4, `uuid` 1, `thiserror` 2, `tracing` 0.1, `stack-ids` (path)

### Core Trait
```rust
pub trait JobHandler: Send + Sync + Serialize + DeserializeOwned + Clone {
    async fn execute(&self, ctx: &JobContext) -> Result<JobResult, QueueError>;
    fn job_type(&self) -> &str { type_name::<Self>() }
}
```

### Queue Manager
```rust
pub struct QueueManager
  fn new(config: QueueConfig) -> Result<Self>
  fn add<H: JobHandler>(&self, job: QueueJob<H>) -> Result<String>
  fn cancel(&self, job_id) -> Result<()>
  fn reorder(&self, job_id, new_priority) -> Result<()>
  fn prune(&self, days: u32) -> Result<u32>
  fn list_jobs(&self) -> Result<Vec<(String, String)>>
  fn get_job_details(&self, job_id) -> Result<Option<QueueJobDetails>>
  fn count_by_status(&self) -> Result<QueueStats>
  async fn process_one<H>(&self, emitter) -> Result<Option<ProcessedJob>>
  fn pause(&self) / fn resume(&self) / fn shutdown(&self)
  fn spawn<H>(self, emitter) -> Arc<Self>      // background executor
  fn spawn_on<H>(self, emitter, handle) -> Arc<Self>
```

### Job Types
```rust
pub struct QueueJob<T> {
    pub id, pub trace_ctx?: TraceCtx, pub attempt_id?: AttemptId, pub trial_id?: TrialId,
    pub priority: QueuePriority, pub status: QueueJobStatus, pub data: T,
}
pub enum QueuePriority { Low, Normal, High }
pub enum QueueJobStatus { Pending, Processing, Completed, Failed, Cancelled }
pub struct JobResult { pub success, pub output?, pub error?, pub failure_class? }
pub enum FailureClass { Transient, Permanent, RateLimited { retry_after_secs } }
```

### Context & Events
```rust
pub struct JobContext {
    pub job_id, pub trace_ctx?: TraceCtx, pub attempt_id?: AttemptId, pub trial_id?: TrialId,
}
  fn emit_progress(&self, current, total)
  fn is_cancelled(&self) -> bool

pub trait QueueEventEmitter: Send + Sync + 'static {
    fn emit_job_started(&self, event); fn emit_job_completed(&self, event);
    fn emit_job_failed(&self, event); fn emit_job_progress(&self, event);
    fn emit_job_cancelled(&self, event);
}
```

### Configuration
```rust
pub struct QueueConfig {
    pub db_path?: PathBuf, pub worker_id, pub cooldown: 0s, pub max_consecutive: 0,
    pub poll_interval: 3s, pub heartbeat_interval: 10s, pub stale_after: 300s, pub max_retries: 3,
}
```

### Retry: exponential backoff (2^(attempt-1) * 5s, cap 5min), respects FailureClass
### Schema: V1-V4 (lifecycle, trace_id, attempt_id/trial_id)
### Tests: ~60 (db, executor, state transitions, retry, lineage persistence)

---

## 9. AI-Batch-Queue

**Crate:** `ai-batch-queue` | **Version:** 0.2.0 | **License:** MIT
**Description:** Model-aware batch processing queue with ETA estimation for Tauri applications

### Dependencies
`tauri` 2, `tokio` 1, `serde` 1, `serde_json` 1, `anyhow` 1, `chrono` 0.4, `uuid` 1, `thiserror` 2, `tracing` 0.1, `stack-ids` (path)

### Core Trait
```rust
pub trait BatchItemHandler<D: Clone + Send + Sync + Serialize>: Send + Sync + 'static {
    async fn process(&self, data: &D, resource_key: &str, operation: &str) -> Result<ItemResult>;
    fn should_skip(&self, _data: &D, _operation: &str) -> bool { false }
}
```

### Queue
```rust
pub struct BatchQueue<D>
  fn new() -> Self
  fn with_scheduling(config: SchedulingConfig) -> Self
  fn enqueue(&self, job: BatchJob<D>) -> Result<String>
  fn next_queued(&self) -> Option<BatchJob<D>>
  fn update_item(&self, job_id, item_id, status, error?, duration_ms?) -> Result<()>
  fn mark_completed(&self, job_id) -> Result<Option<BatchCompletionSummary>>
  fn cancel_job(&self, job_id) -> Result<()>
  fn retry_failed(&self, job_id) -> Result<()>
  fn estimate_remaining(&self, job_id) -> Option<EtaEstimate>
```

### Types
```rust
pub struct BatchJob<D> { pub id, pub resource_key, pub operation, pub items: Vec<BatchItem<D>>, pub status }
pub struct BatchItem<D> { pub id, pub data: D, pub status, pub size_bucket, pub trace_ctx?, pub attempt_id?, pub trial_id? }
pub enum SizeBucket { Small, Medium, Large, Unknown }
pub enum BatchItemStatus { Pending, Running, Completed, Failed, Skipped, Cancelled }
pub struct EtaEstimate { pub remaining_ms, pub items_remaining, pub avg_item_ms, pub confidence: EtaConfidence }
pub struct SchedulingConfig { pub max_consecutive_same_key: 3, pub enable_reordering: true }
```

### Job Builders
```rust
pub fn build_job<D>(resource_key, operation, overwrite_policy, items) -> BatchJob<D>
pub fn build_job_traced<D>(resource_key, operation, overwrite_policy, items, trace_ctx) -> BatchJob<D>
```

### Executor (Tauri Events)
```rust
pub fn spawn<D, H>(app_handle, handler)                    // 2s poll
pub fn spawn_with_interval<D, H>(app_handle, handler, interval)
// Emits: ai_batch:job_started, ai_batch:item_progress, ai_batch:job_completed
```

### Tests: 65+ (queue, ETA, reordering, retry lineage, integration)

---

## 10. Tauri-Queue

**Crate:** `tauri-queue` | **Version:** 0.3.0 | **License:** MIT
**Description:** Tauri integration for job-queue background job processing

### Dependencies
`job-queue` (path), `tauri` 2, `tokio` 1, `serde` 1, `serde_json` 1, `stack-ids` (path)

### Features
- `sqlite` (default) — SQLite persistence

### Key Types
```rust
pub struct TauriEventEmitter { fn new(app_handle) -> Self; fn arc(app_handle) -> Arc<dyn QueueEventEmitter> }
// Emits: queue:job_started, queue:job_completed, queue:job_failed, queue:job_progress, queue:job_cancelled

pub struct CoalescingEmitter
  fn new(inner: Arc<dyn QueueEventEmitter>, config: EmitterConfig) -> Self
  fn arc(inner, config) -> Arc<dyn QueueEventEmitter>

pub struct EmitterConfig {
    pub buffer_size: 256, pub drop_policy: DropNewest, pub coalesce_interval_ms: 50,
    pub include_trace_ctx: true,
}
pub enum DropPolicy { DropOldest, DropNewest, Block }

pub fn trace_ctx_from_event_trace_id(trace_id: &Option<String>) -> Option<TraceCtx>
```

### Re-exports: All of `job-queue` (QueueManager, QueueConfig, QueueJob, JobHandler, etc.) + `stack_ids::TraceCtx`
### Tests: 36 (queue management, priority, coalescing, trace context)

---

## 11. Tauri-React-Hooks

**Package:** `@tauri-hooks/core` | **Version:** 0.1.0 | **License:** MIT
**Description:** React hooks for Tauri 2 apps — async-safe event listeners, command invocation, config management, stream buffering

### Peer Dependencies
`react >= 18`, `@tauri-apps/api >= 2`

### Hooks

```typescript
// Single event listener with async-safe cleanup
function useTauriEvent<T>(event: string, handler: (payload: T) => void, deps?: DependencyList): void

// Multiple event listeners, atomic setup/teardown
function useTauriEvents(bindings: Record<string, (payload: any) => void>, deps?: DependencyList): void

// Auto-fetching query with event-driven refresh
function useTauriQuery<T>(command: string, args?: Record<string, unknown>, options?: {
    enabled?: boolean; refreshOn?: string[];
}, deps?: DependencyList): { data: T | null; loading: boolean; error: string | null; refresh: () => Promise<void> }

// Explicit mutation (no auto-execute)
function useTauriMutation<TArgs extends unknown[], TResult>(command: string, argsFn?, options?: {
    onSuccess?: (result: TResult) => void; onError?: (error: string) => void;
}): { mutate: (...args: TArgs) => Promise<TResult>; loading: boolean; error: string | null; reset: () => void }

// Config load/save/update
function useTauriConfig<T>(loadCmd: string, saveCmd: string, saveArgName?: string): {
    config: T | null; loading: boolean; saving: boolean; error: string | null;
    save: (updated: T) => Promise<boolean>; update: (partial: Partial<T>) => void; reload: () => Promise<void>;
}

// High-frequency data batching (two-layer buffer)
function useBufferedStream<K extends string>(options?: { interval?: number }): {
    buffers: Record<K, string>; push: (key: K, data: string) => void;
    start: () => void; stop: () => void; clear: (key?: K) => void;
}
```

### Build: tsup (ESM + CJS), TypeScript strict, ~2KB gzipped
### Tests: None (examples in demo-usage.md show 58-91% LOC reduction vs manual)

---

## 12. ComfyUI-RS

**Crate:** `comfyui-rs` | **Version:** 0.2.0 | **License:** MIT
**Description:** Async Rust client for ComfyUI — REST, WebSocket progress, and workflow building

### Dependencies
`reqwest` 0.12 (json, multipart), `tokio` 1, `tokio-tungstenite` 0.24, `futures-util` 0.3, `serde` 1, `serde_json` 1, `thiserror` 2, `rand` 0.9, `tracing` 0.1

### Client
```rust
pub struct ComfyClient
  fn new(endpoint) -> Self
  fn with_client_id(self, id) -> Self
  async fn health(&self) -> Result<bool>
  async fn queue_prompt(&self, workflow: &Value) -> Result<String>       // returns prompt_id
  async fn history(&self, prompt_id) -> Result<Option<PromptHistory>>
  async fn image(&self, img: &ImageRef) -> Result<Vec<u8>>
  async fn queue_status(&self) -> Result<QueueStatus>
  async fn free_memory(&self, unload_models: bool) -> Result<()>
  async fn interrupt(&self) -> Result<()>
  async fn upload_image(&self, bytes, filename, overwrite) -> Result<String>
  async fn checkpoints(&self) -> Result<Vec<String>>
  async fn samplers(&self) -> Result<Vec<String>>
  async fn schedulers(&self) -> Result<Vec<String>>
  async fn wait_for_completion(&self, prompt_id, timeout) -> Result<GenerationOutcome>
  async fn wait_for_completion_ws<F>(&self, prompt_id, timeout, on_progress: F) -> Result<GenerationOutcome>
```

### Workflow Builder
```rust
pub struct Txt2ImgRequest
  fn new(prompt, checkpoint) -> Self
  fn negative(self, prompt) -> Self / fn size(self, w, h) -> Self / fn steps(self, n) -> Self
  fn cfg_scale(self, cfg) -> Self / fn sampler(self, s) -> Self / fn seed(self, s) -> Self
  fn build(&self) -> (Value, i64)  // (7-node workflow JSON, actual_seed)
```

### Types
```rust
pub enum GenerationOutcome { Completed { images: Vec<ImageRef> }, Failed { error }, TimedOut }
pub struct ImageRef { pub filename, pub subfolder, pub img_type }
pub struct QueueStatus { pub running: u32, pub pending: u32 }
```

### Examples (3): simple_generation, progress_tracking, workflow_builder
### Tests: 23 (client parsing, workflow nodes, serialization)

---

## 13. Ollama-Vision-RS

**Crate:** `ollama-vision` | **Version:** 0.2.0 | **License:** MIT
**Description:** Robust Ollama vision model toolkit for image tagging and captioning

### Dependencies
`reqwest` 0.12 (json), `serde` 1, `serde_json` 1, `tokio` 1, `thiserror` 2, `base64` 0.22, `llm-output-parser` (path)

### API
```rust
pub async fn tag_image(client, config, image_path, options: &TagOptions) -> Result<Vec<String>, TagError>
pub async fn tag_image_base64(client, config, image_b64, options) -> Result<Vec<String>, TagError>
pub async fn caption_image(client, config, image_path, options: &CaptionOptions) -> Result<String, CaptionError>
pub async fn caption_image_base64(client, config, image_b64, options) -> Result<String, CaptionError>

pub fn parse_tags(input: &str) -> Result<Vec<String>, ParseError>    // re-export from llm-output-parser
pub fn strip_think_tags(input: &str) -> String                       // re-export
```

### Configuration
```rust
pub struct OllamaVisionConfig {
    pub endpoint: String, pub model: String, pub timeout: Duration, pub connect_timeout: Duration,
    pub options: GenerateOptions,
}
  fn with_model(model) -> Self
pub struct TagOptions { pub prompt?, pub request_json_format: true, pub max_tags: 30, pub max_tag_length: 50, pub max_retries: 2 }
pub struct CaptionOptions { pub prompt?, pub max_caption_length: 500, pub max_retries: 2 }
```

### Examples (3): tag_images, caption_images, thinking_mode
### Tests: 7 (truncation, defaults, parse_tags smoke)

---

## 14. Primitives

**Location:** `Primitives/` | **10 crates** | All v0.1.0, MIT, Rust 1.75+

### Crate Inventory

| Crate | Description |
|-------|-------------|
| **typed-patch** | Structured patch model with validation and application |
| **sandbox-workspace** | Safe workspace staging and patch filesystem helpers |
| **forge-policy** | Path, environment, and database guardrails |
| **effect-signature** | Stable identifiers for validation effects |
| **check-runner** | Host/container execution for patch verification |
| **cea-core** | Causal edit attribution: graphs, scoring, prediction |
| **cea-store** | Persistence interface for CEA graphs |
| **cea-sqlite** | SQLite implementation of cea-store |
| **mindstate-core** | Deterministic rendering and hashing for agent mindstate |
| **stabilizer-core** | Attempt-phase and novelty helpers for patch generation |

### Dependency Flow
```
typed-patch --> sandbox-workspace --> forge-policy
check-runner --> effect-signature + sandbox-workspace + forge-policy
cea-core --> typed-patch + check-runner
cea-store --> cea-core + check-runner
cea-sqlite --> cea-store + forge-policy
mindstate-core (standalone)
stabilizer-core --> typed-patch
```

### Key Types (typed-patch)
```rust
pub struct StructuredPatch { pub patch_id: Uuid, pub summary, pub edits: Vec<FileEdit>, pub notes }
pub struct FileEdit { pub path, pub ops: Vec<EditOp>, pub mode?: FileMode }
pub enum EditOp { Insert { anchor, lines }, Delete { range }, Replace { range, lines } }
pub enum Anchor { AfterLine{..}, BeforeLine{..}, AfterMatch{..}, BeforeMatch{..} }
pub fn validate_patch(patch, policy) -> ValidationResult
pub fn apply_patch(patch, fs) -> Result<LineAttributionMap>
```

### Key Types (check-runner)
```rust
pub trait ExecutionBackend: Send + Sync {
    async fn prepare_workspace(&self, fixture) -> Result<Workspace>;
    async fn run_command(&self, workspace, program, args, env, timeout) -> Result<CommandOutput>;
}
pub struct CheckResult { pub fmt_pass, pub clippy_pass, pub test_pass, pub total_duration_ms }
pub enum CheckKind { Fmt, Clippy, Test }
pub fn select_backend(config) -> Result<Box<dyn ExecutionBackend>>
```

### Key Types (cea-core)
```rust
pub struct AttributionTriple { pub cause: EditOpSignature, pub effect: EffectSignature, pub distance, pub weight }
pub struct CausalGraph { /* petgraph DiGraph */ }
  fn ingest_run(&mut self, result: &AttributedRunResult)
  fn apply_decay(&mut self, factor: f64)
pub fn attribute_effects(patch, check_result, line_map, max_distance) -> Result<Vec<AttributionTriple>>
pub fn predict(signatures, graph, config) -> CausalPrediction
pub struct CausalPrediction { pub predicted_correctness, pub predicted_novelty, pub confidence, pub risk_flags }
```

### Key Types (forge-policy)
```rust
pub fn validate_forbidden_paths(paths, forbidden_patterns, allow_test_mods) -> Vec<Violation>
pub fn validate_patch_caps(files_changed, total_lines, per_file, max_files, max_total, max_per_file) -> Vec<Violation>
pub fn verify_sqlite_db_identity(path, spec) -> Result<()>
pub fn is_env_allowed(key) -> bool
```

### Key Types (mindstate-core)
```rust
pub struct MindState { pub request, pub repo_context, pub evidence: Vec<EvidenceItem>, pub traces: Vec<TraceRef> }
  fn render(&self) -> Result<String>
  fn hash(&self) -> Result<String>   // BLAKE3
```

### Key Types (stabilizer-core)
```rust
pub enum AttemptPhase { Innovative, Stabilize1, Stabilize2, Clamp }
pub struct Stabilizer { fn next_attempt(&mut self) -> Result<AttemptOverrides>; fn has_next() -> bool }
pub fn extract_strategy_tags(patch) -> Vec<StrategyTag>
pub fn compute_tag_novelty(current_tags, recent_tags) -> f64
```

---

## 15. living-memory

**Crate:** `semantic-memory-forge` | **Version:** 0.2.0 | **License:** MIT
**Description:** Causal edit attribution and structured patch evaluation engine (full forge)

### Dependencies
All 10 Primitives crates + `semantic-memory` (path) + `tokio` 1, `serde` 1, `serde_json` 1, `thiserror` 2, `anyhow` 1, `uuid` 1, `blake3` 1, `rusqlite` 0.32, `tempfile` 3, `walkdir` 2, `petgraph` 0.6, `regex` 1, `async-trait` 0.1, `chrono` 0.4, `rand` 0.9, `similar` 2

### Features
- `danger-sm-write` — Enables write-through to semantic-memory for episodes

### Module Structure
```
config          → ForgeConfig (master config with ~15 sub-configs)
error           → ForgeError (20+ variants), ForgeResult
baseline        → Provenance capture, comparability policies
runtime/
  mindstate     → MindState compilation from evidence + traces
  patch/        → validate, apply, render_diff (from typed-patch)
  novelty       → Strategy tags, novelty scoring (from stabilizer-core)
  stabilizer    → AttemptPhase control
exec/
  backend       → ExecutionBackend trait, select_backend
  host           → HostBackend (from check-runner)
  container     → ContainerBackend (from check-runner)
adapters/       → ProjectAdapter trait, CargoAdapter
lab/
  suite         → EvalTask, EvalSuite loading
  evaluate      → ScoreVector, compute_correctness, compute_novelty_score
  evidence      → EvidenceBundle (Phase 5), HypothesisEdge, VerificationPlan
  archive       → MAP-Elites archive (cell_key, insert, replacement)
  promote       → BasisVersion, promotion gates
  emitters      → AlgebraSpec mutation, crossover
cea/
  graph         → CausalGraph (from cea-core)
  instrumentation → Attribution, run hashing
  predictor     → Risk prediction
  store         → Persistence (cea-sqlite)
store/          → ForgeStore (SQLite, Mutex-protected)
experiment      → ExperimentDiff, EffectKind, TrialRecord, paired execution
export          → EpisodeExport, semantic-memory import envelope generation
failure         → FailureClass, FailureRecord
scoring         → ObjectivePolicy (BugFix/Refactor/Safety/Perf/Exploration)
invariants      → Policy enforcement, CEA raw source validation
```

### Key Types
```rust
pub struct ForgeConfig {
    pub mode: "standard" | "sealed_local",
    pub caps: CapsConfig { max_files: 8, max_total_lines: 400 },
    pub lab: LabConfig { promotion_min_suite_pass_rate: 0.95, promotion_min_weighted_improvement: 0.05 },
    pub cea: CeaConfig { zero_shot_coverage_threshold: 0.80, risk_confidence_threshold: 0.60 },
    pub limits: ForgeLimits { max_patch_bytes: 1MB, max_check_runtime: 300s, max_graph_nodes: 10k },
    pub comparability: ComparabilityPolicy { require_fingerprint_match: true, min_trials: 3 },
    // + mindstate, novelty, stabilization, container, workspace, statistics, danger
}

pub struct ForgeStore
  fn open(path) -> ForgeResult<Self>
  fn with_transaction<F, T>(&self, f) -> ForgeResult<T>
  // CRUD: candidates, eval runs, archive cells, evidence bundles, traces, export receipts, CEA nodes/edges

pub struct EvidenceBundle {
    pub bundle_id, pub candidate_id, pub eval_id, pub version_id, pub scores: ScoreVector,
    pub causal_question, pub claim_strength: ProvisionalSinglePair,
    pub treatment: Treatment, pub covariates: Covariates,
    pub hypothesis_edges: Vec<HypothesisEdge>, pub receipts: Vec<ReceiptRef>,
    pub sealed: bool,
}
  fn seal(&mut self) -> ForgeResult<()>
  fn to_episode_meta(&self) -> Value
  fn to_episode_content(&self) -> String

pub struct ScoreVector { pub correctness, pub novelty, pub stability, pub weighted_total, pub cea_confidence }
pub struct ExperimentDiff { pub effects, pub regressions, pub improvements, pub stable_failures, pub statistically_meaningful }
  fn from_paired(baseline: &CheckResult, patched: &CheckResult) -> Self
pub struct BasisVersion { pub version_id, pub candidate_id, pub frozen_spec: AlgebraSpec, pub cea_fingerprint }
```

### Evaluation Flow
```
EvalTask (fixture) → StructuredPatch (candidate) → validate_patch()
  → ExecutionBackend::prepare_workspace() + run_command() → CheckResult
  → compute_scores() → ScoreVector
  → ExperimentDiff::from_paired(baseline, patched)
  → EvidenceBundle (with hypothesis edges, receipts)
  → archive_insert() (MAP-Elites) → promote() (BasisVersion)
  → EpisodeExport → semantic-memory ingestion
```

### Tests: 11 files (CEA, evidence, exec, experiment, export, lab, migration, patch, phase5, runtime, safety)

---

## Cross-Cutting Concerns

### Trace Context (stack-ids)
All crates use `stack_ids::TraceCtx` as the canonical in-process trace form. Legacy `trace_id: String` fields are `#[deprecated]` with migration helpers. Resolution: `trace_ctx` takes precedence; legacy auto-derived via BLAKE3 hash.

### Retry Ownership
- **job-queue**: Owns retry for background jobs (AttemptId per re-enqueue, TrialId per execution)
- **llm-pipeline**: Owns transport retry (BackoffConfig) and semantic retry (RetryConfig with LLM-in-the-loop)
- **AI-Batch-Queue**: Owns batch-item retry (retry_failed resets Failed→Pending, preserves AttemptId)

### Authority Boundaries
| Layer | Authority |
|-------|-----------|
| `semantic-memory-forge` (schemas) | Raw verification truth |
| `semantic-memory` | Queryable projected truth |
| `forge-memory-bridge` | Transformation only |
| `knowledge-runtime` | Planning/merge only (non-authoritative) |
| `stack-ids` | Canonical cross-crate IDs, trace, digest |

### Error Pattern
All error enums implement `fn kind(&self) -> &'static str` for stable programmatic discriminants.

### Serde Convention
Tauri-facing types use `#[serde(rename_all = "camelCase")]`. Internal types use default (snake_case). All optional fields use `#[serde(skip_serializing_if = "Option::is_none")]`.
