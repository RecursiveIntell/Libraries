//! A. Safety and Isolation tests (A1–A5)

use forge_engine::ForgeStore;
use tempfile::TempDir;

/// A1: refuse_to_open_non_forge_db
/// Create a temp SQLite file with no tables. Must refuse.
#[test]
fn a1_refuse_to_open_non_forge_db() {
    let dir = TempDir::new().unwrap();
    let db_path = dir.path().join("empty.db");

    // Create an empty SQLite database
    let conn = rusqlite::Connection::open(&db_path).unwrap();
    conn.execute_batch("SELECT 1;").unwrap();
    // Set user_version to 0 (outside valid range)
    drop(conn);

    let result = ForgeStore::open(&db_path);
    assert!(result.is_err());
    let err = result.unwrap_err();
    let msg = format!("{err}");
    assert!(
        msg.contains("refuse to open"),
        "Expected RefuseToOpenDb, got: {msg}"
    );
}

/// A2: refuse_to_open_missing_forge_meta
/// Create a temp SQLite file with a `users` table but no `forge_meta`. Must refuse.
#[test]
fn a2_refuse_to_open_missing_forge_meta() {
    let dir = TempDir::new().unwrap();
    let db_path = dir.path().join("no_meta.db");

    let conn = rusqlite::Connection::open(&db_path).unwrap();
    conn.execute_batch(
        "PRAGMA user_version = 1;
         CREATE TABLE users (id TEXT PRIMARY KEY, name TEXT);",
    )
    .unwrap();
    drop(conn);

    let result = ForgeStore::open(&db_path);
    assert!(result.is_err());
    let msg = format!("{}", result.unwrap_err());
    assert!(
        msg.contains("forge_meta") || msg.contains("refuse to open"),
        "Expected forge_meta error, got: {msg}"
    );
}

/// A3: refuse_to_open_wrong_schema_hash
/// Create a valid forge DB but manually set `forge_meta.schema_hash = 'wrong'`. Must refuse.
#[test]
fn a3_refuse_to_open_wrong_schema_hash() {
    let dir = TempDir::new().unwrap();
    let db_path = dir.path().join("wrong_hash.db");

    let conn = rusqlite::Connection::open(&db_path).unwrap();
    conn.execute_batch(
        "PRAGMA user_version = 1;
         CREATE TABLE forge_meta (key TEXT PRIMARY KEY, value TEXT NOT NULL);
         INSERT INTO forge_meta VALUES ('schema_hash', 'wrong');
         INSERT INTO forge_meta VALUES ('schema_version', '1');",
    )
    .unwrap();
    drop(conn);

    let result = ForgeStore::open(&db_path);
    assert!(result.is_err());
    let msg = format!("{}", result.unwrap_err());
    assert!(
        msg.contains("schema_hash") || msg.contains("refuse to open"),
        "Expected schema_hash mismatch, got: {msg}"
    );
}

/// A4: refuse_to_open_semantic_memory_signature
/// Create a DB that looks like semantic-memory (has `memory_entries` table, no `forge_meta`).
#[test]
fn a4_refuse_to_open_semantic_memory_signature() {
    let dir = TempDir::new().unwrap();
    let db_path = dir.path().join("memory.db");

    let conn = rusqlite::Connection::open(&db_path).unwrap();
    conn.execute_batch(
        "PRAGMA user_version = 1;
         CREATE TABLE memory_entries (id TEXT PRIMARY KEY, content TEXT);",
    )
    .unwrap();
    drop(conn);

    let result = ForgeStore::open(&db_path);
    assert!(result.is_err());
    let msg = format!("{}", result.unwrap_err());
    assert!(
        msg.contains("refuse to open"),
        "Expected RefuseToOpenDb, got: {msg}"
    );
}

/// A5: forge_writes_only_to_forge_db
/// Open a forge store, verify it creates forge.db correctly and nothing else.
#[test]
fn a5_forge_writes_only_to_forge_db() {
    let dir = TempDir::new().unwrap();
    let db_path = dir.path().join("forge.db");

    // Open should create the DB
    let store = ForgeStore::open(&db_path).unwrap();

    // Verify the DB was created
    assert!(db_path.exists());

    // Insert a candidate to exercise writes
    store
        .insert_candidate("test-candidate", "{}", "[]", "active")
        .unwrap();

    // Verify no memory.db was created
    assert!(!dir.path().join("memory.db").exists());

    // Verify only forge.db exists (plus WAL files)
    let files: Vec<_> = std::fs::read_dir(dir.path())
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().to_string())
        .collect();

    for file in &files {
        assert!(
            file.starts_with("forge.db"),
            "Unexpected file created: {file}"
        );
    }
}

/// A6: refuse_to_open_db only reads first 16 bytes (not the entire file).
/// Create a large non-SQLite file and verify it refuses fast.
#[test]
fn a6_refuse_to_open_reads_only_magic_bytes() {
    let dir = TempDir::new().unwrap();
    let db_path = dir.path().join("big_file.db");

    // Write a 1 MB file with non-SQLite content
    let content = vec![b'X'; 1_000_000];
    std::fs::write(&db_path, &content).unwrap();

    let result = ForgeStore::open(&db_path);
    assert!(result.is_err());
    let msg = format!("{}", result.unwrap_err());
    assert!(
        msg.contains("not a valid SQLite database"),
        "Expected magic bytes check failure, got: {msg}"
    );
}

/// Verify that with_transaction commits atomically and rolls back on error.
#[test]
fn transaction_commit_and_rollback() {
    let dir = TempDir::new().unwrap();
    let db_path = dir.path().join("forge.db");
    let store = ForgeStore::open(&db_path).unwrap();

    // Successful transaction should commit
    store
        .with_transaction(|conn| {
            conn.execute(
                "INSERT INTO candidates (candidate_id, spec_json, parents_json, created_at, status) VALUES ('tx-ok', '{}', '[]', '2024-01-01', 'active')",
                [],
            )?;
            Ok(())
        })
        .unwrap();

    let spec = store.get_candidate_spec("tx-ok").unwrap();
    assert_eq!(spec, "{}");

    // Failed transaction should rollback
    let result: Result<(), _> = store.with_transaction(|conn| {
        conn.execute(
            "INSERT INTO candidates (candidate_id, spec_json, parents_json, created_at, status) VALUES ('tx-fail', '{}', '[]', '2024-01-01', 'active')",
            [],
        )?;
        Err(forge_engine::ForgeError::Other("rollback".into()))
    });
    assert!(result.is_err());

    // tx-fail should not exist due to rollback
    let not_found = store.get_candidate_spec("tx-fail");
    assert!(not_found.is_err());
}

/// Verify that a newly created Forge DB can be reopened successfully.
#[test]
fn forge_db_round_trip() {
    let dir = TempDir::new().unwrap();
    let db_path = dir.path().join("forge.db");

    // Create
    {
        let store = ForgeStore::open(&db_path).unwrap();
        store
            .insert_candidate("c1", r#"{"k": 5.0}"#, "[]", "active")
            .unwrap();
    }

    // Reopen
    {
        let store = ForgeStore::open(&db_path).unwrap();
        let spec = store.get_candidate_spec("c1").unwrap();
        assert!(spec.contains("5.0"));
    }
}
