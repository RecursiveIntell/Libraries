# semantic-memory — Technical Specification

**Crate Name:** `semantic-memory`
**Location:** `~/Coding/Libraries/semantic-memory/`
**Purpose:** Standalone Rust library for hybrid semantic search over conversations, facts, and arbitrary text. SQLite-backed, local-first, zero external services beyond an embedding endpoint.
**Reuse targets:** Ironforge agent, Gloss, Sortarr, Homelab Mission Control, any future project needing RAG.

---

## 1. What This Crate Does (And Does Not Do)

### Does:
- Stores text with vector embeddings in a single SQLite database file
- Full-text search via FTS5 with BM25 scoring
- Vector similarity search via brute-force cosine similarity over BLOB-stored embeddings
- Hybrid retrieval merging BM25 + cosine via Reciprocal Rank Fusion (RRF)
- Conversation message logging with session isolation and token counting
- Fact/knowledge store with semantic lookup ("What do I know about X?")
- Text chunking for long documents before embedding
- Configurable embedding provider (trait-based, Ollama default)

### Does NOT:
- Run its own embedding model (it calls an external endpoint — Ollama, OpenAI, etc.)
- Manage GPU resources (that's the caller's responsibility)
- Provide LLM integration (that's llm-bridge / LLM-Pipeline)
- Handle prompt construction (the caller decides what to do with retrieved results)
- Implement HNSW or any approximate nearest neighbor index (brute-force is fast enough for <100K vectors at 768 dimensions)

---

## 2. Dependencies

```toml
[dependencies]
rusqlite = { version = "0.32", features = ["bundled", "blob"] }
reqwest = { version = "0.12", features = ["json"], default-features = false }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tokio = { version = "1", features = ["rt", "macros"] }
thiserror = "2"
tracing = "0.1"
uuid = { version = "1", features = ["v4"] }
chrono = { version = "0.4", features = ["serde"] }

[dev-dependencies]
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
tempfile = "3"
```

### Why these specific crates:
- **rusqlite with `bundled`**: Guarantees FTS5 is compiled in. The system SQLite on many Linux distros does NOT include FTS5. Bundled compiles SQLite from source with FTS5 enabled. The `blob` feature enables incremental BLOB I/O for large embeddings.
- **reqwest**: HTTP client for calling Ollama's `/api/embed` endpoint. Already in your dependency tree via LLM-Pipeline.
- **No fastembed**: Embedding is done via HTTP to Ollama, not in-process. Zero GPU contention, zero ONNX dependency, zero VRAM management.
- **No usearch/hnswlib**: Brute-force cosine similarity over SQLite BLOBs. For a single-user agent with <100K vectors at 768 dimensions, a full scan takes <5ms. The complexity of maintaining a separate HNSW index file is not justified.

### What about your SQLite Migrations crate?
Don't depend on it. This crate manages its own schema with a simple version table (same pattern, inlined). Keeps the dependency tree minimal and the crate truly standalone. If you later want to converge them, it's a 20-line refactor.

---

## 3. Crate Structure

```
semantic-memory/
├── Cargo.toml
├── src/
│   ├── lib.rs                  # Public API re-exports
│   ├── error.rs                # Error types (thiserror)
│   ├── config.rs               # MemoryConfig struct
│   ├── db.rs                   # Database initialization, migrations, connection management
│   ├── embedder.rs             # Embedder trait + OllamaEmbedder implementation
│   ├── chunker.rs              # Text chunking (recursive split)
│   ├── conversation.rs         # Conversation message storage and retrieval
│   ├── knowledge.rs            # Fact/knowledge store with semantic lookup
│   ├── search.rs               # Hybrid search engine (BM25 + vector + RRF)
│   └── types.rs                # Shared types (SearchResult, Message, Fact, etc.)
├── reference/
│   ├── hybrid_search.rs        # Gloss's hybrid_search.rs — algorithm reference
│   └── chunk.rs                # Gloss's chunk.rs — chunking reference
├── tests/
│   ├── integration_tests.rs    # Full pipeline tests (embed → store → search)
│   ├── chunker_tests.rs        # Chunking edge cases
│   ├── search_tests.rs         # Scoring and ranking validation
│   └── conversation_tests.rs   # Message storage and retrieval
├── examples/
│   ├── basic_search.rs         # Minimal: store 3 facts, search for one
│   └── conversation_memory.rs  # Log a conversation, retrieve context
└── README.md
```

---

## 4. Database Schema

Single SQLite file. WAL mode. Foreign keys enabled.

### 4.1 Schema Version Table

```sql
CREATE TABLE IF NOT EXISTS _schema_version (
    version INTEGER PRIMARY KEY,
    applied_at TEXT NOT NULL DEFAULT (datetime('now'))
);
```

### 4.2 Migration V1 — Core Tables

```sql
-- ═══════════════════════════════════════════════════════════════════
-- CONVERSATIONS
-- ═══════════════════════════════════════════════════════════════════

-- Each conversation session. A session is a contiguous exchange on one channel.
CREATE TABLE sessions (
    id          TEXT PRIMARY KEY,               -- UUID v4
    channel     TEXT NOT NULL DEFAULT 'repl',   -- 'repl', 'telegram', 'websocket', etc.
    created_at  TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at  TEXT NOT NULL DEFAULT (datetime('now')),
    metadata    TEXT                            -- JSON blob, optional (channel-specific data)
);

CREATE INDEX idx_sessions_updated ON sessions(updated_at DESC);

-- Individual messages within a session.
CREATE TABLE messages (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id  TEXT NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
    role        TEXT NOT NULL CHECK (role IN ('system', 'user', 'assistant', 'tool')),
    content     TEXT NOT NULL,
    token_count INTEGER,                       -- Estimated token count (caller provides)
    created_at  TEXT NOT NULL DEFAULT (datetime('now')),
    metadata    TEXT                            -- JSON blob: tool_call_id, model used, latency, etc.
);

CREATE INDEX idx_messages_session ON messages(session_id, created_at ASC);
CREATE INDEX idx_messages_created ON messages(created_at DESC);

-- ═══════════════════════════════════════════════════════════════════
-- KNOWLEDGE (Facts / Long-term Memory)
-- ═══════════════════════════════════════════════════════════════════

-- Discrete facts the agent knows. Each fact has an embedding for semantic search.
CREATE TABLE facts (
    id          TEXT PRIMARY KEY,               -- UUID v4
    namespace   TEXT NOT NULL DEFAULT 'general',-- Categorization: 'user', 'system', 'project:foo'
    content     TEXT NOT NULL,                  -- The fact text
    source      TEXT,                           -- Where this fact came from: 'conversation:SESSION_ID', 'manual', 'tool:X'
    embedding   BLOB,                           -- f32 vector as raw bytes (768 dims = 3072 bytes for nomic-embed-text)
    created_at  TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at  TEXT NOT NULL DEFAULT (datetime('now')),
    metadata    TEXT                            -- JSON blob: confidence, extraction_model, etc.
);

CREATE INDEX idx_facts_namespace ON facts(namespace);
CREATE INDEX idx_facts_updated ON facts(updated_at DESC);

-- FTS5 virtual table for full-text search over facts.
-- content='' means this is a contentless FTS table — it indexes but doesn't store
-- a copy of the text. Reads go through the facts table via rowid join.
-- We use content_rowid to link FTS entries back to facts via a numeric alias.
CREATE TABLE facts_rowid_map (
    rowid       INTEGER PRIMARY KEY AUTOINCREMENT,
    fact_id     TEXT NOT NULL UNIQUE REFERENCES facts(id) ON DELETE CASCADE
);

CREATE VIRTUAL TABLE facts_fts USING fts5(
    content,
    content='',
    content_rowid='rowid',
    tokenize='porter unicode61'
);

-- ═══════════════════════════════════════════════════════════════════
-- DOCUMENTS (Chunked long-form content)
-- ═══════════════════════════════════════════════════════════════════

-- Source documents that have been chunked and embedded.
CREATE TABLE documents (
    id          TEXT PRIMARY KEY,               -- UUID v4
    title       TEXT NOT NULL,
    source_path TEXT,                           -- File path, URL, or identifier
    namespace   TEXT NOT NULL DEFAULT 'general',
    created_at  TEXT NOT NULL DEFAULT (datetime('now')),
    metadata    TEXT                            -- JSON: file_type, size, etc.
);

-- Individual chunks of a document, each with its own embedding.
CREATE TABLE chunks (
    id          TEXT PRIMARY KEY,               -- UUID v4
    document_id TEXT NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
    chunk_index INTEGER NOT NULL,              -- Position within document (0-based)
    content     TEXT NOT NULL,
    token_count INTEGER,
    embedding   BLOB,                           -- f32 vector as raw bytes
    created_at  TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_chunks_document ON chunks(document_id, chunk_index ASC);

-- FTS for chunks (same contentless pattern as facts)
CREATE TABLE chunks_rowid_map (
    rowid       INTEGER PRIMARY KEY AUTOINCREMENT,
    chunk_id    TEXT NOT NULL UNIQUE REFERENCES chunks(id) ON DELETE CASCADE
);

CREATE VIRTUAL TABLE chunks_fts USING fts5(
    content,
    content='',
    content_rowid='rowid',
    tokenize='porter unicode61'
);

-- ═══════════════════════════════════════════════════════════════════
-- EMBEDDING METADATA
-- ═══════════════════════════════════════════════════════════════════

-- Track which model produced the embeddings so we know when to re-embed.
CREATE TABLE embedding_metadata (
    id          INTEGER PRIMARY KEY CHECK (id = 1),  -- Singleton row
    model_name  TEXT NOT NULL,                        -- e.g., 'nomic-embed-text'
    dimensions  INTEGER NOT NULL,                     -- e.g., 768
    updated_at  TEXT NOT NULL DEFAULT (datetime('now'))
);
```

### Why this schema:

**Contentless FTS5 (`content=''`):** The actual text lives in `facts.content` and `chunks.content`. The FTS table only stores the inverted index, not a duplicate of the text. This saves ~50% storage vs a regular FTS table. The tradeoff is you must manually keep FTS in sync (insert/delete via `facts_rowid_map`), but that's handled in the application code.

**`facts_rowid_map` bridge table:** FTS5 requires integer rowids. Our facts use UUID text primary keys. The bridge table maps between them. When inserting a fact: (1) insert into `facts`, (2) insert into `facts_rowid_map`, (3) insert into `facts_fts` using the bridge rowid. Same pattern for chunks.

**`embedding` as BLOB:** A 768-dimension f32 vector is exactly 3072 bytes. Stored as a raw BLOB (no JSON overhead, no base64 overhead). To read: cast the BLOB to `&[u8]` then reinterpret as `&[f32]`. To write: cast `&[f32]` to `&[u8]` via manual byte conversion.

**`embedding_metadata` singleton:** If the user switches embedding models (e.g., from `nomic-embed-text` to `mxbai-embed-large`), all existing embeddings are invalid. This table tracks the current model. On startup, if the model doesn't match, the crate logs a warning and can optionally trigger a re-embed job.

**`porter unicode61` tokenizer:** Porter stemming means "running" matches "run", "runs", etc. Unicode61 handles non-ASCII text properly. This is the standard choice for English-primary content with occasional non-English.

---

## 5. Public API

### 5.1 Configuration

```rust
/// Configuration for the memory system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryConfig {
    /// Path to the SQLite database file.
    /// Example: "/home/josh/.local/share/ironforge/memory.db"
    pub database_path: PathBuf,

    /// Embedding provider configuration.
    pub embedding: EmbeddingConfig,

    /// Search tuning parameters.
    pub search: SearchConfig,

    /// Chunking parameters.
    pub chunking: ChunkingConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingConfig {
    /// Ollama base URL.
    /// Default: "http://localhost:11434"
    pub ollama_url: String,

    /// Embedding model name.
    /// Default: "nomic-embed-text"
    pub model: String,

    /// Expected embedding dimensions.
    /// Default: 768 (nomic-embed-text)
    /// Used for validation — if the model returns a different dimension count,
    /// the crate returns an error rather than silently storing mismatched vectors.
    pub dimensions: usize,

    /// Maximum texts to embed in a single API call.
    /// Ollama's /api/embed supports batch input.
    /// Default: 32
    pub batch_size: usize,

    /// Timeout for embedding requests in seconds.
    /// Default: 30
    pub timeout_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchConfig {
    /// Weight for BM25 score in RRF fusion.
    /// Higher = trust keyword matches more.
    /// Default: 1.0
    pub bm25_weight: f64,

    /// Weight for vector similarity in RRF fusion.
    /// Higher = trust semantic similarity more.
    /// Default: 1.0
    pub vector_weight: f64,

    /// RRF constant (k). Controls how quickly rank importance decays.
    /// Standard value is 60. Lower values make top ranks more dominant.
    /// Default: 60.0
    pub rrf_k: f64,

    /// Number of candidates to pull from each search method before fusion.
    /// The BM25 search and vector search each return this many candidates.
    /// RRF merges them, then truncates to the requested top_k.
    /// Default: 50
    pub candidate_pool_size: usize,

    /// Default number of results to return.
    /// Default: 5
    pub default_top_k: usize,

    /// Minimum cosine similarity threshold.
    /// Results below this score are excluded from vector search candidates.
    /// Range: 0.0 to 1.0. Default: 0.3
    pub min_similarity: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkingConfig {
    /// Target chunk size in characters.
    /// Default: 1000
    pub target_size: usize,

    /// Minimum chunk size. Chunks smaller than this are merged with neighbors.
    /// Default: 100
    pub min_size: usize,

    /// Maximum chunk size. Chunks larger than this are force-split.
    /// Default: 2000
    pub max_size: usize,

    /// Overlap between adjacent chunks in characters.
    /// Default: 200
    pub overlap: usize,
}
```

### 5.2 Core Struct: `MemoryStore`

This is the primary entry point. One `MemoryStore` per database file.

```rust
/// Thread-safe handle to the memory database.
/// Clone is cheap (Arc internals). Send + Sync.
pub struct MemoryStore {
    // Internal: Arc<MemoryStoreInner>
    // MemoryStoreInner {
    //     conn: Mutex<Connection>,    // rusqlite Connection (not Send, needs Mutex)
    //     embedder: Box<dyn Embedder>,
    //     config: MemoryConfig,
    // }
}

impl MemoryStore {
    /// Open or create a memory database at the configured path.
    /// Runs migrations automatically.
    /// Validates embedding model dimensions against stored metadata.
    ///
    /// # Errors
    /// - Database file cannot be opened/created
    /// - Migration failure
    /// - Embedding model mismatch (different dimensions than stored data)
    pub fn open(config: MemoryConfig) -> Result<Self, MemoryError>;

    /// Open with a custom embedder (for testing or non-Ollama providers).
    pub fn open_with_embedder(
        config: MemoryConfig,
        embedder: Box<dyn Embedder>,
    ) -> Result<Self, MemoryError>;
}
```

### 5.3 Conversation API

```rust
impl MemoryStore {
    // ─── Session Management ─────────────────────────────────────

    /// Create a new conversation session.
    /// Returns the session ID (UUID v4).
    pub fn create_session(&self, channel: &str) -> Result<String, MemoryError>;

    /// List recent sessions, newest first.
    pub fn list_sessions(
        &self,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<Session>, MemoryError>;

    /// Delete a session and all its messages.
    pub fn delete_session(&self, session_id: &str) -> Result<(), MemoryError>;

    // ─── Message Storage ────────────────────────────────────────

    /// Append a message to a session.
    /// Updates the session's updated_at timestamp.
    /// Returns the message's auto-increment ID.
    ///
    /// token_count is optional. If None, the crate does NOT estimate it —
    /// the caller is responsible for token counting (because tokenization
    /// depends on the LLM model being used, which this crate doesn't know).
    pub fn add_message(
        &self,
        session_id: &str,
        role: Role,
        content: &str,
        token_count: Option<u32>,
        metadata: Option<serde_json::Value>,
    ) -> Result<i64, MemoryError>;

    /// Get the most recent N messages from a session, in chronological order.
    /// This is your "conversation window" — feed these to the LLM as context.
    pub fn get_recent_messages(
        &self,
        session_id: &str,
        limit: usize,
    ) -> Result<Vec<Message>, MemoryError>;

    /// Get messages from a session, but only up to max_tokens total.
    /// Walks backward from newest, accumulating token counts, stops when
    /// the budget is exceeded. Returns messages in chronological order.
    ///
    /// If any messages have token_count = None, they are counted as 0
    /// (meaning they're always included — the caller should have provided counts).
    pub fn get_messages_within_budget(
        &self,
        session_id: &str,
        max_tokens: u32,
    ) -> Result<Vec<Message>, MemoryError>;

    /// Get total token count for a session.
    /// Only counts messages that have a non-null token_count.
    pub fn session_token_count(&self, session_id: &str) -> Result<u64, MemoryError>;
}
```

### 5.4 Knowledge API

```rust
impl MemoryStore {
    // ─── Fact CRUD ──────────────────────────────────────────────

    /// Store a fact with automatic embedding.
    /// The content is embedded via the configured provider.
    /// Returns the fact ID (UUID v4).
    ///
    /// # Example
    /// store.add_fact(
    ///     "user",                                     // namespace
    ///     "Josh's son is 19 and works at Whataburger", // content
    ///     Some("conversation:abc-123"),                // source
    ///     None,                                        // metadata
    /// ).await?;
    pub async fn add_fact(
        &self,
        namespace: &str,
        content: &str,
        source: Option<&str>,
        metadata: Option<serde_json::Value>,
    ) -> Result<String, MemoryError>;

    /// Store a fact with a pre-computed embedding.
    /// Use this when you've already embedded the text (e.g., batch processing).
    pub fn add_fact_with_embedding(
        &self,
        namespace: &str,
        content: &str,
        embedding: &[f32],
        source: Option<&str>,
        metadata: Option<serde_json::Value>,
    ) -> Result<String, MemoryError>;

    /// Update a fact's content. Re-embeds automatically.
    pub async fn update_fact(
        &self,
        fact_id: &str,
        content: &str,
    ) -> Result<(), MemoryError>;

    /// Delete a fact by ID. Removes from FTS and main table.
    pub fn delete_fact(&self, fact_id: &str) -> Result<(), MemoryError>;

    /// Delete all facts in a namespace.
    pub fn delete_namespace(&self, namespace: &str) -> Result<usize, MemoryError>;

    /// Get a fact by ID.
    pub fn get_fact(&self, fact_id: &str) -> Result<Option<Fact>, MemoryError>;

    /// List all facts in a namespace.
    pub fn list_facts(
        &self,
        namespace: &str,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<Fact>, MemoryError>;
}
```

### 5.5 Document Ingestion API

```rust
impl MemoryStore {
    /// Ingest a document: chunk it, embed all chunks, store everything.
    /// Returns the document ID.
    ///
    /// This is the "fire and forget" API for adding a whole document.
    /// For fine-grained control, use add_document + add_chunks separately.
    ///
    /// # What happens internally:
    /// 1. Text is split into chunks via the configured chunking strategy
    /// 2. All chunks are embedded in batches (batch_size from config)
    /// 3. Document record is created
    /// 4. Chunks are stored with embeddings
    /// 5. FTS entries are created for all chunks
    /// All within a single transaction — if any step fails, nothing is written.
    pub async fn ingest_document(
        &self,
        title: &str,
        content: &str,
        namespace: &str,
        source_path: Option<&str>,
        metadata: Option<serde_json::Value>,
    ) -> Result<String, MemoryError>;

    /// Delete a document and all its chunks.
    pub fn delete_document(&self, document_id: &str) -> Result<(), MemoryError>;

    /// List documents in a namespace.
    pub fn list_documents(
        &self,
        namespace: &str,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<Document>, MemoryError>;
}
```

### 5.6 Search API

```rust
impl MemoryStore {
    /// Hybrid search across facts and document chunks.
    /// This is the main retrieval entry point.
    ///
    /// Returns results from BOTH the facts table and the chunks table,
    /// merged and ranked together. Each result indicates its source type.
    ///
    /// # Algorithm:
    /// 1. Embed the query via the embedding provider
    /// 2. BM25 search via FTS5 → top candidate_pool_size results from each table
    /// 3. Vector similarity search via cosine over BLOBs → top candidate_pool_size results
    /// 4. Reciprocal Rank Fusion to merge and re-rank
    /// 5. Return top top_k results
    ///
    /// # Arguments
    /// - query: Natural language search query
    /// - top_k: Number of results to return (overrides config default if Some)
    /// - namespaces: Optional filter — only search these namespaces. None = search all.
    /// - source_types: Optional filter — SearchSourceType::Facts, Chunks, or both.
    pub async fn search(
        &self,
        query: &str,
        top_k: Option<usize>,
        namespaces: Option<&[&str]>,
        source_types: Option<&[SearchSourceType]>,
    ) -> Result<Vec<SearchResult>, MemoryError>;

    /// Full-text search only (no embeddings, no vector similarity).
    /// Useful when Ollama is unavailable or for exact keyword matching.
    /// Returns results ranked by BM25 score only.
    pub fn search_fts_only(
        &self,
        query: &str,
        top_k: Option<usize>,
        namespaces: Option<&[&str]>,
        source_types: Option<&[SearchSourceType]>,
    ) -> Result<Vec<SearchResult>, MemoryError>;

    /// Vector similarity search only (no FTS).
    /// Useful for "find things similar to X" where keyword overlap is unreliable.
    pub async fn search_vector_only(
        &self,
        query: &str,
        top_k: Option<usize>,
        namespaces: Option<&[&str]>,
        source_types: Option<&[SearchSourceType]>,
    ) -> Result<Vec<SearchResult>, MemoryError>;
}
```

### 5.7 Utility API

```rust
impl MemoryStore {
    /// Chunk text using the configured strategy.
    /// Exposed publicly for callers who want to inspect chunks before storing.
    pub fn chunk_text(&self, text: &str) -> Vec<TextChunk>;

    /// Embed a single text via the configured provider.
    /// Exposed for callers who need embeddings for other purposes.
    pub async fn embed(&self, text: &str) -> Result<Vec<f32>, MemoryError>;

    /// Embed multiple texts in a batch.
    pub async fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>, MemoryError>;

    /// Get database statistics.
    pub fn stats(&self) -> Result<MemoryStats, MemoryError>;

    /// Re-embed all facts and chunks. Call this after changing embedding models.
    /// This is a potentially long operation — logs progress via tracing.
    /// Returns the number of items re-embedded.
    pub async fn reembed_all(&self) -> Result<usize, MemoryError>;

    /// Vacuum the database (reclaim space after deletions).
    pub fn vacuum(&self) -> Result<(), MemoryError>;
}
```

---

## 6. Types

```rust
// ─── Enums ──────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Role {
    System,
    User,
    Assistant,
    Tool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchSourceType {
    Facts,
    Chunks,
}

// ─── Data Structs ───────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub channel: String,
    pub created_at: String,    // ISO 8601
    pub updated_at: String,
    pub metadata: Option<serde_json::Value>,
    pub message_count: u32,    // Populated on list queries via COUNT
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: i64,
    pub session_id: String,
    pub role: Role,
    pub content: String,
    pub token_count: Option<u32>,
    pub created_at: String,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fact {
    pub id: String,
    pub namespace: String,
    pub content: String,
    pub source: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub metadata: Option<serde_json::Value>,
    // Note: embedding is NOT included in Fact struct.
    // It's 3KB of binary data that's useless to display.
    // Access via get_fact_embedding() if you really need it.
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub id: String,
    pub title: String,
    pub source_path: Option<String>,
    pub namespace: String,
    pub created_at: String,
    pub metadata: Option<serde_json::Value>,
    pub chunk_count: u32,      // Populated via COUNT on list queries
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextChunk {
    pub index: usize,          // Position in original document
    pub content: String,
    pub token_count_estimate: usize,  // Rough estimate: chars / 4
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    /// What was found.
    pub content: String,

    /// Where it came from.
    pub source: SearchSource,

    /// Combined RRF score. Higher = more relevant.
    /// Range is roughly 0.0 to 0.03 for typical RRF with k=60.
    /// Only useful for relative ranking, not absolute thresholds.
    pub score: f64,

    /// Individual component scores for debugging/tuning.
    pub bm25_rank: Option<usize>,
    pub vector_rank: Option<usize>,
    pub cosine_similarity: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SearchSource {
    Fact {
        fact_id: String,
        namespace: String,
    },
    Chunk {
        chunk_id: String,
        document_id: String,
        document_title: String,
        chunk_index: usize,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStats {
    pub total_facts: u64,
    pub total_documents: u64,
    pub total_chunks: u64,
    pub total_sessions: u64,
    pub total_messages: u64,
    pub database_size_bytes: u64,
    pub embedding_model: Option<String>,
    pub embedding_dimensions: Option<usize>,
}
```

---

## 7. Embedder Trait

```rust
/// Trait for embedding text into vectors.
/// Implement this to swap embedding providers.
#[async_trait]
pub trait Embedder: Send + Sync {
    /// Embed a single text. Returns a vector of f32.
    async fn embed(&self, text: &str) -> Result<Vec<f32>, MemoryError>;

    /// Embed multiple texts in a batch. Default implementation calls embed() in a loop.
    /// Override for providers that support native batching (like Ollama).
    async fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>, MemoryError> {
        let mut results = Vec::with_capacity(texts.len());
        for text in texts {
            results.push(self.embed(text).await?);
        }
        Ok(results)
    }

    /// The model name this embedder uses. Used for metadata tracking.
    fn model_name(&self) -> &str;

    /// Expected embedding dimensions. Used for validation.
    fn dimensions(&self) -> usize;
}
```

### 7.1 OllamaEmbedder (Default Implementation)

```rust
pub struct OllamaEmbedder {
    client: reqwest::Client,
    base_url: String,    // e.g., "http://localhost:11434"
    model: String,       // e.g., "nomic-embed-text"
    dimensions: usize,   // e.g., 768
    batch_size: usize,   // Max texts per API call
}

impl OllamaEmbedder {
    pub fn new(config: &EmbeddingConfig) -> Self;
}
```

**Ollama API call (the `/api/embed` endpoint):**

```
POST http://localhost:11434/api/embed
Content-Type: application/json

{
    "model": "nomic-embed-text",
    "input": ["text one", "text two", "text three"]
}

Response:
{
    "model": "nomic-embed-text",
    "embeddings": [
        [0.123, -0.456, ...],   // 768 floats
        [0.789, -0.012, ...],
        [0.345, -0.678, ...]
    ]
}
```

**IMPORTANT:** Use the NEW `/api/embed` endpoint with the `input` field (array), NOT the legacy `/api/embeddings` endpoint with the `prompt` field (single string). The new endpoint supports native batching.

**Batch strategy:** Split input texts into sub-batches of `batch_size`. Call `/api/embed` for each sub-batch. Concatenate results. This prevents Ollama from OOM-ing on huge batches.

**Error handling for OllamaEmbedder:**
- Connection refused → `MemoryError::EmbedderUnavailable("Ollama not running at {url}")`
- Timeout → `MemoryError::EmbedderUnavailable("Ollama embedding timed out after {n}s")`
- Model not found (404) → `MemoryError::EmbedderUnavailable("Model '{model}' not available in Ollama. Run: ollama pull {model}")`
- Dimension mismatch → `MemoryError::DimensionMismatch { expected, actual }`

### 7.2 MockEmbedder (For Testing)

```rust
/// Deterministic embedder for unit tests.
/// Generates a consistent embedding based on a hash of the input text.
/// This means the same text always produces the same embedding,
/// and similar texts produce somewhat similar embeddings.
pub struct MockEmbedder {
    dimensions: usize,
}

impl MockEmbedder {
    pub fn new(dimensions: usize) -> Self;
}
```

The mock embedder hashes the input text, seeds an RNG with the hash, and generates `dimensions` random f32 values. This is deterministic (same input → same output) but NOT semantically meaningful (similar texts won't have similar embeddings). It's sufficient for testing the storage/retrieval pipeline without needing Ollama running.

---

## 8. Search Algorithm — Hybrid Retrieval with RRF

This is the core of the crate. Reference Gloss's `hybrid_search.rs` during implementation.

### 8.1 Step-by-Step Algorithm

Given a query string `q` and a desired result count `top_k`:

**Step 1: Embed the query**
```
q_embedding = embedder.embed(q)
```

**Step 2: BM25 search via FTS5**
```sql
-- For facts:
SELECT fm.fact_id, bm25(facts_fts) AS score
FROM facts_fts
JOIN facts_rowid_map fm ON facts_fts.rowid = fm.rowid
WHERE facts_fts MATCH ?1
ORDER BY bm25(facts_fts)  -- FTS5 BM25 returns negative scores; more negative = better match
LIMIT ?2                   -- candidate_pool_size
```
Same query structure for chunks_fts.

**FTS5 query construction:** The raw user query must be sanitized before passing to FTS5 MATCH. FTS5 has its own query syntax (AND, OR, NOT, phrases in quotes, column filters, etc.). Unescaped special characters will cause SQL errors.

Sanitization rules:
1. Strip characters that are FTS5 operators: `"`, `*`, `+`, `-`, `(`, `)`, `^`, `{`, `}`, `~`
2. Split on whitespace into tokens
3. Remove empty tokens
4. If no tokens remain after sanitization, return empty results (don't query FTS)
5. Join tokens with spaces (implicit AND in FTS5 — all terms must appear)
6. Optionally, append `*` to each token for prefix matching: `"rust embed*"` matches "embedding", "embeddings", etc. Make this configurable via a `prefix_search: bool` field on SearchConfig (default: false, because prefix matching can produce noisy results).

**Step 3: Vector similarity search**

Load ALL embeddings from the target tables and compute cosine similarity in Rust. Yes, all of them. Here's why this isn't insane:

- 10,000 facts × 768 dims × 4 bytes = ~29 MB. Fits in memory trivially.
- Cosine similarity is a dot product + two norms. With 768 dims, that's ~2300 FLOPs per comparison.
- 10,000 comparisons = ~23M FLOPs. On a single core at ~1 GFLOP/s, that's ~23ms.
- In practice, Rust's autovectorization (SIMD) makes this faster. Expect <5ms for 10K vectors.
- At 100K vectors, it's ~50ms. Still acceptable for an interactive agent.

```sql
-- Load all embeddings + IDs from facts
SELECT id, embedding FROM facts WHERE embedding IS NOT NULL
-- Optionally filter by namespace:
-- AND namespace IN (?, ?, ...)
```

Then in Rust:
```rust
fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    debug_assert_eq!(a.len(), b.len(), "embedding dimension mismatch");
    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }
    dot / (norm_a * norm_b)
}
```

Filter by `min_similarity`, sort descending, take top `candidate_pool_size`.

**Step 4: Reciprocal Rank Fusion (RRF)**

Merge the two ranked lists. Each item gets a score from each list based on its rank:

```
rrf_score(item) = Σ (weight / (k + rank))
```

Where:
- `k` is the RRF constant (default 60)
- `rank` is the 1-based position in each list (1 = best)
- `weight` is `bm25_weight` or `vector_weight` from config
- If an item appears in only one list, its score from the other list is 0

Concrete example with k=60, both weights=1.0:
- Item A: BM25 rank 1, vector rank 3 → score = 1/(60+1) + 1/(60+3) = 0.01639 + 0.01587 = 0.03226
- Item B: BM25 rank 2, vector rank 1 → score = 1/(60+2) + 1/(60+1) = 0.01613 + 0.01639 = 0.03252
- Item C: BM25 only, rank 3 → score = 1/(60+3) + 0 = 0.01587

Sort by combined score descending, return top `top_k`.

**Implementation detail:** Use a `HashMap<String, RrfCandidate>` keyed by item ID. Walk the BM25 list, inserting/updating candidates with their BM25 rank. Walk the vector list, inserting/updating with their vector rank. Then compute the final score for each candidate, sort, and truncate.

```rust
struct RrfCandidate {
    id: String,
    content: String,
    source: SearchSource,
    bm25_rank: Option<usize>,      // 1-based
    vector_rank: Option<usize>,    // 1-based
    cosine_similarity: Option<f64>,
}

impl RrfCandidate {
    fn score(&self, config: &SearchConfig) -> f64 {
        let bm25_score = self.bm25_rank
            .map(|r| config.bm25_weight / (config.rrf_k + r as f64))
            .unwrap_or(0.0);
        let vector_score = self.vector_rank
            .map(|r| config.vector_weight / (config.rrf_k + r as f64))
            .unwrap_or(0.0);
        bm25_score + vector_score
    }
}
```

### 8.2 BLOB Encoding/Decoding

Embeddings are stored as raw bytes. No JSON. No base64. Raw little-endian f32 values.

```rust
/// Encode an f32 slice as bytes for SQLite BLOB storage.
fn embedding_to_bytes(embedding: &[f32]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(embedding.len() * 4);
    for &val in embedding {
        bytes.extend_from_slice(&val.to_le_bytes());
    }
    bytes
}

/// Decode a SQLite BLOB back to f32 slice.
/// Returns an error if the byte length is not divisible by 4.
fn bytes_to_embedding(bytes: &[u8]) -> Result<Vec<f32>, MemoryError> {
    if bytes.len() % 4 != 0 {
        return Err(MemoryError::InvalidEmbedding {
            expected_bytes: bytes.len() - (bytes.len() % 4),
            actual_bytes: bytes.len(),
        });
    }
    let mut embedding = Vec::with_capacity(bytes.len() / 4);
    for chunk in bytes.chunks_exact(4) {
        embedding.push(f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]));
    }
    Ok(embedding)
}
```

### 8.3 FTS Sync Procedures

When inserting a fact:
```rust
fn insert_fact_with_fts(conn: &Connection, fact_id: &str, content: &str, /* ... */) -> Result<()> {
    // 1. Insert into facts table
    conn.execute(
        "INSERT INTO facts (id, namespace, content, embedding, source, metadata) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![...],
    )?;

    // 2. Insert into rowid bridge
    conn.execute("INSERT INTO facts_rowid_map (fact_id) VALUES (?1)", params![fact_id])?;
    let fts_rowid: i64 = conn.last_insert_rowid();

    // 3. Insert into FTS (using the bridge rowid)
    conn.execute(
        "INSERT INTO facts_fts(rowid, content) VALUES (?1, ?2)",
        params![fts_rowid, content],
    )?;

    Ok(())
}
```

When deleting a fact:
```rust
fn delete_fact_with_fts(conn: &Connection, fact_id: &str) -> Result<()> {
    // 1. Get the FTS rowid from bridge
    let fts_rowid: i64 = conn.query_row(
        "SELECT rowid FROM facts_rowid_map WHERE fact_id = ?1",
        params![fact_id],
        |row| row.get(0),
    )?;

    // 2. Get the content (needed for FTS delete on contentless tables)
    let content: String = conn.query_row(
        "SELECT content FROM facts WHERE id = ?1",
        params![fact_id],
        |row| row.get(0),
    )?;

    // 3. Delete from FTS (contentless FTS requires you to supply the original content)
    conn.execute(
        "INSERT INTO facts_fts(facts_fts, rowid, content) VALUES('delete', ?1, ?2)",
        params![fts_rowid, content],
    )?;

    // 4. Delete from bridge
    conn.execute("DELETE FROM facts_rowid_map WHERE fact_id = ?1", params![fact_id])?;

    // 5. Delete from facts
    conn.execute("DELETE FROM facts WHERE id = ?1", params![fact_id])?;

    Ok(())
}
```

**CRITICAL:** Contentless FTS5 delete requires the EXACT original content. If you update a fact's content, you must delete the old FTS entry (with old content), then insert a new one (with new content). Wrap updates in a transaction.

When updating a fact:
```rust
fn update_fact_with_fts(conn: &Connection, fact_id: &str, new_content: &str, new_embedding: &[u8]) -> Result<()> {
    let tx = conn.transaction()?;

    // 1. Get old FTS rowid and content
    let (fts_rowid, old_content): (i64, String) = tx.query_row(
        "SELECT fm.rowid, f.content FROM facts f
         JOIN facts_rowid_map fm ON fm.fact_id = f.id
         WHERE f.id = ?1",
        params![fact_id],
        |row| Ok((row.get(0)?, row.get(1)?)),
    )?;

    // 2. Delete old FTS entry
    tx.execute(
        "INSERT INTO facts_fts(facts_fts, rowid, content) VALUES('delete', ?1, ?2)",
        params![fts_rowid, old_content],
    )?;

    // 3. Update facts table
    tx.execute(
        "UPDATE facts SET content = ?1, embedding = ?2, updated_at = datetime('now') WHERE id = ?3",
        params![new_content, new_embedding, fact_id],
    )?;

    // 4. Insert new FTS entry (reuse same rowid — still valid in the bridge)
    tx.execute(
        "INSERT INTO facts_fts(rowid, content) VALUES (?1, ?2)",
        params![fts_rowid, new_content],
    )?;

    tx.commit()?;
    Ok(())
}
```

---

## 9. Text Chunker

Port from Gloss's `chunk.rs`. Recursive split strategy.

### Algorithm:

1. If text length ≤ `max_size`, return as a single chunk.
2. Try to split on paragraph boundaries (`\n\n`)
3. If paragraphs are still too large, split on sentence boundaries (`. `, `? `, `! `)
4. If sentences are still too large, split on word boundaries (` `)
5. If words are still too large, split at `max_size` character boundary (force split)
6. Merge small adjacent chunks until they reach `target_size`
7. Add `overlap` characters from the end of chunk N to the start of chunk N+1

### Infinite Loop Guard:

Gloss had a bug where recursive_split could loop forever if a chunk couldn't be split further. Guard against this:
```rust
const MAX_RECURSION_DEPTH: usize = 10;

fn recursive_split(text: &str, config: &ChunkingConfig, depth: usize) -> Vec<String> {
    if depth >= MAX_RECURSION_DEPTH {
        tracing::warn!("Chunker hit max recursion depth, force-splitting at max_size");
        return force_split(text, config.max_size);
    }
    // ... normal split logic with depth + 1 passed to recursive calls
}
```

### UTF-8 Safety:

NEVER split in the middle of a UTF-8 character. When force-splitting at a byte offset, use `str::is_char_boundary()` to find the nearest valid split point:

```rust
fn safe_split_at(text: &str, pos: usize) -> usize {
    let mut split = pos.min(text.len());
    while split > 0 && !text.is_char_boundary(split) {
        split -= 1;
    }
    split
}
```

---

## 10. Error Types

```rust
#[derive(Debug, thiserror::Error)]
pub enum MemoryError {
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("Embedding request failed: {0}")]
    EmbeddingRequest(#[from] reqwest::Error),

    #[error("Embedding provider returned {actual} dimensions, expected {expected}")]
    DimensionMismatch { expected: usize, actual: usize },

    #[error("Invalid embedding data: expected {expected_bytes} bytes, got {actual_bytes}")]
    InvalidEmbedding { expected_bytes: usize, actual_bytes: usize },

    #[error("Embedding model mismatch: database has '{stored}', config specifies '{configured}'")]
    ModelMismatch { stored: String, configured: String },

    #[error("Session not found: {0}")]
    SessionNotFound(String),

    #[error("Fact not found: {0}")]
    FactNotFound(String),

    #[error("Document not found: {0}")]
    DocumentNotFound(String),

    #[error("Embedding provider unavailable: {0}")]
    EmbedderUnavailable(String),

    #[error("Migration failed at version {version}: {reason}")]
    MigrationFailed { version: u32, reason: String },

    #[error("{0}")]
    Other(String),
}
```

---

## 11. Thread Safety Model

`MemoryStore` is `Clone + Send + Sync`. Internally it holds an `Arc<Inner>` with:

- `Mutex<rusqlite::Connection>` — rusqlite's `Connection` is NOT `Send`. All database access goes through a Mutex. This is fine for a single-user agent. If you ever need concurrent reads, swap to a connection pool (r2d2-sqlite), but don't prematurely optimize.
- `Box<dyn Embedder>` — behind the Arc, shared read-only after construction.
- `MemoryConfig` — immutable after construction.

**async boundary:** The `search()`, `add_fact()`, `ingest_document()`, `embed()`, and `embed_batch()` methods are `async` because they call the embedding provider over HTTP. Database operations are synchronous (rusqlite is sync). The pattern is:

```rust
pub async fn add_fact(&self, ...) -> Result<String, MemoryError> {
    // 1. Embed (async — HTTP call to Ollama)
    let embedding = self.inner.embedder.embed(content).await?;

    // 2. Store (sync — SQLite, behind Mutex)
    let conn = self.inner.conn.lock().unwrap();
    insert_fact_with_fts(&conn, &fact_id, content, &embedding_to_bytes(&embedding), ...)?;

    Ok(fact_id)
}
```

If Ollama is unavailable, async methods return `MemoryError::EmbeddingRequest` or `MemoryError::EmbedderUnavailable`. The sync methods (`search_fts_only`, `get_recent_messages`, etc.) always work regardless of Ollama status.

---

## 12. Initialization Sequence

When `MemoryStore::open(config)` is called:

1. **Open SQLite connection** at `config.database_path`. Create parent directories if needed. Create file if it doesn't exist.
2. **Configure pragmas:**
   ```sql
   PRAGMA journal_mode = WAL;
   PRAGMA foreign_keys = ON;
   PRAGMA busy_timeout = 5000;
   PRAGMA synchronous = NORMAL;  -- WAL mode makes this safe
   ```
3. **Run migrations.** Check `_schema_version` table. Apply any unapplied migrations in order.
4. **Check embedding metadata.** If `embedding_metadata` has a row:
   - If `model_name` != `config.embedding.model` OR `dimensions` != `config.embedding.dimensions`:
     - Log a WARNING: "Embedding model changed from X to Y. Existing embeddings are invalid. Call reembed_all() to re-embed."
     - Update the metadata row.
     - Do NOT block startup. The crate works with stale embeddings — search quality degrades but doesn't crash.
   - If no row exists (fresh database): insert the current model info.
5. **Return `MemoryStore`.**

---

## 13. Testing Strategy

### Unit tests (no Ollama required):
- All database operations use `MockEmbedder` and `tempfile::NamedTempFile` for the SQLite path.
- Chunker tests: various text sizes, Unicode edge cases, overlap correctness.
- FTS sync: insert → search → update → search → delete → search (verify no stale results).
- BLOB encoding roundtrip: random f32 vectors → bytes → back.
- BM25 ranking: insert 5 docs with known term frequencies, assert rank order.
- Cosine similarity: known vectors with known similarities (orthogonal, identical, opposite).
- RRF fusion: two ranked lists with known overlap → verify merged order.
- Token budget: messages with known token counts → verify correct window.
- Conversation: insert messages across sessions, verify isolation.
- FTS query sanitization: special characters, empty input, Unicode, SQL injection attempts.

### Integration tests (require Ollama):
- Gated behind `#[cfg(feature = "integration-tests")]` or `#[ignore]` attribute.
- Embed real text, store, search, verify semantic relevance.
- Batch embedding: verify 100 texts embedded correctly.
- Document ingestion end-to-end: chunk → embed → store → search.

### Example test:

```rust
#[test]
fn test_rrf_fusion() {
    // BM25 results: [A(rank 1), B(rank 2), C(rank 3)]
    // Vector results: [B(rank 1), D(rank 2), A(rank 3)]
    // With k=60, weights=1.0:
    //   A: 1/61 + 1/63 = 0.03226
    //   B: 1/62 + 1/61 = 0.03252  <-- highest
    //   C: 1/63 + 0    = 0.01587
    //   D: 0    + 1/62 = 0.01613
    // Expected order: B, A, D, C

    let bm25_results = vec!["A", "B", "C"];
    let vector_results = vec!["B", "D", "A"];
    let fused = rrf_fuse(&bm25_results, &vector_results, 60.0, 1.0, 1.0);
    assert_eq!(fused, vec!["B", "A", "D", "C"]);
}
```

---

## 14. Performance Expectations

| Operation | Expected Latency | Notes |
|-----------|-----------------|-------|
| `add_message` | <1ms | Single INSERT, no embedding |
| `get_recent_messages(20)` | <1ms | Indexed query |
| `add_fact` | 50-200ms | Dominated by Ollama embedding HTTP call |
| `ingest_document` (10 pages) | 2-5s | ~20 chunks × embedding time + batch INSERT |
| `search` (hybrid, 10K facts) | 100-300ms | ~80ms embed + ~5ms vector scan + ~10ms FTS + merge |
| `search_fts_only` | <10ms | No embedding, pure SQLite FTS5 |
| Database size per 1K facts | ~3-4 MB | ~3KB embedding + ~500B text + index overhead |

---

## 15. Future Extensions (NOT in V1)

These are explicitly out of scope for the initial build but should be kept in mind so the design doesn't paint us into a corner:

- **Reranker pass:** After RRF fusion, re-rank top-30 candidates with a cross-encoder model. The `SearchConfig` could gain a `reranker: Option<Box<dyn Reranker>>` field.
- **Conversation summarization:** Periodically summarize old messages into a single summary message to compress context windows. The conversation API has `metadata` fields ready for this.
- **Automatic fact extraction:** After each conversation, extract factual claims and store them. This is LLM-level work — the agent calls `add_fact()`, the crate doesn't do extraction itself.
- **Namespace-scoped embedding models:** Different namespaces using different embedding models. The current design uses one model for everything, which is correct for V1.
- **Connection pooling:** If concurrent access becomes a bottleneck, swap `Mutex<Connection>` for `r2d2::Pool<SqliteConnectionManager>`. The public API doesn't change.
- **Embedding cache:** Cache recent embeddings to avoid re-embedding the same query. Simple LRU with the query text as key.

---

## 16. Build Checklist

Implementation order within the crate:

1. **`error.rs`** — Define all error types first. Everything depends on these.
2. **`config.rs`** — Config structs with `Default` implementations.
3. **`types.rs`** — All shared types (Role, Session, Message, Fact, SearchResult, etc.)
4. **`db.rs`** — Open database, configure pragmas, run migrations. Test: open, close, reopen — schema is intact.
5. **`embedder.rs`** — Embedder trait + MockEmbedder. Test: mock returns consistent dimensions.
6. **`chunker.rs`** — Text chunking. Test: all the edge cases. No dependencies on db or embedder.
7. **`conversation.rs`** — Session + message CRUD. Test with real SQLite (tempfile), no embedder needed.
8. **`knowledge.rs`** — Fact CRUD + FTS sync. Test: insert, search FTS, update, delete, verify FTS consistency.
9. **`search.rs`** — The big one. BM25 retrieval, vector retrieval, cosine similarity, RRF fusion. Test each piece independently, then the full pipeline.
10. **`lib.rs`** — Wire up `MemoryStore` as the public facade. Integration tests.
11. **OllamaEmbedder** — Implement after everything else works with MockEmbedder. This is the last piece because it requires a running Ollama instance to test.
12. **Examples** — `basic_search.rs`, `conversation_memory.rs`.

---

## 17. Reference Material

Copy these files from Gloss into `reference/` before starting implementation:

- `src-tauri/src/retrieval/hybrid_search.rs` → `reference/hybrid_search.rs`
  - Contains: RRF fusion algorithm, BM25 score normalization, cosine similarity computation, candidate merging logic
  - Port the algorithm; rewrite the data access layer (HNSW → brute-force BLOB scan, fastembed → Ollama HTTP)

- `src-tauri/src/ingestion/chunk.rs` → `reference/chunk.rs`
  - Contains: Recursive text splitting, paragraph/sentence/word boundary detection, overlap application, merge small chunks
  - Port mostly as-is; add the infinite loop guard and UTF-8 safety from the crash fix analysis

Do NOT copy `embed.rs` from Gloss — the embedding approach is completely different (fastembed/ONNX vs Ollama HTTP). Build `embedder.rs` fresh.
