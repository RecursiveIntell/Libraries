use semantic_memory::{
    MemoryConfig, MemoryStore, MockEmbedder, ReceiptMode, SearchContext, VerifyMode,
};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

fn config(base_dir: PathBuf) -> MemoryConfig {
    MemoryConfig {
        base_dir,
        ..Default::default()
    }
}

fn snapshot(root: &Path) -> BTreeMap<String, Vec<u8>> {
    let mut files = BTreeMap::new();
    if !root.exists() {
        return files;
    }
    let mut pending = vec![root.to_path_buf()];
    while let Some(path) = pending.pop() {
        let metadata = fs::symlink_metadata(&path).expect("metadata");
        if metadata.is_dir() {
            let mut children: Vec<_> = fs::read_dir(&path)
                .expect("read directory")
                .map(|entry| entry.expect("directory entry").path())
                .collect();
            children.sort();
            pending.extend(children.into_iter().rev());
        } else {
            let relative = path
                .strip_prefix(root)
                .expect("snapshot path under root")
                .to_string_lossy()
                .into_owned();
            files.insert(relative, fs::read(&path).expect("read snapshot file"));
        }
    }
    files
}

fn durable_files(files: &BTreeMap<String, Vec<u8>>) -> BTreeMap<String, Vec<u8>> {
    files
        .iter()
        .filter(|(path, _)| !path.ends_with("-wal") && !path.ends_with("-shm"))
        .map(|(path, bytes)| (path.clone(), bytes.clone()))
        .collect()
}

fn writable_store(base: &Path) -> MemoryStore {
    MemoryStore::open_with_embedder(config(base.to_path_buf()), Box::new(MockEmbedder::new(768)))
        .expect("open writable store")
}

#[tokio::test]
async fn missing_database_fails_without_creating_store_paths() {
    let temp = TempDir::new().expect("temporary directory");
    let base = temp.path().join("missing");
    assert!(!base.exists());

    let result = MemoryStore::open_existing_read_only_with_embedder(
        config(base.clone()),
        Box::new(MockEmbedder::new(768)),
    );

    assert!(result.is_err(), "missing database must be refused");
    assert!(
        !base.exists(),
        "read-only open must not create the base path"
    );
}

#[tokio::test]
async fn read_only_query_refuses_persistence_and_preserves_files_on_drop() {
    let temp = TempDir::new().expect("temporary directory");
    let base = temp.path().join("memory");
    let writable = writable_store(&base);
    writable
        .add_fact("private", "profile-owned evidence", None, None)
        .await
        .expect("seed fact");
    drop(writable);

    let before = snapshot(&base);
    assert!(
        !before.is_empty(),
        "writable fixture must create a database"
    );

    let read_only = MemoryStore::open_existing_read_only_with_embedder(
        config(base.clone()),
        Box::new(MockEmbedder::new(768)),
    )
    .expect("open existing store read-only");
    let results = read_only
        .search("profile-owned evidence", Some(1), None, None)
        .await
        .expect("read-only query");
    assert_eq!(results.len(), 1);
    assert!(
        read_only
            .add_fact("private", "must not persist", None, None)
            .await
            .is_err(),
        "read-only write must be refused"
    );
    let report = read_only
        .verify_integrity(VerifyMode::Quick)
        .await
        .expect("read-only quick integrity check");
    assert!(report.ok, "read-only quick integrity check: {report:?}");
    drop(read_only);

    let after = snapshot(&base);
    assert_eq!(
        durable_files(&before),
        durable_files(&after),
        "read-only query/drop changed durable files; SQLite WAL/SHM sidecars are reported separately",
    );
}

#[tokio::test]
async fn receipt_enabled_search_is_refused_without_persisting_a_receipt() {
    let temp = TempDir::new().expect("temporary directory");
    let base = temp.path().join("memory");
    let writable = writable_store(&base);
    writable
        .add_fact("private", "receipt refusal fixture", None, None)
        .await
        .expect("seed fact");
    drop(writable);
    let before = snapshot(&base);

    let read_only = MemoryStore::open_existing_read_only_with_embedder(
        config(base.clone()),
        Box::new(MockEmbedder::new(768)),
    )
    .expect("open existing store read-only");
    let mut context = SearchContext::default_now();
    context.receipt_mode = ReceiptMode::ReturnReceipt;
    let result = read_only
        .search_with_context("receipt refusal fixture", Some(1), None, None, context)
        .await;
    assert!(result.is_err(), "receipt persistence must be refused");
    drop(read_only);

    let after = snapshot(&base);
    assert_eq!(
        durable_files(&before),
        durable_files(&after),
        "refused receipt search changed durable files; SQLite WAL/SHM sidecars are reported separately",
    );
}

#[tokio::test]
async fn read_only_wal_shm_behavior_is_explicitly_observed() {
    let temp = TempDir::new().expect("temporary directory");
    let base = temp.path().join("memory");
    let writable = writable_store(&base);
    writable
        .add_fact("private", "WAL sidecar fixture", None, None)
        .await
        .expect("seed fact");
    drop(writable);
    let before = snapshot(&base);

    let read_only = MemoryStore::open_existing_read_only_with_embedder(
        config(base.clone()),
        Box::new(MockEmbedder::new(768)),
    )
    .expect("open existing store read-only");
    read_only
        .search("WAL sidecar fixture", Some(1), None, None)
        .await
        .expect("read-only query");
    drop(read_only);

    let after = snapshot(&base);
    let sidecars: Vec<_> = after
        .keys()
        .filter(|path| path.ends_with("-wal") || path.ends_with("-shm"))
        .cloned()
        .collect();
    assert!(
        sidecars
            .iter()
            .all(|path| path == "memory.db-wal" || path == "memory.db-shm"),
        "unexpected read-only sidecar paths: {sidecars:?}"
    );
    assert_eq!(
        durable_files(&before),
        durable_files(&after),
        "read-only WAL/SHM access changed durable files"
    );
    println!("read_only_sqlite_coordination_sidecars={sidecars:?}");
}
