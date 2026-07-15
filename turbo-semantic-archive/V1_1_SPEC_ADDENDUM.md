# V1.1 Spec Addendum — semantic-memory

This document specifies additions and changes to the V1 spec (`SPEC.md`). Where this document and `SPEC.md` conflict, this document wins. All section references (e.g. "§4.2") refer to the original `SPEC.md`.

---

## A1. Schema V2 Migration (supplements §4)

Add to `db.rs` as `MIGRATION_V2`:

```sql
-- V2: Message embeddings for conversation search
ALTER TABLE messages ADD COLUMN embedding BLOB;

CREATE TABLE messages_rowid_map (
    rowid       INTEGER PRIMARY KEY AUTOINCREMENT,
    message_id  INTEGER NOT NULL UNIQUE REFERENCES messages(id) ON DELETE CASCADE
);

CREATE VIRTUAL TABLE messages_fts USING fts5(
    content,
    content='',
    content_rowid='rowid',
    tokenize='porter unicode61'
);
```

The migration runner applies V2 if `current_version < 2`. Same transaction + version-recording pattern as V1.

Existing databases upgrade transparently — `ALTER TABLE ADD COLUMN` preserves all existing data, and the new column defaults to NULL (no embedding).

---

## A2. Conversation Semantic Search (supplements §5.3 and §8)

### A2.1 New Types

Add to `SearchSourceType`:
```rust
pub enum SearchSourceType {
    Facts,
    Chunks,
    Messages,  // NEW
}
```

Add to `SearchSource`:
```rust
pub enum SearchSource {
    Fact { fact_id: String, namespace: String },
    Chunk { chunk_id: String, document_id: String, document_title: String, chunk_index: usize },
    Message { message_id: i64, session_id: String, role: String },  // NEW
}
```

### A2.2 Message Embedding Pipeline

Two paths for adding messages:

| Method | Sync/Async | Embeds? | FTS? | Use Case |
|--------|-----------|---------|------|----------|
| `add_message` (existing) | Sync | No | No | Fast logging, Ollama-independent |
| `add_message_embedded` (new) | Async | Yes | Yes | When conversation search is needed |

`add_message_embedded` transaction sequence:
1. Embed content (async, outside lock)
2. Lock mutex
3. INSERT into messages (with embedding BLOB)
4. INSERT into messages_rowid_map
5. Get last_insert_rowid
6. INSERT into messages_fts(rowid, content)
7. UPDATE sessions SET updated_at
8. Commit

### A2.3 Conversation Search

New `MemoryStore` method:
```rust
pub async fn search_conversations(
    &self,
    query: &str,
    top_k: Option<usize>,
    session_ids: Option<&[&str]>,
) -> Result<Vec<SearchResult>, MemoryError>
```

This performs hybrid search (BM25 + vector + RRF) over **messages only**. The `session_ids` parameter filters to specific sessions (parameterized, not string-interpolated). If None, searches all sessions.

Only messages that were added via `add_message_embedded` (i.e., have non-NULL embedding and FTS entries) are searchable. Messages added via `add_message` are invisible to search.

### A2.4 Including Messages in Global Search

The existing `search()` method does NOT include messages by default. To search across facts, chunks, AND messages, pass `source_types: Some(&[SearchSourceType::Facts, SearchSourceType::Chunks, SearchSourceType::Messages])`.

---

## A3. Recency-Weighted Scoring (supplements §8.1)

### A3.1 Configuration

New fields on `SearchConfig`:

```rust
/// Half-life for recency decay in days. None = disabled (default).
pub recency_half_life_days: Option<f64>,

/// Weight of recency boost in RRF scoring. Default: 0.5.
pub recency_weight: f64,
```

Defaults: `recency_half_life_days: None`, `recency_weight: 0.5`.

### A3.2 Scoring Formula

When `recency_half_life_days` is `Some(h)`:

```
age_days = (now_utc - item_updated_at).as_fractional_days()
decay = 2^(-age_days / h)
recency_score = recency_weight * decay / (rrf_k + 1)

final_score = bm25_rrf_score + vector_rrf_score + recency_score
```

The `/ (rrf_k + 1)` normalization ensures the recency term is on the same scale as the maximum possible RRF score from a single retriever (which occurs at rank 1: `weight / (k + 1)`).

**Edge cases:**
- `h = 0` or `h < 0` → treat as disabled (no recency boost), log `tracing::warn!`
- `age_days < 0` (future timestamp) → clamp to 0 (full boost)
- Missing timestamp (None) → no recency boost for that candidate

### A3.3 Timestamp Propagation

BM25 and vector hit structs gain `updated_at: Option<String>`. The SQL queries SELECT:
- Facts: `f.updated_at`
- Chunks: `c.created_at`
- Messages: `m.created_at`

The `RrfCandidate` struct carries the timestamp and uses it during scoring.

### A3.4 Test Case

Two facts with identical content relevance, half_life = 30 days:

| Fact | Age | BM25 Rank | Vector Rank | Expected |
|------|-----|-----------|-------------|----------|
| A (recent) | 0 days | 1 | 1 | Higher score |
| B (old) | 60 days | 2 | 2 | Lower score |

Fact A recency: `0.5 * 2^(0/30) / 61 = 0.5 * 1.0 / 61 ≈ 0.00820`
Fact B recency: `0.5 * 2^(-60/30) / 61 = 0.5 * 0.25 / 61 ≈ 0.00205`

The recency term should be enough to differentiate when combined with the already-favorable RRF ranking.

---

## A4. Parameterized Namespace Filtering (supplements §8.1)

### Current (BROKEN)

```rust
fn namespace_filter_sql(column: &str, namespaces: Option<&[&str]>) -> String {
    // String interpolation with manual quote escaping — SQL injection risk
}
```

### Replacement

New function signature:
```rust
fn build_namespace_clause(
    column: &str,
    namespaces: Option<&[&str]>,
    param_offset: usize,
) -> (String, Vec<String>)
```

Returns a tuple of:
1. SQL fragment like `AND column IN (?3, ?4, ?5)` where parameter numbers start at `param_offset`
2. Vec of namespace values to bind

If `namespaces` is None or empty, returns `("".to_string(), vec![])`.

**Callers** (`bm25_search`, `vector_search`) must append the returned parameter values to their `params![]` call. Since rusqlite's `params![]` macro is compile-time, you'll likely need to switch to `rusqlite::params_from_iter` or build a `Vec<Box<dyn rusqlite::types::ToSql>>` for the dynamic parameter list.

---

## A5. Content Deduplication (supplements §8.1)

After RRF fusion produces the final sorted + truncated result list, apply content-based deduplication:

1. Normalize each result's content: `content.trim().to_lowercase()`
2. Track seen content in a `HashSet<String>`
3. Drop results whose normalized content was already seen
4. Keep the first (highest-scored) occurrence

This is applied in `hybrid_search`, `fts_only_search`, and `vector_only_search` — all three search entry points.

---

## A6. Brute-Force Vector Search Performance Bounds (supplements §8.1)

### Performance Model

The vector search scans ALL embeddings matching the namespace filter, computes cosine similarity for each, and keeps the top candidates. This is O(N × D) where N is the number of embeddings and D is the dimension count.

**Measured bounds (approximate, single-threaded, 768 dimensions):**

| N vectors | Scan time | Acceptable? |
|-----------|-----------|-------------|
| 1,000 | <1ms | Yes |
| 10,000 | ~5ms | Yes |
| 50,000 | ~25ms | Yes |
| 100,000 | ~50ms | Borderline |
| 500,000 | ~250ms | No — agent response feels sluggish |

### When to Worry

If any single namespace exceeds 100K vectors, search latency becomes noticeable. Mitigation strategies (none implemented in V1, listed for future reference):

1. **Namespace partitioning:** Split large namespaces into sub-namespaces. Search only the relevant sub-namespace.
2. **ANN index:** Add an HNSW or IVF index. The `usearch` crate provides a Rust-native HNSW implementation. This would require a separate index file alongside the SQLite database.
3. **Pre-filtering:** Add metadata columns (e.g., date range, category) and filter in SQL before loading BLOBs. Reduces N without changing the algorithm.
4. **Quantization:** Store int8 quantized embeddings (768 bytes vs 3072) and use dot-product approximation for the initial scan. Re-score top candidates with full f32.

For the target use case (single-user agent, <50K facts/chunks accumulated over months of use), brute-force is the right choice. The simplicity of "it's all in one SQLite file" outweighs the performance cost that only manifests at scales this crate won't reach in normal use.

---

## A7. TLS Support (supplements §2)

Change reqwest dependency:

```toml
# Before
reqwest = { version = "0.12", features = ["json"], default-features = false }

# After
reqwest = { version = "0.12", features = ["json", "rustls-tls"], default-features = false }
```

This enables HTTPS connections to remote embedding endpoints without adding an OpenSSL build dependency. `rustls` compiles from source and works on all platforms.

---

## A8. Updated "Things That Will Bite You" (supplements CLAUDE.md)

Add as item 8:

> **Vector search is brute-force O(N×D).** Fast at 10K vectors, fine at 50K, sluggish at 100K+. If you're accumulating facts/chunks without ever pruning, monitor the count. See SPEC Addendum §A6 for the full performance model and mitigation strategies.

Add as item 9:

> **Message embedding is opt-in.** `add_message` (sync, no embedding) and `add_message_embedded` (async, embeds + FTS) are separate methods. If you call `add_message` and later wonder why `search_conversations` finds nothing — that's why. You have to choose at insert time.
