//! Scratch-only SQLite backup boundary. No live restore/export authority.
use rusqlite::{Connection, OpenFlags};
use semantic_memory::{
    AuthorityIssuer, AuthorityPermit, IntegritySnapshotError, MemoryConfig, MemoryStore,
    MockEmbedder, VerifyMode,
};
use sha2::{Digest, Sha256};
use std::path::PathBuf;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use tempfile::TempDir;
struct NoEmbedding(Arc<AtomicUsize>);
impl semantic_memory::Embedder for NoEmbedding {
    fn embed<'a>(&'a self, _text: &'a str) -> semantic_memory::EmbedFuture<'a> {
        self.0.fetch_add(1, Ordering::SeqCst);
        Box::pin(async {
            Err(semantic_memory::MemoryError::Other(
                "snapshot embedding forbidden".into(),
            ))
        })
    }
    fn embed_batch<'a>(&'a self, _texts: Vec<String>) -> semantic_memory::EmbedBatchFuture<'a> {
        self.0.fetch_add(1, Ordering::SeqCst);
        Box::pin(async {
            Err(semantic_memory::MemoryError::Other(
                "snapshot embedding forbidden".into(),
            ))
        })
    }
    fn model_name(&self) -> &str {
        "snapshot-reader-disabled"
    }
    fn dimensions(&self) -> usize {
        768
    }
}

type TestResult = Result<(), Box<dyn std::error::Error>>;
const CAP: u64 = 128 * 1024 * 1024;

struct Fixture {
    root: TempDir,
    source: PathBuf,
    destination: PathBuf,
    writer: MemoryStore,
    reader: MemoryStore,
    fact_id: String,
    embedding_calls: Arc<AtomicUsize>,
}
fn config(base_dir: PathBuf) -> MemoryConfig {
    MemoryConfig {
        base_dir,
        ..Default::default()
    }
}
async fn fixture() -> Result<Fixture, Box<dyn std::error::Error>> {
    let root = TempDir::new()?;
    let source = root.path().join("source");
    let destination = root.path().join("snapshot");
    std::fs::create_dir(&destination)?;
    let writer =
        MemoryStore::open_with_embedder(config(source.clone()), Box::new(MockEmbedder::new(768)))?;
    let issuer =
        AuthorityIssuer::from_operator_token("snapshot-test-only").ok_or("fixture issuer")?;
    let receipt = writer
        .authority()
        .append(
            issuer.mint_operator_system(
                "principal:fixture",
                "snapshot-test",
                AuthorityPermit::APPEND_CAPABILITY,
            ),
            "snapshot-seed".into(),
            "general".into(),
            "uncheckpointed snapshot sentinel".into(),
            None,
        )
        .await?;
    let fact_id = receipt
        .affected_ids
        .first()
        .ok_or("missing appended fact")?
        .clone();
    let embedding_calls = Arc::new(AtomicUsize::new(0));
    let reader = MemoryStore::open_existing_read_only_with_embedder(
        config(source.clone()),
        Box::new(NoEmbedding(embedding_calls.clone())),
    )?;
    Ok(Fixture {
        root,
        source,
        destination,
        writer,
        reader,
        fact_id,
        embedding_calls,
    })
}

#[tokio::test]
async fn read_only_owner_reports_its_own_sqlite_connection_settings() -> TestResult {
    let f = fixture().await?;
    let result = f.reader.sqlite_connection_diagnostic().await?;
    assert_eq!(result.connection_role, "reader");
    assert!(result.read_only);
    assert!(result.foreign_keys_enabled);
    assert!(result.query_only);
    assert_eq!(result.journal_mode, "wal");
    assert!(!result.sqlite_version.is_empty());
    assert!(!result.sqlite_source_id.is_empty());
    assert!(!result.compile_options.is_empty());
    assert_eq!(result.schema_version, 39);
    let writer = f.writer.sqlite_writer_connection_diagnostic().await?;
    assert_eq!(writer.connection_role, "writer");
    assert!(!writer.read_only);
    assert!(!writer.query_only);
    assert!(writer.foreign_keys_enabled);
    assert_eq!(writer.journal_mode, "wal");
    assert_eq!(writer.sqlite_source_id, result.sqlite_source_id);
    assert_eq!(writer.compile_options, result.compile_options);
    let writable_reader = f.writer.sqlite_connection_diagnostic().await?;
    assert_eq!(writable_reader.connection_role, "reader");
    assert!(!writable_reader.read_only);
    assert!(writable_reader.query_only);
    assert!(writable_reader.foreign_keys_enabled);
    assert_eq!(writable_reader.sqlite_source_id, writer.sqlite_source_id);
    assert!(matches!(
        f.reader.sqlite_writer_connection_diagnostic().await,
        Err(semantic_memory::SqliteDiagnosticError::RequiresWritableStore)
    ));
    assert_eq!(f.embedding_calls.load(Ordering::SeqCst), 0);
    Ok(())
}

#[tokio::test]
async fn captures_wal_and_opens_detached_through_the_canonical_owner() -> TestResult {
    let f = fixture().await?;
    assert!(std::fs::metadata(f.source.join("memory.db-wal"))?.len() > 0);
    // Pin a witness that ignoring WAL does not see the committed owner append.
    let raw = Connection::open_with_flags(
        format!("file:{}?immutable=1", f.source.join("memory.db").display()),
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_URI,
    )?;
    let absent = raw.query_row(
        "SELECT COUNT(*) FROM facts WHERE id = ?1",
        [&f.fact_id],
        |row| row.get::<_, i64>(0),
    );
    match absent {
        Ok(0) => {}
        Err(rusqlite::Error::SqliteFailure(_, Some(message)))
            if message.contains("no such table: facts") => {}
        value => {
            return Err(format!("fixture did not leave the append WAL-dependent: {value:?}").into())
        }
    }
    drop(raw);
    let source_main = std::fs::read(f.source.join("memory.db"))?;
    let source_wal = std::fs::read(f.source.join("memory.db-wal"))?;
    let state = f.reader.authority().current_state().await?;
    let target = f.destination.join("memory.db");
    let report = f.reader.create_integrity_snapshot(&target, CAP).await?;
    assert_eq!(
        report.schema_version,
        "semantic_memory_integrity_snapshot_v1"
    );
    let bytes = std::fs::read(&target)?;
    assert_eq!(
        report.database_sha256,
        format!("sha256:{:x}", Sha256::digest(&bytes))
    );
    assert_eq!(report.database_size_bytes, u64::try_from(bytes.len())?);
    assert!(report.sqlite_user_version > 0);
    assert_eq!(state, f.reader.authority().current_state().await?);
    assert_eq!(std::fs::read(f.source.join("memory.db"))?, source_main);
    assert_eq!(std::fs::read(f.source.join("memory.db-wal"))?, source_wal);
    assert_eq!(report.directory_synced, cfg!(unix));
    assert_eq!(f.embedding_calls.load(Ordering::SeqCst), 0);
    drop(f.reader);
    drop(f.writer);
    std::fs::rename(&f.source, f.root.path().join("source-unavailable"))?;
    let copy = MemoryStore::open_existing_read_only_with_embedder(
        config(f.destination),
        Box::new(MockEmbedder::new(768)),
    )?;
    let integrity = copy.verify_integrity(VerifyMode::Full).await?;
    if cfg!(feature = "hnsw") {
        // This contract copies SQLite, deliberately not derived HNSW files.
        // The owner must expose that degradation, not certify a clean store.
        assert!(!integrity.ok);
        assert_eq!(
            integrity.issues,
            vec![
                "1 pending HNSW sidecar ops queued in SQLite".to_string(),
                format!(
                    "pending sidecar op: fact upsert fact:{} (attempts: 0)",
                    f.fact_id
                ),
                "HNSW sidecar files are missing while 1 embedded rows exist in SQLite".to_string(),
                format!(
                    "HNSW keymap missing live embedded SQLite row: fact:{}",
                    f.fact_id
                ),
                "HNSW keymap drift: 0 active keymap rows vs 1 embedded SQLite rows".to_string(),
            ]
        );
    } else {
        assert!(integrity.ok, "{:?}", integrity.issues);
    }
    let sql = Connection::open_with_flags(&target, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
    let integrity_rows = sql
        .prepare("PRAGMA integrity_check")?
        .query_map([], |row| row.get::<_, String>(0))?
        .collect::<Result<Vec<_>, _>>()?;
    assert_eq!(integrity_rows, vec!["ok"]);
    let fk_count: i64 =
        sql.query_row("SELECT COUNT(*) FROM pragma_foreign_key_check", [], |row| {
            row.get(0)
        })?;
    assert_eq!(fk_count, 0);
    drop(sql);
    assert_eq!(state, copy.authority().current_state().await?);
    let access = semantic_memory::GovernedAccessRequestV1::new(
        "principal:fixture",
        "principal:fixture",
        semantic_memory::GovernedAccessPurposeV1::Recall,
        "general",
    );
    let result = copy
        .authority()
        .get_fact_governed(&f.fact_id, access)
        .await?;
    assert!(result.decision.allowed);
    assert_eq!(
        result.fact.ok_or("detached fact missing")?.content,
        "uncheckpointed snapshot sentinel"
    );
    assert_eq!(
        std::fs::read(target)?,
        bytes,
        "read-only validation changed image bytes"
    );
    Ok(())
}

#[tokio::test]
async fn consistent_snapshot_preserves_corruption_instead_of_repairing_it() -> TestResult {
    let f = fixture().await?;
    let raw = Connection::open(f.source.join("memory.db"))?;
    raw.execute_batch("PRAGMA foreign_keys=OFF; INSERT INTO authority_lineages VALUES ('orphan-lineage', 'missing-parent', 0);")?;
    drop(raw);
    let before = f.reader.verify_integrity(VerifyMode::Full).await?;
    assert!(!before.ok);
    let target = f.destination.join("memory.db");
    f.reader.create_integrity_snapshot(&target, CAP).await?;
    let copy = MemoryStore::open_existing_read_only_with_embedder(
        config(f.destination),
        Box::new(MockEmbedder::new(768)),
    )?;
    let after = copy.verify_integrity(VerifyMode::Full).await?;
    assert!(!after.ok);
    assert_eq!(before.issues, after.issues);
    assert_eq!(
        before.issues,
        f.reader.verify_integrity(VerifyMode::Full).await?.issues
    );
    let conn = Connection::open_with_flags(&target, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
    let parent_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM facts WHERE id='missing-parent'",
        [],
        |r| r.get(0),
    )?;
    assert_eq!(
        parent_count, 0,
        "snapshot must never synthesize a missing parent"
    );
    Ok(())
}

#[tokio::test]
async fn refuses_writable_source_zero_limit_and_too_small_budget_without_output() -> TestResult {
    let f = fixture().await?;
    let target = f.destination.join("memory.db");
    assert!(matches!(
        f.writer.create_integrity_snapshot(&target, CAP).await,
        Err(IntegritySnapshotError::RequiresReadOnlyStore)
    ));
    assert!(matches!(
        f.reader.create_integrity_snapshot(&target, 0).await,
        Err(IntegritySnapshotError::InvalidLimit)
    ));
    assert!(matches!(
        f.reader.create_integrity_snapshot(&target, 1).await,
        Err(IntegritySnapshotError::LimitExceeded { .. })
    ));
    assert!(!target.exists());
    assert_eq!(std::fs::read_dir(&f.destination)?.count(), 0);
    Ok(())
}

#[tokio::test]
async fn never_replaces_an_existing_destination_or_writes_inside_source() -> TestResult {
    let f = fixture().await?;
    let target = f.destination.join("memory.db");
    std::fs::write(&target, b"preserved destination")?;
    assert!(matches!(
        f.reader.create_integrity_snapshot(&target, CAP).await,
        Err(IntegritySnapshotError::DestinationExists)
    ));
    assert_eq!(std::fs::read(&target)?, b"preserved destination");
    assert!(matches!(
        f.reader
            .create_integrity_snapshot(f.source.join("not-a-sidecar.db"), CAP)
            .await,
        Err(IntegritySnapshotError::SourceDestinationOverlap)
    ));
    assert!(!f.source.join("not-a-sidecar.db").exists());
    assert_eq!(std::fs::read_dir(&f.destination)?.count(), 1);
    Ok(())
}

#[cfg(unix)]
#[tokio::test]
async fn refuses_symlink_destination_without_following_it() -> TestResult {
    let f = fixture().await?;
    let target = f.destination.join("memory.db");
    let victim = f.root.path().join("victim");
    std::fs::write(&victim, b"untouched")?;
    std::os::unix::fs::symlink(&victim, &target)?;
    assert!(matches!(
        f.reader.create_integrity_snapshot(&target, CAP).await,
        Err(IntegritySnapshotError::DestinationExists)
    ));
    assert_eq!(std::fs::read(victim)?, b"untouched");
    Ok(())
}

#[cfg(unix)]
#[tokio::test]
async fn refuses_source_containment_through_a_symlink_parent() -> TestResult {
    let f = fixture().await?;
    let alias = f.destination.join("source-alias");
    std::os::unix::fs::symlink(&f.source, &alias)?;
    assert!(matches!(
        f.reader
            .create_integrity_snapshot(alias.join("forbidden.db"), CAP)
            .await,
        Err(IntegritySnapshotError::SourceDestinationOverlap)
    ));
    assert!(!f.source.join("forbidden.db").exists());
    Ok(())
}

#[tokio::test]
async fn competing_publications_keep_one_complete_image() -> TestResult {
    let f = fixture().await?;
    let target = f.destination.join("memory.db");
    let (a, b) = tokio::join!(
        f.reader.create_integrity_snapshot(&target, CAP),
        f.reader.create_integrity_snapshot(&target, CAP),
    );
    let successful = match (a, b) {
        (Ok(report), Err(IntegritySnapshotError::DestinationExists))
        | (Err(IntegritySnapshotError::DestinationExists), Ok(report)) => report,
        value => return Err(format!("expected exactly one publication: {value:?}").into()),
    };
    assert_eq!(
        successful.database_sha256,
        format!("sha256:{:x}", Sha256::digest(std::fs::read(&target)?))
    );
    assert_eq!(std::fs::read_dir(&f.destination)?.count(), 1);
    Ok(())
}
