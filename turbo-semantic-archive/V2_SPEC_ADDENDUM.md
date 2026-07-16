# V2 Spec Addendum — semantic-memory

This document specifies additions and changes introduced in V2. Where this document and `SPEC.md` or `V1_1_SPEC_ADDENDUM.md` conflict, this document wins. All section references (e.g. "§4") refer to the original `SPEC.md`.

---

## B1. Stable Rust Compatibility (supplements §2)

### B1.1 Nightly-Only API Removal

Replace all uses of unstable standard library methods with stable equivalents:

| Nightly | Stable Replacement | Location |
|---------|--------------------|----------|
| `usize::is_multiple_of(n)` | `x % n == 0` | `db.rs::bytes_to_embedding` |

The crate's MSRV (minimum supported Rust version) is **1.75** (for native async traits in future iterations). All code must compile on stable Rust ≥ 1.75.

---

## B2. Async-Safe Database Access (supplements §5 and all async methods)

### B2.1 Problem

SQLite operations via rusqlite are synchronous. Currently, they run inline on tokio executor threads while holding a `Mutex<Connection>`. Under concurrent agent workloads, this blocks the async runtime.

### B2.2 Solution: `spawn_blocking` Wrapper

All database access goes through a helper method that moves the work to tokio's blocking thread pool:

```rust
async fn with_conn<F, T>(&self, f: F) -> Result<T, MemoryError>
where
    F: FnOnce(&rusqlite::Connection) -> Result<T, MemoryError> + Send + 'static,
    T: Send + 'static,
```

This guarantees:
- The Mutex is only held on a blocking thread, never on an async executor thread
- The `Connection` never crosses an await point
- All database I/O is off the executor

### B2.3 API Surface Change

All previously-sync public methods on `MemoryStore` become `async fn`. This is a **breaking change** acceptable at v0.1 (pre-1.0 semver).

| Method | V1.1 | V2 |
|--------|------|----|
| `create_session` | sync | async |
| `list_sessions` | sync | async |
| `delete_session` | sync | async |
| `add_message` | sync | async |
| `get_recent_messages` | sync | async |
| `get_messages_within_budget` | sync | async |
| `session_token_count` | sync | async |
| `delete_fact` | sync | async |
| `delete_namespace` | sync | async |
| `get_fact` | sync | async |
| `list_facts` | sync | async |
| `add_fact_with_embedding` | sync | async |
| `stats` | sync | async |
| `vacuum` | sync | async |
| `embeddings_are_dirty` (new) | — | async |
| `chunk_text` | sync | **stays sync** (no DB) |
| `embed` | async | async (no change) |
| `embed_batch` | async | async (no change) |
| `open` | sync | **stays sync** (initialization) |
| `open_with_embedder` | sync | **stays sync** (initialization) |

`open` and `open_with_embedder` stay sync because they run once at startup, not during request processing.

`chunk_text` stays sync because it doesn't touch the database.

### B2.4 Closure Capture Pattern

Because `with_conn` requires `F: Send + 'static`, closures cannot borrow from `&self`. Values must be cloned/moved into the closure:

```rust
// WRONG — borrows &self across spawn_blocking
pub async fn get_fact(&self, fact_id: &str) -> Result<Option<Fact>, MemoryError> {
    self.with_conn(|conn| knowledge::get_fact(conn, fact_id)).await
    //                                                ^^^^^^^ borrowed
}

// RIGHT — clone into owned value
pub async fn get_fact(&self, fact_id: &str) -> Result<Option<Fact>, MemoryError> {
    let fact_id = fact_id.to_string();
    self.with_conn(move |conn| knowledge::get_fact(conn, &fact_id)).await
}
```

This pattern applies to every method. String parameters become owned. Slice parameters become `Vec`. Config values are cloned from `Arc<Inner>`.

---

## B3. Robust Embedding Response Parsing (supplements Embedder §)

### B3.1 Non-Numeric Value Handling

The Ollama embed response parser must reject non-numeric values explicitly instead of coercing to 0.0.

Error format:
```
MemoryError::Other("Embedding contains non-numeric value: <json_value>")
```

### B3.2 HTTP Status Validation

Before parsing the response body as JSON, check the HTTP status:
- 2xx → proceed
- 404 → `MemoryError::EmbedderUnavailable("Model 'X' not available. Run: ollama pull X")`
- Other → `MemoryError::Other("Ollama returned HTTP <status>: <body>")`

---

## B4. Message Re-embedding (supplements `reembed_all`)

`reembed_all()` gains a third pass after facts and chunks:

1. SELECT all messages with `embedding IS NOT NULL`
2. Re-embed each message's content
3. UPDATE the embedding BLOB

Only messages previously embedded via `add_message_embedded` are re-embedded. Messages added via `add_message` (no embedding) are skipped.

The return count includes re-embedded messages. The final tracing log reports:
```
tracing::info!(facts = fact_count, chunks = chunk_count, messages = msg_count, total = count, "Re-embedding complete");
```

---

## B5. Embedding Staleness Tracking (supplements §4 and `check_embedding_metadata`)

### B5.1 Schema V3 Migration

```sql
-- V3: Embedding staleness tracking
ALTER TABLE embedding_metadata ADD COLUMN embeddings_dirty INTEGER NOT NULL DEFAULT 0;
```

### B5.2 Dirty Flag Lifecycle

| Event | Action |
|-------|--------|
| `check_embedding_metadata` detects model/dims mismatch | Set `embeddings_dirty = 1` |
| `reembed_all()` completes successfully | Set `embeddings_dirty = 0` |
| `hybrid_search` or `vector_only_search` called while dirty | Log `tracing::warn!` per call |

### B5.3 Public Query Method

```rust
/// Check if embeddings need re-generation after a model change.
pub async fn embeddings_are_dirty(&self) -> Result<bool, MemoryError>
```

Returns `true` if `embeddings_dirty = 1`, `false` otherwise. Returns `false` if the metadata row doesn't exist yet (fresh database).

---

## B6. Zero-Allocation Vector Search (supplements §8.1 vector scan)

### B6.1 Buffer Reuse Strategy

Instead of allocating a `Vec<f32>` per row in `vector_search`, reuse a single pre-allocated buffer:

```rust
let mut decode_buf: Vec<f32> = Vec::with_capacity(expected_dimensions);

for each row {
    let bytes: Vec<u8> = row.get(embedding_column)?;
    decode_buf.clear();
    for chunk in bytes.chunks_exact(4) {
        decode_buf.push(f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]));
    }
    let sim = cosine_similarity(&query_embedding, &decode_buf);
    // ... collect if above threshold
}
```

This eliminates O(N) allocations during vector search. The single buffer is reused across all rows.

### B6.2 bytemuck Dependency

Added for potential future zero-copy decode paths. In V2, used only if alignment allows:
```toml
bytemuck = { version = "1", features = ["derive"] }
```

The buffer reuse approach is the primary optimization. `bytemuck::try_cast_slice` is available as an optional fast path when the byte slice happens to be 4-byte aligned.

---

## B7. Pluggable Token Counting (new module)

### B7.1 Trait

```rust
pub trait TokenCounter: Send + Sync {
    fn count_tokens(&self, text: &str) -> usize;
}
```

### B7.2 Default Implementation

```rust
pub struct EstimateTokenCounter;
impl TokenCounter for EstimateTokenCounter {
    fn count_tokens(&self, text: &str) -> usize {
        // ~4 chars per token for English prose
        text.len() / 4
    }
}
```

### B7.3 Integration Points

| Location | Current | V2 |
|----------|---------|----|
| `chunker.rs` `token_count_estimate` | `text.len() / 4` | `token_counter.count_tokens(text)` |
| `add_message` when `token_count` is None | Stored as NULL | Auto-computed via token counter |
| `add_message_embedded` when `token_count` is None | Stored as NULL | Auto-computed via token counter |

### B7.4 Configuration

```rust
pub struct MemoryConfig {
    // ... existing ...
    #[serde(skip)]
    pub token_counter: Option<Arc<dyn TokenCounter>>,
}
```

`serde(skip)` because trait objects aren't serializable. Set programmatically, not from config files.

---

## B8. Vector Search Row Count Warning (supplements §8.1)

When the total number of rows to scan in `vector_search` exceeds 50,000, emit:

```
tracing::warn!(row_count = N, "Vector search scanning {} rows — latency will be degraded. Consider namespace partitioning or pruning.", N);
```

This is advisory only. The search still completes. The threshold is a `const` in `search.rs`, not configurable (it's a diagnostic aid, not a tuning knob).

---

## B9. Standard Trait Implementations (supplements types)

`Role` gains:
- `impl Display for Role` — delegates to `as_str()`
- `impl FromStr for Role` — delegates to `from_str_value()`, returns `MemoryError::Other` on unknown

These are additive. Existing `as_str()` and `from_str_value()` remain unchanged.
