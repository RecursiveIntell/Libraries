# V2 Agents Addendum — semantic-memory

Module-specific implementation guidance for the V2 patch. Read the relevant section before touching each file.

---

## Agent: db.rs V2

**Fixes you're implementing:** Fix 1 (is_multiple_of), Fix 5 (dirty flag), Fix 10a (safety comments)

### Fix 1: One-Liner

```rust
// BEFORE (nightly-only, won't compile on stable)
if !bytes.len().is_multiple_of(4) {

// AFTER (stable Rust)
if bytes.len() % 4 != 0 {
```

That's it. Don't touch anything else in `bytes_to_embedding`.

### Fix 5: V3 Migration

Add `MIGRATION_V3` constant:
```rust
const MIGRATION_V3: &str = r#"
ALTER TABLE embedding_metadata ADD COLUMN embeddings_dirty INTEGER NOT NULL DEFAULT 0;
"#;
```

Add to `run_migrations()` after the V2 block — same pattern:
```rust
let current_version: u32 = conn.query_row(
    "SELECT COALESCE(MAX(version), 0) FROM _schema_version", [], |row| row.get(0)
).unwrap_or(0);

if current_version < 3 {
    let tx = conn.unchecked_transaction()...;
    tx.execute_batch(MIGRATION_V3)...;
    tx.execute("INSERT INTO _schema_version (version) VALUES (?1)", params![3u32])...;
    tx.commit()...;
    tracing::info!("Applied migration V3");
}
```

### Fix 5: check_embedding_metadata Changes

When model/dims mismatch is detected, after updating the row, also set the dirty flag:
```rust
conn.execute(
    "UPDATE embedding_metadata SET model_name = ?1, dimensions = ?2, \
     embeddings_dirty = 1, updated_at = datetime('now') WHERE id = 1",
    params![config.model, config.dimensions],
)?;
```

Add a new public function:
```rust
/// Check if embeddings are stale after a model change.
pub fn is_embeddings_dirty(conn: &Connection) -> Result<bool, MemoryError> {
    let dirty: i32 = conn.query_row(
        "SELECT COALESCE(embeddings_dirty, 0) FROM embedding_metadata WHERE id = 1",
        [],
        |row| row.get(0),
    ).unwrap_or(0);
    Ok(dirty != 0)
}

/// Clear the dirty flag after re-embedding.
pub fn clear_embeddings_dirty(conn: &Connection) -> Result<(), MemoryError> {
    conn.execute(
        "UPDATE embedding_metadata SET embeddings_dirty = 0 WHERE id = 1",
        [],
    )?;
    Ok(())
}
```

### Fix 10a: Safety Comments

Every `unchecked_transaction()` call in db.rs (there are 2 — in `run_migrations`). Add before each:
```rust
// SAFETY: We hold &Connection (not &mut) via Mutex::lock(). unchecked_transaction()
// is required because transaction() needs &mut self. The Mutex serializes all access,
// preventing concurrent transaction nesting.
```

---

## Agent: embedder.rs V2

**Fixes you're implementing:** Fix 3 (silent coercion)

### Response Status Check

Add immediately after the `.send().await?` call, before `.json().await`:

```rust
if response.status() == reqwest::StatusCode::NOT_FOUND {
    return Err(MemoryError::EmbedderUnavailable(format!(
        "Model '{}' not available in Ollama. Run: ollama pull {}",
        self.model, self.model
    )));
}

if !response.status().is_success() {
    let status = response.status();
    let body = response.text().await.unwrap_or_default();
    return Err(MemoryError::Other(format!(
        "Ollama returned HTTP {}: {}",
        status,
        &body[..body.len().min(500)] // Truncate huge error bodies
    )));
}
```

### Strict Numeric Parsing

Replace the inner embedding parse loop:

```rust
// BEFORE (silent coercion)
let embedding: Vec<f32> = embedding_val
    .as_array()
    .ok_or_else(|| MemoryError::Other("Embedding is not an array".to_string()))?
    .iter()
    .map(|v| v.as_f64().unwrap_or(0.0) as f32)
    .collect();

// AFTER (explicit error)
let raw_array = embedding_val
    .as_array()
    .ok_or_else(|| MemoryError::Other("Embedding is not an array".to_string()))?;

let mut embedding = Vec::with_capacity(raw_array.len());
for (i, v) in raw_array.iter().enumerate() {
    let val = v.as_f64().ok_or_else(|| {
        MemoryError::Other(format!(
            "Embedding dimension {} contains non-numeric value: {}",
            i, v
        ))
    })?;
    embedding.push(val as f32);
}
```

The `for` loop with indexed error reporting is more debuggable than a chained `.map().collect()`.

---

## Agent: types.rs V2

**Fixes you're implementing:** Fix 7 (Display/FromStr for Role)

Add after the existing `impl Role` block:

```rust
impl std::fmt::Display for Role {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for Role {
    type Err = MemoryError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_str_value(s).ok_or_else(|| {
            MemoryError::Other(format!("Unknown role: '{}'", s))
        })
    }
}
```

**Requires adding** `use crate::error::MemoryError;` at the top of types.rs.

---

## Agent: search.rs V2

**Fixes you're implementing:** Fix 6 (buffer reuse), Fix 9 (row count guard)

### Fix 6: Buffer Reuse in vector_search

The current `vector_search` function loads each row's embedding, decodes to a fresh `Vec<f32>`, computes cosine, and discards. Replace with buffer reuse.

**Before (per-row allocation):**
```rust
let stored_embedding = bytes_to_embedding(&embedding_bytes)?;
let similarity = cosine_similarity(&query_embedding, &stored_embedding) as f64;
```

**After (buffer reuse):**
```rust
// Allocate once before the loop, at the expected dimension size
let expected_dims = query_embedding.len();
let mut decode_buf: Vec<f32> = Vec::with_capacity(expected_dims);

// Inside the row loop:
decode_buf.clear();
if embedding_bytes.len() % 4 != 0 {
    tracing::warn!("Skipping row with invalid embedding length: {}", embedding_bytes.len());
    continue;
}
for chunk in embedding_bytes.chunks_exact(4) {
    decode_buf.push(f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]));
}
if decode_buf.len() != expected_dims {
    tracing::warn!(
        expected = expected_dims, actual = decode_buf.len(),
        "Skipping row with wrong embedding dimensions"
    );
    continue;
}
let similarity = cosine_similarity(query_embedding, &decode_buf) as f64;
```

**Key points:**
- `decode_buf.clear()` resets length to 0 but keeps the allocation
- `chunks_exact(4)` is safe because we checked `% 4 == 0`
- Dimension mismatch rows are skipped with a warning, not errors (graceful degradation)
- This applies to ALL three vector scan loops (facts, chunks, messages)

### Fix 9: Row Count Warning

Add a counter before each scan loop and check after:

```rust
const VECTOR_SCAN_WARN_THRESHOLD: usize = 50_000;

// After collecting all hits from facts + chunks + messages:
let total_rows_scanned = fact_row_count + chunk_row_count + message_row_count;
if total_rows_scanned > VECTOR_SCAN_WARN_THRESHOLD {
    tracing::warn!(
        rows_scanned = total_rows_scanned,
        threshold = VECTOR_SCAN_WARN_THRESHOLD,
        "Vector search scanned {} rows — latency may be degraded. \
         Consider namespace partitioning or pruning old data.",
        total_rows_scanned
    );
}
```

Alternatively, count per-table and warn per-table — that's more actionable:
```rust
if fact_count > VECTOR_SCAN_WARN_THRESHOLD {
    tracing::warn!(count = fact_count, "facts table exceeds scan threshold");
}
// ... same for chunks, messages
```

**Use the per-table approach.** It tells the user *which* table is the problem.

---

## Agent: lib.rs V2

**Fixes you're implementing:** Fix 2 (spawn_blocking), Fix 4 (reembed messages), Fix 5 (dirty warning in search)

### Fix 2: The `with_conn` Helper

Add as a private method on `MemoryStore`:

```rust
/// Run a closure that needs the database connection on a blocking thread.
///
/// This prevents SQLite I/O from stalling the tokio executor. The closure
/// receives a reference to the Connection (already locked via Mutex).
async fn with_conn<F, T>(&self, f: F) -> Result<T, MemoryError>
where
    F: FnOnce(&rusqlite::Connection) -> Result<T, MemoryError> + Send + 'static,
    T: Send + 'static,
{
    let inner = self.inner.clone();
    tokio::task::spawn_blocking(move || {
        let conn = inner.conn.lock().expect("mutex poisoned");
        f(&conn)
    })
    .await
    .map_err(|e| MemoryError::Other(format!("Blocking task panicked: {}", e)))?
}
```

### Fix 2: Method Migration Table

Every public method follows one of three patterns:

**Pattern A — Pure DB, no embedding (was sync, now async):**
```rust
// BEFORE
pub fn create_session(&self, channel: &str) -> Result<String, MemoryError> {
    let conn = self.inner.conn.lock().expect("mutex poisoned");
    conversation::create_session(&conn, channel, None)
}

// AFTER
pub async fn create_session(&self, channel: &str) -> Result<String, MemoryError> {
    self.with_conn(move |conn| {
        conversation::create_session(conn, channel, None)
    }).await
}
```

**Wait** — `channel: &str` is a borrow. The closure needs `'static`. Fix:
```rust
pub async fn create_session(&self, channel: &str) -> Result<String, MemoryError> {
    let channel = channel.to_string();
    self.with_conn(move |conn| {
        conversation::create_session(conn, &channel, None)
    }).await
}
```

**Pattern B — Embed (async) then store (DB):**
```rust
// BEFORE
pub async fn add_fact(&self, namespace: &str, content: &str, ...) -> Result<String, MemoryError> {
    let embedding = self.inner.embedder.embed(content).await?;
    let embedding_bytes = db::embedding_to_bytes(&embedding);
    let fact_id = uuid::Uuid::new_v4().to_string();
    let conn = self.inner.conn.lock().expect("mutex poisoned");
    knowledge::insert_fact_with_fts(&conn, ...)?;
    Ok(fact_id)
}

// AFTER
pub async fn add_fact(&self, namespace: &str, content: &str, ...) -> Result<String, MemoryError> {
    // Step 1: Embed (async, on executor — this is fine, it's network I/O)
    let embedding = self.inner.embedder.embed(content).await?;
    let embedding_bytes = db::embedding_to_bytes(&embedding);
    let fact_id = uuid::Uuid::new_v4().to_string();

    // Step 2: Store (blocking, off executor)
    let ns = namespace.to_string();
    let ct = content.to_string();
    let fid = fact_id.clone();
    let src = source.map(|s| s.to_string());
    let meta = metadata.clone();
    self.with_conn(move |conn| {
        knowledge::insert_fact_with_fts(conn, &fid, &ns, &ct, &embedding_bytes, src.as_deref(), meta.as_ref())
    }).await?;

    Ok(fact_id)
}
```

**Pattern C — Pure search (embed query, then scan DB):**

Same as Pattern B but the DB step is the search:
```rust
pub async fn search(&self, query: &str, ...) -> Result<Vec<SearchResult>, MemoryError> {
    let query_embedding = self.inner.embedder.embed(query).await?;
    let q = query.to_string();
    let config = self.inner.config.search.clone();
    // ... clone all filter params to owned ...
    self.with_conn(move |conn| {
        search::hybrid_search(conn, &q, &query_embedding, &config, k, ...)
    }).await
}
```

**Apply Pattern A to:** `create_session`, `list_sessions`, `delete_session`, `add_message`, `get_recent_messages`, `get_messages_within_budget`, `session_token_count`, `delete_fact`, `delete_namespace`, `get_fact`, `list_facts`, `add_fact_with_embedding`, `stats`, `vacuum`, `raw_execute`, `embeddings_are_dirty`

**Apply Pattern B to:** `add_fact`, `update_fact`, `add_message_embedded`, `ingest_document`

**Apply Pattern C to:** `search`, `search_fts_only` (no embed needed — still use `with_conn` for DB), `search_vector_only`, `search_conversations`

**`search_fts_only` note:** This doesn't need embedding but still does DB work. Convert to async + `with_conn`:
```rust
pub async fn search_fts_only(&self, ...) -> Result<Vec<SearchResult>, MemoryError> {
    let q = query.to_string();
    let config = self.inner.config.search.clone();
    // ... clone filters ...
    self.with_conn(move |conn| {
        search::fts_only_search(conn, &q, &config, k, ...)
    }).await
}
```

### Fix 2: Config Cloning

`SearchConfig` is already `Clone` (derives it). For the `with_conn` closures, clone it from `self.inner.config.search.clone()`. This is a small struct, cloning is cheap.

For namespace/source_type slices: convert `Option<&[&str]>` to `Option<Vec<String>>`:
```rust
let ns_owned: Option<Vec<String>> = namespaces.map(|ns| ns.iter().map(|s| s.to_string()).collect());
```

Then inside the closure, convert back:
```rust
let ns_refs: Option<Vec<&str>> = ns_owned.as_ref().map(|v| v.iter().map(|s| s.as_str()).collect());
let ns_slice: Option<&[&str]> = ns_refs.as_deref();
```

This is ugly but necessary for the `'static` bound. Wrap it in a helper if it appears more than twice.

### Fix 4: reembed_all Messages

After the existing chunk re-embedding loop, add:

```rust
// Re-embed messages (only those originally embedded via add_message_embedded)
let mut msg_count = 0usize;
let message_data: Vec<(i64, String)> = {
    self.with_conn(|conn| {
        let mut stmt = conn.prepare("SELECT id, content FROM messages WHERE embedding IS NOT NULL")?;
        let result = stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(result)
    }).await?
};

for (msg_id, content) in &message_data {
    let embedding = self.inner.embedder.embed(content).await?;
    let bytes = db::embedding_to_bytes(&embedding);
    let mid = *msg_id;
    self.with_conn(move |conn| {
        conn.execute(
            "UPDATE messages SET embedding = ?1 WHERE id = ?2",
            rusqlite::params![bytes, mid],
        )?;
        Ok(())
    }).await?;
    msg_count += 1;
    count += 1;
    if msg_count % 100 == 0 {
        tracing::info!(msg_count, "Re-embedded {} messages so far", msg_count);
    }
}
```

Update the final log:
```rust
tracing::info!(
    facts = fact_count, chunks = chunk_count, messages = msg_count, total = count,
    "Re-embedding complete"
);
```

After the complete reembed, clear the dirty flag:
```rust
self.with_conn(|conn| db::clear_embeddings_dirty(conn)).await?;
```

### Fix 5: Dirty Warning in Search

In both `search()` and `search_vector_only()`, before returning results, check the dirty flag:

```rust
// At the start of the with_conn closure for search:
if db::is_embeddings_dirty(conn)? {
    tracing::warn!(
        "Embeddings are stale after model change — search quality is degraded. \
         Call reembed_all() to regenerate embeddings."
    );
}
```

Do this check inside the `with_conn` closure so it doesn't require an extra DB round-trip. The check is a single row read and adds negligible latency.

---

## Agent: tokenizer.rs (NEW)

**Fixes you're implementing:** Fix 8 (pluggable token counting)

### Full Module

```rust
//! Pluggable token counting for context budget management.
//!
//! Provides the [`TokenCounter`] trait for text-to-token-count conversion,
//! with [`EstimateTokenCounter`] as a simple default.

use std::sync::Arc;

/// Trait for counting tokens in text.
///
/// Implement this to plug in tiktoken, sentencepiece, or any
/// model-specific tokenizer for accurate context budget management.
///
/// # Examples
///
/// ```rust
/// use semantic_memory::TokenCounter;
///
/// struct MyTokenizer;
/// impl TokenCounter for MyTokenizer {
///     fn count_tokens(&self, text: &str) -> usize {
///         // Use tiktoken, sentencepiece, etc.
///         text.split_whitespace().count() // placeholder
///     }
/// }
/// ```
pub trait TokenCounter: Send + Sync {
    /// Count the number of tokens in the given text.
    fn count_tokens(&self, text: &str) -> usize;
}

/// Default token counter: estimates tokens as `len / 4`.
///
/// Acceptable for English prose (~4 chars per token on average).
/// Inaccurate for CJK text (~1 token per char), code, or structured data.
/// Replace with a real tokenizer for accurate budget management.
pub struct EstimateTokenCounter;

impl TokenCounter for EstimateTokenCounter {
    fn count_tokens(&self, text: &str) -> usize {
        // Minimum 1 token for non-empty text
        if text.is_empty() {
            0
        } else {
            (text.len() / 4).max(1)
        }
    }
}

/// Create the default token counter (estimate-based).
pub(crate) fn default_token_counter() -> Arc<dyn TokenCounter> {
    Arc::new(EstimateTokenCounter)
}
```

### Integration in config.rs

```rust
use crate::tokenizer::TokenCounter;
use std::sync::Arc;

pub struct MemoryConfig {
    // ... existing fields ...

    /// Custom token counter. None = use EstimateTokenCounter (chars / 4).
    /// Set this to a real tokenizer for accurate context budget management.
    #[serde(skip)]
    pub token_counter: Option<Arc<dyn TokenCounter>>,
}
```

In `Default for MemoryConfig`, set `token_counter: None`.

### Integration in MemoryStoreInner

```rust
struct MemoryStoreInner {
    conn: Mutex<rusqlite::Connection>,
    embedder: Box<dyn Embedder>,
    config: MemoryConfig,
    token_counter: Arc<dyn TokenCounter>,  // NEW
}
```

In `open_with_embedder`:
```rust
let token_counter = config.token_counter.clone()
    .unwrap_or_else(|| crate::tokenizer::default_token_counter());
```

### Integration in conversation.rs

In `add_message` and `add_message_embedded`, when `token_count` is `None`:
```rust
let effective_token_count = token_count.or_else(|| {
    Some(token_counter.count_tokens(content) as u32)
});
```

This means the `add_message` and `add_message_embedded` functions need access to the token counter. Pass it as a parameter from `MemoryStore` methods, or make the `MemoryStore` methods compute it before calling the conversation module functions.

**Preferred approach:** Compute in `MemoryStore` methods before calling into conversation module:
```rust
pub async fn add_message(&self, session_id: &str, role: Role, content: &str,
    token_count: Option<u32>, metadata: Option<serde_json::Value>,
) -> Result<i64, MemoryError> {
    let effective_token_count = token_count.unwrap_or_else(|| {
        self.inner.token_counter.count_tokens(content) as u32
    });
    let sid = session_id.to_string();
    let ct = content.to_string();
    // ... with_conn ...
    self.with_conn(move |conn| {
        conversation::add_message(conn, &sid, role, &ct, Some(effective_token_count), meta.as_ref())
    }).await
}
```

This way conversation.rs doesn't need any changes — it already accepts `Option<u32>` and stores it.

### Integration in chunker.rs

Change `chunk_text` signature:
```rust
pub fn chunk_text(text: &str, config: &ChunkingConfig, token_counter: &dyn TokenCounter) -> Vec<TextChunk>
```

Replace `text.len() / 4` with `token_counter.count_tokens(&content)` in the `TextChunk` construction.

Update `MemoryStore::chunk_text`:
```rust
pub fn chunk_text(&self, text: &str) -> Vec<TextChunk> {
    chunker::chunk_text(text, &self.inner.config.chunking, self.inner.token_counter.as_ref())
}
```

This method stays sync (no DB, no embed).

---

## Agent: knowledge.rs V2

**Fixes you're implementing:** Fix 10a (safety comments)

Add the safety comment before every `unchecked_transaction()` call. There are 4 in this file:
- `insert_fact_with_fts`
- `delete_fact_with_fts`
- `update_fact_with_fts`

(Plus `delete_namespace` calls `delete_fact_with_fts` in a loop — those inner calls already have the comment.)

---

## Agent: conversation.rs V2

**Fixes you're implementing:** Fix 10a (safety comments), Fix 10b (budget doc)

### Safety Comments

Add before every `unchecked_transaction()`:
- `add_message`
- `add_message_with_embedding`
- `delete_session`

### Budget Documentation

Add to `get_messages_within_budget`:
```rust
/// Get messages from a session up to `max_tokens` total.
///
/// Walks backward from newest, accumulating token counts, stops when
/// the budget is exceeded. Returns messages in chronological order.
///
/// **Edge case:** The first (most recent) message is always included even
/// if it alone exceeds `max_tokens`. This ensures the method never returns
/// an empty Vec for a non-empty session. Callers that need strict budget
/// enforcement should check the total token count of returned messages.
```

---

## Agent: documents.rs V2

**Fixes you're implementing:** Fix 10a (safety comments)

Add before every `unchecked_transaction()`:
- `insert_document_with_chunks`
- `delete_document_with_chunks`
