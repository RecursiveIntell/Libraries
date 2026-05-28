# LATEST5.md — Code-Only Reference (All Crates)

> **Type:** current-state snapshot
> **Generated:** 2026-03-07
> **Source:** Direct source code reads across entire workspace. No prior markdown used.

---

## Table of Contents

1. [semantic-memory](#1-semantic-memory)
2. [knowledge-runtime](#2-knowledge-runtime)
3. [agent-graph](#3-agent-graph)
4. [LLM-Pipeline](#4-llm-pipeline)
5. [job-queue](#5-job-queue)
6. [AI-Batch-Queue](#6-ai-batch-queue)
7. [Tauri-Queue](#7-tauri-queue)
8. [ComfyUI-RS](#8-comfyui-rs)
9. [Ollama-Vision-RS](#9-ollama-vision-rs)
10. [stack-ids](#10-stack-ids)
11. [forge-memory-bridge](#11-forge-memory-bridge)
12. [Tauri-React-Hooks](#12-tauri-react-hooks)
13. [llm-output-parser](#13-llm-output-parser)
14. [semantic-memory-forge (living-memory)](#14-semantic-memory-forge)
15. [Primitives](#15-primitives)

---

## 1. semantic-memory

**Version:** 0.5.0
**Edition:** 2021
**Tests:** 234
**Schema:** V11 (19+ SQLite tables)

### Dependencies
- rusqlite (0.32, bundled)
- serde / serde_json
- uuid (v4)
- blake3
- tokio (rt, sync, macros)
- chrono (serde)
- thiserror (2)
- tracing (0.1)
- parking_lot
- rand

### Core Type: MemoryStore

```rust
pub struct MemoryStore { /* SQLite-backed, thread-safe */ }

impl MemoryStore {
    // Construction
    pub fn open(path: &Path) -> Result<Self, MemoryError>
    pub fn open_with_config(path: &Path, config: MemoryStoreConfig) -> Result<Self, MemoryError>
    pub fn in_memory() -> Result<Self, MemoryError>
    pub fn in_memory_with_config(config: MemoryStoreConfig) -> Result<Self, MemoryError>

    // Documents
    pub fn store_document(&self, doc: &Document) -> Result<DocumentId, MemoryError>
    pub fn get_document(&self, id: &DocumentId) -> Result<Option<Document>, MemoryError>
    pub fn delete_document(&self, id: &DocumentId) -> Result<bool, MemoryError>
    pub fn list_documents(&self, namespace: &str) -> Result<Vec<Document>, MemoryError>
    pub fn count_documents(&self, namespace: &str) -> Result<usize, MemoryError>

    // Knowledge / Facts
    pub fn store_knowledge(&self, fact: &Fact) -> Result<FactId, MemoryError>
    pub fn get_fact(&self, id: &FactId) -> Result<Option<Fact>, MemoryError>
    pub fn delete_fact(&self, id: &FactId) -> Result<bool, MemoryError>
    pub fn list_facts(&self, namespace: &str) -> Result<Vec<Fact>, MemoryError>
    pub fn count_facts(&self, namespace: &str) -> Result<usize, MemoryError>

    // Conversations
    pub fn store_conversation(&self, conv: &Conversation) -> Result<ConversationId, MemoryError>
    pub fn get_conversation(&self, id: &ConversationId) -> Result<Option<Conversation>, MemoryError>
    pub fn delete_conversation(&self, id: &ConversationId) -> Result<bool, MemoryError>
    pub fn list_conversations(&self, namespace: &str) -> Result<Vec<Conversation>, MemoryError>
    pub fn add_message(&self, conv_id: &ConversationId, msg: &Message) -> Result<MessageId, MemoryError>
    pub fn get_messages(&self, conv_id: &ConversationId) -> Result<Vec<Message>, MemoryError>
    pub fn delete_message(&self, msg_id: &MessageId) -> Result<bool, MemoryError>

    // Episodes
    pub fn store_episode(&self, episode: &Episode) -> Result<EpisodeId, MemoryError>
    pub fn get_episode(&self, id: &EpisodeId) -> Result<Option<Episode>, MemoryError>
    pub fn delete_episode(&self, id: &EpisodeId) -> Result<bool, MemoryError>
    pub fn list_episodes(&self, namespace: &str) -> Result<Vec<Episode>, MemoryError>

    // Search
    pub fn search(&self, query: &SearchQuery) -> Result<Vec<SearchResult>, MemoryError>
    pub fn search_messages(&self, query: &MessageSearchQuery) -> Result<Vec<MessageSearchResult>, MemoryError>
    pub fn search_with_trace(&self, query: &SearchQuery) -> Result<(Vec<SearchResult>, SearchTrace), MemoryError>

    // Graph
    pub fn store_relation(&self, relation: &Relation) -> Result<RelationId, MemoryError>
    pub fn get_relations(&self, entity_id: &str, namespace: &str) -> Result<Vec<Relation>, MemoryError>
    pub fn delete_relation(&self, id: &RelationId) -> Result<bool, MemoryError>
    pub fn get_entity_graph(&self, entity_id: &str, namespace: &str, depth: u32) -> Result<EntityGraph, MemoryError>

    // Import / Export
    pub fn import_envelope(&self, envelope: &ImportEnvelope) -> Result<ImportReceipt, MemoryError>
    pub fn query_import_log(&self, namespace: &str) -> Result<Vec<ImportLogEntry>, MemoryError>
    pub fn last_import_at(&self, namespace: &str) -> Result<Option<String>, MemoryError>

    // Embedding / HNSW
    pub fn rebuild_hnsw_index(&self, namespace: &str) -> Result<HnswRebuildStats, MemoryError>
    pub fn get_hnsw_stats(&self, namespace: &str) -> Result<HnswStats, MemoryError>

    // Storage Management
    pub fn compact(&self) -> Result<CompactionStats, MemoryError>
    pub fn vacuum(&self) -> Result<(), MemoryError>
    pub fn database_size(&self) -> Result<u64, MemoryError>

    // Quantization
    pub fn quantize_vectors(&self, namespace: &str, config: &QuantizationConfig) -> Result<QuantizationStats, MemoryError>
    pub fn get_quantization_stats(&self, namespace: &str) -> Result<Option<QuantizationStats>, MemoryError>

    // Pool
    pub fn pool_stats(&self) -> PoolStats
}
```

### Data Types

```rust
pub struct Document {
    pub id: Option<DocumentId>,
    pub namespace: String,
    pub title: String,
    pub content: String,
    pub metadata: serde_json::Value,
    pub embedding: Option<Vec<f32>>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

pub struct Fact {
    pub id: Option<FactId>,
    pub namespace: String,
    pub subject: String,
    pub predicate: String,
    pub object: String,
    pub confidence: f64,
    pub source: Option<String>,
    pub metadata: serde_json::Value,
    pub embedding: Option<Vec<f32>>,
    pub created_at: Option<String>,
}

pub struct Conversation {
    pub id: Option<ConversationId>,
    pub namespace: String,
    pub title: String,
    pub metadata: serde_json::Value,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

pub struct Message {
    pub id: Option<MessageId>,
    pub conversation_id: ConversationId,
    pub role: MessageRole,
    pub content: String,
    pub metadata: serde_json::Value,
    pub embedding: Option<Vec<f32>>,
    pub created_at: Option<String>,
}

pub enum MessageRole { User, Assistant, System }

pub struct Episode {
    pub id: Option<EpisodeId>,
    pub namespace: String,
    pub session_id: Option<String>,
    pub content: String,
    pub metadata: serde_json::Value,
    pub embedding: Option<Vec<f32>>,
    pub created_at: Option<String>,
}

pub struct Relation {
    pub id: Option<RelationId>,
    pub namespace: String,
    pub source_entity: String,
    pub relation_type: String,
    pub target_entity: String,
    pub weight: f64,
    pub metadata: serde_json::Value,
    pub created_at: Option<String>,
}

pub struct EntityGraph {
    pub center: String,
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
}
pub struct GraphNode { pub entity_id: String, pub depth: u32 }
pub struct GraphEdge { pub source: String, pub target: String, pub relation_type: String, pub weight: f64 }
```

### Search Types

```rust
pub struct SearchQuery {
    pub namespace: String,
    pub query_text: String,
    pub query_embedding: Option<Vec<f32>>,
    pub limit: usize,
    pub min_score: Option<f64>,
    pub search_type: SearchType,
    pub recency_weight: Option<f64>,
}

pub enum SearchType { Semantic, Keyword, Hybrid { semantic_weight: f64 } }

pub struct SearchResult {
    pub id: String,
    pub content: String,
    pub score: f64,
    pub result_type: SearchResultType,
    pub metadata: serde_json::Value,
}

pub enum SearchResultType { Document, Fact, Episode, Message }

pub struct MessageSearchQuery {
    pub namespace: String,
    pub query_text: String,
    pub query_embedding: Option<Vec<f32>>,
    pub limit: usize,
    pub conversation_id: Option<ConversationId>,
    pub recency_weight: Option<f64>,
}

pub struct MessageSearchResult {
    pub message: Message,
    pub conversation_id: ConversationId,
    pub score: f64,
}

pub struct SearchTrace {
    pub stages: Vec<SearchStage>,
    pub total_candidates: usize,
    pub final_count: usize,
}
pub struct SearchStage { pub name: String, pub candidates_in: usize, pub candidates_out: usize, pub duration_ms: u64 }
```

### Configuration

```rust
pub struct MemoryStoreConfig {
    pub embedding_dimensions: usize,      // default: 384
    pub hnsw: HnswConfig,
    pub pool_size: usize,                 // default: 4
    pub busy_timeout_ms: u64,             // default: 5000
}

pub struct HnswConfig {
    pub m: usize,                         // default: 16
    pub ef_construction: usize,           // default: 200
    pub ef_search: usize,                 // default: 50
    pub max_elements: usize,              // default: 100_000
}

pub struct QuantizationConfig {
    pub method: QuantizationMethod,
    pub dimensions: Option<usize>,
}
pub enum QuantizationMethod { ProductQuantization { num_subvectors: usize }, ScalarQuantization }
```

### Import/Export Types

```rust
pub struct ImportEnvelope {
    pub envelope_id: String,
    pub schema_version: u32,
    pub namespace: String,
    pub source_authority: String,
    pub content_digest: String,
    pub trace_id: Option<TraceId>,
    pub records: Vec<ImportRecord>,
}

pub enum ImportRecord {
    Fact(Fact),
    Episode(Episode),
}

pub struct ImportReceipt {
    pub envelope_id: String,
    pub records_imported: usize,
    pub imported_at: String,
}

pub struct ImportLogEntry {
    pub envelope_id: String,
    pub schema_version: u32,
    pub content_digest: String,
    pub source_authority: String,
    pub records_imported: usize,
    pub imported_at: String,
}
```

### Embedder

```rust
pub trait Embedder: Send + Sync {
    fn embed(&self, text: &str) -> Result<Vec<f32>, MemoryError>;
    fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>, MemoryError>;
    fn dimensions(&self) -> usize;
}

pub struct MockEmbedder { pub dimensions: usize }
```

### Error

```rust
pub enum MemoryError {
    Database(String),
    Serialization(String),
    InvalidInput(String),
    NotFound(String),
    EmbeddingError(String),
    ImportError(String),
    ImportDuplicate { envelope_id: String },
    HnswError(String),
    QuantizationError(String),
    StorageError(String),
    PoolError(String),
}
```

### Schema (V11) Tables
`documents`, `facts`, `episodes`, `conversations`, `messages`, `relations`, `embeddings`, `hnsw_nodes`, `hnsw_edges`, `hnsw_metadata`, `import_log`, `projection_import_log`, `compaction_log`, `quantization_log`, `document_chunks`, `knowledge_snapshots`, `search_cache`, `graph_cache`, `schema_version`

---

## 2. knowledge-runtime

**Version:** 0.1.0
**Edition:** 2021
**Tests:** ~91

### Dependencies
- semantic-memory (path)
- stack-ids (path)
- serde / serde_json
- tokio (rt, sync)
- thiserror (2)
- tracing (0.1)
- chrono (serde)
- uuid (v4)

### Query Pipeline (5 stages)

```rust
pub struct QueryPipeline {
    pub adapter: Box<dyn MemoryAdapter>,
    pub entity_registry: EntityRegistry,
    pub projection_tracker: ProjectionTracker,
}

impl QueryPipeline {
    pub fn new(adapter: Box<dyn MemoryAdapter>, config: RuntimeConfig) -> Self
    pub async fn query(&self, request: &QueryRequest) -> Result<QueryResponse, RuntimeError>
    pub async fn query_with_trace(&self, request: &QueryRequest) -> Result<(QueryResponse, QueryTrace), RuntimeError>
}
```

**Stages:**
1. Scope resolution — validate namespace/scope, resolve ScopeKey
2. Retrieval — call adapter search with semantic/keyword/hybrid
3. Entity enrichment — link results to entity registry entries
4. Ranking — apply scoring with explicit weights and tie-breaking
5. Assembly — produce final response with provenance metadata

### Adapter Trait

```rust
#[async_trait]
pub trait MemoryAdapter: Send + Sync {
    async fn search(&self, query: &AdapterQuery) -> Result<Vec<AdapterResult>, RuntimeError>;
    async fn search_messages(&self, query: &AdapterMessageQuery) -> Result<Vec<AdapterMessageResult>, RuntimeError>;
    fn capabilities(&self) -> AdapterCapabilities;
    fn adapter_name(&self) -> &str;
}

pub struct AdapterCapabilities {
    pub semantic_search: bool,
    pub keyword_search: bool,
    pub hybrid_search: bool,
    pub message_search: bool,
    pub graph_queries: bool,
    pub recency_boost: bool,
}

pub struct SemanticMemoryAdapter { /* wraps MemoryStore */ }
```

### Entity Registry

```rust
pub struct EntityRegistry { /* in-memory, non-authoritative */ }

impl EntityRegistry {
    pub fn new() -> Self
    pub fn register(&mut self, entity: EntityEntry) -> Result<(), RuntimeError>
    pub fn lookup(&self, entity_id: &str) -> Option<&EntityEntry>
    pub fn lookup_by_alias(&self, alias: &str) -> Option<&EntityEntry>
    pub fn merge(&mut self, source: &str, target: &str, resolution: MergeResolution) -> Result<(), RuntimeError>
    pub fn unmerge(&mut self, entity_id: &str) -> Result<(), RuntimeError>
    pub fn list_all(&self) -> Vec<&EntityEntry>
}

pub struct EntityEntry {
    pub entity_id: String,
    pub canonical_name: String,
    pub aliases: Vec<String>,
    pub entity_type: EntityType,
    pub metadata: serde_json::Value,
    pub merge_history: Vec<MergeRecord>,
}

pub enum EntityType { Person, Organization, Concept, Location, Event, Custom(String) }

pub enum MergeResolution { AutomaticConfident, AutomaticTentative, HumanConfirmedFinal }
```

### Projection Tracker

```rust
pub struct ProjectionTracker { /* tracks staleness of projected data */ }

impl ProjectionTracker {
    pub fn new() -> Self
    pub fn record_import(&mut self, namespace: &str, imported_at: String)
    pub fn check_freshness(&self, namespace: &str) -> ProjectionFreshness
    pub fn last_import(&self, namespace: &str) -> Option<&str>
}

pub enum ProjectionFreshness { Fresh, Stale { last_import: String, age_secs: u64 }, Unknown }
```

### Query/Response Types

```rust
pub struct QueryRequest {
    pub namespace: String,
    pub query_text: String,
    pub query_embedding: Option<Vec<f32>>,
    pub limit: usize,
    pub search_type: SearchType,
    pub scope: Option<ScopeKey>,
    pub recency_weight: Option<f64>,
    pub include_entity_context: bool,
}

pub struct QueryResponse {
    pub results: Vec<RankedResult>,
    pub entity_context: Vec<EntityContext>,
    pub freshness: ProjectionFreshness,
}

pub struct RankedResult {
    pub id: String,
    pub content: String,
    pub score: f64,
    pub rank: usize,
    pub result_type: ResultType,
    pub provenance: ResultProvenance,
    pub entity_links: Vec<String>,
}

pub struct ResultProvenance {
    pub source: String,
    pub retrieved_at: String,
    pub adapter: String,
    pub search_type: String,
}

pub struct QueryTrace {
    pub stages: Vec<TraceStage>,
    pub total_duration_ms: u64,
    pub degradations: Vec<String>,
    pub warnings: Vec<String>,
}
```

### Error

```rust
pub enum RuntimeError {
    AdapterError(String),
    EntityError(String),
    ScopeError(String),
    ProjectionStale { namespace: String, age_secs: u64 },
    UnsupportedCapability { capability: String, adapter: String },
    InvalidQuery(String),
    InternalError(String),
}
```

### Config

```rust
pub struct RuntimeConfig {
    pub default_limit: usize,             // 10
    pub max_limit: usize,                 // 100
    pub staleness_threshold_secs: u64,    // 3600
    pub enable_entity_enrichment: bool,   // true
    pub ranking: RankingConfig,
}

pub struct RankingConfig {
    pub semantic_weight: f64,             // 0.7
    pub recency_weight: f64,             // 0.2
    pub entity_weight: f64,              // 0.1
    pub tie_break: TieBreakPolicy,
}

pub enum TieBreakPolicy { ByRecency, ById, Stable }
```

---

## 3. agent-graph

**Version:** 0.2.0
**Edition:** 2021
**Tests:** 260

### Dependencies
- serde / serde_json
- tokio (rt, sync, macros, time)
- thiserror (2)
- tracing (0.1)
- uuid (v4)
- async-trait

### Graph Definition

```rust
pub struct GraphBuilder<S: State> {
    nodes: HashMap<NodeId, NodeDef<S>>,
    edges: Vec<Edge>,
    entry: Option<NodeId>,
}

impl<S: State> GraphBuilder<S> {
    pub fn new() -> Self
    pub fn add_node<F>(self, id: impl Into<NodeId>, handler: F) -> Self
        where F: Fn(&mut S, &NodeContext) -> Pin<Box<dyn Future<Output = Result<NodeOutcome, GraphError>> + Send>> + Send + Sync + 'static
    pub fn add_edge(self, from: impl Into<NodeId>, to: impl Into<NodeId>) -> Self
    pub fn add_conditional_edge<F>(self, from: impl Into<NodeId>, router: F) -> Self
        where F: Fn(&S) -> Pin<Box<dyn Future<Output = Result<NodeId, GraphError>> + Send>> + Send + Sync + 'static
    pub fn set_entry(self, id: impl Into<NodeId>) -> Self
    pub fn build(self) -> Result<Graph<S>, GraphError>
}

pub struct Graph<S: State> { /* immutable after build */ }

impl<S: State> Graph<S> {
    pub async fn run(&self, state: S) -> Result<S, GraphError>
    pub async fn run_with_config(&self, state: S, config: RunConfig) -> Result<S, GraphError>
    pub async fn run_with_checkpoint(&self, state: S, store: &dyn CheckpointStore) -> Result<S, GraphError>
    pub async fn resume(&self, checkpoint: Checkpoint<S>) -> Result<S, GraphError>
    pub async fn step(&self, state: S) -> Result<StepResult<S>, GraphError>
}
```

### State Trait

```rust
pub trait State: Clone + Send + Sync + Serialize + DeserializeOwned + 'static {}
impl<T: Clone + Send + Sync + Serialize + DeserializeOwned + 'static> State for T {}
```

### Execution Model

```rust
pub enum NodeOutcome {
    Next,                              // proceed along default edge
    Route(NodeId),                     // take named conditional edge
    Halt,                              // stop execution
    Interrupt(InterruptPayload),       // pause for external input
}

pub struct RunConfig {
    pub max_steps: usize,              // default: 1000
    pub max_supersteps: usize,         // default: 100
    pub timeout: Option<Duration>,
    pub event_sink: Option<Box<dyn EventSink>>,
}

pub struct StepResult<S: State> {
    pub state: S,
    pub current_node: NodeId,
    pub outcome: NodeOutcome,
    pub step_count: usize,
}
```

### Checkpointing

```rust
pub struct Checkpoint<S: State> {
    pub checkpoint_id: String,
    pub graph_id: String,
    pub state: S,
    pub current_node: NodeId,
    pub step_count: usize,
    pub created_at: String,
    pub metadata: serde_json::Value,
}

#[async_trait]
pub trait CheckpointStore: Send + Sync {
    async fn save<S: State>(&self, checkpoint: &Checkpoint<S>) -> Result<(), GraphError>;
    async fn load<S: State>(&self, checkpoint_id: &str) -> Result<Option<Checkpoint<S>>, GraphError>;
    async fn list(&self, graph_id: &str) -> Result<Vec<String>, GraphError>;
    async fn delete(&self, checkpoint_id: &str) -> Result<(), GraphError>;
}

pub struct InMemoryCheckpointStore { /* default implementation */ }
```

### Interrupts

```rust
pub struct InterruptPayload {
    pub interrupt_type: String,
    pub data: serde_json::Value,
    pub resumable: bool,
}

pub struct InterruptResume {
    pub response: serde_json::Value,
}
```

### Events

```rust
#[async_trait]
pub trait EventSink: Send + Sync {
    async fn on_node_start(&self, node_id: &NodeId, step: usize);
    async fn on_node_complete(&self, node_id: &NodeId, outcome: &NodeOutcome, step: usize);
    async fn on_graph_complete(&self, total_steps: usize);
    async fn on_error(&self, error: &GraphError);
}
```

### Reducers

```rust
pub trait Reducer<S: State, V>: Send + Sync {
    fn reduce(&self, state: &mut S, value: V);
}

pub struct AppendReducer;  // appends to Vec
pub struct SetReducer;     // replaces value
pub struct MergeReducer;   // merges maps
```

### Superstep BSP

Execution uses a Bulk Synchronous Parallel (BSP) model:
- Nodes execute within supersteps
- Barrier synchronization between supersteps
- `max_supersteps` prevents infinite loops
- Deterministic execution order within a superstep

### Error

```rust
pub enum GraphError {
    NodeNotFound(String),
    EdgeNotFound { from: String, to: String },
    NoEntryPoint,
    CycleDetected(Vec<String>),
    MaxStepsExceeded(usize),
    MaxSuperstepsExceeded(usize),
    Timeout,
    CheckpointError(String),
    InterruptError(String),
    NodeError { node_id: String, source: Box<dyn std::error::Error + Send + Sync> },
    BuildError(String),
    SerializationError(String),
}
```

### Prelude

```rust
pub mod prelude {
    pub use crate::{
        Graph, GraphBuilder, GraphError, NodeOutcome, RunConfig, StepResult,
        State, NodeId, NodeContext, Checkpoint, CheckpointStore,
        InMemoryCheckpointStore, EventSink, InterruptPayload, InterruptResume,
        Reducer, AppendReducer, SetReducer, MergeReducer,
    };
}
```

---

## 4. LLM-Pipeline

**Version:** 0.2.0
**Edition:** 2021
**Tests:** 168

### Dependencies
- serde / serde_json
- tokio (rt, sync, macros, time)
- reqwest (json, stream)
- thiserror (2)
- tracing (0.1)
- uuid (v4)
- async-trait
- futures (0.3)
- bytes

### Payload Trait

```rust
pub trait Payload: Clone + Send + Sync + 'static {
    fn system_prompt(&self) -> Option<&str>;
    fn user_prompt(&self) -> &str;
    fn model(&self) -> &str;
    fn temperature(&self) -> Option<f64>;
    fn max_tokens(&self) -> Option<u32>;
    fn metadata(&self) -> Option<&serde_json::Value> { None }
}
```

### Pipeline

```rust
pub struct Pipeline<P: Payload> {
    backend: Box<dyn Backend<P>>,
    retry_policy: RetryPolicy,
    rate_limiter: Option<RateLimiter>,
    trace_config: TraceConfig,
}

impl<P: Payload> Pipeline<P> {
    pub fn new(backend: Box<dyn Backend<P>>) -> Self
    pub fn with_retry(self, policy: RetryPolicy) -> Self
    pub fn with_rate_limit(self, limiter: RateLimiter) -> Self
    pub fn with_trace(self, config: TraceConfig) -> Self
    pub async fn call(&self, payload: &P) -> Result<LlmResponse, PipelineError>
    pub async fn call_with_trace(&self, payload: &P) -> Result<(LlmResponse, CallTrace), PipelineError>
    pub async fn stream(&self, payload: &P) -> Result<Pin<Box<dyn Stream<Item = Result<StreamChunk, PipelineError>> + Send>>, PipelineError>
}
```

### Backend Trait

```rust
#[async_trait]
pub trait Backend<P: Payload>: Send + Sync {
    async fn call(&self, payload: &P, ctx: &ExecCtx) -> Result<LlmResponse, PipelineError>;
    async fn stream(&self, payload: &P, ctx: &ExecCtx) -> Result<Pin<Box<dyn Stream<Item = Result<StreamChunk, PipelineError>> + Send>>, PipelineError>;
    fn name(&self) -> &str;
}
```

### Backend Implementations

```rust
// Ollama (local)
pub struct OllamaBackend {
    pub base_url: String,
    pub client: reqwest::Client,
}
impl OllamaBackend {
    pub fn new(base_url: impl Into<String>) -> Self
    pub fn default_local() -> Self  // http://localhost:11434
}

// OpenAI-compatible
pub struct OpenAiBackend {
    pub base_url: String,
    pub api_key: String,
    pub client: reqwest::Client,
}
impl OpenAiBackend {
    pub fn new(base_url: impl Into<String>, api_key: impl Into<String>) -> Self
    pub fn from_env() -> Result<Self, PipelineError>  // OPENAI_API_KEY + OPENAI_BASE_URL
}

// Mock (testing)
pub struct MockBackend {
    pub responses: Arc<Mutex<VecDeque<Result<LlmResponse, PipelineError>>>>,
}
impl MockBackend {
    pub fn new() -> Self
    pub fn push_response(&self, response: LlmResponse)
    pub fn push_error(&self, error: PipelineError)
}

// Recording (captures requests/responses)
pub struct RecordingBackend<P: Payload> {
    pub inner: Box<dyn Backend<P>>,
    pub recordings: Arc<Mutex<Vec<Recording>>>,
}
```

### Retry Policy

```rust
pub struct RetryPolicy {
    pub max_retries: u32,                 // default: 3
    pub initial_backoff_ms: u64,          // default: 1000
    pub max_backoff_ms: u64,              // default: 30000
    pub backoff_multiplier: f64,          // default: 2.0
    pub retry_on: RetryCondition,
}

pub enum RetryCondition {
    TransportOnly,                        // network errors only
    TransportAndServerErrors,             // + 5xx
    TransportAndRateLimits,               // + 429
    All,                                  // all retriable errors
    Custom(Box<dyn Fn(&PipelineError) -> bool + Send + Sync>),
}
```

### Rate Limiting

```rust
pub struct RateLimiter {
    pub requests_per_second: f64,
    pub burst_size: u32,
}
```

### Tracing

```rust
pub struct TraceConfig {
    pub capture_prompts: bool,            // default: false (privacy)
    pub capture_responses: bool,          // default: false
    pub capture_timing: bool,             // default: true
}

pub struct CallTrace {
    pub trace_id: String,
    pub attempt_count: u32,
    pub total_duration_ms: u64,
    pub attempts: Vec<AttemptTrace>,
}

pub struct AttemptTrace {
    pub attempt: u32,
    pub duration_ms: u64,
    pub result: AttemptResult,
    pub backend: String,
}

pub enum AttemptResult { Success, Retried { reason: String }, Failed { error: String } }
```

### Response Types

```rust
pub struct LlmResponse {
    pub content: String,
    pub model: String,
    pub usage: Option<TokenUsage>,
    pub finish_reason: Option<String>,
    pub response_metadata: serde_json::Value,
}

pub struct TokenUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

pub struct StreamChunk {
    pub content: String,
    pub done: bool,
    pub metadata: Option<serde_json::Value>,
}
```

### Output Parsing (re-exports from llm-output-parser)

```rust
pub use llm_output_parser::{
    parse_json, parse_json_value, parse_json_with_trace, parse_json_value_with_trace,
    parse_xml_tag, parse_xml_tags, parse_xml_tag_with_trace, parse_xml_tags_with_trace,
    parse_text, parse_text_with_trace,
    parse_choice, parse_choice_with_trace,
    parse_number, parse_number_with_trace, parse_number_in_range, parse_number_in_range_with_trace,
    parse_string_list, parse_string_list_raw, parse_string_list_with_trace,
    strip_think_tags, try_repair_json,
    ParseError, ParseOptions, ParseTrace,
};
```

### Error

```rust
pub enum PipelineError {
    BackendError(String),
    TransportError(String),
    RateLimited { retry_after_ms: Option<u64> },
    ServerError { status: u16, body: String },
    Timeout,
    SerializationError(String),
    ConfigError(String),
    StreamError(String),
    ParseError(String),
    MaxRetriesExceeded { attempts: u32, last_error: Box<PipelineError> },
}
```

### Diagnostics

```rust
pub struct PipelineDiagnostics {
    pub total_calls: u64,
    pub total_retries: u64,
    pub total_errors: u64,
    pub avg_latency_ms: f64,
    pub p99_latency_ms: f64,
    pub calls_by_model: HashMap<String, u64>,
}
```

---

## 5. job-queue

**Version:** 0.2.0
**Edition:** 2021
**Tests:** 35

### Dependencies
- rusqlite (0.32, bundled)
- serde / serde_json
- tokio (rt, sync, macros, time)
- thiserror (2)
- tracing (0.1)
- uuid (v4)
- chrono (serde)

### Queue

```rust
pub struct JobQueue {
    db: Arc<Mutex<Connection>>,
    config: QueueConfig,
}

impl JobQueue {
    pub fn open(path: &Path) -> Result<Self, QueueError>
    pub fn open_with_config(path: &Path, config: QueueConfig) -> Result<Self, QueueError>
    pub fn in_memory() -> Result<Self, QueueError>
    pub fn in_memory_with_config(config: QueueConfig) -> Result<Self, QueueError>

    // Enqueue
    pub fn enqueue(&self, job: &JobSpec) -> Result<JobId, QueueError>
    pub fn enqueue_with_priority(&self, job: &JobSpec, priority: i32) -> Result<JobId, QueueError>

    // Dequeue / Lease
    pub fn dequeue(&self) -> Result<Option<Job>, QueueError>
    pub fn dequeue_batch(&self, max: usize) -> Result<Vec<Job>, QueueError>
    pub fn lease(&self, job_id: &JobId, duration: Duration) -> Result<bool, QueueError>
    pub fn renew_lease(&self, job_id: &JobId, duration: Duration) -> Result<bool, QueueError>
    pub fn heartbeat(&self, job_id: &JobId) -> Result<bool, QueueError>

    // Completion
    pub fn complete(&self, job_id: &JobId, result: &serde_json::Value) -> Result<(), QueueError>
    pub fn fail(&self, job_id: &JobId, error: &str, class: FailureClass) -> Result<(), QueueError>
    pub fn retry(&self, job_id: &JobId) -> Result<bool, QueueError>
    pub fn cancel(&self, job_id: &JobId) -> Result<bool, QueueError>

    // Query
    pub fn get_job(&self, job_id: &JobId) -> Result<Option<Job>, QueueError>
    pub fn list_jobs(&self, filter: &JobFilter) -> Result<Vec<Job>, QueueError>
    pub fn count_by_status(&self) -> Result<HashMap<JobStatus, usize>, QueueError>
    pub fn dead_letter_jobs(&self) -> Result<Vec<Job>, QueueError>

    // Maintenance
    pub fn expire_leases(&self) -> Result<usize, QueueError>
    pub fn reap_dead_jobs(&self, max_age: Duration) -> Result<usize, QueueError>
    pub fn compact(&self) -> Result<(), QueueError>
}
```

### Types

```rust
pub struct JobSpec {
    pub job_type: String,
    pub payload: serde_json::Value,
    pub priority: Option<i32>,
    pub max_retries: Option<u32>,
    pub retry_backoff_ms: Option<u64>,
    pub timeout_secs: Option<u64>,
    pub metadata: Option<serde_json::Value>,
}

pub struct Job {
    pub id: JobId,
    pub job_type: String,
    pub payload: serde_json::Value,
    pub status: JobStatus,
    pub priority: i32,
    pub attempt: u32,
    pub max_retries: u32,
    pub result: Option<serde_json::Value>,
    pub error: Option<String>,
    pub failure_class: Option<FailureClass>,
    pub created_at: String,
    pub updated_at: String,
    pub lease_expires_at: Option<String>,
    pub completed_at: Option<String>,
    pub metadata: serde_json::Value,
}

pub enum JobStatus { Pending, Running, Completed, Failed, DeadLetter, Cancelled }

pub enum FailureClass { Transient, Permanent, Unknown }

pub struct JobFilter {
    pub status: Option<JobStatus>,
    pub job_type: Option<String>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

pub struct QueueConfig {
    pub default_max_retries: u32,          // 3
    pub default_lease_duration_secs: u64,  // 300
    pub default_retry_backoff_ms: u64,     // 1000
    pub heartbeat_interval_secs: u64,      // 30
    pub dead_letter_max_age_secs: u64,     // 86400
}
```

### State Machine

```
Pending -> Running (dequeue)
Running -> Completed (complete)
Running -> Failed (fail with Transient, attempt < max)
Running -> DeadLetter (fail with Permanent, or max retries exceeded)
Running -> Pending (retry / lease expiry)
Failed -> Pending (retry)
Any -> Cancelled (cancel)
```

### Error

```rust
pub enum QueueError {
    Database(String),
    Serialization(String),
    InvalidState { job_id: String, current: JobStatus, attempted: String },
    NotFound(String),
    LeaseExpired(String),
    ConfigError(String),
}
```

### Events

```rust
pub enum QueueEvent {
    JobEnqueued { job_id: JobId, job_type: String },
    JobStarted { job_id: JobId },
    JobCompleted { job_id: JobId },
    JobFailed { job_id: JobId, error: String, failure_class: FailureClass },
    JobRetried { job_id: JobId, attempt: u32 },
    JobDeadLettered { job_id: JobId },
    JobCancelled { job_id: JobId },
    LeaseExpired { job_id: JobId },
}

pub trait QueueEventListener: Send + Sync {
    fn on_event(&self, event: &QueueEvent);
}
```

### Executor

```rust
pub struct JobExecutor {
    queue: Arc<JobQueue>,
    handlers: HashMap<String, Box<dyn JobHandler>>,
    config: ExecutorConfig,
}

#[async_trait]
pub trait JobHandler: Send + Sync {
    async fn handle(&self, job: &Job) -> Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>>;
}

pub struct ExecutorConfig {
    pub poll_interval_ms: u64,             // 1000
    pub max_concurrent: usize,             // 4
    pub shutdown_timeout_secs: u64,        // 30
}
```

---

## 6. AI-Batch-Queue

**Version:** 0.2.0
**Edition:** 2021
**Tests:** 63

### Dependencies
- job-queue (path)
- serde / serde_json
- tokio (rt, sync, macros, time)
- thiserror (2)
- tracing (0.1)
- uuid (v4)
- chrono (serde)

### Batch Queue

```rust
pub struct AiBatchQueue {
    queue: JobQueue,
    config: BatchConfig,
    eta_estimator: EtaEstimator,
    event_emitter: Option<Box<dyn BatchEventEmitter>>,
}

impl AiBatchQueue {
    pub fn new(queue: JobQueue, config: BatchConfig) -> Self
    pub fn with_event_emitter(self, emitter: Box<dyn BatchEventEmitter>) -> Self

    // Batch Operations
    pub fn submit_batch(&self, batch: &BatchSpec) -> Result<BatchId, BatchError>
    pub fn submit_item(&self, batch_id: &BatchId, item: &BatchItem) -> Result<ItemId, BatchError>
    pub fn get_batch_status(&self, batch_id: &BatchId) -> Result<BatchStatus, BatchError>
    pub fn get_item_status(&self, item_id: &ItemId) -> Result<ItemStatus, BatchError>
    pub fn cancel_batch(&self, batch_id: &BatchId) -> Result<(), BatchError>
    pub fn list_batches(&self) -> Result<Vec<BatchSummary>, BatchError>

    // ETA
    pub fn estimate_eta(&self, batch_id: &BatchId) -> Result<EtaEstimate, BatchError>
    pub fn record_completion(&self, item_id: &ItemId, duration_ms: u64) -> Result<(), BatchError>
}
```

### Types

```rust
pub struct BatchSpec {
    pub name: String,
    pub model: String,
    pub items: Vec<BatchItem>,
    pub priority: Option<i32>,
    pub metadata: Option<serde_json::Value>,
}

pub struct BatchItem {
    pub prompt: String,
    pub system_prompt: Option<String>,
    pub parameters: Option<serde_json::Value>,
    pub metadata: Option<serde_json::Value>,
}

pub struct BatchStatus {
    pub batch_id: BatchId,
    pub name: String,
    pub total_items: usize,
    pub completed: usize,
    pub failed: usize,
    pub pending: usize,
    pub running: usize,
    pub progress_pct: f64,
    pub eta: Option<EtaEstimate>,
    pub created_at: String,
    pub status: BatchState,
}

pub enum BatchState { Pending, Running, Completed, Failed, Cancelled }

pub struct ItemStatus {
    pub item_id: ItemId,
    pub batch_id: BatchId,
    pub status: ItemState,
    pub result: Option<serde_json::Value>,
    pub error: Option<String>,
    pub attempt: u32,
}

pub enum ItemState { Pending, Running, Completed, Failed }

pub struct BatchConfig {
    pub max_concurrent_items: usize,       // 4
    pub default_model: String,
    pub max_retries_per_item: u32,         // 3
    pub item_timeout_secs: u64,            // 300
    pub model_rate_limits: HashMap<String, ModelRateLimit>,
}

pub struct ModelRateLimit {
    pub requests_per_minute: u32,
    pub tokens_per_minute: Option<u32>,
}
```

### ETA Estimation

```rust
pub struct EtaEstimator { /* model-aware estimation */ }

impl EtaEstimator {
    pub fn new() -> Self
    pub fn record(&mut self, model: &str, duration_ms: u64)
    pub fn estimate(&self, model: &str, remaining: usize) -> EtaEstimate
}

pub struct EtaEstimate {
    pub estimated_secs: f64,
    pub confidence: EtaConfidence,
    pub samples: usize,
}

pub enum EtaConfidence { High, Medium, Low, NoData }
```

### Events

```rust
pub trait BatchEventEmitter: Send + Sync {
    fn emit(&self, event: BatchEvent);
}

pub enum BatchEvent {
    BatchCreated { batch_id: BatchId, name: String },
    BatchStarted { batch_id: BatchId },
    BatchCompleted { batch_id: BatchId },
    BatchFailed { batch_id: BatchId, error: String },
    BatchCancelled { batch_id: BatchId },
    ItemStarted { batch_id: BatchId, item_id: ItemId },
    ItemCompleted { batch_id: BatchId, item_id: ItemId },
    ItemFailed { batch_id: BatchId, item_id: ItemId, error: String },
    ProgressUpdated { batch_id: BatchId, progress_pct: f64, eta: Option<EtaEstimate> },
}
```

### Executor

```rust
pub struct BatchExecutor {
    queue: Arc<AiBatchQueue>,
    handler: Box<dyn BatchItemHandler>,
    config: ExecutorConfig,
}

#[async_trait]
pub trait BatchItemHandler: Send + Sync {
    async fn process(&self, item: &BatchItem, model: &str) -> Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>>;
}
```

### Error

```rust
pub enum BatchError {
    QueueError(QueueError),
    InvalidBatch(String),
    BatchNotFound(String),
    ItemNotFound(String),
    RateLimited { model: String, retry_after_ms: u64 },
    ConfigError(String),
}
```

---

## 7. Tauri-Queue

**Version:** 0.3.0
**Edition:** 2021
**Tests:** 23

### Dependencies
- job-queue (path)
- serde / serde_json
- tokio (rt, sync)
- thiserror (2)
- tracing (0.1)

### Core Types

```rust
pub struct TauriQueuePlugin {
    queue: Arc<JobQueue>,
    emitter: CoalescingEmitter,
}

impl TauriQueuePlugin {
    pub fn new(queue: Arc<JobQueue>) -> Self
    pub fn with_emitter(queue: Arc<JobQueue>, config: EmitterConfig) -> Self
    pub fn queue(&self) -> &JobQueue
}
```

### Coalescing Emitter

```rust
pub struct CoalescingEmitter {
    inner: Arc<dyn TauriEventEmitter>,
    config: EmitterConfig,
    pending: Arc<Mutex<HashMap<String, serde_json::Value>>>,
}

impl CoalescingEmitter {
    pub fn new(inner: Arc<dyn TauriEventEmitter>, config: EmitterConfig) -> Self
    pub fn emit(&self, event_name: &str, payload: serde_json::Value)
    pub fn flush(&self)
    pub fn start_flush_loop(&self)
}

pub struct EmitterConfig {
    pub coalesce_interval_ms: u64,         // 100
    pub max_pending: usize,                // 1000
}

pub trait TauriEventEmitter: Send + Sync {
    fn emit(&self, event: &str, payload: &serde_json::Value) -> Result<(), String>;
}
```

### Re-exports from job-queue

```rust
pub use job_queue::{
    Job, JobId, JobSpec, JobStatus, JobFilter, JobHandler,
    QueueConfig, QueueError, QueueEvent, FailureClass,
};
```

### Tauri Commands (for #[tauri::command])

```rust
pub fn enqueue_job(queue: &TauriQueuePlugin, spec: JobSpec) -> Result<JobId, String>
pub fn get_job_status(queue: &TauriQueuePlugin, id: JobId) -> Result<Option<Job>, String>
pub fn list_jobs(queue: &TauriQueuePlugin, filter: JobFilter) -> Result<Vec<Job>, String>
pub fn cancel_job(queue: &TauriQueuePlugin, id: JobId) -> Result<bool, String>
pub fn queue_stats(queue: &TauriQueuePlugin) -> Result<HashMap<JobStatus, usize>, String>
```

---

## 8. ComfyUI-RS

**Version:** 0.2.0
**Edition:** 2021
**Tests:** 30

### Dependencies
- reqwest (json, multipart)
- tokio (rt, sync)
- serde / serde_json
- thiserror (2)
- tracing (0.1)
- uuid (v4)
- tokio-tungstenite (connect-async)
- futures-util (sink)
- base64

### Client

```rust
pub struct ComfyClient {
    pub base_url: String,
    pub client_id: String,
    http: reqwest::Client,
}

impl ComfyClient {
    pub fn new(base_url: impl Into<String>) -> Self
    pub fn with_client_id(base_url: impl Into<String>, client_id: impl Into<String>) -> Self

    // Workflow
    pub async fn queue_prompt(&self, workflow: &serde_json::Value) -> Result<PromptResponse, ComfyError>
    pub async fn get_history(&self, prompt_id: &str) -> Result<serde_json::Value, ComfyError>
    pub async fn get_queue(&self) -> Result<QueueStatus, ComfyError>
    pub async fn interrupt(&self) -> Result<(), ComfyError>

    // Images
    pub async fn get_image(&self, filename: &str, subfolder: &str, folder_type: &str) -> Result<Vec<u8>, ComfyError>
    pub async fn upload_image(&self, filename: &str, data: &[u8], overwrite: bool) -> Result<UploadResponse, ComfyError>

    // System
    pub async fn get_system_stats(&self) -> Result<SystemStats, ComfyError>
    pub async fn get_object_info(&self) -> Result<serde_json::Value, ComfyError>

    // WebSocket
    pub async fn connect_ws(&self) -> Result<ComfyWsStream, ComfyError>
}
```

### Types

```rust
pub struct PromptResponse {
    pub prompt_id: String,
    pub number: u32,
    pub node_errors: serde_json::Value,
}

pub struct QueueStatus {
    pub queue_running: Vec<serde_json::Value>,
    pub queue_pending: Vec<serde_json::Value>,
}

pub struct UploadResponse {
    pub name: String,
    pub subfolder: String,
    pub type_field: String,
}

pub struct SystemStats {
    pub system: SystemInfo,
    pub devices: Vec<DeviceInfo>,
}
pub struct SystemInfo { pub os: String, pub python_version: String, pub embedded_python: bool }
pub struct DeviceInfo { pub name: String, pub type_field: String, pub vram_total: u64, pub vram_free: u64, pub torch_vram_total: u64, pub torch_vram_free: u64 }
```

### Txt2Img Builder

```rust
pub struct Txt2ImgRequest {
    pub prompt: String,
    pub negative_prompt: Option<String>,
    pub model: Option<String>,
    pub width: u32,
    pub height: u32,
    pub steps: u32,
    pub cfg_scale: f64,
    pub seed: Option<i64>,
    pub sampler: String,
    pub scheduler: String,
    pub batch_size: u32,
}

impl Txt2ImgRequest {
    pub fn new(prompt: impl Into<String>) -> Self
    pub fn negative_prompt(self, v: impl Into<String>) -> Self
    pub fn model(self, v: impl Into<String>) -> Self
    pub fn width(self, v: u32) -> Self
    pub fn height(self, v: u32) -> Self
    pub fn steps(self, v: u32) -> Self
    pub fn cfg_scale(self, v: f64) -> Self
    pub fn seed(self, v: i64) -> Self
    pub fn sampler(self, v: impl Into<String>) -> Self
    pub fn scheduler(self, v: impl Into<String>) -> Self
    pub fn batch_size(self, v: u32) -> Self
    pub fn to_workflow(&self) -> serde_json::Value
}
```

### WebSocket

```rust
pub struct ComfyWsStream { /* wraps tokio-tungstenite */ }

impl ComfyWsStream {
    pub async fn next_message(&mut self) -> Result<Option<WsMessage>, ComfyError>
    pub async fn close(self) -> Result<(), ComfyError>
}

pub enum WsMessage {
    Status(serde_json::Value),
    Progress { value: u32, max: u32 },
    Executing { node: Option<String> },
    Executed { node: String, output: serde_json::Value },
    ExecutionError { node: String, error: String },
    Other(serde_json::Value),
}
```

### Error

```rust
pub enum ComfyError {
    HttpError(String),
    WebSocketError(String),
    SerializationError(String),
    ConnectionError(String),
    ApiError { status: u16, message: String },
    Timeout,
}
```

---

## 9. Ollama-Vision-RS

**Version:** 0.2.0
**Edition:** 2021
**Tests:** 6

### Dependencies
- reqwest (json)
- serde / serde_json
- tokio (rt)
- thiserror (2)
- base64
- llm-output-parser (path to .parser-lib)

### Public Functions

```rust
/// Tag an image with descriptive labels.
pub async fn tag_image(config: &VisionConfig, image_path: &Path) -> Result<Vec<String>, VisionError>

/// Generate a caption/description for an image.
pub async fn caption_image(config: &VisionConfig, image_path: &Path) -> Result<String, VisionError>
```

### Types

```rust
pub struct VisionConfig {
    pub base_url: String,                 // default: "http://localhost:11434"
    pub model: String,                    // default: "llava"
    pub tag_prompt: Option<String>,       // custom tagging prompt
    pub caption_prompt: Option<String>,   // custom caption prompt
    pub temperature: Option<f64>,
    pub max_tokens: Option<u32>,
}

impl VisionConfig {
    pub fn new() -> Self
    pub fn with_model(self, model: impl Into<String>) -> Self
    pub fn with_base_url(self, url: impl Into<String>) -> Self
    pub fn with_tag_prompt(self, prompt: impl Into<String>) -> Self
    pub fn with_caption_prompt(self, prompt: impl Into<String>) -> Self
}
```

### Parser Integration

Uses `llm_output_parser::parse_string_list()` for tag extraction and `llm_output_parser::parse_text()` for caption cleaning.

### Error

```rust
pub enum VisionError {
    HttpError(String),
    IoError(String),
    ParseError(String),
    ImageError(String),
    ConfigError(String),
}
```

---

## 10. stack-ids

**Version:** 0.1.0
**Edition:** 2021
**Tests:** 40

### Dependencies
- serde (derive)
- serde_json
- uuid (v4, serde)
- blake3
- thiserror (2)

### ID Newtypes

```rust
macro_rules! define_id { ... }

define_id!(EnvelopeId);
define_id!(ClaimId);
define_id!(ClaimVersionId);
define_id!(RelationId);
define_id!(RelationVersionId);
define_id!(EntityId);
define_id!(ProjectionId);
define_id!(ImportBatchId);

// Each type:
pub struct XxxId(String);
impl XxxId {
    pub fn new(id: impl Into<String>) -> Self
    pub fn generate() -> Self              // UUID v4
    pub fn as_str(&self) -> &str
}
// Implements: Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Display, From<String>, AsRef<str>
```

### ScopeKey

```rust
pub struct ScopeKey {
    pub namespace: String,
    pub domain: Option<String>,
    pub workspace_id: Option<String>,
    pub repo_id: Option<String>,
}

impl ScopeKey {
    pub fn new(namespace: impl Into<String>) -> Self
    pub fn from_legacy_namespace(ns: impl Into<String>) -> Self
    pub fn to_legacy_namespace(&self) -> &str
    pub fn is_namespace_only(&self) -> bool
    pub fn with_domain(self, domain: impl Into<String>) -> Self
    pub fn with_workspace(self, id: impl Into<String>) -> Self
    pub fn with_repo(self, id: impl Into<String>) -> Self
    pub fn canonical_string(&self) -> String
}
```

### TraceCtx

```rust
pub struct TraceCtx {
    pub trace_id: String,
    pub parent_id: Option<String>,
    pub baggage: Vec<BaggageEntry>,
}
pub struct BaggageEntry { pub key: String, pub value: String }

pub const MAX_BAGGAGE_ENTRIES: usize = 16;
pub const MAX_BAGGAGE_ITEM_BYTES: usize = 256;

impl TraceCtx {
    pub fn generate() -> Self                        // UUID v4 hex
    pub fn from_trace_id(id: impl Into<String>) -> Self
    pub fn from_legacy_trace_id(id: impl Into<String>) -> Self   // compatibility / migration-only
    pub fn to_legacy_trace_id(&self) -> &str                     // compatibility / migration-only
    pub fn with_parent(self, parent_id: impl Into<String>) -> Self
    pub fn child(&self, span_id: impl Into<String>) -> Self
    pub fn add_baggage(&mut self, key: impl Into<String>, value: impl Into<String>) -> Result<(), TraceError>
    pub fn baggage_value(&self, key: &str) -> Option<&str>
    pub fn to_traceparent(&self) -> Result<String, TraceError>   // W3C format
    pub fn from_traceparent(header: &str) -> Result<Self, TraceError>
}

/// Convert non-W3C trace ID via deterministic BLAKE3 hash truncation.
pub fn hash_to_w3c_trace_id(legacy_id: &str) -> String

pub enum TraceError {
    BaggageLimitExceeded { max: usize },
    BaggageItemTooLarge { field: String, len: usize, max: usize },
    InvalidTraceparent { reason: String },
}
```

### ContentDigest

```rust
pub struct ContentDigest {
    pub algorithm: DigestAlgorithm,
    pub hash: String,
    pub fields_in_domain: Vec<String>,
}

pub enum DigestAlgorithm { Blake3 }

impl ContentDigest {
    pub fn compute_blake3(canonical_json: &str, fields: Vec<String>) -> Self
    pub fn verify(&self, canonical_json: &str) -> bool
    pub fn algorithm_name(&self) -> &str
}
```

---

## 11. forge-memory-bridge

**Version:** 0.1.0
**Edition:** 2021
**Tests:** 22

### Dependencies
- stack-ids (path)
- serde / serde_json
- thiserror (2)
- uuid (v4)
- chrono (serde)

### Export Envelope

```rust
pub struct ExportEnvelopeV1 {
    pub envelope_id: EnvelopeId,
    pub schema_version: u32,                    // always 1
    pub scope: ScopeKey,
    pub source_authority: String,
    pub content_digest: ContentDigest,
    pub trace: TraceCtx,
    pub records: Vec<ExportRecord>,
    pub created_at: String,
}

pub enum ExportRecord {
    Claim {
        claim_id: ClaimId,
        claim_version_id: ClaimVersionId,
        subject_entity_id: String,
        predicate: String,
        object_value: serde_json::Value,
        confidence: f64,
        valid_from: Option<String>,
        valid_to: Option<String>,
        projection_family: String,
        source_authority: String,
        evidence_refs: Vec<EvidenceRef>,
        metadata: serde_json::Value,
    },
    Relation {
        relation_id: RelationId,
        relation_version_id: RelationVersionId,
        source_entity_id: String,
        relation_type: String,
        target_entity_id: String,
        weight: f64,
        source_authority: String,
        valid_from: Option<String>,
        valid_to: Option<String>,
        metadata: serde_json::Value,
    },
    Episode {
        session_id: Option<String>,
        content: String,
        metadata: serde_json::Value,
    },
}

pub struct EvidenceRef {
    pub claim_id: ClaimId,
    pub source_authority: String,
    pub envelope_provenance: String,
    pub raw_evidence_handle: String,
    pub audit_dereference_path: String,
    pub version_linkage: Option<String>,
}
```

### Projection Import Batch

```rust
pub struct ProjectionImportBatchV1 {
    pub batch_id: ImportBatchId,
    pub envelope_id: EnvelopeId,
    pub scope: ScopeKey,
    pub source_authority: String,
    pub content_digest: ContentDigest,
    pub trace: TraceCtx,
    pub rows: Vec<ProjectionRow>,
    pub created_at: String,
}

pub enum ProjectionRow {
    Claim {
        claim_id: ClaimId,
        claim_version_id: ClaimVersionId,
        subject_entity_id: String,
        predicate: String,
        object_value: serde_json::Value,
        confidence: f64,
        valid_from: Option<String>,
        valid_to: Option<String>,
        projection_family: String,
        source_authority: String,
        evidence_refs: Vec<EvidenceRef>,
        embedding: Option<Vec<f32>>,
        metadata: serde_json::Value,
    },
    Relation {
        relation_id: RelationId,
        relation_version_id: RelationVersionId,
        source_entity_id: String,
        relation_type: String,
        target_entity_id: String,
        weight: f64,
        source_authority: String,
        valid_from: Option<String>,
        valid_to: Option<String>,
        metadata: serde_json::Value,
    },
    Episode {
        session_id: Option<String>,
        content: String,
        embedding: Option<Vec<f32>>,
        metadata: serde_json::Value,
    },
}
```

### Transform Pipeline

```rust
pub fn transform_envelope(envelope: &ExportEnvelopeV1) -> Result<ProjectionImportBatchV1, BridgeError>
pub fn validate_envelope(envelope: &ExportEnvelopeV1) -> Result<(), BridgeError>
```

### Legacy Compatibility

```rust
// Phase status: compatibility / migration-only
pub struct LegacyImportEnvelopeV1 {
    pub envelope_id: String,
    pub schema_version: u32,
    pub namespace: String,
    pub source_authority: String,
    pub content_digest: String,
    pub trace_id: Option<String>,
    pub records: Vec<LegacyImportRecord>,
}

pub enum LegacyImportRecord {
    Fact { subject: String, predicate: String, object: String, confidence: f64, source: Option<String>, metadata: serde_json::Value },
    Episode { session_id: Option<String>, content: String, metadata: serde_json::Value },
}

pub fn upgrade_legacy_envelope(legacy: &LegacyImportEnvelopeV1) -> Result<ExportEnvelopeV1, BridgeError>
pub fn transform_legacy_envelope(legacy: &LegacyImportEnvelopeV1) -> Result<ProjectionImportBatchV1, BridgeError>
```

### Error

```rust
pub enum BridgeError {
    ValidationError(String),
    TransformError(String),
    DigestMismatch { expected: String, actual: String },
    InvalidEnvelope(String),
    SerializationError(String),
}
```

---

## 12. Tauri-React-Hooks

**Package:** @tauri-hooks/core
**Version:** 0.1.0
**Language:** TypeScript (ES2020, React 18+, Tauri 2+)

### Hooks

```typescript
// Single event listener
function useTauriEvent<T = unknown>(
    event: string,
    handler: TauriEventHandler<T>,
    deps?: DependencyList,
): void

// Multiple event listeners
function useTauriEvents(
    bindings: EventBindings,
    deps?: DependencyList,
): void

// Auto-fetching query
function useTauriQuery<T>(
    command: string,
    args?: Record<string, unknown>,
    options?: TauriQueryOptions,
    deps?: DependencyList,
): TauriQueryState<T>

// Manual mutation
function useTauriMutation<TArgs extends unknown[] = [], TResult = void>(
    command: string,
    argsFn?: (...args: TArgs) => Record<string, unknown>,
    options?: TauriMutationOptions<TResult>,
): TauriMutationState<TArgs, TResult>

// Config load/save/update
function useTauriConfig<T extends Record<string, unknown>>(
    loadCmd: string,
    saveCmd: string,
    saveArgName?: string,
): TauriConfigState<T>

// High-frequency buffered stream (~30fps flush)
function useBufferedStream<K extends string = string>(
    options?: BufferedStreamOptions,
): BufferedStreamState<K>
```

### Types

```typescript
type TauriEventHandler<T> = (payload: T) => void
type EventBindings = Record<string, TauriEventHandler<any>>

interface TauriQueryOptions {
    enabled?: boolean                   // default: true
    refreshOn?: string[]                // event names triggering refresh
}
interface TauriQueryState<T> {
    data: T | null
    loading: boolean
    error: string | null
    refresh: () => Promise<void>
}

interface TauriMutationOptions<TResult> {
    onSuccess?: (result: TResult) => void
    onError?: (error: string) => void
}
interface TauriMutationState<TArgs extends unknown[], TResult> {
    mutate: (...args: TArgs) => Promise<TResult>
    loading: boolean
    error: string | null
    reset: () => void
}

interface TauriConfigState<T> {
    config: T | null
    loading: boolean
    error: string | null
    saving: boolean
    save: (updated: T) => Promise<boolean>
    update: (partial: Partial<T>) => void    // optimistic local merge
    reload: () => Promise<void>
}

interface BufferedStreamOptions { interval?: number }  // default: 33ms
interface BufferedStreamState<K extends string = string> {
    buffers: Record<K, string>
    push: (key: K, data: string) => void
    start: () => void
    stop: () => void
    clear: (key?: K) => void
}
```

### Key Patterns
- Async-safe cleanup with `cancelled` flag
- Handler freshness via `useRef` (no re-subscription on handler identity change)
- Stable args serialization via `JSON.stringify`
- Two-layer buffering: sync writes to `pendingRef`, periodic flushes to React state

---

## 13. llm-output-parser

**Crate:** llm-output-parser
**Version:** 0.2.0
**Edition:** 2021
**Tests:** 100+
**Features:** `yaml` (optional)

### Dependencies
- serde (derive)
- serde_json
- thiserror (2)
- serde_yaml (0.9, optional behind `yaml` feature)

### Public Functions

```rust
// JSON
pub fn parse_json<T: DeserializeOwned>(response: &str) -> Result<T, ParseError>
pub fn parse_json_value(response: &str) -> Result<serde_json::Value, ParseError>
pub fn parse_json_with_trace<T: DeserializeOwned>(response: &str, opts: &ParseOptions) -> Result<(T, ParseTrace), ParseError>
pub fn parse_json_value_with_trace(response: &str, opts: &ParseOptions) -> Result<(serde_json::Value, ParseTrace), ParseError>

// XML tags
pub fn parse_xml_tag(response: &str, tag: &str) -> Result<String, ParseError>
pub fn parse_xml_tag_with_trace(response: &str, tag: &str, opts: &ParseOptions) -> Result<(String, ParseTrace), ParseError>
pub fn parse_xml_tags(response: &str, tags: &[&str]) -> Result<HashMap<String, String>, ParseError>
pub fn parse_xml_tags_with_trace(response: &str, tags: &[&str], opts: &ParseOptions) -> Result<(HashMap<String, String>, ParseTrace), ParseError>

// Text
pub fn parse_text(response: &str) -> Result<String, ParseError>
pub fn parse_text_with_trace(response: &str, opts: &ParseOptions) -> Result<(String, ParseTrace), ParseError>

// Choice
pub fn parse_choice<'a>(response: &str, valid_choices: &[&'a str]) -> Result<&'a str, ParseError>
pub fn parse_choice_with_trace<'a>(response: &str, valid_choices: &[&'a str], opts: &ParseOptions) -> Result<(&'a str, ParseTrace), ParseError>

// Number
pub fn parse_number<T: FromStr>(response: &str) -> Result<T, ParseError>
pub fn parse_number_with_trace<T: FromStr>(response: &str, opts: &ParseOptions) -> Result<(T, ParseTrace), ParseError>
pub fn parse_number_in_range<T: FromStr + PartialOrd + Display>(response: &str, min: T, max: T) -> Result<T, ParseError>
pub fn parse_number_in_range_with_trace<T: FromStr + PartialOrd + Display>(response: &str, min: T, max: T, opts: &ParseOptions) -> Result<(T, ParseTrace), ParseError>

// List
pub fn parse_string_list(response: &str) -> Result<Vec<String>, ParseError>       // cleaned: lowercase, dedup, ≤50 chars
pub fn parse_string_list_raw(response: &str) -> Result<Vec<String>, ParseError>    // uncleaned
pub fn parse_string_list_with_trace(response: &str, opts: &ParseOptions) -> Result<(Vec<String>, ParseTrace), ParseError>

// Utilities
pub fn preprocess(text: &str) -> String                                             // strip think tags + trim
pub fn strip_think_tags(text: &str) -> String
pub fn extract_code_block(text: &str) -> Option<(Option<&str>, &str)>              // (lang_hint, content)
pub fn extract_code_block_for<'a>(text: &'a str, lang: &str) -> Option<&'a str>
pub fn find_bracketed(text: &str, open: char, close: char) -> Option<&str>

// Repair
pub fn try_repair_json(broken: &str) -> Option<String>

// YAML (feature-gated)
#[cfg(feature = "yaml")]
pub fn parse_yaml<T: DeserializeOwned>(response: &str) -> Result<T, ParseError>
```

### Types

```rust
pub struct ParseOptions {
    pub max_input_bytes: usize,        // 2_097_152 (2 MB)
    pub max_nesting_depth: usize,      // 64
    pub max_repair_attempts: usize,    // 3
    pub strip_think_tags: bool,        // true
    pub allow_code_fences: bool,       // true
}

pub struct ParseTrace {
    pub strategies_tried: Vec<&'static str>,
    pub repaired: bool,
    pub repair_actions: Vec<String>,
    pub extracted_span: Option<(usize, usize)>,
    pub warnings: Vec<String>,
}

pub enum ParseError {
    EmptyResponse,
    Unparseable { expected_format: &'static str, text: String },
    DeserializationFailed { reason: String, raw_json: String },
    NoMatchingChoice { valid: Vec<String> },
    NoNumber,
    TooLarge { size: usize, limit: usize },
    TooDeep { depth: usize, limit: usize },
}
```

### Strategy Pipelines
- **JSON:** direct → code block (lang-specific) → any code block → bracket-match `{}` → bracket-match `[]` → repair + retry
- **Number:** direct → labeled (score:/rating:) → fraction (X/Y) → scan all numbers (prefer last)
- **List:** JSON array → JSON object with list keys → code block → bracket-match → repair → numbered/bulleted → comma-separated
- **Choice:** exact match → prefix match → word-boundary search (all case-insensitive)
- **Text:** strip think tags → trim → strip boilerplate prefixes

### JSON Repair Steps
1. Strip inline comments
2. Replace Python literals (`True`→`true`, `False`→`false`, `None`→`null`)
3. Remove trailing commas
4. Replace single-quoted strings
5. Quote unquoted keys
6. Close missing brackets
7. Escape raw newlines in strings

---

## 14. semantic-memory-forge

**Crate:** semantic-memory-forge
**Version:** 0.2.0
**Edition:** 2021
**Location:** living-memory/

### Dependencies
- semantic-memory (path)
- All 10 Primitives crates (path)
- rusqlite (bundled), tokio, serde/serde_json, thiserror, uuid, blake3, chrono, petgraph, regex, async-trait, tracing, tempfile, walkdir, once_cell, rand, glob, similar, anyhow

### Module Structure

```
pub mod adapters     // ProjectAdapter trait, CargoAdapter
pub mod baseline     // BaselineDescriptor, WorkspacePolicy, ComparabilityPolicy
pub mod cea          // CausalGraph, attribution, prediction, store
pub mod config       // ForgeConfig + 12 sub-configs
pub mod error        // ForgeError (20+ variants), ForgeResult
pub mod exec         // ExecutionBackend trait, HostBackend, ContainerBackend
pub mod experiment   // PairedExperimentRunner, ExperimentResult, ExperimentDiff
pub mod export       // EpisodeExport seam to semantic-memory
pub mod failure      // FailureClass, FailureRecord
pub mod invariants   // refuse_to_open_db, validate_*, cea_no_raw_source
pub mod lab          // EvalSuite, EvidenceBundle, promotion, archive, scoring
pub mod runtime      // patch apply/validate, MindState compile, stabilizer, novelty
pub mod scoring      // ObjectivePolicy, PatchExecutionPlan, ComparabilityClass
pub mod store        // ForgeStore (SQLite-backed)
```

### Core Config

```rust
pub struct ForgeConfig {
    pub mode: String,
    pub execution_backend_preference: String,
    pub container_runtime_preference: String,
    pub allow_test_modifications: bool,
    pub sealed_allow_host_backend: bool,
    pub forbidden_paths: Vec<String>,
    pub caps: CapsConfig,
    pub mindstate: MindstateConfig,
    pub novelty: NoveltyConfig,
    pub stabilization: StabilizationConfig,
    pub container: ContainerConfig,
    pub lab: LabConfig,
    pub cea: CeaConfig,
    pub danger: DangerConfig,
    pub limits: ForgeLimits,
    pub workspace: WorkspacePolicy,
    pub statistics: StatisticsPolicy,
    pub comparability: ComparabilityPolicy,
}
```

### Evidence Bundle (central type)

```rust
pub struct EvidenceBundle {
    pub bundle_id: String,
    pub candidate_id: String,
    pub eval_id: String,
    pub version_id: String,
    pub scores: ScoreVector,
    pub hypotheses: Vec<CausalHypothesis>,
    pub verification: Option<VerificationPlan>,
    pub trace_id: String,
    pub experiment_diff: Option<ExperimentDiff>,
    pub attribution_json: Option<String>,
    pub assessment: Option<EvidenceAssessment>,
    pub warnings: Vec<String>,
    pub run_id: String,
    pub attempt_id: String,
    pub causal_question: String,
    pub unit_definition: String,
    pub bundle_scope: BundleScope,
    pub claim_strength: ClaimStrength,
    pub identification_rationale: String,
    pub known_threats: Vec<String>,
    pub patch_hash: String,
    pub treatment: Treatment,
    pub outcome: serde_json::Value,
    pub covariates: Covariates,
    pub primary_effect: Option<TypedLocatedEffect>,
    pub all_effects: Vec<TypedLocatedEffect>,
    pub hypothesis_edges: Vec<HypothesisEdge>,
    pub receipts: Vec<ReceiptRef>,
    pub sealed: bool,
}
```

### Execution

```rust
#[async_trait]
pub trait ExecutionBackend: Send + Sync {
    fn kind(&self) -> ExecutionBackendKind;
    async fn prepare_workspace(&self, fixture: &Path) -> ForgeResult<Workspace>;
    async fn run_command(&self, workspace: &Path, program: &str, args: &[&str], env: &[(&str, &str)], timeout_secs: u64) -> ForgeResult<CommandOutput>;
    async fn collect_logs(&self, fmt: &CommandOutput, clippy: &CommandOutput, test: &CommandOutput) -> ForgeResult<LogBundle>;
}

pub trait ProjectAdapter: Send + Sync {
    fn detect(workspace: &Path) -> bool;
    fn name(&self) -> &str;
    fn check_commands(&self, config: &ForgeConfig) -> Vec<CheckCommand>;
    fn parse_check_output(&self, cmd: &CheckCommand, stdout: &str, stderr: &str, exit_code: i32) -> ParsedCheckOutput;
}
```

### Causal Edit Attribution

```rust
pub fn attribute_effects(patch: &StructuredPatch, check_result: &CheckResult, line_map: &LineAttributionMap, max_line_distance: u32) -> ForgeResult<Vec<AttributionTriple>>
pub fn predict(signatures: &[EditOpSignature], graph: &CausalGraph, config: &CeaConfig) -> CausalPrediction
pub fn update_graph(store: &ForgeStore, result: &AttributedRunResult, eval_id: &str, version_id: &str, config: &ForgeConfig) -> ForgeResult<UpdateResult>
pub fn load_graph(store: &ForgeStore, version_id: Option<&str>) -> ForgeResult<CausalGraph>
```

### Lab / Promotion

```rust
pub fn promote(store: &ForgeStore, candidate_id: &str, config: &ForgeConfig) -> ForgeResult<BasisVersion>
pub fn archive_insert(store: &ForgeStore, candidate_id: &str, scores: &ScoreVector, patch: &StructuredPatch, config: &ForgeConfig, cea_fingerprint: Option<&str>) -> ForgeResult<ArchiveUpdate>
pub fn generate_verification_plan(hypotheses: &[CausalHypothesis], bundle: &EvidenceBundle, policy: &VerificationPolicy) -> VerificationPlan
```

### Error

```rust
pub enum ForgeError {
    RefuseToOpenDb, Database, Io, Serialization, PatchValidation,
    AnchorResolution, PatchApply, CommandTimeout, CommandFailed,
    SealedModeUnsupported, RemoteModelForbiddenInSealedMode,
    NoContainerRuntime, PromotionFailed, GoldenMindStateMismatch,
    CeaRawSourceDetected, Fixture, Config, NotFound, WorkspacePath,
    ExperimentFailed, LimitExceeded, Export, WriteThroughBlocked,
    PairIncomparable, SealedBundle, Other,
}
```

---

## 15. Primitives

**Location:** Primitives/
**10 sub-crates, all v0.1.0, edition 2021**

### 15.1 typed-patch

Strongly-typed patch representation with validation and application.

```rust
pub struct StructuredPatch { pub patch_id: Uuid, pub summary: String, pub edits: Vec<FileEdit>, pub notes: Option<String> }
pub struct FileEdit { pub path: String, pub ops: Vec<EditOp>, pub mode: Option<FileMode> }
pub enum FileMode { Create, Delete, Modify }
pub enum EditOp {
    Insert { anchor: Anchor, lines: Vec<String> },
    Delete { range: LineRange },
    Replace { range: LineRange, lines: Vec<String> },
}
pub enum Anchor {
    AfterLine { line: u32, context_before: Option<String>, context_after: Option<String> },
    BeforeLine { line: u32, context_before: Option<String>, context_after: Option<String> },
    AfterMatch { needle: String, occurrence: Option<u32> },
    BeforeMatch { needle: String, occurrence: Option<u32> },
}
pub struct LineRange { pub start: u32, pub end_exclusive: u32 }
pub struct LineAttributionMap { pub mappings: BTreeMap<...>, pub resolved_anchors: BTreeMap<...> }

pub fn validate_patch(patch: &StructuredPatch, policy: &PatchPolicy) -> ValidationResult
pub fn apply_patch<F: PatchFs>(patch: &StructuredPatch, fs: &F) -> Result<LineAttributionMap, PatchError>
pub async fn render_diff(original_dir: &Path, patched_dir: &Path) -> Result<String, PatchError>
```

### 15.2 forge-policy

Patch validation and workspace access policy.

```rust
pub fn verify_sqlite_db_identity(path: &Path, spec: &DbIdentitySpec) -> Result<(), PolicyError>
pub fn ensure_relative_path(path: &str) -> Result<(), PolicyError>
pub fn reject_symlinks(root: &Path, relative_path: &str) -> Result<(), PolicyError>
pub fn resolve_workspace_path(root: &Path, relative_path: &str) -> Result<PathBuf, PolicyError>
pub fn validate_forbidden_paths(paths: &[&str], forbidden_patterns: &[String], allow_test_modifications: bool) -> Vec<Violation>
pub fn validate_patch_caps(files_changed: usize, total_lines_changed: usize, per_file_lines: &[usize], max_files: usize, max_total_lines: usize, max_per_file: usize) -> Vec<Violation>
pub fn is_env_allowed(key: &str) -> bool

pub enum ViolationKind { ForbiddenPath, CapExceeded, EmptyPatch, UselessEdit, DegenerateRange, InvalidOccurrence, DuplicatePath }
```

### 15.3 sandbox-workspace

Temporary workspace isolation for patch application.

```rust
pub struct Workspace { pub host_path: PathBuf }
pub struct PatchedWorkspace { pub host_path: PathBuf }

pub trait PatchFs {
    fn root(&self) -> &Path;
    fn exists(&self, path: &str) -> bool;
    fn read_lines(&self, path: &str) -> Result<Vec<String>>;
    fn write_lines(&self, path: &str, lines: &[String]) -> Result<()>;
    fn remove_file(&self, path: &str) -> Result<()>;
    fn create_parent_dirs(&self, path: &str) -> Result<()>;
    fn snapshot_lines(&self, path: &str) -> Result<Vec<String>>;
}

pub fn prepare_workspace(fixture: &Path) -> WorkspaceResult<Workspace>
pub fn as_patched_workspace(workspace: &Workspace) -> PatchedWorkspace
```

### 15.4 check-runner

Executes fmt/clippy/test checks with standardized output parsing.

```rust
pub enum CheckKind { Fmt, Clippy, Test }
pub struct CheckResult { pub fmt_pass: bool, pub clippy_pass: bool, pub test_pass: bool, pub total_duration_ms: u64, ... }
pub struct CommandOutput { pub stdout: String, pub stderr: String, pub exit_code: i32, pub duration_ms: u64 }
pub enum ExecutionBackendKind { Host, Container }

#[async_trait]
pub trait ExecutionBackend: Send + Sync {
    fn kind(&self) -> ExecutionBackendKind;
    async fn prepare_workspace(&self, fixture: &Path) -> Result<Workspace>;
    async fn run_command(&self, workspace: &Path, program: &str, args: &[&str], env: &[(&str, &str)], timeout_secs: u64) -> Result<CommandOutput>;
    async fn collect_logs(&self, ...) -> Result<LogBundle>;
}

pub fn select_backend(config: &BackendConfig) -> Result<Box<dyn ExecutionBackend>>
```

### 15.5 effect-signature

Effect type definitions for check results.

```rust
pub struct EffectSignature { pub check_kind: String, pub outcome: String, pub severity: String, pub message_class: String, pub line_offset_from_edit: Option<i32> }
pub struct LocatedEffect { pub file: Option<PathBuf>, pub line: Option<u32>, pub col: Option<u32>, pub message: String, pub sig: EffectSignature }
pub fn effect_signature_hash(signature: &EffectSignature) -> String
```

### 15.6 cea-core

Causal Edit Attribution graph learning and prediction (no persistence).

```rust
pub struct CausalGraph { /* petgraph DiGraph */ }
pub enum CausalNode { Cause(EditOpSignature), Effect(EffectSignature) }
pub struct CausalEdge { pub weight: f64, pub count: u32, pub confidence: f64, pub stats: EdgeStats }
pub struct EdgeStats { pub alpha: f64, pub beta: f64, pub observations: u32 }
pub struct AttributionTriple { pub cause: EditOpSignature, pub effect: EffectSignature, pub distance: u32, pub weight: f64 }
pub struct CausalPrediction { pub predicted_correctness: f64, pub predicted_novelty: f64, pub confidence: f64, pub coverage_fraction: f64, pub risk_flags: Vec<RiskFlag>, pub zero_shot_eligible: bool }

pub fn attribute_effects(patch: &StructuredPatch, check_result: &CheckResult, ...) -> Result<Vec<AttributionTriple>>
pub fn predict(signatures: &[EditOpSignature], graph: &CausalGraph, ...) -> CausalPrediction
```

### 15.7 cea-store

Abstract store interface for CEA graph persistence.

```rust
pub trait CeaStore {
    fn has_run(&self, run_hash: &str) -> Result<bool>;
    fn upsert_node(&self, node_id: &str, node_kind: &str, sig_json: &str) -> Result<()>;
    fn upsert_edge(&self, edge_id: &str, cause: &str, effect: &str, weight_delta: f64, version_id: &str) -> Result<()>;
    fn insert_run_log(&self, run_hash: &str, eval_id: &str, edges_added: u32, edges_updated: u32) -> Result<()>;
    fn load_nodes(&self) -> Result<Vec<CeaNodeRow>>;
    fn load_edges(&self, version_id: Option<&str>) -> Result<Vec<CeaEdgeRow>>;
}

pub fn update_graph<S: CeaStore>(store: &S, result: &AttributedRunResult, ...) -> Result<UpdateResult>
pub fn load_graph<S: CeaStore>(store: &S, version_id: Option<&str>) -> Result<CausalGraph>
```

### 15.8 cea-sqlite

SQLite persistence for CEA graph.

```rust
pub struct SqliteCeaStore { /* rusqlite connection */ }
impl SqliteCeaStore { pub fn open(path: &Path) -> Result<Self> }
// Implements CeaStore trait
```

### 15.9 mindstate-core

Serializable mind state for LLM context.

```rust
pub struct MindState { pub request: String, pub repo_context: String, pub evidence: Vec<EvidenceItem>, pub traces: Vec<TraceRef>, pub basis_version_id: String, pub config_overrides: BTreeMap<String, String> }
pub struct EvidenceItem { pub key: String, pub content: String, pub score: OrderedFloat }
pub struct TraceRef { pub question_sig: String, pub strategy_tags: Vec<String>, pub score: OrderedFloat }
pub struct OrderedFloat(pub f64);  // total ordering wrapper

impl MindState {
    pub fn render(&self) -> Result<String>   // deterministic serialization
    pub fn hash(&self) -> Result<String>     // BLAKE3
}
pub fn compute_question_sig(request: &str, repo_context: &str) -> String
pub fn budget_evidence(evidence: Vec<EvidenceItem>, budget: usize) -> Vec<EvidenceItem>
```

### 15.10 stabilizer-core

Multi-phase stabilization for patch generation attempts.

```rust
pub enum AttemptPhase { Innovative = 0, Stabilize1 = 1, Stabilize2 = 2, Clamp = 3 }
pub struct DeltaPolicy { pub delta_amp_default: f64, pub delta_amp_stabilize1: f64, pub delta_amp_stabilize2: f64, pub delta_amp_clamp: f64 }
pub struct Stabilizer { /* tracks phase progression */ }
pub struct AttemptOverrides { pub phase: AttemptPhase, pub delta_amplitude: f64, pub force_family: Option<String>, pub force_minimal_diff: bool, pub weight_factor: f64 }

impl Stabilizer {
    pub fn new(delta_policy: DeltaPolicy, config: StabilizerConfig) -> Self
    pub fn next_attempt(&mut self) -> Result<AttemptOverrides, StabilizerError>
    pub fn has_next(&self) -> bool
    pub fn reset(&mut self)
}

pub fn extract_strategy_tags(patch: &StructuredPatch) -> Vec<String>
pub fn compute_tag_novelty(current_tags: &[String], recent_tags_union: &BTreeSet<String>) -> f64
pub fn determine_approach_family(tags: &[String]) -> &'static str
// Families: mechanical, pattern_refactor, architectural, perf, safety
```

---

*End of code-only reference.*
