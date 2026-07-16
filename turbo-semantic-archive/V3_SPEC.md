# V3_SPEC.md — semantic-memory v0.3.0 Technical Specification

**Status:** Authoritative. Where this conflicts with older specs or code comments, this document wins.

---

## §1. Fix SQ8 Quantization Math

### Problem

Current `quantize.rs` uses a `254.0` divisor with a `[-128, 127]` output range. This is ambiguous — it's neither clean symmetric quantization (which would use `[-127, 127]` / 254 steps) nor proper asymmetric (which would use `[-128, 127]` / 255 steps). The zero_point calculation compounds this by using `-128.0` as the base.

### Solution: Commit to Symmetric `[-127, 127]`

Symmetric quantization is correct for normalized embedding vectors (which nomic-embed-text produces). The zero value maps cleanly to 0, and the range is balanced.

```rust
// BEFORE (ambiguous)
let scale = (max - min) / 254.0;
let zero_point_f = (-128.0 - min / scale).round();
let zero_point = zero_point_f.clamp(-128.0, 127.0) as i8;

// AFTER (symmetric, correct)
let scale = (max - min) / 254.0; // 254 = 127 - (-127)
let zero_point_f = -127.0 - (min / scale);
let zero_point = zero_point_f.round().clamp(-127.0, 127.0) as i8;

// Quantize
let q = (v / scale + zero_point as f32).round();
q.clamp(-127.0, 127.0) as i8
```

**Key invariants:**
- Output range: `[-127, 127]` (254 discrete levels)
- `scale = (max - min) / 254.0`
- `zero_point ∈ [-127, 127]`
- Dequantize: `original[i] ≈ (data[i] as f32 - zero_point as f32) * scale`
- Constant vectors produce `data = [0; dims], scale = 1.0, zero_point = 0`

### Test Assertion Update

Existing test `round_trip_accuracy` should be tightened:
- Max absolute error per dimension: `< scale` (one quantization step)
- Cosine similarity between original and dequantized: `> 0.995`

---

## §2. HNSW Persistence — Key Mapping and Tombstones

### Problem

`hnsw_rs` persists the graph topology and raw vector data, but not our application-level mappings (`key_to_id`, `id_to_key`, `deleted_ids`, `next_id`). After process restart, these are empty, so HNSW search returns node IDs we can't resolve to fact/chunk/message keys. The library silently degrades to BM25-only results.

### Solution: SQLite-backed Keymap Table

Add a new table in migration V5:

```sql
-- V5: HNSW key mapping persistence
CREATE TABLE IF NOT EXISTS hnsw_keymap (
    node_id     INTEGER PRIMARY KEY,
    item_key    TEXT NOT NULL UNIQUE,
    deleted     INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX idx_hnsw_keymap_key ON hnsw_keymap(item_key);
```

**Metadata in `hnsw_metadata` table (already exists from V4):**

```sql
INSERT OR REPLACE INTO hnsw_metadata (key, value) VALUES ('next_id', '0');
```

### Write Path

On every `HnswIndex::insert()`:
```rust
// After successful graph insert:
conn.execute(
    "INSERT OR REPLACE INTO hnsw_keymap (node_id, item_key, deleted) VALUES (?1, ?2, 0)",
    params![id, key],
)?;
// Update next_id
conn.execute(
    "INSERT OR REPLACE INTO hnsw_metadata (key, value) VALUES ('next_id', ?1)",
    params![next_id.to_string()],
)?;
```

On `HnswIndex::delete()`:
```rust
conn.execute(
    "UPDATE hnsw_keymap SET deleted = 1 WHERE item_key = ?1",
    params![key],
)?;
```

### Architecture Decision: Deferred Writes via Dirty Flag

To avoid a SQLite write on every single HNSW insert (which would be a performance regression), use a **dirty flag + batch flush** strategy:

1. `HnswIndex` maintains its in-memory maps as before (they are the source of truth during runtime)
2. Add a `keymap_dirty: AtomicBool` flag
3. Set dirty on every insert/delete/update
4. `flush_keymap(&self, conn: &Connection)` writes all mappings to SQLite in a single transaction
5. `flush_keymap` is called:
   - Inside `MemoryStoreInner::drop()` (alongside HNSW graph save)
   - Inside `flush_hnsw()` (explicit durability)
   - Inside `rebuild_hnsw_index()` (after rebuild)

### Load Path

In `MemoryStore::open()`, after loading the HNSW graph:
```rust
// Rebuild key mappings from SQLite
let mut key_to_id = HashMap::new();
let mut id_to_key = HashMap::new();
let mut deleted_ids = HashSet::new();

let mut stmt = conn.prepare(
    "SELECT node_id, item_key, deleted FROM hnsw_keymap"
)?;
let rows = stmt.query_map([], |row| {
    Ok((row.get::<_, usize>(0)?, row.get::<_, String>(1)?, row.get::<_, bool>(2)?))
})?;

for row in rows {
    let (node_id, key, is_deleted) = row?;
    if is_deleted {
        deleted_ids.insert(node_id);
    } else {
        key_to_id.insert(key.clone(), node_id);
        id_to_key.insert(node_id, key);
    }
}

let next_id: usize = conn.query_row(
    "SELECT value FROM hnsw_metadata WHERE key = 'next_id'",
    [],
    |row| row.get::<_, String>(0),
)?.parse().unwrap_or(graph.get_nb_point());
```

### HnswIndex API Change

`HnswIndex` must not directly hold a database connection. Instead, change the persistence methods:

```rust
impl HnswIndex {
    /// Flush key mappings to SQLite. Called during MemoryStore::drop and flush_hnsw.
    pub fn flush_keymap(&self, conn: &Connection) -> Result<(), MemoryError> { ... }

    /// Load key mappings from SQLite into in-memory maps.
    pub fn load_keymap(&self, conn: &Connection) -> Result<(), MemoryError> { ... }
}
```

`MemoryStore` calls these at the appropriate lifecycle points.

---

## §3. HNSW Hot-Swap for `rebuild_hnsw_index`

### Problem

`rebuild_hnsw_index()` builds a new index and saves it to disk, but the current `MemoryStore` instance keeps using the old in-memory index. Searches after rebuild return stale results until the process restarts.

### Solution: `RwLock<HnswIndex>`

Change `MemoryStoreInner`:

```rust
struct MemoryStoreInner {
    // ... existing fields ...
    #[cfg(feature = "hnsw")]
    hnsw_index: RwLock<HnswIndex>,  // was: HnswIndex
}
```

**Locking contract:**

| Operation | Lock Type | Held Duration |
|-----------|-----------|---------------|
| `insert` | Read | Single graph insert |
| `delete` | Read | Single map update |
| `update` | Read | Delete + insert |
| `search` | Read | Graph search + map lookup |
| `rebuild_hnsw_index` | Write | Full rebuild (bulk) |
| `compact` | Write | Full rebuild (bulk) |

**Why read lock for insert/delete?** `HnswIndex` is internally thread-safe (its maps use `RwLock`). The outer `RwLock<HnswIndex>` only needs write access when replacing the entire index object.

### rebuild_hnsw_index Implementation

```rust
pub async fn rebuild_hnsw_index(&self) -> Result<(), MemoryError> {
    // 1. Build new index (no lock needed — working on a separate HnswIndex)
    let new_index = HnswIndex::new(config)?;
    // ... load all embeddings from SQLite, insert into new_index ...

    // 2. Swap (write lock — brief, just pointer swap)
    {
        let mut guard = self.inner.hnsw_index.write().unwrap();
        *guard = new_index;
    }

    // 3. Persist (read lock is fine now)
    let guard = self.inner.hnsw_index.read().unwrap();
    guard.save(&paths.hnsw_dir, &paths.hnsw_basename)?;
    guard.flush_keymap(&conn)?;

    Ok(())
}
```

### Migration of All Existing Call Sites

Every existing access to `self.inner.hnsw_index` must go through `self.inner.hnsw_index.read().unwrap()`. Search for all occurrences in `lib.rs` and update them.

Pattern:
```rust
// BEFORE
self.inner.hnsw_index.insert(key, &embedding)?;

// AFTER
self.inner.hnsw_index.read().unwrap().insert(key, &embedding)?;
```

---

## §4. Route `search_vector_only` Through HNSW

### Problem

`MemoryStore::search_vector_only()` calls `search::vector_only_search()`, which does a brute-force scan of all rows even when HNSW is enabled. This defeats the purpose of having an ANN index.

### Solution

Add a new function `vector_only_search_with_hnsw` in `search.rs`:

```rust
#[cfg(feature = "hnsw")]
pub fn vector_only_search_with_hnsw(
    conn: &Connection,
    config: &SearchConfig,
    top_k: usize,
    namespaces: Option<&[&str]>,
    source_types: Option<&[SearchSourceType]>,
    session_ids: Option<&[&str]>,
    hnsw_hits: &[HnswHit],
) -> Result<Vec<SearchResult>, MemoryError>
```

This follows the same pattern as `hybrid_search_with_hnsw` but skips the BM25 phase entirely. HNSW hits are resolved to content via batched SQLite lookups (see §6), namespace-filtered, and returned with vector-only scoring.

In `lib.rs`, the `search_vector_only` method:

```rust
pub async fn search_vector_only(&self, ...) -> Result<Vec<SearchResult>, MemoryError> {
    let query_embedding = self.inner.embedder.embed(query).await?;

    #[cfg(feature = "hnsw")]
    {
        let hnsw_hits = {
            let guard = self.inner.hnsw_index.read().unwrap();
            guard.search(&query_embedding, k * 3)?
        };
        // ... resolve via vector_only_search_with_hnsw
    }

    #[cfg(not(feature = "hnsw"))]
    {
        // ... existing brute-force path
    }
}
```

---

## §5. Wire Quantization Into the Pipeline

### Overview

Currently, quantization exists as a standalone module but nothing uses it. This phase integrates SQ8 quantization into the insert and search hot paths.

### Storage Changes (Migration V5)

```sql
ALTER TABLE facts ADD COLUMN embedding_q8 BLOB;
ALTER TABLE chunks ADD COLUMN embedding_q8 BLOB;
ALTER TABLE messages ADD COLUMN embedding_q8 BLOB;
```

The `embedding_q8` column stores a packed format:
```
[scale: f32 LE][zero_point: i8][data: i8 × dims]
```

Total bytes: `4 + 1 + dims = 773` for 768-dim vectors (vs `3072` for f32). **3.97× compression.**

### Helper Functions (quantize.rs)

```rust
/// Pack a QuantizedVector into bytes for SQLite storage.
pub fn pack_quantized(qv: &QuantizedVector) -> Vec<u8> {
    let mut buf = Vec::with_capacity(5 + qv.data.len());
    buf.extend_from_slice(&qv.scale.to_le_bytes());
    buf.push(qv.zero_point as u8);
    buf.extend_from_slice(bytemuck::cast_slice(&qv.data));
    buf
}

/// Unpack bytes from SQLite into a QuantizedVector.
pub fn unpack_quantized(bytes: &[u8], dimensions: usize) -> Result<QuantizedVector, MemoryError> {
    if bytes.len() != 5 + dimensions { ... }
    let scale = f32::from_le_bytes(bytes[0..4].try_into().unwrap());
    let zero_point = bytes[4] as i8;
    let data: Vec<i8> = bytes[5..].iter().map(|&b| b as i8).collect();
    Ok(QuantizedVector { data, scale, zero_point })
}
```

### Insert Path Change

In `MemoryStore::add_fact()`:
```rust
let embedding = self.inner.embedder.embed(content).await?;
let embedding_bytes = db::embedding_to_bytes(&embedding);

// NEW: quantize for storage and HNSW
let quantizer = Quantizer::new(self.inner.config.embedding.dimensions);
let quantized = quantizer.quantize(&embedding)?;
let q8_bytes = quantize::pack_quantized(&quantized);

// Store both f32 and q8 in SQLite
// ... INSERT with embedding_bytes AND q8_bytes ...

// HNSW insert uses f32 (hnsw_rs doesn't natively support i8)
self.inner.hnsw_index.read().unwrap().insert(key, &embedding)?;
```

**Note:** `hnsw_rs` v0.3 operates on f32 vectors. The quantized column is for future use when migrating to a Qi8-native backend, and for immediate disk/memory savings on the SQLite side. HNSW continues to use f32 for now.

### Search Path — Optional F32 Rerank

Add to `SearchConfig`:
```rust
/// When true, rerank top HNSW candidates using exact f32 cosine similarity
/// from SQLite. Improves recall at the cost of one batched SQL query.
/// Default: true
pub rerank_from_f32: bool,
```

When `rerank_from_f32` is true, after HNSW returns approximate matches:
1. Batch-load f32 embeddings from SQLite for the top candidates
2. Compute exact cosine similarity
3. Re-sort by exact similarity before RRF fusion

### reembed_all Update

`reembed_all()` must also regenerate `embedding_q8` columns:
```rust
// After computing new f32 embedding:
let quantized = quantizer.quantize(&new_embedding)?;
let q8_bytes = quantize::pack_quantized(&quantized);
tx.execute(
    "UPDATE facts SET embedding = ?1, embedding_q8 = ?2, updated_at = datetime('now') WHERE id = ?3",
    params![f32_bytes, q8_bytes, fact_id],
)?;
```

---

## §6. Batch HNSW→SQLite Lookups

### Problem

`hybrid_search_with_hnsw` does individual `query_row` calls per HNSW hit. For facts, it even does *two* queries (one for content, one for `updated_at`). At `top_k * 3 = 15` candidates, that's 15-30 individual queries per search.

### Solution

Partition HNSW hits by domain (fact/chunk/msg), then batch-load each:

```rust
// Partition keys
let mut fact_ids = Vec::new();
let mut chunk_ids = Vec::new();
let mut msg_ids = Vec::new();

for hit in hnsw_hits {
    match hit.key.split_once(':') {
        Some(("fact", id)) => fact_ids.push((id.to_string(), hit.similarity())),
        Some(("chunk", id)) => chunk_ids.push((id.to_string(), hit.similarity())),
        Some(("msg", id)) => msg_ids.push((id.parse::<i64>().ok(), hit.similarity())),
        _ => continue,
    }
}

// Batch load facts
if !fact_ids.is_empty() && search_facts {
    let placeholders = fact_ids.iter().enumerate()
        .map(|(i, _)| format!("?{}", i + 1))
        .collect::<Vec<_>>().join(", ");
    let sql = format!(
        "SELECT id, content, namespace, updated_at FROM facts WHERE id IN ({})",
        placeholders
    );
    // ... single query, build HashMap<String, FactRow>, then merge with similarities
}
```

**The `updated_at` double-query for facts is eliminated** — the batch query fetches it in the same SELECT.

### Performance Target

Single search should execute ≤ 4 SQL queries total:
1. BM25 facts
2. BM25 chunks  
3. Batch fact/chunk/message content lookups (1-3 queries depending on which domains have hits)

Down from potentially 30+ queries in the current implementation.

---

## §7. HNSW Compaction

### Problem

`HnswIndex::delete()` marks IDs as deleted (tombstones) but never reclaims them. After sustained churn, the deleted set grows, causing:
- Over-fetching during search (`top_k + deleted.len()`)
- Wasted memory in the graph
- Degraded search performance

### Solution

Add compaction support:

```rust
// config.rs
pub struct HnswConfig {
    // ... existing fields ...
    /// Ratio of deleted/total above which compaction is recommended.
    /// Default: 0.3 (30%)
    pub compaction_threshold: f32,
}

// hnsw.rs
impl HnswIndex {
    /// Ratio of deleted nodes to total nodes.
    pub fn deleted_ratio(&self) -> f32 {
        let total = self.inner.graph.get_nb_point();
        if total == 0 { return 0.0; }
        let deleted = self.inner.deleted_ids.read().unwrap().len();
        deleted as f32 / total as f32
    }

    /// Returns true if compaction is recommended.
    pub fn needs_compaction(&self) -> bool {
        self.deleted_ratio() > self.inner.config.compaction_threshold
    }
}

// lib.rs
impl MemoryStore {
    /// Compact the HNSW index by rebuilding without tombstones.
    /// This is equivalent to rebuild_hnsw_index but explicitly named for the use case.
    pub async fn compact_hnsw(&self) -> Result<(), MemoryError> {
        if !self.inner.hnsw_index.read().unwrap().needs_compaction() {
            tracing::info!("HNSW compaction not needed (deleted ratio below threshold)");
            return Ok(());
        }
        self.rebuild_hnsw_index().await
    }
}
```

In the `search` method, add a warning:
```rust
#[cfg(feature = "hnsw")]
{
    let guard = self.inner.hnsw_index.read().unwrap();
    if guard.needs_compaction() {
        tracing::warn!(
            deleted_ratio = guard.deleted_ratio(),
            "HNSW index has high tombstone ratio. Call compact_hnsw() to reclaim."
        );
    }
}
```

---

## §8. Address `Box::leak` in HNSW Load

### Problem

`HnswIndex::load()` uses `Box::leak(Box::new(HnswIo::new(...)))` to satisfy the `'static` lifetime requirement on `Hnsw`. This leaks memory on every load.

### Solution

Wrap the `HnswIo` in a struct that owns it alongside the graph:

```rust
/// Owns the HnswIo reloader alongside the graph to avoid Box::leak.
///
/// The HnswIo must live as long as the Hnsw graph because the graph
/// may reference data owned by the reloader. We use ManuallyDrop to
/// ensure the graph is dropped before the reloader.
struct OwnedHnsw {
    /// Dropped first (via ManuallyDrop in Drop impl)
    graph: std::mem::ManuallyDrop<Hnsw<'static, f32, DistCosine>>,
    /// Dropped second — outlives graph
    _reloader: Pin<Box<HnswIo>>,
}

impl Drop for OwnedHnsw {
    fn drop(&mut self) {
        // SAFETY: Drop graph first so it can't reference reloader data
        unsafe { std::mem::ManuallyDrop::drop(&mut self.graph); }
        // _reloader drops automatically after
    }
}
```

**Important:** The `'static` lifetime on `Hnsw` is a lie we tell the compiler. The real invariant is "reloader outlives graph," which `OwnedHnsw` enforces via drop order. This is safe because:
1. `HnswIo` doesn't use mmap by default (all data is copied into memory during load)
2. The graph doesn't hold direct references to the reloader's memory after construction
3. If `hnsw_rs` changes this assumption, the `ManuallyDrop` ensures we don't use-after-free — we'd get a different failure mode instead

**If `hnsw_rs` load semantics don't actually require `'static` references to the `HnswIo` data post-construction** (which appears to be the case from the source), then a simpler approach is fine:

```rust
struct HnswIndexInner {
    graph: Hnsw<'static, f32, DistCosine>,
    /// Kept alive to prevent the leaked Box from being truly leaked.
    /// In practice, hnsw_rs copies all data during load, so this is
    /// just belt-and-suspenders.
    _reloader_keepalive: Option<Box<HnswIo>>,
    // ... rest of fields
}
```

Use the simpler approach unless testing reveals the graph actually borrows from the reloader.

---

## §9. Migration V5

```sql
-- V5: Quantized embeddings + HNSW keymap persistence
ALTER TABLE facts ADD COLUMN embedding_q8 BLOB;
ALTER TABLE chunks ADD COLUMN embedding_q8 BLOB;
ALTER TABLE messages ADD COLUMN embedding_q8 BLOB;

CREATE TABLE IF NOT EXISTS hnsw_keymap (
    node_id     INTEGER PRIMARY KEY,
    item_key    TEXT NOT NULL UNIQUE,
    deleted     INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX idx_hnsw_keymap_key ON hnsw_keymap(item_key);
```

Add to `MIGRATIONS` array in `db.rs`:
```rust
const MIGRATION_V5: &str = r#"..."#;

const MIGRATIONS: &[(u32, &str)] = &[
    (1, MIGRATION_V1),
    (2, MIGRATION_V2),
    (3, MIGRATION_V3),
    (4, MIGRATION_V4),
    (5, MIGRATION_V5),
];
```

---

## §10. Config Additions

```rust
// config.rs — additions to SearchConfig
pub struct SearchConfig {
    // ... existing fields ...

    /// When true, rerank top HNSW candidates using exact f32 cosine similarity.
    /// Only applies when HNSW feature is enabled.
    /// Default: true
    pub rerank_from_f32: bool,
}

impl Default for SearchConfig {
    fn default() -> Self {
        Self {
            // ... existing defaults ...
            rerank_from_f32: true,
        }
    }
}
```

```rust
// config.rs — additions to HnswConfig
pub struct HnswConfig {
    // ... existing fields ...

    /// Deleted-to-total ratio above which compaction is recommended.
    /// Default: 0.3
    pub compaction_threshold: f32,
}

impl Default for HnswConfig {
    fn default() -> Self {
        Self {
            // ... existing defaults ...
            compaction_threshold: 0.3,
        }
    }
}
```
