# V2 Testing Addendum — semantic-memory

New and modified tests required by the V2 patch. These supplement existing tests — do NOT delete any existing tests. All existing tests must continue to pass (with `async` adaptation where sync methods became async).

---

## Existing Test Adaptation (Fix 2)

When previously-sync methods become async, every existing test that calls them needs `.await`. The test functions must also become `async`:

```rust
// BEFORE
#[test]
fn test_create_session() {
    let store = setup_store();
    let id = store.create_session("test").unwrap();
    assert!(!id.is_empty());
}

// AFTER
#[tokio::test]
async fn test_create_session() {
    let store = setup_store();
    let id = store.create_session("test").await.unwrap();
    assert!(!id.is_empty());
}
```

**Apply this to every existing test** in:
- `tests/conversation_tests.rs`
- `tests/knowledge_tests.rs`
- `tests/search_tests.rs`
- `tests/conversation_search_tests.rs`
- `tests/integration_tests.rs`

Change `#[test]` → `#[tokio::test]` and add `.await` after every `MemoryStore` method call.

---

## New Test File: `tests/tokenizer_tests.rs`

```
#[cfg(test)] setup:
- No store needed for trait tests
- MemoryStore with custom TokenCounter for integration
```

### Test Cases

**1. EstimateTokenCounter basic behavior**
```rust
#[test]
fn test_estimate_counter_basic() {
    let counter = EstimateTokenCounter;
    assert_eq!(counter.count_tokens(""), 0);
    assert_eq!(counter.count_tokens("hi"), 1); // len=2, /4=0, max(1)=1
    assert_eq!(counter.count_tokens("hello world test"), 4); // len=16, /4=4
}
```

**2. Custom TokenCounter plugs in**
```rust
struct WordCounter;
impl TokenCounter for WordCounter {
    fn count_tokens(&self, text: &str) -> usize {
        text.split_whitespace().count()
    }
}

#[tokio::test]
async fn test_custom_token_counter() {
    let config = MemoryConfig {
        token_counter: Some(Arc::new(WordCounter)),
        ..Default::default()
    };
    // Set up store with custom counter
    // Add a message with token_count = None
    // Verify the stored token_count uses WordCounter, not chars/4
}
```

**3. Token counter affects chunking**
```rust
#[test]
fn test_chunk_token_counts_use_counter() {
    let counter = WordCounter;
    let text = "one two three four five six seven eight nine ten";
    let chunks = chunk_text(text, &ChunkingConfig::default(), &counter);
    // Each chunk's token_count_estimate should reflect word count, not chars/4
    for chunk in &chunks {
        let expected = chunk.content.split_whitespace().count();
        assert_eq!(chunk.token_count_estimate, expected);
    }
}
```

**4. Auto-computed token count on add_message**
```rust
#[tokio::test]
async fn test_auto_token_count() {
    let store = setup_store(); // uses EstimateTokenCounter by default
    let session = store.create_session("test").await.unwrap();
    // Add message with token_count = None
    store.add_message(&session, Role::User, "hello world testing", None, None).await.unwrap();
    let messages = store.get_recent_messages(&session, 10).await.unwrap();
    // Should have auto-computed token count (19 chars / 4 ≈ 4), not None
    assert!(messages[0].token_count.is_some());
    assert!(messages[0].token_count.unwrap() > 0);
}
```

---

## New Tests in: `tests/search_tests.rs`

### Test Cases

**5. Buffer reuse doesn't affect search results (Fix 6 regression)**
```rust
#[tokio::test]
async fn test_vector_search_buffer_reuse_correctness() {
    // Insert 100 facts with known embeddings
    // Search and verify results are identical to V1.1 behavior
    // (scores, ordering, content all match)
    // This proves the buffer reuse optimization doesn't change results
}
```

**6. Large row count warning doesn't prevent results (Fix 9)**
```rust
#[tokio::test]
async fn test_vector_search_completes_with_many_rows() {
    // Insert 100 facts (can't easily test 50K in unit tests)
    // Search should succeed — the warning threshold is about logging, not blocking
    // Verify all expected results are returned
}
```

---

## New Tests in: `tests/integration_tests.rs`

### Test Cases

**7. V3 migration applies cleanly on existing DB**
```rust
#[tokio::test]
async fn test_v3_migration() {
    // Open DB (triggers V1 + V2 + V3)
    // Verify embedding_metadata has embeddings_dirty column
    // Verify default value is 0 (not dirty)
}
```

**8. Embedding dirty flag lifecycle**
```rust
#[tokio::test]
async fn test_embedding_dirty_flag() {
    // Open store with model "model-a" at 128 dims
    let store_a = open_store("model-a", 128);
    store_a.add_fact("ns", "test fact", None, None).await.unwrap();
    assert!(!store_a.embeddings_are_dirty().await.unwrap());

    // Reopen with different model — triggers mismatch
    drop(store_a);
    let store_b = open_store_at_same_path("model-b", 256);
    assert!(store_b.embeddings_are_dirty().await.unwrap());

    // Reembed clears the flag
    store_b.reembed_all().await.unwrap();
    assert!(!store_b.embeddings_are_dirty().await.unwrap());
}
```

**9. reembed_all now includes messages (Fix 4)**
```rust
#[tokio::test]
async fn test_reembed_all_includes_messages() {
    let store = setup_store();
    let session = store.create_session("test").await.unwrap();

    // Add an embedded message
    store.add_message_embedded(&session, Role::User, "fluid dynamics", None, None).await.unwrap();

    // Add a non-embedded message (should be skipped)
    store.add_message(&session, Role::User, "not embedded", None, None).await.unwrap();

    // reembed_all should count the embedded message
    let count = store.reembed_all().await.unwrap();
    assert!(count >= 1); // At least the one embedded message
}
```

**10. Silent coercion is gone (Fix 3)**

This test requires mocking Ollama, which is complex. Instead, test the parsing logic directly if possible, or mark as `#[ignore]` with a note:

```rust
#[tokio::test]
#[ignore] // Requires a mock HTTP server or Ollama running
async fn test_non_numeric_embedding_rejected() {
    // Set up OllamaEmbedder pointing at a mock server that returns
    // {"embeddings": [[1.0, "not_a_number", 3.0]]}
    // Verify embed() returns Err, not a vector with 0.0 in position 1
}
```

For a unit-testable version, extract the parsing logic into a standalone function:
```rust
// In embedder.rs
pub(crate) fn parse_embedding_response(
    body: &serde_json::Value,
    expected_dims: usize,
) -> Result<Vec<Vec<f32>>, MemoryError> { ... }

// In tests:
#[test]
fn test_parse_rejects_non_numeric() {
    let body = serde_json::json!({
        "embeddings": [[1.0, "bad", 3.0]]
    });
    let result = parse_embedding_response(&body, 3);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("non-numeric"));
}

#[test]
fn test_parse_valid_embedding() {
    let body = serde_json::json!({
        "embeddings": [[1.0, 2.0, 3.0]]
    });
    let result = parse_embedding_response(&body, 3);
    assert!(result.is_ok());
    assert_eq!(result.unwrap()[0], vec![1.0f32, 2.0, 3.0]);
}
```

**Recommended:** Extract the parsing function. It makes the code more testable AND cleaner.

---

## New Tests in: `tests/db_tests.rs` (NEW FILE)

### Test Cases

**11. bytes_to_embedding stable Rust compatibility (Fix 1)**
```rust
#[test]
fn test_bytes_to_embedding_valid() {
    let original = vec![1.0f32, 2.0, 3.0];
    let bytes = embedding_to_bytes(&original);
    let decoded = bytes_to_embedding(&bytes).unwrap();
    assert_eq!(original, decoded);
}

#[test]
fn test_bytes_to_embedding_invalid_length() {
    let bytes = vec![0u8; 5]; // Not divisible by 4
    let result = bytes_to_embedding(&bytes);
    assert!(result.is_err());
}

#[test]
fn test_bytes_to_embedding_empty() {
    let bytes: Vec<u8> = vec![];
    let decoded = bytes_to_embedding(&bytes).unwrap();
    assert!(decoded.is_empty());
}
```

**12. embeddings_dirty default is false**
```rust
#[tokio::test]
async fn test_fresh_db_not_dirty() {
    let store = setup_store();
    assert!(!store.embeddings_are_dirty().await.unwrap());
}
```

---

## Modified Tests: Role Trait Impls

### In `tests/conversation_tests.rs` or a new `tests/types_tests.rs`

**13. Role Display**
```rust
#[test]
fn test_role_display() {
    assert_eq!(format!("{}", Role::User), "user");
    assert_eq!(format!("{}", Role::Assistant), "assistant");
    assert_eq!(format!("{}", Role::System), "system");
    assert_eq!(format!("{}", Role::Tool), "tool");
}
```

**14. Role FromStr**
```rust
#[test]
fn test_role_from_str() {
    assert_eq!("user".parse::<Role>().unwrap(), Role::User);
    assert_eq!("assistant".parse::<Role>().unwrap(), Role::Assistant);
    assert!("invalid".parse::<Role>().is_err());
}
```

---

## Test Utilities Update

### Async Setup Helper

The existing `setup_store` helper (if any) should return a store that works in async tests:

```rust
fn setup_store() -> MemoryStore {
    let tmp = tempfile::NamedTempFile::new().unwrap();
    let config = MemoryConfig {
        database_path: tmp.path().to_path_buf(),
        embedding: EmbeddingConfig {
            dimensions: 128,
            ..Default::default()
        },
        ..Default::default()
    };
    let embedder = Box::new(MockEmbedder::new(128));
    MemoryStore::open_with_embedder(config, embedder).unwrap()
}
```

**Note:** `tempfile::NamedTempFile` will delete the file when dropped. If your test needs to reopen the same DB (like test #8), use `tempfile::TempDir` instead and construct the path manually:

```rust
fn setup_store_in_dir(dir: &Path) -> MemoryStore {
    let config = MemoryConfig {
        database_path: dir.join("test.db"),
        ..Default::default()
    };
    // ...
}
```

### Embedding Response Parser Test Helper

For Fix 3 tests, if you extract the parsing function:
```rust
// This should be pub(crate) in embedder.rs so tests can call it
pub(crate) fn parse_embedding_response(
    body: &serde_json::Value,
    expected_dims: usize,
) -> Result<Vec<Vec<f32>>, MemoryError>
```

Tests import via `use semantic_memory::embedder::parse_embedding_response;` — but since it's `pub(crate)`, integration tests (in `tests/`) can't see it. Either:
- Make it `pub` with `#[doc(hidden)]`
- Or test it through the `MockEmbedder` path (less ideal but works)

**Recommended:** Make it `pub` with `#[doc(hidden)]` and a `// Test-accessible parsing function` comment.
