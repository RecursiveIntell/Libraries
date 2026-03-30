/// Range of valid user_version values for Forge databases.
pub const FORGE_MIN_USER_VERSION: u32 = 1;
pub const FORGE_MAX_USER_VERSION: u32 = 999;
pub const FORGE_CURRENT_USER_VERSION: u32 = 5;

/// Required tables in a Forge database.
pub const REQUIRED_TABLES: &[&str] = &[
    "forge_meta",
    "candidates",
    "tasks",
    "eval_runs",
    "archive_cells",
    "promotions",
    "answer_traces",
    "cea_nodes",
    "cea_edges",
    "cea_run_log",
];

/// All CREATE TABLE and CREATE INDEX statements in alphabetical order,
/// used to compute the schema hash. This is the single source of truth —
/// `db.rs` iterates this array directly instead of duplicating SQL.
pub const CREATE_STATEMENTS: &[&str] = &[
    // Tables
    "CREATE TABLE IF NOT EXISTS answer_traces (trace_id TEXT PRIMARY KEY, question_sig TEXT NOT NULL, version_id TEXT NOT NULL, strategy_tags_json TEXT NOT NULL, patch_hash TEXT NOT NULL, structural_sig TEXT NOT NULL, score_json TEXT NOT NULL, created_at TEXT NOT NULL)",
    "CREATE TABLE IF NOT EXISTS archive_cells (cell_key TEXT PRIMARY KEY, candidate_id TEXT NOT NULL, score_summary_json TEXT NOT NULL, cea_fingerprint TEXT, updated_at TEXT NOT NULL)",
    "CREATE TABLE IF NOT EXISTS candidates (candidate_id TEXT PRIMARY KEY, spec_json TEXT NOT NULL, parents_json TEXT NOT NULL DEFAULT '[]', created_at TEXT NOT NULL, status TEXT NOT NULL)",
    "CREATE TABLE IF NOT EXISTS cea_edges (edge_id TEXT PRIMARY KEY, cause_node_id TEXT NOT NULL REFERENCES cea_nodes(node_id), effect_node_id TEXT NOT NULL REFERENCES cea_nodes(node_id), weight REAL NOT NULL DEFAULT 0.0, count INTEGER NOT NULL DEFAULT 0, confidence REAL NOT NULL DEFAULT 0.0, alpha REAL NOT NULL DEFAULT 1.0, beta REAL NOT NULL DEFAULT 1.0, version_id TEXT NOT NULL, last_seen TEXT NOT NULL, UNIQUE(cause_node_id, effect_node_id, version_id))",
    "CREATE TABLE IF NOT EXISTS cea_nodes (node_id TEXT PRIMARY KEY, node_kind TEXT NOT NULL, sig_json TEXT NOT NULL, first_seen TEXT NOT NULL, last_seen TEXT NOT NULL)",
    "CREATE TABLE IF NOT EXISTS cea_run_log (run_hash TEXT PRIMARY KEY, eval_id TEXT NOT NULL, edges_added INTEGER NOT NULL, edges_updated INTEGER NOT NULL, processed_at TEXT NOT NULL)",
    "CREATE TABLE IF NOT EXISTS eval_runs (eval_id TEXT PRIMARY KEY, candidate_id TEXT NOT NULL, task_id TEXT NOT NULL, backend TEXT NOT NULL, seed INTEGER NOT NULL, mindstate_hash TEXT NOT NULL, patch_hash TEXT NOT NULL, structural_sig TEXT NOT NULL, scores_json TEXT NOT NULL, violations_json TEXT NOT NULL, logs_ref TEXT NOT NULL, cea_run_hash TEXT, created_at TEXT NOT NULL)",
    "CREATE TABLE IF NOT EXISTS forge_meta (key TEXT PRIMARY KEY, value TEXT NOT NULL)",
    "CREATE TABLE IF NOT EXISTS promotions (version_id TEXT PRIMARY KEY, candidate_id TEXT NOT NULL, frozen_spec_json TEXT NOT NULL, bounds_json TEXT NOT NULL, invariants_json TEXT NOT NULL, checksum TEXT NOT NULL, cea_fingerprint_json TEXT, promoted_at TEXT NOT NULL)",
    "CREATE TABLE IF NOT EXISTS tasks (task_id TEXT PRIMARY KEY, suite_name TEXT NOT NULL, fixture_ref TEXT NOT NULL, prompt TEXT NOT NULL, constraints_json TEXT NOT NULL, weights_json TEXT NOT NULL, created_at TEXT NOT NULL)",
    // Indexes
    "CREATE INDEX IF NOT EXISTS idx_answer_traces_question_sig ON answer_traces(question_sig)",
    "CREATE INDEX IF NOT EXISTS idx_cea_edges_cause ON cea_edges(cause_node_id)",
    "CREATE INDEX IF NOT EXISTS idx_cea_edges_effect ON cea_edges(effect_node_id)",
    "CREATE INDEX IF NOT EXISTS idx_cea_edges_version ON cea_edges(version_id)",
    "CREATE INDEX IF NOT EXISTS idx_eval_runs_candidate ON eval_runs(candidate_id)",
    "CREATE INDEX IF NOT EXISTS idx_eval_runs_task ON eval_runs(task_id)",
];

/// Migration v2 statements — additive tables for experiments, evidence, and exports.
///
/// These are NOT included in CREATE_STATEMENTS to preserve backward compatibility
/// with v1 databases. The schema hash only covers v1 tables.
pub const MIGRATION_V2_STATEMENTS: &[&str] = &[
    "CREATE TABLE IF NOT EXISTS evidence_bundles (bundle_id TEXT PRIMARY KEY, candidate_id TEXT NOT NULL, eval_id TEXT NOT NULL, version_id TEXT NOT NULL, trace_id TEXT NOT NULL, canonical_bundle_json TEXT, scores_json TEXT NOT NULL, hypotheses_json TEXT NOT NULL, verification_plan_json TEXT, diff_json TEXT, assessment_json TEXT, warnings_json TEXT NOT NULL DEFAULT '[]', created_at TEXT NOT NULL)",
    "CREATE TABLE IF NOT EXISTS experiment_runs (run_id TEXT PRIMARY KEY, candidate_id TEXT NOT NULL, task_id TEXT NOT NULL, trace_id TEXT NOT NULL, mode TEXT NOT NULL, baseline_json TEXT NOT NULL, patched_json TEXT NOT NULL, diff_json TEXT NOT NULL, baseline_descriptor_json TEXT NOT NULL, trials_json TEXT NOT NULL, completed_at TEXT NOT NULL)",
    "CREATE TABLE IF NOT EXISTS export_receipts (export_key TEXT PRIMARY KEY, bundle_id TEXT NOT NULL, rendering_version INTEGER NOT NULL, namespace TEXT NOT NULL, write_through_ok INTEGER, exported_at TEXT NOT NULL)",
    "CREATE TABLE IF NOT EXISTS run_failures (failure_id TEXT PRIMARY KEY, run_id TEXT NOT NULL, class TEXT NOT NULL, message TEXT NOT NULL, phase TEXT NOT NULL, retriable INTEGER NOT NULL DEFAULT 0, retry_count INTEGER NOT NULL DEFAULT 0, occurred_at TEXT NOT NULL)",
    "CREATE TABLE IF NOT EXISTS verification_plans (plan_id TEXT PRIMARY KEY, bundle_id TEXT NOT NULL, target_hypotheses_json TEXT NOT NULL, steps_json TEXT NOT NULL, policy_json TEXT, created_at TEXT NOT NULL)",
    // Indexes for v2 tables
    "CREATE INDEX IF NOT EXISTS idx_evidence_bundles_candidate ON evidence_bundles(candidate_id)",
    "CREATE INDEX IF NOT EXISTS idx_experiment_runs_candidate ON experiment_runs(candidate_id)",
    "CREATE INDEX IF NOT EXISTS idx_export_receipts_bundle ON export_receipts(bundle_id)",
    "CREATE INDEX IF NOT EXISTS idx_run_failures_run ON run_failures(run_id)",
    "CREATE INDEX IF NOT EXISTS idx_verification_plans_bundle ON verification_plans(bundle_id)",
];

/// The user_version value for schema v2.
pub const FORGE_V2_USER_VERSION: u32 = 2;

/// Migration v3 statements — additive raw tool receipt storage.
pub const MIGRATION_V3_STATEMENTS: &[&str] = &[
    "CREATE TABLE IF NOT EXISTS tool_receipts (receipt_id TEXT PRIMARY KEY, tool_run_id TEXT NOT NULL, tool_name TEXT NOT NULL, tool_version TEXT NOT NULL, backend_kind TEXT NOT NULL, input_digest TEXT NOT NULL, output_digest_or_refs_json TEXT NOT NULL, policy_hash TEXT NOT NULL, approval_state TEXT NOT NULL, host_identity TEXT NOT NULL, started_at TEXT NOT NULL, finished_at TEXT NOT NULL, trace_id TEXT NOT NULL, trace_ctx_json TEXT NOT NULL, attempt_id TEXT NOT NULL, trial_id TEXT NOT NULL, error_class TEXT, retry_owner TEXT NOT NULL, replay_link TEXT, provider_call_id TEXT, raw_payload_json TEXT NOT NULL, recorded_at TEXT NOT NULL)",
    "CREATE INDEX IF NOT EXISTS idx_tool_receipts_tool_name ON tool_receipts(tool_name)",
    "CREATE INDEX IF NOT EXISTS idx_tool_receipts_attempt ON tool_receipts(attempt_id)",
];

/// The user_version value for schema v3.
pub const FORGE_V3_USER_VERSION: u32 = 3;

/// The user_version value for schema v4.
pub const FORGE_V4_USER_VERSION: u32 = 4;

/// The user_version value for schema v5.
pub const FORGE_V5_USER_VERSION: u32 = 5;

/// Compute the schema hash from the sorted CREATE TABLE statements.
pub fn compute_schema_hash() -> String {
    let joined = CREATE_STATEMENTS.join("\n");
    blake3::hash(joined.as_bytes()).to_hex().to_string()
}

/// Compile-time schema hash (computed from CREATE_STATEMENTS).
pub fn forge_schema_hash() -> &'static str {
    static HASH: once_cell::sync::Lazy<String> = once_cell::sync::Lazy::new(compute_schema_hash);
    &HASH
}
