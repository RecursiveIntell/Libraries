//! Granular checkpoint store for recording node execution attempts.
//!
//! [`CheckpointStore`] provides per-attempt recording with input/output/status,
//! complementing the legacy [`CheckpointSaver`](crate::checkpointer::CheckpointSaver)
//! which operates at the superstep level.

use crate::error::AgentGraphError;
use crate::outcome::Interrupt;
use crate::Result;
use rusqlite::OptionalExtension;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Unique identifier for a graph run.
pub type RunId = String;
/// Unique identifier for a checkpoint-level node execution attempt.
///
/// This is an opaque checkpoint-level ID, distinct from `stack_ids::AttemptId`
/// which represents a retry-lineage primitive. The checkpoint store generates
/// these IDs internally for tracking per-node execution records.
pub type CheckpointAttemptId = String;

/// Status of a node execution attempt.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AttemptStatus {
    Running,
    Completed,
    Failed,
    Interrupted,
    Cancelled,
}

/// Record of a single node execution attempt.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttemptRecord {
    pub attempt_id: CheckpointAttemptId,
    pub run_id: RunId,
    pub node_id: String,
    pub attempt: u32,
    pub input: Value,
    pub output: Option<Value>,
    pub status: AttemptStatus,
    pub error: Option<String>,
    pub meta: HashMap<String, Value>,
    /// Canonical trace context for this attempt.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub trace_ctx: Option<stack_ids::TraceCtx>,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub finished_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Persisted state of a run, sufficient to resume execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunState {
    pub run_id: RunId,
    pub graph_name: String,
    pub status: RunStatus,
    pub attempts: Vec<AttemptRecord>,
    pub state_snapshot: HashMap<String, Value>,
    pub interrupted: Option<Interrupt>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Overall status of a run.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RunStatus {
    Running,
    Completed,
    Failed,
    Interrupted,
    Cancelled,
}

/// Granular checkpoint store for per-attempt recording.
///
/// This trait uses boxed futures instead of async-trait for forward compat.
pub trait CheckpointStore: Send + Sync {
    /// Create a new run and return its ID.
    fn create_run(
        &self,
        graph_name: &str,
    ) -> Pin<Box<dyn Future<Output = Result<RunId>> + Send + '_>>;

    /// Record a new node attempt (status: Running).
    fn record_attempt(
        &self,
        run_id: &str,
        node_id: &str,
        attempt: u32,
        input: &Value,
    ) -> Pin<Box<dyn Future<Output = Result<CheckpointAttemptId>> + Send + '_>>;

    /// Mark an attempt as completed with output.
    fn complete_attempt(
        &self,
        attempt_id: &str,
        output: &Value,
        meta: &HashMap<String, Value>,
    ) -> Pin<Box<dyn Future<Output = Result<()>> + Send + '_>>;

    /// Mark an attempt as failed.
    fn fail_attempt(
        &self,
        attempt_id: &str,
        error: &str,
    ) -> Pin<Box<dyn Future<Output = Result<()>> + Send + '_>>;

    /// Record an interrupt on an attempt.
    fn record_interrupt(
        &self,
        attempt_id: &str,
        interrupt: &Interrupt,
    ) -> Pin<Box<dyn Future<Output = Result<()>> + Send + '_>>;

    /// Save the current state snapshot for a run.
    fn save_state_snapshot(
        &self,
        run_id: &str,
        state: &HashMap<String, Value>,
    ) -> Pin<Box<dyn Future<Output = Result<()>> + Send + '_>>;

    /// Load the full run state (for resume).
    fn load_run(
        &self,
        run_id: &str,
    ) -> Pin<Box<dyn Future<Output = Result<Option<RunState>>> + Send + '_>>;

    /// Mark a run as completed.
    fn complete_run(&self, run_id: &str) -> Pin<Box<dyn Future<Output = Result<()>> + Send + '_>>;

    /// Mark a run as failed.
    fn fail_run(
        &self,
        run_id: &str,
        error: &str,
    ) -> Pin<Box<dyn Future<Output = Result<()>> + Send + '_>>;
}

/// Metadata attached to a checkpoint for validation and auditing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckpointMetadata {
    /// Hash of the graph definition at checkpoint time.
    /// Used to detect graph-definition drift on resume.
    pub graph_hash: String,
    /// The run this checkpoint belongs to.
    pub run_id: String,
    /// Node that was active when the checkpoint was taken.
    pub node_id: String,
    /// Superstep number.
    pub step: usize,
    /// When the checkpoint was created.
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Summary of a completed (or failed) graph run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunSummary {
    pub run_id: String,
    pub graph_name: String,
    pub status: RunStatus,
    pub total_nodes_executed: usize,
    pub total_attempts: usize,
    pub failed_attempts: usize,
    /// Phase status: compatibility / migration-only
    pub trace_id: Option<String>,
    /// Canonical trace context for this run.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub trace_ctx: Option<stack_ids::TraceCtx>,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub finished_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl InMemoryCheckpointStore {
    /// Build a [`RunSummary`] for the given run.
    pub async fn summarize_run(&self, run_id: &str) -> Option<RunSummary> {
        let runs = self.runs.read().await;
        let run = runs.get(run_id)?;
        let total_attempts = run.attempts.len();
        let failed_attempts = run
            .attempts
            .iter()
            .filter(|a| a.status == AttemptStatus::Failed)
            .count();
        let trace_id = run.attempts.iter().find_map(|attempt| {
            attempt
                .meta
                .get("trace_id")
                .and_then(|value| value.as_str())
                .map(str::to_owned)
        });
        let trace_ctx = run
            .attempts
            .iter()
            .find_map(|attempt| attempt.trace_ctx.clone());
        let unique_nodes: std::collections::HashSet<&str> =
            run.attempts.iter().map(|a| a.node_id.as_str()).collect();
        Some(RunSummary {
            run_id: run.run_id.clone(),
            graph_name: run.graph_name.clone(),
            status: run.status.clone(),
            total_nodes_executed: unique_nodes.len(),
            total_attempts,
            failed_attempts,
            trace_id,
            trace_ctx,
            started_at: run.created_at,
            finished_at: if run.status == RunStatus::Running {
                None
            } else {
                Some(run.updated_at)
            },
        })
    }
}

/// In-memory checkpoint store for testing and lightweight use.
pub struct InMemoryCheckpointStore {
    runs: Arc<RwLock<HashMap<RunId, RunState>>>,
    attempts: Arc<RwLock<HashMap<CheckpointAttemptId, AttemptRecord>>>,
}

impl InMemoryCheckpointStore {
    pub fn new() -> Self {
        Self {
            runs: Arc::new(RwLock::new(HashMap::new())),
            attempts: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// List all runs. Useful for testing and inspection.
    pub async fn list_runs(&self) -> Vec<RunState> {
        self.runs.read().await.values().cloned().collect()
    }
}

impl Default for InMemoryCheckpointStore {
    fn default() -> Self {
        Self::new()
    }
}

impl CheckpointStore for InMemoryCheckpointStore {
    fn create_run(
        &self,
        graph_name: &str,
    ) -> Pin<Box<dyn Future<Output = Result<RunId>> + Send + '_>> {
        let graph_name = graph_name.to_string();
        Box::pin(async move {
            let run_id = stack_ids::GraphRunId::random("agent-graph").to_string();
            let now = chrono::Utc::now();
            let run = RunState {
                run_id: run_id.clone(),
                graph_name,
                status: RunStatus::Running,
                attempts: Vec::new(),
                state_snapshot: HashMap::new(),
                interrupted: None,
                created_at: now,
                updated_at: now,
            };
            self.runs.write().await.insert(run_id.clone(), run);
            Ok(run_id)
        })
    }

    fn record_attempt(
        &self,
        run_id: &str,
        node_id: &str,
        attempt: u32,
        input: &Value,
    ) -> Pin<Box<dyn Future<Output = Result<CheckpointAttemptId>> + Send + '_>> {
        let run_id = run_id.to_string();
        let node_id = node_id.to_string();
        let input = input.clone();
        Box::pin(async move {
            let attempt_id =
                stack_ids::GraphCheckpointAttemptId::random("agent-graph-checkpoint").to_string();
            let now = chrono::Utc::now();
            let record = AttemptRecord {
                attempt_id: attempt_id.clone(),
                run_id: run_id.clone(),
                node_id: node_id.clone(),
                attempt,
                input,
                output: None,
                status: AttemptStatus::Running,
                error: None,
                meta: HashMap::new(),
                trace_ctx: None,
                started_at: now,
                finished_at: None,
            };
            self.attempts
                .write()
                .await
                .insert(attempt_id.clone(), record.clone());
            if let Some(run) = self.runs.write().await.get_mut(&run_id) {
                run.attempts.push(record);
                run.updated_at = now;
            }
            Ok(attempt_id)
        })
    }

    fn complete_attempt(
        &self,
        attempt_id: &str,
        output: &Value,
        meta: &HashMap<String, Value>,
    ) -> Pin<Box<dyn Future<Output = Result<()>> + Send + '_>> {
        let attempt_id = attempt_id.to_string();
        let output = output.clone();
        let meta = meta.clone();
        Box::pin(async move {
            let now = chrono::Utc::now();
            let mut attempts = self.attempts.write().await;
            if let Some(record) = attempts.get_mut(&attempt_id) {
                record.status = AttemptStatus::Completed;
                record.output = Some(output.clone());
                record.meta = meta.clone();
                record.finished_at = Some(now);
                // Also update in run
                let run_id = record.run_id.clone();
                drop(attempts);
                if let Some(run) = self.runs.write().await.get_mut(&run_id) {
                    if let Some(a) = run.attempts.iter_mut().find(|a| a.attempt_id == attempt_id) {
                        a.status = AttemptStatus::Completed;
                        a.output = Some(output);
                        a.meta = meta;
                        a.finished_at = Some(now);
                    }
                    run.updated_at = now;
                }
            }
            Ok(())
        })
    }

    fn fail_attempt(
        &self,
        attempt_id: &str,
        error: &str,
    ) -> Pin<Box<dyn Future<Output = Result<()>> + Send + '_>> {
        let attempt_id = attempt_id.to_string();
        let error = error.to_string();
        Box::pin(async move {
            let now = chrono::Utc::now();
            let mut attempts = self.attempts.write().await;
            if let Some(record) = attempts.get_mut(&attempt_id) {
                record.status = AttemptStatus::Failed;
                record.error = Some(error.clone());
                record.finished_at = Some(now);
                let run_id = record.run_id.clone();
                drop(attempts);
                if let Some(run) = self.runs.write().await.get_mut(&run_id) {
                    if let Some(a) = run.attempts.iter_mut().find(|a| a.attempt_id == attempt_id) {
                        a.status = AttemptStatus::Failed;
                        a.error = Some(error);
                        a.finished_at = Some(now);
                    }
                    run.updated_at = now;
                }
            }
            Ok(())
        })
    }

    fn record_interrupt(
        &self,
        attempt_id: &str,
        interrupt: &Interrupt,
    ) -> Pin<Box<dyn Future<Output = Result<()>> + Send + '_>> {
        let attempt_id = attempt_id.to_string();
        let interrupt = interrupt.clone();
        Box::pin(async move {
            let now = chrono::Utc::now();
            let mut attempts = self.attempts.write().await;
            if let Some(record) = attempts.get_mut(&attempt_id) {
                record.status = AttemptStatus::Interrupted;
                record.finished_at = Some(now);
                let run_id = record.run_id.clone();
                drop(attempts);
                if let Some(run) = self.runs.write().await.get_mut(&run_id) {
                    run.interrupted = Some(interrupt);
                    run.status = RunStatus::Interrupted;
                    if let Some(a) = run.attempts.iter_mut().find(|a| a.attempt_id == attempt_id) {
                        a.status = AttemptStatus::Interrupted;
                        a.finished_at = Some(now);
                    }
                    run.updated_at = now;
                }
            }
            Ok(())
        })
    }

    fn save_state_snapshot(
        &self,
        run_id: &str,
        state: &HashMap<String, Value>,
    ) -> Pin<Box<dyn Future<Output = Result<()>> + Send + '_>> {
        let run_id = run_id.to_string();
        let state = state.clone();
        Box::pin(async move {
            if let Some(run) = self.runs.write().await.get_mut(&run_id) {
                run.state_snapshot = state;
                run.updated_at = chrono::Utc::now();
            }
            Ok(())
        })
    }

    fn load_run(
        &self,
        run_id: &str,
    ) -> Pin<Box<dyn Future<Output = Result<Option<RunState>>> + Send + '_>> {
        let run_id = run_id.to_string();
        Box::pin(async move { Ok(self.runs.read().await.get(&run_id).cloned()) })
    }

    fn complete_run(&self, run_id: &str) -> Pin<Box<dyn Future<Output = Result<()>> + Send + '_>> {
        let run_id = run_id.to_string();
        Box::pin(async move {
            let mut runs = self.runs.write().await;
            let run = runs
                .get_mut(&run_id)
                .ok_or_else(|| crate::AgentGraphError::RunNotFound(run_id.clone()))?;
            if run.status != RunStatus::Running {
                return Err(crate::AgentGraphError::TerminalStateConflict(run_id));
            }
            run.status = RunStatus::Completed;
            run.updated_at = chrono::Utc::now();
            Ok(())
        })
    }

    fn fail_run(
        &self,
        run_id: &str,
        _error: &str,
    ) -> Pin<Box<dyn Future<Output = Result<()>> + Send + '_>> {
        let run_id = run_id.to_string();
        Box::pin(async move {
            let mut runs = self.runs.write().await;
            let run = runs
                .get_mut(&run_id)
                .ok_or_else(|| crate::AgentGraphError::RunNotFound(run_id.clone()))?;
            if run.status != RunStatus::Running {
                return Err(crate::AgentGraphError::TerminalStateConflict(run_id));
            }
            run.status = RunStatus::Failed;
            run.updated_at = chrono::Utc::now();
            Ok(())
        })
    }
}

/// SQLite-backed durable [`CheckpointStore`] using `rusqlite` with WAL mode.
///
/// All database access runs inside `tokio::task::spawn_blocking` to avoid
/// blocking the async runtime.  The schema mirrors the in-memory structures
/// exactly — runs, attempts, and state snapshots — so that a future
/// `PostgresCheckpointStore` can share the same layout.
///
/// ## Crash recovery
///
/// On construction, any run that was left in the `running` state by a previous
/// crash is atomically transitioned to `interrupted`.  Callers that know the
/// run ID can then resume it from the last persisted snapshot and attempt
/// records.
#[cfg(feature = "checkpointing")]
pub struct SqliteCheckpointStore {
    conn: Arc<std::sync::Mutex<rusqlite::Connection>>,
}

#[cfg(feature = "checkpointing")]
impl SqliteCheckpointStore {
    /// Open (or create) the checkpoint database at `path`.
    ///
    /// Schema is created if it does not exist.  WAL mode is enabled for
    /// concurrent read/write safety, matching the pattern used by `job-queue`.
    pub fn new(path: &str) -> Result<Self> {
        let conn = rusqlite::Connection::open(path).map_err(AgentGraphError::DatabaseError)?;

        conn.execute_batch(
            "PRAGMA journal_mode = WAL;
             PRAGMA foreign_keys = ON;
             PRAGMA busy_timeout = 5000;",
        )
        .map_err(AgentGraphError::DatabaseError)?;

        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS graph_runs (
                run_id          TEXT PRIMARY KEY,
                graph_name      TEXT NOT NULL,
                status          TEXT NOT NULL,
                interrupted     TEXT,
                created_at      TEXT NOT NULL,
                updated_at      TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS graph_attempts (
                attempt_id      TEXT PRIMARY KEY,
                run_id          TEXT NOT NULL REFERENCES graph_runs(run_id) ON DELETE CASCADE,
                node_id         TEXT NOT NULL,
                attempt         INTEGER NOT NULL,
                input           TEXT NOT NULL,
                output          TEXT,
                status          TEXT NOT NULL,
                error           TEXT,
                meta            TEXT NOT NULL DEFAULT '{}',
                trace_ctx       TEXT,
                started_at      TEXT NOT NULL,
                finished_at     TEXT,
                UNIQUE (run_id, node_id, attempt)
            );

            CREATE TABLE IF NOT EXISTS graph_state_snapshots (
                run_id          TEXT PRIMARY KEY REFERENCES graph_runs(run_id) ON DELETE CASCADE,
                state           TEXT NOT NULL,
                updated_at      TEXT NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_attempts_run
                ON graph_attempts(run_id);",
        )
        .map_err(AgentGraphError::DatabaseError)?;

        // Crash recovery: mark any run that was left running as interrupted.
        let now = chrono::Utc::now().to_rfc3339();
        let recovered = conn
            .execute(
                "UPDATE graph_runs SET status = 'interrupted', updated_at = ?1
                 WHERE status = 'running'",
                rusqlite::params![now],
            )
            .map_err(AgentGraphError::DatabaseError)?;

        if recovered > 0 {
            tracing::warn!(count = recovered, "Crash recovery: marked interrupted runs");
        }

        Ok(Self {
            conn: Arc::new(std::sync::Mutex::new(conn)),
        })
    }

    /// Build a [`RunSummary`] for the given run from its persisted records.
    pub fn summarize_run(&self, run_id: &str) -> Result<Option<RunSummary>> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| AgentGraphError::Other(e.to_string()))?;
        Self::summarize_run_impl(&conn, run_id)
    }

    fn summarize_run_impl(conn: &rusqlite::Connection, run_id: &str) -> Result<Option<RunSummary>> {
        let mut stmt = conn
            .prepare(
                "SELECT graph_name, status, created_at, updated_at FROM graph_runs WHERE run_id = ?1",
            )
            .map_err(AgentGraphError::DatabaseError)?;

        let run_row = stmt
            .query_row(rusqlite::params![run_id], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                ))
            })
            .optional()
            .map_err(AgentGraphError::DatabaseError)?;

        let Some((graph_name, status_str, created_at_str, updated_at_str)) = run_row else {
            return Ok(None);
        };

        let status = match status_str.as_str() {
            "running" => RunStatus::Running,
            "completed" => RunStatus::Completed,
            "failed" => RunStatus::Failed,
            "interrupted" => RunStatus::Interrupted,
            "cancelled" => RunStatus::Cancelled,
            _ => RunStatus::Failed,
        };

        let status_for_finished = status.clone();

        let (total_attempts, failed_attempts): (i64, i64) = conn
            .query_row(
                "SELECT COUNT(*), COUNT(CASE WHEN status = 'failed' THEN 1 END)
                 FROM graph_attempts WHERE run_id = ?1",
                rusqlite::params![run_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap_or((0, 0));

        let node_count: i64 = conn
            .query_row(
                "SELECT COUNT(DISTINCT node_id) FROM graph_attempts WHERE run_id = ?1",
                rusqlite::params![run_id],
                |row| row.get(0),
            )
            .unwrap_or(0);

        Ok(Some(RunSummary {
            run_id: run_id.to_string(),
            graph_name,
            status,
            total_nodes_executed: node_count as usize,
            total_attempts: total_attempts as usize,
            failed_attempts: failed_attempts as usize,
            trace_id: None,
            trace_ctx: None,
            started_at: chrono::DateTime::parse_from_rfc3339(&created_at_str)
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .unwrap_or_else(|_| chrono::Utc::now()),
            finished_at: if status_for_finished == RunStatus::Running {
                None
            } else {
                Some(
                    chrono::DateTime::parse_from_rfc3339(&updated_at_str)
                        .map(|dt| dt.with_timezone(&chrono::Utc))
                        .unwrap_or_else(|_| chrono::Utc::now()),
                )
            },
        }))
    }

    fn now_iso() -> String {
        chrono::Utc::now().to_rfc3339()
    }
}

#[cfg(feature = "checkpointing")]
impl CheckpointStore for SqliteCheckpointStore {
    fn create_run(
        &self,
        graph_name: &str,
    ) -> Pin<Box<dyn Future<Output = Result<RunId>> + Send + '_>> {
        let graph_name = graph_name.to_string();
        let run_id = stack_ids::GraphRunId::random("agent-graph").to_string();
        let now = Self::now_iso();
        let conn = self.conn.clone();

        Box::pin(async move {
            let rid = run_id.clone();
            tokio::task::spawn_blocking(move || -> Result<RunId> {
                let conn = conn
                    .lock()
                    .map_err(|e| AgentGraphError::Other(e.to_string()))?;
                conn.execute(
                    "INSERT INTO graph_runs (run_id, graph_name, status, created_at, updated_at)
                     VALUES (?1, ?2, 'running', ?3, ?3)",
                    rusqlite::params![rid, graph_name, now],
                )
                .map_err(AgentGraphError::DatabaseError)?;
                Ok(rid)
            })
            .await
            .map_err(|e| AgentGraphError::Other(e.to_string()))?
        })
    }

    fn record_attempt(
        &self,
        run_id: &str,
        node_id: &str,
        attempt: u32,
        input: &Value,
    ) -> Pin<Box<dyn Future<Output = Result<CheckpointAttemptId>> + Send + '_>> {
        let rid = run_id.to_string();
        let nid = node_id.to_string();
        let att = attempt;
        let input_json = serde_json::to_string(input).unwrap_or_else(|_| "null".to_string());
        let now = Self::now_iso();
        let attempt_id =
            stack_ids::GraphCheckpointAttemptId::random("agent-graph-checkpoint").to_string();
        let conn = self.conn.clone();

        Box::pin(async move {
            let aid = attempt_id.clone();
            tokio::task::spawn_blocking(move || -> Result<CheckpointAttemptId> {
                let conn = conn
                    .lock()
                    .map_err(|e| AgentGraphError::Other(e.to_string()))?;
                conn.execute(
                    "INSERT INTO graph_attempts
                     (attempt_id, run_id, node_id, attempt, input, status, started_at)
                     VALUES (?1, ?2, ?3, ?4, ?5, 'running', ?6)",
                    rusqlite::params![aid, rid, nid, att, input_json, now],
                )
                .map_err(AgentGraphError::DatabaseError)?;
                Ok(aid)
            })
            .await
            .map_err(|e| AgentGraphError::Other(e.to_string()))?
        })
    }

    fn complete_attempt(
        &self,
        attempt_id: &str,
        output: &Value,
        meta: &HashMap<String, Value>,
    ) -> Pin<Box<dyn Future<Output = Result<()>> + Send + '_>> {
        let aid = attempt_id.to_string();
        let output_json = serde_json::to_string(output).unwrap_or_else(|_| "null".to_string());
        let meta_json = serde_json::to_string(meta).unwrap_or_else(|_| "{}".to_string());
        let now = Self::now_iso();
        let conn = self.conn.clone();

        Box::pin(async move {
            tokio::task::spawn_blocking(move || -> Result<()> {
                let conn = conn
                    .lock()
                    .map_err(|e| AgentGraphError::Other(e.to_string()))?;
                conn.execute(
                    "UPDATE graph_attempts
                     SET status = 'completed', output = ?2, meta = ?3, finished_at = ?4
                     WHERE attempt_id = ?1",
                    rusqlite::params![aid, output_json, meta_json, now],
                )
                .map_err(AgentGraphError::DatabaseError)?;
                Ok(())
            })
            .await
            .map_err(|e| AgentGraphError::Other(e.to_string()))?
        })
    }

    fn fail_attempt(
        &self,
        attempt_id: &str,
        error: &str,
    ) -> Pin<Box<dyn Future<Output = Result<()>> + Send + '_>> {
        let aid = attempt_id.to_string();
        let err = error.to_string();
        let now = Self::now_iso();
        let conn = self.conn.clone();

        Box::pin(async move {
            tokio::task::spawn_blocking(move || -> Result<()> {
                let conn = conn
                    .lock()
                    .map_err(|e| AgentGraphError::Other(e.to_string()))?;
                conn.execute(
                    "UPDATE graph_attempts
                     SET status = 'failed', error = ?2, finished_at = ?3
                     WHERE attempt_id = ?1",
                    rusqlite::params![aid, err, now],
                )
                .map_err(AgentGraphError::DatabaseError)?;
                Ok(())
            })
            .await
            .map_err(|e| AgentGraphError::Other(e.to_string()))?
        })
    }

    fn record_interrupt(
        &self,
        attempt_id: &str,
        interrupt: &Interrupt,
    ) -> Pin<Box<dyn Future<Output = Result<()>> + Send + '_>> {
        let aid = attempt_id.to_string();
        let interrupt_json =
            serde_json::to_string(interrupt).unwrap_or_else(|_| "null".to_string());
        let now = Self::now_iso();
        let conn = self.conn.clone();

        Box::pin(async move {
            tokio::task::spawn_blocking(move || -> Result<()> {
                let conn = conn
                    .lock()
                    .map_err(|e| AgentGraphError::Other(e.to_string()))?;
                conn.execute(
                    "UPDATE graph_attempts SET status = 'interrupted', finished_at = ?2
                     WHERE attempt_id = ?1",
                    rusqlite::params![aid, now],
                )
                .map_err(AgentGraphError::DatabaseError)?;
                conn.execute(
                    "UPDATE graph_runs SET status = 'interrupted', interrupted = ?2,
                     updated_at = ?3 WHERE run_id = (
                        SELECT run_id FROM graph_attempts WHERE attempt_id = ?1
                     )",
                    rusqlite::params![aid, interrupt_json, now],
                )
                .map_err(AgentGraphError::DatabaseError)?;
                Ok(())
            })
            .await
            .map_err(|e| AgentGraphError::Other(e.to_string()))?
        })
    }

    fn save_state_snapshot(
        &self,
        run_id: &str,
        state: &HashMap<String, Value>,
    ) -> Pin<Box<dyn Future<Output = Result<()>> + Send + '_>> {
        let rid = run_id.to_string();
        let state_json = serde_json::to_string(state).unwrap_or_else(|_| "{}".to_string());
        let now = Self::now_iso();
        let conn = self.conn.clone();

        Box::pin(async move {
            tokio::task::spawn_blocking(move || -> Result<()> {
                let conn = conn
                    .lock()
                    .map_err(|e| AgentGraphError::Other(e.to_string()))?;
                conn.execute(
                    "INSERT INTO graph_state_snapshots (run_id, state, updated_at)
                     VALUES (?1, ?2, ?3)
                     ON CONFLICT(run_id) DO UPDATE SET state = excluded.state,
                     updated_at = excluded.updated_at",
                    rusqlite::params![rid, state_json, now],
                )
                .map_err(AgentGraphError::DatabaseError)?;
                Ok(())
            })
            .await
            .map_err(|e| AgentGraphError::Other(e.to_string()))?
        })
    }

    fn load_run(
        &self,
        run_id: &str,
    ) -> Pin<Box<dyn Future<Output = Result<Option<RunState>>> + Send + '_>> {
        let rid = run_id.to_string();
        let conn = self.conn.clone();

        Box::pin(async move {
            let rid2 = rid.clone();
            tokio::task::spawn_blocking(move || -> Result<Option<RunState>> {
                let conn = conn
                    .lock()
                    .map_err(|e| AgentGraphError::Other(e.to_string()))?;

                let mut stmt = conn
                    .prepare(
                        "SELECT graph_name, status, interrupted, created_at, updated_at
                         FROM graph_runs WHERE run_id = ?1",
                    )
                    .map_err(AgentGraphError::DatabaseError)?;

                let run_row = stmt
                    .query_row(rusqlite::params![rid2], |row| {
                        Ok((
                            row.get::<_, String>(0)?,
                            row.get::<_, String>(1)?,
                            row.get::<_, Option<String>>(2)?,
                            row.get::<_, String>(3)?,
                            row.get::<_, String>(4)?,
                        ))
                    })
                    .optional()
                    .map_err(AgentGraphError::DatabaseError)?;

                let Some((
                    graph_name,
                    status_str,
                    interrupted_json,
                    created_at_str,
                    updated_at_str,
                )) = run_row
                else {
                    return Ok(None);
                };

                let status = match status_str.as_str() {
                    "running" => RunStatus::Running,
                    "completed" => RunStatus::Completed,
                    "failed" => RunStatus::Failed,
                    "interrupted" => RunStatus::Interrupted,
                    "cancelled" => RunStatus::Cancelled,
                    _ => RunStatus::Failed,
                };

                let mut a_stmt = conn
                    .prepare(
                        "SELECT attempt_id, node_id, attempt, input, output, status,
                                error, meta, trace_ctx, started_at, finished_at
                         FROM graph_attempts WHERE run_id = ?1 ORDER BY started_at",
                    )
                    .map_err(AgentGraphError::DatabaseError)?;

                let attempts: Vec<AttemptRecord> = a_stmt
                    .query_map(rusqlite::params![rid2], |row| {
                        let status_str: String = row.get(5)?;
                        let status = match status_str.as_str() {
                            "running" => AttemptStatus::Running,
                            "completed" => AttemptStatus::Completed,
                            "failed" => AttemptStatus::Failed,
                            "interrupted" => AttemptStatus::Interrupted,
                            "cancelled" => AttemptStatus::Cancelled,
                            _ => AttemptStatus::Failed,
                        };
                        Ok(AttemptRecord {
                            attempt_id: row.get(0)?,
                            run_id: rid2.clone(),
                            node_id: row.get(1)?,
                            attempt: row.get::<_, i64>(2)? as u32,
                            input: serde_json::from_str(&row.get::<_, String>(3)?)
                                .unwrap_or(Value::Null),
                            output: row
                                .get::<_, Option<String>>(4)?
                                .map(|s| serde_json::from_str(&s).unwrap_or(Value::Null)),
                            status,
                            error: row.get(6)?,
                            meta: serde_json::from_str(
                                &row.get::<_, Option<String>>(7)?
                                    .unwrap_or_else(|| "{}".to_string()),
                            )
                            .unwrap_or_default(),
                            trace_ctx: row
                                .get::<_, Option<String>>(8)?
                                .and_then(|s| serde_json::from_str(&s).ok()),
                            started_at: chrono::DateTime::parse_from_rfc3339(
                                &row.get::<_, String>(9)?,
                            )
                            .map(|dt| dt.with_timezone(&chrono::Utc))
                            .unwrap_or_else(|_| chrono::Utc::now()),
                            finished_at: row.get::<_, Option<String>>(10)?.and_then(|s| {
                                chrono::DateTime::parse_from_rfc3339(&s)
                                    .map(|dt| dt.with_timezone(&chrono::Utc))
                                    .ok()
                            }),
                        })
                    })
                    .map_err(AgentGraphError::DatabaseError)?
                    .filter_map(|r| r.ok())
                    .collect();

                let state_snapshot: HashMap<String, Value> = conn
                    .query_row(
                        "SELECT state FROM graph_state_snapshots WHERE run_id = ?1",
                        rusqlite::params![rid2],
                        |row| row.get::<_, String>(0),
                    )
                    .optional()
                    .map_err(AgentGraphError::DatabaseError)?
                    .and_then(|s| serde_json::from_str(&s).ok())
                    .unwrap_or_default();

                let interrupted: Option<Interrupt> =
                    interrupted_json.and_then(|s| serde_json::from_str(&s).ok());

                let created_at = chrono::DateTime::parse_from_rfc3339(&created_at_str)
                    .map(|dt| dt.with_timezone(&chrono::Utc))
                    .unwrap_or_else(|_| chrono::Utc::now());
                let updated_at = chrono::DateTime::parse_from_rfc3339(&updated_at_str)
                    .map(|dt| dt.with_timezone(&chrono::Utc))
                    .unwrap_or_else(|_| chrono::Utc::now());

                Ok(Some(RunState {
                    run_id: rid2,
                    graph_name,
                    status,
                    attempts,
                    state_snapshot,
                    interrupted,
                    created_at,
                    updated_at,
                }))
            })
            .await
            .map_err(|e| AgentGraphError::Other(e.to_string()))?
        })
    }

    fn complete_run(&self, run_id: &str) -> Pin<Box<dyn Future<Output = Result<()>> + Send + '_>> {
        let rid = run_id.to_string();
        let now = Self::now_iso();
        let conn = self.conn.clone();

        Box::pin(async move {
            tokio::task::spawn_blocking(move || -> Result<()> {
                let conn = conn
                    .lock()
                    .map_err(|e| AgentGraphError::Other(e.to_string()))?;
                let affected = conn
                    .execute(
                        "UPDATE graph_runs SET status = 'completed', updated_at = ?2
                         WHERE run_id = ?1 AND status = 'running'",
                        rusqlite::params![rid, now],
                    )
                    .map_err(AgentGraphError::DatabaseError)?;
                if affected == 0 {
                    return Err(AgentGraphError::RunNotFound(rid));
                }
                Ok(())
            })
            .await
            .map_err(|e| AgentGraphError::Other(e.to_string()))?
        })
    }

    fn fail_run(
        &self,
        run_id: &str,
        _error: &str,
    ) -> Pin<Box<dyn Future<Output = Result<()>> + Send + '_>> {
        let rid = run_id.to_string();
        let now = Self::now_iso();
        let conn = self.conn.clone();

        Box::pin(async move {
            tokio::task::spawn_blocking(move || -> Result<()> {
                let conn = conn
                    .lock()
                    .map_err(|e| AgentGraphError::Other(e.to_string()))?;
                let affected = conn
                    .execute(
                        "UPDATE graph_runs SET status = 'failed', updated_at = ?2
                         WHERE run_id = ?1 AND status = 'running'",
                        rusqlite::params![rid, now],
                    )
                    .map_err(AgentGraphError::DatabaseError)?;
                if affected == 0 {
                    return Err(AgentGraphError::RunNotFound(rid));
                }
                Ok(())
            })
            .await
            .map_err(|e| AgentGraphError::Other(e.to_string()))?
        })
    }
}
