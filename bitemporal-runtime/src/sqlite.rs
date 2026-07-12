//! SQLite-backed bitemporal store.
//!
//! This is the durable counterpart to [`InMemoryDb`]. It implements
//! the same surface — `insert`, `snapshot_at` — against a single
//! SQLite table `bitemporal_records`:
//!
//! ```sql
//! CREATE TABLE bitemporal_records (
//!     record_id      TEXT NOT NULL,
//!     valid_time     INTEGER NOT NULL,  -- unix epoch seconds
//!     recorded_time  INTEGER NOT NULL,  -- unix epoch seconds
//!     superseded_by  TEXT,              -- NULL if current
//!     value_json     BLOB NOT NULL,     -- the payload, JSON-encoded
//!     PRIMARY KEY (record_id, valid_time, recorded_time)
//! );
//! ```
//!
//! `insert` is the only mutating operation. It inserts a new row and
//! marks all prior non-superseded rows for the same `record_id` with
//! `superseded_by = <new_record_id>`. This is the SQLite-side analogue
//! of the in-memory `append_supersede`.

use crate::BitemporalError;
use rusqlite::{params, Connection};
use std::path::Path;

/// SQLite-backed bitemporal store. Wraps a single `rusqlite::Connection`.
#[derive(Debug)]
pub struct SqliteDb {
    conn: Connection,
}

impl SqliteDb {
    /// Open (or create) a SQLite-backed bitemporal store at the given path.
    pub fn open(path: impl AsRef<Path>) -> Result<Self, BitemporalError> {
        let conn = Connection::open(path).map_err(|e| {
            BitemporalError::DatabaseError(format!("failed to open sqlite db: {e}"))
        })?;
        let db = Self { conn };
        db.migrate()?;
        Ok(db)
    }

    /// Open an in-memory SQLite-backed bitemporal store. Useful for tests
    /// that want the persistence semantics of SQLite without touching disk.
    pub fn open_in_memory() -> Result<Self, BitemporalError> {
        let conn = Connection::open_in_memory().map_err(|e| {
            BitemporalError::DatabaseError(format!("failed to open in-memory sqlite db: {e}"))
        })?;
        let db = Self { conn };
        db.migrate()?;
        Ok(db)
    }

    fn migrate(&self) -> Result<(), BitemporalError> {
        let has_table: bool = self.conn.query_row(
            "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type='table' AND name='bitemporal_records')",
            [], |row| row.get(0),
        ).map_err(|e| BitemporalError::DatabaseError(format!("migration probe failed: {e}")))?;
        if has_table {
            let has_event_id: bool = self.conn.query_row(
                "SELECT EXISTS(SELECT 1 FROM pragma_table_info('bitemporal_records') WHERE name='event_id')",
                [], |row| row.get(0),
            ).map_err(|e| BitemporalError::DatabaseError(format!("migration probe failed: {e}")))?;
            if !has_event_id {
                self.conn.execute_batch(
                    "DROP INDEX IF EXISTS idx_bt_recorded;
                     DROP INDEX IF EXISTS idx_bt_superseded;
                     ALTER TABLE bitemporal_records RENAME TO bitemporal_records_seconds_v1;
                     CREATE TABLE bitemporal_records (
                       event_id TEXT NOT NULL PRIMARY KEY,
                       record_id TEXT NOT NULL,
                       valid_time_ns INTEGER NOT NULL,
                       recorded_time_ns INTEGER NOT NULL,
                       superseded_by TEXT,
                       value_json BLOB NOT NULL
                     );
                     INSERT INTO bitemporal_records
                       (event_id, record_id, valid_time_ns, recorded_time_ns, superseded_by, value_json)
                     SELECT printf('legacy-v1-%016x', rowid), record_id,
                            valid_time * 1000000000, recorded_time * 1000000000,
                            superseded_by, value_json
                     FROM bitemporal_records_seconds_v1;
                     DROP TABLE bitemporal_records_seconds_v1;"
                ).map_err(|e| BitemporalError::DatabaseError(format!("legacy migration failed: {e}")))?;
            }
        }
        self.conn
            .execute_batch(
                "CREATE TABLE IF NOT EXISTS bitemporal_records (
                    event_id       TEXT NOT NULL PRIMARY KEY,
                    record_id      TEXT NOT NULL,
                    valid_time_ns  INTEGER NOT NULL,
                    recorded_time_ns INTEGER NOT NULL,
                    superseded_by  TEXT,
                    value_json     BLOB NOT NULL
                );
                CREATE INDEX IF NOT EXISTS idx_bt_recorded ON bitemporal_records(recorded_time_ns, event_id);
                CREATE INDEX IF NOT EXISTS idx_bt_superseded ON bitemporal_records(superseded_by);",
            )
            .map_err(|e| BitemporalError::DatabaseError(format!("migration failed: {e}")))?;
        Ok(())
    }

    /// Insert a bitemporal record. Returns the number of prior rows
    /// that were marked superseded (0 if this is the first version).
    pub fn insert(
        &self,
        record: crate::BitemporalRecord<serde_json::Value>,
    ) -> Result<usize, BitemporalError> {
        let value_bytes = serde_json::to_vec(&record.value).map_err(|e| {
            BitemporalError::SerializationError(format!("value serialization failed: {e}"))
        })?;
        let valid_ns = record.valid_time.timestamp_nanos_opt().ok_or_else(|| {
            BitemporalError::SerializationError("valid_time outside nanosecond range".into())
        })?;
        let recorded_ns = record.recorded_time.timestamp_nanos_opt().ok_or_else(|| {
            BitemporalError::SerializationError("recorded_time outside nanosecond range".into())
        })?;
        use sha2::{Digest, Sha256};
        let event_id = format!(
            "{:x}",
            Sha256::digest(
                [
                    b"bitemporal-event:v2\0".as_slice(),
                    record.id.as_bytes(),
                    &valid_ns.to_be_bytes(),
                    &recorded_ns.to_be_bytes(),
                    &value_bytes
                ]
                .concat()
            )
        );

        let tx = self
            .conn
            .unchecked_transaction()
            .map_err(|e| BitemporalError::DatabaseError(format!("failed to begin tx: {e}")))?;

        // Step 1: mark all prior non-superseded rows superseded.
        // Must run BEFORE the insert so the new row doesn't get
        // marked superseded by itself.
        let superseded_count = tx
            .execute(
                "UPDATE bitemporal_records
                 SET superseded_by = ?1
                 WHERE record_id = ?2
                   AND superseded_by IS NULL",
                params![record.id, record.id],
            )
            .map_err(|e| {
                BitemporalError::DatabaseError(format!("supersession update failed: {e}"))
            })?;

        // Step 2: insert the new row.
        tx.execute(
            "INSERT INTO bitemporal_records
             (event_id, record_id, valid_time_ns, recorded_time_ns, superseded_by, value_json)
             VALUES (?1, ?2, ?3, ?4, NULL, ?5)",
            params![event_id, record.id, valid_ns, recorded_ns, value_bytes,],
        )
        .map_err(|e| BitemporalError::DatabaseError(format!("insert failed: {e}")))?;

        tx.commit()
            .map_err(|e| BitemporalError::DatabaseError(format!("commit failed: {e}")))?;

        Ok(superseded_count)
    }

    /// Snapshot of the bitemporal state at `recorded_time`. Returns one
    /// record per `record_id` — the latest version of that record whose
    /// `recorded_time` is `<= recorded_time`, regardless of whether it
    /// has been superseded since. This is the "as of" semantics: what
    /// did we believe at time T, even if a later version has
    /// superseded it.
    pub fn snapshot_at(
        &self,
        recorded_time: chrono::DateTime<chrono::Utc>,
    ) -> Result<Vec<crate::BitemporalRecord<serde_json::Value>>, BitemporalError> {
        let query_ns = recorded_time.timestamp_nanos_opt().ok_or_else(|| {
            BitemporalError::DatabaseError("snapshot timestamp outside nanosecond range".into())
        })?;
        let mut stmt = self
            .conn
            .prepare(
                "SELECT record_id, valid_time_ns, recorded_time_ns, value_json
                 FROM bitemporal_records
                 WHERE recorded_time_ns <= ?1
                 ORDER BY record_id, recorded_time_ns DESC, event_id DESC",
            )
            .map_err(|e| BitemporalError::DatabaseError(format!("snapshot prepare failed: {e}")))?;

        let rows = stmt
            .query_map(params![query_ns], |row| {
                let id: String = row.get(0)?;
                let valid_ts: i64 = row.get(1)?;
                let recorded_ts: i64 = row.get(2)?;
                let value_bytes: Vec<u8> = row.get(3)?;
                let value: serde_json::Value =
                    serde_json::from_slice(&value_bytes).map_err(|e| {
                        rusqlite::Error::FromSqlConversionFailure(
                            3,
                            rusqlite::types::Type::Blob,
                            Box::new(e),
                        )
                    })?;
                let valid_time = chrono::DateTime::<chrono::Utc>::from_timestamp(
                    valid_ts.div_euclid(1_000_000_000),
                    valid_ts.rem_euclid(1_000_000_000) as u32,
                )
                .ok_or_else(|| rusqlite::Error::InvalidQuery)?;
                let recorded_time = chrono::DateTime::<chrono::Utc>::from_timestamp(
                    recorded_ts.div_euclid(1_000_000_000),
                    recorded_ts.rem_euclid(1_000_000_000) as u32,
                )
                .ok_or_else(|| rusqlite::Error::InvalidQuery)?;
                Ok(crate::BitemporalRecord {
                    id,
                    valid_time,
                    recorded_time,
                    value,
                })
            })
            .map_err(|e| BitemporalError::DatabaseError(format!("snapshot query failed: {e}")))?;

        // Dedup: keep only the latest recorded_time per record_id.
        let mut by_id: std::collections::BTreeMap<
            String,
            crate::BitemporalRecord<serde_json::Value>,
        > = std::collections::BTreeMap::new();
        for row in rows {
            let r = row.map_err(|e| BitemporalError::DatabaseError(format!("row decode: {e}")))?;
            by_id
                .entry(r.id.clone())
                .and_modify(|existing| {
                    if r.recorded_time > existing.recorded_time {
                        *existing = r.clone();
                    }
                })
                .or_insert(r);
        }
        Ok(by_id.into_values().collect())
    }
}
