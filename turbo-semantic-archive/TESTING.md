# TESTING.md — semantic-memory

## Testing Philosophy

This is a **library crate** that other projects depend on. Bugs here propagate to every consumer. Test coverage is not optional.

Every public function must have at least one happy-path test and one error-path test. FTS sync operations (the most bug-prone area) need exhaustive lifecycle tests.

---

## Test Infrastructure

### Test Helpers

Create a `tests/common/mod.rs` (or inline in each test file) with:

```rust
use semantic_memory::{MemoryConfig, MemoryStore};
use semantic_memory::embedder::MockEmbedder;
use tempfile::NamedTempFile;
use std::path::PathBuf;

/// Create a MemoryStore backed by a temporary SQLite file with MockEmbedder.
/// The temp file is deleted when the returned NamedTempFile is dropped.
pub fn test_store() -> (MemoryStore, NamedTempFile) {
    let tmp = NamedTempFile::new().unwrap();
    let config = MemoryConfig {
        database_path: PathBuf::from(tmp.path()),
        ..Default::default()
    };
    let embedder = Box::new(MockEmbedder::new(768));
    let store = MemoryStore::open_with_embedder(config, embedder).unwrap();
    (store, tmp) // caller must hold tmp to keep the file alive
}
```

### No Ollama Requirement

All unit tests use `MockEmbedder`. Tests that require a real Ollama instance use `#[ignore]`:

```rust
#[tokio::test]
#[ignore] // Requires: ollama running with nomic-embed-text pulled
async fn test_real_ollama_embedding() {
    // ...
}
```

Run ignored tests explicitly: `cargo test -- --ignored`

---

## Test Cases by Module

### chunker_tests.rs

| Test | Input | Expected |
|------|-------|----------|
| empty_input | `""` | `vec![]` |
| short_input | `"Hello world"` (< max_size) | Single chunk with that text |
| paragraph_split | Two paragraphs joined by `\n\n` | Two chunks |
| sentence_split | Long text with `. ` boundaries, no `\n\n` | Splits on sentences |
| word_split | Long text with spaces, no sentence enders | Splits on word boundaries |
| force_split | Single 5000-char string, no spaces | Force-splits at max_size |
| unicode_cjk | Chinese text: `"这是一个很长的中文句子..."` repeated | No panic, valid UTF-8 chunks |
| unicode_emoji | Text with multi-byte emoji: `"Hello 🌍🌎🌏 World"` | Correct splits, no mangled emoji |
| overlap_applied | Two chunks that should overlap | chunk[1] starts with tail of chunk[0] |
| small_merge | Three tiny chunks below min_size | Merged into fewer chunks |
| recursion_guard | Pathological input (single repeated char, max_size=1) | Doesn't hang, returns force-splits |
| preserves_content | Roundtrip: join all chunks (minus overlap) ≈ original | Content is preserved |

### conversation_tests.rs

| Test | Scenario | Assertion |
|------|----------|-----------|
| create_and_list | Create session → list | Session appears with correct channel |
| add_messages | Add 5 messages | get_recent_messages(5) returns all 5 in chrono order |
| recent_limit | Add 10 messages, get 3 | Returns last 3 chronologically |
| token_budget_exact | 5 msgs × 100 tokens, budget=200 | Returns last 2 messages |
| token_budget_null | Mix of counted and NULL token messages | NULL messages always included |
| token_budget_zero | Budget = 0 | Returns empty vec |
| session_isolation | Two sessions, messages in each | get_recent_messages per session only returns its own |
| delete_cascade | Delete session | Messages are gone too |
| session_updated_at | Add message | Session's updated_at changes |
| list_with_count | Create sessions, add varying message counts | message_count field is correct |

### knowledge_tests.rs

| Test | Scenario | Assertion |
|------|----------|-----------|
| add_and_get | Add fact, get by ID | Content matches |
| add_with_embedding | Provide pre-computed embedding | Stored BLOB decodes to same floats |
| fts_find | Add fact "Rust is a systems programming language" | FTS for "programming" finds it |
| fts_not_found | Add fact, search unrelated term | Empty results |
| update_fts_consistency | Add fact, update content, search old term | Old term NOT found |
| update_fts_new_content | Add fact, update, search new term | New term found |
| delete_fts_cleanup | Add fact, delete, search original term | Not found (no ghost entries) |
| bulk_delete | Add 20 facts, delete 10 | FTS finds only remaining 10 |
| namespace_filter | Facts in namespaces "a" and "b" | list_facts("a") only returns "a" facts |
| delete_namespace | Add 5 facts in "temp", delete_namespace("temp") | All 5 gone, returns count 5 |
| embedding_roundtrip | Add fact with embedding → read BLOB → decode | Floats match within f32 precision |
| metadata_json | Add fact with JSON metadata | get_fact returns same JSON |

### search_tests.rs

| Test | Scenario | Assertion |
|------|----------|-----------|
| cosine_identical | cosine_similarity(v, v) | 1.0 (within tolerance) |
| cosine_orthogonal | cosine_similarity([1,0,0], [0,1,0]) | 0.0 |
| cosine_opposite | cosine_similarity([1,0], [-1,0]) | -1.0 |
| cosine_zero_vector | cosine_similarity([0,0,0], [1,2,3]) | 0.0 (not NaN) |
| sanitize_clean | `"hello world"` | `Some("hello world")` |
| sanitize_operators | `"hello + world - foo"` | `Some("hello world foo")` |
| sanitize_empty | `"+-*()"` | `None` |
| sanitize_unicode | `"日本語テスト"` | `Some("日本語テスト")` |
| rrf_basic | BM25=[A,B,C], Vec=[B,D,A], k=60 | Order: B, A, D, C (per SPEC.md §13) |
| rrf_no_overlap | BM25=[A,B], Vec=[C,D] | All 4 present, BM25 rank 1 item highest |
| rrf_single_source | BM25=[A,B,C], Vec=[] | Same order as BM25, just with lower scores |
| rrf_weights | BM25 weight=2.0, Vec weight=1.0 | BM25 rank 1 item boosted |
| hybrid_search_e2e | Add 5 facts, search | Returns relevant facts ranked |
| fts_only_search | Add facts, call search_fts_only | Works without embedder |
| vector_only_search | Add facts, call search_vector_only | Works without FTS |
| namespace_filtering | Facts in two namespaces | Filtered search only returns target namespace |
| source_type_filter | Facts and chunks | Filter to Facts only → no chunks in results |
| empty_query | `""` | Empty results, no error |
| min_similarity_filter | Embeddings far from query | Below threshold → excluded |

### integration_tests.rs

| Test | Scenario | Assertion |
|------|----------|-----------|
| full_lifecycle | Add facts + ingest doc → search → update → delete → search | Complete CRUD works |
| document_ingestion | Ingest 3-page document | Chunks created, searchable |
| document_deletion | Ingest → delete | All chunks + FTS cleaned |
| stats | Add various data | stats() returns correct counts |
| reopen_database | Open, add data, drop store, reopen same file | Data persists |
| concurrent_clones | Clone store, write from clone A, read from clone B | Shared state |

---

## Running Tests

```bash
# All unit tests (no Ollama needed)
cargo test

# With output for debugging
cargo test -- --nocapture

# Only search tests
cargo test --test search_tests

# Integration tests requiring Ollama
cargo test -- --ignored

# All tests + ignored
cargo test -- --include-ignored
```

---

## Coverage Goals

| Module | Target Coverage | Rationale |
|--------|----------------|-----------|
| chunker.rs | 95%+ | Pure logic, easy to test, must handle edge cases |
| conversation.rs | 90%+ | Simple CRUD, straightforward |
| knowledge.rs | 95%+ | FTS sync bugs are subtle and destructive |
| search.rs | 90%+ | Core algorithm, but some paths hard to test without real embeddings |
| embedder.rs | 80%+ | MockEmbedder fully tested; OllamaEmbedder tested via `#[ignore]` |
| db.rs | 85%+ | Migration logic and pragma setup |
| config.rs | 100% | Just Default impls and struct definitions |
