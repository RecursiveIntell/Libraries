//! Authenticated, rebuildable lineage-tip projection.
//!
//! JSON receipts remain the authority. This catalog contains only the metadata
//! needed to select one session's candidate tip without parsing every receipt
//! payload. Every lookup validates the catalog against receipt-file metadata
//! and its governed HMAC before using it; a stale or unauthenticated catalog is
//! a typed maintenance state, never an empty lineage.

use crate::{receipt_index, ContextGovernorError};
use rusqlite::{params, Connection, OptionalExtension, TransactionBehavior};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs::{self, File};
use std::path::{Path, PathBuf};

pub(crate) const INDEX_FILE_NAME: &str = ".lineage-index.sqlite3";
const INDEX_SCHEMA: &str = "ContextGovernorLineageIndexV1";
const CANONICALIZATION: &str = "sorted-json-rows-v1";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct LineageCatalogRow {
    pub(crate) receipt_id: String,
    pub(crate) receipt_schema: String,
    pub(crate) session_id: String,
    pub(crate) generation: Option<u32>,
    pub(crate) parent_receipt_id: Option<String>,
    pub(crate) file_bytes: u64,
    pub(crate) modified_ns: i64,
    pub(crate) changed_ns: i64,
}

pub(crate) fn index_path(root: &Path) -> PathBuf {
    root.join(INDEX_FILE_NAME)
}

fn rebuild_required(reason: impl Into<String>) -> ContextGovernorError {
    ContextGovernorError::LineageIndexRebuildRequired {
        reason: reason.into(),
    }
}

fn canonical_rows(rows: &[LineageCatalogRow]) -> Result<String, ContextGovernorError> {
    Ok(serde_json::to_string(rows)?)
}

fn catalog_signature(
    rows: &[LineageCatalogRow],
    ring: &receipt_index::KeyRing,
) -> Result<String, ContextGovernorError> {
    let key_id = ring.active_key_id()?;
    let payload = canonical_rows(rows)?;
    Ok(format!(
        "{key_id}:{}",
        receipt_index::sign_receipt_content(&payload, &ring.active)
    ))
}

fn sort_rows(rows: &mut [LineageCatalogRow]) {
    rows.sort_by(|left, right| left.receipt_id.cmp(&right.receipt_id));
}

fn open_read_only(path: &Path) -> Result<Connection, ContextGovernorError> {
    let connection = Connection::open_with_flags(
        path,
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )?;
    connection.busy_timeout(std::time::Duration::from_secs(30))?;
    Ok(connection)
}

fn metadata(connection: &Connection, key: &str) -> Result<Option<String>, ContextGovernorError> {
    Ok(connection
        .query_row(
            "SELECT value FROM metadata WHERE key = ?1",
            params![key],
            |row| row.get::<_, String>(0),
        )
        .optional()?)
}

fn read_rows(connection: &Connection) -> Result<Vec<LineageCatalogRow>, ContextGovernorError> {
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
        let file_bytes = row.get::<_, i64>(5)?;
        if file_bytes < 0 {
            return Err(rusqlite::Error::InvalidQuery);
        }
        Ok(LineageCatalogRow {
            receipt_id: row.get(0)?,
            receipt_schema: row.get(1)?,
            session_id: row.get(2)?,
            generation,
            parent_receipt_id: row.get(4)?,
            file_bytes: file_bytes as u64,
            modified_ns: row.get(6)?,
            changed_ns: row.get(7)?,
        })
    })?;
    let mut output = rows.collect::<Result<Vec<_>, _>>()?;
    sort_rows(&mut output);
    Ok(output)
}

fn read_catalog(
    root: &Path,
    ring: &receipt_index::KeyRing,
) -> Result<Vec<LineageCatalogRow>, ContextGovernorError> {
    let path = index_path(root);
    if !path.exists() {
        return Err(rebuild_required("catalog file is missing"));
    }
    let connection = open_read_only(&path)
        .map_err(|error| rebuild_required(format!("catalog cannot be opened: {error}")))?;
    let schema = metadata(&connection, "schema")?
        .ok_or_else(|| rebuild_required("catalog schema metadata is missing"))?;
    let canonicalization = metadata(&connection, "canonicalization")?
        .ok_or_else(|| rebuild_required("catalog canonicalization metadata is missing"))?;
    let signature = metadata(&connection, "catalog_hmac")?
        .ok_or_else(|| rebuild_required("catalog HMAC metadata is missing"))?;
    if schema != INDEX_SCHEMA || canonicalization != CANONICALIZATION {
        return Err(rebuild_required(
            "catalog schema or canonicalization is stale",
        ));
    }
    let rows = read_rows(&connection)
        .map_err(|error| rebuild_required(format!("catalog rows are unreadable: {error}")))?;
    let payload = canonical_rows(&rows)?;
    if !ring.sign_and_verify(&payload, &signature) {
        return Err(rebuild_required("catalog HMAC verification failed"));
    }
    let declared_key_id = metadata(&connection, "signing_key_id")?
        .ok_or_else(|| rebuild_required("catalog signing-key metadata is missing"))?;
    let signature_key_id = signature
        .split_once(':')
        .map(|(key_id, _)| key_id)
        .unwrap_or_default();
    if declared_key_id != signature_key_id {
        return Err(rebuild_required(
            "catalog signer metadata disagrees with HMAC",
        ));
    }
    Ok(rows)
}

fn compare_fingerprints(
    root: &Path,
    rows: &[LineageCatalogRow],
    fingerprints: &[receipt_index::ReceiptFingerprint],
) -> Result<(), ContextGovernorError> {
    let actual = fingerprints
        .iter()
        .map(|fingerprint| {
            (
                fingerprint.receipt_id.clone(),
                (
                    fingerprint.file_bytes,
                    fingerprint.modified_ns,
                    fingerprint.changed_ns,
                ),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let indexed = rows
        .iter()
        .map(|row| {
            (
                row.receipt_id.clone(),
                (row.file_bytes, row.modified_ns, row.changed_ns),
            )
        })
        .collect::<BTreeMap<_, _>>();
    if actual != indexed {
        return Err(rebuild_required(format!(
            "catalog fingerprint coverage differs from authoritative receipts at {}",
            root.display()
        )));
    }
    Ok(())
}

/// Validate the projection using metadata only. Receipt payloads are not read.
pub(crate) fn validate(
    root: &Path,
    ring: &receipt_index::KeyRing,
) -> Result<Vec<LineageCatalogRow>, ContextGovernorError> {
    let rows = read_catalog(root, ring)?;
    let fingerprints = receipt_index::scan_fingerprints(root)?;
    compare_fingerprints(root, &rows, &fingerprints)?;
    Ok(rows)
}

fn write_catalog(
    root: &Path,
    rows: &[LineageCatalogRow],
    ring: &receipt_index::KeyRing,
) -> Result<(), ContextGovernorError> {
    fs::create_dir_all(root)?;
    let mut rows = rows.to_vec();
    sort_rows(&mut rows);
    let signature = catalog_signature(&rows, ring)?;
    let temporary_path = root.join(format!(
        ".lineage-index.{}.sqlite3.tmp",
        uuid::Uuid::new_v4()
    ));
    let result = (|| -> Result<(), ContextGovernorError> {
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
                 file_bytes INTEGER NOT NULL,
                 modified_ns INTEGER NOT NULL,
                 changed_ns INTEGER NOT NULL
             ) WITHOUT ROWID;
             CREATE INDEX receipts_session_generation
                 ON receipts(session_id, generation, receipt_id);",
        )?;
        let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
        for (key, value) in [
            ("schema", INDEX_SCHEMA.to_string()),
            ("canonicalization", CANONICALIZATION.to_string()),
            ("signing_key_id", ring.active_key_id()?),
            ("catalog_hmac", signature),
        ] {
            transaction.execute(
                "INSERT INTO metadata(key, value) VALUES(?1, ?2)",
                params![key, value],
            )?;
        }
        for row in &rows {
            transaction.execute(
                "INSERT INTO receipts(
                    receipt_id, receipt_schema, session_id, generation,
                    parent_receipt_id, file_bytes, modified_ns, changed_ns
                 ) VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![
                    row.receipt_id,
                    row.receipt_schema,
                    row.session_id,
                    row.generation.map(i64::from),
                    row.parent_receipt_id,
                    i64::try_from(row.file_bytes).map_err(|_| {
                        ContextGovernorError::Io(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            "receipt file size exceeds SQLite integer range",
                        ))
                    })?,
                    row.modified_ns,
                    row.changed_ns,
                ],
            )?;
        }
        transaction.commit()?;
        drop(connection);
        File::open(&temporary_path)?.sync_all()?;
        fs::rename(&temporary_path, index_path(root))?;
        File::open(root)?.sync_all()?;
        Ok(())
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temporary_path);
    }
    result
}

pub(crate) fn initialize_empty(
    root: &Path,
    ring: &receipt_index::KeyRing,
) -> Result<(), ContextGovernorError> {
    if index_path(root).exists() {
        return Ok(());
    }
    write_catalog(root, &[], ring)
}

pub(crate) fn rebuild(
    root: &Path,
    rows: &[LineageCatalogRow],
    ring: &receipt_index::KeyRing,
) -> Result<(), ContextGovernorError> {
    write_catalog(root, rows, ring)?;
    validate(root, ring).map(|_| ())
}

pub(crate) fn append(
    root: &Path,
    row: &LineageCatalogRow,
    ring: &receipt_index::KeyRing,
) -> Result<(), ContextGovernorError> {
    let mut rows = read_catalog(root, ring)?;
    if rows
        .iter()
        .any(|existing| existing.receipt_id == row.receipt_id)
    {
        return Err(rebuild_required(format!(
            "catalog already contains receipt {}",
            row.receipt_id
        )));
    }
    let fingerprints = receipt_index::scan_fingerprints(root)?;
    let Some(new_fingerprint) = fingerprints
        .iter()
        .find(|fingerprint| fingerprint.receipt_id == row.receipt_id)
    else {
        return Err(rebuild_required(format!(
            "new receipt {} is absent from authoritative store",
            row.receipt_id
        )));
    };
    if (row.file_bytes, row.modified_ns, row.changed_ns)
        != (
            new_fingerprint.file_bytes,
            new_fingerprint.modified_ns,
            new_fingerprint.changed_ns,
        )
    {
        return Err(rebuild_required(format!(
            "new receipt {} fingerprint changed before catalog append",
            row.receipt_id
        )));
    }
    let existing_fingerprints = fingerprints
        .into_iter()
        .filter(|fingerprint| fingerprint.receipt_id != row.receipt_id)
        .collect::<Vec<_>>();
    compare_fingerprints(root, &rows, &existing_fingerprints)?;
    rows.push(row.clone());
    write_catalog(root, &rows, ring)
}

pub(crate) fn remove(
    root: &Path,
    removed_ids: &[String],
    ring: &receipt_index::KeyRing,
) -> Result<(), ContextGovernorError> {
    if removed_ids.is_empty() {
        return Ok(());
    }
    let rows = read_catalog(root, ring)?;
    let removed = removed_ids
        .iter()
        .collect::<std::collections::BTreeSet<_>>();
    let remaining = rows
        .into_iter()
        .filter(|row| !removed.contains(&row.receipt_id))
        .collect::<Vec<_>>();
    let fingerprints = receipt_index::scan_fingerprints(root)?;
    compare_fingerprints(root, &remaining, &fingerprints)?;
    write_catalog(root, &remaining, ring)
}
