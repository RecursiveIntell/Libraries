use semantic_memory::{MemoryConfig, MemoryStore, MockEmbedder};
use tempfile::TempDir;

fn config(base_dir: std::path::PathBuf) -> MemoryConfig {
    MemoryConfig {
        base_dir,
        ..Default::default()
    }
}

#[tokio::test]
async fn existing_read_only_store_searches_without_accepting_mutations() {
    let temp = TempDir::new().unwrap();
    let base_dir = temp.path().join("memory");
    let writable =
        MemoryStore::open_with_embedder(config(base_dir.clone()), Box::new(MockEmbedder::new(768)))
            .unwrap();
    writable
        .add_fact("private", "profile-owned evidence", None, None)
        .await
        .unwrap();
    drop(writable);

    let read_only = MemoryStore::open_existing_read_only_with_embedder(
        config(base_dir),
        Box::new(MockEmbedder::new(768)),
    )
    .unwrap();
    let results = read_only
        .search("profile-owned evidence", Some(1), None, None)
        .await
        .unwrap();
    assert_eq!(results.len(), 1);
    assert!(read_only
        .add_fact("private", "must not persist", None, None)
        .await
        .is_err());
}
