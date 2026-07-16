# V3_TESTING.md — Required Test Coverage for v0.3.0

All tests use `MockEmbedder` unless stated otherwise. Tests requiring HNSW must be gated behind `#[cfg(feature = "hnsw")]`.

---

## 1. Quantization Tests (`tests/quantization.rs` — update existing)

### 1.1 `round_trip_symmetric_range`
- Quantize a random normalized 768-dim vector
- Assert all quantized values ∈ `[-127, 127]` (NOT -128)
- Assert `zero_point ∈ [-127, 127]`
- Dequantize and compute cosine similarity with original
- Assert cosine similarity > 0.995

### 1.2 `round_trip_max_error`
- Quantize, dequantize
- For each dimension: `|original[i] - reconstructed[i]| < scale`
- (Max error is one quantization step)

### 1.3 `constant_vector_handling`
- Quantize `[0.5; 768]`
- Assert `data = [0; 768]`, `scale = 1.0`, `zero_point = 0`

### 1.4 `pack_unpack_round_trip`
- Quantize → pack → unpack → compare all fields match exactly
- Test with several different vectors

### 1.5 `pack_unpack_wrong_dimensions`
- Pack a 768-dim quantized vector
- Attempt unpack with `dimensions = 384`
- Assert `MemoryError::QuantizationError`

### 1.6 `cosine_similarity_preserved`
- Generate 100 random normalized vectors
- Compute pairwise cosine similarity on f32 originals
- Quantize all, dequantize, recompute pairwise cosine similarity
- Assert rank ordering is preserved (Spearman correlation > 0.99)

---

## 2. HNSW Persistence Tests (`tests/hnsw_persistence.rs` — new file)

All tests gated behind `#[cfg(feature = "hnsw")]`.

### 2.1 `keymap_survives_reopen`
- Open MemoryStore, add 10 facts
- Close (drop MemoryStore)
- Reopen same directory
- Search for one of the facts by content
- Assert it appears in results with correct fact_id
- Assert HNSW index `len()` matches expected count

### 2.2 `deletions_survive_reopen`
- Open, add 10 facts, delete 3
- Close, reopen
- Search for deleted facts — assert they do NOT appear
- Search for remaining facts — assert they DO appear
- Assert `len()` == 7

### 2.3 `keymap_flush_on_explicit_flush`
- Open, add facts
- Call `flush_hnsw()` explicitly
- Verify `hnsw_keymap` table has correct row count (query SQLite directly via `raw_execute` or test helper)

### 2.4 `rebuild_preserves_keymap`
- Open, add facts
- Call `rebuild_hnsw_index()`
- Verify search still works
- Close, reopen
- Verify search still works

### 2.5 `reopen_with_no_keymap_table_graceful`
- Open on a fresh directory, add facts, close
- Manually delete `hnsw_keymap` contents (simulate pre-v0.3 database)
- Reopen — should log warning and rebuild keymap from graph point count
- (Degraded state is acceptable; crash is not)

---

## 3. HNSW Hot-Swap Tests (`tests/hnsw_hotswap.rs` — new file)

### 3.1 `rebuild_updates_live_instance`
- Open, add 10 facts
- Delete 5 facts from SQLite directly (simulating corruption)
- Call `rebuild_hnsw_index()`
- Search for remaining 5 — assert they appear
- Search for deleted 5 — assert they do NOT appear
- (No reopen needed — the point is testing in-memory swap)

### 3.2 `concurrent_search_during_rebuild`
- Open, add 100 facts
- Spawn a task that does search in a loop
- Spawn a task that calls `rebuild_hnsw_index()`
- Assert no panics, no poisoned locks
- Assert search always returns results (may be from old or new index)

---

## 4. Vector-Only HNSW Path Tests (`tests/vector_only_hnsw.rs` — new file)

### 4.1 `vector_only_uses_hnsw`
- Open with HNSW feature
- Add 20 facts across 2 namespaces
- Call `search_vector_only()` with namespace filter
- Assert results only from filtered namespace
- Assert results are non-empty

### 4.2 `vector_only_respects_source_type_filter`
- Add facts AND ingest a document
- `search_vector_only()` with `source_types = [Facts]`
- Assert no chunk results

### 4.3 `vector_only_matches_hybrid_vector_component`
- Add 10 facts
- Run `search()` (hybrid) and `search_vector_only()` with same query
- Assert the top result is the same
- (Scores will differ due to RRF weighting, but ranking should be similar)

---

## 5. Quantization Pipeline Integration (`tests/quantization_pipeline.rs` — new file)

### 5.1 `add_fact_stores_q8`
- Open MemoryStore, add a fact
- Query SQLite directly: `SELECT embedding_q8 FROM facts WHERE id = ?`
- Assert blob is non-null
- Unpack and verify dimensions match

### 5.2 `ingest_document_stores_q8`
- Ingest a document
- Query: `SELECT embedding_q8 FROM chunks WHERE document_id = ?`
- Assert all chunks have non-null q8 embeddings

### 5.3 `add_message_embedded_stores_q8`
- Create session, add embedded message
- Query: `SELECT embedding_q8 FROM messages WHERE id = ?`
- Assert non-null q8 embedding

### 5.4 `reembed_all_regenerates_q8`
- Add facts, then call `reembed_all()`
- Verify q8 embeddings are regenerated (compare before/after bytes — they should change because MockEmbedder is deterministic by content, not by call order)

---

## 6. Batch Lookup Tests (`tests/batch_lookup.rs` — new file)

### 6.1 `hybrid_search_no_n_plus_one`
- Add 20 facts
- Run `search()` with `top_k = 5`
- Assert results are correct and complete
- (Correctness test — performance verification is manual)

### 6.2 `search_with_mixed_domains`
- Add 10 facts + ingest 2 documents + add 5 embedded messages
- Run `search()` across all source types
- Assert results include facts, chunks, and messages
- Assert no duplicate results

---

## 7. Compaction Tests (`tests/compaction.rs` — new file)

### 7.1 `deleted_ratio_computation`
- Create HNSW index, insert 10 items, delete 3
- Assert `deleted_ratio()` ≈ 0.3
- Assert `needs_compaction()` == true (if threshold is 0.3, it's at the boundary — set threshold to 0.25 for this test)

### 7.2 `compact_reduces_tombstones`
- Open MemoryStore, add 20 facts, delete 10
- Verify `needs_compaction()` == true
- Call `compact_hnsw()`
- Verify search still returns correct results for remaining 10
- Verify the deleted_ids set is empty after compaction (check via index len)

### 7.3 `compact_skips_when_healthy`
- Add 10 facts, delete 0
- Call `compact_hnsw()`
- Assert it returns Ok without rebuilding (check logs or verify index identity hasn't changed)

---

## 8. Migration Tests (`tests/migration_v5.rs` — new file)

### 8.1 `v5_migration_adds_columns`
- Open a fresh MemoryStore (triggers all migrations)
- Query `PRAGMA table_info(facts)` — assert `embedding_q8` column exists
- Query `PRAGMA table_info(chunks)` — assert `embedding_q8` column exists
- Query `PRAGMA table_info(messages)` — assert `embedding_q8` column exists
- Query `SELECT name FROM sqlite_master WHERE name = 'hnsw_keymap'` — assert exists

### 8.2 `v5_migration_idempotent`
- Open MemoryStore, close, reopen
- Assert no errors (migration should be a no-op on second open)

---

## 9. Existing Tests — Must Still Pass

All existing tests in `tests/` must continue to pass without modification (unless a test explicitly tested wrong behavior that's now fixed, e.g., the quantization range). Specifically verify:

- `tests/integration_tests.rs`
- `tests/db_tests.rs`
- `tests/search_tests.rs`
- `tests/hnsw_integration.rs`
- `tests/brute_force_parity.rs`
- `tests/storage_lifecycle.rs`
- `tests/concurrent_access.rs`
- `tests/chunker_tests.rs`
- `tests/tokenizer_tests.rs`
- `tests/conversation_tests.rs`
- `tests/conversation_search_tests.rs`
- `tests/knowledge_tests.rs`

---

## Running Tests

```bash
# Full suite with HNSW
cargo test --features "hnsw,testing"

# Full suite with brute-force fallback
cargo test --features "brute-force,testing"

# Just the new v0.3 tests
cargo test --features "hnsw,testing" -- hnsw_persistence hnsw_hotswap vector_only_hnsw quantization_pipeline batch_lookup compaction migration_v5

# Clippy (treat warnings as errors)
cargo clippy --all-features -- -D warnings
```
