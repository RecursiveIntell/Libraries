//! Demonstrates the current library stack in one app:
//! - Forge export envelope construction (`semantic-memory-forge`)
//! - Canonical envelope -> projection batch bridge (`forge-memory-bridge`)
//! - Projection ingestion and query projection APIs (`semantic-memory`)
//! - Runtime planning/classification/query/evidence lookup (`knowledge-runtime`)
//! - Queue lifecycle and frontend progress events (`tauri-queue`)

use chrono::{DateTime, Utc};
use forge_memory_bridge::{transform_envelope, PROJECTION_IMPORT_BATCH_V1_SCHEMA};
use knowledge_runtime::ids::{EntityId, ScopeKey};
use knowledge_runtime::{
    adapters::semantic_memory::SemanticMemoryAdapter,
    config::{EntityConfig, ProjectionConfig, QueryConfig},
    KnowledgeRuntime, QueryMode, QueryWarning, RoutePlan, RuntimeConfig, Scope,
};
use semantic_memory::{
    MemoryConfig, MemoryStore, MockEmbedder, ProjectionClaimVersion, ProjectionEpisode,
    ProjectionEvidenceRef, ProjectionQuery, ProjectionRelationVersion, SearchResult,
};
use semantic_memory_forge::{
    ExportClaim, ExportEntityAlias, ExportEnvelopeV1, ExportEpisode, ExportEvidenceRef,
    ExportRecord, ExportRelation, EXPORT_ENVELOPE_V1_SCHEMA,
};
use serde::{Deserialize, Serialize};
use stack_ids::EpisodeId;
use std::fs;
use std::path::PathBuf;
use std::time::Duration;
use tauri::{AppHandle, Manager, State};
use tauri_queue::{
    CoalescingEmitter, DropPolicy, EmitterConfig, JobContext, JobHandler, JobResult, QueueConfig,
    QueueError, QueueJob, QueueManager, TauriEventEmitter, TraceCtx,
};
use tokio::sync::Mutex;
use tokio::time::sleep;

#[cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
const DEMO_NAMESPACE: &str = "demo-stack";
const DEMO_ENVELOPE_ID: &str = "demo-envelope-v1";
const DEMO_FACT_SOURCE: &str = "demo-seed";
const DEMO_FACT_CONTENT: &str = "The demo auth layer can validate and refresh JWT sessions.";
const DEMO_EXPORTED_AT: &str = "2026-03-01T00:00:00Z";

type SharedState = std::sync::Arc<Mutex<DemoState>>;

#[derive(Default)]
struct DemoState {
    demo_dir: Option<PathBuf>,
    seeded_at: Option<DateTime<Utc>>,
    seeded_fact_id: Option<String>,
    imported_envelope_id: Option<String>,
    imported_record_count: usize,
    imported_status: Option<String>,
    memory: Option<MemoryStore>,
    runtime: Option<KnowledgeRuntime>,
    queue: Option<std::sync::Arc<QueueManager>>,
}

#[derive(Debug, Clone)]
struct DemoSeedOutcome {
    seeded_fact_id: String,
    import_envelope_id: String,
    import_record_count: usize,
    import_status: String,
}

#[derive(Debug, Serialize)]
struct InitResponse {
    namespace: String,
    initialized: bool,
    seeded_fact_id: String,
    import_envelope_id: String,
    import_record_count: usize,
    import_status: String,
    queue_ready: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct RuntimeSearchInput {
    query: String,
    limit: usize,
}

#[derive(Debug, Serialize)]
struct QueryResponse {
    input: String,
    scope: String,
    result_count: usize,
    warnings: Vec<QueryWarning>,
    trace_id: String,
    results: Vec<SearchResult>,
}

#[derive(Debug, Serialize, Deserialize)]
struct TemporalQueryInput {
    query: String,
    valid_at: String,
    recorded_at_or_before: String,
    limit: usize,
}

#[derive(Debug, Serialize)]
struct ProjectionRowSummary {
    text_query: Option<String>,
    claims: usize,
    claim_ids: Vec<String>,
    relations: usize,
    relation_ids: Vec<String>,
    episodes: usize,
    episode_ids: Vec<String>,
    aliases: usize,
    evidence_refs: usize,
    import_log_count: usize,
    latest_import_envelopes: Vec<String>,
}

#[derive(Debug, Serialize)]
struct QueueJobSubmission {
    job_id: String,
    query: String,
    steps: u32,
}

#[derive(Debug, Serialize)]
struct JobRow {
    id: String,
    status: String,
}

#[derive(Debug, Serialize)]
struct PlanResponse {
    query: String,
    mode: QueryMode,
    confidence: f32,
    reason: Option<String>,
    plan: RoutePlan,
    default_leg_count: usize,
}

#[derive(Debug, Serialize)]
struct MemorySearchResponse {
    query: String,
    namespace: String,
    result_count: usize,
    results: Vec<SearchResult>,
}

#[derive(Debug, Serialize)]
struct EvidenceResponse {
    claim_id: String,
    claim_version_id: Option<String>,
    records: Vec<ProjectionEvidenceRef>,
}

#[derive(Debug, Serialize)]
struct JobStatusResponse {
    jobs: Vec<JobRow>,
    queue_present: bool,
}

fn demo_not_initialized_err() -> String {
    "Demo not initialized. Call initialize_demo first.".to_string()
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct DemoIndexJob {
    query: String,
    steps: u32,
}

#[tauri::command]
async fn initialize_demo(
    state: State<'_, SharedState>,
    app_handle: AppHandle,
) -> Result<InitResponse, String> {
    let mut state = state.lock().await;

    if state.runtime.is_none() || state.memory.is_none() || state.queue.is_none() {
        initialize_stack_in_state(&mut state, &app_handle).await?;
    }

    Ok(InitResponse {
        namespace: DEMO_NAMESPACE.to_string(),
        initialized: true,
        seeded_fact_id: state
            .seeded_fact_id
            .clone()
            .unwrap_or_else(|| "unknown".to_string()),
        import_envelope_id: state
            .imported_envelope_id
            .clone()
            .unwrap_or_else(|| "none".to_string()),
        import_record_count: state.imported_record_count,
        import_status: state
            .imported_status
            .clone()
            .unwrap_or_else(|| "none".to_string()),
        queue_ready: state.queue.is_some(),
    })
}

#[tauri::command]
async fn run_knowledge_query(
    state: State<'_, SharedState>,
    input: RuntimeSearchInput,
) -> Result<QueryResponse, String> {
    let state = state.lock().await;
    let runtime = state
        .runtime
        .as_ref()
        .ok_or_else(demo_not_initialized_err)?;
    let scope = Scope::new(DEMO_NAMESPACE);

    let (results, trace) = runtime
        .query(&input.query, Some(&scope))
        .await
        .map_err(|e| format!("runtime query failed: {e}"))?;

    let limit = input.limit.min(results.len());
    let results: Vec<SearchResult> = results.into_iter().take(limit).collect();

    Ok(QueryResponse {
        input: input.query,
        scope: scope.key().to_string(),
        result_count: results.len(),
        warnings: trace.warnings,
        trace_id: trace.trace_ctx.trace_id,
        results,
    })
}

#[tauri::command]
async fn run_temporal_query(
    state: State<'_, SharedState>,
    input: TemporalQueryInput,
) -> Result<QueryResponse, String> {
    let state = state.lock().await;
    let runtime = state
        .runtime
        .as_ref()
        .ok_or_else(demo_not_initialized_err)?;
    let scope = Scope::new(DEMO_NAMESPACE);

    let (results, trace) = runtime
        .query_temporal(
            &input.query,
            Some(&scope),
            &input.valid_at,
            &input.recorded_at_or_before,
        )
        .await
        .map_err(|e| format!("runtime temporal query failed: {e}"))?;

    let limit = input.limit.min(results.len());
    let results: Vec<SearchResult> = results.into_iter().take(limit).collect();

    Ok(QueryResponse {
        input: input.query,
        scope: scope.key().to_string(),
        result_count: results.len(),
        warnings: trace.warnings,
        trace_id: trace.trace_ctx.trace_id,
        results,
    })
}

#[tauri::command]
async fn run_plan_preview(
    state: State<'_, SharedState>,
    input: RuntimeSearchInput,
) -> Result<PlanResponse, String> {
    let state = state.lock().await;
    let runtime = state
        .runtime
        .as_ref()
        .ok_or_else(demo_not_initialized_err)?;
    let scope = Scope::new(DEMO_NAMESPACE);
    let classification = runtime.classify(&input.query);
    let plan = runtime.plan(&input.query, Some(&scope));

    let route_lens = plan.legs.iter().map(|leg| &leg.strategy).count();

    Ok(PlanResponse {
        query: input.query,
        mode: classification.mode,
        confidence: classification.confidence,
        reason: classification.reason,
        plan,
        default_leg_count: route_lens,
    })
}

#[tauri::command]
async fn run_memory_query(
    state: State<'_, SharedState>,
    input: RuntimeSearchInput,
) -> Result<MemorySearchResponse, String> {
    let state = state.lock().await;
    let memory = state.memory.as_ref().ok_or_else(demo_not_initialized_err)?;
    let names = [&DEMO_NAMESPACE as &str];
    let limit = input.limit.max(1);
    let results = memory
        .search(&input.query, Some(limit), Some(&names[..]), None)
        .await
        .map_err(|e| format!("semantic-memory search failed: {e}"))?;

    Ok(MemorySearchResponse {
        query: input.query,
        namespace: DEMO_NAMESPACE.to_string(),
        result_count: results.len(),
        results,
    })
}

#[tauri::command]
async fn run_projection_summary(
    state: State<'_, SharedState>,
    text_query: Option<String>,
) -> Result<ProjectionRowSummary, String> {
    let state = state.lock().await;
    let memory = state.memory.as_ref().ok_or_else(demo_not_initialized_err)?;

    let scope_key = ScopeKey::namespace_only(DEMO_NAMESPACE);
    let mut query = ProjectionQuery::new(scope_key);
    query.limit = 20;
    query.text_query = text_query.clone();

    let claims: Vec<ProjectionClaimVersion> = memory
        .query_claim_versions(query.clone())
        .await
        .map_err(|e| format!("claim projection query failed: {e}"))?;
    let relations: Vec<ProjectionRelationVersion> = memory
        .query_relation_versions(query.clone())
        .await
        .map_err(|e| format!("relation projection query failed: {e}"))?;
    let episodes: Vec<ProjectionEpisode> = memory
        .query_episodes(query.clone())
        .await
        .map_err(|e| format!("episode projection query failed: {e}"))?;
    let aliases = memory
        .query_entity_aliases(query.clone())
        .await
        .map_err(|e| format!("alias projection query failed: {e}"))?;
    let evidence_refs = memory
        .query_evidence_refs(query)
        .await
        .map_err(|e| format!("evidence projection query failed: {e}"))?;
    let imports = memory
        .query_projection_imports(Some(DEMO_NAMESPACE), 5)
        .await
        .map_err(|e| format!("projection import log query failed: {e}"))?;

    Ok(ProjectionRowSummary {
        text_query: text_query,
        claims: claims.len(),
        claim_ids: claims
            .iter()
            .map(|c| c.claim_version_id.to_string())
            .take(5)
            .collect(),
        relations: relations.len(),
        relation_ids: relations
            .iter()
            .map(|r| r.relation_version_id.to_string())
            .take(5)
            .collect(),
        episodes: episodes.len(),
        episode_ids: episodes
            .iter()
            .map(|e| e.episode_id.to_string())
            .take(5)
            .collect(),
        aliases: aliases.len(),
        evidence_refs: evidence_refs.len(),
        import_log_count: imports.len(),
        latest_import_envelopes: imports.into_iter().map(|i| i.source_envelope_id).collect(),
    })
}

#[tauri::command]
async fn query_claim_evidence(
    state: State<'_, SharedState>,
    claim_id: String,
    claim_version_id: Option<String>,
    limit: usize,
) -> Result<EvidenceResponse, String> {
    let state = state.lock().await;
    let runtime = state
        .runtime
        .as_ref()
        .ok_or_else(demo_not_initialized_err)?;

    let scope = Scope::new(DEMO_NAMESPACE);
    let records = runtime
        .query_evidence_refs_for_claim(&claim_id, claim_version_id.as_deref(), Some(&scope), limit)
        .await
        .map_err(|e| format!("evidence query failed: {e}"))?;

    Ok(EvidenceResponse {
        claim_id,
        claim_version_id,
        records,
    })
}

#[tauri::command]
async fn enqueue_demo_job(
    state: State<'_, SharedState>,
    query: String,
    steps: u32,
) -> Result<QueueJobSubmission, String> {
    let queue = {
        let state = state.lock().await;
        state.queue.clone().ok_or_else(demo_not_initialized_err)?
    };

    let steps = steps.max(1);
    let job = QueueJob::new(DemoIndexJob {
        query: query.clone(),
        steps,
    });
    let job_id = queue
        .add(job)
        .map_err(|e: QueueError| format!("enqueue failed: {e}"))?;

    Ok(QueueJobSubmission {
        job_id,
        query,
        steps,
    })
}

#[tauri::command]
async fn list_demo_jobs(state: State<'_, SharedState>) -> Result<JobStatusResponse, String> {
    let state = state.lock().await;
    let jobs = state
        .queue
        .as_ref()
        .map(|queue| {
            queue
                .list_jobs()
                .map(|jobs| {
                    jobs.into_iter()
                        .map(|(id, status)| JobRow { id, status })
                        .collect::<Vec<_>>()
                })
                .map_err(|e| e.to_string())
        })
        .transpose()?
        .unwrap_or_default();

    Ok(JobStatusResponse {
        jobs,
        queue_present: state.queue.is_some(),
    })
}

#[tauri::command]
async fn cancel_demo_job(state: State<'_, SharedState>, job_id: String) -> Result<String, String> {
    let queue = {
        let state = state.lock().await;
        state.queue.clone().ok_or_else(demo_not_initialized_err)?
    };

    queue
        .cancel(&job_id)
        .map_err(|e: QueueError| format!("cancel failed: {e}"))?;

    Ok(format!("cancel requested for job {job_id}"))
}

#[allow(clippy::too_many_lines)]
async fn initialize_stack_in_state(
    state: &mut DemoState,
    app_handle: &AppHandle,
) -> Result<(), String> {
    let app_data = app_handle
        .path()
        .app_data_dir()
        .unwrap_or_else(|_| std::env::temp_dir().join("demo-tauri-libraries"));
    let demo_root = app_data.join("libraries-demo");
    let memory_dir = demo_root.join("memory");
    let queue_db = demo_root.join("queue.sqlite");
    fs::create_dir_all(&memory_dir).map_err(|e| format!("failed to create demo data dir: {e}"))?;

    let mut memory_config = MemoryConfig::default();
    memory_config.base_dir = memory_dir.clone();
    let memory = MemoryStore::open_with_embedder(memory_config, Box::new(MockEmbedder::new(768)))
        .map_err(|e| format!("open memory failed: {e}"))?;

    let demo_seed = seed_demo_memory(&memory).await?;

    let runtime_config = RuntimeConfig {
        default_scope: Scope::new(DEMO_NAMESPACE),
        query: QueryConfig::default(),
        entity: EntityConfig::default(),
        projection: ProjectionConfig::default(),
        strict_temporal: false,
        strict_scope: false,
    };
    let adapter = SemanticMemoryAdapter::new(memory.clone());
    let runtime = KnowledgeRuntime::new(runtime_config, adapter)
        .map_err(|e| format!("runtime init failed: {e}"))?;

    let queue_config = QueueConfig::builder()
        .with_db_path(queue_db)
        .with_max_consecutive(2)
        .with_cooldown(Duration::from_millis(100))
        .build();
    let mut emitter_config = EmitterConfig::default();
    emitter_config.buffer_size = 64;
    emitter_config.drop_policy = DropPolicy::DropNewest;
    emitter_config.coalesce_interval_ms = 50;
    emitter_config.include_trace_ctx = true;

    let queue = QueueManager::new(queue_config)
        .map_err(|e: QueueError| format!("queue init failed: {e}"))?
        .spawn::<DemoIndexJob>(CoalescingEmitter::arc(
            TauriEventEmitter::arc(app_handle.clone()),
            emitter_config,
        ));

    state.demo_dir = Some(demo_root);
    state.seeded_at = Some(Utc::now());
    state.seeded_fact_id = Some(demo_seed.seeded_fact_id.clone());
    state.imported_envelope_id = Some(demo_seed.import_envelope_id.clone());
    state.imported_record_count = demo_seed.import_record_count;
    state.imported_status = Some(demo_seed.import_status.clone());
    state.memory = Some(memory);
    state.runtime = Some(runtime);
    state.queue = Some(queue);

    Ok(())
}

async fn seed_demo_memory(memory: &MemoryStore) -> Result<DemoSeedOutcome, String> {
    let seeded_fact_id = ensure_demo_seeded_fact(memory).await?;

    if let Some(existing_import) = find_existing_demo_import(memory).await? {
        return Ok(DemoSeedOutcome {
            seeded_fact_id,
            import_envelope_id: existing_import.source_envelope_id,
            import_record_count: existing_import.record_count,
            import_status: format!("{} (reused existing import)", existing_import.status),
        });
    }

    let envelope = build_demo_envelope()?;
    let batch =
        transform_envelope(&envelope).map_err(|e| format!("transform envelope failed: {e}"))?;
    assert_eq!(batch.schema_version, PROJECTION_IMPORT_BATCH_V1_SCHEMA);

    let import = memory
        .import_projection_batch(&batch)
        .await
        .map_err(|e| format!("projection import failed: {e}"))?;

    Ok(DemoSeedOutcome {
        seeded_fact_id,
        import_envelope_id: envelope.envelope_id.to_string(),
        import_record_count: import.record_count,
        import_status: import.status,
    })
}

async fn ensure_demo_seeded_fact(memory: &MemoryStore) -> Result<String, String> {
    let existing_facts = memory
        .list_facts(DEMO_NAMESPACE, 128, 0)
        .await
        .map_err(|e| format!("list facts failed: {e}"))?;

    if let Some(existing_fact) = existing_facts.into_iter().find(|fact| {
        fact.source.as_deref() == Some(DEMO_FACT_SOURCE) && fact.content == DEMO_FACT_CONTENT
    }) {
        return Ok(existing_fact.id);
    }

    memory
        .add_fact(
            DEMO_NAMESPACE,
            DEMO_FACT_CONTENT,
            Some(DEMO_FACT_SOURCE),
            Some(serde_json::json!({
                "seeded_by": "demo-tauri-libraries",
                "source": "direct-fact-demo"
            })),
        )
        .await
        .map_err(|e| format!("seed fact failed: {e}"))
}

async fn find_existing_demo_import(
    memory: &MemoryStore,
) -> Result<Option<semantic_memory::ProjectionImportLogEntry>, String> {
    let imports = memory
        .query_projection_imports(Some(DEMO_NAMESPACE), 64)
        .await
        .map_err(|e| format!("query projection imports failed: {e}"))?;

    Ok(imports
        .into_iter()
        .find(|entry| entry.source_envelope_id == DEMO_ENVELOPE_ID))
}

fn build_demo_envelope() -> Result<ExportEnvelopeV1, String> {
    let scope_key = ScopeKey::namespace_only(DEMO_NAMESPACE);
    let exported_at = DEMO_EXPORTED_AT.to_string();

    let records = vec![
        ExportRecord::Claim(ExportClaim {
            claim_id: Some("claim-auth".into()),
            claim_version_id: None,
            subject_entity_id: EntityId::new("entity-auth-service"),
            predicate: "can_validate".to_string(),
            object_anchor: serde_json::json!("oauth jwts"),
            valid_from: Some(exported_at.clone()),
            valid_to: None,
            confidence: 0.97,
            content: "Auth service can validate OAuth JWT sessions".to_string(),
            projection_family: "demo-projection".to_string(),
            supersedes_claim_id: None,
            supersedes_claim_version_id: None,
            metadata: Some(serde_json::json!({
                "kind": "seed",
                "bundle": "stack-demo",
            })),
        }),
        ExportRecord::Relation(ExportRelation {
            relation_version_id: None,
            subject_entity_id: EntityId::new("entity-auth-service"),
            predicate: "integrates_with".to_string(),
            object_anchor: serde_json::json!("entity-identity-provider"),
            valid_from: Some(exported_at.clone()),
            valid_to: None,
            confidence: 0.89,
            projection_family: "demo-projection".to_string(),
            source_claim_id: Some("claim-auth".into()),
            source_episode_id: Some(EpisodeId::new("episode-initial".to_string())),
            supersedes_relation_version_id: None,
            metadata: Some(serde_json::json!({
                "kind": "seed",
                "bundle": "stack-demo",
            })),
        }),
        ExportRecord::Episode(ExportEpisode {
            episode_id: Some("episode-initial".into()),
            document_id: "doc-auth-integration".to_string(),
            cause_ids: vec!["claim-auth".into()],
            effect_type: "verification".to_string(),
            outcome: "pass".to_string(),
            confidence: 0.95,
            experiment_id: Some("exp-101".to_string()),
            metadata: Some(serde_json::json!({
                "seed_type": "integration_check",
                "owner": "demo-ui",
            })),
        }),
        ExportRecord::EntityAlias(ExportEntityAlias {
            canonical_entity_id: EntityId::new("entity-auth-service"),
            alias_text: "Demo Authentication Service".to_string(),
            alias_source: "forge".to_string(),
            match_evidence: None,
            confidence: 0.98,
            scope: Some(scope_key.clone()),
            superseded_by_entity_id: None,
            split_from_entity_id: None,
        }),
        ExportRecord::EvidenceRef(ExportEvidenceRef {
            claim_id: "claim-auth".into(),
            claim_version_id: None,
            fetch_handle: "forge://evidence/auth/seed-1".to_string(),
            source_authority: "forge-demo".to_string(),
            metadata: Some(serde_json::json!({
                "seeded": true,
            })),
        }),
    ];

    let digest = ExportEnvelopeV1::compute_digest("forge-demo", &scope_key, &records)
        .map_err(|e| format!("invalid seed payload: {e}"))?;

    Ok(ExportEnvelopeV1 {
        envelope_id: DEMO_ENVELOPE_ID.into(),
        schema_version: EXPORT_ENVELOPE_V1_SCHEMA.to_string(),
        content_digest: digest,
        source_authority: "forge-demo".to_string(),
        scope_key,
        trace_ctx: Some(TraceCtx::generate()),
        exported_at,
        records,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_memory_store() -> MemoryStore {
        let unique_suffix = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock should be after unix epoch")
            .as_nanos();
        let test_dir = std::env::temp_dir().join(format!(
            "demo-tauri-libraries-seed-{unique_suffix}-{}",
            std::process::id()
        ));
        fs::create_dir_all(&test_dir).expect("test memory dir should be creatable");

        let mut memory_config = MemoryConfig::default();
        memory_config.base_dir = test_dir;
        MemoryStore::open_with_embedder(memory_config, Box::new(MockEmbedder::new(768)))
            .expect("test memory store should open")
    }

    #[tokio::test]
    async fn demo_seed_is_restart_safe() {
        let memory = test_memory_store();

        let first = seed_demo_memory(&memory)
            .await
            .expect("first demo seed should succeed");
        let second = seed_demo_memory(&memory)
            .await
            .expect("second demo seed should reuse the existing demo data");

        let facts = memory
            .list_facts(DEMO_NAMESPACE, 128, 0)
            .await
            .expect("demo facts should be queryable");
        let demo_facts: Vec<_> = facts
            .into_iter()
            .filter(|fact| fact.source.as_deref() == Some(DEMO_FACT_SOURCE))
            .collect();

        let imports = memory
            .query_projection_imports(Some(DEMO_NAMESPACE), 64)
            .await
            .expect("demo imports should be queryable");
        let demo_imports: Vec<_> = imports
            .into_iter()
            .filter(|entry| entry.source_envelope_id == DEMO_ENVELOPE_ID)
            .collect();

        assert_eq!(first.seeded_fact_id, second.seeded_fact_id);
        assert_eq!(demo_facts.len(), 1, "demo fact should be seeded once");
        assert_eq!(demo_imports.len(), 1, "demo import should be reused");
        assert_eq!(second.import_envelope_id, DEMO_ENVELOPE_ID);
        assert!(
            second.import_status.contains("reused existing import"),
            "second seed should report that it reused the existing import"
        );
    }

    #[test]
    fn demo_envelope_export_timestamp_is_stable() {
        let first = build_demo_envelope().expect("first demo envelope should build");
        let second = build_demo_envelope().expect("second demo envelope should build");

        assert_eq!(first.exported_at, DEMO_EXPORTED_AT);
        assert_eq!(second.exported_at, DEMO_EXPORTED_AT);
        assert_eq!(first.content_digest, second.content_digest);
    }
}

#[tokio::main]
async fn main() {
    let shared_state: SharedState = std::sync::Arc::new(Mutex::new(DemoState::default()));

    let _ = tauri::Builder::default()
        .manage(shared_state)
        .invoke_handler(tauri::generate_handler![
            initialize_demo,
            run_knowledge_query,
            run_temporal_query,
            run_plan_preview,
            run_memory_query,
            run_projection_summary,
            query_claim_evidence,
            enqueue_demo_job,
            list_demo_jobs,
            cancel_demo_job,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri app");
}

impl JobHandler for DemoIndexJob {
    async fn execute(&self, ctx: &JobContext) -> Result<JobResult, QueueError> {
        let total = self.steps.max(1);
        for step in 0..total {
            if ctx.is_cancelled() {
                return Ok(JobResult::failure("job cancelled".into()));
            }
            ctx.emit_progress(step, total);
            sleep(Duration::from_millis(250)).await;
        }

        Ok(JobResult::success_with_output(format!(
            "Indexed query '{}' at {} step(s)",
            self.query, self.steps
        )))
    }
}
