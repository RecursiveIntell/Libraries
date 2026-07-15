# V1.1 Testing Addendum

New and modified tests required by the V1.1 patch. These supplement the existing test files — do NOT delete existing tests.

---

## New Test File: `tests/conversation_search_tests.rs`

```
#[cfg(test)] setup:
- MockEmbedder (128 dims)
- tempfile::NamedTempFile for DB
- MemoryStore::open_with_embedder
```

### Test Cases

1. **Embedded message is searchable**
   - Create session → `add_message_embedded` with content "The Navier-Stokes equations govern fluid dynamics"
   - `search_conversations("fluid dynamics", None, None)` → 1 result, content matches
   - Result source is `SearchSource::Message` with correct session_id and message_id

2. **Non-embedded message is invisible to search**
   - Create session → `add_message` (sync, no embedding) with content "invisible message"
   - `search_conversations("invisible", None, None)` → 0 results

3. **Session ID filtering**
   - Create 2 sessions, add embedded messages to both
   - `search_conversations(query, None, Some(&[session_a_id]))` → only results from session A

4. **Mixed source type search**
   - Add a fact and an embedded message with related content
   - `search(query, None, None, Some(&[Facts, Messages]))` → both appear in results
   - Default `search(query, None, None, None)` → only the fact appears (messages not included by default)

5. **Message FTS delete on session delete**
   - Add embedded messages → delete session → `search_conversations` → 0 results
   - Verify no ghost FTS entries (search for the exact content → empty)

---

## Modified Test File: `tests/search_tests.rs`

### New Test Cases

6. **Parameterized namespace filtering (Fix 1)**
   - Add facts in namespaces "safe", "also-safe", and one with an adversarial name containing a single quote: "it's-a-test"
   - Search with namespace filter `&["it's-a-test"]` → finds the fact, doesn't crash
   - Search with namespace filter `&["safe"]` → only finds facts in "safe"

7. **Content deduplication (Fix 6)**
   - Add a fact with content "Rust was released in 2015"
   - Ingest a document with a chunk containing the exact same text "Rust was released in 2015"
   - `search("Rust released")` → exactly 1 result (not 2)
   - The result with the higher score is the one kept

8. **Dedup doesn't merge different content**
   - Add two facts: "Rust 2015" and "Go 2009"
   - `search("programming language released")` → 2 results (both kept, they're different)

9. **Recency weighting disabled (Fix 3 regression)**
   - Config with `recency_half_life_days: None`
   - Add 2 facts at different times → search → scores are identical to V1 behavior
   - Specifically: score depends only on BM25 rank + vector rank, no timestamp influence

10. **Recency weighting enabled**
    - Config with `recency_half_life_days: Some(30.0), recency_weight: 0.5`
    - Add fact A "today" and fact B with `updated_at` manually set to 60 days ago (both with identical content relevance)
    - Search → fact A scores higher than fact B
    - The score difference should be approximately: `0.5 * (1.0 - 0.25) / 61 ≈ 0.00615`

11. **Recency with zero half-life**
    - Config with `recency_half_life_days: Some(0.0)`
    - Search should still work (no panic, no NaN) — recency term is simply not applied

---

## Modified Test File: `tests/integration_tests.rs`

### New Test Cases

12. **V2 migration on existing DB**
    - Open DB (triggers V1 migration) → close → reopen (triggers V2) → verify `messages` table has `embedding` column
    - Verify existing messages (from V1) have NULL embedding — no data loss

13. **TLS build verification**
    - Not a runtime test, but verify the build: `cargo build --release` should succeed with the rustls-tls feature

---

## Test Utilities

For the recency tests, you'll need to manipulate timestamps. Add a test helper:

```rust
/// Insert a fact with a specific updated_at timestamp (for recency testing).
fn insert_fact_with_timestamp(
    store: &MemoryStore,
    namespace: &str,
    content: &str,
    embedding: &[f32],
    updated_at: &str,  // ISO 8601
) -> String {
    let fact_id = store.add_fact_with_embedding(namespace, content, embedding, None, None).unwrap();
    // Directly update the timestamp via raw SQL
    // This is test-only — production code never does this
    let conn = /* need access to inner conn for tests */;
    conn.execute(
        "UPDATE facts SET updated_at = ?1 WHERE id = ?2",
        params![updated_at, fact_id],
    ).unwrap();
    fact_id
}
```

**Problem:** `MemoryStoreInner` is private, so tests can't access the connection directly.

**Solution:** Add a `#[cfg(test)]` method on `MemoryStore`:
```rust
#[cfg(test)]
pub fn raw_execute(&self, sql: &str, params: &[&dyn rusqlite::types::ToSql]) -> Result<usize, MemoryError> {
    let conn = self.inner.conn.lock().expect("mutex poisoned");
    Ok(conn.execute(sql, params)?)
}
```

This is gated behind `#[cfg(test)]` so it never compiles into release builds.
