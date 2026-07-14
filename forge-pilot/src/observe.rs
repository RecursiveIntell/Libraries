//! Observation phase of the OODA loop.
//!
//! Inspects workspace paths, semantic-memory store contents, and kernel
//! compilation state to produce a snapshot that the orient and decide
//! phases consume.

use crate::bootstrap::{
    current_state_from_latest_manifest, is_auxiliary_bootstrap_claim,
    is_supported_source_file_path, should_skip_workspace_dir,
};
use crate::config::LoopConfig;
use crate::error::PilotError;
use crate::{BootstrapCurrentState, BootstrapRichness};
use chrono::{DateTime, Utc};
use constraint_compiler::{compile_batch, CompileOutput, CompilerPolicy, ConstraintDegradation};
use forge_engine::ForgeStore;
use forge_memory_bridge::ProjectionImportBatchV3;
use kernel_execution::{schedule_execution, ExecutionBudget, ScheduledExecution};
use kernel_oracles::{
    evaluate_causal_refuter, evaluate_conservative, evaluate_exact_bounded,
    evaluate_minimal_perturbation, OracleAssessment, TemporalReplaySnapshot,
};
use knowledge_runtime::{
    InferenceAdvisory, InferenceExplanation, KnowledgeRuntime, RiskGateDecision, Scope,
};
use schemars::JsonSchema;
use semantic_memory::{
    MemoryStore, ProjectionClaimVersion, ProjectionEntityAlias, ProjectionEpisode,
    ProjectionEvidenceRef, ProjectionImportLogEntry, ProjectionQuery, ProjectionRelationVersion,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

/// A degradation detected during scope observation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ObservationDegradation {
    pub kind: String,
    pub detail: String,
}

/// Whether a filesystem path required for observation is present and valid.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum PathAvailability {
    Present,
    Missing,
    Invalid,
}

/// Disposition of the most recent canonical import record for the queried namespace.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ImportRecordDisposition {
    Found,
    NeverImported,
    NamespaceMismatch,
    StorageUnavailable,
    StorageCorrupt,
}

/// Overall readiness disposition of the observation for a namespace.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ObservationDisposition {
    Ready,
    EmptyWorkspace,
    ImportRequired,
    NamespaceMismatch,
    StorageUnavailable,
    StorageCorrupt,
}

/// Resolved filesystem paths and their availability for a configured namespace.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ObservationPaths {
    pub namespace: String,
    pub workspace_path: String,
    pub memory_dir: String,
    pub forge_db_path: String,
    pub memory_dir_state: PathAvailability,
    pub forge_db_state: PathAvailability,
}

/// Count of supported source files and imported state for the workspace.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct SourceInventory {
    pub supported_file_count: usize,
    pub imported_current_state: BootstrapCurrentState,
}

/// Count of Forge evidence bundles available for export.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ForgeEvidenceInventory {
    pub available_bundle_count: usize,
}

/// Machine-readable status summary for a namespace observation.
///
/// Includes import disposition, file counts, and a human-readable
/// `exact_next_step` describing what the operator should do next.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ObservationStatus {
    pub disposition: ObservationDisposition,
    pub namespace_queried: String,
    pub resolved_workspace_path: String,
    pub supported_file_count: usize,
    pub imported_file_count: usize,
    pub imported_chunk_count: usize,
    pub imported_symbol_count: usize,
    pub bootstrap_richness: BootstrapRichness,
    pub available_forge_bundle_count: usize,
    pub import_record_disposition: ImportRecordDisposition,
    pub import_records_found: bool,
    pub exact_next_step: String,
    #[serde(default)]
    pub other_import_namespaces: Vec<String>,
}

/// Aggregate health metrics for a projected scope within semantic-memory.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ScopeHealthSummary {
    pub total_claim_versions: usize,
    pub total_relation_versions: usize,
    pub total_episodes: usize,
    pub fragile_node_count: usize,
    pub syndrome_count: usize,
    pub degradation_count: usize,
    pub unverified_claim_count: usize,
    pub supersession_candidate_count: usize,
    pub last_import_at: Option<String>,
}

/// Full observation snapshot produced by the observe phase.
///
/// Contains every input the orient, decide, and act phases need:
/// paths, source inventory, kernel compilation output, oracle assessment,
/// claim/relation/episode projections, and scope health.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Observation {
    pub scope_key: stack_ids::ScopeKey,
    pub paths: ObservationPaths,
    pub source_inventory: SourceInventory,
    pub status: ObservationStatus,
    pub advisory: Option<InferenceAdvisory>,
    pub explanation: Option<InferenceExplanation>,
    pub risk_gate: Option<RiskGateDecision>,
    pub import_log: Option<ProjectionImportLogEntry>,
    pub recent_imports: Vec<ProjectionImportLogEntry>,
    pub batch: Option<ProjectionImportBatchV3>,
    pub compiled: Option<CompileOutput>,
    pub scheduled: Option<ScheduledExecution>,
    pub oracle: Option<OracleAssessment>,
    pub causal_refutation: Option<kernel_oracles::OracleRefutationResult>,
    pub minimal_perturbation: Option<kernel_oracles::OracleRefutationResult>,
    pub temporal_snapshots: Vec<TemporalReplaySnapshot>,
    pub claim_versions: Vec<ProjectionClaimVersion>,
    pub relation_versions: Vec<ProjectionRelationVersion>,
    pub episodes: Vec<ProjectionEpisode>,
    pub aliases: Vec<ProjectionEntityAlias>,
    pub evidence_refs: Vec<ProjectionEvidenceRef>,
    pub scope_health: ScopeHealthSummary,
    pub degradations: Vec<ObservationDegradation>,
    #[cfg(feature = "governance")]
    pub governance: Option<crate::governance_gate::GovernanceObservation>,
}

impl Observation {
    /// Returns true when the observation is missing kernel payload needed for execution.
    pub fn missing_kernel_payload(&self) -> bool {
        self.degradations
            .iter()
            .any(|degradation| degradation.kind == "missing_kernel_payload")
    }

    /// Returns true when the observation is constrained to a thin-export lane.
    pub fn thin_export_active(&self) -> bool {
        self.degradations
            .iter()
            .any(|degradation| degradation.kind == "thin_export")
    }
}

/// Builds a complete observation for the configured scope.
///
/// Queries the semantic-memory store, compiles constraints, runs kernel
/// oracle evaluation, and assembles the full `Observation` snapshot.
pub async fn observe_scope(
    runtime: &KnowledgeRuntime,
    store: &MemoryStore,
    config: &LoopConfig,
) -> Result<Observation, PilotError> {
    let scope = &config.scope;
    let scope_key = scope.key();
    let paths = inspect_observation_paths(config);
    let mut source_inventory = discover_source_inventory(config, &paths);
    let forge_evidence_inventory = discover_forge_evidence_inventory(config, &paths);
    let advisory = runtime.latest_inference_advisory(Some(scope)).await?;
    let explanation = runtime.latest_inference_explanation(Some(scope)).await?;
    let risk_gate = runtime.latest_risk_gate(Some(scope)).await?;

    let recent_imports = store
        .query_projection_imports(Some(&scope.namespace), config.observation_import_limit)
        .await?;
    let all_recent_imports = store
        .query_projection_imports(None, config.observation_import_limit)
        .await?;
    let import_log = recent_imports
        .iter()
        .find(|row| row.status == "complete")
        .cloned();
    source_inventory.imported_current_state = current_state_from_latest_manifest(&recent_imports);
    let status = build_observation_status(
        scope,
        &paths,
        &source_inventory,
        &forge_evidence_inventory,
        &recent_imports,
        &all_recent_imports,
        import_log.is_some(),
    );

    let mut degradations = Vec::new();
    let batch = match import_log
        .as_ref()
        .and_then(|row| row.kernel_payload_json.clone())
    {
        Some(kernel_payload_json) => {
            match serde_json::from_value::<ProjectionImportBatchV3>(kernel_payload_json) {
                Ok(batch) => Some(batch),
                Err(error) => {
                    degradations.push(ObservationDegradation {
                        kind: "invalid_kernel_payload".into(),
                        detail: error.to_string(),
                    });
                    None
                }
            }
        }
        None => {
            if import_log.is_some() {
                degradations.push(ObservationDegradation {
                    kind: "missing_kernel_payload".into(),
                    detail: scope.namespace.clone(),
                });
            }
            None
        }
    };

    let (compiled, scheduled, oracle, causal_refutation, minimal_perturbation) =
        if let Some(batch) = &batch {
            let compiled = compile_batch(
                batch,
                &CompilerPolicy {
                    policy_version: "forge-pilot.v1".into(),
                    include_hyperedges: config.include_hyperedges,
                },
            );
            for marker in &compiled.degradations {
                degradations.push(ObservationDegradation {
                    kind: degradation_kind_name(marker).to_string(),
                    detail: format!("{marker:?}"),
                });
            }
            let scheduled = schedule_execution(
                &compiled,
                &ExecutionBudget {
                    max_iterations: config.oracle_max_iterations.max(1),
                    ..ExecutionBudget::default()
                },
            );
            let oracle = evaluate_exact_bounded(&compiled)
                .unwrap_or_else(|| evaluate_conservative(&compiled));
            let primary_node_id = scheduled
                .execution
                .witnesses
                .first()
                .map(|witness| witness.node_id.clone())
                .or_else(|| {
                    compiled
                        .nodes
                        .iter()
                        .find(|node| node.kind != "nuisance_state")
                        .map(|node| node.node_id.clone())
                })
                .unwrap_or_else(|| "unsupported".into());
            let causal_refutation = evaluate_causal_refuter(
                &compiled,
                &primary_node_id,
                config.minimal_perturbation_budget,
            );
            let minimal_perturbation = evaluate_minimal_perturbation(
                &compiled,
                &primary_node_id,
                config.minimal_perturbation_budget,
            );
            (
                Some(compiled),
                Some(scheduled),
                Some(oracle),
                Some(causal_refutation),
                Some(minimal_perturbation),
            )
        } else {
            (None, None, None, None, None)
        };

    let temporal_snapshots = build_temporal_snapshots(&recent_imports);

    let mut projection_query = ProjectionQuery::new(scope_key.clone());
    projection_query.limit = config.observation_projection_limit;
    let mut claim_versions = store.query_claim_versions(projection_query.clone()).await?;
    let relation_versions = store
        .query_relation_versions(projection_query.clone())
        .await?;
    let episodes = store.query_episodes(projection_query.clone()).await?;
    let aliases = store.query_entity_aliases(projection_query.clone()).await?;
    let evidence_refs = store.query_evidence_refs(projection_query).await?;
    claim_versions.retain(|claim| !is_auxiliary_bootstrap_claim(claim));

    let scope_health = build_scope_health_summary(
        &claim_versions,
        &relation_versions,
        &episodes,
        scheduled.as_ref(),
        compiled.as_ref(),
        import_log.as_ref(),
        &config
            .runtime_config
            .projection
            .import_staleness_threshold_secs,
    );

    if matches!(status.disposition, ObservationDisposition::Ready)
        && is_scope_stale(
            scope,
            import_log.as_ref(),
            config
                .runtime_config
                .projection
                .import_staleness_threshold_secs,
        )
    {
        degradations.push(ObservationDegradation {
            kind: "scope_stale".into(),
            detail: import_log
                .as_ref()
                .map(|row| row.imported_at.clone())
                .unwrap_or_else(|| "never_imported".into()),
        });
    }

    Ok(Observation {
        scope_key,
        paths,
        source_inventory,
        status,
        advisory,
        explanation,
        risk_gate,
        import_log,
        recent_imports,
        batch,
        compiled,
        scheduled,
        oracle,
        causal_refutation,
        minimal_perturbation,
        temporal_snapshots,
        claim_versions,
        relation_versions,
        episodes,
        aliases,
        evidence_refs,
        scope_health,
        degradations,
        #[cfg(feature = "governance")]
        governance: match crate::governance_gate::observe_governance(store).await {
            Ok(obs) => Some(obs),
            Err(e) => {
                tracing::warn!(error = %e, "governance observation failed in observe_scope");
                None
            }
        },
    })
}

fn build_temporal_snapshots(imports: &[ProjectionImportLogEntry]) -> Vec<TemporalReplaySnapshot> {
    let mut snapshots = imports
        .iter()
        .filter_map(|row| {
            row.kernel_payload_json.clone().and_then(|payload| {
                serde_json::from_value::<ProjectionImportBatchV3>(payload)
                    .ok()
                    .map(|batch| TemporalReplaySnapshot {
                        recorded_at: row.imported_at.clone(),
                        batch,
                    })
            })
        })
        .collect::<Vec<_>>();
    snapshots.sort_by(|left, right| left.recorded_at.cmp(&right.recorded_at));
    snapshots
}

fn build_scope_health_summary(
    claim_versions: &[ProjectionClaimVersion],
    relation_versions: &[ProjectionRelationVersion],
    episodes: &[ProjectionEpisode],
    scheduled: Option<&ScheduledExecution>,
    compiled: Option<&CompileOutput>,
    import_log: Option<&ProjectionImportLogEntry>,
    _staleness_threshold_secs: &u64,
) -> ScopeHealthSummary {
    let fragile_node_count = scheduled
        .map(|scheduled| {
            scheduled
                .execution
                .node_beliefs
                .iter()
                .filter(|belief| belief.belief_micros < 900_000)
                .count()
        })
        .unwrap_or(0);
    let syndrome_count = scheduled
        .map(|scheduled| scheduled.execution.syndromes.len())
        .unwrap_or(0);
    let degradation_count = compiled
        .map(|compiled| compiled.degradations.len())
        .unwrap_or(0);
    let unverified_claim_count = claim_versions
        .iter()
        .filter(|claim| claim.claim_state == "pending_review")
        .count();
    let supersession_candidate_count = claim_versions
        .iter()
        .filter(|claim| claim.supersedes_claim_version_id.is_some())
        .count();

    ScopeHealthSummary {
        total_claim_versions: claim_versions.len(),
        total_relation_versions: relation_versions.len(),
        total_episodes: episodes.len(),
        fragile_node_count,
        syndrome_count,
        degradation_count,
        unverified_claim_count,
        supersession_candidate_count,
        last_import_at: import_log.map(|row| row.imported_at.clone()),
    }
}

fn is_scope_stale(
    _scope: &Scope,
    import_log: Option<&ProjectionImportLogEntry>,
    threshold_secs: u64,
) -> bool {
    if threshold_secs == 0 {
        return false;
    }
    let Some(import_log) = import_log else {
        return true;
    };
    let Ok(imported_at) = DateTime::parse_from_rfc3339(&import_log.imported_at) else {
        return false;
    };
    let age_secs = Utc::now()
        .signed_duration_since(imported_at.with_timezone(&Utc))
        .num_seconds();
    age_secs >= threshold_secs as i64
}

fn degradation_kind_name(marker: &ConstraintDegradation) -> &'static str {
    match marker {
        ConstraintDegradation::MissingClaimFamily => "missing_claim_family",
        ConstraintDegradation::MissingAssertionGroup => "missing_assertion_group",
        ConstraintDegradation::MissingRelationGroup => "missing_relation_group",
        ConstraintDegradation::ThinExport => "thin_export",
    }
}

/// Inspects the configured observation paths without opening the underlying stores.
pub fn inspect_observation_paths(config: &LoopConfig) -> ObservationPaths {
    ObservationPaths {
        namespace: config.scope.namespace.clone(),
        workspace_path: resolve_display_path(&config.workspace_path),
        memory_dir: resolve_display_path(&config.memory_dir),
        forge_db_path: resolve_display_path(&config.forge_db_path),
        memory_dir_state: inspect_memory_dir(&config.memory_dir),
        forge_db_state: inspect_forge_db(&config.forge_db_path),
    }
}

fn build_observation_status(
    scope: &Scope,
    paths: &ObservationPaths,
    source_inventory: &SourceInventory,
    forge_evidence_inventory: &ForgeEvidenceInventory,
    recent_imports: &[ProjectionImportLogEntry],
    all_recent_imports: &[ProjectionImportLogEntry],
    completed_import_found: bool,
) -> ObservationStatus {
    let import_records_found = !recent_imports.is_empty();
    let other_import_namespaces = all_recent_imports
        .iter()
        .filter_map(|row| {
            if row.scope_namespace != scope.namespace {
                Some(row.scope_namespace.clone())
            } else {
                None
            }
        })
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let base = |disposition: ObservationDisposition,
                import_record_disposition: ImportRecordDisposition,
                exact_next_step: String| ObservationStatus {
        disposition,
        namespace_queried: scope.namespace.clone(),
        resolved_workspace_path: paths.workspace_path.clone(),
        supported_file_count: source_inventory.supported_file_count,
        imported_file_count: source_inventory.imported_current_state.file_count,
        imported_chunk_count: source_inventory.imported_current_state.chunk_count,
        imported_symbol_count: source_inventory.imported_current_state.symbol_count,
        bootstrap_richness: source_inventory.imported_current_state.richness,
        available_forge_bundle_count: forge_evidence_inventory.available_bundle_count,
        import_record_disposition,
        import_records_found,
        exact_next_step,
        other_import_namespaces: other_import_namespaces.clone(),
    };

    if matches!(paths.forge_db_state, PathAvailability::Invalid) {
        return base(
            ObservationDisposition::StorageCorrupt,
            ImportRecordDisposition::StorageCorrupt,
            format!(
                "Repair the configured canonical storage before running the pilot. Forge DB: {} ({:?}); memory folder: {} ({:?}).",
                paths.forge_db_path,
                paths.forge_db_state,
                paths.memory_dir,
                paths.memory_dir_state
            ),
        );
    }

    if matches!(paths.forge_db_state, PathAvailability::Missing) {
        return base(
            ObservationDisposition::StorageUnavailable,
            ImportRecordDisposition::StorageUnavailable,
            format!(
                "Fix or bootstrap the configured canonical storage paths before running the pilot. Forge DB: {} ({:?}); memory folder: {} ({:?}).",
                paths.forge_db_path,
                paths.forge_db_state,
                paths.memory_dir,
                paths.memory_dir_state
            ),
        );
    }

    if !completed_import_found && !other_import_namespaces.is_empty() {
        return base(
            ObservationDisposition::NamespaceMismatch,
            ImportRecordDisposition::NamespaceMismatch,
            format!(
                "No completed canonical import was found for namespace {}. Recent imports exist for other namespaces: {}. Run the canonical Forge export -> bridge -> semantic-memory import path for the requested namespace or switch the queried namespace.",
                scope.namespace,
                other_import_namespaces.join(", ")
            ),
        );
    }

    if source_inventory.supported_file_count > 0 && !completed_import_found {
        let import_command = format_import_command(scope, paths);
        let run_command = format_run_command(scope, paths);
        let bootstrap_command = format_bootstrap_source_command(scope, paths);
        return base(
            ObservationDisposition::ImportRequired,
            ImportRecordDisposition::NeverImported,
            if forge_evidence_inventory.available_bundle_count > 0 {
                format!(
                    "Forge has {} evidence bundle(s) available. Run {} and then rerun {}.",
                    forge_evidence_inventory.available_bundle_count, import_command, run_command
                )
            } else {
                format!(
                    "No completed canonical import or exportable Forge evidence bundle exists yet for namespace {}. Run {} to bootstrap thin imported source state, or populate Forge through the canonical verification/export lane and then run {}.",
                    scope.namespace, bootstrap_command, import_command
                )
            },
        );
    }

    if source_inventory.supported_file_count == 0 && !completed_import_found {
        return base(
            ObservationDisposition::EmptyWorkspace,
            ImportRecordDisposition::NeverImported,
            "No supported source files and no completed canonical imports were found for this namespace."
                .into(),
        );
    }

    base(
        ObservationDisposition::Ready,
        ImportRecordDisposition::Found,
        "No bootstrap step is required; the pilot can use canonical imported state.".into(),
    )
}

fn discover_source_inventory(config: &LoopConfig, paths: &ObservationPaths) -> SourceInventory {
    let workspace_path = resolve_runtime_path(&config.workspace_path);
    let memory_dir = resolve_runtime_path(&config.memory_dir);
    SourceInventory {
        supported_file_count: count_supported_source_files(
            &workspace_path,
            &memory_dir,
            matches!(paths.memory_dir_state, PathAvailability::Present),
        ),
        imported_current_state: BootstrapCurrentState::default(),
    }
}

fn discover_forge_evidence_inventory(
    config: &LoopConfig,
    paths: &ObservationPaths,
) -> ForgeEvidenceInventory {
    if !matches!(paths.forge_db_state, PathAvailability::Present) {
        return ForgeEvidenceInventory {
            available_bundle_count: 0,
        };
    }

    let available_bundle_count = ForgeStore::open(Path::new(&config.forge_db_path))
        .ok()
        .and_then(|store| store.count_evidence_bundles().ok())
        .unwrap_or(0);

    ForgeEvidenceInventory {
        available_bundle_count,
    }
}

fn format_import_command(scope: &Scope, paths: &ObservationPaths) -> String {
    format!(
        "`cargo run -p forge-pilot -- import --namespace {} --workspace-path {} --memory-dir {} --forge-db {}`",
        shell_quote(&scope.namespace),
        shell_quote(&paths.workspace_path),
        shell_quote(&paths.memory_dir),
        shell_quote(&paths.forge_db_path)
    )
}

fn format_run_command(scope: &Scope, paths: &ObservationPaths) -> String {
    format!(
        "`cargo run -p forge-pilot -- run --namespace {} --workspace-path {} --memory-dir {} --forge-db {}`",
        shell_quote(&scope.namespace),
        shell_quote(&paths.workspace_path),
        shell_quote(&paths.memory_dir),
        shell_quote(&paths.forge_db_path)
    )
}

fn format_bootstrap_source_command(scope: &Scope, paths: &ObservationPaths) -> String {
    format!(
        "`cargo run -p forge-pilot -- bootstrap-source --namespace {} --workspace {} --memory-dir {} --forge-db {}`",
        shell_quote(&scope.namespace),
        shell_quote(&paths.workspace_path),
        shell_quote(&paths.memory_dir),
        shell_quote(&paths.forge_db_path)
    )
}

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\"'\"'"))
}

fn count_supported_source_files(
    workspace_path: &Path,
    memory_dir: &Path,
    memory_dir_present: bool,
) -> usize {
    let mut total = 0usize;
    let mut stack = vec![workspace_path.to_path_buf()];
    let excluded_memory_dir = memory_dir_present
        .then(|| fs::canonicalize(memory_dir).ok())
        .flatten();

    while let Some(path) = stack.pop() {
        let Ok(entries) = fs::read_dir(&path) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let Ok(file_type) = entry.file_type() else {
                continue;
            };
            if file_type.is_dir() {
                if should_skip_workspace_dir(&path, excluded_memory_dir.as_deref()) {
                    continue;
                }
                stack.push(path);
                continue;
            }
            if file_type.is_file() && is_supported_source_file_path(&path) {
                total += 1;
            }
        }
    }

    total
}

fn inspect_memory_dir(path: &str) -> PathAvailability {
    let path = resolve_runtime_path(path);
    if !path.exists() {
        return PathAvailability::Missing;
    }
    if path.is_dir() {
        PathAvailability::Present
    } else {
        PathAvailability::Invalid
    }
}

fn inspect_forge_db(path: &str) -> PathAvailability {
    let path = resolve_runtime_path(path);
    if !path.exists() {
        return PathAvailability::Missing;
    }
    match rusqlite::Connection::open(&path)
        .and_then(|conn| conn.query_row("PRAGMA schema_version", [], |row| row.get::<_, i64>(0)))
    {
        Ok(_) => PathAvailability::Present,
        Err(_) => PathAvailability::Invalid,
    }
}

fn resolve_display_path(raw: &str) -> String {
    let joined = resolve_runtime_path(raw);
    fs::canonicalize(&joined)
        .unwrap_or(joined)
        .to_string_lossy()
        .to_string()
}

fn resolve_runtime_path(raw: &str) -> PathBuf {
    let expanded = expand_user_path(raw);
    if expanded.is_absolute() {
        expanded
    } else {
        std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(expanded)
    }
}

fn expand_user_path(raw: &str) -> PathBuf {
    if raw == "~" {
        return std::env::var_os("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(raw));
    }
    if let Some(rest) = raw.strip_prefix("~/") {
        return std::env::var_os("HOME")
            .map(|home| PathBuf::from(home).join(rest))
            .unwrap_or_else(|| PathBuf::from(raw));
    }
    PathBuf::from(raw)
}
