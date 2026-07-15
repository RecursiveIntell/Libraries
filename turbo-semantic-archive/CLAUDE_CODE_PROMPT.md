# semantic-memory v0.3.0 Upgrade — Claude Code Master Prompt

## Mission

Upgrade `semantic-memory` from v0.2.0 to v0.3.0. This is a correctness + performance release that wires quantization into the live pipeline, fixes HNSW persistence/lifecycle bugs, eliminates N+1 query patterns, and routes all vector search paths through HNSW when the feature is enabled.

**Read `V3_SPEC.md` completely before writing any code.** It is the authoritative specification. Where it conflicts with existing code comments or older spec files, `V3_SPEC.md` wins.

**Read `V3_TESTING.md` before writing tests.** It defines required test coverage.

---

## Ground Rules

1. **No new dependencies** unless explicitly approved in V3_SPEC.md. Do not add `r2d2`, `deadpool`, or connection-pooling crates — that's a future change.
2. **Preserve the public API surface** for `MemoryStore` methods. Signature changes are allowed only where V3_SPEC.md explicitly calls for them (e.g., `rebuild_hnsw_index` return type change).
3. **All code must compile on stable Rust ≥ 1.75.** No nightly features.
4. **Run `cargo test --all-features` after every logical change group.** Do not batch all changes and test once at the end.
5. **Do not modify files in `/mnt/user-data/uploads/`.** Work from the extracted copy.
6. **Development happens locally.** All file creation and iteration happens in the working directory.

---

## Change Execution Order

Execute changes in this exact order. Each phase should compile and pass tests before moving to the next.

### Phase 1: Fix SQ8 Math (quantize.rs)
- Fix the range ambiguity: commit to symmetric `[-127, 127]` with 254 steps
- See V3_SPEC.md §1 for exact formulas
- Run existing quantization tests — they should still pass with tighter assertions

### Phase 2: HNSW Persistence — Key Mapping + Tombstones (hnsw.rs, db.rs)
- Persist `key_to_id` / `id_to_key` / `deleted_ids` / `next_id` in SQLite (new table `hnsw_keymap`)
- Rebuild mappings on `HnswIndex::load()` from SQLite, not from scratch
- See V3_SPEC.md §2 for schema and serialization format

### Phase 3: HNSW Hot-Swap for rebuild_hnsw_index (lib.rs, hnsw.rs)
- Change `hnsw_index` in `MemoryStoreInner` from `HnswIndex` to `RwLock<HnswIndex>`
- `rebuild_hnsw_index()` atomically swaps the inner index after building
- All search/insert/delete paths acquire read lock; rebuild acquires write lock
- See V3_SPEC.md §3 for the locking contract

### Phase 4: Route vector_only_search Through HNSW (search.rs, lib.rs)
- Add `hybrid_search_with_hnsw` equivalent for vector-only path
- `search_vector_only()` on `MemoryStore` should use HNSW when feature is enabled
- See V3_SPEC.md §4

### Phase 5: Wire Quantization Into the Pipeline (lib.rs, hnsw.rs, search.rs)
- Store quantized i8 vectors alongside f32 in SQLite (new `embedding_q8` BLOB column)
- HNSW index operates on quantized vectors for insert/search
- Optional f32 rerank for top-N HNSW candidates (configurable)
- See V3_SPEC.md §5 for the full data flow

### Phase 6: Batch HNSW→SQLite Lookups (search.rs)
- Replace per-hit `query_row` calls in `hybrid_search_with_hnsw` with batched `WHERE id IN (...)` 
- Deduplicate the fact `updated_at` double-query
- See V3_SPEC.md §6

### Phase 7: HNSW Compaction (hnsw.rs, lib.rs)
- Add `compaction_threshold` to `HnswConfig` (default: 0.3 = 30% deleted)
- Add `compact()` method that rebuilds the index dropping tombstones
- `search()` logs a warning when deleted ratio exceeds threshold
- See V3_SPEC.md §7

### Phase 8: Address Box::leak (hnsw.rs)
- Wrap the leaked `HnswIo` in a struct that owns it properly
- Use `ManuallyDrop` + custom Drop to control deallocation order
- See V3_SPEC.md §8

### Phase 9: Migration + Integration Tests
- Add SQLite migration V5 for new columns/tables
- Write all tests specified in V3_TESTING.md
- Verify `cargo test --all-features` passes clean

### Phase 10: Documentation + Cleanup
- Update doc comments on all changed public items
- Update `lib.rs` module-level docs
- Bump `Cargo.toml` to version `0.3.0`
- Remove dead code paths and unused imports

---

## Files to Modify

| File | Phases | Nature of Change |
|------|--------|-----------------|
| `src/quantize.rs` | 1, 5 | Fix math, add `quantize_for_storage()` |
| `src/hnsw.rs` | 2, 3, 7, 8 | Persistence, RwLock wrapper, compaction, leak fix |
| `src/db.rs` | 2, 5, 9 | New migration V5, `hnsw_keymap` table, q8 columns |
| `src/lib.rs` | 3, 4, 5, 7 | RwLock plumbing, vector_only HNSW path, compaction API |
| `src/search.rs` | 4, 5, 6 | HNSW vector-only path, reranking, batch lookups |
| `src/config.rs` | 5, 7 | Rerank config, compaction threshold |
| `src/error.rs` | 7 | `CompactionNeeded` variant (warning-level) |
| `Cargo.toml` | 10 | Version bump |
| `tests/` | 9 | New test files per V3_TESTING.md |

---

## Quality Checks

After all phases:

```bash
# Must all pass
cargo check --all-features
cargo test --all-features
cargo clippy --all-features -- -D warnings
cargo doc --all-features --no-deps
```

---

## Reference Documents

- **V3_SPEC.md** — Authoritative technical specification for all changes
- **V3_TESTING.md** — Required test coverage
- **UPGRADE_SPEC.md** — Historical context (v0.1→v0.2), do NOT re-implement
- **V2_SPEC_ADDENDUM.md** — Historical context (v0.2), do NOT re-implement
