//! Tests for additive migrations, store CRUD for new tables, and failure recording.

use forge_engine::store::schema;
use forge_engine::{FailureClass, ForgeStore};
use tempfile::TempDir;

/// Additive migrations run on a fresh DB without error.
#[test]
fn fresh_db_has_v2_and_v3_tables() {
    let dir = TempDir::new().unwrap();
    let db_path = dir.path().join("forge.db");
    let store = ForgeStore::open(&db_path).unwrap();

    // Verify v2 tables exist by exercising them
    store
        .insert_evidence_bundle(
            "eb-1", "c-1", "e-1", "v0001", "trace-1", "{}", "[]", None, None, None, "[]",
        )
        .unwrap();

    let row = store.get_evidence_bundle("eb-1").unwrap();
    assert!(row.is_some());
    assert_eq!(row.unwrap().trace_id, "trace-1");

    let row = store.get_tool_receipt("missing-receipt").unwrap();
    assert!(row.is_none());
}

/// Additive migrations are idempotent: opening the same DB twice works.
#[test]
fn migration_v2_idempotent_reopen() {
    let dir = TempDir::new().unwrap();
    let db_path = dir.path().join("forge.db");

    {
        let store = ForgeStore::open(&db_path).unwrap();
        store
            .insert_evidence_bundle(
                "eb-reopen",
                "c-1",
                "e-1",
                "v0001",
                "trace-r",
                "{}",
                "[]",
                None,
                None,
                None,
                "[]",
            )
            .unwrap();
    }

    // Reopen — additive migrations run again (IF NOT EXISTS), data preserved
    {
        let store = ForgeStore::open(&db_path).unwrap();
        let row = store.get_evidence_bundle("eb-reopen").unwrap();
        assert!(row.is_some());
        assert_eq!(row.unwrap().bundle_id, "eb-reopen");
        let row = store.get_tool_receipt("missing-receipt").unwrap();
        assert!(row.is_none());
    }
}

/// V1 DB (created before additive migrations) gets upgraded transparently.
#[test]
fn v1_db_upgraded_to_latest() {
    let dir = TempDir::new().unwrap();
    let db_path = dir.path().join("forge.db");

    // Simulate a v1 DB: create tables + forge_meta manually
    {
        let conn = rusqlite::Connection::open(&db_path).unwrap();
        conn.pragma_update(None, "user_version", schema::FORGE_CURRENT_USER_VERSION)
            .unwrap();
        for stmt in schema::CREATE_STATEMENTS {
            conn.execute(stmt, []).unwrap();
        }
        conn.execute_batch(&format!(
            "INSERT INTO forge_meta VALUES ('schema_hash', '{}');
             INSERT INTO forge_meta VALUES ('schema_version', '1');
             INSERT INTO forge_meta VALUES ('created_at', '2025-01-01T00:00:00Z');",
            schema::forge_schema_hash(),
        ))
        .unwrap();

        // Insert a v1 candidate
        conn.execute(
            "INSERT INTO candidates (candidate_id, spec_json, parents_json, created_at, status) VALUES ('c-v1', '{}', '[]', '2025-01-01', 'active')",
            [],
        ).unwrap();
    }

    // Open via ForgeStore — should run additive migrations.
    let store = ForgeStore::open(&db_path).unwrap();

    // v1 data preserved
    let spec = store.get_candidate_spec("c-v1").unwrap();
    assert_eq!(spec, "{}");

    // v2 tables now exist
    store
        .insert_evidence_bundle(
            "eb-v2", "c-v1", "e-v2", "v0001", "trace-v2", "{}", "[]", None, None, None, "[]",
        )
        .unwrap();
    let row = store.get_evidence_bundle("eb-v2").unwrap();
    assert!(row.is_some());

    // v3 raw tool receipt storage now exists too.
    let row = store.get_tool_receipt("missing-receipt").unwrap();
    assert!(row.is_none());
}

/// User version gets bumped to the latest additive migration after open.
#[test]
fn user_version_is_v4_after_open() {
    let dir = TempDir::new().unwrap();
    let db_path = dir.path().join("forge.db");

    let _store = ForgeStore::open(&db_path).unwrap();

    let conn = rusqlite::Connection::open(&db_path).unwrap();
    let version: u32 = conn
        .pragma_query_value(None, "user_version", |row| row.get(0))
        .unwrap();
    assert_eq!(version, schema::FORGE_V5_USER_VERSION);
}

// ── Evidence bundle CRUD ──

#[test]
fn evidence_bundle_crud() {
    let dir = TempDir::new().unwrap();
    let db_path = dir.path().join("forge.db");
    let store = ForgeStore::open(&db_path).unwrap();

    // Insert
    store
        .insert_evidence_bundle(
            "eb-crud",
            "c-1",
            "e-1",
            "v0001",
            "trace-crud",
            r#"{"correctness": 0.9}"#,
            r#"[{"hypothesis_id":"h1","cause_signature":"cause","effect_signature":"effect","confidence":0.8,"status":"Supported","support_count":2,"contradiction_count":0}]"#,
            Some(r#"{"plan_id":"p1","target_hypotheses":["h1"],"steps":[]}"#),
            Some(r#"{"effects":[],"regressions":0,"improvements":0,"stable_failures":0,"stable_passes":0,"statistically_meaningful":false,"sample_warning":null}"#),
            Some(r#"{"reproducibility":"Strong","isolation":"Strong","contradiction_state":"Clean","sample_support":"Sufficient"}"#),
            r#"["warning1"]"#,
        )
        .unwrap();

    // Retrieve
    let row = store.get_evidence_bundle("eb-crud").unwrap().unwrap();
    assert_eq!(row.candidate_id, "c-1");
    assert_eq!(row.trace_id, "trace-crud");
    assert!(row.canonical_bundle_json.is_some());
    assert!(row.scores_json.contains("0.9"));
    assert!(row.hypotheses_json.contains("h1"));
    assert!(row
        .verification_plan_json
        .as_deref()
        .unwrap()
        .contains("p1"));
    assert!(row.diff_json.as_deref().unwrap().contains("regressions"));
    assert!(row.assessment_json.as_deref().unwrap().contains("Strong"));
    assert!(row.warnings_json.contains("warning1"));
    let canonical = row.canonical_bundle().unwrap();
    assert_eq!(canonical.id.as_str(), "eb-crud");
    assert_eq!(
        canonical
            .metadata
            .as_ref()
            .and_then(|metadata| metadata.get("candidate_id"))
            .and_then(|value| value.as_str()),
        Some("c-1")
    );

    // Count
    let count = store.count_evidence_bundles_for_candidate("c-1").unwrap();
    assert_eq!(count, 1);

    // Insert another for same candidate
    store
        .insert_evidence_bundle(
            "eb-crud-2",
            "c-1",
            "e-2",
            "v0001",
            "trace-crud2",
            "{}",
            "[]",
            None,
            None,
            None,
            "[]",
        )
        .unwrap();
    assert_eq!(
        store.count_evidence_bundles_for_candidate("c-1").unwrap(),
        2
    );
}

#[test]
fn evidence_bundle_not_found() {
    let dir = TempDir::new().unwrap();
    let db_path = dir.path().join("forge.db");
    let store = ForgeStore::open(&db_path).unwrap();

    let row = store.get_evidence_bundle("nonexistent").unwrap();
    assert!(row.is_none());
}

#[test]
fn evidence_bundle_insert_or_replace() {
    let dir = TempDir::new().unwrap();
    let db_path = dir.path().join("forge.db");
    let store = ForgeStore::open(&db_path).unwrap();

    store
        .insert_evidence_bundle(
            "eb-dup",
            "c-1",
            "e-1",
            "v0001",
            "trace-1",
            r#"{"correctness": 0.5}"#,
            "[]",
            None,
            None,
            None,
            "[]",
        )
        .unwrap();

    // Insert again with updated scores — should replace
    store
        .insert_evidence_bundle(
            "eb-dup",
            "c-1",
            "e-1",
            "v0001",
            "trace-1",
            r#"{"correctness": 0.95}"#,
            "[]",
            None,
            None,
            None,
            "[]",
        )
        .unwrap();

    let row = store.get_evidence_bundle("eb-dup").unwrap().unwrap();
    assert!(row.scores_json.contains("0.95"));
    assert_eq!(
        store.count_evidence_bundles_for_candidate("c-1").unwrap(),
        1
    );
}

#[test]
fn legacy_split_only_evidence_row_rebuilds_canonical_bundle() {
    let dir = TempDir::new().unwrap();
    let db_path = dir.path().join("forge.db");
    let store = ForgeStore::open(&db_path).unwrap();

    let conn = rusqlite::Connection::open(&db_path).unwrap();
    conn.execute(
        "INSERT INTO evidence_bundles (bundle_id, candidate_id, eval_id, version_id, trace_id, scores_json, hypotheses_json, verification_plan_json, diff_json, assessment_json, warnings_json, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
        rusqlite::params![
            "eb-legacy",
            "cand-legacy",
            "eval-legacy",
            "v0001",
            "trace-legacy",
            r#"{"correctness":0.8,"novelty":0.1,"stability":0.2,"weighted_total":0.7,"cea_confidence":null,"cea_predicted_correctness":null}"#,
            r#"[{"hypothesis_id":"h-1","cause_signature":"cause","effect_signature":"effect","confidence":0.6,"status":"Proposed","support_count":1,"contradiction_count":0}]"#,
            r#"{"plan_id":"plan-1","target_hypotheses":["h-1"],"steps":[]}"#,
            Option::<&str>::None,
            r#"{"reproducibility":"Strong","isolation":"Strong","contradiction_state":"Clean","sample_support":"Sufficient"}"#,
            r#"["legacy-row"]"#,
            "2026-03-12T00:00:00Z"
        ],
    )
    .unwrap();
    drop(conn);

    let row = store.get_evidence_bundle("eb-legacy").unwrap().unwrap();
    assert!(row.canonical_bundle_json.is_none());

    let canonical = row.canonical_bundle().unwrap();
    assert_eq!(canonical.id.as_str(), "eb-legacy");
    assert_eq!(
        canonical
            .metadata
            .as_ref()
            .and_then(|metadata| metadata.get("candidate_id"))
            .and_then(|value| value.as_str()),
        Some("cand-legacy")
    );

    let local = row.local_bundle().unwrap();
    assert_eq!(local.bundle_id, "eb-legacy");
    assert_eq!(local.candidate_id, "cand-legacy");
    assert_eq!(
        local.assessment.unwrap().sample_support,
        forge_engine::SampleSupport::Sufficient
    );
}

// ── Experiment run CRUD ──

#[test]
fn experiment_run_insert() {
    let dir = TempDir::new().unwrap();
    let db_path = dir.path().join("forge.db");
    let store = ForgeStore::open(&db_path).unwrap();

    store
        .insert_experiment_run(
            "run-1",
            "c-1",
            "t-1",
            "trace-run",
            "Paired",
            "{}",
            "{}",
            "{}",
            r#"{"source_kind": "GitCommit"}"#,
            "[]",
        )
        .unwrap();
}

// ── Export receipt CRUD ──

#[test]
fn export_receipt_insert_and_check() {
    let dir = TempDir::new().unwrap();
    let db_path = dir.path().join("forge.db");
    let store = ForgeStore::open(&db_path).unwrap();

    assert!(!store.has_export_receipt("key-1").unwrap());

    let inserted = store
        .insert_export_receipt("key-1", "b-1", 1, "default", Some(true))
        .unwrap();
    assert!(inserted);

    assert!(store.has_export_receipt("key-1").unwrap());

    // Duplicate insert should be ignored
    let dup = store
        .insert_export_receipt("key-1", "b-1", 1, "default", Some(true))
        .unwrap();
    assert!(!dup);
}

// ── Run failure CRUD ──

#[test]
fn run_failure_insert_and_query() {
    let dir = TempDir::new().unwrap();
    let db_path = dir.path().join("forge.db");
    let store = ForgeStore::open(&db_path).unwrap();

    store
        .insert_run_failure(
            "f-1",
            "run-1",
            "BaselineTimeout",
            "timed out after 30s",
            "baseline_exec",
            true,
            0,
        )
        .unwrap();

    store
        .insert_run_failure(
            "f-2",
            "run-1",
            "PatchedCrash",
            "segfault",
            "patched_exec",
            false,
            0,
        )
        .unwrap();

    let failures = store.get_failures_for_run("run-1").unwrap();
    assert_eq!(failures.len(), 2);
    assert_eq!(failures[0].class, "BaselineTimeout");
    assert!(failures[0].retriable);
    assert_eq!(failures[1].class, "PatchedCrash");
    assert!(!failures[1].retriable);
}

#[test]
fn run_failure_empty_for_unknown_run() {
    let dir = TempDir::new().unwrap();
    let db_path = dir.path().join("forge.db");
    let store = ForgeStore::open(&db_path).unwrap();

    let failures = store.get_failures_for_run("no-such-run").unwrap();
    assert!(failures.is_empty());
}

// ── Verification plan CRUD ──

#[test]
fn verification_plan_insert() {
    let dir = TempDir::new().unwrap();
    let db_path = dir.path().join("forge.db");
    let store = ForgeStore::open(&db_path).unwrap();

    store
        .insert_verification_plan(
            "plan-1",
            "b-1",
            r#"["h1", "h2"]"#,
            r#"[{"verification_type": "Reproduce"}]"#,
            Some(r#"{"max_ablation_combinations": 16}"#),
        )
        .unwrap();
}

// ── FailureClass tests ──

#[test]
fn failure_class_retriability() {
    assert!(FailureClass::DbBusy.is_retriable());
    assert!(FailureClass::MemoryIngestUnavailable.is_retriable());
    assert!(FailureClass::PostReceiptCrash.is_retriable());
    assert!(FailureClass::BaselineTimeout.is_retriable());

    assert!(!FailureClass::PatchedCrash.is_retriable());
    assert!(!FailureClass::DuplicateCeaReplay.is_retriable());
    assert!(!FailureClass::WorkspaceSizeExceeded.is_retriable());
    assert!(!FailureClass::Other.is_retriable());
}

#[test]
fn failure_class_from_error() {
    use forge_engine::ForgeError;

    let timeout = ForgeError::CommandTimeout {
        command: "cargo test".into(),
        timeout_secs: 30,
    };
    assert_eq!(
        FailureClass::from_error(&timeout),
        FailureClass::BaselineTimeout
    );

    let other = ForgeError::Other("something".into());
    assert_eq!(FailureClass::from_error(&other), FailureClass::Other);
}

#[test]
fn failure_class_serde_round_trip() {
    let class = FailureClass::PostReceiptCrash;
    let json = serde_json::to_string(&class).unwrap();
    let back: FailureClass = serde_json::from_str(&json).unwrap();
    assert_eq!(back, class);
}

// ── Schema hash stability ──

#[test]
fn schema_hash_is_stable() {
    let h1 = schema::compute_schema_hash();
    let h2 = schema::compute_schema_hash();
    assert_eq!(h1, h2, "schema hash must be deterministic");
    assert_eq!(h1.len(), 64);
}

#[test]
fn schema_hash_matches_forge_schema_hash() {
    assert_eq!(
        schema::compute_schema_hash(),
        schema::forge_schema_hash(),
        "lazy-init and direct computation must agree"
    );
}
