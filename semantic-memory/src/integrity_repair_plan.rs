//! Unsigned, read-only inspection of sealed schema-39 authority orphans.
//! This module never issues application permission and has no mutation path.
use crate::{MemoryError, MemoryStore};
use rusqlite::{types::ValueRef, Connection, OptionalExtension};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::{collections::BTreeSet, fs::File, io::Read, path::Path};

const MAX_PLAN_ROWS: usize = 10_000;
const MAX_PLAN_BYTES: usize = 16 * 1024 * 1024;
const TABLES: [&str; 5] = [
    "authority_lineages",
    "authority_versions",
    "origin_authority_labels",
    "origin_authority_revocations",
    "forgotten_facts",
];
const EPOCHS: [&str; 5] = [
    "retrieval_epoch",
    "projection_epoch",
    "cache_epoch",
    "export_epoch",
    "replay_epoch",
];

#[derive(Debug, thiserror::Error)]
pub enum AuthorityRelationQuarantinePlanError {
    #[error("planner requires explicitly read-only store")]
    RequiresReadOnlyStore,
    #[error("planner requires a standalone DELETE-journal image")]
    NotSealed,
    #[error("planner requires exact schema 39")]
    UnsupportedSchema,
    #[error("planner row or byte budget exceeded")]
    LimitExceeded,
    #[error("unsupported foreign key or orphan shape: {0}")]
    UnsupportedShape(String),
    #[error("snapshot bytes changed during inspection")]
    SnapshotChanged,
    #[error("owner connection: {0}")]
    Owner(#[from] MemoryError),
    #[error("sqlite: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("source I/O: {0}")]
    Io(#[from] std::io::Error),
    #[error("serialization: {0}")]
    Json(#[from] serde_json::Error),
}
use AuthorityRelationQuarantinePlanError as Error;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "type", content = "value")]
pub enum SqliteCellV1 {
    Null,
    Integer(i64),
    RealBits(String),
    TextHex(String),
    BlobHex(String),
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct OrphanViolationV1 {
    pub table: String,
    pub rowid: i64,
    pub parent: String,
    pub foreign_key_id: i64,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct OrphanRowV1 {
    pub table: String,
    pub rowid: i64,
    pub columns: Vec<String>,
    pub cells: Vec<SqliteCellV1>,
    pub row_sha256: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AuthorityRelationQuarantinePlanV1 {
    pub schema_version: &'static str,
    pub disposition: &'static str,
    pub application_authorization_required: bool,
    pub database_sha256: String,
    pub sqlite_user_version: u32,
    pub schema_manifest_sha256: String,
    pub epochs: Vec<(String, i64)>,
    pub violations: Vec<OrphanViolationV1>,
    pub violations_sha256: String,
    pub rows: Vec<OrphanRowV1>,
    pub rows_sha256: String,
    pub affected_fact_ids: Vec<String>,
    pub affected_lineage_ids: Vec<String>,
    pub plan_sha256: String,
}
fn digest<T: Serialize>(value: &T) -> Result<String, Error> {
    Ok(format!(
        "sha256:{:x}",
        Sha256::digest(serde_json::to_vec(value)?)
    ))
}
fn file_digest(path: &Path) -> Result<String, Error> {
    let mut file = File::open(path)?;
    let mut hash = Sha256::new();
    let mut buf = [0_u8; 65536];
    loop {
        let n = file.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hash.update(&buf[..n]);
    }
    Ok(format!("sha256:{:x}", hash.finalize()))
}
fn hex_bytes(bytes: &[u8]) -> String {
    use std::fmt::Write;
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        let _ = write!(out, "{byte:02x}");
    }
    out
}
fn unhex(value: &str) -> Result<Vec<u8>, Error> {
    if value.len() % 2 != 0 {
        return Err(Error::UnsupportedShape("bad identity encoding".into()));
    }
    value
        .as_bytes()
        .chunks_exact(2)
        .map(|pair| {
            let part = std::str::from_utf8(pair)
                .map_err(|_| Error::UnsupportedShape("bad identity encoding".into()))?;
            u8::from_str_radix(part, 16)
                .map_err(|_| Error::UnsupportedShape("bad identity encoding".into()))
        })
        .collect()
}
fn cell(value: ValueRef<'_>) -> SqliteCellV1 {
    match value {
        ValueRef::Null => SqliteCellV1::Null,
        ValueRef::Integer(v) => SqliteCellV1::Integer(v),
        ValueRef::Real(v) => SqliteCellV1::RealBits(format!("{:016x}", v.to_bits())),
        ValueRef::Text(v) => SqliteCellV1::TextHex(hex_bytes(v)),
        ValueRef::Blob(v) => SqliteCellV1::BlobHex(hex_bytes(v)),
    }
}
fn text(cell: &SqliteCellV1) -> Result<String, Error> {
    let SqliteCellV1::TextHex(hex) = cell else {
        return Err(Error::UnsupportedShape("identity is not TEXT".into()));
    };
    let bytes = unhex(hex)?;
    let value = String::from_utf8(bytes)
        .map_err(|_| Error::UnsupportedShape("invalid identity UTF-8".into()))?;
    if value.is_empty() || value.trim() != value || value.contains('\0') {
        return Err(Error::UnsupportedShape("malformed identity".into()));
    }
    Ok(value)
}
fn column<'a>(row: &'a OrphanRowV1, name: &str) -> Result<&'a SqliteCellV1, Error> {
    let pos = row
        .columns
        .iter()
        .position(|v| v == name)
        .ok_or_else(|| Error::UnsupportedShape(format!("missing column {name}")))?;
    row.cells
        .get(pos)
        .ok_or_else(|| Error::UnsupportedShape(format!("missing cell {name}")))
}
fn capture(
    conn: &Connection,
    table: &str,
    rowid: i64,
    max_bytes: usize,
) -> Result<OrphanRowV1, Error> {
    // table is from the closed static allowlist, never caller-controlled.
    let mut stmt = conn.prepare(&format!("SELECT rowid, * FROM \"{table}\" WHERE rowid=?1"))?;
    let columns: Vec<_> = stmt
        .column_names()
        .iter()
        .skip(1)
        .map(|s| (*s).to_owned())
        .collect();
    let mut rows = stmt.query([rowid])?;
    let raw = rows
        .next()?
        .ok_or_else(|| Error::UnsupportedShape(format!("missing rowid in {table}")))?;
    let mut cells = Vec::with_capacity(columns.len());
    let mut encoded_bytes = 0usize;
    for i in 1..=columns.len() {
        let value = raw.get_ref(i)?;
        let estimated = match value {
            ValueRef::Text(bytes) | ValueRef::Blob(bytes) => bytes.len().checked_mul(2),
            _ => Some(32),
        }
        .ok_or(Error::LimitExceeded)?;
        encoded_bytes = encoded_bytes
            .checked_add(estimated)
            .ok_or(Error::LimitExceeded)?;
        if encoded_bytes > max_bytes {
            return Err(Error::LimitExceeded);
        }
        cells.push(cell(value));
    }
    if rows.next()?.is_some() {
        return Err(Error::UnsupportedShape("duplicate rowid".into()));
    }
    let mut result = OrphanRowV1 {
        table: table.into(),
        rowid,
        columns,
        cells,
        row_sha256: String::new(),
    };
    let bytes = serde_json::to_vec(&result)?;
    if bytes.len() > max_bytes {
        return Err(Error::LimitExceeded);
    }
    result.row_sha256 = digest(&result)?;
    Ok(result)
}

pub(crate) async fn plan(
    store: &MemoryStore,
    max_rows: usize,
    max_bytes: usize,
) -> Result<AuthorityRelationQuarantinePlanV1, Error> {
    if !store.inner.read_only {
        return Err(Error::RequiresReadOnlyStore);
    }
    if max_rows == 0 || max_rows > MAX_PLAN_ROWS || max_bytes == 0 || max_bytes > MAX_PLAN_BYTES {
        return Err(Error::LimitExceeded);
    }
    let path = store.inner.paths.sqlite_path.clone();
    let before = file_digest(&path)?;
    let report = store
        .with_read_conn(move |conn| {
            // Keep a single pinned read view; never invoke owner write helpers.
            let tx = conn.unchecked_transaction()?;
            let result = inspect(&tx, max_rows, max_bytes);
            // Preserve typed errors in the inner result while the pool owns its connection.
            drop(tx);
            Ok(result)
        })
        .await??;
    let after = file_digest(&store.inner.paths.sqlite_path)?;
    if before != after {
        return Err(Error::SnapshotChanged);
    }
    let mut report = AuthorityRelationQuarantinePlanV1 {
        database_sha256: after,
        ..report
    };
    report.plan_sha256.clear();
    report.plan_sha256 = digest(&report)?;
    if serde_json::to_vec(&report)?.len() > max_bytes {
        return Err(Error::LimitExceeded);
    }
    Ok(report)
}
fn inspect(
    conn: &Connection,
    max_rows: usize,
    max_bytes: usize,
) -> Result<AuthorityRelationQuarantinePlanV1, Error> {
    let mode: String = conn.query_row("PRAGMA journal_mode", [], |r| r.get(0))?;
    if mode != "delete" {
        return Err(Error::NotSealed);
    }
    let version: u32 = conn.query_row("PRAGMA user_version", [], |r| r.get(0))?;
    if version != 39 {
        return Err(Error::UnsupportedSchema);
    }
    let migration: u32 = conn.query_row(
        "SELECT COALESCE(MAX(version),0) FROM _schema_version",
        [],
        |r| r.get(0),
    )?;
    if migration != 39 {
        return Err(Error::UnsupportedSchema);
    }
    let mut stmt = conn.prepare(
        "SELECT type,name,tbl_name,sql FROM sqlite_schema ORDER BY type,name,tbl_name,sql",
    )?;
    let schema = stmt
        .query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, Option<String>>(3)?,
            ))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    // A user_version marker alone cannot identify added or modified DDL.
    // Reconstruct the reference in memory; never migrate the inspected image.
    let reference = Connection::open_in_memory()?;
    crate::db::run_migrations(&reference)?;
    let mut reference_stmt = reference.prepare(
        "SELECT type,name,tbl_name,sql FROM sqlite_schema ORDER BY type,name,tbl_name,sql",
    )?;
    let expected_schema = reference_stmt
        .query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, Option<String>>(3)?,
            ))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    if schema != expected_schema {
        return Err(Error::UnsupportedSchema);
    }
    let schema_manifest_sha256 = digest(&schema)?;
    let mut epochs = Vec::new();
    for name in EPOCHS {
        let value: i64 = conn.query_row(
            &format!("SELECT \"{name}\" FROM authority_state WHERE id=1"),
            [],
            |r| r.get(0),
        )?;
        epochs.push((name.into(), value));
    }
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM authority_state", [], |r| r.get(0))?;
    if count != 1 {
        return Err(Error::UnsupportedShape(
            "authority epoch cardinality".into(),
        ));
    }
    let mut stmt = conn.prepare("PRAGMA foreign_key_check")?;
    let mut cursor = stmt.query([])?;
    let mut violations = Vec::new();
    while let Some(row) = cursor.next()? {
        if violations.len() >= max_rows {
            return Err(Error::LimitExceeded);
        }
        violations.push(OrphanViolationV1 {
            table: row.get(0)?,
            rowid: row
                .get::<_, Option<i64>>(1)?
                .ok_or_else(|| Error::UnsupportedShape("WITHOUT ROWID violation".into()))?,
            parent: row.get(2)?,
            foreign_key_id: row.get(3)?,
        });
    }
    violations.sort_by(|a, b| {
        (&a.table, a.rowid, &a.parent, a.foreign_key_id).cmp(&(
            &b.table,
            b.rowid,
            &b.parent,
            b.foreign_key_id,
        ))
    });
    let violations_sha256 = digest(&violations)?;
    let mut keys = BTreeSet::new();
    let mut facts = BTreeSet::new();
    let mut lineages = BTreeSet::new();
    let mut total = serde_json::to_vec(&schema)?
        .len()
        .checked_add(serde_json::to_vec(&violations)?.len())
        .ok_or(Error::LimitExceeded)?;
    for violation in &violations {
        if !TABLES.contains(&violation.table.as_str()) || violation.parent != "facts" {
            return Err(Error::UnsupportedShape(format!(
                "{} -> {}",
                violation.table, violation.parent
            )));
        }
        let fk_sql = format!("PRAGMA foreign_key_list(\"{}\")", violation.table);
        let mut fk_stmt = conn.prepare(&fk_sql)?;
        let mappings = fk_stmt
            .query_map([], |r| {
                Ok((
                    r.get::<_, i64>(0)?,
                    r.get::<_, i64>(1)?,
                    r.get::<_, String>(2)?,
                    r.get::<_, String>(3)?,
                    r.get::<_, String>(4)?,
                ))
            })?
            .collect::<Result<Vec<_>, _>>()?;
        let expected = if violation.table == "authority_lineages" {
            "active_head_id"
        } else {
            "fact_id"
        };
        if mappings
            .iter()
            .filter(|m| m.0 == violation.foreign_key_id)
            .collect::<Vec<_>>()
            != vec![&(
                violation.foreign_key_id,
                0,
                "facts".into(),
                expected.into(),
                "id".into(),
            )]
        {
            return Err(Error::UnsupportedShape(
                "foreign-key mapping differs from supported shape".into(),
            ));
        }
        keys.insert((violation.table.as_str(), violation.rowid));
    }
    if keys.len() > max_rows {
        return Err(Error::LimitExceeded);
    }
    let mut rows = Vec::new();
    for (table, rowid) in keys {
        let row = capture(conn, table, rowid, max_bytes)?;
        let fact_id = text(column(
            &row,
            if table == "authority_lineages" {
                "active_head_id"
            } else {
                "fact_id"
            },
        )?)?;
        let present: bool = conn
            .query_row("SELECT 1 FROM facts WHERE id=?1", [&fact_id], |_| Ok(true))
            .optional()?
            .unwrap_or(false);
        if present {
            return Err(Error::UnsupportedShape("parent fact still exists".into()));
        }
        facts.insert(fact_id);
        if table == "authority_lineages" {
            lineages.insert(text(column(&row, "lineage_id")?)?);
        }
        if table == "authority_versions" {
            lineages.insert(text(column(&row, "lineage_id")?)?);
        }
        total = total
            .checked_add(serde_json::to_vec(&row)?.len())
            .ok_or(Error::LimitExceeded)?;
        if total > max_bytes {
            return Err(Error::LimitExceeded);
        }
        rows.push(row);
    }
    for lineage in &lineages {
        let present: i64 = conn.query_row("SELECT COUNT(*) FROM authority_versions v JOIN facts f ON f.id=v.fact_id WHERE v.lineage_id=?1", [lineage], |r| r.get(0))?;
        if present != 0 {
            return Err(Error::UnsupportedShape(
                "affected lineage has a present sibling fact".into(),
            ));
        }
    }
    // Every selected version must have its lineage captured; no incomplete proposed set.
    for row in rows.iter().filter(|r| r.table == "authority_versions") {
        let id = text(column(row, "lineage_id")?)?;
        if !rows.iter().any(|r| {
            r.table == "authority_lineages"
                && column(r, "lineage_id")
                    .ok()
                    .and_then(|c| text(c).ok())
                    .as_deref()
                    == Some(&id)
        }) {
            return Err(Error::UnsupportedShape(
                "orphan version lacks selected lineage".into(),
            ));
        }
    }
    let rows_sha256 = digest(&rows)?;
    Ok(AuthorityRelationQuarantinePlanV1 {
        schema_version: "authority_relation_quarantine_plan_v1",
        disposition: if rows.is_empty() {
            "no_orphaned_authority_rows"
        } else {
            "proposal_requires_owner_application_authorization"
        },
        application_authorization_required: true,
        database_sha256: String::new(),
        sqlite_user_version: version,
        schema_manifest_sha256,
        epochs,
        violations,
        violations_sha256,
        rows,
        rows_sha256,
        affected_fact_ids: facts.into_iter().collect(),
        affected_lineage_ids: lineages.into_iter().collect(),
        plan_sha256: String::new(),
    })
}

#[cfg(test)]
mod cell_encoding_tests {
    use super::*;

    #[test]
    fn preserves_each_sqlite_cell_type_without_json_numeric_or_utf8_coercion() {
        assert_eq!(cell(ValueRef::Null), SqliteCellV1::Null);
        assert_eq!(
            cell(ValueRef::Integer(i64::MIN)),
            SqliteCellV1::Integer(i64::MIN)
        );
        assert_eq!(
            cell(ValueRef::Real(-0.0)),
            SqliteCellV1::RealBits("8000000000000000".into())
        );
        assert_eq!(
            cell(ValueRef::Text(&[0xff, 0x00])),
            SqliteCellV1::TextHex("ff00".into())
        );
        assert_eq!(
            cell(ValueRef::Blob(&[0x00, 0xff])),
            SqliteCellV1::BlobHex("00ff".into())
        );
    }
}
