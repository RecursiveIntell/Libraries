# semantic-memory — Claude Code Implementation Prompt

## What You're Building

`semantic-memory` is a standalone Rust library crate that provides hybrid semantic search over conversations, facts, and documents. It stores text with vector embeddings in SQLite and retrieves using combined BM25 (FTS5) + cosine similarity, merged via Reciprocal Rank Fusion (RRF).

**Location:** `~/Coding/Libraries/semantic-memory/`

This is a LIBRARY crate, not a binary. No `main.rs`. It exports `MemoryStore` as the primary public interface.

---

## Before Writing Any Code

**Read these files in this exact order:**

1. **`SPEC.md`** — The authoritative design document. Every type, function signature, database schema, and algorithm is defined here. 17 sections. Do not skim it.
2. **`CLAUDE.md`** — Project conventions, coding rules, tech stack, and common footguns.
3. **`AGENTS.md`** — Module-specific implementation notes and testing requirements for each file.
4. **`reference/hybrid_search.rs`** — Gloss's hybrid search implementation. The RRF fusion algorithm ports from here. Study the scoring logic before writing `search.rs`.
5. **`reference/chunk.rs`** — Gloss's text chunking. The recursive split algorithm ports from here. Note the bugs that need fixing (infinite loop, UTF-8 boundary).

---

## Implementation Plan

Build in this exact order. Each milestone must `cargo build` cleanly and pass `cargo test` before advancing to the next. Do NOT skip ahead — each module depends on the ones before it.

### Milestone 1: Foundation (error.rs, config.rs, types.rs)

1. Initialize the crate: `cargo init --lib` in `~/Coding/Libraries/semantic-memory/`
2. Set up `Cargo.toml` with exact dependencies from SPEC.md §2. Set edition = "2021".
3. Implement `error.rs` — the `MemoryError` enum exactly as defined in SPEC.md §10.
4. Implement `config.rs` — all config structs from SPEC.md §5.1. Implement `Default` for each:
   - `EmbeddingConfig::default()`: ollama_url = "http://localhost:11434", model = "nomic-embed-text", dimensions = 768, batch_size = 32, timeout_secs = 30
   - `SearchConfig::default()`: bm25_weight = 1.0, vector_weight = 1.0, rrf_k = 60.0, candidate_pool_size = 50, default_top_k = 5, min_similarity = 0.3
   - `ChunkingConfig::default()`: target_size = 1000, min_size = 100, max_size = 2000, overlap = 200
   - `MemoryConfig::default()`: database_path = "memory.db" (current dir), others = their defaults
5. Implement `types.rs` — all types from SPEC.md §6.
6. Wire up `lib.rs` with `pub mod` declarations for all modules (they can be empty stubs with `todo!()` for now).

**Test:** `cargo build` succeeds. `cargo clippy` is clean.

### Milestone 2: Database (db.rs)

1. Implement `open_database(path: &Path) -> Result<Connection, MemoryError>`:
   - Create parent directories if they don't exist
   - Open rusqlite::Connection
   - Set pragmas: WAL, foreign_keys, busy_timeout, synchronous
   - Call `run_migrations(&conn)?`
   - Return connection
2. Implement `run_migrations(conn: &Connection) -> Result<(), MemoryError>`:
   - Create `_schema_version` IF NOT EXISTS
   - Query current max version
   - If version < 1, apply V1 migration (full schema from SPEC.md §4.2)
   - Record version in `_schema_version`
3. Implement `check_embedding_metadata(conn: &Connection, config: &EmbeddingConfig) -> Result<(), MemoryError>`:
   - If singleton row exists and model/dimensions don't match → warn + update
   - If no row → insert
   - If matches → no-op

**Test:**
- Open fresh DB → all tables exist, version = 1
- Reopen same DB → migration is no-op
- Check pragmas are set (`PRAGMA journal_mode` returns "wal")
- FTS5 tables are queryable (empty MATCH query doesn't error)

### Milestone 3: Embedder (embedder.rs)

1. Define the `Embedder` trait as per SPEC.md §7.
2. Implement `MockEmbedder` — deterministic hash-based embedding with normalization.
3. Implement `OllamaEmbedder` — reqwest client calling `/api/embed`.
4. `OllamaEmbedder::embed_batch` splits into sub-batches of `batch_size`, concatenates results.
5. Validate dimensions on every response.

**Test (MockEmbedder only, no Ollama):**
- Same input → same output
- Different inputs → different outputs
- Correct dimensions
- Output is normalized (magnitude ≈ 1.0, tolerance 0.01)
- `embed_batch` with 5 inputs → 5 outputs with correct dimensions

### Milestone 4: Chunker (chunker.rs)

1. Port from `reference/chunk.rs`.
2. Implement recursive split: `\n\n` → sentence → word → force split.
3. Add merging of small adjacent chunks.
4. Add overlap between adjacent chunks.
5. Add MAX_RECURSION_DEPTH = 10 guard.
6. All string splits use `str::is_char_boundary()` for UTF-8 safety.

**Test:**
- Empty input → empty vec
- Short input (< max_size) → single chunk
- Paragraph-separated text → splits on `\n\n`
- Long sentence without paragraphs → splits on `. `
- Single long word → force-splits at max_size
- Unicode (CJK, emoji) → no panic, valid UTF-8 output
- Overlap: chunk[1] starts with tail of chunk[0]
- Small chunk merging works
- Recursion depth guard triggers on adversarial input

### Milestone 5: Conversation (conversation.rs)

1. Implement all session and message CRUD from SPEC.md §5.3.
2. `get_messages_within_budget` walks backward, accumulates tokens, reverses result.

**Test:**
- Full CRUD cycle for sessions
- Message ordering (chronological)
- Token budget limiting
- Session isolation
- CASCADE delete

### Milestone 6: Knowledge (knowledge.rs)

This is the most footgun-laden module. Follow SPEC.md §8.3 and AGENTS.md exactly.

1. Implement `add_fact` (async) and `add_fact_with_embedding` (sync).
2. Implement `delete_fact` with proper FTS cleanup sequence.
3. Implement `update_fact` (async) with transactional FTS swap.
4. Implement `delete_namespace` and `list_facts`.
5. Every FTS-touching operation is in a transaction.

**Test:**
- Insert → FTS finds it
- Update → FTS finds new content, NOT old content
- Delete → FTS returns nothing
- Bulk insert 20 → delete 10 → FTS only finds remaining 10
- Namespace filtering
- Embedding BLOB roundtrip (write, read back, decode, compare floats)

### Milestone 7: Search (search.rs)

The core algorithm. Read `reference/hybrid_search.rs` first.

1. Implement `sanitize_fts_query()` — strip FTS5 operators, split, rejoin.
2. Implement `cosine_similarity(a: &[f32], b: &[f32]) -> f32`.
3. Implement BM25 retrieval: query facts_fts + chunks_fts, join through bridge tables.
4. Implement vector retrieval: load all embeddings, compute cosine similarity, filter, sort, truncate.
5. Implement RRF fusion: HashMap-based candidate merging as per SPEC.md §8.1 Step 4.
6. Implement `search()` (async): embed query → BM25 → vector → RRF → return.
7. Implement `search_fts_only()` (sync): BM25 only, no embedding needed.
8. Implement `search_vector_only()` (async): vector only.

**Test:**
- Cosine similarity: known vectors with known results
- FTS sanitization: special chars, empty, Unicode
- RRF fusion: deterministic ranked lists → verify order (SPEC.md §13)
- Full hybrid search with MockEmbedder
- Namespace and source_type filtering
- Empty query → empty results

### Milestone 8: Documents (documents.rs)

1. Implement `ingest_document`: chunk → batch embed → transaction (document + chunks + FTS).
2. Implement `delete_document`: transactional removal of document + all chunks + FTS.
3. Implement `list_documents` with chunk count.

**Test:**
- Ingest → chunks searchable via FTS and vector
- Delete → all traces gone
- Namespace filtering

### Milestone 9: MemoryStore Facade (lib.rs)

1. Implement the `MemoryStore` struct with `Arc<Inner>` internals.
2. Wire all public methods to delegate to the module implementations.
3. `MemoryStore::open()` calls `db::open_database`, creates default OllamaEmbedder.
4. `MemoryStore::open_with_embedder()` takes a custom embedder.
5. Implement `Clone` (cheap, Arc).
6. Implement `stats()`, `reembed_all()`, `vacuum()`, `chunk_text()`, `embed()`, `embed_batch()`.

**Test:**
- End-to-end: open store → add facts → search → find them
- Store with MockEmbedder works fully offline
- Clone shares state (add via clone A, find via clone B)

### Milestone 10: Examples

1. `examples/basic_search.rs` — Create store, add 5 facts, search, print results.
2. `examples/conversation_memory.rs` — Create session, add messages, retrieve with token budget.

Both should use `OllamaEmbedder` (real Ollama) so they serve as integration smoke tests.

---

## Final Validation

Before declaring done:

```bash
cargo fmt --check
cargo clippy -- -W clippy::all
cargo test
cargo doc --no-deps  # All pub items have doc comments
cargo build --release
```

All five must pass clean.

---

## Do NOT Do These Things

- **Do NOT add HNSW or any ANN index.** Brute-force cosine similarity is fast enough.
- **Do NOT add the `rand` crate.** MockEmbedder uses a simple xorshift seeded from a hash.
- **Do NOT use `println!`.** Use `tracing` for all output.
- **Do NOT use `unwrap()` in lib code.** Only in tests.
- **Do NOT string-interpolate SQL.** Use `rusqlite::params![]` always.
- **Do NOT hold the Mutex across an await point.** Embed first, then lock and store.
- **Do NOT use the legacy Ollama `/api/embeddings` endpoint.** Use `/api/embed` with `input` array.
- **Do NOT skip the FTS content on delete.** Contentless FTS requires the original text.
- **Do NOT add optional dependencies or feature flags in V1.** Ship one thing that works.
