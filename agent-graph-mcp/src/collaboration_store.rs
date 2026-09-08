//! Daemon-owned append-only collaboration facts and rebuildable task projections.

use agent_collaboration_contract::{ArtifactRefV1, TaskEnvelopeV1, TaskEventV1, TaskStatusV1};
use boundary_compiler::Canonicalizer;
use rusqlite::{params, Connection, OptionalExtension};
use serde::Serialize;
use stack_ids::ContentDigest;
use std::sync::{Arc, Mutex};
use thiserror::Error;

pub const COLLABORATION_SCHEMA_VERSION: i64 = 1;

#[derive(Debug, Error)]
pub enum CollaborationStoreError {
    #[error("database error: {0}")]
    Database(String),
    #[error("serialization error: {0}")]
    Serialization(String),
    #[error("contract validation error: {0}")]
    Contract(String),
    #[error("task not found: {0}")]
    TaskNotFound(String),
    #[error("idempotency conflict for key {key}")]
    IdempotencyConflict { key: String },
    #[error("event conflict: {0}")]
    EventConflict(String),
    #[error("artifact conflict: {0}")]
    ArtifactConflict(String),
    #[error("owner mismatch: expected {expected}, got {actual}")]
    OwnerMismatch { expected: String, actual: String },
    #[error("invalid transition: {from:?} -> {to:?}")]
    InvalidTransition {
        from: TaskStatusV1,
        to: TaskStatusV1,
    },
    #[error("projection integrity failure: {0}")]
    ProjectionIntegrity(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskProjection {
    pub task_id: String,
    pub owner_agent_id: String,
    pub status: TaskStatusV1,
    pub sequence: u64,
    pub last_event_digest: Option<String>,
}

#[derive(Clone)]
pub struct CollaborationStore {
    conn: Arc<Mutex<Connection>>,
}

pub(crate) fn migrate(conn: &Connection) -> Result<(), String> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS collaboration_tasks (
            task_id TEXT PRIMARY KEY,
            owner_agent_id TEXT NOT NULL,
            request_digest TEXT NOT NULL,
            status TEXT NOT NULL,
            envelope_json TEXT NOT NULL,
            last_sequence INTEGER NOT NULL DEFAULT 0,
            last_event_digest TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS collaboration_task_events (
            task_id TEXT NOT NULL,
            sequence INTEGER NOT NULL,
            event_id TEXT NOT NULL UNIQUE,
            event_digest TEXT NOT NULL,
            previous_event_digest TEXT,
            status TEXT NOT NULL,
            event_json TEXT NOT NULL,
            recorded_at TEXT NOT NULL,
            PRIMARY KEY (task_id, sequence),
            FOREIGN KEY (task_id) REFERENCES collaboration_tasks(task_id)
        );
        CREATE TABLE IF NOT EXISTS collaboration_idempotency (
            idempotency_key TEXT PRIMARY KEY,
            request_digest TEXT NOT NULL,
            task_id TEXT NOT NULL,
            created_at TEXT NOT NULL,
            FOREIGN KEY (task_id) REFERENCES collaboration_tasks(task_id)
        );
        CREATE TABLE IF NOT EXISTS collaboration_conflicts (
            conflict_id INTEGER PRIMARY KEY AUTOINCREMENT,
            task_id TEXT,
            event_id TEXT,
            reason TEXT NOT NULL,
            evidence_json TEXT NOT NULL,
            recorded_at TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS collaboration_attempts (
            task_id TEXT NOT NULL,
            attempt_id TEXT PRIMARY KEY,
            trial_id TEXT NOT NULL,
            worker_agent_id TEXT NOT NULL,
            started_at TEXT NOT NULL,
            finished_at TEXT,
            FOREIGN KEY (task_id) REFERENCES collaboration_tasks(task_id)
        );
        CREATE TABLE IF NOT EXISTS collaboration_leases (
            task_id TEXT NOT NULL,
            lease_id TEXT PRIMARY KEY,
            lease_epoch INTEGER NOT NULL,
            fencing_token TEXT NOT NULL,
            holder_agent_id TEXT NOT NULL,
            expires_at TEXT NOT NULL,
            recorded_at TEXT NOT NULL,
            FOREIGN KEY (task_id) REFERENCES collaboration_tasks(task_id)
        );
        CREATE TABLE IF NOT EXISTS collaboration_artifacts (
            artifact_id TEXT PRIMARY KEY,
            task_id TEXT NOT NULL,
            producer_agent_id TEXT NOT NULL,
            digest TEXT NOT NULL,
            size_bytes INTEGER NOT NULL,
            media_type TEXT NOT NULL,
            artifact_ref_json TEXT NOT NULL,
            created_at TEXT NOT NULL,
            FOREIGN KEY (task_id) REFERENCES collaboration_tasks(task_id)
        );
        CREATE TABLE IF NOT EXISTS collaboration_deliveries (
            delivery_id TEXT PRIMARY KEY,
            task_id TEXT NOT NULL,
            event_id TEXT NOT NULL,
            recipient_agent_id TEXT NOT NULL,
            status TEXT NOT NULL,
            observed_at TEXT NOT NULL,
            FOREIGN KEY (task_id) REFERENCES collaboration_tasks(task_id)
        );
        CREATE INDEX IF NOT EXISTS collaboration_task_events_task_idx
            ON collaboration_task_events(task_id, sequence);
        CREATE INDEX IF NOT EXISTS collaboration_artifacts_task_idx
            ON collaboration_artifacts(task_id);
        CREATE TABLE IF NOT EXISTS collaboration_schema_migrations (
            version INTEGER PRIMARY KEY,
            migration_digest TEXT NOT NULL,
            applied_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        );
        INSERT OR IGNORE INTO collaboration_schema_migrations(version, migration_digest)
            VALUES (1, 'agent-graph-mcp:collaboration:v1');",
    )
    .map_err(|error| error.to_string())
}

impl CollaborationStore {
    pub(crate) fn from_connection(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }

    pub fn append_task(
        &self,
        envelope: &TaskEnvelopeV1,
    ) -> Result<TaskProjection, CollaborationStoreError> {
        envelope
            .validate()
            .map_err(|error| CollaborationStoreError::Contract(error.to_string()))?;
        let task_id = envelope.task_id.to_string();
        let owner = envelope.owner_agent_id.to_string();
        let request_digest = envelope.request_digest.to_string();
        let envelope_json = serde_json::to_string(envelope)
            .map_err(|error| CollaborationStoreError::Serialization(error.to_string()))?;
        let now = chrono::Utc::now().to_rfc3339();
        let conn = self
            .conn
            .lock()
            .map_err(|error| CollaborationStoreError::Database(error.to_string()))?;
        let tx = conn
            .unchecked_transaction()
            .map_err(|error| CollaborationStoreError::Database(error.to_string()))?;
        if let Some((existing_digest, existing_task)) = tx
            .query_row(
                "SELECT request_digest, task_id FROM collaboration_idempotency
                 WHERE idempotency_key = ?1",
                params![envelope.idempotency_key],
                |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
            )
            .optional()
            .map_err(|error| CollaborationStoreError::Database(error.to_string()))?
        {
            if existing_digest != request_digest {
                return Err(CollaborationStoreError::IdempotencyConflict {
                    key: envelope.idempotency_key.clone(),
                });
            }
            return projection_from_row(&tx, &existing_task);
        }
        tx.execute(
            "INSERT INTO collaboration_tasks
             (task_id, owner_agent_id, request_digest, status, envelope_json,
              last_sequence, last_event_digest, created_at, updated_at)
             VALUES (?1, ?2, ?3, 'submitted', ?4, 0, NULL, ?5, ?5)",
            params![task_id, owner, request_digest, envelope_json, now],
        )
        .map_err(|error| CollaborationStoreError::Database(error.to_string()))?;
        tx.execute(
            "INSERT INTO collaboration_idempotency
             (idempotency_key, request_digest, task_id, created_at)
             VALUES (?1, ?2, ?3, ?4)",
            params![envelope.idempotency_key, request_digest, task_id, now],
        )
        .map_err(|error| CollaborationStoreError::Database(error.to_string()))?;
        tx.commit()
            .map_err(|error| CollaborationStoreError::Database(error.to_string()))?;
        Ok(TaskProjection {
            task_id,
            owner_agent_id: owner,
            status: TaskStatusV1::Submitted,
            sequence: 0,
            last_event_digest: None,
        })
    }

    pub fn record_artifact(
        &self,
        task_id: &str,
        producer_agent_id: &str,
        reference: &ArtifactRefV1,
    ) -> Result<(), CollaborationStoreError> {
        reference
            .validate()
            .map_err(|error| CollaborationStoreError::Contract(error.to_string()))?;
        let artifact_id = reference.artifact_id.to_string();
        let artifact_json = serde_json::to_string(reference)
            .map_err(|error| CollaborationStoreError::Serialization(error.to_string()))?;
        let conn = self
            .conn
            .lock()
            .map_err(|error| CollaborationStoreError::Database(error.to_string()))?;
        let existing: Option<(String, u64, String)> = conn
            .query_row(
                "SELECT digest, size_bytes, media_type FROM collaboration_artifacts
                 WHERE artifact_id = ?1",
                params![artifact_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .optional()
            .map_err(|error| CollaborationStoreError::Database(error.to_string()))?;
        if let Some((digest, size_bytes, media_type)) = existing {
            if digest == reference.digest.to_string()
                && size_bytes == reference.size_bytes
                && media_type == reference.media_type
            {
                return Ok(());
            }
            return Err(CollaborationStoreError::ArtifactConflict(
                "artifact ID carries different immutable metadata".into(),
            ));
        }
        conn.execute(
            "INSERT INTO collaboration_artifacts
             (artifact_id, task_id, producer_agent_id, digest, size_bytes, media_type,
              artifact_ref_json, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                artifact_id,
                task_id,
                producer_agent_id,
                reference.digest.to_string(),
                reference.size_bytes,
                reference.media_type,
                artifact_json,
                chrono::Utc::now().to_rfc3339()
            ],
        )
        .map_err(|error| CollaborationStoreError::Database(error.to_string()))?;
        Ok(())
    }

    pub fn append_event(
        &self,
        event: &TaskEventV1,
    ) -> Result<TaskProjection, CollaborationStoreError> {
        event
            .validate()
            .map_err(|error| CollaborationStoreError::Contract(error.to_string()))?;
        let task_id = event.task_id.to_string();
        let event_digest = digest_json(event)?;
        let event_json = serde_json::to_string(event)
            .map_err(|error| CollaborationStoreError::Serialization(error.to_string()))?;
        let conn = self
            .conn
            .lock()
            .map_err(|error| CollaborationStoreError::Database(error.to_string()))?;
        let tx = conn
            .unchecked_transaction()
            .map_err(|error| CollaborationStoreError::Database(error.to_string()))?;
        let (owner, status_text, sequence, previous_digest): (String, String, i64, Option<String>) =
            tx.query_row(
                "SELECT owner_agent_id, status, last_sequence, last_event_digest
                 FROM collaboration_tasks WHERE task_id = ?1",
                params![task_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )
            .map_err(|_| CollaborationStoreError::TaskNotFound(task_id.clone()))?;
        let actual_owner = event.owner_agent_id.to_string();
        if owner != actual_owner {
            return Err(CollaborationStoreError::OwnerMismatch {
                expected: owner,
                actual: actual_owner,
            });
        }
        if let Some((existing_task, existing_digest)) = tx
            .query_row(
                "SELECT task_id, event_digest FROM collaboration_task_events WHERE event_id = ?1",
                params![event.event_id.to_string()],
                |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
            )
            .optional()
            .map_err(|error| CollaborationStoreError::Database(error.to_string()))?
        {
            if existing_task == task_id && existing_digest == event_digest {
                return projection_from_row(&tx, &task_id);
            }
            return Err(CollaborationStoreError::EventConflict(
                "duplicate event ID carries different material".into(),
            ));
        }
        let current_status = parse_status(&status_text)?;
        if !current_status.can_transition_to(event.status) {
            return Err(CollaborationStoreError::InvalidTransition {
                from: current_status,
                to: event.status,
            });
        }
        let expected_previous = previous_digest.as_deref();
        let supplied_previous = event
            .previous_event_digest
            .as_ref()
            .map(ToString::to_string);
        if expected_previous != supplied_previous.as_deref() {
            return Err(CollaborationStoreError::EventConflict(
                "previous event digest does not match projection".into(),
            ));
        }
        let next_sequence = sequence + 1;
        let now = chrono::Utc::now().to_rfc3339();
        tx.execute(
            "INSERT INTO collaboration_task_events
             (task_id, sequence, event_id, event_digest, previous_event_digest,
              status, event_json, recorded_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                task_id,
                next_sequence,
                event.event_id.to_string(),
                event_digest,
                supplied_previous,
                serde_json::to_string(&event.status).unwrap_or_else(|_| "null".into()),
                event_json,
                now,
            ],
        )
        .map_err(|error| CollaborationStoreError::Database(error.to_string()))?;
        tx.execute(
            "UPDATE collaboration_tasks
             SET status = ?2, last_sequence = ?3, last_event_digest = ?4, updated_at = ?5
             WHERE task_id = ?1",
            params![
                task_id,
                serde_json::to_string(&event.status)
                    .unwrap_or_else(|_| "\"submitted\"".into())
                    .trim_matches('"'),
                next_sequence,
                event_digest,
                now
            ],
        )
        .map_err(|error| CollaborationStoreError::Database(error.to_string()))?;
        tx.commit()
            .map_err(|error| CollaborationStoreError::Database(error.to_string()))?;
        drop(conn);
        self.projection(&task_id)
    }

    pub fn projection(&self, task_id: &str) -> Result<TaskProjection, CollaborationStoreError> {
        let conn = self
            .conn
            .lock()
            .map_err(|error| CollaborationStoreError::Database(error.to_string()))?;
        projection_from_row(&conn, task_id)
    }

    pub fn events(&self, task_id: &str) -> Result<Vec<serde_json::Value>, CollaborationStoreError> {
        let conn = self
            .conn
            .lock()
            .map_err(|error| CollaborationStoreError::Database(error.to_string()))?;
        let mut statement = conn
            .prepare(
                "SELECT event_json FROM collaboration_task_events
                 WHERE task_id = ?1 ORDER BY sequence",
            )
            .map_err(|error| CollaborationStoreError::Database(error.to_string()))?;
        let rows = statement
            .query_map(params![task_id], |row| row.get::<_, String>(0))
            .map_err(|error| CollaborationStoreError::Database(error.to_string()))?;
        let mut events = Vec::new();
        for row in rows {
            let json = row.map_err(|error| CollaborationStoreError::Database(error.to_string()))?;
            events.push(
                serde_json::from_str(&json)
                    .map_err(|error| CollaborationStoreError::Serialization(error.to_string()))?,
            );
        }
        Ok(events)
    }

    pub fn rebuild_projection(
        &self,
        task_id: &str,
    ) -> Result<TaskProjection, CollaborationStoreError> {
        let conn = self
            .conn
            .lock()
            .map_err(|error| CollaborationStoreError::Database(error.to_string()))?;
        let owner: String = conn
            .query_row(
                "SELECT owner_agent_id FROM collaboration_tasks WHERE task_id = ?1",
                params![task_id],
                |row| row.get(0),
            )
            .map_err(|_| CollaborationStoreError::TaskNotFound(task_id.into()))?;
        let mut status = TaskStatusV1::Submitted;
        let mut sequence = 0_u64;
        let mut previous_digest: Option<String> = None;
        let mut stmt = conn
            .prepare(
                "SELECT event_json, event_digest, previous_event_digest
                 FROM collaboration_task_events WHERE task_id = ?1 ORDER BY sequence",
            )
            .map_err(|error| CollaborationStoreError::Database(error.to_string()))?;
        let rows = stmt
            .query_map(params![task_id], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, Option<String>>(2)?,
                ))
            })
            .map_err(|error| CollaborationStoreError::Database(error.to_string()))?;
        for row in rows {
            let (json, stored_digest, supplied_previous) =
                row.map_err(|error| CollaborationStoreError::Database(error.to_string()))?;
            let event: TaskEventV1 = serde_json::from_str(&json)
                .map_err(|error| CollaborationStoreError::Serialization(error.to_string()))?;
            if event.task_id.to_string() != task_id || event.owner_agent_id.to_string() != owner {
                return Err(CollaborationStoreError::ProjectionIntegrity(
                    "event identity or owner mismatch".into(),
                ));
            }
            if supplied_previous != previous_digest || !status.can_transition_to(event.status) {
                return Err(CollaborationStoreError::ProjectionIntegrity(
                    "event chain or transition mismatch".into(),
                ));
            }
            if digest_json(&event)? != stored_digest {
                return Err(CollaborationStoreError::ProjectionIntegrity(
                    "event digest mismatch".into(),
                ));
            }
            status = event.status;
            previous_digest = Some(stored_digest);
            sequence += 1;
        }
        conn.execute(
            "UPDATE collaboration_tasks SET status = ?2, last_sequence = ?3,
             last_event_digest = ?4, updated_at = ?5 WHERE task_id = ?1",
            params![
                task_id,
                status_string(status),
                sequence,
                previous_digest,
                chrono::Utc::now().to_rfc3339()
            ],
        )
        .map_err(|error| CollaborationStoreError::Database(error.to_string()))?;
        Ok(TaskProjection {
            task_id: task_id.into(),
            owner_agent_id: owner,
            status,
            sequence,
            last_event_digest: previous_digest,
        })
    }
}

fn projection_from_row(
    conn: &Connection,
    task_id: &str,
) -> Result<TaskProjection, CollaborationStoreError> {
    let (owner, status, sequence, digest) = conn
        .query_row(
            "SELECT owner_agent_id, status, last_sequence, last_event_digest
             FROM collaboration_tasks WHERE task_id = ?1",
            params![task_id],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, i64>(2)?,
                    row.get(3)?,
                ))
            },
        )
        .map_err(|_| CollaborationStoreError::TaskNotFound(task_id.into()))?;
    Ok(TaskProjection {
        task_id: task_id.into(),
        owner_agent_id: owner,
        status: parse_status(&status)?,
        sequence: u64::try_from(sequence).map_err(|_| {
            CollaborationStoreError::ProjectionIntegrity("negative sequence".into())
        })?,
        last_event_digest: digest,
    })
}

fn parse_status(value: &str) -> Result<TaskStatusV1, CollaborationStoreError> {
    serde_json::from_str(&format!("\"{value}\""))
        .map_err(|error| CollaborationStoreError::Serialization(error.to_string()))
}

fn status_string(status: TaskStatusV1) -> String {
    serde_json::to_string(&status)
        .unwrap_or_else(|_| "\"submitted\"".into())
        .trim_matches('"')
        .to_owned()
}

fn digest_json<T: Serialize>(value: &T) -> Result<String, CollaborationStoreError> {
    let value = serde_json::to_value(value)
        .map_err(|error| CollaborationStoreError::Serialization(error.to_string()))?;
    let bytes = Canonicalizer::new()
        .canonicalize_bytes(&value)
        .map_err(|error| CollaborationStoreError::Serialization(error.to_string()))?;
    Ok(ContentDigest::compute(&bytes).to_string())
}
