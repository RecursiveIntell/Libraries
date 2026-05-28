use crate::adapters::semantic_memory::SemanticMemoryAdapter;
use crate::config::RuntimeConfig;
use crate::entity::registry::{Entity, EntityRegistry};
use crate::error::RuntimeError;
use crate::ids::{ProjectionId, ProjectionKind, Scope, ScopeKey};
use crate::inference::InferenceExplanation;
use crate::obs::trace::{QueryTrace, QueryWarning};
use crate::projection::lifecycle::{
    InvalidationEvent, ProjectionActionResult, ProjectionHealth, ProjectionTracker,
    ProjectionVersion, StaleCause,
};
use crate::query::classify::{self, ClassifyResult, QueryMode};
use crate::query::merge::{self, LegResult, MergePolicy};
use crate::query::route::{self, RetrievalStrategy, RoutePlan};
use semantic_memory::{ProjectionClaimVersion, ProjectionEvidenceRef, SearchResult};
use serde::{Deserialize, Serialize};
use stack_ids::TraceCtx;
use std::time::Instant;

const EXPLICIT_TEMPORAL_LABEL: &str = "explicit bitemporal filters";

fn force_explicit_temporal_classification(classification: ClassifyResult) -> ClassifyResult {
    let temporal_component = QueryMode::TemporalLookup {
        temporal_expr: EXPLICIT_TEMPORAL_LABEL.into(),
    };

    let mode = match classification.mode {
        QueryMode::TemporalLookup { .. } => QueryMode::TemporalLookup {
            temporal_expr: EXPLICIT_TEMPORAL_LABEL.into(),
        },
        QueryMode::SemanticLookup => temporal_component,
        QueryMode::EntityLookup { mention } => QueryMode::Mixed {
            components: vec![QueryMode::EntityLookup { mention }, temporal_component],
        },
        QueryMode::Mixed { mut components } => {
            let has_temporal = components
                .iter()
                .any(|component| matches!(component, QueryMode::TemporalLookup { .. }));
            if !has_temporal {
                components.push(temporal_component);
            }
            QueryMode::Mixed { components }
        }
    };

    ClassifyResult {
        mode,
        confidence: classification.confidence,
        reason: Some("explicit temporal query requested".into()),
    }
}

/// Lifecycle state projected from imported verification truth.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectedVerificationLifecycle {
    Unverified,
    Verified,
    Contradicted,
    Superseded,
}

/// Promotion state projected from imported verification truth.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum ProjectedPromotionState {
    NotPromoted,
    Eligible,
    Blocked {
        reason: String,
    },
    Promoted {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        version_id: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        promoted_at: Option<String>,
    },
}

/// Runtime-facing typed read of verification and promotion state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectedVerificationSummary {
    pub claim_id: String,
    pub claim_version_id: String,
    pub lifecycle_state: ProjectedVerificationLifecycle,
    pub promotion_state: ProjectedPromotionState,
    pub claim_state: String,
    pub freshness: String,
    pub contradiction_status: String,
    pub completed_trial_count: u32,
    pub passed_refutation_count: u32,
    pub failed_refutation_count: u32,
    pub comparability_snapshot_version: Option<String>,
    pub notes: Vec<String>,
    pub source_envelope_id: String,
    pub recorded_at: String,
}

#[derive(Debug, Clone, Deserialize)]
struct MetadataVerificationSummary {
    lifecycle_state: ProjectedVerificationLifecycle,
    promotion_state: ProjectedPromotionState,
    completed_trial_count: u32,
    passed_refutation_count: u32,
    failed_refutation_count: u32,
    #[serde(default)]
    comparability_snapshot_version: Option<String>,
    #[serde(default)]
    notes: Vec<String>,
}

/// The main runtime handle.
///
/// `KnowledgeRuntime` orchestrates query classification, route planning,
/// retrieval execution, and result merging. It owns derived projections
/// (entity registry, projection tracker) but NEVER owns source truth —
/// all facts, episodes, and documents live in `semantic-memory`.
///
/// ## What is implemented
///
/// - Rule-based query classification (semantic, entity, temporal, mixed)
/// - Route planning from classification
/// - Projection-backed retrieval for imported claims, relations, and episodes
///   with hybrid fallback only when the projection substrate is unavailable
/// - Entity search with scope-aware registry resolution and bounded imported
///   alias candidate expansion
/// - Result merge with duplicate fusion and multi-leg provenance
/// - Projection health/status tracking with invalidation by scope/kind
/// - Query traces with degradation warnings
/// - Dedicated explain/audit entrypoints for imported evidence refs
/// - **Temporal search execution on supported projection routes**: Temporal legs
///   resolve supported expressions to concrete as-of timestamps and query imported
///   versions directly. Explicit bitemporal parameters in `query_temporal()` are
///   now also applied on projection-backed hybrid and entity routes. Unsupported
///   expressions or missing projection substrate degrade explicitly when strict
///   mode is off.
/// - **Route-aware scope enforcement**: Full scope is enforced on projection-backed
///   routes. `ScopePartiallyEnforced` is emitted only when execution must fall back
///   to namespace-only hybrid retrieval.
/// - **Non-authoritative entity cache**: The entity registry is accessible via
///   `refresh_entity_cache()` which returns a fenced `EntityCacheHandle` that
///   only exposes cache-refresh operations. Direct mutable access to the
///   registry has been removed to enforce non-authoritative semantics.
///
/// ## What is NOT implemented
///
/// - **Range temporal expressions**: `before`, `after`, `since`, `between`,
///   and similar range-oriented phrases still degrade explicitly because the
///   runtime currently supports only concrete as-of timestamps on projection
///   routes.
/// - **Projection persistence**: Not implemented in this crate today. The
///   `persist` config field is retained for serde backward compatibility but
///   rejected at validation time (`InvalidConfig` error). Projections are
///   derived, non-authoritative, and rebuildable from upstream state.
/// - **Projection rebuild execution**: The tracker records build/failure events
///   but does not trigger actual rebuilds. Callers must drive rebuilds.
///   **The projection tracker is observability-only, not an orchestration
///   primitive.** It records state transitions (build, failure, invalidation)
///   for inspection and diagnostics. It does NOT schedule, execute, or retry
///   projection rebuilds. Any rebuild logic must live in an external
///   orchestrator that reads tracker state and calls the appropriate APIs.
/// - **LLM-based classification**: Classifier is rule-based heuristic.
/// - **Advanced explain/audit cross-system dereference**: The runtime does not
///   dereference raw evidence handles.
pub struct KnowledgeRuntime {
    adapter: SemanticMemoryAdapter,
    config: RuntimeConfig,
    entity_registry: EntityRegistry,
    projection_tracker: ProjectionTracker,
}

impl KnowledgeRuntime {
    /// Create a new runtime from a config and a semantic-memory adapter.
    ///
    /// Returns `InvalidConfig` if the config contains invalid values,
    /// including `projection.persist = true` (persistence is not implemented).
    pub fn new(
        mut config: RuntimeConfig,
        adapter: SemanticMemoryAdapter,
    ) -> Result<Self, RuntimeError> {
        config.normalize_and_validate()?;

        let entity_registry =
            EntityRegistry::new(config.entity.max_aliases, config.entity.max_entities);
        let projection_tracker = ProjectionTracker::new(config.projection.staleness_threshold_secs);

        Ok(Self {
            adapter,
            config,
            entity_registry,
            projection_tracker,
        })
    }

    pub(crate) fn adapter_ref(&self) -> &SemanticMemoryAdapter {
        &self.adapter
    }

    pub(crate) fn default_scope(&self) -> &Scope {
        &self.config.default_scope
    }

    // ── Query pipeline ──────────────────────────────────────────

    /// Execute a query through the full pipeline: classify -> plan -> execute -> merge.
    ///
    /// Returns merged results and a query trace for observability.
    /// The trace includes any degradation warnings (e.g. temporal downgrade).
    ///
    /// Generates a fresh `TraceCtx` internally. To supply a caller-owned trace
    /// context (for cross-crate correlation), use [`query_with_trace()`](Self::query_with_trace).
    pub async fn query(
        &self,
        query: &str,
        scope: Option<&Scope>,
    ) -> Result<(Vec<SearchResult>, QueryTrace), RuntimeError> {
        self.query_with_trace(query, scope, None).await
    }

    /// Execute a query with an optional caller-supplied `TraceCtx`.
    ///
    /// If `trace_ctx` is `None`, a fresh trace context is generated.
    /// This is the primary query entry point for callers that need
    /// cross-crate trace correlation (e.g., bridge -> runtime -> memory).
    ///
    /// ## Scope enforcement
    ///
    /// Scope handling is route-aware. Projection-backed routes enforce the full
    /// scope directly against imported rows. Namespace-only hybrid fallback emits
    /// `ScopePartiallyEnforced` only when the runtime lacks a projection-backed
    /// route for the requested leg and strict scope is disabled.
    pub async fn query_with_trace(
        &self,
        query: &str,
        scope: Option<&Scope>,
        trace_ctx: Option<TraceCtx>,
    ) -> Result<(Vec<SearchResult>, QueryTrace), RuntimeError> {
        let start = Instant::now();
        let trace_ctx = trace_ctx.unwrap_or_else(TraceCtx::generate);
        let scope = scope.unwrap_or(&self.config.default_scope);
        let scope_key = scope.key();
        let mut warnings = Vec::new();
        let mut last_import_at = None;
        let has_projection_imports = match self.adapter.last_import_at(&scope.namespace).await {
            Ok(ts) => {
                last_import_at = ts.clone();
                ts.is_some()
            }
            Err(e) => {
                tracing::warn!(
                    namespace = %scope.namespace,
                    error = %e,
                    "failed to check import freshness"
                );
                false
            }
        };
        if self.config.projection.import_staleness_threshold_secs > 0 {
            if let Some(ts) = last_import_at {
                if let Ok(imported_at) = chrono::DateTime::parse_from_rfc3339(&ts) {
                    let age = chrono::Utc::now()
                        .signed_duration_since(imported_at)
                        .num_seconds()
                        .unsigned_abs();
                    if age > self.config.projection.import_staleness_threshold_secs {
                        warnings.push(QueryWarning::ProjectionImportStale {
                            scope: scope_key.clone(),
                            last_import_at: Some(ts),
                        });
                    }
                }
            }
        }

        // Phase 1: Classify
        let classification = classify::classify(query);

        // Phase 2: Plan
        let plan = route::plan(
            query,
            &classification.mode,
            scope,
            self.config.query.default_limit,
        );

        // Phase 3: Execute legs
        let (leg_results, leg_timings) = self
            .execute_plan(
                &plan,
                &scope_key,
                has_projection_imports,
                &mut warnings,
                None,
            )
            .await?;

        // Phase 4: Merge
        let policy = MergePolicy {
            limit: self.config.query.default_limit,
            ..Default::default()
        };
        let merged = merge::merge(leg_results, &policy);

        let total_duration = start.elapsed();

        // Phase 5: Build trace
        let trace = QueryTrace::from_pipeline(
            trace_ctx,
            scope_key,
            classification,
            plan,
            leg_timings,
            total_duration,
            &merged,
            warnings,
        );

        let results = merged.results.into_iter().map(|m| m.result).collect();
        Ok((results, trace))
    }

    /// Execute a query and attach the latest non-authoritative kernel
    /// explanation available for the queried scope.
    pub async fn query_with_inference_explanation(
        &self,
        query: &str,
        scope: Option<&Scope>,
        trace_ctx: Option<TraceCtx>,
    ) -> Result<(Vec<SearchResult>, QueryTrace, Option<InferenceExplanation>), RuntimeError> {
        let (results, mut trace) = self.query_with_trace(query, scope, trace_ctx).await?;
        let explanation = self.latest_inference_explanation(scope).await?;
        trace.kernel_degraded_reason = explanation
            .as_ref()
            .and_then(|value| value.degraded_reason.clone());
        Ok((results, trace, explanation))
    }

    /// Execute a temporal query with explicit bitemporal semantics.
    ///
    /// This is the public equivalent for:
    /// `as_of(valid_t, recorded_t_or_before)`.
    ///
    /// `valid_at` filters by projection-valid time, while
    /// `recorded_at_or_before` filters by importer transaction time.
    /// This method keeps the same warning behavior as the normal pipeline for
    /// scope and temporal fallback on supported routes.
    pub async fn query_temporal(
        &self,
        query: &str,
        scope: Option<&Scope>,
        valid_at: &str,
        recorded_at_or_before: &str,
    ) -> Result<(Vec<SearchResult>, QueryTrace), RuntimeError> {
        self.query_temporal_with_trace(query, scope, None, valid_at, recorded_at_or_before)
            .await
    }

    /// Execute an explicit-temporal query with an explicit trace context.
    ///
    /// See [`query_temporal`](Self::query_temporal) for semantics.
    pub async fn query_temporal_with_trace(
        &self,
        query: &str,
        scope: Option<&Scope>,
        trace_ctx: Option<TraceCtx>,
        valid_at: &str,
        recorded_at_or_before: &str,
    ) -> Result<(Vec<SearchResult>, QueryTrace), RuntimeError> {
        let start = Instant::now();
        let trace_ctx = trace_ctx.unwrap_or_else(TraceCtx::generate);
        let scope = scope.unwrap_or(&self.config.default_scope);
        let scope_key = scope.key();
        let mut warnings = Vec::new();
        let mut last_import_at = None;
        let has_projection_imports = match self.adapter.last_import_at(&scope.namespace).await {
            Ok(ts) => {
                last_import_at = ts.clone();
                ts.is_some()
            }
            Err(e) => {
                tracing::warn!(
                    namespace = %scope.namespace,
                    error = %e,
                    "failed to check import freshness"
                );
                false
            }
        };
        if self.config.projection.import_staleness_threshold_secs > 0 {
            if let Some(ts) = last_import_at {
                if let Ok(imported_at) = chrono::DateTime::parse_from_rfc3339(&ts) {
                    let age = chrono::Utc::now()
                        .signed_duration_since(imported_at)
                        .num_seconds()
                        .unsigned_abs();
                    if age > self.config.projection.import_staleness_threshold_secs {
                        warnings.push(QueryWarning::ProjectionImportStale {
                            scope: scope_key.clone(),
                            last_import_at: Some(ts),
                        });
                    }
                }
            }
        }

        let classification = force_explicit_temporal_classification(classify::classify(query));
        let plan = route::plan(
            query,
            &classification.mode,
            scope,
            self.config.query.default_limit,
        );

        let explicit_temporal = Some((valid_at.to_string(), recorded_at_or_before.to_string()));
        let (leg_results, leg_timings) = self
            .execute_plan(
                &plan,
                &scope_key,
                has_projection_imports,
                &mut warnings,
                explicit_temporal,
            )
            .await?;

        let policy = MergePolicy {
            limit: self.config.query.default_limit,
            ..Default::default()
        };
        let merged = merge::merge(leg_results, &policy);

        let total_duration = start.elapsed();
        let has_downgrade = warnings
            .iter()
            .any(|w| matches!(w, QueryWarning::TemporalDowngradedToHybrid { .. }));
        let temporal_mode = if has_downgrade { "downgraded" } else { "exact" };
        let mut trace = QueryTrace::from_pipeline(
            trace_ctx,
            scope_key,
            classification,
            plan,
            leg_timings,
            total_duration,
            &merged,
            warnings,
        );
        trace.valid_as_of = Some(valid_at.to_string());
        trace.recorded_as_of = Some(recorded_at_or_before.to_string());
        trace.temporal_mode = Some(temporal_mode.to_string());

        let results = merged.results.into_iter().map(|m| m.result).collect();
        Ok((results, trace))
    }

    /// Query evidence references for a claim via the explicit explain/audit path.
    ///
    /// This returns imported evidence handles as opaque rows and does not alter
    /// normal retrieval behavior.
    pub async fn query_evidence_refs_for_claim(
        &self,
        claim_id: &str,
        claim_version_id: Option<&str>,
        scope: Option<&Scope>,
        limit: usize,
    ) -> Result<Vec<ProjectionEvidenceRef>, RuntimeError> {
        let scope = scope.unwrap_or(&self.config.default_scope);
        self.adapter
            .query_evidence_refs_for_claim(claim_id, claim_version_id, scope, None, limit)
            .await
    }

    /// Query evidence references for a claim as of a transaction-time cutoff.
    pub async fn query_evidence_refs_for_claim_as_of(
        &self,
        claim_id: &str,
        claim_version_id: Option<&str>,
        scope: Option<&Scope>,
        recorded_at_or_before: &str,
        limit: usize,
    ) -> Result<Vec<ProjectionEvidenceRef>, RuntimeError> {
        let scope = scope.unwrap_or(&self.config.default_scope);
        self.adapter
            .query_evidence_refs_for_claim(
                claim_id,
                claim_version_id,
                scope,
                Some(recorded_at_or_before),
                limit,
            )
            .await
    }

    /// Query the projected verification summary for a claim.
    ///
    /// This reads imported claim rows only. It does not reconstruct truth from
    /// raw evidence dereference, and it does not mutate runtime state.
    pub async fn query_verification_summary_for_claim(
        &self,
        claim_id: &str,
        claim_version_id: Option<&str>,
        scope: Option<&Scope>,
    ) -> Result<Option<ProjectedVerificationSummary>, RuntimeError> {
        let scope = scope.unwrap_or(&self.config.default_scope);
        let rows = self
            .adapter
            .query_claim_versions_for_claim(claim_id, claim_version_id, scope, None, 8)
            .await?;
        Ok(rows.first().map(projected_verification_summary_from_claim))
    }

    /// Query the projected verification summary for a claim as of a recorded-time cutoff.
    pub async fn query_verification_summary_for_claim_as_of(
        &self,
        claim_id: &str,
        claim_version_id: Option<&str>,
        scope: Option<&Scope>,
        recorded_at_or_before: &str,
    ) -> Result<Option<ProjectedVerificationSummary>, RuntimeError> {
        let scope = scope.unwrap_or(&self.config.default_scope);
        let rows = self
            .adapter
            .query_claim_versions_for_claim(
                claim_id,
                claim_version_id,
                scope,
                Some(recorded_at_or_before),
                8,
            )
            .await?;
        Ok(rows.first().map(projected_verification_summary_from_claim))
    }

    /// Classify a query without executing it.
    pub fn classify(&self, query: &str) -> ClassifyResult {
        classify::classify(query)
    }

    /// Plan a query without executing it.
    pub fn plan(&self, query: &str, scope: Option<&Scope>) -> RoutePlan {
        let scope = scope.unwrap_or(&self.config.default_scope);
        let classification = classify::classify(query);
        route::plan(
            query,
            &classification.mode,
            scope,
            self.config.query.default_limit,
        )
    }

    // ── Entity registry access ──────────────────────────────────

    /// Access the entity registry for read-only lookup.
    pub fn entity_registry(&self) -> &EntityRegistry {
        &self.entity_registry
    }

    /// Get a cache-refresh handle for loading entities from upstream canonical state.
    ///
    /// ## Authority boundary
    ///
    /// The entity registry is a **derived, non-authoritative cache**. This handle
    /// only exposes cache-refresh operations — not authority-like mutation.
    ///
    /// Use this for:
    /// - Loading entities from upstream memory canonical state
    /// - Rebuilding the cache after invalidation
    /// - Clearing scope or full cache for rebuild
    /// - Test setup
    ///
    /// Runtime MUST NOT become an authoritative source of identity state.
    pub fn refresh_entity_cache(&mut self) -> EntityCacheHandle<'_> {
        EntityCacheHandle {
            registry: &mut self.entity_registry,
        }
    }

    // ── Projection maintenance ──────────────────────────────────

    /// Get the health of a specific projection.
    ///
    /// **Observability only.** This reads tracker state; the tracker does not
    /// execute rebuilds or schedule any work based on health status.
    pub fn projection_health(&self, id: &ProjectionId) -> ProjectionHealth {
        self.projection_tracker.health(id)
    }

    /// Query projection status by optional kind and scope filters.
    pub fn projection_status(
        &self,
        kind: Option<&ProjectionKind>,
        scope: Option<&ScopeKey>,
    ) -> Vec<&crate::projection::lifecycle::ProjectionMeta> {
        self.projection_tracker.query_status(kind, scope)
    }

    /// Record that a projection was successfully built.
    pub fn record_projection_build(
        &mut self,
        id: ProjectionId,
        source_count: usize,
        build_duration_ms: u64,
        version: Option<ProjectionVersion>,
    ) {
        self.projection_tracker
            .record_build(id, source_count, build_duration_ms, version);
    }

    /// Record that a projection build failed.
    pub fn record_projection_failure(&mut self, id: ProjectionId, error: String) {
        self.projection_tracker.record_failure(id, error);
    }

    /// Invalidate specific projections.
    pub fn invalidate_projections(&mut self, event: &InvalidationEvent) -> ProjectionActionResult {
        self.projection_tracker.invalidate(event)
    }

    /// Invalidate all projections of a given kind within a scope.
    pub fn invalidate_projections_by_kind(
        &mut self,
        kind: &ProjectionKind,
        scope: &ScopeKey,
        cause: StaleCause,
    ) -> ProjectionActionResult {
        self.projection_tracker
            .invalidate_by_kind_and_scope(kind, scope, cause)
    }

    /// Invalidate all projections within a scope.
    pub fn invalidate_scope(
        &mut self,
        scope: &ScopeKey,
        cause: StaleCause,
    ) -> ProjectionActionResult {
        self.projection_tracker.invalidate_scope(scope, cause)
    }

    /// Clear all projections within a scope.
    pub fn clear_projection_scope(&mut self, scope: &ScopeKey) -> ProjectionActionResult {
        self.projection_tracker.clear_scope(scope)
    }

    /// Access the projection tracker directly (read-only).
    ///
    /// **Observability only, not orchestration.** The tracker records
    /// projection lifecycle events (build, invalidation, failure) but
    /// does not trigger, schedule, or retry any work. External callers
    /// must drive rebuild execution and report outcomes via
    /// `record_projection_build()` / `record_projection_failure()`.
    pub fn projection_tracker(&self) -> &ProjectionTracker {
        &self.projection_tracker
    }

    // ── Adapter / config access ─────────────────────────────────

    /// Access the underlying semantic-memory adapter.
    pub fn adapter(&self) -> &SemanticMemoryAdapter {
        &self.adapter
    }

    /// Access the runtime config.
    pub fn config(&self) -> &RuntimeConfig {
        &self.config
    }

    // ── Internal ────────────────────────────────────────────────

    /// Execute all legs of a route plan, returning per-leg results and timings.
    async fn execute_plan(
        &self,
        plan: &RoutePlan,
        scope_key: &ScopeKey,
        has_projection_imports: bool,
        warnings: &mut Vec<QueryWarning>,
        explicit_temporal: Option<(String, String)>,
    ) -> Result<(Vec<Vec<LegResult>>, Vec<u64>), RuntimeError> {
        let mut all_legs = Vec::with_capacity(plan.legs.len());
        let mut timings = Vec::with_capacity(plan.legs.len());
        let scope_requires_pushdown = scope_has_extra_dimensions(&plan.scope);
        for (i, leg) in plan.legs.iter().enumerate() {
            let leg_start = Instant::now();

            let results = match &leg.strategy {
                RetrievalStrategy::HybridSearch => {
                    if has_projection_imports {
                        if let Some((valid_at, recorded_at_or_before)) = explicit_temporal.as_ref()
                        {
                            self.adapter
                                .search_projection_temporal_with_recorded_at(
                                    &plan.query,
                                    &plan.scope,
                                    valid_at,
                                    recorded_at_or_before,
                                    leg.limit,
                                )
                                .await?
                        } else {
                            let projection_results = self
                                .adapter
                                .search_projection_text(&plan.query, &plan.scope, leg.limit)
                                .await?;
                            // LIB-CRIT-001: Always merge with hybrid (FTS5 + HNSW + RRF)
                            // unless scope requires pushdown and projection alone is sufficient.
                            // Projection only has substring matching — hybrid search provides
                            // semantic retrieval via embeddings.
                            if scope_requires_pushdown && projection_results.len() >= leg.limit {
                                projection_results
                            } else {
                                let fallback = self
                                    .adapter
                                    .search(&plan.query, &plan.scope, Some(leg.limit), None)
                                    .await?;
                                merge_leg_results(projection_results, fallback, leg.limit)
                            }
                        }
                    } else {
                        if explicit_temporal.is_some() {
                            self.handle_temporal_fallback(
                                EXPLICIT_TEMPORAL_LABEL,
                                &plan.scope,
                                warnings,
                            )?;
                        }
                        self.adapter
                            .search(&plan.query, &plan.scope, Some(leg.limit), None)
                            .await?
                    }
                }
                RetrievalStrategy::EntitySearch { mention } => {
                    let resolve = self.entity_registry.resolve(mention, scope_key);
                    if resolve.quality == crate::entity::registry::MatchQuality::ScopedFallback {
                        warnings.push(QueryWarning::EntityScopeFallback {
                            mention: mention.clone(),
                            queried_scope: scope_key.clone(),
                        });
                    }
                    let mut candidate_ids = Vec::new();
                    if let Some(entity) = resolve.entity.as_ref() {
                        candidate_ids.push(entity.id.clone());
                    }

                    if has_projection_imports {
                        let alias_limit = leg.limit.clamp(4, 16);
                        let alias_candidates = if let Some((valid_at, recorded_at_or_before)) =
                            explicit_temporal.as_ref()
                        {
                            self.adapter
                                .query_alias_candidates_with_recorded_at(
                                    mention,
                                    &plan.scope,
                                    valid_at,
                                    recorded_at_or_before,
                                    alias_limit,
                                )
                                .await?
                        } else {
                            self.adapter
                                .query_alias_candidates(mention, &plan.scope, alias_limit)
                                .await?
                        };
                        for alias in alias_candidates {
                            if !candidate_ids
                                .iter()
                                .any(|id| id == &alias.canonical_entity_id)
                            {
                                candidate_ids.push(alias.canonical_entity_id);
                            }
                        }

                        if !candidate_ids.is_empty() {
                            let projection_results =
                                if let Some((valid_at, recorded_at_or_before)) =
                                    explicit_temporal.as_ref()
                                {
                                    self.adapter
                                        .search_projection_by_entity_ids_with_recorded_at(
                                            &plan.scope,
                                            &candidate_ids,
                                            valid_at,
                                            recorded_at_or_before,
                                            leg.limit,
                                        )
                                        .await?
                                } else {
                                    self.adapter
                                        .search_projection_by_entity_ids(
                                            &plan.scope,
                                            &candidate_ids,
                                            leg.limit,
                                        )
                                        .await?
                                };
                            if scope_requires_pushdown || explicit_temporal.is_some() {
                                projection_results
                            } else {
                                let search_term = resolve
                                    .entity
                                    .as_ref()
                                    .map(|e| e.canonical_name.as_str())
                                    .unwrap_or(mention);
                                let fallback = self
                                    .adapter
                                    .search(search_term, &plan.scope, Some(leg.limit), None)
                                    .await?;
                                merge_leg_results(projection_results, fallback, leg.limit)
                            }
                        } else {
                            if explicit_temporal.is_some() {
                                self.handle_temporal_fallback(
                                    EXPLICIT_TEMPORAL_LABEL,
                                    &plan.scope,
                                    warnings,
                                )?;
                            }
                            let search_term = resolve
                                .entity
                                .as_ref()
                                .map(|e| e.canonical_name.as_str())
                                .unwrap_or(mention);
                            self.adapter
                                .search(search_term, &plan.scope, Some(leg.limit), None)
                                .await?
                        }
                    } else {
                        if explicit_temporal.is_some() {
                            self.handle_temporal_fallback(
                                EXPLICIT_TEMPORAL_LABEL,
                                &plan.scope,
                                warnings,
                            )?;
                        }
                        let search_term = resolve
                            .entity
                            .as_ref()
                            .map(|e| e.canonical_name.as_str())
                            .unwrap_or(mention);
                        self.adapter
                            .search(search_term, &plan.scope, Some(leg.limit), None)
                            .await?
                    }
                }
                RetrievalStrategy::TemporalSearch { temporal_expr } => {
                    if has_projection_imports {
                        if let Some((valid_at, recorded_at_or_before)) = explicit_temporal.as_ref()
                        {
                            self.adapter
                                .search_projection_temporal_with_recorded_at(
                                    &plan.query,
                                    &plan.scope,
                                    valid_at,
                                    recorded_at_or_before,
                                    leg.limit,
                                )
                                .await?
                        } else if let Some(valid_at) = resolve_temporal_as_of(temporal_expr) {
                            self.adapter
                                .search_projection_temporal(
                                    &plan.query,
                                    &plan.scope,
                                    &valid_at,
                                    leg.limit,
                                )
                                .await?
                        } else {
                            self.handle_temporal_fallback(temporal_expr, &plan.scope, warnings)?;
                            self.adapter
                                .search(&plan.query, &plan.scope, Some(leg.limit), None)
                                .await?
                        }
                    } else {
                        self.handle_temporal_fallback(temporal_expr, &plan.scope, warnings)?;
                        self.adapter
                            .search(&plan.query, &plan.scope, Some(leg.limit), None)
                            .await?
                    }
                }
            };

            let leg_results: Vec<LegResult> = results
                .into_iter()
                .map(|r| LegResult {
                    leg_index: i,
                    result: r,
                })
                .collect();

            timings.push(leg_start.elapsed().as_millis() as u64);
            all_legs.push(leg_results);
        }

        Ok((all_legs, timings))
    }

    fn handle_temporal_fallback(
        &self,
        temporal_expr: &str,
        scope: &Scope,
        warnings: &mut Vec<QueryWarning>,
    ) -> Result<(), RuntimeError> {
        if self.config.strict_temporal {
            return Err(RuntimeError::TemporalNotSupported {
                temporal_expr: temporal_expr.to_string(),
            });
        }

        push_temporal_warning_once(warnings, temporal_expr);
        tracing::warn!(
            temporal_expr = %temporal_expr,
            "temporal execution unsupported for this route, degrading to hybrid"
        );
        let _ = scope;
        Ok(())
    }
}

fn scope_has_extra_dimensions(scope: &Scope) -> bool {
    scope.domain.is_some() || scope.workspace_id.is_some() || scope.repo_id.is_some()
}

fn push_temporal_warning_once(warnings: &mut Vec<QueryWarning>, temporal_expr: &str) {
    let already_present = warnings.iter().any(|warning| {
        matches!(
            warning,
            QueryWarning::TemporalDowngradedToHybrid { temporal_expr: seen }
                if seen == temporal_expr
        )
    });
    if !already_present {
        warnings.push(QueryWarning::TemporalDowngradedToHybrid {
            temporal_expr: temporal_expr.to_string(),
        });
    }
}

fn projected_verification_summary_from_claim(
    claim: &ProjectionClaimVersion,
) -> ProjectedVerificationSummary {
    let parsed_metadata = claim
        .metadata
        .as_ref()
        .and_then(|metadata| metadata.get("verification_summary").cloned())
        .and_then(|summary| serde_json::from_value::<MetadataVerificationSummary>(summary).ok());
    let top_level_promotion_state = claim
        .metadata
        .as_ref()
        .and_then(|metadata| metadata.get("promotion_state").cloned())
        .and_then(|state| serde_json::from_value::<ProjectedPromotionState>(state).ok());

    let lifecycle_state = parsed_metadata
        .as_ref()
        .map(|summary| summary.lifecycle_state.clone())
        .unwrap_or_else(|| fallback_lifecycle_state(claim));
    let promotion_state = parsed_metadata
        .as_ref()
        .map(|summary| summary.promotion_state.clone())
        .or(top_level_promotion_state)
        .unwrap_or(ProjectedPromotionState::NotPromoted);

    ProjectedVerificationSummary {
        claim_id: claim.claim_id.as_str().to_string(),
        claim_version_id: claim.claim_version_id.as_str().to_string(),
        lifecycle_state,
        promotion_state,
        claim_state: claim.claim_state.clone(),
        freshness: claim.freshness.clone(),
        contradiction_status: normalize_contradiction_status(&claim.contradiction_status),
        completed_trial_count: parsed_metadata
            .as_ref()
            .map(|summary| summary.completed_trial_count)
            .unwrap_or(0),
        passed_refutation_count: parsed_metadata
            .as_ref()
            .map(|summary| summary.passed_refutation_count)
            .unwrap_or(0),
        failed_refutation_count: parsed_metadata
            .as_ref()
            .map(|summary| summary.failed_refutation_count)
            .unwrap_or(0),
        comparability_snapshot_version: parsed_metadata
            .as_ref()
            .and_then(|summary| summary.comparability_snapshot_version.clone())
            .or_else(|| {
                claim.metadata.as_ref().and_then(|metadata| {
                    metadata
                        .get("comparability_snapshot_version")
                        .and_then(serde_json::Value::as_str)
                        .map(str::to_string)
                })
            }),
        notes: parsed_metadata
            .as_ref()
            .map(|summary| summary.notes.clone())
            .unwrap_or_default(),
        source_envelope_id: claim.source_envelope_id.as_str().to_string(),
        recorded_at: claim.recorded_at.clone(),
    }
}

fn fallback_lifecycle_state(claim: &ProjectionClaimVersion) -> ProjectedVerificationLifecycle {
    if claim.claim_state == "superseded" || claim.freshness == "superseded" {
        ProjectedVerificationLifecycle::Superseded
    } else if claim.claim_state == "disputed"
        || !contradiction_status_is_none(&claim.contradiction_status)
    {
        ProjectedVerificationLifecycle::Contradicted
    } else {
        ProjectedVerificationLifecycle::Unverified
    }
}

fn contradiction_status_is_none(status: &str) -> bool {
    matches!(
        serde_json::from_str::<serde_json::Value>(status),
        Ok(serde_json::Value::String(value)) if value == "none"
    ) || status == "none"
}

fn normalize_contradiction_status(status: &str) -> String {
    match serde_json::from_str::<serde_json::Value>(status) {
        Ok(serde_json::Value::String(value)) => value,
        Ok(serde_json::Value::Object(obj)) => {
            if let Some(description) = obj
                .get("possible_contradiction")
                .and_then(|value| value.get("description"))
                .and_then(serde_json::Value::as_str)
            {
                return format!("possible_contradiction:{description}");
            }
            if let Some(contradicted_by) = obj
                .get("confirmed")
                .and_then(|value| value.get("contradicted_by"))
                .and_then(serde_json::Value::as_str)
            {
                return format!("confirmed:{contradicted_by}");
            }
            if let Some(resolution) = obj
                .get("resolved")
                .and_then(|value| value.get("resolution"))
                .and_then(serde_json::Value::as_str)
            {
                return format!("resolved:{resolution}");
            }
            status.to_string()
        }
        _ => status.to_string(),
    }
}

fn merge_leg_results(
    mut primary: Vec<SearchResult>,
    fallback: Vec<SearchResult>,
    limit: usize,
) -> Vec<SearchResult> {
    primary.extend(fallback);
    primary.sort_by(|left, right| {
        right
            .score
            .partial_cmp(&left.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    primary.truncate(limit);
    primary
}

fn resolve_temporal_as_of(temporal_expr: &str) -> Option<String> {
    use chrono::{Datelike, Duration, NaiveDate, Utc};

    let now = Utc::now();
    let resolved = match temporal_expr.to_lowercase().as_str() {
        "today" | "recent" | "recently" | "latest" => now,
        "yesterday" => now - Duration::days(1),
        "last week" => now - Duration::weeks(1),
        "last month" => now - Duration::days(30),
        "last year" => now - Duration::days(365),
        "this week" => {
            let days_from_monday = now.weekday().num_days_from_monday() as i64;
            let date = now.date_naive() - Duration::days(days_from_monday);
            chrono::DateTime::<Utc>::from_naive_utc_and_offset(date.and_hms_opt(0, 0, 0)?, Utc)
        }
        "this month" => {
            let date = NaiveDate::from_ymd_opt(now.year(), now.month(), 1)?;
            chrono::DateTime::<Utc>::from_naive_utc_and_offset(date.and_hms_opt(0, 0, 0)?, Utc)
        }
        _ => return None,
    };
    Some(resolved.to_rfc3339())
}

/// Handle for refreshing the entity cache from upstream canonical state.
///
/// This is intentionally NOT the full [`EntityRegistry`] API — it only exposes
/// cache-refresh operations to enforce non-authoritative semantics. The entity
/// registry is a derived projection that can be rebuilt without data loss.
pub struct EntityCacheHandle<'a> {
    registry: &'a mut EntityRegistry,
}

impl<'a> EntityCacheHandle<'a> {
    /// Create a cache handle from a mutable reference to an entity registry.
    ///
    /// This is useful for testing or when the caller already has a mutable
    /// reference to an `EntityRegistry` (e.g., outside the runtime context).
    /// In normal runtime usage, prefer [`KnowledgeRuntime::refresh_entity_cache()`].
    pub fn from_registry_mut(registry: &'a mut EntityRegistry) -> Self {
        Self { registry }
    }

    /// Load an entity from upstream canonical state into the cache.
    ///
    /// This registers (or updates) an entity in the derived cache.
    /// The entity data should originate from an upstream canonical source
    /// (e.g., `semantic-memory` entity_aliases table).
    pub fn load_from_upstream(&mut self, entity: Entity) -> Result<(), RuntimeError> {
        self.registry.register(entity)
    }

    /// Clear all cached entities in a scope (for targeted rebuild).
    pub fn clear_scope(&mut self, scope: &ScopeKey) {
        self.registry.clear_scope(scope)
    }

    /// Clear all cached entities (for full rebuild).
    pub fn clear_all(&mut self) {
        self.registry.clear_all()
    }

    /// Number of entities currently in the cache.
    pub fn len(&self) -> usize {
        self.registry.len()
    }

    /// Whether the cache is empty.
    pub fn is_empty(&self) -> bool {
        self.registry.is_empty()
    }
}

impl std::fmt::Debug for KnowledgeRuntime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("KnowledgeRuntime")
            .field("config", &self.config)
            .field("entity_count", &self.entity_registry.len())
            .field("projection_count", &self.projection_tracker.list().len())
            .finish()
    }
}
