//! # cea-sqlite
//!
//! SQLite persistence adapter for the internal CEA stack.
//!
//! This Tier 3 crate provides a durable `cea-store` implementation for
//! `forge-engine` and related internal evaluation flows. It owns no causal
//! attribution semantics; those remain in `cea-core`.

use std::path::{Path, PathBuf};
use std::time::Duration;

use rusqlite::OptionalExtension;

const CURRENT_SCHEMA_VERSION: i64 = 2;

const SCHEMA_V2_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS cea_nodes (
    node_id TEXT PRIMARY KEY,
    node_kind TEXT NOT NULL,
    sig_json TEXT NOT NULL,
    first_seen TEXT NOT NULL,
    last_seen TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS cea_edges (
    edge_id TEXT PRIMARY KEY,
    cause_node_id TEXT NOT NULL,
    effect_node_id TEXT NOT NULL,
    weight REAL NOT NULL,
    count INTEGER NOT NULL,
    confidence REAL NOT NULL,
    alpha REAL NOT NULL,
    beta REAL NOT NULL,
    version_id TEXT NOT NULL,
    last_seen TEXT NOT NULL,
    UNIQUE(cause_node_id, effect_node_id, version_id)
);
CREATE INDEX IF NOT EXISTS idx_cea_edges_version_id ON cea_edges(version_id);
CREATE TABLE IF NOT EXISTS cea_run_log (
    run_hash TEXT PRIMARY KEY,
    eval_id TEXT NOT NULL,
    edges_added INTEGER NOT NULL,
    edges_updated INTEGER NOT NULL,
    processed_at TEXT NOT NULL
);
"#;

pub struct SqliteCeaStore {
    path: PathBuf,
}

impl SqliteCeaStore {
    pub fn open(path: &Path) -> Result<Self, cea_store::CeaStoreError> {
        let store = Self {
            path: path.to_path_buf(),
        };
        let _ = store.connect()?;
        Ok(store)
    }

    fn connect(&self) -> Result<rusqlite::Connection, cea_store::CeaStoreError> {
        let connection = rusqlite::Connection::open(&self.path)
            .map_err(|error| cea_store::CeaStoreError::Backend(error.to_string()))?;
        configure_connection(&connection)?;
        ensure_schema(&connection)?;
        Ok(connection)
    }
}

pub struct SqliteCeaStoreConn<'conn> {
    conn: &'conn rusqlite::Connection,
}

impl<'conn> SqliteCeaStoreConn<'conn> {
    pub fn new(conn: &'conn rusqlite::Connection) -> Self {
        Self { conn }
    }
}

pub struct SqliteCeaStoreTx<'tx, 'conn> {
    tx: &'tx rusqlite::Transaction<'conn>,
}

impl<'tx, 'conn> SqliteCeaStoreTx<'tx, 'conn> {
    pub fn new(tx: &'tx rusqlite::Transaction<'conn>) -> Self {
        Self { tx }
    }
}

struct SqliteCeaWriteTx<'tx, 'conn> {
    tx: &'tx rusqlite::Transaction<'conn>,
}

fn configure_connection(conn: &rusqlite::Connection) -> Result<(), cea_store::CeaStoreError> {
    conn.pragma_update(None, "journal_mode", "WAL")
        .map_err(|error| cea_store::CeaStoreError::Backend(error.to_string()))?;
    conn.pragma_update(None, "foreign_keys", "ON")
        .map_err(|error| cea_store::CeaStoreError::Backend(error.to_string()))?;
    conn.busy_timeout(Duration::from_secs(5))
        .map_err(|error| cea_store::CeaStoreError::Backend(error.to_string()))?;
    Ok(())
}

fn ensure_schema(conn: &rusqlite::Connection) -> Result<(), cea_store::CeaStoreError> {
    let mut version = read_schema_version(conn)?;

    if version > CURRENT_SCHEMA_VERSION {
        return Err(cea_store::CeaStoreError::Backend(format!(
            "cea-sqlite schema version {version} is newer than supported {CURRENT_SCHEMA_VERSION}"
        )));
    }

    if version == 0 {
        version = bootstrap_or_adopt_schema(conn)?;
    }

    while version < CURRENT_SCHEMA_VERSION {
        version = migrate_schema(conn, version)?;
    }

    Ok(())
}

fn read_schema_version(conn: &rusqlite::Connection) -> Result<i64, cea_store::CeaStoreError> {
    conn.pragma_query_value(None, "user_version", |row| row.get(0))
        .map_err(|error| cea_store::CeaStoreError::Backend(error.to_string()))
}

fn write_schema_version(
    conn: &rusqlite::Connection,
    version: i64,
) -> Result<(), cea_store::CeaStoreError> {
    conn.pragma_update(None, "user_version", version)
        .map_err(|error| cea_store::CeaStoreError::Backend(error.to_string()))
}

fn bootstrap_or_adopt_schema(conn: &rusqlite::Connection) -> Result<i64, cea_store::CeaStoreError> {
    if !table_exists(conn, "cea_nodes")?
        && !table_exists(conn, "cea_edges")?
        && !table_exists(conn, "cea_run_log")?
    {
        conn.execute_batch(SCHEMA_V2_SQL)
            .map_err(|error| cea_store::CeaStoreError::Backend(error.to_string()))?;
        write_schema_version(conn, CURRENT_SCHEMA_VERSION)?;
        return Ok(CURRENT_SCHEMA_VERSION);
    }

    let version = detect_existing_schema_version(conn)?;
    if version == CURRENT_SCHEMA_VERSION {
        conn.execute_batch(SCHEMA_V2_SQL)
            .map_err(|error| cea_store::CeaStoreError::Backend(error.to_string()))?;
    }
    write_schema_version(conn, version)?;
    Ok(version)
}

fn detect_existing_schema_version(
    conn: &rusqlite::Connection,
) -> Result<i64, cea_store::CeaStoreError> {
    let has_nodes = table_exists(conn, "cea_nodes")?;
    let has_edges = table_exists(conn, "cea_edges")?;
    let has_run_log = table_exists(conn, "cea_run_log")?;
    if !(has_nodes && has_edges && has_run_log) {
        return Err(cea_store::CeaStoreError::Backend(
            "incomplete cea-sqlite schema; expected cea_nodes, cea_edges, and cea_run_log"
                .to_string(),
        ));
    }

    if has_legacy_edges_schema(conn)? || has_blocked_run_log_schema(conn)? {
        return Ok(1);
    }

    Ok(CURRENT_SCHEMA_VERSION)
}

fn migrate_schema(
    conn: &rusqlite::Connection,
    version: i64,
) -> Result<i64, cea_store::CeaStoreError> {
    match version {
        1 => {
            migrate_v1_to_v2(conn)?;
            conn.execute_batch(SCHEMA_V2_SQL)
                .map_err(|error| cea_store::CeaStoreError::Backend(error.to_string()))?;
            write_schema_version(conn, CURRENT_SCHEMA_VERSION)?;
            Ok(CURRENT_SCHEMA_VERSION)
        }
        _ => Err(cea_store::CeaStoreError::Backend(format!(
            "unsupported cea-sqlite schema version migration from {version}"
        ))),
    }
}

fn has_legacy_edges_schema(conn: &rusqlite::Connection) -> Result<bool, cea_store::CeaStoreError> {
    if !table_exists(conn, "cea_edges")? {
        return Ok(false);
    }
    Ok(!column_exists(conn, "cea_edges", "alpha")? || !column_exists(conn, "cea_edges", "beta")?)
}

fn has_blocked_run_log_schema(
    conn: &rusqlite::Connection,
) -> Result<bool, cea_store::CeaStoreError> {
    let sql: Option<String> = conn
        .query_row(
            "SELECT sql FROM sqlite_master WHERE type = 'table' AND name = 'cea_run_log'",
            [],
            |row| row.get(0),
        )
        .optional()
        .map_err(|error| cea_store::CeaStoreError::Backend(error.to_string()))?;
    Ok(sql
        .as_deref()
        .is_some_and(|sql| sql.contains("CHECK(eval_id != 'blocked')")))
}

fn table_exists(
    conn: &rusqlite::Connection,
    table: &str,
) -> Result<bool, cea_store::CeaStoreError> {
    let exists: Option<i64> = conn
        .query_row(
            "SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = ?1",
            [table],
            |row| row.get(0),
        )
        .optional()
        .map_err(|error| cea_store::CeaStoreError::Backend(error.to_string()))?;
    Ok(exists.is_some())
}

fn column_exists(
    conn: &rusqlite::Connection,
    table: &str,
    column: &str,
) -> Result<bool, cea_store::CeaStoreError> {
    let mut stmt = conn
        .prepare(&format!("PRAGMA table_info({table})"))
        .map_err(|error| cea_store::CeaStoreError::Backend(error.to_string()))?;
    let columns = stmt
        .query_map([], |row| row.get::<_, String>(1))
        .map_err(|error| cea_store::CeaStoreError::Backend(error.to_string()))?;
    for name in columns {
        if name.map_err(|error| cea_store::CeaStoreError::Backend(error.to_string()))? == column {
            return Ok(true);
        }
    }
    Ok(false)
}

fn migrate_v1_to_v2(conn: &rusqlite::Connection) -> Result<(), cea_store::CeaStoreError> {
    if has_legacy_edges_schema(conn)? {
        conn.execute_batch(
            r#"
CREATE TABLE cea_edges_v2 (
    edge_id TEXT PRIMARY KEY,
    cause_node_id TEXT NOT NULL,
    effect_node_id TEXT NOT NULL,
    weight REAL NOT NULL,
    count INTEGER NOT NULL,
    confidence REAL NOT NULL,
    alpha REAL NOT NULL,
    beta REAL NOT NULL,
    version_id TEXT NOT NULL,
    last_seen TEXT NOT NULL,
    UNIQUE(cause_node_id, effect_node_id, version_id)
);
INSERT INTO cea_edges_v2 (
    edge_id, cause_node_id, effect_node_id, weight, count, confidence, alpha, beta, version_id, last_seen
)
SELECT
    edge_id,
    cause_node_id,
    effect_node_id,
    weight,
    count,
    confidence,
    CASE
        WHEN count > 0 THEN CAST(count AS REAL)
        ELSE 1.0
    END,
    1.0,
    version_id,
    last_seen
FROM cea_edges;
DROP TABLE cea_edges;
ALTER TABLE cea_edges_v2 RENAME TO cea_edges;
"#,
        )
        .map_err(|error| cea_store::CeaStoreError::Backend(error.to_string()))?;
    }

    if has_blocked_run_log_schema(conn)? {
        conn.execute_batch(
            r#"
CREATE TABLE cea_run_log_v2 (
    run_hash TEXT PRIMARY KEY,
    eval_id TEXT NOT NULL,
    edges_added INTEGER NOT NULL,
    edges_updated INTEGER NOT NULL,
    processed_at TEXT NOT NULL
);
INSERT INTO cea_run_log_v2 (run_hash, eval_id, edges_added, edges_updated, processed_at)
SELECT run_hash, eval_id, edges_added, edges_updated, processed_at
FROM cea_run_log;
DROP TABLE cea_run_log;
ALTER TABLE cea_run_log_v2 RENAME TO cea_run_log;
"#,
        )
        .map_err(|error| cea_store::CeaStoreError::Backend(error.to_string()))?;
    }

    Ok(())
}

fn positive_stats(weight_delta: f64) -> cea_core::EdgeStats {
    let mut stats = cea_core::EdgeStats::default();
    stats.observe_positive(weight_delta);
    stats
}

fn stats_from_persisted(alpha: f64, beta: f64, count: i64) -> cea_core::EdgeStats {
    cea_core::EdgeStats {
        alpha,
        beta,
        observations: count.max(0) as u64,
    }
}

fn map_edge_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<cea_store::CeaEdgeRow> {
    let count: i64 = row.get(4)?;
    let alpha: f64 = row.get(6)?;
    let beta: f64 = row.get(7)?;
    let stats = stats_from_persisted(alpha, beta, count);
    Ok(cea_store::CeaEdgeRow {
        edge_id: row.get(0)?,
        cause_node_id: row.get(1)?,
        effect_node_id: row.get(2)?,
        weight: row.get(3)?,
        alpha,
        beta,
        count,
        confidence: stats.confidence(),
        version_id: row.get(8)?,
        last_seen: row.get(9)?,
    })
}

impl cea_store::CeaStoreWriteTx for SqliteCeaWriteTx<'_, '_> {
    fn has_run(&self, run_hash: &str) -> Result<bool, cea_store::CeaStoreError> {
        let count: i64 = self
            .tx
            .query_row(
                "SELECT COUNT(*) FROM cea_run_log WHERE run_hash = ?1",
                [run_hash],
                |row| row.get(0),
            )
            .map_err(|error| cea_store::CeaStoreError::Backend(error.to_string()))?;
        Ok(count > 0)
    }

    fn upsert_node(
        &self,
        node_id: &str,
        node_kind: &str,
        sig_json: &str,
    ) -> Result<(), cea_store::CeaStoreError> {
        let now = chrono::Utc::now().to_rfc3339();
        self.tx
            .execute(
                "INSERT INTO cea_nodes (node_id, node_kind, sig_json, first_seen, last_seen) VALUES (?1, ?2, ?3, ?4, ?4) ON CONFLICT(node_id) DO UPDATE SET last_seen = ?4",
                rusqlite::params![node_id, node_kind, sig_json, now],
            )
            .map_err(|error| cea_store::CeaStoreError::Backend(error.to_string()))?;
        Ok(())
    }

    fn upsert_edge(
        &self,
        edge_id: &str,
        cause_node_id: &str,
        effect_node_id: &str,
        weight_delta: f64,
        version_id: &str,
    ) -> Result<bool, cea_store::CeaStoreError> {
        let now = chrono::Utc::now().to_rfc3339();
        let weight_delta = weight_delta.max(0.0);
        let existing = self
            .tx
            .query_row(
                "SELECT weight, count, alpha, beta FROM cea_edges WHERE cause_node_id = ?1 AND effect_node_id = ?2 AND version_id = ?3",
                rusqlite::params![cause_node_id, effect_node_id, version_id],
                |row| {
                    Ok((
                        row.get::<_, f64>(0)?,
                        row.get::<_, i64>(1)?,
                        row.get::<_, f64>(2)?,
                        row.get::<_, f64>(3)?,
                    ))
                },
            )
            .optional()
            .map_err(|error| cea_store::CeaStoreError::Backend(error.to_string()))?;

        if let Some((weight, count, alpha, beta)) = existing {
            let mut stats = stats_from_persisted(alpha, beta, count);
            stats.observe_positive(weight_delta);
            self.tx
                .execute(
                    "UPDATE cea_edges SET weight = ?1, count = ?2, confidence = ?3, alpha = ?4, beta = ?5, last_seen = ?6 WHERE cause_node_id = ?7 AND effect_node_id = ?8 AND version_id = ?9",
                    rusqlite::params![
                        weight + weight_delta,
                        stats.observations as i64,
                        stats.confidence(),
                        stats.alpha,
                        stats.beta,
                        now,
                        cause_node_id,
                        effect_node_id,
                        version_id
                    ],
                )
                .map_err(|error| cea_store::CeaStoreError::Backend(error.to_string()))?;
            return Ok(false);
        }

        let stats = positive_stats(weight_delta);
        self.tx
            .execute(
                "INSERT INTO cea_edges (edge_id, cause_node_id, effect_node_id, weight, count, confidence, alpha, beta, version_id, last_seen) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                rusqlite::params![
                    edge_id,
                    cause_node_id,
                    effect_node_id,
                    weight_delta,
                    stats.observations as i64,
                    stats.confidence(),
                    stats.alpha,
                    stats.beta,
                    version_id,
                    now
                ],
            )
            .map_err(|error| cea_store::CeaStoreError::Backend(error.to_string()))?;

        Ok(true)
    }

    fn load_effect_ids_for_cause(
        &self,
        cause_node_id: &str,
        version_id: &str,
    ) -> Result<Vec<String>, cea_store::CeaStoreError> {
        let mut stmt = self
            .tx
            .prepare(
                "SELECT effect_node_id FROM cea_edges WHERE cause_node_id = ?1 AND version_id = ?2 ORDER BY effect_node_id",
            )
            .map_err(|error| cea_store::CeaStoreError::Backend(error.to_string()))?;
        let rows = stmt
            .query_map(rusqlite::params![cause_node_id, version_id], |row| {
                row.get(0)
            })
            .map_err(|error| cea_store::CeaStoreError::Backend(error.to_string()))?;

        let mut effect_ids = Vec::new();
        for row in rows {
            effect_ids
                .push(row.map_err(|error| cea_store::CeaStoreError::Backend(error.to_string()))?);
        }
        Ok(effect_ids)
    }

    fn reinforce_negative_edge(
        &self,
        cause_node_id: &str,
        effect_node_id: &str,
        amount: f64,
        version_id: &str,
    ) -> Result<(), cea_store::CeaStoreError> {
        let now = chrono::Utc::now().to_rfc3339();
        let amount = amount.max(0.0);
        let existing = self
            .tx
            .query_row(
                "SELECT count, alpha, beta FROM cea_edges WHERE cause_node_id = ?1 AND effect_node_id = ?2 AND version_id = ?3",
                rusqlite::params![cause_node_id, effect_node_id, version_id],
                |row| {
                    Ok((
                        row.get::<_, i64>(0)?,
                        row.get::<_, f64>(1)?,
                        row.get::<_, f64>(2)?,
                    ))
                },
            )
            .optional()
            .map_err(|error| cea_store::CeaStoreError::Backend(error.to_string()))?
            .ok_or_else(|| {
                cea_store::CeaStoreError::Backend(format!(
                    "missing edge for negative observation: {cause_node_id} -> {effect_node_id} ({version_id})"
                ))
            })?;

        let (count, alpha, beta) = existing;
        let mut stats = stats_from_persisted(alpha, beta, count);
        stats.observe_negative(amount);
        self.tx
            .execute(
                "UPDATE cea_edges SET count = ?1, confidence = ?2, alpha = ?3, beta = ?4, last_seen = ?5 WHERE cause_node_id = ?6 AND effect_node_id = ?7 AND version_id = ?8",
                rusqlite::params![
                    stats.observations as i64,
                    stats.confidence(),
                    stats.alpha,
                    stats.beta,
                    now,
                    cause_node_id,
                    effect_node_id,
                    version_id
                ],
            )
            .map_err(|error| cea_store::CeaStoreError::Backend(error.to_string()))?;
        Ok(())
    }

    fn insert_run_log(
        &self,
        run_hash: &str,
        eval_id: &str,
        edges_added: i64,
        edges_updated: i64,
    ) -> Result<(), cea_store::CeaStoreError> {
        let now = chrono::Utc::now().to_rfc3339();
        self.tx
            .execute(
                "INSERT INTO cea_run_log (run_hash, eval_id, edges_added, edges_updated, processed_at) VALUES (?1, ?2, ?3, ?4, ?5)",
                rusqlite::params![run_hash, eval_id, edges_added, edges_updated, now],
            )
            .map_err(|error| cea_store::CeaStoreError::Backend(error.to_string()))?;
        Ok(())
    }
}

impl<'conn> cea_store::CeaStore for SqliteCeaStoreConn<'conn> {
    fn with_write_tx<T, F>(&self, f: F) -> Result<T, cea_store::CeaStoreError>
    where
        F: FnOnce(&dyn cea_store::CeaStoreWriteTx) -> Result<T, cea_store::CeaStoreError>,
    {
        let _ = f;
        Err(cea_store::CeaStoreError::Backend(
            "SqliteCeaStoreConn does not open write transactions; use SqliteCeaStore or SqliteCeaStoreTx"
                .to_string(),
        ))
    }

    fn load_nodes(&self) -> Result<Vec<cea_store::CeaNodeRow>, cea_store::CeaStoreError> {
        let mut statement = self
            .conn
            .prepare("SELECT node_id, node_kind, sig_json FROM cea_nodes")
            .map_err(|error| cea_store::CeaStoreError::Backend(error.to_string()))?;
        let rows = statement
            .query_map([], |row| {
                Ok(cea_store::CeaNodeRow {
                    node_id: row.get(0)?,
                    node_kind: row.get(1)?,
                    sig_json: row.get(2)?,
                })
            })
            .map_err(|error| cea_store::CeaStoreError::Backend(error.to_string()))?;

        let mut nodes = Vec::new();
        for row in rows {
            nodes.push(row.map_err(|error| cea_store::CeaStoreError::Backend(error.to_string()))?);
        }
        Ok(nodes)
    }

    fn load_edges(
        &self,
        version_id: Option<&str>,
    ) -> Result<Vec<cea_store::CeaEdgeRow>, cea_store::CeaStoreError> {
        let query = if version_id.is_some() {
            "SELECT edge_id, cause_node_id, effect_node_id, weight, count, confidence, alpha, beta, version_id, last_seen FROM cea_edges WHERE version_id = ?1"
        } else {
            "SELECT edge_id, cause_node_id, effect_node_id, weight, count, confidence, alpha, beta, version_id, last_seen FROM cea_edges"
        };

        let mut statement = self
            .conn
            .prepare(query)
            .map_err(|error| cea_store::CeaStoreError::Backend(error.to_string()))?;

        let rows = if let Some(version_id) = version_id {
            statement
                .query_map([version_id], map_edge_row)
                .map_err(|error| cea_store::CeaStoreError::Backend(error.to_string()))?
        } else {
            statement
                .query_map([], map_edge_row)
                .map_err(|error| cea_store::CeaStoreError::Backend(error.to_string()))?
        };

        let mut edges = Vec::new();
        for row in rows {
            edges.push(row.map_err(|error| cea_store::CeaStoreError::Backend(error.to_string()))?);
        }
        Ok(edges)
    }
}

impl<'tx, 'conn> cea_store::CeaStore for SqliteCeaStoreTx<'tx, 'conn> {
    fn with_write_tx<T, F>(&self, f: F) -> Result<T, cea_store::CeaStoreError>
    where
        F: FnOnce(&dyn cea_store::CeaStoreWriteTx) -> Result<T, cea_store::CeaStoreError>,
    {
        let write_tx = SqliteCeaWriteTx { tx: self.tx };
        f(&write_tx)
    }

    fn load_nodes(&self) -> Result<Vec<cea_store::CeaNodeRow>, cea_store::CeaStoreError> {
        let mut statement = self
            .tx
            .prepare("SELECT node_id, node_kind, sig_json FROM cea_nodes")
            .map_err(|error| cea_store::CeaStoreError::Backend(error.to_string()))?;
        let rows = statement
            .query_map([], |row| {
                Ok(cea_store::CeaNodeRow {
                    node_id: row.get(0)?,
                    node_kind: row.get(1)?,
                    sig_json: row.get(2)?,
                })
            })
            .map_err(|error| cea_store::CeaStoreError::Backend(error.to_string()))?;

        let mut nodes = Vec::new();
        for row in rows {
            nodes.push(row.map_err(|error| cea_store::CeaStoreError::Backend(error.to_string()))?);
        }
        Ok(nodes)
    }

    fn load_edges(
        &self,
        version_id: Option<&str>,
    ) -> Result<Vec<cea_store::CeaEdgeRow>, cea_store::CeaStoreError> {
        let query = if version_id.is_some() {
            "SELECT edge_id, cause_node_id, effect_node_id, weight, count, confidence, alpha, beta, version_id, last_seen FROM cea_edges WHERE version_id = ?1"
        } else {
            "SELECT edge_id, cause_node_id, effect_node_id, weight, count, confidence, alpha, beta, version_id, last_seen FROM cea_edges"
        };

        let mut statement = self
            .tx
            .prepare(query)
            .map_err(|error| cea_store::CeaStoreError::Backend(error.to_string()))?;

        let rows = if let Some(version_id) = version_id {
            statement
                .query_map([version_id], map_edge_row)
                .map_err(|error| cea_store::CeaStoreError::Backend(error.to_string()))?
        } else {
            statement
                .query_map([], map_edge_row)
                .map_err(|error| cea_store::CeaStoreError::Backend(error.to_string()))?
        };

        let mut edges = Vec::new();
        for row in rows {
            edges.push(row.map_err(|error| cea_store::CeaStoreError::Backend(error.to_string()))?);
        }
        Ok(edges)
    }
}

impl cea_store::CeaStore for SqliteCeaStore {
    fn with_write_tx<T, F>(&self, f: F) -> Result<T, cea_store::CeaStoreError>
    where
        F: FnOnce(&dyn cea_store::CeaStoreWriteTx) -> Result<T, cea_store::CeaStoreError>,
    {
        let mut connection = self.connect()?;
        let tx = connection
            .transaction()
            .map_err(|error| cea_store::CeaStoreError::Backend(error.to_string()))?;
        let write_tx = SqliteCeaWriteTx { tx: &tx };
        let result = f(&write_tx)?;
        tx.commit()
            .map_err(|error| cea_store::CeaStoreError::Backend(error.to_string()))?;
        Ok(result)
    }

    fn load_nodes(&self) -> Result<Vec<cea_store::CeaNodeRow>, cea_store::CeaStoreError> {
        let connection = self.connect()?;
        let mut statement = connection
            .prepare("SELECT node_id, node_kind, sig_json FROM cea_nodes")
            .map_err(|error| cea_store::CeaStoreError::Backend(error.to_string()))?;
        let rows = statement
            .query_map([], |row| {
                Ok(cea_store::CeaNodeRow {
                    node_id: row.get(0)?,
                    node_kind: row.get(1)?,
                    sig_json: row.get(2)?,
                })
            })
            .map_err(|error| cea_store::CeaStoreError::Backend(error.to_string()))?;

        let mut nodes = Vec::new();
        for row in rows {
            nodes.push(row.map_err(|error| cea_store::CeaStoreError::Backend(error.to_string()))?);
        }
        Ok(nodes)
    }

    fn load_edges(
        &self,
        version_id: Option<&str>,
    ) -> Result<Vec<cea_store::CeaEdgeRow>, cea_store::CeaStoreError> {
        let connection = self.connect()?;

        let query = if version_id.is_some() {
            "SELECT edge_id, cause_node_id, effect_node_id, weight, count, confidence, alpha, beta, version_id, last_seen FROM cea_edges WHERE version_id = ?1"
        } else {
            "SELECT edge_id, cause_node_id, effect_node_id, weight, count, confidence, alpha, beta, version_id, last_seen FROM cea_edges"
        };

        let mut statement = connection
            .prepare(query)
            .map_err(|error| cea_store::CeaStoreError::Backend(error.to_string()))?;

        let rows = if let Some(version_id) = version_id {
            statement
                .query_map([version_id], map_edge_row)
                .map_err(|error| cea_store::CeaStoreError::Backend(error.to_string()))?
        } else {
            statement
                .query_map([], map_edge_row)
                .map_err(|error| cea_store::CeaStoreError::Backend(error.to_string()))?
        };

        let mut edges = Vec::new();
        for row in rows {
            edges.push(row.map_err(|error| cea_store::CeaStoreError::Backend(error.to_string()))?);
        }
        Ok(edges)
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use cea_core::{
        AnchorKind, AttributedRunResult, AttributionTriple, EditOpKind, EditOpSignature, FileIndex,
        OpIndex, ScopeTag,
    };
    use check_runner::{CheckResult, EffectSignature, ParsedCheckOutput};
    use tempfile::TempDir;

    use super::{SqliteCeaStore, CURRENT_SCHEMA_VERSION, SCHEMA_V2_SQL};

    fn create_current_schema(path: &Path) {
        let conn = rusqlite::Connection::open(path).unwrap();
        conn.execute_batch(SCHEMA_V2_SQL).unwrap();
    }

    fn create_legacy_schema(path: &Path) {
        let conn = rusqlite::Connection::open(path).unwrap();
        conn.execute_batch(
            r#"
CREATE TABLE cea_nodes (
    node_id TEXT PRIMARY KEY,
    node_kind TEXT NOT NULL,
    sig_json TEXT NOT NULL,
    first_seen TEXT NOT NULL,
    last_seen TEXT NOT NULL
);
CREATE TABLE cea_edges (
    edge_id TEXT PRIMARY KEY,
    cause_node_id TEXT NOT NULL,
    effect_node_id TEXT NOT NULL,
    weight REAL NOT NULL,
    count INTEGER NOT NULL,
    confidence REAL NOT NULL,
    version_id TEXT NOT NULL,
    last_seen TEXT NOT NULL,
    UNIQUE(cause_node_id, effect_node_id, version_id)
);
CREATE TABLE cea_run_log (
    run_hash TEXT PRIMARY KEY,
    eval_id TEXT NOT NULL CHECK(eval_id != 'blocked'),
    edges_added INTEGER NOT NULL,
    edges_updated INTEGER NOT NULL,
    processed_at TEXT NOT NULL
);
"#,
        )
        .unwrap();
    }

    fn install_run_log_failure_trigger(path: &Path) {
        let conn = rusqlite::Connection::open(path).unwrap();
        conn.execute_batch(
            r#"
CREATE TRIGGER fail_blocked_run_log_insert
BEFORE INSERT ON cea_run_log
WHEN NEW.eval_id = 'blocked'
BEGIN
    SELECT RAISE(ABORT, 'blocked run log insert');
END;
"#,
        )
        .unwrap();
    }

    #[test]
    fn open_creates_schema_for_fresh_database() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("cea.db");

        let store = SqliteCeaStore::open(&path).unwrap();
        let conn = store.connect().unwrap();
        let user_version: i64 = conn
            .pragma_query_value(None, "user_version", |row| row.get(0))
            .unwrap();
        let nodes: i64 = conn
            .query_row("SELECT COUNT(*) FROM cea_nodes", [], |row| row.get(0))
            .unwrap();
        let edges: i64 = conn
            .query_row("SELECT COUNT(*) FROM cea_edges", [], |row| row.get(0))
            .unwrap();
        let runs: i64 = conn
            .query_row("SELECT COUNT(*) FROM cea_run_log", [], |row| row.get(0))
            .unwrap();

        assert_eq!(user_version, CURRENT_SCHEMA_VERSION);
        assert_eq!((nodes, edges, runs), (0, 0, 0));
        assert!(super::column_exists(&conn, "cea_edges", "alpha").unwrap());
        assert!(super::column_exists(&conn, "cea_edges", "beta").unwrap());
    }

    #[test]
    fn open_adopts_existing_current_schema_without_manual_versioning() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("cea.db");
        create_current_schema(&path);

        let conn = rusqlite::Connection::open(&path).unwrap();
        let user_version: i64 = conn
            .pragma_query_value(None, "user_version", |row| row.get(0))
            .unwrap();
        assert_eq!(user_version, 0);
        drop(conn);

        let store = SqliteCeaStore::open(&path).unwrap();
        let conn = store.connect().unwrap();
        let user_version: i64 = conn
            .pragma_query_value(None, "user_version", |row| row.get(0))
            .unwrap();
        assert_eq!(user_version, CURRENT_SCHEMA_VERSION);
    }

    #[test]
    fn open_migrates_legacy_schema_and_preserves_rows() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("cea.db");
        create_legacy_schema(&path);

        let legacy_conn = rusqlite::Connection::open(&path).unwrap();
        legacy_conn
            .execute(
                "INSERT INTO cea_edges (edge_id, cause_node_id, effect_node_id, weight, count, confidence, version_id, last_seen) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                rusqlite::params![
                    "legacy-edge",
                    "cause",
                    "effect",
                    3.5_f64,
                    4_i64,
                    0.6_f64,
                    "v1",
                    "now"
                ],
            )
            .unwrap();
        drop(legacy_conn);

        let store = SqliteCeaStore::open(&path).unwrap();
        let conn = store.connect().unwrap();
        let user_version: i64 = conn
            .pragma_query_value(None, "user_version", |row| row.get(0))
            .unwrap();
        let (weight, count, alpha, beta): (f64, i64, f64, f64) = conn
            .query_row(
                "SELECT weight, count, alpha, beta FROM cea_edges WHERE edge_id = 'legacy-edge'",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )
            .unwrap();
        let run_log_sql: String = conn
            .query_row(
                "SELECT sql FROM sqlite_master WHERE type = 'table' AND name = 'cea_run_log'",
                [],
                |row| row.get(0),
            )
            .unwrap();

        assert_eq!(user_version, CURRENT_SCHEMA_VERSION);
        assert!(super::column_exists(&conn, "cea_edges", "alpha").unwrap());
        assert!(super::column_exists(&conn, "cea_edges", "beta").unwrap());
        assert_eq!((weight, count, alpha, beta), (3.5, 4, 4.0, 1.0));
        assert!(
            !run_log_sql.contains("CHECK(eval_id != 'blocked')"),
            "legacy run-log constraint should be removed during migration"
        );
        drop(conn);

        let outcome =
            cea_store::update_graph(&store, &sample_result("blocked"), "blocked", "v1", 0.95)
                .unwrap();
        assert_eq!(
            outcome,
            cea_store::UpdateResult::Applied {
                edges_added: 1,
                edges_updated: 0,
            }
        );
    }

    fn sample_result(seed: &str) -> AttributedRunResult {
        sample_result_with(seed, seed)
    }

    fn sample_result_with(cause_seed: &str, effect_seed: &str) -> AttributedRunResult {
        AttributedRunResult::new(
            vec![AttributionTriple {
                cause: EditOpSignature {
                    op_kind: EditOpKind::Insert,
                    anchor_kind: AnchorKind::AfterLine,
                    lines_added: 1,
                    lines_removed: 0,
                    context_hash: format!("{cause_seed:0>8}"),
                    file_extension: "rs".to_string(),
                    scope_tag: ScopeTag::Function,
                    op_index: OpIndex(0),
                    file_index: FileIndex(0),
                },
                effect: EffectSignature {
                    check_kind: "clippy".to_string(),
                    outcome: "warning".to_string(),
                    severity: "warning".to_string(),
                    message_class: format!("unused_{effect_seed}"),
                    line_offset_from_edit: Some(1),
                },
                distance: 1,
                weight: 1.0,
            }],
            CheckResult {
                fmt_pass: true,
                clippy_pass: false,
                test_pass: true,
                fmt_output: ParsedCheckOutput::default(),
                clippy_output: ParsedCheckOutput::default(),
                test_output: ParsedCheckOutput::default(),
                total_duration_ms: 1,
            },
        )
    }

    #[test]
    fn update_graph_persists_nodes_edges_and_run_log() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("cea.db");
        create_current_schema(&path);
        let store = SqliteCeaStore::open(&path).unwrap();
        let result = sample_result("ok");

        let outcome = cea_store::update_graph(&store, &result, "eval-ok", "v1", 0.95).unwrap();
        assert_eq!(
            outcome,
            cea_store::UpdateResult::Applied {
                edges_added: 1,
                edges_updated: 0,
            }
        );

        let conn = rusqlite::Connection::open(&path).unwrap();
        let nodes: i64 = conn
            .query_row("SELECT COUNT(*) FROM cea_nodes", [], |row| row.get(0))
            .unwrap();
        let edges: i64 = conn
            .query_row("SELECT COUNT(*) FROM cea_edges", [], |row| row.get(0))
            .unwrap();
        let runs: i64 = conn
            .query_row("SELECT COUNT(*) FROM cea_run_log", [], |row| row.get(0))
            .unwrap();
        assert_eq!((nodes, edges, runs), (2, 1, 1));
    }

    #[test]
    fn update_graph_rolls_back_when_run_log_insert_fails() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("cea.db");
        create_current_schema(&path);
        let store = SqliteCeaStore::open(&path).unwrap();
        install_run_log_failure_trigger(&path);
        let result = sample_result("rollback");

        let err = cea_store::update_graph(&store, &result, "blocked", "v1", 0.95).unwrap_err();
        assert!(matches!(err, cea_store::CeaStoreError::Backend(_)));

        let conn = rusqlite::Connection::open(&path).unwrap();
        let nodes: i64 = conn
            .query_row("SELECT COUNT(*) FROM cea_nodes", [], |row| row.get(0))
            .unwrap();
        let edges: i64 = conn
            .query_row("SELECT COUNT(*) FROM cea_edges", [], |row| row.get(0))
            .unwrap();
        let runs: i64 = conn
            .query_row("SELECT COUNT(*) FROM cea_run_log", [], |row| row.get(0))
            .unwrap();
        assert_eq!((nodes, edges, runs), (0, 0, 0));
    }

    #[test]
    fn load_edges_filters_by_version() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("cea.db");
        create_current_schema(&path);
        let store = SqliteCeaStore::open(&path).unwrap();

        cea_store::update_graph(&store, &sample_result("one"), "eval-1", "v1", 0.95).unwrap();
        cea_store::update_graph(&store, &sample_result("two"), "eval-2", "v2", 0.95).unwrap();

        let all = cea_store::CeaStore::load_edges(&store, None).unwrap();
        let filtered = cea_store::CeaStore::load_edges(&store, Some("v1")).unwrap();
        assert_eq!(all.len(), 2);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].version_id, "v1");
    }

    #[test]
    fn update_graph_persists_negative_evidence_for_absent_effects() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("cea.db");
        create_current_schema(&path);
        let store = SqliteCeaStore::open(&path).unwrap();

        cea_store::update_graph(
            &store,
            &sample_result_with("shared", "one"),
            "eval-1",
            "v1",
            0.95,
        )
        .unwrap();
        let second = cea_store::update_graph(
            &store,
            &sample_result_with("shared", "two"),
            "eval-2",
            "v1",
            0.95,
        )
        .unwrap();
        assert_eq!(
            second,
            cea_store::UpdateResult::Applied {
                edges_added: 1,
                edges_updated: 1,
            }
        );

        let edges = cea_store::CeaStore::load_edges(&store, Some("v1")).unwrap();
        let cause_id = cea_core::edit_op_node_id(&EditOpSignature {
            op_kind: EditOpKind::Insert,
            anchor_kind: AnchorKind::AfterLine,
            lines_added: 1,
            lines_removed: 0,
            context_hash: format!("{:0>8}", "shared"),
            file_extension: "rs".to_string(),
            scope_tag: ScopeTag::Function,
            op_index: OpIndex(0),
            file_index: FileIndex(0),
        });
        let first_effect_id = cea_core::effect_node_id(&EffectSignature {
            check_kind: "clippy".to_string(),
            outcome: "warning".to_string(),
            severity: "warning".to_string(),
            message_class: "unused_one".to_string(),
            line_offset_from_edit: Some(1),
        });
        let second_effect_id = cea_core::effect_node_id(&EffectSignature {
            check_kind: "clippy".to_string(),
            outcome: "warning".to_string(),
            severity: "warning".to_string(),
            message_class: "unused_two".to_string(),
            line_offset_from_edit: Some(1),
        });
        let first_edge = edges
            .iter()
            .find(|edge| edge.cause_node_id == cause_id && edge.effect_node_id == first_effect_id)
            .unwrap();
        let second_edge = edges
            .iter()
            .find(|edge| edge.cause_node_id == cause_id && edge.effect_node_id == second_effect_id)
            .unwrap();

        assert_eq!(first_edge.beta, 2.0);
        assert_eq!(first_edge.count, 2);
        assert_eq!(second_edge.beta, 1.0);
        assert_eq!(second_edge.count, 1);
    }

    #[test]
    fn load_graph_round_trips_persisted_edge_stats() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("cea.db");
        create_current_schema(&path);

        let cause = cea_core::EditOpSignature {
            op_kind: EditOpKind::Insert,
            anchor_kind: AnchorKind::AfterLine,
            lines_added: 1,
            lines_removed: 0,
            context_hash: "0000stats".to_string(),
            file_extension: "rs".to_string(),
            scope_tag: ScopeTag::Function,
            op_index: OpIndex(0),
            file_index: FileIndex(0),
        };
        let effect = EffectSignature {
            check_kind: "clippy".to_string(),
            outcome: "warning".to_string(),
            severity: "warning".to_string(),
            message_class: "unused_stats".to_string(),
            line_offset_from_edit: Some(1),
        };
        let cause_id = cea_core::edit_op_node_id(&cause);
        let effect_id = cea_core::effect_node_id(&effect);
        let stats = cea_core::EdgeStats {
            alpha: 4.0,
            beta: 2.0,
            observations: 3,
        };

        let conn = rusqlite::Connection::open(&path).unwrap();
        conn.execute(
            "INSERT INTO cea_nodes (node_id, node_kind, sig_json, first_seen, last_seen) VALUES (?1, 'cause', ?2, 'now', 'now')",
            rusqlite::params![cause_id, serde_json::to_string(&cause).unwrap()],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO cea_nodes (node_id, node_kind, sig_json, first_seen, last_seen) VALUES (?1, 'effect', ?2, 'now', 'now')",
            rusqlite::params![effect_id, serde_json::to_string(&effect).unwrap()],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO cea_edges (edge_id, cause_node_id, effect_node_id, weight, count, confidence, alpha, beta, version_id, last_seen) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 'v1', 'now')",
            rusqlite::params![
                format!("{cause_id}_{effect_id}_v1"),
                cause_id,
                effect_id,
                3.0_f64,
                stats.observations as i64,
                0.0_f64,
                stats.alpha,
                stats.beta,
            ],
        )
        .unwrap();

        let store = SqliteCeaStore::open(&path).unwrap();
        let graph = cea_store::load_graph(&store, Some("v1")).unwrap();
        let cause_index = graph.node_index_map[&cea_core::edit_op_node_id(&cause)];
        let effect_index = graph.node_index_map[&cea_core::effect_node_id(&effect)];
        let edge = graph
            .graph
            .edge_weight(graph.graph.find_edge(cause_index, effect_index).unwrap())
            .unwrap();
        assert_eq!(edge.stats, stats);
        assert!((edge.confidence - stats.confidence()).abs() < 1e-12);
    }

    #[test]
    fn conn_write_transactions_are_explicitly_unsupported() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("cea.db");
        create_current_schema(&path);

        let conn = rusqlite::Connection::open(&path).unwrap();
        let store = super::SqliteCeaStoreConn::new(&conn);
        let err = cea_store::CeaStore::with_write_tx(&store, |_| Ok(())).unwrap_err();
        assert!(
            err.to_string().contains("does not open write transactions"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn direct_store_connections_apply_safety_pragmas() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("cea.db");
        create_current_schema(&path);

        let store = SqliteCeaStore::open(&path).unwrap();
        let conn = store.connect().unwrap();

        let foreign_keys: i64 = conn
            .pragma_query_value(None, "foreign_keys", |row| row.get(0))
            .unwrap();
        let journal_mode: String = conn
            .pragma_query_value(None, "journal_mode", |row| row.get(0))
            .unwrap();
        let busy_timeout: i64 = conn
            .pragma_query_value(None, "busy_timeout", |row| row.get(0))
            .unwrap();

        assert_eq!(foreign_keys, 1, "foreign keys should be enabled");
        assert_eq!(journal_mode.to_lowercase(), "wal");
        assert_eq!(busy_timeout, 5_000);
    }
}
