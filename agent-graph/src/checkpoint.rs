use crate::error::{AgentGraphError, Result};
use crate::state::StateSnapshot;
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Checkpoint {
    pub execution_id: String,
    pub timestamp: DateTime<Utc>,
    pub current_node: String,
    pub iteration: usize,
    pub state: StateSnapshot,
    #[serde(default)]
    pub step_number: usize,
    #[serde(default)]
    pub active_nodes: Vec<String>,
}

pub struct CheckpointManager {
    conn: Connection,
}

impl CheckpointManager {
    /// Create a new checkpoint manager
    pub fn new(db_path: &str) -> Result<Self> {
        let conn = Connection::open(db_path)?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS checkpoints (
                execution_id TEXT NOT NULL,
                timestamp TEXT NOT NULL,
                current_node TEXT NOT NULL,
                iteration INTEGER NOT NULL,
                state_data TEXT NOT NULL,
                step_number INTEGER NOT NULL DEFAULT 0,
                active_nodes TEXT NOT NULL DEFAULT '[]',
                PRIMARY KEY (execution_id, timestamp)
            )",
            [],
        )?;
        Self::ensure_scheduler_frontier_columns(&conn)?;

        Ok(Self { conn })
    }

    /// Upgrade legacy SQLite checkpoint tables without rewriting existing rows.
    fn ensure_scheduler_frontier_columns(conn: &Connection) -> Result<()> {
        let mut statement = conn.prepare("PRAGMA table_info(checkpoints)")?;
        let columns = statement
            .query_map([], |row| row.get::<_, String>(1))?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        if !columns.iter().any(|column| column == "step_number") {
            conn.execute(
                "ALTER TABLE checkpoints ADD COLUMN step_number INTEGER NOT NULL DEFAULT 0",
                [],
            )?;
        }
        if !columns.iter().any(|column| column == "active_nodes") {
            conn.execute(
                "ALTER TABLE checkpoints ADD COLUMN active_nodes TEXT NOT NULL DEFAULT '[]'",
                [],
            )?;
        }
        Ok(())
    }

    /// Save a checkpoint
    pub fn save(&self, checkpoint: &Checkpoint) -> Result<()> {
        let state_json = serde_json::to_string(&checkpoint.state)?;
        let active_nodes_json = serde_json::to_string(&checkpoint.active_nodes)?;
        let iteration = i64::try_from(checkpoint.iteration).map_err(|_| {
            AgentGraphError::CheckpointError("checkpoint iteration exceeds SQLite INTEGER".into())
        })?;
        let step_number = i64::try_from(checkpoint.step_number).map_err(|_| {
            AgentGraphError::CheckpointError("checkpoint step number exceeds SQLite INTEGER".into())
        })?;

        self.conn.execute(
            "INSERT OR REPLACE INTO checkpoints (execution_id, timestamp, current_node, iteration, state_data, step_number, active_nodes)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                &checkpoint.execution_id,
                checkpoint.timestamp.to_rfc3339(),
                &checkpoint.current_node,
                iteration,
                &state_json,
                step_number,
                &active_nodes_json,
            ],
        )?;

        Ok(())
    }

    /// Load the most recent checkpoint for an execution
    pub fn load(&self, execution_id: &str) -> Result<Option<Checkpoint>> {
        let mut stmt = self.conn.prepare(
            "SELECT timestamp, current_node, iteration, state_data, step_number, active_nodes
             FROM checkpoints
             WHERE execution_id = ?1
             ORDER BY timestamp DESC
             LIMIT 1",
        )?;

        let checkpoint = stmt
            .query_row(params![execution_id], |row| {
                let timestamp_str: String = row.get(0)?;
                let state_json: String = row.get(3)?;

                let timestamp = DateTime::parse_from_rfc3339(&timestamp_str)
                    .map_err(|e| {
                        rusqlite::Error::FromSqlConversionFailure(
                            0,
                            rusqlite::types::Type::Text,
                            Box::new(e),
                        )
                    })?
                    .with_timezone(&Utc);
                let state: StateSnapshot = serde_json::from_str(&state_json).map_err(|e| {
                    rusqlite::Error::FromSqlConversionFailure(
                        3,
                        rusqlite::types::Type::Text,
                        Box::new(e),
                    )
                })?;
                let active_nodes_json: String = row.get(5)?;
                let active_nodes: Vec<String> =
                    serde_json::from_str(&active_nodes_json).map_err(|e| {
                        rusqlite::Error::FromSqlConversionFailure(
                            5,
                            rusqlite::types::Type::Text,
                            Box::new(e),
                        )
                    })?;

                Ok(Checkpoint {
                    execution_id: execution_id.to_string(),
                    timestamp,
                    current_node: row.get(1)?,
                    iteration: sqlite_nonnegative_usize(row, 2)?,
                    state,
                    step_number: sqlite_nonnegative_usize(row, 4)?,
                    active_nodes,
                })
            })
            .optional()?;

        Ok(checkpoint)
    }

    /// Load all checkpoints for an execution (ordered by timestamp)
    pub fn load_all(&self, execution_id: &str) -> Result<Vec<Checkpoint>> {
        let mut stmt = self.conn.prepare(
            "SELECT timestamp, current_node, iteration, state_data, step_number, active_nodes
             FROM checkpoints
             WHERE execution_id = ?1
             ORDER BY timestamp ASC",
        )?;

        let checkpoints = stmt
            .query_map(params![execution_id], |row| {
                let timestamp_str: String = row.get(0)?;
                let state_json: String = row.get(3)?;

                let timestamp = DateTime::parse_from_rfc3339(&timestamp_str)
                    .map_err(|e| {
                        rusqlite::Error::FromSqlConversionFailure(
                            0,
                            rusqlite::types::Type::Text,
                            Box::new(e),
                        )
                    })?
                    .with_timezone(&Utc);
                let state: StateSnapshot = serde_json::from_str(&state_json).map_err(|e| {
                    rusqlite::Error::FromSqlConversionFailure(
                        3,
                        rusqlite::types::Type::Text,
                        Box::new(e),
                    )
                })?;
                let active_nodes_json: String = row.get(5)?;
                let active_nodes: Vec<String> =
                    serde_json::from_str(&active_nodes_json).map_err(|e| {
                        rusqlite::Error::FromSqlConversionFailure(
                            5,
                            rusqlite::types::Type::Text,
                            Box::new(e),
                        )
                    })?;

                Ok(Checkpoint {
                    execution_id: execution_id.to_string(),
                    timestamp,
                    current_node: row.get(1)?,
                    iteration: sqlite_nonnegative_usize(row, 2)?,
                    state,
                    step_number: sqlite_nonnegative_usize(row, 4)?,
                    active_nodes,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(checkpoints)
    }

    /// Delete all checkpoints for an execution
    pub fn clear(&self, execution_id: &str) -> Result<()> {
        self.conn.execute(
            "DELETE FROM checkpoints WHERE execution_id = ?1",
            params![execution_id],
        )?;
        Ok(())
    }
}

fn sqlite_nonnegative_usize(row: &rusqlite::Row<'_>, column: usize) -> rusqlite::Result<usize> {
    let value: i64 = row.get(column)?;
    usize::try_from(value).map_err(|error| {
        rusqlite::Error::FromSqlConversionFailure(
            column,
            rusqlite::types::Type::Integer,
            Box::new(error),
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn legacy_checkpoint_schema_gains_frontier_columns_with_defaults() {
        let db_path = std::env::temp_dir().join(format!(
            "agent-graph-legacy-checkpoint-{}.db",
            uuid::Uuid::new_v4()
        ));
        let db_path = db_path.to_string_lossy().into_owned();
        let conn = Connection::open(&db_path).unwrap();
        conn.execute(
            "CREATE TABLE checkpoints (
                execution_id TEXT NOT NULL,
                timestamp TEXT NOT NULL,
                current_node TEXT NOT NULL,
                iteration INTEGER NOT NULL,
                state_data TEXT NOT NULL,
                PRIMARY KEY (execution_id, timestamp)
            )",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO checkpoints (execution_id, timestamp, current_node, iteration, state_data)
             VALUES ('legacy-exec', '2026-01-01T00:00:00Z', 'node', 1, '{}')",
            [],
        )
        .unwrap();
        drop(conn);

        let manager = CheckpointManager::new(&db_path).unwrap();
        let (step_number, active_nodes): (i64, String) = manager
            .conn
            .query_row(
                "SELECT step_number, active_nodes FROM checkpoints WHERE execution_id = 'legacy-exec'",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();
        assert_eq!(step_number, 0);
        assert_eq!(active_nodes, "[]");
        std::fs::remove_file(db_path).unwrap();
    }

    #[test]
    fn checkpoint_load_rejects_negative_scheduler_frontier() {
        let db_path = std::env::temp_dir().join(format!(
            "agent-graph-negative-frontier-{}.db",
            uuid::Uuid::new_v4()
        ));
        let db_path = db_path.to_string_lossy().into_owned();
        let manager = CheckpointManager::new(&db_path).unwrap();
        let valid_state = serde_json::to_string(&StateSnapshot {
            timestamp: chrono::Utc::now(),
            data: std::collections::HashMap::new(),
        })
        .unwrap();
        manager
            .conn
            .execute(
                "INSERT INTO checkpoints (execution_id, timestamp, current_node, iteration, state_data, step_number, active_nodes)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    "negative-exec",
                    "2026-01-01T00:00:00Z",
                    "node",
                    -1_i64,
                    valid_state,
                    -1_i64,
                    "[]"
                ],
            )
            .unwrap();

        assert!(
            manager.load("negative-exec").is_err(),
            "corrupt negative scheduler values must not widen into usize values"
        );
        std::fs::remove_file(db_path).unwrap();
    }
}
