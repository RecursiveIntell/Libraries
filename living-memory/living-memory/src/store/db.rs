use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::Instant;

use rusqlite::{Connection, OptionalExtension};

use crate::error::{ForgeError, ForgeResult};
use crate::invariants;
use crate::lab::evaluate::ScoreVector;
use crate::lab::evidence::{
    CausalHypothesis, ClaimStrength, EvidenceAssessment, ExperimentEvidenceBundle, VerificationPlan,
};
use crate::store::schema;

const FORGE_LOCK_WARN_THRESHOLD_MS: u128 = 50;

fn stats_from_persisted(alpha: f64, beta: f64, count: i64) -> cea_core::EdgeStats {
    cea_core::EdgeStats {
        alpha,
        beta,
        observations: count.max(0) as u64,
    }
}

fn positive_stats(weight_delta: f64) -> cea_core::EdgeStats {
    let mut stats = cea_core::EdgeStats::default();
    stats.observe_positive(weight_delta);
    stats
}

/// ForgeStore — SQLite-backed storage for all Forge state.
///
/// The connection is wrapped in a `Mutex` for thread-safety.
pub struct ForgeStore {
    conn: Mutex<Connection>,
    path: PathBuf,
}

// Manual Debug impl to avoid requiring Debug on Mutex<Connection>
impl std::fmt::Debug for ForgeStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ForgeStore")
            .field("path", &self.path)
            .finish()
    }
}

impl ForgeStore {
    /// Open (or create) a Forge database at the given path.
    ///
    /// **Invariant I1**: Calls `invariants::refuse_to_open_db()` as the very first action.
    /// If the file exists and is not a valid Forge DB, returns `ForgeError::RefuseToOpenDb`.
    pub fn open(path: &Path) -> ForgeResult<Self> {
        // FIRST ACTION: refuse-to-open check
        invariants::refuse_to_open_db(path)?;

        let is_new = !path.exists();

        let conn = Connection::open(path)?;

        // Enable WAL mode for better concurrent read performance
        conn.pragma_update(None, "journal_mode", "wal")?;
        // Set busy timeout to avoid SQLITE_BUSY under concurrent access
        conn.pragma_update(None, "busy_timeout", "5000")?;

        if is_new {
            Self::run_initial_migration(&conn)?;
        }

        // Run additive migration v2 (idempotent — uses IF NOT EXISTS)
        Self::run_migration_v2(&conn)?;
        Self::run_migration_v3(&conn)?;
        Self::run_migration_v4(&conn)?;
        Self::run_migration_v5(&conn)?;

        Ok(Self {
            conn: Mutex::new(conn),
            path: path.to_path_buf(),
        })
    }

    /// Run the initial migration (migration 1) — creates all tables and indexes.
    ///
    /// Uses `schema::CREATE_STATEMENTS` as single source of truth to prevent
    /// SQL divergence between the schema hash and actual DDL.
    fn run_initial_migration(conn: &Connection) -> ForgeResult<()> {
        conn.execute_batch(&format!(
            "PRAGMA user_version = {user_version};",
            user_version = schema::FORGE_CURRENT_USER_VERSION,
        ))?;

        // Execute all CREATE TABLE and CREATE INDEX statements from the
        // single source of truth in schema.rs
        for stmt in schema::CREATE_STATEMENTS {
            conn.execute(stmt, [])?;
        }

        // Seed forge_meta
        conn.execute_batch(&format!(
            r#"
INSERT INTO forge_meta VALUES ('schema_hash',    '{schema_hash}');
INSERT INTO forge_meta VALUES ('schema_version', '1');
INSERT INTO forge_meta VALUES ('created_at',     strftime('%Y-%m-%dT%H:%M:%SZ', 'now'));
"#,
            schema_hash = schema::forge_schema_hash(),
        ))?;

        Ok(())
    }

    /// Run migration v2 — additive tables for experiments, evidence, exports.
    ///
    /// This is idempotent: all statements use IF NOT EXISTS.
    /// Historical v1 databases gain the new tables without data loss.
    fn run_migration_v2(conn: &Connection) -> ForgeResult<()> {
        for stmt in schema::MIGRATION_V2_STATEMENTS {
            conn.execute(stmt, [])?;
        }

        // Update user_version if still at v1
        let current: u32 = conn.pragma_query_value(None, "user_version", |row| row.get(0))?;
        if current < schema::FORGE_V2_USER_VERSION {
            conn.pragma_update(None, "user_version", schema::FORGE_V2_USER_VERSION)?;
        }

        Ok(())
    }

    fn run_migration_v3(conn: &Connection) -> ForgeResult<()> {
        for stmt in schema::MIGRATION_V3_STATEMENTS {
            conn.execute(stmt, [])?;
        }

        let current: u32 = conn.pragma_query_value(None, "user_version", |row| row.get(0))?;
        if current < schema::FORGE_V3_USER_VERSION {
            conn.pragma_update(None, "user_version", schema::FORGE_V3_USER_VERSION)?;
        }

        Ok(())
    }

    fn run_migration_v4(conn: &Connection) -> ForgeResult<()> {
        let has_alpha = conn
            .query_row(
                "SELECT 1 FROM pragma_table_info('cea_edges') WHERE name = 'alpha' LIMIT 1",
                [],
                |_| Ok(()),
            )
            .optional()?
            .is_some();
        let has_beta = conn
            .query_row(
                "SELECT 1 FROM pragma_table_info('cea_edges') WHERE name = 'beta' LIMIT 1",
                [],
                |_| Ok(()),
            )
            .optional()?
            .is_some();

        if !has_alpha {
            conn.execute(
                "ALTER TABLE cea_edges ADD COLUMN alpha REAL NOT NULL DEFAULT 1.0",
                [],
            )?;
        }
        if !has_beta {
            conn.execute(
                "ALTER TABLE cea_edges ADD COLUMN beta REAL NOT NULL DEFAULT 1.0",
                [],
            )?;
        }

        let mut stmt = conn.prepare("SELECT edge_id, weight, count, alpha, beta FROM cea_edges")?;
        let rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, f64>(1)?,
                row.get::<_, i64>(2)?,
                row.get::<_, f64>(3)?,
                row.get::<_, f64>(4)?,
            ))
        })?;

        for row in rows {
            let (edge_id, weight, count, alpha, beta) = row?;
            let stats = if has_alpha && has_beta {
                stats_from_persisted(alpha, beta, count)
            } else {
                cea_core::EdgeStats {
                    alpha: 1.0 + weight.max(0.0),
                    beta: 1.0,
                    observations: count.max(0) as u64,
                }
            };
            conn.execute(
                "UPDATE cea_edges SET alpha = ?1, beta = ?2, confidence = ?3 WHERE edge_id = ?4",
                rusqlite::params![stats.alpha, stats.beta, stats.confidence(), edge_id],
            )?;
        }

        let current: u32 = conn.pragma_query_value(None, "user_version", |row| row.get(0))?;
        if current < schema::FORGE_V4_USER_VERSION {
            conn.pragma_update(None, "user_version", schema::FORGE_V4_USER_VERSION)?;
        }

        Ok(())
    }

    fn run_migration_v5(conn: &Connection) -> ForgeResult<()> {
        let has_canonical_bundle_json = conn
            .query_row(
                "SELECT 1 FROM pragma_table_info('evidence_bundles') WHERE name = 'canonical_bundle_json' LIMIT 1",
                [],
                |_| Ok(()),
            )
            .optional()?
            .is_some();

        if !has_canonical_bundle_json {
            conn.execute(
                "ALTER TABLE evidence_bundles ADD COLUMN canonical_bundle_json TEXT",
                [],
            )?;
        }

        let current: u32 = conn.pragma_query_value(None, "user_version", |row| row.get(0))?;
        if current < schema::FORGE_V5_USER_VERSION {
            conn.pragma_update(None, "user_version", schema::FORGE_V5_USER_VERSION)?;
        }

        Ok(())
    }

    /// Acquire the connection lock. All public methods use this internally.
    fn lock_conn(&self) -> ForgeResult<std::sync::MutexGuard<'_, Connection>> {
        let started_at = Instant::now();
        let guard = self
            .conn
            .lock()
            .map_err(|e| ForgeError::Other(format!("ForgeStore lock poisoned: {e}")))?;
        let waited_ms = started_at.elapsed().as_millis();
        if waited_ms >= FORGE_LOCK_WARN_THRESHOLD_MS {
            tracing::warn!(
                path = %self.path.display(),
                waited_ms,
                threshold_ms = FORGE_LOCK_WARN_THRESHOLD_MS,
                "ForgeStore connection lock acquisition exceeded threshold"
            );
        }
        Ok(guard)
    }

    /// Execute a closure with transactional semantics.
    pub fn with_transaction<F, T>(&self, f: F) -> ForgeResult<T>
    where
        F: FnOnce(&Connection) -> ForgeResult<T>,
    {
        let mut conn = self.lock_conn()?;
        let tx = conn.transaction().map_err(ForgeError::Database)?;
        let result = f(&tx)?;
        tx.commit().map_err(ForgeError::Database)?;
        Ok(result)
    }

    /// Execute a closure with a write transaction handle for consumers that can
    /// execute transaction-owned CEA adapters without opening a nested tx.
    pub fn with_transaction_conn<F, T>(&self, f: F) -> ForgeResult<T>
    where
        F: for<'tx> FnOnce(&rusqlite::Transaction<'tx>) -> ForgeResult<T>,
    {
        let mut conn = self.lock_conn()?;
        let tx = conn.transaction()?;
        let result = f(&tx)?;
        tx.commit().map_err(ForgeError::Database)?;
        Ok(result)
    }

    /// Execute a closure with the shared write-locked SQLite connection.
    pub fn with_conn<F, T>(&self, f: F) -> ForgeResult<T>
    where
        F: FnOnce(&Connection) -> ForgeResult<T>,
    {
        let conn = self.lock_conn()?;
        f(&conn)
    }

    /// Get the database path.
    pub fn path(&self) -> &Path {
        &self.path
    }

    // ── Candidate CRUD ──

    pub fn insert_candidate(
        &self,
        candidate_id: &str,
        spec_json: &str,
        parents_json: &str,
        status: &str,
    ) -> ForgeResult<()> {
        let now = chrono::Utc::now().to_rfc3339();
        self.lock_conn()?.execute(
            "INSERT INTO candidates (candidate_id, spec_json, parents_json, created_at, status) VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![candidate_id, spec_json, parents_json, now, status],
        )?;
        Ok(())
    }

    pub fn get_candidate_spec(&self, candidate_id: &str) -> ForgeResult<String> {
        self.lock_conn()?
            .query_row(
                "SELECT spec_json FROM candidates WHERE candidate_id = ?1",
                [candidate_id],
                |row| row.get(0),
            )
            .map_err(|_| ForgeError::NotFound(format!("candidate {candidate_id}")))
    }

    pub fn update_candidate_status(&self, candidate_id: &str, status: &str) -> ForgeResult<()> {
        self.lock_conn()?.execute(
            "UPDATE candidates SET status = ?1 WHERE candidate_id = ?2",
            rusqlite::params![status, candidate_id],
        )?;
        Ok(())
    }

    // ── Task CRUD ──

    pub fn insert_task(
        &self,
        task_id: &str,
        suite_name: &str,
        fixture_ref: &str,
        prompt: &str,
        constraints_json: &str,
        weights_json: &str,
    ) -> ForgeResult<()> {
        let now = chrono::Utc::now().to_rfc3339();
        self.lock_conn()?.execute(
            "INSERT INTO tasks (task_id, suite_name, fixture_ref, prompt, constraints_json, weights_json, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            rusqlite::params![task_id, suite_name, fixture_ref, prompt, constraints_json, weights_json, now],
        )?;
        Ok(())
    }

    // ── EvalRun CRUD ──

    #[allow(clippy::too_many_arguments)]
    pub fn insert_eval_run(
        &self,
        eval_id: &str,
        candidate_id: &str,
        task_id: &str,
        backend: &str,
        seed: i64,
        mindstate_hash: &str,
        patch_hash: &str,
        structural_sig: &str,
        scores_json: &str,
        violations_json: &str,
        logs_ref: &str,
        cea_run_hash: Option<&str>,
    ) -> ForgeResult<()> {
        let now = chrono::Utc::now().to_rfc3339();
        self.lock_conn()?.execute(
            "INSERT INTO eval_runs (eval_id, candidate_id, task_id, backend, seed, mindstate_hash, patch_hash, structural_sig, scores_json, violations_json, logs_ref, cea_run_hash, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
            rusqlite::params![eval_id, candidate_id, task_id, backend, seed, mindstate_hash, patch_hash, structural_sig, scores_json, violations_json, logs_ref, cea_run_hash, now],
        )?;
        Ok(())
    }

    pub fn get_eval_runs_for_candidate(&self, candidate_id: &str) -> ForgeResult<Vec<EvalRunRow>> {
        let conn = self.lock_conn()?;
        let mut stmt = conn.prepare(
            "SELECT eval_id, candidate_id, task_id, backend, seed, mindstate_hash, patch_hash, structural_sig, scores_json, violations_json, logs_ref, cea_run_hash, created_at FROM eval_runs WHERE candidate_id = ?1"
        )?;
        let rows = stmt.query_map([candidate_id], |row| {
            Ok(EvalRunRow {
                eval_id: row.get(0)?,
                candidate_id: row.get(1)?,
                task_id: row.get(2)?,
                backend: row.get(3)?,
                seed: row.get(4)?,
                mindstate_hash: row.get(5)?,
                patch_hash: row.get(6)?,
                structural_sig: row.get(7)?,
                scores_json: row.get(8)?,
                violations_json: row.get(9)?,
                logs_ref: row.get(10)?,
                cea_run_hash: row.get(11)?,
                created_at: row.get(12)?,
            })
        })?;
        let mut result = Vec::new();
        for row in rows {
            result.push(row?);
        }
        Ok(result)
    }

    // ── Archive Cells ──

    pub fn upsert_archive_cell(
        &self,
        cell_key: &str,
        candidate_id: &str,
        score_summary_json: &str,
        cea_fingerprint: Option<&str>,
    ) -> ForgeResult<()> {
        let now = chrono::Utc::now().to_rfc3339();
        self.lock_conn()?.execute(
            "INSERT INTO archive_cells (cell_key, candidate_id, score_summary_json, cea_fingerprint, updated_at) VALUES (?1, ?2, ?3, ?4, ?5) ON CONFLICT(cell_key) DO UPDATE SET candidate_id = ?2, score_summary_json = ?3, cea_fingerprint = ?4, updated_at = ?5",
            rusqlite::params![cell_key, candidate_id, score_summary_json, cea_fingerprint, now],
        )?;
        Ok(())
    }

    pub fn get_archive_cell(&self, cell_key: &str) -> ForgeResult<Option<ArchiveCellRow>> {
        match self.lock_conn()?.query_row(
            "SELECT cell_key, candidate_id, score_summary_json, cea_fingerprint, updated_at FROM archive_cells WHERE cell_key = ?1",
            [cell_key],
            |row| {
                Ok(ArchiveCellRow {
                    cell_key: row.get(0)?,
                    candidate_id: row.get(1)?,
                    score_summary_json: row.get(2)?,
                    cea_fingerprint: row.get(3)?,
                    updated_at: row.get(4)?,
                })
            },
        ) {
            Ok(r) => Ok(Some(r)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    // ── Promotions ──

    #[allow(clippy::too_many_arguments)]
    pub fn insert_promotion(
        &self,
        version_id: &str,
        candidate_id: &str,
        frozen_spec_json: &str,
        bounds_json: &str,
        invariants_json: &str,
        checksum: &str,
        cea_fingerprint_json: Option<&str>,
    ) -> ForgeResult<()> {
        let now = chrono::Utc::now().to_rfc3339();
        self.lock_conn()?.execute(
            "INSERT INTO promotions (version_id, candidate_id, frozen_spec_json, bounds_json, invariants_json, checksum, cea_fingerprint_json, promoted_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            rusqlite::params![version_id, candidate_id, frozen_spec_json, bounds_json, invariants_json, checksum, cea_fingerprint_json, now],
        )?;
        Ok(())
    }

    pub fn get_latest_promotion(&self) -> ForgeResult<Option<PromotionRow>> {
        match self.lock_conn()?.query_row(
            "SELECT version_id, candidate_id, frozen_spec_json, bounds_json, invariants_json, checksum, cea_fingerprint_json, promoted_at FROM promotions ORDER BY version_id DESC LIMIT 1",
            [],
            |row| {
                Ok(PromotionRow {
                    version_id: row.get(0)?,
                    candidate_id: row.get(1)?,
                    frozen_spec_json: row.get(2)?,
                    bounds_json: row.get(3)?,
                    invariants_json: row.get(4)?,
                    checksum: row.get(5)?,
                    cea_fingerprint_json: row.get(6)?,
                    promoted_at: row.get(7)?,
                })
            },
        ) {
            Ok(r) => Ok(Some(r)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    pub fn count_promotions(&self) -> ForgeResult<usize> {
        let count: i64 =
            self.lock_conn()?
                .query_row("SELECT COUNT(*) FROM promotions", [], |row| row.get(0))?;
        Ok(count as usize)
    }

    // ── Answer Traces ──

    #[allow(clippy::too_many_arguments)]
    pub fn insert_answer_trace(
        &self,
        trace_id: &str,
        question_sig: &str,
        version_id: &str,
        strategy_tags_json: &str,
        patch_hash: &str,
        structural_sig: &str,
        score_json: &str,
    ) -> ForgeResult<()> {
        let now = chrono::Utc::now().to_rfc3339();
        self.lock_conn()?.execute(
            "INSERT INTO answer_traces (trace_id, question_sig, version_id, strategy_tags_json, patch_hash, structural_sig, score_json, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            rusqlite::params![trace_id, question_sig, version_id, strategy_tags_json, patch_hash, structural_sig, score_json, now],
        )?;
        Ok(())
    }

    pub fn get_recent_traces_for_question(
        &self,
        question_sig: &str,
        limit: usize,
    ) -> ForgeResult<Vec<AnswerTraceRow>> {
        let conn = self.lock_conn()?;
        let mut stmt = conn.prepare(
            "SELECT trace_id, question_sig, version_id, strategy_tags_json, patch_hash, structural_sig, score_json, created_at FROM answer_traces WHERE question_sig = ?1 ORDER BY created_at DESC LIMIT ?2",
        )?;
        let rows = stmt.query_map(rusqlite::params![question_sig, limit as i64], |row| {
            Ok(AnswerTraceRow {
                trace_id: row.get(0)?,
                question_sig: row.get(1)?,
                version_id: row.get(2)?,
                strategy_tags_json: row.get(3)?,
                patch_hash: row.get(4)?,
                structural_sig: row.get(5)?,
                score_json: row.get(6)?,
                created_at: row.get(7)?,
            })
        })?;
        let mut result = Vec::new();
        for row in rows {
            result.push(row?);
        }
        Ok(result)
    }

    // ── CEA CRUD ──

    pub fn upsert_cea_node(
        &self,
        node_id: &str,
        node_kind: &str,
        sig_json: &str,
    ) -> ForgeResult<()> {
        let now = chrono::Utc::now().to_rfc3339();
        self.lock_conn()?.execute(
            "INSERT INTO cea_nodes (node_id, node_kind, sig_json, first_seen, last_seen) VALUES (?1, ?2, ?3, ?4, ?4) ON CONFLICT(node_id) DO UPDATE SET last_seen = ?4",
            rusqlite::params![node_id, node_kind, sig_json, now],
        )?;
        Ok(())
    }

    pub fn upsert_cea_edge(
        &self,
        edge_id: &str,
        cause_node_id: &str,
        effect_node_id: &str,
        weight_delta: f64,
        version_id: &str,
    ) -> ForgeResult<bool> {
        let now = chrono::Utc::now().to_rfc3339();
        let conn = self.lock_conn()?;
        let weight_delta = weight_delta.max(0.0);
        let existing = conn
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
            .optional()?;
        if let Some((weight, count, alpha, beta)) = existing {
            let mut stats = stats_from_persisted(alpha, beta, count);
            stats.observe_positive(weight_delta);
            conn.execute(
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
            )?;
            return Ok(false);
        }
        let stats = positive_stats(weight_delta);
        conn.execute(
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
        )?;
        Ok(true)
    }

    pub fn check_cea_run_log(&self, run_hash: &str) -> ForgeResult<bool> {
        let count: i64 = self.lock_conn()?.query_row(
            "SELECT COUNT(*) FROM cea_run_log WHERE run_hash = ?1",
            [run_hash],
            |row| row.get(0),
        )?;
        Ok(count > 0)
    }

    pub fn insert_cea_run_log(
        &self,
        run_hash: &str,
        eval_id: &str,
        edges_added: i64,
        edges_updated: i64,
    ) -> ForgeResult<()> {
        let now = chrono::Utc::now().to_rfc3339();
        self.lock_conn()?.execute(
            "INSERT INTO cea_run_log (run_hash, eval_id, edges_added, edges_updated, processed_at) VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![run_hash, eval_id, edges_added, edges_updated, now],
        )?;
        Ok(())
    }

    pub fn get_cea_edges_for_cause(&self, cause_node_id: &str) -> ForgeResult<Vec<CeaEdgeRow>> {
        let conn = self.lock_conn()?;
        let mut stmt = conn.prepare(
            "SELECT edge_id, cause_node_id, effect_node_id, weight, count, confidence, alpha, beta, version_id, last_seen FROM cea_edges WHERE cause_node_id = ?1",
        )?;
        let rows = stmt.query_map([cause_node_id], |row| {
            let count: i64 = row.get(4)?;
            let alpha: f64 = row.get(6)?;
            let beta: f64 = row.get(7)?;
            let stats = stats_from_persisted(alpha, beta, count);
            Ok(CeaEdgeRow {
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
        })?;
        let mut result = Vec::new();
        for row in rows {
            result.push(row?);
        }
        Ok(result)
    }

    /// Get all CEA nodes and edges, optionally filtered by version_id.
    pub fn get_all_cea_nodes_and_edges(
        &self,
        version_id: Option<&str>,
    ) -> ForgeResult<(Vec<CeaNodeRow>, Vec<CeaEdgeRow>)> {
        let conn = self.lock_conn()?;

        // Nodes
        let mut node_stmt = conn.prepare("SELECT node_id, node_kind, sig_json FROM cea_nodes")?;
        let node_rows = node_stmt.query_map([], |row| {
            Ok(CeaNodeRow {
                node_id: row.get(0)?,
                node_kind: row.get(1)?,
                sig_json: row.get(2)?,
            })
        })?;
        let mut nodes = Vec::new();
        for row in node_rows {
            nodes.push(row?);
        }

        // Edges
        let edges = if let Some(vid) = version_id {
            let mut edge_stmt = conn.prepare(
                "SELECT edge_id, cause_node_id, effect_node_id, weight, count, confidence, alpha, beta, version_id, last_seen FROM cea_edges WHERE version_id = ?1",
            )?;
            let edge_rows = edge_stmt.query_map([vid], |row| {
                let count: i64 = row.get(4)?;
                let alpha: f64 = row.get(6)?;
                let beta: f64 = row.get(7)?;
                let stats = stats_from_persisted(alpha, beta, count);
                Ok(CeaEdgeRow {
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
            })?;
            let mut result = Vec::new();
            for row in edge_rows {
                result.push(row?);
            }
            result
        } else {
            let mut edge_stmt = conn.prepare(
                "SELECT edge_id, cause_node_id, effect_node_id, weight, count, confidence, alpha, beta, version_id, last_seen FROM cea_edges",
            )?;
            let edge_rows = edge_stmt.query_map([], |row| {
                let count: i64 = row.get(4)?;
                let alpha: f64 = row.get(6)?;
                let beta: f64 = row.get(7)?;
                let stats = stats_from_persisted(alpha, beta, count);
                Ok(CeaEdgeRow {
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
            })?;
            let mut result = Vec::new();
            for row in edge_rows {
                result.push(row?);
            }
            result
        };

        Ok((nodes, edges))
    }

    pub fn get_cea_node_sig(&self, node_id: &str) -> ForgeResult<Option<String>> {
        match self.lock_conn()?.query_row(
            "SELECT sig_json FROM cea_nodes WHERE node_id = ?1",
            [node_id],
            |row| row.get(0),
        ) {
            Ok(s) => Ok(Some(s)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    // ── Evidence Bundles (v2) ──

    #[allow(clippy::too_many_arguments)]
    pub fn insert_evidence_bundle(
        &self,
        bundle_id: &str,
        candidate_id: &str,
        eval_id: &str,
        version_id: &str,
        trace_id: &str,
        scores_json: &str,
        hypotheses_json: &str,
        verification_plan_json: Option<&str>,
        diff_json: Option<&str>,
        assessment_json: Option<&str>,
        warnings_json: &str,
    ) -> ForgeResult<()> {
        let now = chrono::Utc::now().to_rfc3339();
        let canonical_bundle_json = canonical_bundle_json_from_legacy_parts(
            bundle_id,
            candidate_id,
            eval_id,
            version_id,
            trace_id,
            scores_json,
            hypotheses_json,
            verification_plan_json,
            diff_json,
            assessment_json,
            warnings_json,
            &now,
        )?;
        self.lock_conn()?.execute(
            "INSERT OR REPLACE INTO evidence_bundles (bundle_id, candidate_id, eval_id, version_id, trace_id, canonical_bundle_json, scores_json, hypotheses_json, verification_plan_json, diff_json, assessment_json, warnings_json, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
            rusqlite::params![bundle_id, candidate_id, eval_id, version_id, trace_id, canonical_bundle_json, scores_json, hypotheses_json, verification_plan_json, diff_json, assessment_json, warnings_json, now],
        )?;
        Ok(())
    }

    pub fn get_evidence_bundle(&self, bundle_id: &str) -> ForgeResult<Option<EvidenceBundleRow>> {
        match self.lock_conn()?.query_row(
            "SELECT bundle_id, candidate_id, eval_id, version_id, trace_id, canonical_bundle_json, scores_json, hypotheses_json, verification_plan_json, diff_json, assessment_json, warnings_json, created_at FROM evidence_bundles WHERE bundle_id = ?1",
            [bundle_id],
            |row| {
                Ok(EvidenceBundleRow {
                    bundle_id: row.get(0)?,
                    candidate_id: row.get(1)?,
                    eval_id: row.get(2)?,
                    version_id: row.get(3)?,
                    trace_id: row.get(4)?,
                    canonical_bundle_json: row.get(5)?,
                    scores_json: row.get(6)?,
                    hypotheses_json: row.get(7)?,
                    verification_plan_json: row.get(8)?,
                    diff_json: row.get(9)?,
                    assessment_json: row.get(10)?,
                    warnings_json: row.get(11)?,
                    created_at: row.get(12)?,
                })
            },
        ) {
            Ok(r) => Ok(Some(r)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    pub fn get_latest_evidence_bundle_for_candidate(
        &self,
        candidate_id: &str,
    ) -> ForgeResult<Option<EvidenceBundleRow>> {
        match self.lock_conn()?.query_row(
            "SELECT bundle_id, candidate_id, eval_id, version_id, trace_id, canonical_bundle_json, scores_json, hypotheses_json, verification_plan_json, diff_json, assessment_json, warnings_json, created_at
             FROM evidence_bundles
             WHERE candidate_id = ?1
             ORDER BY created_at DESC, bundle_id DESC
             LIMIT 1",
            [candidate_id],
            |row| {
                Ok(EvidenceBundleRow {
                    bundle_id: row.get(0)?,
                    candidate_id: row.get(1)?,
                    eval_id: row.get(2)?,
                    version_id: row.get(3)?,
                    trace_id: row.get(4)?,
                    canonical_bundle_json: row.get(5)?,
                    scores_json: row.get(6)?,
                    hypotheses_json: row.get(7)?,
                    verification_plan_json: row.get(8)?,
                    diff_json: row.get(9)?,
                    assessment_json: row.get(10)?,
                    warnings_json: row.get(11)?,
                    created_at: row.get(12)?,
                })
            },
        ) {
            Ok(r) => Ok(Some(r)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    pub fn count_evidence_bundles_for_candidate(&self, candidate_id: &str) -> ForgeResult<usize> {
        let count: i64 = self.lock_conn()?.query_row(
            "SELECT COUNT(*) FROM evidence_bundles WHERE candidate_id = ?1",
            [candidate_id],
            |row| row.get(0),
        )?;
        Ok(count as usize)
    }

    pub fn count_evidence_bundles(&self) -> ForgeResult<usize> {
        let count: i64 =
            self.lock_conn()?
                .query_row("SELECT COUNT(*) FROM evidence_bundles", [], |row| {
                    row.get(0)
                })?;
        Ok(count as usize)
    }

    pub fn list_recent_evidence_bundle_ids(&self, limit: usize) -> ForgeResult<Vec<String>> {
        let conn = self.lock_conn()?;
        let mut stmt = conn.prepare(
            "SELECT bundle_id
             FROM evidence_bundles
             ORDER BY created_at DESC, bundle_id DESC
             LIMIT ?1",
        )?;
        let rows = stmt.query_map([limit as i64], |row| row.get::<_, String>(0))?;
        let mut result = Vec::new();
        for row in rows {
            result.push(row?);
        }
        Ok(result)
    }

    // ── Experiment Runs (v2) ──

    #[allow(clippy::too_many_arguments)]
    pub fn insert_experiment_run(
        &self,
        run_id: &str,
        candidate_id: &str,
        task_id: &str,
        trace_id: &str,
        mode: &str,
        baseline_json: &str,
        patched_json: &str,
        diff_json: &str,
        baseline_descriptor_json: &str,
        trials_json: &str,
    ) -> ForgeResult<()> {
        let now = chrono::Utc::now().to_rfc3339();
        self.lock_conn()?.execute(
            "INSERT INTO experiment_runs (run_id, candidate_id, task_id, trace_id, mode, baseline_json, patched_json, diff_json, baseline_descriptor_json, trials_json, completed_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            rusqlite::params![run_id, candidate_id, task_id, trace_id, mode, baseline_json, patched_json, diff_json, baseline_descriptor_json, trials_json, now],
        )?;
        Ok(())
    }

    // ── Export Receipts (v2) ──

    pub fn insert_export_receipt(
        &self,
        export_key: &str,
        bundle_id: &str,
        rendering_version: u32,
        namespace: &str,
        write_through_ok: Option<bool>,
    ) -> ForgeResult<bool> {
        let now = chrono::Utc::now().to_rfc3339();
        let wt = write_through_ok.map(|b| if b { 1i64 } else { 0 });
        let result = self.lock_conn()?.execute(
            "INSERT OR IGNORE INTO export_receipts (export_key, bundle_id, rendering_version, namespace, write_through_ok, exported_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![export_key, bundle_id, rendering_version as i64, namespace, wt, now],
        )?;
        Ok(result > 0)
    }

    pub fn has_export_receipt(&self, export_key: &str) -> ForgeResult<bool> {
        let count: i64 = self.lock_conn()?.query_row(
            "SELECT COUNT(*) FROM export_receipts WHERE export_key = ?1",
            [export_key],
            |row| row.get(0),
        )?;
        Ok(count > 0)
    }

    // ── Run Failures (v2) ──

    #[allow(clippy::too_many_arguments)]
    pub fn insert_run_failure(
        &self,
        failure_id: &str,
        run_id: &str,
        class: &str,
        message: &str,
        phase: &str,
        retriable: bool,
        retry_count: u32,
    ) -> ForgeResult<()> {
        let now = chrono::Utc::now().to_rfc3339();
        self.lock_conn()?.execute(
            "INSERT INTO run_failures (failure_id, run_id, class, message, phase, retriable, retry_count, occurred_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            rusqlite::params![failure_id, run_id, class, message, phase, retriable as i64, retry_count as i64, now],
        )?;
        Ok(())
    }

    pub fn get_failures_for_run(&self, run_id: &str) -> ForgeResult<Vec<RunFailureRow>> {
        let conn = self.lock_conn()?;
        let mut stmt = conn.prepare(
            "SELECT failure_id, run_id, class, message, phase, retriable, retry_count, occurred_at FROM run_failures WHERE run_id = ?1 ORDER BY occurred_at",
        )?;
        let rows = stmt.query_map([run_id], |row| {
            Ok(RunFailureRow {
                failure_id: row.get(0)?,
                run_id: row.get(1)?,
                class: row.get(2)?,
                message: row.get(3)?,
                phase: row.get(4)?,
                retriable: row.get::<_, i64>(5)? != 0,
                retry_count: row.get::<_, i64>(6)? as u32,
                occurred_at: row.get(7)?,
            })
        })?;
        let mut result = Vec::new();
        for row in rows {
            result.push(row?);
        }
        Ok(result)
    }

    // ── Verification Plans (v2) ──

    pub fn insert_verification_plan(
        &self,
        plan_id: &str,
        bundle_id: &str,
        target_hypotheses_json: &str,
        steps_json: &str,
        policy_json: Option<&str>,
    ) -> ForgeResult<()> {
        let now = chrono::Utc::now().to_rfc3339();
        self.lock_conn()?.execute(
            "INSERT INTO verification_plans (plan_id, bundle_id, target_hypotheses_json, steps_json, policy_json, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![plan_id, bundle_id, target_hypotheses_json, steps_json, policy_json, now],
        )?;
        Ok(())
    }

    // ── Tool Receipts (v3) ──

    pub fn insert_tool_receipt(&self, row: &ToolReceiptRow) -> ForgeResult<bool> {
        let result = self.lock_conn()?.execute(
            "INSERT OR REPLACE INTO tool_receipts (
                receipt_id, tool_run_id, tool_name, tool_version, backend_kind,
                input_digest, output_digest_or_refs_json, policy_hash, approval_state,
                host_identity, started_at, finished_at, trace_id, trace_ctx_json,
                attempt_id, trial_id, error_class, retry_owner, replay_link,
                provider_call_id, raw_payload_json, recorded_at
            ) VALUES (
                ?1, ?2, ?3, ?4, ?5,
                ?6, ?7, ?8, ?9,
                ?10, ?11, ?12, ?13, ?14,
                ?15, ?16, ?17, ?18, ?19,
                ?20, ?21, ?22
            )",
            rusqlite::params![
                row.receipt_id,
                row.tool_run_id,
                row.tool_name,
                row.tool_version,
                row.backend_kind,
                row.input_digest,
                row.output_digest_or_refs_json,
                row.policy_hash,
                row.approval_state,
                row.host_identity,
                row.started_at,
                row.finished_at,
                row.trace_id,
                row.trace_ctx_json,
                row.attempt_id,
                row.trial_id,
                row.error_class,
                row.retry_owner,
                row.replay_link,
                row.provider_call_id,
                row.raw_payload_json,
                row.recorded_at,
            ],
        )?;
        Ok(result > 0)
    }

    pub fn get_tool_receipt(&self, receipt_id: &str) -> ForgeResult<Option<ToolReceiptRow>> {
        match self.lock_conn()?.query_row(
            "SELECT receipt_id, tool_run_id, tool_name, tool_version, backend_kind,
                    input_digest, output_digest_or_refs_json, policy_hash, approval_state,
                    host_identity, started_at, finished_at, trace_id, trace_ctx_json,
                    attempt_id, trial_id, error_class, retry_owner, replay_link,
                    provider_call_id, raw_payload_json, recorded_at
             FROM tool_receipts
             WHERE receipt_id = ?1",
            [receipt_id],
            |row| {
                Ok(ToolReceiptRow {
                    receipt_id: row.get(0)?,
                    tool_run_id: row.get(1)?,
                    tool_name: row.get(2)?,
                    tool_version: row.get(3)?,
                    backend_kind: row.get(4)?,
                    input_digest: row.get(5)?,
                    output_digest_or_refs_json: row.get(6)?,
                    policy_hash: row.get(7)?,
                    approval_state: row.get(8)?,
                    host_identity: row.get(9)?,
                    started_at: row.get(10)?,
                    finished_at: row.get(11)?,
                    trace_id: row.get(12)?,
                    trace_ctx_json: row.get(13)?,
                    attempt_id: row.get(14)?,
                    trial_id: row.get(15)?,
                    error_class: row.get(16)?,
                    retry_owner: row.get(17)?,
                    replay_link: row.get(18)?,
                    provider_call_id: row.get(19)?,
                    raw_payload_json: row.get(20)?,
                    recorded_at: row.get(21)?,
                })
            },
        ) {
            Ok(row) => Ok(Some(row)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(err) => Err(err.into()),
        }
    }
}

impl Drop for ForgeStore {
    fn drop(&mut self) {
        // LIB-019: Flush WAL on teardown to avoid stale journal files.
        match self.conn.lock() {
            Ok(conn) => {
                if let Err(e) = conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE)") {
                    tracing::warn!(error = %e, path = %self.path.display(), "ForgeStore: WAL checkpoint on drop failed");
                }
            }
            Err(e) => {
                tracing::warn!(error = %e, "ForgeStore: mutex poisoned during drop, skipping WAL checkpoint");
            }
        }
    }
}

// ── Row types ──

#[derive(Debug, Clone)]
pub struct EvalRunRow {
    pub eval_id: String,
    pub candidate_id: String,
    pub task_id: String,
    pub backend: String,
    pub seed: i64,
    pub mindstate_hash: String,
    pub patch_hash: String,
    pub structural_sig: String,
    pub scores_json: String,
    pub violations_json: String,
    pub logs_ref: String,
    pub cea_run_hash: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone)]
pub struct ArchiveCellRow {
    pub cell_key: String,
    pub candidate_id: String,
    pub score_summary_json: String,
    pub cea_fingerprint: Option<String>,
    pub updated_at: String,
}

#[derive(Debug, Clone)]
pub struct PromotionRow {
    pub version_id: String,
    pub candidate_id: String,
    pub frozen_spec_json: String,
    pub bounds_json: String,
    pub invariants_json: String,
    pub checksum: String,
    pub cea_fingerprint_json: Option<String>,
    pub promoted_at: String,
}

#[derive(Debug, Clone)]
pub struct AnswerTraceRow {
    pub trace_id: String,
    pub question_sig: String,
    pub version_id: String,
    pub strategy_tags_json: String,
    pub patch_hash: String,
    pub structural_sig: String,
    pub score_json: String,
    pub created_at: String,
}

#[derive(Debug, Clone)]
pub struct CeaNodeRow {
    pub node_id: String,
    pub node_kind: String,
    pub sig_json: String,
}

#[derive(Debug, Clone)]
pub struct CeaEdgeRow {
    pub edge_id: String,
    pub cause_node_id: String,
    pub effect_node_id: String,
    pub weight: f64,
    pub alpha: f64,
    pub beta: f64,
    pub count: i64,
    pub confidence: f64,
    pub version_id: String,
    pub last_seen: String,
}

// ── v2 row types ──

#[derive(Debug, Clone)]
pub struct EvidenceBundleRow {
    pub bundle_id: String,
    pub candidate_id: String,
    pub eval_id: String,
    pub version_id: String,
    pub trace_id: String,
    pub canonical_bundle_json: Option<String>,
    pub scores_json: String,
    pub hypotheses_json: String,
    pub verification_plan_json: Option<String>,
    pub diff_json: Option<String>,
    pub assessment_json: Option<String>,
    pub warnings_json: String,
    pub created_at: String,
}

impl EvidenceBundleRow {
    pub fn canonical_bundle(&self) -> ForgeResult<semantic_memory_forge::EvidenceBundle> {
        if let Some(canonical_bundle_json) = &self.canonical_bundle_json {
            return Ok(serde_json::from_str(canonical_bundle_json)?);
        }

        Ok(legacy_row_to_local_bundle(self)?.to_canonical_evidence_bundle())
    }

    pub fn local_bundle(&self) -> ForgeResult<ExperimentEvidenceBundle> {
        let canonical = self.canonical_bundle()?;
        ExperimentEvidenceBundle::from_canonical_evidence_bundle(&canonical)
    }
}

#[derive(Debug, Clone)]
pub struct RunFailureRow {
    pub failure_id: String,
    pub run_id: String,
    pub class: String,
    pub message: String,
    pub phase: String,
    pub retriable: bool,
    pub retry_count: u32,
    pub occurred_at: String,
}

#[derive(Debug, Clone)]
pub struct ToolReceiptRow {
    pub receipt_id: String,
    pub tool_run_id: String,
    pub tool_name: String,
    pub tool_version: String,
    pub backend_kind: String,
    pub input_digest: String,
    pub output_digest_or_refs_json: String,
    pub policy_hash: String,
    pub approval_state: String,
    pub host_identity: String,
    pub started_at: String,
    pub finished_at: String,
    pub trace_id: String,
    pub trace_ctx_json: String,
    pub attempt_id: String,
    pub trial_id: String,
    pub error_class: Option<String>,
    pub retry_owner: String,
    pub replay_link: Option<String>,
    pub provider_call_id: Option<String>,
    pub raw_payload_json: String,
    pub recorded_at: String,
}

#[allow(clippy::too_many_arguments)]
fn canonical_bundle_json_from_legacy_parts(
    bundle_id: &str,
    candidate_id: &str,
    eval_id: &str,
    version_id: &str,
    trace_id: &str,
    scores_json: &str,
    hypotheses_json: &str,
    verification_plan_json: Option<&str>,
    diff_json: Option<&str>,
    assessment_json: Option<&str>,
    warnings_json: &str,
    created_at: &str,
) -> ForgeResult<String> {
    Ok(serde_json::to_string(
        &legacy_bundle_from_parts(
            bundle_id,
            candidate_id,
            eval_id,
            version_id,
            trace_id,
            scores_json,
            hypotheses_json,
            verification_plan_json,
            diff_json,
            assessment_json,
            warnings_json,
            created_at,
        )?
        .to_canonical_evidence_bundle(),
    )?)
}

fn legacy_row_to_local_bundle(row: &EvidenceBundleRow) -> ForgeResult<ExperimentEvidenceBundle> {
    legacy_bundle_from_parts(
        &row.bundle_id,
        &row.candidate_id,
        &row.eval_id,
        &row.version_id,
        &row.trace_id,
        &row.scores_json,
        &row.hypotheses_json,
        row.verification_plan_json.as_deref(),
        row.diff_json.as_deref(),
        row.assessment_json.as_deref(),
        &row.warnings_json,
        &row.created_at,
    )
}

#[allow(clippy::too_many_arguments)]
fn legacy_bundle_from_parts(
    bundle_id: &str,
    candidate_id: &str,
    eval_id: &str,
    version_id: &str,
    trace_id: &str,
    scores_json: &str,
    hypotheses_json: &str,
    verification_plan_json: Option<&str>,
    diff_json: Option<&str>,
    assessment_json: Option<&str>,
    warnings_json: &str,
    created_at: &str,
) -> ForgeResult<ExperimentEvidenceBundle> {
    Ok(ExperimentEvidenceBundle {
        bundle_id: bundle_id.to_string(),
        candidate_id: candidate_id.to_string(),
        eval_id: eval_id.to_string(),
        version_id: version_id.to_string(),
        supersedes_claim_version_id: None,
        relation_lineage_hints: Default::default(),
        scores: parse_legacy_score_vector(scores_json)?,
        hypotheses: serde_json::from_str::<Vec<CausalHypothesis>>(hypotheses_json)?,
        verification: verification_plan_json
            .map(serde_json::from_str::<VerificationPlan>)
            .transpose()?,
        trace_id: Some(trace_id.to_string()),
        experiment_diff: parse_legacy_experiment_diff(diff_json),
        attribution_json: None,
        assessment: assessment_json
            .map(serde_json::from_str::<EvidenceAssessment>)
            .transpose()?,
        warnings: serde_json::from_str::<Vec<String>>(warnings_json)?,
        created_at: created_at.to_string(),
        run_id: None,
        attempt_id: None,
        causal_question: None,
        unit_definition: None,
        bundle_scope: None,
        pair_comparability: None,
        claim_strength: ClaimStrength::ProvisionalSinglePair,
        identification_rationale: None,
        known_threats: Vec::new(),
        patch_hash: None,
        treatment: None,
        outcome: None,
        covariates: None,
        promotion_state: None,
        primary_effect: None,
        all_effects: Vec::new(),
        hypothesis_edges: Vec::new(),
        receipts: Vec::new(),
        verification_trials: Vec::new(),
        refutation_artifacts: Vec::new(),
        sealed: false,
    })
}

fn parse_legacy_score_vector(scores_json: &str) -> ForgeResult<ScoreVector> {
    let value: serde_json::Value = serde_json::from_str(scores_json)?;
    if let Ok(scores) = serde_json::from_value::<ScoreVector>(value.clone()) {
        return Ok(scores);
    }

    let object = value.as_object().ok_or_else(|| {
        ForgeError::Other("legacy evidence scores must deserialize to a JSON object".into())
    })?;

    Ok(ScoreVector {
        correctness: object
            .get("correctness")
            .and_then(serde_json::Value::as_f64)
            .unwrap_or(0.0),
        novelty: object
            .get("novelty")
            .and_then(serde_json::Value::as_f64)
            .unwrap_or(0.0),
        stability: object
            .get("stability")
            .and_then(serde_json::Value::as_f64)
            .unwrap_or(0.0),
        weighted_total: object
            .get("weighted_total")
            .and_then(serde_json::Value::as_f64)
            .unwrap_or(0.0),
        cea_confidence: object
            .get("cea_confidence")
            .and_then(serde_json::Value::as_f64),
        cea_predicted_correctness: object
            .get("cea_predicted_correctness")
            .and_then(serde_json::Value::as_f64),
    })
}

fn parse_legacy_experiment_diff(
    diff_json: Option<&str>,
) -> Option<crate::experiment::ExperimentDiff> {
    diff_json.and_then(|diff_json| serde_json::from_str(diff_json).ok())
}
