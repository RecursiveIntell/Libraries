//! Trusted-local, read-only SQLite snapshots for integrity investigation.
//!
//! The image contains truth-bearing rows, not merely derived indexes. The
//! unsigned metadata below is NOT authorization to export remotely, restore,
//! repair, or recall it. No transport endpoint or restore API is registered.
use crate::{MemoryError, MemoryStore};
use rusqlite::{
    backup::{Backup, StepResult},
    Connection,
};
use serde::Serialize;
use sha2::{Digest, Sha256};
#[cfg(unix)]
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use tempfile::{Builder, NamedTempFile};

/// An observation of one canonical SQLite pool connection in this process.
/// `read_only` describes the store constructor, while `query_only` is read from
/// the acquired connection. This is not a claim about another process or binary.
#[derive(Debug, Clone, Serialize)]
pub struct SqliteConnectionDiagnosticV1 {
    pub connection_role: &'static str,
    pub sqlite_version: String,
    pub sqlite_source_id: String,
    pub compile_options: Vec<String>,
    pub journal_mode: String,
    pub foreign_keys_enabled: bool,
    pub query_only: bool,
    pub read_only: bool,
    pub schema_version: u32,
}

#[derive(Debug, thiserror::Error)]
pub enum SqliteDiagnosticError {
    #[error("writer diagnostic requires a writable store")]
    RequiresWritableStore,
    #[error("owner connection unavailable: {0}")]
    Owner(#[from] MemoryError),
    #[error("SQLite diagnostic query failed: {0}")]
    Database(#[from] rusqlite::Error),
}

fn observe_sqlite_connection(
    conn: &Connection,
    connection_role: &'static str,
    read_only: bool,
) -> Result<SqliteConnectionDiagnosticV1, rusqlite::Error> {
    let sqlite_version = conn.query_row("SELECT sqlite_version()", [], |r| r.get(0))?;
    let sqlite_source_id = conn.query_row("SELECT sqlite_source_id()", [], |r| r.get(0))?;
    let journal_mode = conn.query_row("PRAGMA journal_mode", [], |r| r.get(0))?;
    let foreign_keys_enabled = conn.query_row("PRAGMA foreign_keys", [], |r| r.get(0))?;
    let query_only = conn.query_row("PRAGMA query_only", [], |r| r.get(0))?;
    let schema_version = conn.query_row("PRAGMA user_version", [], |r| r.get(0))?;
    let mut compile_options = conn
        .prepare("PRAGMA compile_options")?
        .query_map([], |r| r.get::<_, String>(0))?
        .collect::<Result<Vec<_>, _>>()?;
    compile_options.sort();
    Ok(SqliteConnectionDiagnosticV1 {
        connection_role,
        sqlite_version,
        sqlite_source_id,
        compile_options,
        journal_mode,
        foreign_keys_enabled,
        query_only,
        read_only,
        schema_version,
    })
}

impl MemoryStore {
    /// Observe the canonical reader pool of this store through read queries.
    /// An already-open writable store can report its query-only readers here;
    /// opening a new writable store may migrate and is a separate admission.
    pub async fn sqlite_connection_diagnostic(
        &self,
    ) -> Result<SqliteConnectionDiagnosticV1, SqliteDiagnosticError> {
        let store_read_only = self.inner.read_only;
        self.with_read_conn(move |conn| {
            Ok(observe_sqlite_connection(conn, "reader", store_read_only))
        })
        .await?
        .map_err(SqliteDiagnosticError::Database)
    }

    /// Observe the canonical writer connection using only read queries.
    /// Opening a writable store may already have migrated it; this does not
    /// make an unapproved live store safe to open for diagnosis.
    pub async fn sqlite_writer_connection_diagnostic(
        &self,
    ) -> Result<SqliteConnectionDiagnosticV1, SqliteDiagnosticError> {
        if self.inner.read_only {
            return Err(SqliteDiagnosticError::RequiresWritableStore);
        }
        self.with_write_conn(|conn| Ok(observe_sqlite_connection(conn, "writer", false)))
            .await?
            .map_err(SqliteDiagnosticError::Database)
    }
}

/// Unsigned file-operation metadata, never a permission or semantic witness.
#[derive(Debug, Clone, Serialize)]
pub struct IntegritySnapshotV1 {
    pub schema_version: &'static str,
    pub database_sha256: String,
    pub database_size_bytes: u64,
    pub sqlite_user_version: u32,
    /// True only when the parent-directory sync completed on a Unix platform.
    /// Non-Unix callers have image-file fsync only, not this durability claim.
    pub directory_synced: bool,
}

/// Snapshot failures distinguish refusal, unpublished failure, and a published
/// image whose directory durability could not be confirmed. Never blindly retry
/// a publication-uncertain result or remove an existing destination.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum IntegritySnapshotError {
    #[error("integrity snapshot requires an explicitly read-only store")]
    RequiresReadOnlyStore,
    #[error("snapshot byte limit must be positive")]
    InvalidLimit,
    #[error("snapshot destination needs a filename and an existing parent")]
    InvalidDestination,
    #[error("snapshot destination already exists (including symlinks)")]
    DestinationExists,
    #[error("snapshot destination must be outside the source directory")]
    SourceDestinationOverlap,
    #[error("source has no database pages or its byte-size calculation overflowed")]
    InvalidSourceSize,
    #[error("snapshot size {bytes} exceeds byte limit {max_bytes}")]
    LimitExceeded { bytes: u64, max_bytes: u64 },
    #[error("SQLite backup did not complete within its bounded steps")]
    BackupIncomplete,
    #[error("SQLite backup is busy or locked; no retry was performed")]
    BackupBusy,
    #[error("SQLite backup exceeded its cooperative deadline")]
    BackupDeadline,
    #[error("snapshot destination did not enter standalone delete-journal mode")]
    DestinationNotSealed,
    #[error("source owner connection failed: {0}")]
    Owner(#[from] MemoryError),
    #[error("SQLite snapshot operation failed: {0}")]
    Database(#[from] rusqlite::Error),
    #[error("snapshot I/O failed at {stage}: {source}")]
    Io {
        stage: &'static str,
        source: std::io::Error,
    },
    #[error("snapshot publication failed; destination state must be inspected: {0}")]
    PublicationUncertain(std::io::Error),
    #[error("image was published but parent-directory durability is unknown: {source}")]
    PublishedDurabilityUnknown {
        metadata: IntegritySnapshotV1,
        source: std::io::Error,
    },
    #[error("snapshot failed ({cause}); owned staging cleanup also failed: {source}")]
    Cleanup {
        cause: Box<IntegritySnapshotError>,
        staging_path: PathBuf,
        source: std::io::Error,
    },
}

fn io_error(stage: &'static str, source: std::io::Error) -> IntegritySnapshotError {
    IntegritySnapshotError::Io { stage, source }
}

impl MemoryStore {
    /// Copy a pinned SQLite read snapshot into an exclusively published file.
    ///
    /// This is a trusted-local administrative operation, not an authorization
    /// boundary. The caller owns/trusts the existing destination parent and must
    /// independently establish permission for handling the complete database.
    /// Source paths must remain stable; adversarial same-UID filesystem changes
    /// are outside this API's claim. Source journal mode, schema, rows, epochs,
    /// metadata and vector sidecars are not changed. No embedder is invoked.
    ///
    /// The byte cap is checked before and after copying. Backup steps have a
    /// cooperative 30-second deadline; a caller needing a hard bound must also
    /// supervise the operation externally (a blocked OS I/O call is not timed).
    /// A corrupt source is copied, not repaired or certified healthy. Only the
    /// SQLite image is included; derived vector sidecars are intentionally absent.
    pub async fn create_integrity_snapshot(
        &self,
        destination: impl AsRef<Path> + Send,
        max_bytes: u64,
    ) -> Result<IntegritySnapshotV1, IntegritySnapshotError> {
        if !self.inner.read_only {
            return Err(IntegritySnapshotError::RequiresReadOnlyStore);
        }
        if max_bytes == 0 {
            return Err(IntegritySnapshotError::InvalidLimit);
        }
        let source_base = self.inner.paths.base_dir.clone();
        let destination = destination.as_ref().to_path_buf();
        self.with_read_conn(move |conn| {
            Ok(snapshot_sync(
                conn,
                &source_base,
                &destination,
                max_bytes,
                |_, _| Ok(()),
            ))
        })
        .await?
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum SnapshotStage {
    BeforeBackup,
    AfterBackupStep,
    BeforePublication,
}

fn cleanup(staging: NamedTempFile, cause: IntegritySnapshotError) -> IntegritySnapshotError {
    let staging_path = staging.path().to_path_buf();
    match staging.close() {
        Ok(()) => cause,
        Err(source) => IntegritySnapshotError::Cleanup {
            cause: Box::new(cause),
            staging_path,
            source,
        },
    }
}

fn snapshot_sync(
    source: &Connection,
    source_base: &Path,
    destination: &Path,
    max_bytes: u64,
    mut hook: impl FnMut(SnapshotStage, &Path) -> Result<(), IntegritySnapshotError>,
) -> Result<IntegritySnapshotV1, IntegritySnapshotError> {
    let name = destination
        .file_name()
        .ok_or(IntegritySnapshotError::InvalidDestination)?;
    let parent = destination
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .ok_or(IntegritySnapshotError::InvalidDestination)?;
    let source_base = source_base
        .canonicalize()
        .map_err(|e| io_error("resolve_source", e))?;
    let parent = parent
        .canonicalize()
        .map_err(|e| io_error("resolve_destination_parent", e))?;
    if !parent.is_dir() {
        return Err(IntegritySnapshotError::InvalidDestination);
    }
    let destination = parent.join(name);
    if destination.starts_with(&source_base) {
        return Err(IntegritySnapshotError::SourceDestinationOverlap);
    }
    match std::fs::symlink_metadata(&destination) {
        Ok(_) => return Err(IntegritySnapshotError::DestinationExists),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
        Err(e) => return Err(io_error("inspect_destination", e)),
    }

    let transaction = source.unchecked_transaction()?;
    // A real read pins the WAL view before either budgeting or backup.
    let _: i64 = transaction.query_row("SELECT COUNT(*) FROM sqlite_schema", [], |r| r.get(0))?;
    let page_count: u64 = transaction.query_row("PRAGMA page_count", [], |r| r.get(0))?;
    let page_size: u64 = transaction.query_row("PRAGMA page_size", [], |r| r.get(0))?;
    let bytes = page_count
        .checked_mul(page_size)
        .filter(|v| *v > 0)
        .ok_or(IntegritySnapshotError::InvalidSourceSize)?;
    if bytes > max_bytes {
        return Err(IntegritySnapshotError::LimitExceeded { bytes, max_bytes });
    }
    let max_steps = page_count / 256 + 2;
    let mut staging = Builder::new()
        .prefix(".semantic-memory-snapshot-")
        .tempfile_in(&parent)
        .map_err(|e| io_error("create_staging", e))?;
    let result = (|| {
        hook(SnapshotStage::BeforeBackup, &destination)?;
        let mut target = Connection::open(staging.path())?;
        {
            let backup = Backup::new(&transaction, &mut target)?;
            let started = Instant::now();
            let mut complete = false;
            for _ in 0..max_steps {
                let state = backup.step(256)?;
                hook(SnapshotStage::AfterBackupStep, &destination)?;
                if started.elapsed() > Duration::from_secs(30) {
                    return Err(IntegritySnapshotError::BackupDeadline);
                }
                match state {
                    StepResult::Done => {
                        complete = true;
                        break;
                    }
                    StepResult::More => {}
                    StepResult::Busy | StepResult::Locked => {
                        return Err(IntegritySnapshotError::BackupBusy)
                    }
                    _ => return Err(IntegritySnapshotError::BackupIncomplete),
                }
            }
            if !complete {
                return Err(IntegritySnapshotError::BackupIncomplete);
            }
        }
        let mode: String = target.query_row("PRAGMA journal_mode=DELETE", [], |r| r.get(0))?;
        if mode != "delete" {
            return Err(IntegritySnapshotError::DestinationNotSealed);
        }
        let sqlite_user_version: u32 = target.query_row("PRAGMA user_version", [], |r| r.get(0))?;
        target
            .close()
            .map_err(|(_, e)| IntegritySnapshotError::Database(e))?;
        let database_size_bytes = staging
            .as_file()
            .metadata()
            .map_err(|e| io_error("measure_image", e))?
            .len();
        if database_size_bytes > max_bytes {
            return Err(IntegritySnapshotError::LimitExceeded {
                bytes: database_size_bytes,
                max_bytes,
            });
        }
        staging
            .as_file()
            .sync_all()
            .map_err(|e| io_error("sync_image", e))?;
        staging
            .as_file_mut()
            .seek(SeekFrom::Start(0))
            .map_err(|e| io_error("seek_image", e))?;
        let mut hash = Sha256::new();
        let mut buffer = [0_u8; 64 * 1024];
        loop {
            let n = staging
                .as_file_mut()
                .read(&mut buffer)
                .map_err(|e| io_error("hash_image", e))?;
            if n == 0 {
                break;
            }
            hash.update(&buffer[..n]);
        }
        hook(SnapshotStage::BeforePublication, &destination)?;
        Ok(IntegritySnapshotV1 {
            schema_version: "semantic_memory_integrity_snapshot_v1",
            database_sha256: format!("sha256:{:x}", hash.finalize()),
            database_size_bytes,
            sqlite_user_version,
            directory_synced: false,
        })
    })();
    let metadata = match result {
        Ok(metadata) => metadata,
        Err(error) => return Err(cleanup(staging, error)),
    };
    // End our source read transaction before publishing the completed image.
    if let Err(error) = transaction.commit() {
        return Err(cleanup(staging, error.into()));
    }
    let published = staging.persist_noclobber(&destination).map_err(|error| {
        let cause = if error.error.kind() == std::io::ErrorKind::AlreadyExists {
            IntegritySnapshotError::DestinationExists
        } else {
            IntegritySnapshotError::PublicationUncertain(error.error)
        };
        cleanup(error.file, cause)
    })?;
    drop(published);
    #[cfg(unix)]
    {
        if let Err(source) = File::open(&parent).and_then(|directory| directory.sync_all()) {
            return Err(IntegritySnapshotError::PublishedDurabilityUnknown { metadata, source });
        }
    }
    Ok(IntegritySnapshotV1 {
        directory_synced: cfg!(unix),
        ..metadata
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::OpenFlags;
    type TestResult = Result<(), Box<dyn std::error::Error>>;
    const CAP: u64 = 32 * 1024 * 1024;

    struct Scratch {
        _root: tempfile::TempDir,
        source: PathBuf,
        output: PathBuf,
        writer: Connection,
        reader: Connection,
    }
    fn scratch() -> Result<Scratch, Box<dyn std::error::Error>> {
        let root = tempfile::tempdir()?;
        let source = root.path().join("source");
        let output = root.path().join("output");
        std::fs::create_dir(&source)?;
        std::fs::create_dir(&output)?;
        let writer = Connection::open(source.join("memory.db"))?;
        // Raw SQLite fixture for pin/publication mechanics, not semantic-store
        // validity. The separate public integration test uses canonical appends.
        writer.execute_batch(
            "PRAGMA journal_mode=WAL; PRAGMA wal_autocheckpoint=0;
            CREATE TABLE left_state(value INTEGER); INSERT INTO left_state VALUES (0);
            CREATE TABLE right_state(value INTEGER); INSERT INTO right_state VALUES (0);
            CREATE TABLE padding(value BLOB); INSERT INTO padding VALUES (zeroblob(2097152));
            PRAGMA user_version=17;",
        )?;
        let reader = Connection::open_with_flags(
            source.join("memory.db"),
            OpenFlags::SQLITE_OPEN_READ_ONLY,
        )?;
        reader.execute_batch("PRAGMA query_only=ON;")?;
        Ok(Scratch {
            _root: root,
            source,
            output,
            writer,
            reader,
        })
    }

    #[test]
    fn source_view_stays_pinned_across_a_later_writer_commit() -> TestResult {
        let s = scratch()?;
        let destination = s.output.join("memory.db");
        let mut wrote = false;
        let report = snapshot_sync(&s.reader, &s.source, &destination, CAP, |stage, _| {
            if matches!(stage, SnapshotStage::BeforeBackup) {
                s.writer.execute_batch("BEGIN IMMEDIATE; UPDATE left_state SET value=1; UPDATE right_state SET value=1; COMMIT;")?;
                wrote = true;
            }
            Ok(())
        })?;
        assert!(wrote);
        assert_eq!(report.sqlite_user_version, 17);
        let copied = Connection::open_with_flags(destination, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
        let old: (i64, i64) = copied.query_row(
            "SELECT (SELECT value FROM left_state), (SELECT value FROM right_state)",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )?;
        let current: (i64, i64) = s.writer.query_row(
            "SELECT (SELECT value FROM left_state), (SELECT value FROM right_state)",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )?;
        assert_eq!(old, (0, 0), "backup left its budgeted read snapshot");
        assert_eq!(current, (1, 1), "concurrent writer did not really commit");
        Ok(())
    }

    #[test]
    fn injected_copy_and_prepublication_failures_remove_only_owned_staging() -> TestResult {
        for fail_at in [
            SnapshotStage::BeforeBackup,
            SnapshotStage::AfterBackupStep,
            SnapshotStage::BeforePublication,
        ] {
            let s = scratch()?;
            let destination = s.output.join("memory.db");
            let result = snapshot_sync(&s.reader, &s.source, &destination, CAP, |stage, _| {
                if stage == fail_at {
                    return Err(io_error(
                        "injected_failure",
                        std::io::Error::other("fixture"),
                    ));
                }
                Ok(())
            });
            assert!(matches!(
                result,
                Err(IntegritySnapshotError::Io {
                    stage: "injected_failure",
                    ..
                })
            ));
            assert!(!destination.exists());
            assert_eq!(std::fs::read_dir(&s.output)?.count(), 0);
            assert!(
                s.reader.is_autocommit(),
                "read transaction leaked into the pool"
            );
        }
        Ok(())
    }

    #[test]
    fn raced_destination_after_precheck_is_never_replaced() -> TestResult {
        let s = scratch()?;
        let destination = s.output.join("memory.db");
        let result = snapshot_sync(&s.reader, &s.source, &destination, CAP, |stage, path| {
            if matches!(stage, SnapshotStage::BeforePublication) {
                std::fs::write(path, b"other publisher").map_err(|e| io_error("fixture", e))?;
            }
            Ok(())
        });
        assert!(matches!(
            result,
            Err(IntegritySnapshotError::DestinationExists)
        ));
        assert_eq!(std::fs::read(destination)?, b"other publisher");
        assert_eq!(std::fs::read_dir(&s.output)?.count(), 1);
        Ok(())
    }

    #[cfg(unix)]
    #[test]
    fn cleanup_failure_is_explicit_and_retains_original_cause() -> TestResult {
        let s = scratch()?;
        let result = snapshot_sync(
            &s.reader,
            &s.source,
            &s.output.join("memory.db"),
            CAP,
            |stage, _| {
                if matches!(stage, SnapshotStage::BeforePublication) {
                    // Replace only this test's staging pathname with a directory.
                    // remove_file must fail even when the test runner is root;
                    // the owner must not recursively delete the replacement.
                    let paths = std::fs::read_dir(&s.output)
                        .map_err(|e| io_error("fixture", e))?
                        .map(|entry| entry.map(|entry| entry.path()))
                        .collect::<Result<Vec<_>, _>>()
                        .map_err(|e| io_error("fixture", e))?;
                    if paths.len() != 1 {
                        return Err(io_error(
                            "fixture",
                            std::io::Error::other("expected one staging path"),
                        ));
                    }
                    std::fs::remove_file(&paths[0]).map_err(|e| io_error("fixture", e))?;
                    std::fs::create_dir(&paths[0]).map_err(|e| io_error("fixture", e))?;
                    return Err(io_error(
                        "injected_failure",
                        std::io::Error::other("fixture"),
                    ));
                }
                Ok(())
            },
        );
        let path = match result {
            Err(IntegritySnapshotError::Cleanup {
                cause,
                staging_path,
                ..
            }) => {
                assert!(matches!(
                    *cause,
                    IntegritySnapshotError::Io {
                        stage: "injected_failure",
                        ..
                    }
                ));
                staging_path
            }
            other => return Err(format!("expected explicit cleanup failure: {other:?}").into()),
        };
        assert!(path.starts_with(&s.output));
        assert!(path.is_dir());
        assert!(!s.output.join("memory.db").exists());
        std::fs::remove_dir(path)?;
        Ok(())
    }
}
