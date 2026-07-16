# V1.1 Agents Addendum

Module-specific implementation guidance for the V1.1 patch. Supplements `AGENTS.md` — do NOT replace existing sections.

---

## Agent: Search V1.1 (`search.rs`)

**You're touching this module for 4 of the 6 fixes.** Read the full existing `search.rs` before making any changes.

### Fix 1: Parameterized Namespace Filtering

The core problem: rusqlite's `params![]` is a compile-time macro that can't handle dynamic-length parameter lists. You need to switch to runtime parameter binding.

**Recommended approach:**

```rust
use rusqlite::types::Value as SqlValue;

fn build_namespace_clause(
    column: &str,
    namespaces: Option<&[&str]>,
    param_offset: usize,
) -> (String, Vec<SqlValue>) {
    match namespaces {
        Some(ns) if !ns.is_empty() => {
            let placeholders: Vec<String> = (0..ns.len())
                .map(|i| format!("?{}", param_offset + i))
                .collect();
            let clause = format!("AND {} IN ({})", column, placeholders.join(", "));
            let values: Vec<SqlValue> = ns.iter()
                .map(|n| SqlValue::Text(n.to_string()))
                .collect();
            (clause, values)
        }
        _ => (String::new(), vec![]),
    }
}
```

Then in `bm25_search` and `vector_search`, build the full parameter list:

```rust
let (ns_clause, ns_params) = build_namespace_clause("f.namespace", namespaces, 3);
let sql = format!("SELECT ... WHERE facts_fts MATCH ?1 {} ... LIMIT ?2", ns_clause);

// Build dynamic param list
let mut all_params: Vec<Box<dyn rusqlite::types::ToSql>> = vec![
    Box::new(sanitized_query.to_string()),
    Box::new(pool_size as i64),
];
for val in &ns_params {
    all_params.push(Box::new(val.clone()));
}
let param_refs: Vec<&dyn rusqlite::types::ToSql> = all_params.iter().map(|p| p.as_ref()).collect();

let mut stmt = conn.prepare(&sql)?;
let rows = stmt.query_map(param_refs.as_slice(), |row| { ... })?;
```

**Watch out:** The LIMIT parameter and the MATCH parameter use numbered placeholders (?1, ?2). The namespace placeholders start after those. Make sure `param_offset` accounts for the existing parameters in each query.

**Actually** — rusqlite supports `rusqlite::params_from_iter` which is cleaner. Use that if the version supports it. Check `rusqlite` 0.32 docs.

### Fix 3: Recency Scoring

The timestamp comes as ISO 8601 text from SQLite. Parse it to compute age:

```rust
fn days_since(iso_timestamp: &str) -> Option<f64> {
    let dt = chrono::NaiveDateTime::parse_from_str(iso_timestamp, "%Y-%m-%d %H:%M:%S").ok()?;
    let now = chrono::Utc::now().naive_utc();
    let duration = now - dt;
    Some(duration.num_seconds() as f64 / 86400.0)
}
```

**Gotcha:** SQLite's `datetime('now')` produces `"YYYY-MM-DD HH:MM:SS"` format (no timezone, no T separator). The chrono format string must match.

### Fix 6: Deduplication

Apply AFTER the final `results.truncate(top_k)` call, not before. Rationale: dedup might remove items, so you'd want to over-fetch to compensate. But in V1.1, just deduplicate what you have — if dedup drops 2 of 5 results, the caller gets 3. This is acceptable; the alternative (fetching extra and deduplicating pre-truncation) changes the scoring dynamics.

### Fix 2: Message Search Integration

Add `messages_fts` and `messages` to the BM25 and vector search functions. Gate behind `SearchSourceType::Messages`. The query patterns mirror the facts queries:

**BM25:**
```sql
SELECT mm.message_id, m.content, m.session_id, m.role, bm25(messages_fts) AS score
FROM messages_fts
JOIN messages_rowid_map mm ON messages_fts.rowid = mm.rowid
JOIN messages m ON m.id = mm.message_id
WHERE messages_fts MATCH ?1
-- Optional: AND m.session_id IN (...)
ORDER BY bm25(messages_fts)
LIMIT ?2
```

**Vector:**
```sql
SELECT m.id, m.content, m.session_id, m.role, m.embedding
FROM messages m
WHERE m.embedding IS NOT NULL
-- Optional: AND m.session_id IN (...)
```

**Session ID filtering** uses the same parameterized approach as namespace filtering.

---

## Agent: Database V1.1 (`db.rs`)

### V2 Migration

Add `MIGRATION_V2` as a `const &str`. Apply it in `run_migrations` when `current_version < 2`.

**Critical:** `ALTER TABLE messages ADD COLUMN embedding BLOB` must work on existing databases with data. SQLite ALTER TABLE ADD COLUMN sets the new column to NULL for existing rows. This is exactly what we want — existing messages have no embedding.

**Test the migration path:** Open a V1 database (one that already has sessions and messages), apply V2, verify:
- Existing messages still have all their data
- `messages.embedding` is NULL for all existing messages  
- `messages_rowid_map` table exists and is empty
- `messages_fts` table exists and is empty
- `_schema_version` shows version 2

---

## Agent: Conversation V1.1 (`conversation.rs`)

### New: `add_message_with_embedding`

Follow the EXACT same pattern as `knowledge.rs::insert_fact_with_fts`. The transaction sequence is:

1. Verify session exists (SELECT EXISTS)
2. Begin transaction
3. INSERT into messages (with embedding BLOB in the new column)
4. Get last_insert_rowid → this is the message_id
5. INSERT into messages_rowid_map (message_id)
6. Get last_insert_rowid → this is the FTS rowid
7. INSERT into messages_fts(rowid, content)
8. UPDATE sessions SET updated_at
9. Commit
10. Return message_id

**The caller (lib.rs) does the embedding BEFORE calling this function.** This function is synchronous — it only does database work.

### CASCADE Cleanup

When a session is deleted (`DELETE FROM sessions WHERE id = ?`), the `ON DELETE CASCADE` on messages handles message deletion. But the FTS entries are NOT automatically cleaned up — they're in a contentless virtual table that doesn't participate in CASCADE.

**Two options:**

A. **Explicit cleanup before CASCADE:** Before deleting a session, query all message_ids for that session that have entries in `messages_rowid_map`, and run the FTS delete for each. Then delete the session (CASCADE handles the rest).

B. **Accept ghost FTS entries:** The bridge table has `ON DELETE CASCADE` pointing at `messages(id)`, so the `messages_rowid_map` rows will be deleted. But the FTS entries will become orphans. Searches will find FTS hits but the JOIN back through the bridge will exclude them (the bridge row is gone). This is technically safe but wastes FTS space over time.

**Go with Option A.** It's more correct and the performance cost is negligible. Add this cleanup to `delete_session`:

```rust
pub fn delete_session(conn: &Connection, session_id: &str) -> Result<(), MemoryError> {
    let tx = conn.unchecked_transaction()?;
    
    // Clean up message FTS entries before CASCADE
    let fts_data: Vec<(i64, String, i64)> = {
        let mut stmt = tx.prepare(
            "SELECT m.id, m.content, mm.rowid
             FROM messages m
             JOIN messages_rowid_map mm ON mm.message_id = m.id
             WHERE m.session_id = ?1"
        )?;
        stmt.query_map(params![session_id], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?))
        })?.collect::<Result<Vec<_>, _>>()?
    };
    
    for (_msg_id, content, fts_rowid) in &fts_data {
        tx.execute(
            "INSERT INTO messages_fts(messages_fts, rowid, content) VALUES('delete', ?1, ?2)",
            params![fts_rowid, content],
        )?;
    }
    
    // Now delete session (CASCADE handles messages + messages_rowid_map)
    let affected = tx.execute("DELETE FROM sessions WHERE id = ?1", params![session_id])?;
    if affected == 0 {
        tx.commit()?;
        return Err(MemoryError::SessionNotFound(session_id.to_string()));
    }
    
    tx.commit()?;
    Ok(())
}
```

**This replaces the existing `delete_session`.** The old version was a single DELETE without FTS cleanup.
