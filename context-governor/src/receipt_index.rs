use crate::{CompactResponse, ContextGovernorError, FileContextStore};
use rusqlite::{params, Connection, OptionalExtension, TransactionBehavior};
use serde::Serialize;
use std::collections::{BTreeMap, HashSet};
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

pub(crate) const INDEX_FILE_NAME: &str = ".receipt-index.sqlite3";
pub(crate) const LEGACY_INDEX_FILE_NAME: &str = ".receipt-index.json";
const INDEX_SCHEMA: &str = "ReceiptLineageSignatureIndexV4";
const TRIGRAM_ALGORITHM: &str = "fnv1a64-unicode-lowercase-scalar-trigram-v1";
const FNV_OFFSET_BASIS: u64 = 0xcbf29ce484222325;
const FNV_PRIME: u64 = 0x100000001b3;
static INDEX_WRITE_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

fn index_write_guard() -> std::sync::MutexGuard<'static, ()> {
    INDEX_WRITE_LOCK
        .get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ReceiptFingerprint {
    pub receipt_id: String,
    pub path: PathBuf,
    pub file_bytes: u64,
    pub modified_ns: i64,
    pub changed_ns: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct IndexedLineageRow {
    pub(crate) receipt_id: String,
    pub(crate) receipt_schema: String,
    pub(crate) session_id: String,
    pub(crate) generation: Option<u32>,
    pub(crate) parent_receipt_id: Option<String>,
    pub(crate) receipt_identity_sha256: Option<String>,
    pub(crate) lineage_sha256: Option<String>,
    pub(crate) compacted_transcript_sha256: Option<String>,
}

#[derive(Debug)]
struct IndexedReceipt {
    receipt_id: String,
    receipt_schema: String,
    session_id: String,
    generation: Option<u32>,
    parent_receipt_id: Option<String>,
    receipt_identity_sha256: Option<String>,
    lineage_sha256: Option<String>,
    compacted_transcript_sha256: Option<String>,
    created_utc: String,
    file_bytes: u64,
    modified_ns: i64,
    changed_ns: i64,
    trigram_hashes: Vec<u8>,
    trigram_hashes_blake3: String,
}

#[derive(Debug, Serialize)]
struct LineageCatalogRowV1 {
    receipt_id: String,
    receipt_schema: String,
    session_id: String,
    generation: Option<u32>,
    parent_receipt_id: Option<String>,
    file_bytes: u64,
    modified_ns: i64,
    changed_ns: i64,
}

#[derive(Debug, Serialize)]
struct LineageCatalogPayloadV1 {
    schema: &'static str,
    rows: Vec<LineageCatalogRowV1>,
}

pub(crate) fn index_path(root: &Path) -> PathBuf {
    root.join(INDEX_FILE_NAME)
}

pub(crate) fn scan_fingerprints(
    root: &Path,
) -> Result<Vec<ReceiptFingerprint>, ContextGovernorError> {
    if !root.exists() {
        return Ok(Vec::new());
    }
    let mut out = Vec::new();
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|value| value.to_str()) != Some("json") {
            continue;
        }
        if path.file_name().and_then(|value| value.to_str()) == Some(LEGACY_INDEX_FILE_NAME) {
            continue;
        }
        let Some(receipt_id) = path.file_stem().and_then(|value| value.to_str()) else {
            continue;
        };
        if !receipt_id.starts_with("ctxr_") && !receipt_id.starts_with("rehydrated-") {
            continue;
        }
        let metadata = entry.metadata()?;
        out.push(ReceiptFingerprint {
            receipt_id: receipt_id.to_string(),
            path,
            file_bytes: metadata.len(),
            modified_ns: modified_ns(&metadata),
            changed_ns: changed_ns(&metadata),
        });
    }
    out.sort_by(|left, right| left.receipt_id.cmp(&right.receipt_id));
    Ok(out)
}

pub(crate) fn fingerprint_for_path(
    receipt_id: &str,
    path: &Path,
) -> Result<ReceiptFingerprint, ContextGovernorError> {
    let metadata = fs::metadata(path)?;
    Ok(ReceiptFingerprint {
        receipt_id: receipt_id.to_string(),
        path: path.to_path_buf(),
        file_bytes: metadata.len(),
        modified_ns: modified_ns(&metadata),
        changed_ns: changed_ns(&metadata),
    })
}

/// A cheap readiness check. It validates the application schema and compares
/// receipt metadata, but deliberately does not run PRAGMA quick_check: SQLite
/// documents quick_check as O(N), which makes it unsuitable for a query hot path.
pub(crate) fn index_is_valid(
    root: &Path,
    fingerprints: &[ReceiptFingerprint],
) -> Result<bool, ContextGovernorError> {
    let path = index_path(root);
    if !path.exists() {
        return Ok(false);
    }
    let connection = match open_read_only(&path) {
        Ok(connection) => connection,
        Err(_) => return Ok(false),
    };
    validate_connection(&connection, fingerprints)
}

/// Make the derived index match the authoritative receipt files. A healthy
/// index is reconciled incrementally; only a missing, corrupt, or incompatible
/// database is rebuilt from the full corpus.
pub(crate) fn ensure_index(
    root: &Path,
    fingerprints: &[ReceiptFingerprint],
) -> Result<(), ContextGovernorError> {
    let _write_guard = index_write_guard();
    if index_path(root).exists() {
        match reconcile_index(root, fingerprints) {
            Ok(true) => return Ok(()),
            Ok(false) | Err(_) => remove_index_files(root)?,
        }
    }
    rebuild_index(root, fingerprints, None)
}

pub(crate) fn ensure_lineage_index(
    root: &Path,
    fingerprints: &[ReceiptFingerprint],
    ring: &KeyRing,
) -> Result<(), ContextGovernorError> {
    if fingerprints.is_empty() {
        let valid = index_path(root).exists()
            && open_read_only(&index_path(root))
                .ok()
                .map(|connection| lineage_connection_is_valid(&connection, fingerprints, ring))
                .transpose()?
                .unwrap_or(false);
        if !valid {
            return rebuild_lineage_index(root, fingerprints, ring);
        }
        return Ok(());
    }

    let connection = open_read_only(&index_path(root)).map_err(|_| {
        ContextGovernorError::LineageIndexRebuildRequired {
            reason: "lineage catalog is missing or unreadable".to_string(),
        }
    })?;
    if !lineage_connection_is_valid(&connection, fingerprints, ring)? {
        return Err(ContextGovernorError::LineageIndexRebuildRequired {
            reason: "lineage catalog fingerprints or authenticated row set do not match authoritative receipts".to_string(),
        });
    }
    Ok(())
}

pub(crate) fn rebuild_lineage_index(
    root: &Path,
    fingerprints: &[ReceiptFingerprint],
    ring: &KeyRing,
) -> Result<(), ContextGovernorError> {
    let _write_guard = index_write_guard();
    rebuild_index(root, fingerprints, Some(ring))?;
    let mut connection = open_writable(&index_path(root))?;
    let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
    write_lineage_catalog_auth(&transaction, ring)?;
    transaction.commit()?;
    drop(connection);
    let verified = open_read_only(&index_path(root))?;
    if !lineage_connection_is_valid(&verified, fingerprints, ring)? {
        return Err(ContextGovernorError::LineageIndexRebuildRequired {
            reason:
                "lineage catalog failed post-rebuild authentication or fingerprint verification"
                    .to_string(),
        });
    }
    Ok(())
}

pub(crate) fn lineage_rows_for_session(
    root: &Path,
    fingerprints: &[ReceiptFingerprint],
    session_id: &str,
    ring: &KeyRing,
) -> Result<Vec<IndexedLineageRow>, ContextGovernorError> {
    ensure_lineage_index(root, fingerprints, ring)?;
    let connection = open_read_only(&index_path(root))?;
    if !lineage_connection_is_valid(&connection, fingerprints, ring)? {
        return Err(ContextGovernorError::LineageIndexRebuildRequired {
            reason: "receipt index changed during lineage lookup".to_string(),
        });
    }
    let mut statement = connection.prepare(
        "SELECT receipt_id, receipt_schema, session_id, generation,
                parent_receipt_id, receipt_identity_sha256, lineage_sha256,
                compacted_transcript_sha256
         FROM receipts WHERE session_id = ?1
         ORDER BY generation, receipt_id",
    )?;
    let rows = statement.query_map(params![session_id], |row| {
        let generation = row
            .get::<_, Option<i64>>(3)?
            .map(|value| u32::try_from(value).map_err(|_| rusqlite::Error::InvalidQuery))
            .transpose()?;
        Ok(IndexedLineageRow {
            receipt_id: row.get(0)?,
            receipt_schema: row.get(1)?,
            session_id: row.get(2)?,
            generation,
            parent_receipt_id: row.get(4)?,
            receipt_identity_sha256: row.get(5)?,
            lineage_sha256: row.get(6)?,
            compacted_transcript_sha256: row.get(7)?,
        })
    })?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

/// receipt is authoritative and must be published first; a crash between that
/// rename and this transaction is repaired by ensure_index on the next search.
pub(crate) fn upsert_if_present(
    root: &Path,
    fingerprint: &ReceiptFingerprint,
    response: &CompactResponse,
) -> Result<bool, ContextGovernorError> {
    let _write_guard = index_write_guard();
    let path = index_path(root);
    if !path.exists() {
        return Ok(false);
    }
    let mut connection = open_writable(&path)?;
    if !schema_is_current(&connection)? {
        return Ok(false);
    }
    let row = indexed_receipt(fingerprint, response, None)?;
    let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
    upsert_row(&transaction, &row)?;
    transaction.commit()?;
    Ok(true)
}

pub(crate) fn upsert_versioned_if_present(
    root: &Path,
    fingerprint: &ReceiptFingerprint,
    response: &crate::lineage::VersionedCompactResponse,
    ring: &KeyRing,
) -> Result<bool, ContextGovernorError> {
    let _write_guard = index_write_guard();
    let path = index_path(root);
    if !path.exists() {
        return Ok(false);
    }
    let mut connection = open_writable(&path)?;
    if !schema_is_current(&connection)? {
        return Ok(false);
    }
    let projection = response.as_v1_projection();
    let row = indexed_receipt(fingerprint, &projection, Some(response))?;
    let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
    upsert_row(&transaction, &row)?;
    write_lineage_catalog_auth(&transaction, ring)?;
    transaction.commit()?;
    Ok(true)
}

pub(crate) fn remove_if_present(
    root: &Path,
    receipt_ids: &[String],
    ring: Option<&KeyRing>,
) -> Result<bool, ContextGovernorError> {
    let _write_guard = index_write_guard();
    if receipt_ids.is_empty() || !index_path(root).exists() {
        return Ok(index_path(root).exists());
    }
    let mut connection = open_writable(&index_path(root))?;
    if !schema_is_current(&connection)? {
        return Ok(false);
    }
    let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
    {
        let mut statement = transaction.prepare("DELETE FROM receipts WHERE receipt_id = ?1")?;
        for receipt_id in receipt_ids {
            statement.execute(params![receipt_id])?;
        }
    }
    if let Some(ring) = ring {
        write_lineage_catalog_auth(&transaction, ring)?;
    } else {
        transaction.execute(
            "DELETE FROM metadata WHERE key IN ('lineage_catalog_hmac', 'lineage_catalog_key_id')",
            [],
        )?;
    }
    transaction.commit()?;
    Ok(true)
}

pub(crate) fn ordered_receipt_ids(root: &Path) -> Result<Vec<String>, ContextGovernorError> {
    let connection = open_read_only(&index_path(root))?;
    let mut statement =
        connection.prepare("SELECT receipt_id FROM receipts ORDER BY created_utc, receipt_id")?;
    let rows = statement.query_map([], |row| row.get(0))?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

/// Return a complete candidate superset for the existing case-insensitive
/// substring contract. Hash collisions can only add false positives. Queries
/// shorter than three Unicode scalar values fall back to the authoritative scan.
pub(crate) fn candidate_receipts(
    root: &Path,
    fingerprints: &[ReceiptFingerprint],
    query: &str,
) -> Result<Option<Vec<String>>, ContextGovernorError> {
    let connection = match open_read_only(&index_path(root)) {
        Ok(connection) => connection,
        Err(_) => return Ok(None),
    };
    if !validate_connection(&connection, fingerprints)? {
        return Ok(None);
    }
    if query.is_empty() {
        return Ok(Some(ordered_receipt_ids_from_connection(&connection)?));
    }
    let query_hashes = trigram_hashes(query);
    if query_hashes.is_empty() {
        return Ok(None);
    }

    let mut statement = connection.prepare(
        "SELECT receipt_id, trigram_hashes, trigram_hashes_blake3
         FROM receipts ORDER BY created_utc, receipt_id",
    )?;
    let rows = statement.query_map([], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, Vec<u8>>(1)?,
            row.get::<_, String>(2)?,
        ))
    })?;
    let mut candidates = Vec::new();
    let mut corrupt_signature = false;
    for row in rows {
        let (receipt_id, encoded, expected_digest) = row?;
        if encoded.len() % 8 != 0 || blake3::hash(&encoded).to_hex().as_str() != expected_digest {
            corrupt_signature = true;
            break;
        }
        if contains_all_hashes(&encoded, &query_hashes) {
            candidates.push(receipt_id);
        }
    }
    drop(statement);
    drop(connection);
    if corrupt_signature {
        remove_index_files(root)?;
        return Ok(None);
    }
    Ok(Some(candidates))
}

pub(crate) fn invalidate(root: &Path) -> Result<(), ContextGovernorError> {
    remove_index_files(root)
}

fn reconcile_index(
    root: &Path,
    fingerprints: &[ReceiptFingerprint],
) -> Result<bool, ContextGovernorError> {
    let path = index_path(root);
    let mut connection = open_writable(&path)?;
    if !schema_is_current(&connection)? {
        return Ok(false);
    }

    let stored = stored_fingerprints(&connection)?;
    let expected = fingerprints
        .iter()
        .map(|fingerprint| (fingerprint.receipt_id.clone(), fingerprint))
        .collect::<BTreeMap<_, _>>();
    let removed = stored
        .keys()
        .filter(|receipt_id| !expected.contains_key(*receipt_id))
        .cloned()
        .collect::<Vec<_>>();
    let changed = fingerprints
        .iter()
        .filter(|fingerprint| {
            stored.get(&fingerprint.receipt_id).map_or(true, |stored| {
                *stored
                    != (
                        fingerprint.file_bytes,
                        fingerprint.modified_ns,
                        fingerprint.changed_ns,
                    )
            })
        })
        .map(|fingerprint| load_indexed_receipt(fingerprint, None))
        .collect::<Result<Vec<_>, _>>()?;

    if removed.is_empty() && changed.is_empty() {
        return Ok(true);
    }

    let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
    {
        let mut delete = transaction.prepare("DELETE FROM receipts WHERE receipt_id = ?1")?;
        for receipt_id in removed {
            delete.execute(params![receipt_id])?;
        }
    }
    for row in &changed {
        upsert_row(&transaction, row)?;
    }
    transaction.commit()?;
    Ok(true)
}

fn rebuild_index(
    root: &Path,
    fingerprints: &[ReceiptFingerprint],
    ring: Option<&KeyRing>,
) -> Result<(), ContextGovernorError> {
    fs::create_dir_all(root)?;
    let temporary_path = root.join(format!(
        ".receipt-index.{}.sqlite3.tmp",
        uuid::Uuid::new_v4()
    ));
    let rows = load_indexed_receipts_parallel(fingerprints, ring)?;

    let build_result = (|| -> Result<(), ContextGovernorError> {
        let mut connection = Connection::open(&temporary_path)?;
        connection.busy_timeout(std::time::Duration::from_secs(30))?;
        connection.execute_batch(
            "PRAGMA journal_mode=DELETE;
             PRAGMA synchronous=FULL;
             CREATE TABLE metadata (
                 key TEXT PRIMARY KEY,
                 value TEXT NOT NULL
             ) WITHOUT ROWID;
             CREATE TABLE receipts (
                 receipt_id TEXT PRIMARY KEY,
                 receipt_schema TEXT NOT NULL,
                 session_id TEXT NOT NULL,
                 generation INTEGER,
                 parent_receipt_id TEXT,
                 receipt_identity_sha256 TEXT,
                 lineage_sha256 TEXT,
                 compacted_transcript_sha256 TEXT,
                 created_utc TEXT NOT NULL,
                 file_bytes INTEGER NOT NULL,
                 modified_ns INTEGER NOT NULL,
                 changed_ns INTEGER NOT NULL,
                 trigram_hashes BLOB NOT NULL,
                 trigram_hashes_blake3 TEXT NOT NULL
             ) WITHOUT ROWID;
             CREATE INDEX receipts_created ON receipts(created_utc, receipt_id);
             CREATE INDEX receipts_session_generation ON receipts(session_id, generation, receipt_id);",
        )?;
        let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
        transaction.execute(
            "INSERT INTO metadata(key, value) VALUES('schema', ?1)",
            params![INDEX_SCHEMA],
        )?;
        transaction.execute(
            "INSERT INTO metadata(key, value) VALUES('trigram_algorithm', ?1)",
            params![TRIGRAM_ALGORITHM],
        )?;
        for row in &rows {
            upsert_row(&transaction, row)?;
        }
        transaction.commit()?;
        drop(connection);
        File::open(&temporary_path)?.sync_all()?;
        fs::rename(&temporary_path, index_path(root))?;
        sync_directory(root)?;
        if !index_is_valid(root, fingerprints)? {
            return Err(ContextGovernorError::Sqlite(
                "receipt index failed post-publication verification".into(),
            ));
        }
        Ok(())
    })();

    if build_result.is_err() {
        let _ = fs::remove_file(&temporary_path);
    }
    build_result
}

fn load_indexed_receipt(
    fingerprint: &ReceiptFingerprint,
    ring: Option<&KeyRing>,
) -> Result<IndexedReceipt, ContextGovernorError> {
    let versioned = crate::lineage::read_versioned_path(&fingerprint.path)?;
    if let Some(ring) = ring {
        let root = fingerprint.path.parent().ok_or_else(|| {
            ContextGovernorError::LineageIndexRebuildRequired {
                reason: "receipt path has no store root".to_string(),
            }
        })?;
        let store = FileContextStore::with_key_ring(root, ring.clone());
        store.verify_versioned_for_use(&versioned, true, "lineage catalog rebuild")?;
    }
    let response = versioned.as_v1_projection();
    indexed_receipt(fingerprint, &response, Some(&versioned))
}

fn indexed_receipt(
    fingerprint: &ReceiptFingerprint,
    response: &CompactResponse,
    versioned: Option<&crate::lineage::VersionedCompactResponse>,
) -> Result<IndexedReceipt, ContextGovernorError> {
    if response.receipt.receipt_id != fingerprint.receipt_id {
        return Err(ContextGovernorError::ReceiptNotFound(format!(
            "receipt identity mismatch: path={} payload={}",
            fingerprint.receipt_id, response.receipt.receipt_id
        )));
    }
    let (
        receipt_schema,
        session_id,
        generation,
        parent_receipt_id,
        receipt_identity_sha256,
        lineage_sha256,
        compacted_transcript_sha256,
    ) = match versioned {
        Some(crate::lineage::VersionedCompactResponse::V1(response)) => (
            "ContextCompactionReceiptV1".to_string(),
            response.receipt.session_id.clone(),
            None,
            None,
            None,
            None,
            Some(response.receipt.compacted_transcript_sha256.clone()),
        ),
        Some(crate::lineage::VersionedCompactResponse::V2(response)) => (
            "ContextCompactionReceiptV2".to_string(),
            response.receipt.session_id.clone(),
            Some(response.receipt.generation),
            response
                .receipt
                .parent_receipt
                .as_ref()
                .map(|parent| parent.receipt_id.clone()),
            Some(response.receipt.receipt_identity_sha256.clone()),
            Some(response.receipt.lineage_sha256.clone()),
            Some(response.receipt.compacted_transcript_sha256.clone()),
        ),
        None => (
            "ContextCompactionReceiptV1".to_string(),
            response.receipt.session_id.clone(),
            None,
            None,
            None,
            None,
            Some(response.receipt.compacted_transcript_sha256.clone()),
        ),
    };
    let mut hashes = HashSet::new();
    for item in &response.exact_store {
        add_trigram_hashes(&item.content, &mut hashes);
    }
    for message in &response.compacted_messages {
        add_trigram_hashes(&message.content, &mut hashes);
    }
    add_trigram_hashes(&serde_json::to_string(&response.receipt)?, &mut hashes);
    let mut hashes = hashes.into_iter().collect::<Vec<_>>();
    hashes.sort_unstable();
    let encoded = encode_hashes(&hashes);
    let digest = blake3::hash(&encoded).to_hex().to_string();
    Ok(IndexedReceipt {
        receipt_id: fingerprint.receipt_id.clone(),
        receipt_schema,
        session_id,
        generation,
        parent_receipt_id,
        receipt_identity_sha256,
        lineage_sha256,
        compacted_transcript_sha256,
        created_utc: response.receipt.created_utc.to_rfc3339(),
        file_bytes: fingerprint.file_bytes,
        modified_ns: fingerprint.modified_ns,
        changed_ns: fingerprint.changed_ns,
        trigram_hashes: encoded,
        trigram_hashes_blake3: digest,
    })
}
fn load_indexed_receipts_parallel(
    fingerprints: &[ReceiptFingerprint],
    ring: Option<&KeyRing>,
) -> Result<Vec<IndexedReceipt>, ContextGovernorError> {
    if fingerprints.len() <= 1 {
        return fingerprints
            .iter()
            .map(|fingerprint| load_indexed_receipt(fingerprint, ring))
            .collect();
    }
    let workers = std::thread::available_parallelism()
        .map(usize::from)
        .unwrap_or(1)
        .min(4)
        .min(fingerprints.len());
    let chunk_size = fingerprints.len().div_ceil(workers);
    std::thread::scope(|scope| {
        let handles = fingerprints
            .chunks(chunk_size)
            .map(|chunk| {
                scope.spawn(move || {
                    chunk
                        .iter()
                        .map(|fingerprint| load_indexed_receipt(fingerprint, ring))
                        .collect::<Result<Vec<_>, _>>()
                })
            })
            .collect::<Vec<_>>();
        let mut rows = Vec::with_capacity(fingerprints.len());
        for handle in handles {
            let chunk = handle.join().map_err(|_| {
                ContextGovernorError::Io(std::io::Error::other(
                    "receipt-index build worker panicked",
                ))
            })??;
            rows.extend(chunk);
        }
        Ok(rows)
    })
}

fn upsert_row(
    transaction: &rusqlite::Transaction<'_>,
    row: &IndexedReceipt,
) -> Result<(), ContextGovernorError> {
    transaction.execute(
        "INSERT INTO receipts(
             receipt_id, receipt_schema, session_id, generation, parent_receipt_id,
             receipt_identity_sha256, lineage_sha256, compacted_transcript_sha256,
             created_utc, file_bytes, modified_ns, changed_ns,
             trigram_hashes, trigram_hashes_blake3
         ) VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)
         ON CONFLICT(receipt_id) DO UPDATE SET
             receipt_schema = excluded.receipt_schema,
             session_id = excluded.session_id,
             generation = excluded.generation,
             parent_receipt_id = excluded.parent_receipt_id,
             receipt_identity_sha256 = excluded.receipt_identity_sha256,
             lineage_sha256 = excluded.lineage_sha256,
             compacted_transcript_sha256 = excluded.compacted_transcript_sha256,
             created_utc = excluded.created_utc,
             file_bytes = excluded.file_bytes,
             modified_ns = excluded.modified_ns,
             changed_ns = excluded.changed_ns,
             trigram_hashes = excluded.trigram_hashes,
             trigram_hashes_blake3 = excluded.trigram_hashes_blake3",
        params![
            &row.receipt_id,
            &row.receipt_schema,
            &row.session_id,
            row.generation.map(i64::from),
            &row.parent_receipt_id,
            &row.receipt_identity_sha256,
            &row.lineage_sha256,
            &row.compacted_transcript_sha256,
            &row.created_utc,
            row.file_bytes as i64,
            row.modified_ns,
            row.changed_ns,
            &row.trigram_hashes,
            &row.trigram_hashes_blake3,
        ],
    )?;
    Ok(())
}

fn open_read_only(path: &Path) -> Result<Connection, ContextGovernorError> {
    let connection = Connection::open_with_flags(
        path,
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )?;
    connection.busy_timeout(std::time::Duration::from_secs(30))?;
    Ok(connection)
}

fn open_writable(path: &Path) -> Result<Connection, ContextGovernorError> {
    let connection = Connection::open(path)?;
    connection.busy_timeout(std::time::Duration::from_secs(30))?;
    // This index is derived from fsync'd JSON receipts. WAL+NORMAL preserves
    // database consistency while avoiding a sync on every incremental update;
    // a lost final index transaction is detected from receipt fingerprints.
    connection.execute_batch(
        "PRAGMA journal_mode=WAL;
         PRAGMA synchronous=NORMAL;",
    )?;
    Ok(connection)
}

fn schema_is_current(connection: &Connection) -> Result<bool, ContextGovernorError> {
    let schema = connection
        .query_row(
            "SELECT value FROM metadata WHERE key = 'schema'",
            [],
            |row| row.get::<_, String>(0),
        )
        .optional();
    let Ok(Some(schema)) = schema else {
        return Ok(false);
    };
    if schema != INDEX_SCHEMA {
        return Ok(false);
    }
    let algorithm = connection
        .query_row(
            "SELECT value FROM metadata WHERE key = 'trigram_algorithm'",
            [],
            |row| row.get::<_, String>(0),
        )
        .optional();
    Ok(matches!(algorithm, Ok(Some(value)) if value == TRIGRAM_ALGORITHM))
}

fn validate_connection(
    connection: &Connection,
    fingerprints: &[ReceiptFingerprint],
) -> Result<bool, ContextGovernorError> {
    if !schema_is_current(connection)? {
        return Ok(false);
    }
    let stored = match stored_fingerprints(connection) {
        Ok(stored) => stored,
        Err(_) => return Ok(false),
    };
    if stored.len() != fingerprints.len() {
        return Ok(false);
    }
    Ok(fingerprints.iter().all(|fingerprint| {
        stored.get(&fingerprint.receipt_id)
            == Some(&(
                fingerprint.file_bytes,
                fingerprint.modified_ns,
                fingerprint.changed_ns,
            ))
    }))
}

fn lineage_connection_is_valid(
    connection: &Connection,
    fingerprints: &[ReceiptFingerprint],
    ring: &KeyRing,
) -> Result<bool, ContextGovernorError> {
    Ok(validate_connection(connection, fingerprints)?
        && lineage_catalog_auth_valid(connection, ring)?)
}

fn lineage_catalog_payload(
    connection: &Connection,
) -> Result<serde_json::Value, ContextGovernorError> {
    let mut statement = connection.prepare(
        "SELECT receipt_id, receipt_schema, session_id, generation,
                parent_receipt_id, file_bytes, modified_ns, changed_ns
         FROM receipts ORDER BY receipt_id",
    )?;
    let rows = statement.query_map([], |row| {
        let generation = row
            .get::<_, Option<i64>>(3)?
            .map(|value| u32::try_from(value).map_err(|_| rusqlite::Error::InvalidQuery))
            .transpose()?;
        Ok(LineageCatalogRowV1 {
            receipt_id: row.get(0)?,
            receipt_schema: row.get(1)?,
            session_id: row.get(2)?,
            generation,
            parent_receipt_id: row.get(4)?,
            file_bytes: row.get(5)?,
            modified_ns: row.get(6)?,
            changed_ns: row.get(7)?,
        })
    })?;
    let payload = LineageCatalogPayloadV1 {
        schema: "ContextGovernorLineageCatalogV1",
        rows: rows.collect::<Result<Vec<_>, _>>()?,
    };
    Ok(serde_json::to_value(payload)?)
}

fn lineage_catalog_auth_valid(
    connection: &Connection,
    ring: &KeyRing,
) -> Result<bool, ContextGovernorError> {
    let key_id = connection
        .query_row(
            "SELECT value FROM metadata WHERE key = 'lineage_catalog_key_id'",
            [],
            |row| row.get::<_, String>(0),
        )
        .optional()?;
    let signature = connection
        .query_row(
            "SELECT value FROM metadata WHERE key = 'lineage_catalog_hmac'",
            [],
            |row| row.get::<_, String>(0),
        )
        .optional()?;
    let (Some(key_id), Some(signature)) = (key_id, signature) else {
        return Ok(false);
    };
    if key_id != ring.active_key_id()? {
        return Ok(false);
    }
    let mut payload = lineage_catalog_payload(connection)?;
    let Some(object) = payload.as_object_mut() else {
        return Ok(false);
    };
    object.insert("hmac".to_string(), serde_json::Value::String(signature));
    Ok(ring.verify_json(&payload, "hmac"))
}

fn write_lineage_catalog_auth(
    connection: &Connection,
    ring: &KeyRing,
) -> Result<(), ContextGovernorError> {
    let payload = lineage_catalog_payload(connection)?;
    let signature = ring.sign_json(&payload, "hmac")?;
    let key_id = ring.active_key_id()?;
    connection.execute(
        "INSERT INTO metadata(key, value) VALUES('lineage_catalog_key_id', ?1)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        params![key_id],
    )?;
    connection.execute(
        "INSERT INTO metadata(key, value) VALUES('lineage_catalog_hmac', ?1)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        params![signature],
    )?;
    Ok(())
}

fn stored_fingerprints(
    connection: &Connection,
) -> Result<BTreeMap<String, (u64, i64, i64)>, ContextGovernorError> {
    let mut statement = connection.prepare(
        "SELECT receipt_id, file_bytes, modified_ns, changed_ns
         FROM receipts ORDER BY receipt_id",
    )?;
    let rows = statement.query_map([], |row| {
        Ok((
            row.get::<_, String>(0)?,
            (
                row.get::<_, u64>(1)?,
                row.get::<_, i64>(2)?,
                row.get::<_, i64>(3)?,
            ),
        ))
    })?;
    Ok(rows.collect::<Result<BTreeMap<_, _>, _>>()?)
}

fn ordered_receipt_ids_from_connection(
    connection: &Connection,
) -> Result<Vec<String>, ContextGovernorError> {
    let mut statement =
        connection.prepare("SELECT receipt_id FROM receipts ORDER BY created_utc, receipt_id")?;
    let rows = statement.query_map([], |row| row.get(0))?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

fn trigram_hashes(text: &str) -> Vec<u64> {
    let mut hashes = HashSet::new();
    add_trigram_hashes(text, &mut hashes);
    let mut hashes = hashes.into_iter().collect::<Vec<_>>();
    hashes.sort_unstable();
    hashes
}

fn add_trigram_hashes(text: &str, hashes: &mut HashSet<u64>) {
    let mut lowered = text.chars().flat_map(char::to_lowercase);
    let Some(mut first) = lowered.next() else {
        return;
    };
    let Some(mut second) = lowered.next() else {
        return;
    };
    for third in lowered {
        hashes.insert(hash_trigram(first, second, third));
        first = second;
        second = third;
    }
}

fn hash_trigram(first: char, second: char, third: char) -> u64 {
    let mut hash = FNV_OFFSET_BASIS;
    let mut buffer = [0u8; 4];
    for character in [first, second, third] {
        for byte in character.encode_utf8(&mut buffer).as_bytes() {
            hash ^= u64::from(*byte);
            hash = hash.wrapping_mul(FNV_PRIME);
        }
    }
    hash
}

fn encode_hashes(hashes: &[u64]) -> Vec<u8> {
    let mut encoded = Vec::with_capacity(hashes.len().saturating_mul(8));
    for hash in hashes {
        encoded.extend_from_slice(&hash.to_le_bytes());
    }
    encoded
}

fn contains_all_hashes(encoded: &[u8], query_hashes: &[u64]) -> bool {
    query_hashes
        .iter()
        .all(|query_hash| contains_hash(encoded, *query_hash))
}

fn contains_hash(encoded: &[u8], query_hash: u64) -> bool {
    let mut left = 0usize;
    let mut right = encoded.len() / 8;
    while left < right {
        let middle = left + (right - left) / 2;
        let start = middle * 8;
        let mut bytes = [0u8; 8];
        bytes.copy_from_slice(&encoded[start..start + 8]);
        match u64::from_le_bytes(bytes).cmp(&query_hash) {
            std::cmp::Ordering::Less => left = middle + 1,
            std::cmp::Ordering::Greater => right = middle,
            std::cmp::Ordering::Equal => return true,
        }
    }
    false
}

fn remove_index_files(root: &Path) -> Result<(), ContextGovernorError> {
    for suffix in ["", "-journal", "-wal", "-shm"] {
        let path = PathBuf::from(format!("{}{}", index_path(root).display(), suffix));
        match fs::remove_file(path) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
        }
    }
    Ok(())
}

fn modified_ns(metadata: &fs::Metadata) -> i64 {
    metadata
        .modified()
        .ok()
        .and_then(|modified| modified.duration_since(std::time::UNIX_EPOCH).ok())
        .and_then(|duration| i64::try_from(duration.as_nanos()).ok())
        .unwrap_or_default()
}

#[cfg(unix)]
fn changed_ns(metadata: &fs::Metadata) -> i64 {
    use std::os::unix::fs::MetadataExt;
    metadata
        .ctime()
        .saturating_mul(1_000_000_000)
        .saturating_add(metadata.ctime_nsec())
}

#[cfg(not(unix))]
fn changed_ns(metadata: &fs::Metadata) -> i64 {
    modified_ns(metadata)
}

fn sync_directory(path: &Path) -> Result<(), ContextGovernorError> {
    File::open(path)?.sync_all()?;
    Ok(())
}

// ── HMAC receipt integrity ────────────────────────────────────────────

/// Compute an HMAC-SHA256 over caller-supplied bytes.
///
/// For JSON receipts, use [`canonical_json_payload`] first. Signing raw JSON
/// text couples verification to whitespace and object-key order.
pub fn sign_receipt_content(content: &str, key: &[u8]) -> String {
    use hmac::{Hmac, Mac};
    use sha2::Sha256;
    type HmacSha256 = Hmac<Sha256>;
    let mut mac = HmacSha256::new_from_slice(key).expect("HMAC accepts any key length");
    mac.update(content.as_bytes());
    hex::encode(mac.finalize().into_bytes())
}

/// Verify caller-supplied bytes against an HMAC digest in constant time.
pub fn verify_receipt_integrity(content: &str, key: &[u8], expected: &str) -> bool {
    use hmac::{Hmac, Mac};
    use sha2::Sha256;
    type HmacSha256 = Hmac<Sha256>;
    let Ok(expected_bytes) = hex::decode(expected) else {
        return false;
    };
    let Ok(mut mac) = HmacSha256::new_from_slice(key) else {
        return false;
    };
    mac.update(content.as_bytes());
    mac.verify_slice(&expected_bytes).is_ok()
}

/// Produce the deterministic JSON payload covered by an HMAC.
///
/// The signature field is removed before serialization, so adding the HMAC
/// cannot change the bytes being authenticated. This operates on parsed JSON,
/// not source text, so formatting differences do not affect verification.
pub fn canonical_json_payload(
    value: &serde_json::Value,
    signature_field: &str,
) -> Result<String, ContextGovernorError> {
    let mut unsigned = value.clone();
    let Some(object) = unsigned.as_object_mut() else {
        return Err(ContextGovernorError::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "signed receipt must be a JSON object",
        )));
    };
    object.remove(signature_field);
    Ok(serde_json::to_string(&unsigned)?)
}

/// Display-only fingerprint. It is never a receipt identity or binding.
pub fn key_fingerprint(key: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    hex::encode(&Sha256::digest(key)[..4])
}

fn full_key_digest(key: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    hex::encode(Sha256::digest(key))
}

/// Canonical key identity: full lowercase SHA-256 of exactly 32 bytes.
pub fn key_id(key: &[u8]) -> Result<String, ContextGovernorError> {
    if key.len() != 32 {
        return Err(ContextGovernorError::InvalidKeyLength {
            path: "in-memory key".to_string(),
            actual: key.len(),
        });
    }
    Ok(full_key_digest(key))
}

/// A key ring: one active key + zero or more retired keys (for legacy verification).
#[derive(Debug, Clone)]
pub struct KeyRing {
    pub active: Vec<u8>,
    pub retired: Vec<(String, Vec<u8>)>, // (full key ID, key)
    pub compromised: HashSet<String>,
}

impl KeyRing {
    pub fn new(active: Vec<u8>) -> Self {
        Self {
            active,
            retired: Vec::new(),
            compromised: HashSet::new(),
        }
    }

    pub fn active_key_id(&self) -> Result<String, ContextGovernorError> {
        key_id(&self.active)
    }

    /// Try to verify a signed HMAC against any key in the ring.
    /// Signs with the active key, verifies against all keys.
    pub fn sign_and_verify(&self, content: &str, full_hmac: &str) -> bool {
        // Full HMAC format: "fpr:signature" when stored in receipt
        if let Some((fpr, sig)) = full_hmac.split_once(':') {
            return self.verify_with_fingerprint(content, fpr, sig);
        }
        // Legacy: no fingerprint — try active key
        let active_full_id = full_key_digest(&self.active);
        let active_short_id = key_fingerprint(&self.active);
        if self.compromised.contains(&active_full_id) || self.compromised.contains(&active_short_id)
        {
            return false;
        }
        verify_receipt_integrity(content, &self.active, full_hmac)
    }

    /// Sign a JSON value after removing its signature field.
    pub fn sign_json(
        &self,
        value: &serde_json::Value,
        signature_field: &str,
    ) -> Result<String, ContextGovernorError> {
        let active_key_id = self.active_key_id()?;
        if self.compromised.contains(&active_key_id) {
            return Err(ContextGovernorError::CompromisedKey {
                key_id: active_key_id,
            });
        }
        let payload = canonical_json_payload(value, signature_field)?;
        Ok(format!(
            "{}:{}",
            active_key_id,
            sign_receipt_content(&payload, &self.active)
        ))
    }

    /// Verify a JSON value against a detached field within that JSON object.
    pub fn verify_json(&self, value: &serde_json::Value, signature_field: &str) -> bool {
        let Some(signature) = value
            .get(signature_field)
            .and_then(serde_json::Value::as_str)
        else {
            return false;
        };
        canonical_json_payload(value, signature_field)
            .is_ok_and(|payload| self.sign_and_verify(&payload, signature))
    }

    fn verify_with_fingerprint(&self, content: &str, fpr: &str, sig: &str) -> bool {
        let active_full_id = full_key_digest(&self.active);
        let active_short_id = key_fingerprint(&self.active);
        if fpr == active_full_id || fpr == active_short_id {
            if self.compromised.contains(fpr)
                || self.compromised.contains(&active_full_id)
                || self.compromised.contains(&active_short_id)
            {
                return false;
            }
            return verify_receipt_integrity(content, &self.active, sig);
        }
        for (declared_id, retired_key) in &self.retired {
            let full_id = full_key_digest(retired_key);
            let short_id = key_fingerprint(retired_key);
            if fpr == declared_id || fpr == full_id || fpr == short_id {
                if self.compromised.contains(fpr)
                    || self.compromised.contains(declared_id)
                    || self.compromised.contains(&full_id)
                    || self.compromised.contains(&short_id)
                {
                    return false;
                }
                return verify_receipt_integrity(content, retired_key, sig);
            }
        }
        false
    }
}

// ── Key lifecycle ───────────────────────────────────────────────────────

/// Generate a fresh 32-byte HMAC-SHA256 key using OS CSPRNG.
pub fn generate_hmac_key() -> Vec<u8> {
    use rand::RngCore;
    let mut key = vec![0u8; 32];
    rand::thread_rng().fill_bytes(&mut key);
    key
}

/// Save an HMAC key to disk with restrictive permissions (0600 on Unix).
pub fn save_hmac_key(path: &Path, key: &[u8]) -> Result<(), ContextGovernorError> {
    use std::io::Write;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut f = File::create(path)?;
    f.write_all(key)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        f.set_permissions(fs::Permissions::from_mode(0o600))?;
    }
    Ok(())
}

/// Load an HMAC key from disk.
pub fn load_hmac_key(path: &Path) -> Result<Vec<u8>, ContextGovernorError> {
    let key = fs::read(path).map_err(|_| ContextGovernorError::KeyUnreadable {
        path: path.display().to_string(),
    })?;
    key_id(&key)?;
    Ok(key)
}

/// Load historical V1 verification material. Legacy releases accepted HMAC
/// keys of arbitrary non-zero length and stored only an 8-hex fingerprint.
/// This loader is never used by V2 governed authority or signing paths.
pub fn load_legacy_hmac_key(path: &Path) -> Result<Vec<u8>, ContextGovernorError> {
    let key = fs::read(path).map_err(|_| ContextGovernorError::KeyUnreadable {
        path: path.display().to_string(),
    })?;
    if key.is_empty() {
        return Err(ContextGovernorError::InvalidKeyLength {
            path: path.display().to_string(),
            actual: 0,
        });
    }
    Ok(key)
}

/// The one-generation retired-key path associated with an active key path.
pub fn retired_hmac_key_path(key_path: &Path) -> std::path::PathBuf {
    std::path::PathBuf::from(format!("{}.bak", key_path.display()))
}

/// Load the active key plus the previous generation retained by rotation.
pub fn load_hmac_key_ring(key_path: &Path) -> Result<KeyRing, ContextGovernorError> {
    let active = load_legacy_hmac_key(key_path)?;
    let mut ring = KeyRing::new(active);
    let retired_path = retired_hmac_key_path(key_path);
    if retired_path.exists() {
        let retired = load_legacy_hmac_key(&retired_path)?;
        ring.retired.push((full_key_digest(&retired), retired));
    }
    Ok(ring)
}

/// Load Ares-managed active and retained keys. Lifecycle metadata remains an
/// Ares concern; this function only turns an already-governed keyring into
/// cryptographic verification material and checks each declared identity.
pub fn load_governed_key_ring(
    active_path: &Path,
    keyring_path: &Path,
) -> Result<KeyRing, ContextGovernorError> {
    let active = load_hmac_key(active_path)?;
    let active_id = key_id(&active)?;
    let raw =
        fs::read_to_string(keyring_path).map_err(|_| ContextGovernorError::KeyUnreadable {
            path: keyring_path.display().to_string(),
        })?;
    let value: serde_json::Value =
        serde_json::from_str(&raw).map_err(|_| ContextGovernorError::InvalidKeyEncoding {
            path: keyring_path.display().to_string(),
        })?;
    let configured = value
        .get("active_key_id")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| ContextGovernorError::ConflictingActiveKeyState {
            reason: "keyring has no active_key_id".to_string(),
        })?;
    if configured != active_id {
        return Err(ContextGovernorError::WrongConfiguredKeyId {
            expected: configured.to_string(),
            actual: active_id,
        });
    }
    let compromised: HashSet<String> = value
        .get("compromised_key_ids")
        .and_then(serde_json::Value::as_array)
        .map(|values| {
            values
                .iter()
                .filter_map(serde_json::Value::as_str)
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default();
    if compromised.contains(configured) {
        return Err(ContextGovernorError::CompromisedKey {
            key_id: configured.to_string(),
        });
    }
    let mut ring = KeyRing::new(active);
    ring.compromised = compromised;
    let retired_ids = value
        .get("retired_key_ids")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| ContextGovernorError::ConflictingActiveKeyState {
            reason: "keyring has no retired_key_ids".to_string(),
        })?;
    let root = keyring_path
        .parent()
        .ok_or_else(|| ContextGovernorError::KeyPathEscape {
            path: keyring_path.display().to_string(),
        })?;
    for value in retired_ids {
        let Some(retired_id) = value.as_str() else {
            return Err(ContextGovernorError::InvalidKeyEncoding {
                path: keyring_path.display().to_string(),
            });
        };
        let path = root.join("retired").join(format!("{retired_id}.key"));
        let key = load_hmac_key(&path).map_err(|_| {
            ContextGovernorError::RequiredHistoricalKeyUnavailable {
                key_id: retired_id.to_string(),
            }
        })?;
        let actual = key_id(&key)?;
        if actual != retired_id {
            return Err(ContextGovernorError::ComputedKeyIdMismatch {
                expected: retired_id.to_string(),
                actual,
            });
        }
        ring.retired.push((retired_id.to_string(), key));
    }
    Ok(ring)
}

/// Rotate the HMAC key: generate new, save to path, return old key for re-signing.
pub fn rotate_hmac_key(key_path: &Path) -> Result<(Vec<u8>, Vec<u8>), ContextGovernorError> {
    let old = load_hmac_key(key_path)?;
    let new = generate_hmac_key();
    let backup = retired_hmac_key_path(key_path);
    if backup.exists() {
        fs::remove_file(&backup)?;
    }
    fs::rename(key_path, &backup)?;
    if let Err(e) = save_hmac_key(key_path, &new) {
        // Restore backup
        let _ = fs::rename(&backup, key_path);
        return Err(e);
    }
    Ok((old, new))
}

/// Verify all signed receipts in a directory using a key ring.
///
/// This is deliberately read-only. Missing signatures, malformed JSON, unknown
/// key fingerprints, and invalid signatures are failures; verification never
/// mints a key or mutates a receipt.
/// Returns (total, passed, failed_details).
pub fn verify_all_receipts(
    dir: &Path,
    ring: &KeyRing,
    receipt_ids: Option<&[String]>,
) -> (usize, usize, Vec<String>) {
    let mut total = 0usize;
    let mut passed = 0usize;
    let mut failures = Vec::new();

    let ids = receipt_ids.map(|s| s.iter().collect::<HashSet<_>>());
    for entry in fs::read_dir(dir).into_iter().flatten().flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        let Some(name) = path.file_stem().and_then(|s| s.to_str()) else {
            continue;
        };
        if !name.starts_with("ctxr_") && !name.starts_with("rehydrated-") {
            continue;
        }
        if let Some(ref ids_set) = ids {
            if !ids_set.contains(&name.to_string()) {
                continue;
            }
        }
        total += 1;
        match fs::read_to_string(&path) {
            Ok(content) => match serde_json::from_str::<serde_json::Value>(&content) {
                Ok(receipt) if receipt.get("hmac").and_then(|v| v.as_str()).is_none() => {
                    failures.push(format!("{name}: missing hmac"));
                }
                Ok(receipt) if ring.verify_json(&receipt, "hmac") => passed += 1,
                Ok(_) => failures.push(format!("{name}: HMAC verification failed")),
                Err(_) => failures.push(format!("{name}: JSON parse error")),
            },
            Err(e) => failures.push(format!("{name}: read error {}", e)),
        }
    }
    (total, passed, failures)
}
