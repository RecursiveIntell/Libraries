//! SQLite-backed activity store for persisted LLM monitoring data.

use crate::models::{ActivityFilter, ActivityStats, MonitoredEvent};
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, OptionalExtension};
use stack_observation::{ObservationEnvelope, ObservationKind, PrivacyPolicy};
use std::io::Write;
use std::path::Path;
use std::sync::{Arc, Mutex};
use thiserror::Error;

const STORE_SCHEMA_VERSION: i64 = 2;

#[derive(Error, Debug)]
pub enum StoreError {
    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("observation contract error: {0}")]
    Observation(#[from] stack_observation::ObservationError),
    #[error("unsupported activity store schema version {0}")]
    UnsupportedSchema(i64),
}

/// Structured filter for normalized observation queries.
#[derive(Debug, Clone, Default)]
pub struct ObservationFilter {
    pub producer_id: Option<String>,
    pub source_crate: Option<String>,
    pub kind: Option<ObservationKind>,
    pub after: Option<DateTime<Utc>>,
    pub before: Option<DateTime<Utc>>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

/// A thread-safe activity store backed by SQLite.
///
/// Uses WAL mode and a single reader-writer Mutex for simplicity.
/// For high-concurrency scenarios, consider replacing with async SQLite
/// (e.g., `rusqlite` with `tokio` thread pool or `sqlx`).
///
/// `Clone` is cheap: the inner `Connection` is behind an `Arc`, so clones
/// share the same database handle and lock.
#[derive(Clone)]
pub struct ActivityStore {
    conn: Arc<Mutex<Connection>>,
}

impl ActivityStore {
    /// Open (or create) the activity database at the given path.
    ///
    /// Creates the schema if it doesn't exist.
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, StoreError> {
        let conn = Connection::open(path)?;
        conn.execute_batch(
            "PRAGMA journal_mode = WAL;
             PRAGMA foreign_keys = ON;
             PRAGMA busy_timeout = 5000;
             PRAGMA synchronous = NORMAL;",
        )?;

        let store = Self {
            conn: Arc::new(Mutex::new(conn)),
        };
        store.initialize_schema()?;
        Ok(store)
    }

    /// Initialize the database schema (idempotent).
    fn initialize_schema(&self) -> Result<(), StoreError> {
        let conn = self.conn.lock().unwrap();
        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS activity (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp TEXT NOT NULL,
                crate_name TEXT NOT NULL,
                event_type TEXT NOT NULL,
                summary TEXT NOT NULL,
                detail TEXT,
                trace_ctx_json TEXT,
                attempt_id TEXT,
                trial_id TEXT,
                tags_json TEXT NOT NULL DEFAULT '[]',
                source_loc TEXT
            );

            CREATE INDEX IF NOT EXISTS idx_activity_timestamp
                ON activity(timestamp);

            CREATE INDEX IF NOT EXISTS idx_activity_crate
                ON activity(crate_name);

            CREATE INDEX IF NOT EXISTS idx_activity_event_type
                ON activity(event_type);

            CREATE INDEX IF NOT EXISTS idx_activity_trace_id
                ON activity(trace_ctx_json);

            CREATE TABLE IF NOT EXISTS store_metadata (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS schema_migrations (
                version INTEGER PRIMARY KEY,
                applied_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS observations (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                event_id TEXT NOT NULL UNIQUE,
                observed_at TEXT NOT NULL,
                ingested_at TEXT NOT NULL,
                producer_id TEXT NOT NULL,
                process_id INTEGER,
                source_crate TEXT NOT NULL,
                adapter_id TEXT NOT NULL,
                provenance TEXT NOT NULL,
                producer_sequence INTEGER NOT NULL,
                event_kind TEXT NOT NULL,
                lifecycle_status TEXT NOT NULL,
                privacy_tier TEXT NOT NULL,
                redaction_state TEXT NOT NULL,
                envelope_json TEXT NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_observations_observed_at
                ON observations(observed_at);
            CREATE INDEX IF NOT EXISTS idx_observations_producer_sequence
                ON observations(producer_id, producer_sequence);
            CREATE INDEX IF NOT EXISTS idx_observations_source_kind
                ON observations(source_crate, event_kind);
            "#,
        )?;
        let existing: Option<String> = conn
            .query_row(
                "SELECT value FROM store_metadata WHERE key = 'schema_version'",
                [],
                |row| row.get::<_, String>(0),
            )
            .optional()?;
        if let Some(version) = existing {
            let version = version
                .parse::<i64>()
                .map_err(|_| StoreError::UnsupportedSchema(-1))?;
            if version > STORE_SCHEMA_VERSION {
                return Err(StoreError::UnsupportedSchema(version));
            }
        } else {
            conn.execute(
                "INSERT INTO store_metadata(key, value) VALUES ('schema_version', ?1)",
                [STORE_SCHEMA_VERSION.to_string()],
            )?;
        }
        for version in 1..=STORE_SCHEMA_VERSION {
            conn.execute(
                "INSERT OR IGNORE INTO schema_migrations(version, applied_at) VALUES (?1, ?2)",
                params![version, Utc::now().to_rfc3339()],
            )?;
        }
        Ok(())
    }

    /// Record a single activity event.
    #[deprecated(
        note = "use MonitorClient and start_collector for non-blocking observation ingestion"
    )]
    pub fn record(&self, event: &MonitoredEvent) -> Result<i64, StoreError> {
        let conn = self.conn.lock().unwrap();
        let trace_ctx_json = event
            .trace_ctx
            .as_ref()
            .map(|tc| serde_json::to_string(tc).unwrap_or_default())
            .unwrap_or_default();
        let tags_json = serde_json::to_string(&event.tags).unwrap_or_default();
        let detail_json = event.detail.clone().unwrap_or_default();

        conn.execute(
            r#"
            INSERT INTO activity
                (timestamp, crate_name, event_type, summary, detail, trace_ctx_json, attempt_id, trial_id, tags_json, source_loc)
            VALUES
                (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
            "#,
            params![
                event.timestamp.format("%Y-%m-%dT%H:%M:%S%.fZ").to_string(),
                event.crate_name,
                event.event_type,
                event.summary,
                detail_json,
                trace_ctx_json,
                event.attempt_id,
                event.trial_id,
                tags_json,
                event.source_loc,
            ],
        )?;
        Ok(conn.last_insert_rowid())
    }

    /// Persist one validated observation envelope.
    ///
    /// Returns `true` when a new row was inserted and `false` for an idempotent
    /// duplicate event ID.
    pub fn record_observation(&self, event: &ObservationEnvelope) -> Result<bool, StoreError> {
        let mut event = event.clone();
        event.apply_privacy_policy(&PrivacyPolicy::default());
        event.validate()?;
        let ingested_at = Utc::now();
        let event = event.with_ingested_at(ingested_at);
        let envelope_json = serde_json::to_string(&event)?;
        let conn = self.conn.lock().unwrap();
        let inserted = conn.execute(
            r#"
            INSERT OR IGNORE INTO observations
                (event_id, observed_at, ingested_at, producer_id, process_id,
                 source_crate, adapter_id, provenance, producer_sequence,
                 event_kind, lifecycle_status, privacy_tier, redaction_state,
                 envelope_json)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)
            "#,
            params![
                event.event_id.to_string(),
                event.observed_at.to_rfc3339(),
                ingested_at.to_rfc3339(),
                event.producer_id,
                event.process_id,
                event.source_crate,
                event.adapter_id,
                serde_json::to_string(&event.provenance)?,
                event.producer_sequence,
                serde_json::to_string(&event.kind)?,
                serde_json::to_string(&event.status)?,
                serde_json::to_string(&event.privacy.tier)?,
                serde_json::to_string(&event.privacy.redaction)?,
                envelope_json,
            ],
        )?;
        Ok(inserted == 1)
    }

    /// Record multiple events in a batch (transactional).
    #[deprecated(
        note = "use MonitorClient and start_collector for non-blocking observation ingestion"
    )]
    pub fn record_batch(&self, events: &[MonitoredEvent]) -> Result<usize, StoreError> {
        let conn = self.conn.lock().unwrap();
        conn.execute("BEGIN TRANSACTION", [])?;

        for event in events {
            if let Err(error) = self.record_inner(&conn, event) {
                let _ = conn.execute("ROLLBACK", []);
                return Err(error);
            }
        }

        conn.execute("COMMIT", [])?;
        Ok(events.len())
    }

    fn record_inner(&self, conn: &Connection, event: &MonitoredEvent) -> Result<(), StoreError> {
        let trace_ctx_json = event
            .trace_ctx
            .as_ref()
            .map(|tc| serde_json::to_string(tc).unwrap_or_default())
            .unwrap_or_default();
        let tags_json = serde_json::to_string(&event.tags).unwrap_or_default();
        let detail_json = event.detail.clone().unwrap_or_default();

        conn.execute(
            r#"
            INSERT INTO activity
                (timestamp, crate_name, event_type, summary, detail, trace_ctx_json, attempt_id, trial_id, tags_json, source_loc)
            VALUES
                (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
            "#,
            params![
                event.timestamp.format("%Y-%m-%dT%H:%M:%S%.fZ").to_string(),
                event.crate_name,
                event.event_type,
                event.summary,
                detail_json,
                trace_ctx_json,
                event.attempt_id,
                event.trial_id,
                tags_json,
                event.source_loc,
            ],
        )?;
        Ok(())
    }

    /// Query recent events, newest first.
    pub fn get_recent(&self, limit: usize) -> Result<Vec<MonitoredEvent>, StoreError> {
        self.query(&ActivityFilter {
            limit: Some(limit),
            ..Default::default()
        })
    }

    /// Search/filter activity events.
    pub fn query(&self, filter: &ActivityFilter) -> Result<Vec<MonitoredEvent>, StoreError> {
        let conn = self.conn.lock().unwrap();
        let mut sql = String::from(
            "SELECT id, timestamp, crate_name, event_type, summary, detail, trace_ctx_json, attempt_id, trial_id, tags_json, source_loc
             FROM activity
             WHERE 1=1",
        );
        let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        if let Some(ref crate_name) = filter.crate_name {
            sql.push_str(" AND crate_name = ?");
            params_vec.push(Box::new(crate_name.clone()));
        }
        if let Some(ref event_type) = filter.event_type {
            sql.push_str(" AND event_type = ?");
            params_vec.push(Box::new(event_type.clone()));
        }
        if let Some(ref trace_id) = filter.trace_id {
            // Partial match on trace ID within the JSON
            sql.push_str(" AND trace_ctx_json LIKE ?");
            params_vec.push(Box::new(format!("%{}%", trace_id)));
        }
        if let Some(after) = filter.after {
            sql.push_str(" AND timestamp >= ?");
            params_vec.push(Box::new(after.format("%Y-%m-%dT%H:%M:%S%.fZ").to_string()));
        }
        if let Some(before) = filter.before {
            sql.push_str(" AND timestamp <= ?");
            params_vec.push(Box::new(before.format("%Y-%m-%dT%H:%M:%S%.fZ").to_string()));
        }
        if !filter.tags.is_empty() {
            // Events must have ALL specified tags
            for tag in &filter.tags {
                sql.push_str(" AND tags_json LIKE ?");
                params_vec.push(Box::new(format!("%\"{}\"%", tag)));
            }
        }

        sql.push_str(" ORDER BY timestamp DESC");

        if let Some(limit) = filter.limit {
            sql.push_str(&format!(" LIMIT {}", limit));
        }
        if let Some(offset) = filter.offset {
            sql.push_str(&format!(" OFFSET {}", offset));
        }

        let mut stmt = conn.prepare(&sql)?;
        let params_refs: Vec<&dyn rusqlite::ToSql> =
            params_vec.iter().map(|p| p.as_ref()).collect();
        let rows = stmt.query_map(rusqlite::params_from_iter(params_refs), |row| {
            let timestamp_str: String = row.get(1)?;
            let trace_ctx_json: String = row.get(6)?;
            let tags_json: String = row.get(9)?;

            let timestamp = DateTime::parse_from_str(&timestamp_str, "%Y-%m-%dT%H:%M:%S%.fZ")
                .map(|timestamp| timestamp.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now());

            let mut trace_ctx: Option<stack_ids::TraceCtx> = None;
            if !trace_ctx_json.is_empty() {
                trace_ctx = serde_json::from_str(&trace_ctx_json).ok();
            }

            let tags: Vec<String> = serde_json::from_str(&tags_json).unwrap_or_default();

            Ok(MonitoredEvent {
                id: row.get(0)?,
                timestamp,
                crate_name: row.get(2)?,
                event_type: row.get(3)?,
                summary: row.get(4)?,
                detail: {
                    let d: String = row.get(5)?;
                    if d.is_empty() {
                        None
                    } else {
                        Some(d)
                    }
                },
                trace_ctx,
                attempt_id: row.get(7)?,
                trial_id: row.get(8)?,
                tags,
                source_loc: row.get(10)?,
            })
        })?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row.map_err(StoreError::Sqlite)?);
        }
        Ok(results)
    }

    /// Get aggregated stats for a time window.
    pub fn stats(
        &self,
        window_start: DateTime<Utc>,
        window_end: DateTime<Utc>,
    ) -> Result<ActivityStats, StoreError> {
        let conn = self.conn.lock().unwrap();
        let start_str = window_start.format("%Y-%m-%dT%H:%M:%S%.fZ").to_string();
        let end_str = window_end.format("%Y-%m-%dT%H:%M:%S%.fZ").to_string();

        let total: u64 = conn.query_row(
            "SELECT COUNT(*) FROM activity WHERE timestamp >= ? AND timestamp <= ?",
            params![start_str, end_str],
            |row| row.get(0),
        )?;

        let llm_calls: u64 = conn.query_row(
            "SELECT COUNT(*) FROM activity WHERE timestamp >= ? AND timestamp <= ? AND event_type = 'llm_call'",
            params![start_str, end_str],
            |row| row.get(0),
        )?;

        let tool_invocations: u64 = conn.query_row(
            "SELECT COUNT(*) FROM activity WHERE timestamp >= ? AND timestamp <= ? AND event_type = 'tool_invocation'",
            params![start_str, end_str],
            |row| row.get(0),
        )?;

        // By type
        let mut by_type_stmt = conn.prepare(
            "SELECT event_type, COUNT(*) FROM activity WHERE timestamp >= ? AND timestamp <= ? GROUP BY event_type ORDER BY COUNT(*) DESC",
        )?;
        let by_type: Vec<(String, u64)> = by_type_stmt
            .query_map(params![start_str, end_str], |row| {
                Ok((row.get(0)?, row.get(1)?))
            })?
            .collect::<Result<Vec<_>, _>>()?;

        // By crate
        let mut by_crate_stmt = conn.prepare(
            "SELECT crate_name, COUNT(*) FROM activity WHERE timestamp >= ? AND timestamp <= ? GROUP BY crate_name ORDER BY COUNT(*) DESC",
        )?;
        let by_crate: Vec<(String, u64)> = by_crate_stmt
            .query_map(params![start_str, end_str], |row| {
                Ok((row.get(0)?, row.get(1)?))
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(ActivityStats {
            total_events: total,
            by_type,
            by_crate,
            llm_calls,
            tool_invocations,
            window_start,
            window_end,
        })
    }

    /// Get total event count.
    pub fn total_count(&self) -> Result<u64, StoreError> {
        let conn = self.conn.lock().unwrap();
        let count: u64 = conn.query_row("SELECT COUNT(*) FROM activity", [], |row| row.get(0))?;
        Ok(count)
    }

    /// Count normalized observation envelopes.
    pub fn observation_count(&self) -> Result<u64, StoreError> {
        let conn = self.conn.lock().unwrap();
        Ok(conn.query_row("SELECT COUNT(*) FROM observations", [], |row| row.get(0))?)
    }

    /// Count normalized observations from one producer sequence domain.
    pub fn observation_count_for_producer(&self, producer_id: &str) -> Result<u64, StoreError> {
        let conn = self.conn.lock().unwrap();
        Ok(conn.query_row(
            "SELECT COUNT(*) FROM observations WHERE producer_id = ?1",
            params![producer_id],
            |row| row.get(0),
        )?)
    }

    /// Count normalized observations with one stable event ID.
    pub fn observation_count_for_event_id(&self, event_id: &str) -> Result<u64, StoreError> {
        let conn = self.conn.lock().unwrap();
        Ok(conn.query_row(
            "SELECT COUNT(*) FROM observations WHERE event_id = ?1",
            params![event_id],
            |row| row.get(0),
        )?)
    }

    /// Query normalized observations using typed columns and envelope decoding.
    pub fn query_observations(
        &self,
        filter: &ObservationFilter,
    ) -> Result<Vec<ObservationEnvelope>, StoreError> {
        let conn = self.conn.lock().unwrap();
        let mut sql = String::from("SELECT envelope_json FROM observations WHERE 1=1");
        let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();
        if let Some(producer_id) = &filter.producer_id {
            sql.push_str(" AND producer_id = ?");
            params_vec.push(Box::new(producer_id.clone()));
        }
        if let Some(source_crate) = &filter.source_crate {
            sql.push_str(" AND source_crate = ?");
            params_vec.push(Box::new(source_crate.clone()));
        }
        if let Some(kind) = &filter.kind {
            sql.push_str(" AND event_kind = ?");
            params_vec.push(Box::new(serde_json::to_string(kind)?));
        }
        if let Some(after) = filter.after {
            sql.push_str(" AND observed_at >= ?");
            params_vec.push(Box::new(after.to_rfc3339()));
        }
        if let Some(before) = filter.before {
            sql.push_str(" AND observed_at <= ?");
            params_vec.push(Box::new(before.to_rfc3339()));
        }
        sql.push_str(" ORDER BY observed_at DESC, id DESC");
        if let Some(limit) = filter.limit {
            sql.push_str(&format!(" LIMIT {}", limit));
        }
        if let Some(offset) = filter.offset {
            sql.push_str(&format!(" OFFSET {}", offset));
        }

        let mut stmt = conn.prepare(&sql)?;
        let params_refs: Vec<&dyn rusqlite::ToSql> =
            params_vec.iter().map(|param| param.as_ref()).collect();
        let rows = stmt.query_map(rusqlite::params_from_iter(params_refs), |row| {
            let encoded: String = row.get(0)?;
            Ok(encoded)
        })?;
        let mut observations = Vec::new();
        for row in rows {
            observations.push(serde_json::from_str::<ObservationEnvelope>(&row?)?);
        }
        Ok(observations)
    }

    /// Delete events older than the given cutoff.
    pub fn prune_before(&self, cutoff: DateTime<Utc>) -> Result<usize, StoreError> {
        let conn = self.conn.lock().unwrap();
        let cutoff_str = cutoff.format("%Y-%m-%dT%H:%M:%S%.fZ").to_string();
        let deleted: usize = conn.execute(
            "DELETE FROM activity WHERE timestamp < ?",
            params![cutoff_str],
        )?;
        Ok(deleted)
    }

    /// Delete normalized observations older than the given cutoff.
    pub fn prune_observations_before(&self, cutoff: DateTime<Utc>) -> Result<usize, StoreError> {
        let conn = self.conn.lock().unwrap();
        let cutoff_str = cutoff.to_rfc3339();
        Ok(conn.execute(
            "DELETE FROM observations WHERE observed_at < ?",
            params![cutoff_str],
        )?)
    }

    /// Export all activity as JSON Lines (for external processing/backup).
    #[deprecated(note = "use export_observations_jsonl_to for privacy-sanitized normalized export")]
    pub fn export_jsonl(&self) -> Result<String, StoreError> {
        let events = self.query(&ActivityFilter::default())?;
        let mut output = String::new();
        for event in events {
            output.push_str(&serde_json::to_string(&event)?);
            output.push('\n');
        }
        Ok(output)
    }

    /// Export normalized observations incrementally as JSON Lines.
    pub fn export_observations_jsonl_to<W: Write>(
        &self,
        writer: &mut W,
    ) -> Result<u64, StoreError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT envelope_json FROM observations ORDER BY id ASC")?;
        let rows = stmt.query_map([], |row| row.get::<_, String>(0))?;
        let mut count = 0;
        for row in rows {
            writeln!(writer, "{}", row?)?;
            count += 1;
        }
        Ok(count)
    }

    /// Clear all data (for testing or reset).
    pub fn clear(&self) -> Result<(), StoreError> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM activity", [])?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;
    use stack_observation::{LifecycleStatus, ObservationKind};

    fn observation() -> ObservationEnvelope {
        ObservationEnvelope::metadata(
            "store-test",
            "llm-pipeline",
            "store-adapter",
            1,
            ObservationKind::LlmCall,
            LifecycleStatus::Completed,
            "stored",
        )
    }

    #[test]
    fn collector_storage_sanitizes_content_before_export() {
        let store = ActivityStore::open(":memory:").unwrap();
        let mut event = observation();
        event.payload = serde_json::json!({
            "prompt": "private prompt",
            "authorization": "Bearer sk-secret"
        });
        store.record_observation(&event).unwrap();
        let mut output = Vec::new();
        assert_eq!(store.export_observations_jsonl_to(&mut output).unwrap(), 1);
        let encoded = String::from_utf8(output).unwrap();
        assert!(!encoded.contains("private prompt"));
        assert!(!encoded.contains("sk-secret"));
        assert!(encoded.contains("content disabled"));
    }

    #[test]
    fn normalized_query_uses_typed_filters_and_decodes_envelopes() {
        let store = ActivityStore::open(":memory:").unwrap();
        let mut first = observation();
        first.producer_id = "producer-a".into();
        let mut second = observation();
        second.producer_id = "producer-b".into();
        store.record_observation(&first).unwrap();
        store.record_observation(&second).unwrap();
        let results = store
            .query_observations(&ObservationFilter {
                producer_id: Some("producer-a".into()),
                kind: Some(ObservationKind::LlmCall),
                limit: Some(10),
                ..Default::default()
            })
            .unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].producer_id, "producer-a");
    }

    #[test]
    fn future_schema_version_is_rejected() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("activity.db");
        let store = ActivityStore::open(&path).unwrap();
        drop(store);
        let connection = Connection::open(&path).unwrap();
        connection
            .execute(
                "UPDATE store_metadata SET value = '999' WHERE key = 'schema_version'",
                [],
            )
            .unwrap();
        let result = ActivityStore::open(&path);
        assert!(matches!(result, Err(StoreError::UnsupportedSchema(999))));
    }

    #[test]
    fn normalized_retention_prunes_old_observations() {
        let store = ActivityStore::open(":memory:").unwrap();
        let mut event = observation();
        event.observed_at = Utc::now() - Duration::days(10);
        store.record_observation(&event).unwrap();
        let deleted = store
            .prune_observations_before(Utc::now() - Duration::days(1))
            .unwrap();
        assert_eq!(deleted, 1);
        assert_eq!(store.observation_count().unwrap(), 0);
    }
}
