# V3_CHANGE_MAP.md — Exact Code Locations and Transformations

This is a quick-reference for which lines in which files need to change. Use alongside V3_SPEC.md for the detailed "what" and "why".

---

## Phase 1: Fix SQ8 Math

**File: `src/quantize.rs`**

| Line(s) | Current | Change To |
|---------|---------|-----------|
| ~41 | `let scale = (max - min) / 254.0;` | Keep (254.0 is correct for symmetric) |
| ~42 | `let zero_point_f = (-128.0 - min / scale).round();` | `let zero_point_f = (-127.0 - min / scale).round();` |
| ~43 | `let zero_point = zero_point_f.clamp(-128.0, 127.0) as i8;` | `let zero_point = zero_point_f.clamp(-127.0, 127.0) as i8;` |
| ~48 | `let q = (v / scale + zero_point as f32).round();` | Keep |
| ~49 | `q.clamp(-128.0, 127.0) as i8` | `q.clamp(-127.0, 127.0) as i8` |

**Also add:** `pack_quantized()` and `unpack_quantized()` functions (see V3_SPEC.md §5).

---

## Phase 2: HNSW Keymap Persistence

**File: `src/db.rs`**

- Add `MIGRATION_V5` constant with SQL from V3_SPEC.md §9
- Add entry `(5, MIGRATION_V5)` to `MIGRATIONS` array

**File: `src/hnsw.rs`**

- Add `keymap_dirty: AtomicBool` to `HnswIndexInner`
- Add `pub fn flush_keymap(&self, conn: &Connection) -> Result<(), MemoryError>`
- Add `pub fn load_keymap(&self, conn: &Connection) -> Result<(), MemoryError>`
- In `insert()`: set `keymap_dirty` to true
- In `delete()`: set `keymap_dirty` to true

**File: `src/lib.rs`**

- In `MemoryStore::open()` after HNSW load: call `hnsw_index.load_keymap(&conn)`
- In `MemoryStoreInner::drop()`: call `hnsw_index.flush_keymap(&conn)` (needs conn access — extract from Mutex)
- In `flush_hnsw()`: add `flush_keymap()` call

**Caution:** `Drop` needs a `Connection` reference. Currently `Drop` already accesses `self.hnsw_index` for save. The `conn` Mutex is still accessible in `Drop` — lock it there.

---

## Phase 3: RwLock<HnswIndex>

**File: `src/lib.rs`**

- Line ~27: Change `hnsw_index: HnswIndex` → `hnsw_index: RwLock<HnswIndex>`
- Add `use std::sync::RwLock;` (already imported for other uses? Check.)

**Every `self.inner.hnsw_index.xxx()` call becomes `self.inner.hnsw_index.read().unwrap().xxx()`:**

Search for all occurrences. Known locations (approximate line numbers):
- `rebuild_hnsw_index()` — change to write lock for swap
- `flush_hnsw()` — read lock
- `add_fact()` — read lock for insert
- `add_fact_with_embedding()` — read lock for insert  
- `update_fact()` — read lock for update
- `delete_fact()` — read lock for delete
- `delete_namespace()` — read lock for delete loop
- `ingest_document()` — read lock for insert loop
- `delete_document()` — read lock for delete loop
- `add_message_embedded()` — read lock for insert
- `search()` — read lock for search
- `Drop` impl — read lock for save

**File: `src/lib.rs` — `rebuild_hnsw_index()` rewrite:**

Currently builds new index, saves to disk, returns. Change to:
1. Build new index (no lock)
2. Write-lock, swap `*guard = new_index`
3. Read-lock, save + flush_keymap

---

## Phase 4: Vector-Only HNSW Path

**File: `src/search.rs`**

- Add new function `vector_only_search_with_hnsw()` (see V3_SPEC.md §4)
- Pattern: Copy structure from `hybrid_search_with_hnsw`, remove BM25 phase

**File: `src/lib.rs`**

- In `search_vector_only()`: add `#[cfg(feature = "hnsw")]` block that calls HNSW search then `vector_only_search_with_hnsw`
- Keep existing brute-force path under `#[cfg(not(feature = "hnsw"))]`

---

## Phase 5: Wire Quantization

**File: `src/quantize.rs`**

- Add `pack_quantized()` and `unpack_quantized()` functions

**File: `src/lib.rs`**

All insert methods that compute embeddings need an additional quantize + store step:

| Method | Add After Embedding |
|--------|-------------------|
| `add_fact()` | Quantize, pass q8_bytes to insert SQL |
| `add_fact_with_embedding()` | Quantize pre-computed embedding, pass q8_bytes |
| `update_fact()` | Quantize new embedding, pass q8_bytes to update SQL |
| `ingest_document()` | Quantize each chunk embedding, pass q8_bytes |
| `add_message_embedded()` | Quantize, pass q8_bytes |
| `reembed_all()` | Quantize each re-embedded vector, update q8 column |

**File: `src/knowledge.rs`**

- `insert_fact_with_fts()`: Add `q8_bytes` parameter, update INSERT SQL
- `update_fact_with_fts()`: Add `q8_bytes` parameter, update UPDATE SQL

**File: `src/documents.rs`**

- `insert_document_with_chunks()` / `insert_document_with_chunks_and_ids()`: Add q8 data parameter

**File: `src/conversation.rs`**

- `add_message_with_embedding()`: Add q8_bytes parameter

**File: `src/config.rs`**

- Add `rerank_from_f32: bool` to `SearchConfig` with default `true`

---

## Phase 6: Batch Lookups

**File: `src/search.rs` — `hybrid_search_with_hnsw()`**

Replace the `for hit in hnsw_hits { ... match domain { ... conn.query_row() ... } }` loop (approximately lines 450-560 of current search.rs) with:

1. Partition hits by domain into `Vec<(String, f64)>` for facts, chunks, msgs
2. For each non-empty domain, execute one `WHERE id IN (...)` query
3. Build a `HashMap<id, row_data>` from results
4. Iterate partitioned hits, look up from HashMap, build VectorHit vec

Also apply same pattern to `vector_only_search_with_hnsw()` (Phase 4).

**Key detail:** The current code does TWO queries per fact hit (content + updated_at). The batched query fetches both in one SELECT.

---

## Phase 7: Compaction

**File: `src/hnsw.rs`**

- Add to `HnswConfig`: `pub compaction_threshold: f32` (default 0.3)
- Add `pub fn deleted_ratio(&self) -> f32`
- Add `pub fn needs_compaction(&self) -> bool`

**File: `src/lib.rs`**

- Add `pub async fn compact_hnsw(&self) -> Result<(), MemoryError>`
- In `search()`, after HNSW search: add warning log if `needs_compaction()`

---

## Phase 8: Box::leak Fix

**File: `src/hnsw.rs`**

- In `HnswIndexInner`: add `_reloader_keepalive: Option<Pin<Box<HnswIo>>>`
- In `HnswIndex::new()`: set `_reloader_keepalive: None`
- In `HnswIndex::load()`: change from `Box::leak(...)` to storing in `_reloader_keepalive`

**The tricky part:** `HnswIo` → `Hnsw<'static>` requires the reloader to have a `'static` lifetime. By keeping it in a `Pin<Box<>>` alongside the graph in the same struct, and using `unsafe` transmute to `'static`, we can avoid the leak. Add a `// SAFETY` comment explaining the invariant.

**Simpler alternative if the above is too complex:** Keep `Box::leak` but track the leaked pointer in `_reloader_keepalive: Option<*mut HnswIo>` and deallocate in `Drop`. Less elegant but eliminates the leak.

---

## Phase 9: Tests

New test files to create:

```
tests/
├── hnsw_persistence.rs      (Phase 2 tests)
├── hnsw_hotswap.rs           (Phase 3 tests)
├── vector_only_hnsw.rs       (Phase 4 tests)
├── quantization_pipeline.rs  (Phase 5 tests)
├── batch_lookup.rs           (Phase 6 tests)
├── compaction.rs             (Phase 7 tests)
└── migration_v5.rs           (Phase 9 tests)
```

See `V3_TESTING.md` for exact test specifications.

---

## Phase 10: Cleanup

- `Cargo.toml`: `version = "0.3.0"`
- `src/lib.rs`: Update module-level doc comment to mention quantization and HNSW persistence
- Remove any `#[allow(dead_code)]` that was added temporarily
- Run `cargo doc --all-features --no-deps` and fix any doc warnings
