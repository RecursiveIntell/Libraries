# semantic-memory V1.1 — Claude Code Patch Prompt

## What You're Doing

You are patching the existing `semantic-memory` crate to fix 6 issues identified during code review. The crate already compiles and passes tests. You are NOT rewriting anything — you are surgically adding capabilities and fixing defects.

**Location:** `~/Coding/Libraries/semantic-memory/`

---

## Before Writing Any Code

**Read these files in this exact order:**

1. **`SPEC.md`** — The authoritative spec. Your changes must be consistent with its patterns.
2. **`CLAUDE.md`** — Coding rules and conventions. Follow them exactly.
3. **`AGENTS.md`** — Module-specific notes. Read the section for every module you touch.
4. **`V1_1_SPEC_ADDENDUM.md`** — The new requirements for this patch. THIS is what you're implementing.
5. **`V1_1_AGENTS_ADDENDUM.md`** — Module-specific implementation guidance for this patch. Covers parameterized query binding, timestamp parsing, CASCADE + FTS cleanup, and other per-file gotchas.
6. **`V1_1_TESTING_ADDENDUM.md`** — New and modified test cases. 13 new tests across 3 files, plus test utilities.

After reading all six, come back here for the implementation plan.

---

## The 6 Fixes

### Fix 1: Parameterized Namespace Filtering (search.rs) — SECURITY

**Problem:** `namespace_filter_sql()` builds SQL via string interpolation with manual quote escaping. This is a SQL injection vector if namespaces ever come from user input.

**What to do:**

Replace the `namespace_filter_sql` function with a parameterized approach. The challenge is that rusqlite doesn't support `IN (?)` with a dynamic list, so use one of these strategies:

**Strategy A (preferred):** Generate `AND column IN (?1, ?2, ?3)` with numbered placeholders matching the namespace count, then pass the namespace values as additional parameters to `query_map`. This requires changing `bm25_search` and `vector_search` to accept the namespaces as bind parameters rather than baking them into the SQL string.

**Strategy B (acceptable):** If Strategy A makes the query construction too ugly, use multiple `OR column = ?N` clauses joined together.

**Do NOT:**
- Keep the current string interpolation approach
- Use `format!` to inject namespace values into SQL

**Changes required:**
- `search.rs`: Rewrite `namespace_filter_sql` → new function `namespace_filter_clause(column: &str, namespaces: Option<&[&str]>) -> (String, Vec<String>)` that returns both the SQL fragment AND the parameter values
- `search.rs`: Update `bm25_search` and `vector_search` to thread these parameters through to `query_map`
- Existing tests must still pass
- See `V1_1_TESTING_ADDENDUM.md` test case #6 for the adversarial namespace test
- See `V1_1_AGENTS_ADDENDUM.md` "Agent: Search V1.1" → Fix 1 for the `params_from_iter` binding pattern

### Fix 2: Conversation Semantic Search (conversation.rs, search.rs, db.rs, lib.rs, types.rs) — FEATURE

**Problem:** Messages are stored but never embedded. An agent can't semantically search past conversations — "what approach did we try for X?" requires scanning raw text.

**What to do:**

Add an opt-in message embedding pipeline and a conversation search method.

**Schema change (db.rs):** Add a V2 migration:
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

**conversation.rs changes:**
- New function: `add_message_with_embedding(conn, session_id, role, content, token_count, metadata, embedding_bytes) -> Result<i64, MemoryError>` — same as `add_message` but also stores the embedding BLOB and inserts into the FTS bridge + FTS table. Wrap in a transaction.
- New function: `delete_message_fts(conn, message_id) -> Result<(), MemoryError>` — needed for cleanup. Same contentless FTS delete pattern as facts.

**search.rs changes:**
- Add a new `SearchSourceType::Messages` variant
- Add a new `SearchSource::Message { message_id, session_id, role }` variant  
- Update `bm25_search` to also query `messages_fts` when source_types includes `Messages` (or when source_types is None and an explicit `include_messages: bool` flag is true). Default behavior should NOT include messages — the caller opts in.
- Update `vector_search` similarly — scan message embeddings when opted in.
- The RRF fusion doesn't care about source type, so it needs no changes.

**lib.rs changes:**
- New method: `add_message_embedded(session_id, role, content, token_count, metadata) -> Result<i64, MemoryError>` — async. Embeds the content, then stores message + embedding + FTS in one transaction.
- New method: `search_conversations(query, top_k, session_ids: Option<&[&str]>) -> Result<Vec<SearchResult>, MemoryError>` — async. Hybrid search over messages only. If `session_ids` is Some, filter to those sessions. If None, search all.

**types.rs changes:**
- Add `SearchSourceType::Messages` 
- Add `SearchSource::Message { message_id: i64, session_id: String, role: String }`

**Design decisions:**
- Message embedding is OPT-IN. `add_message` (existing) stays sync and doesn't embed. `add_message_embedded` (new) is async and embeds. This preserves the "conversation module works without Ollama" property.
- Session ID filtering on conversation search uses the same parameterized approach as Fix 1 (not string interpolation).
- Don't embed system messages by default — but don't prevent it either. The caller decides by choosing which method to call.

**Tests (new file: `tests/conversation_search_tests.rs`):** See `V1_1_TESTING_ADDENDUM.md` for full test cases #1–5. See `V1_1_AGENTS_ADDENDUM.md` for conversation.rs and db.rs module guidance (CASCADE + FTS cleanup is critical).
- Add 5 embedded messages across 2 sessions → search → find relevant message
- Session filtering: search with session_id filter → only results from that session
- Non-embedded messages don't appear in search results
- Mixed search: facts + messages when both source types requested

### Fix 3: Recency-Weighted Scoring (search.rs, config.rs) — FEATURE

**Problem:** All search results are scored purely by relevance (BM25 + cosine). An agent running for months accumulates stale facts that rank equally with fresh ones. There's no way to boost recent information.

**What to do:**

Add an optional time-decay factor to RRF scoring.

**config.rs changes:**
```rust
pub struct SearchConfig {
    // ... existing fields ...
    
    /// Optional recency boost. If enabled, results are boosted based on how 
    /// recently they were created/updated. The value is the half-life in days —
    /// a fact that is `recency_half_life_days` old gets 50% of the recency boost.
    /// None = no recency weighting (current behavior, default).
    pub recency_half_life_days: Option<f64>,
    
    /// Weight of the recency boost relative to BM25 and vector scores in RRF.
    /// Only used when recency_half_life_days is Some.
    /// Default: 0.5
    pub recency_weight: f64,
}
```

Default: `recency_half_life_days: None, recency_weight: 0.5` — this means existing behavior is unchanged unless the caller explicitly opts in.

**search.rs changes:**

Update `RrfCandidate` to carry an optional `created_at: Option<String>` (ISO 8601 timestamp).

Update `bm25_search` and `vector_search` to also SELECT the `created_at` or `updated_at` column and propagate it into their hit types.

Update `rrf_fuse` to accept an `Option<f64>` for half-life and `f64` for recency weight. When recency is enabled:

```
recency_score = recency_weight * 2^(-age_days / half_life) / (rrf_k + 1)
final_score = bm25_rrf_score + vector_rrf_score + recency_score
```

The `/ (rrf_k + 1)` normalization keeps the recency term on the same scale as the RRF scores (which max out at `weight / (k + 1)` for rank 1).

**Why exponential decay:** It's the standard approach, it's simple, and the half-life parameter is intuitive. "Facts older than 30 days are half as boosted" is easy to reason about.

**Changes to function signatures:**

`rrf_fuse` gains two parameters: `recency_half_life_days: Option<f64>` and `recency_weight: f64`. Thread these from `SearchConfig` through `hybrid_search`.

`Bm25Hit` and `VectorHit` gain `updated_at: Option<String>`.

The SQL queries in `bm25_search` and `vector_search` need to SELECT the timestamp column. For facts it's `f.updated_at`, for chunks it's `c.created_at`, for messages it's `m.created_at`.

**Tests:** See `V1_1_TESTING_ADDENDUM.md` test cases #9–11 for full details, and the `raw_execute` test helper for timestamp manipulation. See `V1_1_AGENTS_ADDENDUM.md` "Agent: Search V1.1" → Fix 3 for the chrono parsing gotcha.
- Recency disabled (None) → same scores as before (regression test)
- Two identical-relevance facts, one from today and one from 60 days ago, with half_life=30 → today's fact scores higher
- Half-life = 0 edge case → don't divide by zero, treat as no decay

### Fix 4: Enable TLS for reqwest (Cargo.toml) — BUGFIX

**Problem:** `reqwest` is configured with `default-features = false`, which disables TLS. This means `OllamaEmbedder` will fail at runtime if anyone points it at an `https://` URL (e.g., a remote embedding API, OpenAI-compatible endpoint, Ollama behind a reverse proxy with TLS).

**What to do:**

Change the reqwest dependency to enable `rustls-tls`:

```toml
reqwest = { version = "0.12", features = ["json", "rustls-tls"], default-features = false }
```

**Why rustls, not native-tls:** No OpenSSL dependency. Compiles everywhere. Already standard for Rust HTTP clients. Smaller attack surface.

**No code changes needed.** Just the Cargo.toml line.

**Test:** `cargo build` still succeeds. Verify with `cargo tree -i reqwest` that `rustls` appears in the dependency tree.

### Fix 5: Document the Brute-Force Vector Cliff (SPEC.md only) — DOCUMENTATION

**Problem:** The brute-force cosine scan is fast at <100K vectors but has a hard performance cliff. This is documented nowhere except a passing mention in the spec's "does not" section.

**What to do:**

Add this to SPEC.md after the current §8.1 or wherever the vector search algorithm is described. Also add it as item 8 in CLAUDE.md's "Things That Will Bite You" section.

See `V1_1_SPEC_ADDENDUM.md` for the exact text to add.

### Fix 6: Search Result Deduplication Across Source Types (search.rs) — BUGFIX

**Problem:** If the same text exists as both a fact and a document chunk (e.g., a fact was extracted from a document), both will appear in search results as separate entries. The RRF fusion deduplicates by ID, but a fact and chunk have different IDs even if the content is identical.

**What to do:**

After RRF fusion produces the final sorted result list, add a content-based deduplication pass:

```rust
fn deduplicate_results(results: Vec<SearchResult>) -> Vec<SearchResult> {
    let mut seen_content = std::collections::HashSet::new();
    results.into_iter().filter(|r| {
        // Normalize: trim + lowercase for comparison
        let key = r.content.trim().to_lowercase();
        seen_content.insert(key)
    }).collect()
}
```

Call this at the end of `hybrid_search`, `fts_only_search`, and `vector_only_search`, AFTER truncating to top_k.

**Note:** This is a soft dedup — it catches exact content matches after normalization. Near-duplicates (same fact slightly reworded) still appear. That's fine for V1; cosine similarity–based dedup is a V2 problem.

**Test:** See `V1_1_TESTING_ADDENDUM.md` test cases #7–8. See `V1_1_AGENTS_ADDENDUM.md` "Agent: Search V1.1" → Fix 6 for dedup ordering notes.
- Insert same text as both a fact and a document chunk → search → only one result returned
- Two genuinely different results with similar scores → both returned

---

## Implementation Order

Do these in exactly this order. Each fix must `cargo build` + `cargo test` clean before moving on.

1. **Fix 4** (TLS) — Cargo.toml only, zero risk
2. **Fix 1** (namespace SQL) — search.rs only, existing tests validate
3. **Fix 6** (dedup) — search.rs only, add tests
4. **Fix 3** (recency) — config.rs + search.rs, add tests
5. **Fix 2** (conversation search) — schema migration + multi-file, most complex
6. **Fix 5** (docs) — markdown only, do last

---

## Validation

After all fixes:

```bash
cargo fmt --check
cargo clippy -- -W clippy::all
cargo test
cargo doc --no-deps
cargo build --release
```

All five must pass clean.

---

## Do NOT Do These Things

- **Do NOT rewrite modules that aren't listed.** Touch only what the fixes require.
- **Do NOT change the `MemoryStore` struct layout.** The `Arc<Inner>` pattern stays.
- **Do NOT add new dependencies** beyond the reqwest feature flag change. Everything needed is already in the dependency tree.
- **Do NOT change existing public API signatures.** New methods are additive. Existing callers must not break.
- **Do NOT make conversation embedding the default.** It's opt-in via `add_message_embedded`.
- **Do NOT make recency weighting the default.** It's opt-in via `recency_half_life_days: Some(...)`.
- **Do NOT hold the Mutex across an await point.** Same rule as V1.
- **Do NOT skip the FTS content on delete for message FTS.** Same contentless FTS pattern as facts.
