# semantic-memory V2 — Claude Code Patch Prompt

## What You're Doing

You are patching the `semantic-memory` crate to fix critical bugs, architectural deficiencies, and API gaps identified by two independent code reviews. The crate already compiles and passes tests from V1.1. You are NOT rewriting the crate — you are surgically fixing defects and adding targeted capabilities.

**Location:** `~/Coding/Libraries/semantic-memory/`

---

## Before Writing Any Code

**Read these files in this exact order:**

1. **`SPEC.md`** — The authoritative V1 spec. Your changes must be consistent with its patterns.
2. **`CLAUDE.md`** — Coding rules and conventions. Follow them exactly.
3. **`AGENTS.md`** — Module-specific notes. Read the section for every module you touch.
4. **`V2_SPEC_ADDENDUM.md`** — The new requirements for this patch. THIS is what you're implementing.
5. **`V2_AGENTS_ADDENDUM.md`** — Module-specific implementation guidance for this patch. Covers spawn_blocking patterns, bytemuck usage, tokenizer trait design, and other per-file details.
6. **`V2_TESTING_ADDENDUM.md`** — New and modified test cases. All new tests must pass alongside existing ones.

After reading all six, come back here for the implementation plan.

---

## The 10 Fixes

### Fix 1: `is_multiple_of` Nightly-Only Method (db.rs) — COMPILE BUG

**Problem:** `bytes.len().is_multiple_of(4)` in `bytes_to_embedding()` uses `u64::is_multiple_of()` which is unstable (nightly-only, `feature(unsigned_is_multiple_of)`). **This crate does not compile on stable Rust.**

**What to do:**

Replace `bytes.len().is_multiple_of(4)` with `bytes.len() % 4 == 0`.

One line. Do this first.

**Verify:** `cargo build` succeeds on stable toolchain.

---

### Fix 2: Blocking SQLite on Async Executor (lib.rs) — ARCHITECTURE

**Problem:** Async methods (`add_fact`, `search`, `update_fact`, etc.) do embedding (async, outside lock — good) then run SQLite operations synchronously **on the tokio executor thread** while holding the mutex. Under concurrency (multi-agent workflows, tool spam, ingestion while chatting), this stalls the async runtime.

**What to do:**

Wrap all Mutex-locked database work in `tokio::task::spawn_blocking`. Create a private helper method on `MemoryStore` so every call site doesn't repeat the pattern:

```rust
impl MemoryStore {
    /// Run a synchronous closure that needs the database connection on a blocking thread.
    /// Prevents SQLite I/O from stalling the tokio executor.
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
        .map_err(|e| MemoryError::Other(format!("Blocking task failed: {}", e)))?
    }
}
```

**Then refactor every method that currently does `let conn = self.inner.conn.lock()...`:**

- **Sync-only methods** (no async, no embedding): `create_session`, `list_sessions`, `delete_session`, `add_message`, `get_recent_messages`, `get_messages_within_budget`, `session_token_count`, `delete_fact`, `delete_namespace`, `get_fact`, `list_facts`, `chunk_text`, `stats`, `vacuum` — these become `async fn` using `self.with_conn(...)`.await`.
- **Async methods** that already exist (`add_fact`, `update_fact`, `search`, etc.): Replace the inline `conn.lock()` block with `self.with_conn(...)`.await`.

**IMPORTANT:** The `with_conn` helper needs `F: Send + 'static`, which means closures can't borrow from `&self` — they must capture `Arc` clones or owned data. Pass needed config/values by clone into the closure.

**API change:** Previously-sync methods like `create_session` become async. This is a **breaking change** for callers. It's acceptable because:
1. The crate is v0.1 (pre-1.0, semver allows it)
2. Every real consumer is already in an async context
3. The alternative (keeping sync + async dual APIs) doubles the surface area

**See `V2_AGENTS_ADDENDUM.md` "Agent: lib.rs V2" for the full method-by-method migration table and the `with_conn` closure capture patterns.**

---

### Fix 3: Silent Embedding Parse Coercion (embedder.rs) — BUGFIX

**Problem:** In `OllamaEmbedder::embed_batch`, non-numeric values in the embedding response are silently coerced to `0.0` via `as_f64().unwrap_or(0.0)`. This hides corrupt responses from broken Ollama builds or misconfigured proxies. A vector with injected zeros has degraded similarity accuracy and the user has no idea why search quality is bad.

**What to do:**

Replace the silent coercion with an explicit error:

```rust
// BEFORE (bad)
.map(|v| v.as_f64().unwrap_or(0.0) as f32)

// AFTER (good)
.map(|v| v.as_f64().ok_or_else(|| {
    MemoryError::Other(format!(
        "Embedding contains non-numeric value: {}",
        v
    ))
}))
.collect::<Result<Vec<f64>, _>>()?
.into_iter()
.map(|v| v as f32)
.collect();
```

Also add a validation check that the response status is 2xx before parsing:
```rust
if !response.status().is_success() {
    let status = response.status();
    let body = response.text().await.unwrap_or_default();
    return Err(MemoryError::Other(format!(
        "Ollama returned HTTP {}: {}", status, body
    )));
}
```

---

### Fix 4: `reembed_all()` Skips Messages (lib.rs) — BUGFIX

**Problem:** `reembed_all()` re-embeds facts and chunks but ignores messages. After changing embedding models, conversation search vectors go stale while facts and chunks are fresh — silently degrading cross-source search accuracy.

**What to do:**

Add a third pass in `reembed_all()` that processes messages with non-NULL embeddings:

```rust
// Re-embed messages (only those that were originally embedded)
let message_data: Vec<(i64, String)> = {
    let conn = self.inner.conn.lock().expect("mutex poisoned");
    let mut stmt = conn.prepare(
        "SELECT id, content FROM messages WHERE embedding IS NOT NULL"
    )?;
    stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
        .collect::<Result<Vec<_>, _>>()?
};

for (msg_id, content) in &message_data {
    let embedding = self.inner.embedder.embed(content).await?;
    let bytes = db::embedding_to_bytes(&embedding);
    let conn = self.inner.conn.lock().expect("mutex poisoned");
    conn.execute(
        "UPDATE messages SET embedding = ?1 WHERE id = ?2",
        rusqlite::params![bytes, msg_id],
    )?;
    count += 1;
}
```

**After Fix 2 is applied**, this should use `self.with_conn()` instead of direct `conn.lock()`. Implement it alongside Fix 2.

**Update the return count** to include re-embedded messages. Update the `tracing::info!` at the end to report facts/chunks/messages separately.

---

### Fix 5: Embedding Metadata Update Footgun (db.rs, error.rs) — BUGFIX

**Problem:** When the configured embedding model/dimensions don't match what's stored in `embedding_metadata`, the code warns and updates the metadata row. After the update, the database *claims* it matches the new model — but every existing embedding is still from the old model. This is a data integrity lie. A user who doesn't notice the warning will get silently wrong search results.

**What to do:**

Add an `embeddings_dirty` flag to the metadata table. Add a V3 migration:

```sql
-- V3: Embedding staleness tracking
ALTER TABLE embedding_metadata ADD COLUMN embeddings_dirty INTEGER NOT NULL DEFAULT 0;
```

When `check_embedding_metadata` detects a model/dimensions mismatch:
1. Update the model/dimensions as before
2. Set `embeddings_dirty = 1`
3. Log `tracing::warn!` as before

Add a new check in `hybrid_search` and `vector_only_search`: if `embeddings_dirty = 1`, log `tracing::warn!("Embeddings are stale — search quality degraded. Call reembed_all() to fix.")` on every search call. This is intentionally noisy to force the user to act.

In `reembed_all()`, after completing all re-embeddings, set `embeddings_dirty = 0`.

Add a public method:
```rust
pub fn embeddings_are_dirty(&self) -> Result<bool, MemoryError>
```

**See `V2_AGENTS_ADDENDUM.md` "Agent: db.rs V2" for the migration and check_embedding_metadata changes.**

---

### Fix 6: Zero-Allocation Embedding Decode (db.rs, Cargo.toml) — PERFORMANCE

**Problem:** `bytes_to_embedding()` allocates a new `Vec<f32>` for every row during vector search. For a scan of 50K rows, that's 50K allocations of ~3KB each. Combined with the full-table-scan nature of vector search, this is the single hottest allocation path.

**What to do:**

Add `bytemuck` to dependencies:
```toml
bytemuck = { version = "1", features = ["derive"] }
```

Add a zero-copy decode function alongside the existing one:
```rust
/// Zero-copy view of a BLOB as f32 slice. No allocation.
/// Returns an error if the byte slice isn't properly aligned or sized.
pub fn bytes_as_embedding(bytes: &[u8]) -> Result<&[f32], MemoryError> {
    if bytes.len() % 4 != 0 {
        return Err(MemoryError::InvalidEmbedding {
            expected_bytes: bytes.len() - (bytes.len() % 4),
            actual_bytes: bytes.len(),
        });
    }
    bytemuck::try_cast_slice(bytes).map_err(|e| {
        MemoryError::Other(format!("Embedding alignment error: {}", e))
    })
}
```

**Update `vector_search` in `search.rs`** to use `bytes_as_embedding` instead of `bytes_to_embedding` for the per-row cosine computation. Keep `bytes_to_embedding` (allocating) for cases where ownership is needed (like returning embeddings to callers).

**Note on alignment:** SQLite BLOB data from `row.get::<_, Vec<u8>>()` is heap-allocated and will be 1-byte aligned. `bytemuck::try_cast_slice` will fail if alignment is wrong. If this happens in practice, fall back to `bytes_to_embedding`. The `try_cast_slice` attempt is still worth it because it succeeds on most allocators and saves ~50K allocs per search.

**ACTUALLY — the safer approach:** Since SQLite returns `Vec<u8>` which may not be 4-byte aligned, use `bytemuck::pod_collect_to_vec` or simply stick with the manual decode but **reuse a single buffer** across rows instead of allocating per-row:

```rust
// In vector_search, before the loop:
let mut decode_buf: Vec<f32> = Vec::with_capacity(expected_dims);

// Per-row:
decode_buf.clear();
for chunk in bytes.chunks_exact(4) {
    decode_buf.push(f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]));
}
let similarity = cosine_similarity(&query_embedding, &decode_buf);
```

This reuses a single allocation across all rows. Much simpler than fighting alignment. **Use this approach.**

**See `V2_AGENTS_ADDENDUM.md` "Agent: search.rs V2" for the buffer reuse pattern in the full vector_search context.**

---

### Fix 7: Standard Trait Impls for Role (types.rs) — API IMPROVEMENT

**Problem:** `Role` has `as_str()` and `from_str_value()` but doesn't implement `Display` or `FromStr`. This means it can't be used with `format!`, `println!`, `str::parse()`, or any generic code expecting standard traits.

**What to do:**

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

Keep the existing `as_str()` and `from_str_value()` methods — they're used internally and by serde. The trait impls delegate to them.

---

### Fix 8: Pluggable Token Counting (new: tokenizer.rs, config.rs, chunker.rs, conversation.rs) — FEATURE

**Problem:** Token counting is hardcoded as `len / 4` (chars ÷ 4). This is wrong for CJK text (~1 token per char), code (highly variable), and structured data. For an OpenClaw agent that manages token budgets for context windows, inaccurate counting means either wasting context space or blowing limits.

**What to do:**

Create a new file `src/tokenizer.rs` with a trait and default implementation:

```rust
/// Trait for counting tokens in text.
/// Implement this to plug in tiktoken, sentencepiece, or model-specific tokenizers.
pub trait TokenCounter: Send + Sync {
    /// Count the number of tokens in the given text.
    fn count_tokens(&self, text: &str) -> usize;
}

/// Default estimator: chars / 4. Acceptable for English prose,
/// inaccurate for CJK, code, or structured data.
pub struct EstimateTokenCounter;

impl TokenCounter for EstimateTokenCounter {
    fn count_tokens(&self, text: &str) -> usize {
        text.len() / 4
    }
}
```

**Add to `MemoryConfig`:**
```rust
// Not serializable — skip in serde, set programmatically
#[serde(skip)]
pub token_counter: Option<Arc<dyn TokenCounter>>,
```

Default: `None` (falls back to `EstimateTokenCounter`).

**Add to `MemoryStoreInner`:**
```rust
token_counter: Arc<dyn TokenCounter>,
```

Initialized from config in `open_with_embedder`:
```rust
token_counter: config.token_counter.clone()
    .unwrap_or_else(|| Arc::new(EstimateTokenCounter)),
```

**Update `chunker.rs`:** `chunk_text` currently computes `token_count_estimate: text.len() / 4`. Change the signature to accept `&dyn TokenCounter` and use it. Update `MemoryStore::chunk_text` to pass the store's token counter.

**Update `add_message` / `add_message_embedded`:** If `token_count` is `None`, use the token counter to compute it automatically instead of storing NULL. This makes `get_messages_within_budget` reliable by default.

**Export from `lib.rs`:**
```rust
pub use tokenizer::{TokenCounter, EstimateTokenCounter};
```

**See `V2_AGENTS_ADDENDUM.md` "Agent: tokenizer.rs" for the full module.**

---

### Fix 9: Vector Search Row Count Guard (search.rs) — SAFETY

**Problem:** Vector search does a full table scan with no upper bound. If someone dumps 500K facts, every single search silently becomes a 250ms+ operation with no warning.

**What to do:**

After loading the count of rows to scan in `vector_search`, check against a threshold:

```rust
const VECTOR_SCAN_WARN_THRESHOLD: usize = 50_000;

// After counting rows to scan:
if row_count > VECTOR_SCAN_WARN_THRESHOLD {
    tracing::warn!(
        row_count,
        "Vector search scanning {} rows — search latency will be degraded. \
         Consider namespace partitioning or pruning old data.",
        row_count
    );
}
```

This is a warning, not an error. Don't limit the scan — just make the problem visible.

---

### Fix 10: Code Quality Sweep — HYGIENE

Several small fixes that should be done across the crate:

**a) Safety comments on `unchecked_transaction()`:**

Every call to `conn.unchecked_transaction()` should have a comment explaining why:
```rust
// SAFETY: We receive &Connection (not &mut) from Mutex::lock().
// unchecked_transaction() is required because transaction() needs &mut self.
// Nesting safety is guaranteed because the Mutex serializes all DB access.
let tx = conn.unchecked_transaction()?;
```

Add this comment to every `unchecked_transaction()` call site (there are ~10 across knowledge.rs, documents.rs, conversation.rs, db.rs).

**b) Document the token budget edge case:**

In `get_messages_within_budget`, add a doc comment:
```rust
/// Note: The first (most recent) message is always included even if it
/// alone exceeds `max_tokens`. This ensures the method never returns an
/// empty Vec for a non-empty session. Callers that need strict budget
/// enforcement should check the returned token total.
```

**c) `#[doc(hidden)]` raw_execute should stay test-only:**

The existing `raw_execute` method has `#[doc(hidden)]`. Keep it, but also add `#[cfg(test)]` so it doesn't compile into release builds. Wait — it's used by integration tests (separate crate), so `#[cfg(test)]` won't work. Keep `#[doc(hidden)]` only.

---

## New Dependencies Summary

```toml
[dependencies]
# ADD:
bytemuck = { version = "1", features = ["derive"] }

# EXISTING — no changes needed:
rusqlite = { version = "0.32", features = ["bundled", "blob"] }
reqwest = { version = "0.12", features = ["json", "rustls-tls"], default-features = false }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tokio = { version = "1", features = ["rt", "macros"] }
thiserror = "2"
tracing = "0.1"
uuid = { version = "1", features = ["v4"] }
chrono = { version = "0.4", features = ["serde"] }
```

---

## New Files

| File | Purpose |
|------|---------|
| `src/tokenizer.rs` | `TokenCounter` trait + `EstimateTokenCounter` default |

---

## Implementation Order

Do these in exactly this order. Each fix must `cargo build` + `cargo test` clean before moving on.

1. **Fix 1** (is_multiple_of) — one line in db.rs, zero risk
2. **Fix 7** (Role traits) — types.rs only, additive
3. **Fix 3** (silent embed coercion) — embedder.rs only, isolated
4. **Fix 10** (code quality sweep) — comments only, no logic changes
5. **Fix 6** (buffer reuse in vector search) — search.rs + Cargo.toml, performance
6. **Fix 9** (vector scan guard) — search.rs, warning only
7. **Fix 8** (pluggable token counter) — new file + config + chunker + conversation
8. **Fix 5** (embedding dirty flag) — db.rs migration + check logic
9. **Fix 2** (spawn_blocking) — lib.rs refactor, biggest change, most risk
10. **Fix 4** (reembed messages) — lib.rs, depends on Fix 2 patterns

---

## Validation

After all fixes:

```bash
cargo fmt --check
cargo clippy -- -W clippy::all
cargo test
cargo test -- --ignored  # if Ollama is available
cargo doc --no-deps
cargo build --release
```

All must pass clean.

---

## Do NOT Do These Things

- **Do NOT rewrite modules that aren't listed.** Touch only what the fixes require.
- **Do NOT change the `MemoryStore` struct layout** beyond adding `token_counter` to `Inner`.
- **Do NOT add dependencies** beyond `bytemuck`. Everything else is already available.
- **Do NOT change existing public type signatures** except `Role` (additive traits) and the sync→async migration in Fix 2.
- **Do NOT implement HNSW/ANN indexing.** The buffer reuse in Fix 6 is the V2 optimization. ANN is a V3 concern.
- **Do NOT hold the Mutex across an await point.** The `with_conn` helper in Fix 2 explicitly prevents this.
- **Do NOT remove `MockEmbedder` or change its deterministic behavior.** All tests depend on it.
- **Do NOT touch the FTS5 tables, bridge table pattern, or migration V1/V2 SQL** except to add V3.
