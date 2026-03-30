//! Query and projection observability traces.
//!
//! Every query execution produces a `QueryTrace` that records classification,
//! route plan, per-leg timing, merge statistics, and any degradation warnings.
//! Projection lifecycle actions produce a `ProjectionTrace`.

use crate::ids::ScopeKey;
use crate::projection::lifecycle::{ProjectionHealth, StaleCause};
use crate::query::classify::ClassifyResult;
use crate::query::merge::MergedResults;
use crate::query::route::{RetrievalStrategy, RoutePlan};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use stack_ids::TraceCtx;
use std::time::Duration;

/// A warning or degradation notice emitted during query execution.
///
/// These make runtime behavior transparent to callers — especially
/// when the runtime falls back to simpler behavior than requested.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryWarning {
    /// A temporal search leg was downgraded to hybrid search because
    /// temporal execution is not implemented.
    TemporalDowngradedToHybrid {
        /// The original temporal expression from the query.
        temporal_expr: String,
    },
    /// Scope enforcement is limited to namespace-only at the adapter level.
    /// Full scope (domain, workspace_id, repo_id) was used for runtime-owned
    /// logic but NOT enforced during upstream search.
    ScopePartiallyEnforced {
        /// The full scope that was requested.
        full_scope: ScopeKey,
    },
    /// Entity resolution fell back to a broader scope.
    EntityScopeFallback {
        mention: String,
        queried_scope: ScopeKey,
    },
    /// Projection data for this scope is stale due to import lag or failure.
    ProjectionImportStale {
        /// Scope affected.
        scope: ScopeKey,
        /// When the last successful import occurred, if ever.
        last_import_at: Option<String>,
    },
}

/// Logical view through which a query leg is executed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeView {
    Semantic,
    Temporal,
    Entity,
    Causal,
    Control,
}

/// Disclosure record emitted when a query leg is widened or degraded from its requested view.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct WideningDisclosure {
    pub leg_index: usize,
    pub requested_view: RuntimeView,
    pub executed_view: RuntimeView,
    pub reason: String,
}

/// Observability record for a single query execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryTrace {
    /// Canonical trace context for cross-crate correlation.
    pub trace_ctx: TraceCtx,
    /// Phase status: compatibility / migration-only.
    /// Legacy trace_id extracted from trace_ctx for backward compat.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace_id: Option<String>,
    /// Scope used for this query.
    pub scope: ScopeKey,
    /// Classification result.
    pub classification: ClassifyResult,
    /// Route plan that was executed.
    pub plan: RoutePlan,
    /// Per-leg timing in milliseconds.
    pub leg_timings_ms: Vec<u64>,
    /// Total query duration.
    #[serde(with = "duration_ms")]
    pub total_duration: Duration,
    /// Number of raw results before merge.
    pub raw_result_count: usize,
    /// Number of results after merge.
    pub merged_result_count: usize,
    /// Number of duplicates fused during merge.
    pub duplicates_fused: usize,
    /// Distinct views requested by the route plan.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub requested_views: Vec<RuntimeView>,
    /// Distinct views actually executed after degradation/widening.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub executed_views: Vec<RuntimeView>,
    /// Explicit widening/degradation disclosures.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub widenings: Vec<WideningDisclosure>,
    /// Warnings and degradation notices.
    pub warnings: Vec<QueryWarning>,
    /// Kernel scheduler degradation surfaced by attached runtime inference, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kernel_degraded_reason: Option<String>,
    /// Bitemporal valid-time coordinate used for this query, if temporal.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub valid_as_of: Option<String>,
    /// Bitemporal recorded-time ceiling used for this query, if temporal.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recorded_as_of: Option<String>,
    /// How the temporal coordinates were resolved: "exact", "downgraded", "none".
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub temporal_mode: Option<String>,
}

/// Observability record for a projection lifecycle action.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectionTrace {
    /// Canonical trace context.
    pub trace_ctx: TraceCtx,
    /// Phase status: compatibility / migration-only.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace_id: Option<String>,
    /// Which projection was affected.
    pub projection_id: String,
    /// Scope of the projection.
    pub scope: ScopeKey,
    /// What action was taken.
    pub action: ProjectionAction,
    /// Duration in milliseconds (for builds).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<u64>,
    /// Number of source items processed (for builds).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_count: Option<usize>,
    /// Whether the action succeeded.
    pub success: bool,
    /// Error message if action failed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// Resulting health after the action.
    pub resulting_health: ProjectionHealth,
    /// Stale cause if the projection is now stale.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stale_cause: Option<StaleCause>,
}

/// What lifecycle action was taken on a projection.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectionAction {
    Build,
    Invalidate,
    Remove,
    Clear,
    Failure,
}

impl QueryTrace {
    /// Create a trace from query pipeline components.
    #[allow(clippy::too_many_arguments)]
    pub fn from_pipeline(
        trace_ctx: TraceCtx,
        scope: ScopeKey,
        classification: ClassifyResult,
        plan: RoutePlan,
        leg_timings_ms: Vec<u64>,
        total_duration: Duration,
        merged: &MergedResults,
        warnings: Vec<QueryWarning>,
    ) -> Self {
        let legacy_id = Some(trace_ctx.to_legacy_trace_id().to_string());
        let (requested_views, executed_views) = merged_trace_plan_views(&warnings, &plan);
        let widenings = build_widenings(&plan, &warnings);
        Self {
            trace_ctx,
            trace_id: legacy_id,
            scope,
            classification,
            plan,
            leg_timings_ms,
            total_duration,
            raw_result_count: merged.total_raw,
            merged_result_count: merged.results.len(),
            duplicates_fused: merged.duplicates_fused,
            requested_views: collect_requested_views(&requested_views),
            executed_views: collect_requested_views(&executed_views),
            widenings,
            warnings,
            kernel_degraded_reason: None,
            valid_as_of: None,
            recorded_as_of: None,
            temporal_mode: None,
        }
    }

    /// Whether any temporal requests were downgraded.
    pub fn has_temporal_downgrade(&self) -> bool {
        self.warnings
            .iter()
            .any(|w| matches!(w, QueryWarning::TemporalDowngradedToHybrid { .. }))
    }

    /// Whether scope enforcement was only partial.
    pub fn has_scope_enforcement_warning(&self) -> bool {
        self.warnings
            .iter()
            .any(|w| matches!(w, QueryWarning::ScopePartiallyEnforced { .. }))
    }

    /// Whether import staleness was detected.
    pub fn has_import_staleness_warning(&self) -> bool {
        self.warnings
            .iter()
            .any(|w| matches!(w, QueryWarning::ProjectionImportStale { .. }))
    }

    /// Whether any degradation warnings were emitted.
    pub fn is_degraded(&self) -> bool {
        !self.warnings.is_empty()
    }

    /// Build a versioned provenance snapshot from this trace for cross-crate audit.
    pub fn runtime_query_provenance(&self) -> RuntimeQueryProvenanceV1 {
        RuntimeQueryProvenanceV1 {
            schema_version: "runtime_query_provenance_v1".into(),
            trace_ctx: self.trace_ctx.clone(),
            scope: self.scope.clone(),
            classification_mode: self.classification.mode.kind().into(),
            classification_reason: self.classification.reason.clone(),
            query: self.plan.query.clone(),
            requested_views: self.requested_views.clone(),
            executed_views: self.executed_views.clone(),
            widenings: self.widenings.clone(),
            warnings: self.warnings.clone(),
            kernel_degraded_reason: self.kernel_degraded_reason.clone(),
            leg_strategies: self
                .plan
                .legs
                .iter()
                .map(|leg| leg.strategy.kind().into())
                .collect(),
            leg_timings_ms: self.leg_timings_ms.clone(),
            total_duration_ms: self.total_duration.as_millis() as u64,
            raw_result_count: self.raw_result_count,
            merged_result_count: self.merged_result_count,
            duplicates_fused: self.duplicates_fused,
            valid_as_of: self.valid_as_of.clone(),
            recorded_as_of: self.recorded_as_of.clone(),
            temporal_mode: self.temporal_mode.clone(),
        }
    }
}

/// Versioned provenance record summarizing a full query execution for downstream audit.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "RuntimeQueryProvenanceV1")]
pub struct RuntimeQueryProvenanceV1 {
    pub schema_version: String,
    pub trace_ctx: TraceCtx,
    pub scope: ScopeKey,
    pub classification_mode: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub classification_reason: Option<String>,
    pub query: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub requested_views: Vec<RuntimeView>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub executed_views: Vec<RuntimeView>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub widenings: Vec<WideningDisclosure>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub warnings: Vec<QueryWarning>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kernel_degraded_reason: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub leg_strategies: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub leg_timings_ms: Vec<u64>,
    pub total_duration_ms: u64,
    pub raw_result_count: usize,
    pub merged_result_count: usize,
    pub duplicates_fused: usize,
    /// Bitemporal valid-time coordinate used for this query, if temporal.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub valid_as_of: Option<String>,
    /// Bitemporal recorded-time ceiling used for this query, if temporal.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recorded_as_of: Option<String>,
    /// How the temporal coordinates were resolved: "exact", "downgraded", "none".
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub temporal_mode: Option<String>,
}

fn strategy_view(strategy: &RetrievalStrategy) -> RuntimeView {
    match strategy {
        RetrievalStrategy::HybridSearch => RuntimeView::Semantic,
        RetrievalStrategy::EntitySearch { .. } => RuntimeView::Entity,
        RetrievalStrategy::TemporalSearch { .. } => RuntimeView::Temporal,
    }
}

fn collect_requested_views(views: &[RuntimeView]) -> Vec<RuntimeView> {
    let mut deduped = Vec::new();
    for view in views {
        if !deduped.contains(view) {
            deduped.push(view.clone());
        }
    }
    deduped
}

fn merged_trace_plan_views(
    warnings: &[QueryWarning],
    plan: &RoutePlan,
) -> (Vec<RuntimeView>, Vec<RuntimeView>) {
    let requested = plan
        .legs
        .iter()
        .map(|leg| strategy_view(&leg.strategy))
        .collect::<Vec<_>>();
    let mut executed = requested.clone();
    if warnings
        .iter()
        .any(|warning| matches!(warning, QueryWarning::TemporalDowngradedToHybrid { .. }))
    {
        for (index, leg) in plan.legs.iter().enumerate() {
            if matches!(leg.strategy, RetrievalStrategy::TemporalSearch { .. }) {
                executed[index] = RuntimeView::Semantic;
            }
        }
    }
    (requested, executed)
}

fn build_widenings(plan: &RoutePlan, warnings: &[QueryWarning]) -> Vec<WideningDisclosure> {
    let mut widenings = Vec::new();
    for (index, leg) in plan.legs.iter().enumerate() {
        match &leg.strategy {
            RetrievalStrategy::TemporalSearch { temporal_expr } => {
                if warnings.iter().any(|warning| {
                    matches!(
                        warning,
                        QueryWarning::TemporalDowngradedToHybrid { temporal_expr: seen }
                            if seen == temporal_expr
                    )
                }) {
                    widenings.push(WideningDisclosure {
                        leg_index: index,
                        requested_view: RuntimeView::Temporal,
                        executed_view: RuntimeView::Semantic,
                        reason: "temporal route degraded to semantic hybrid execution".into(),
                    });
                }
            }
            RetrievalStrategy::EntitySearch { mention } => {
                if warnings.iter().any(|warning| {
                    matches!(
                        warning,
                        QueryWarning::EntityScopeFallback { mention: seen, .. } if seen == mention
                    )
                }) {
                    widenings.push(WideningDisclosure {
                        leg_index: index,
                        requested_view: RuntimeView::Entity,
                        executed_view: RuntimeView::Entity,
                        reason: "entity resolution widened to a broader candidate scope".into(),
                    });
                }
            }
            RetrievalStrategy::HybridSearch => {}
        }
    }
    if let Some(scope_warning) = warnings
        .iter()
        .find(|warning| matches!(warning, QueryWarning::ScopePartiallyEnforced { .. }))
    {
        let reason = match scope_warning {
            QueryWarning::ScopePartiallyEnforced { .. } => {
                "adapter enforced namespace-only scope; wider scope was disclosed".to_string()
            }
            _ => unreachable!(),
        };
        widenings.push(WideningDisclosure {
            leg_index: 0,
            requested_view: RuntimeView::Semantic,
            executed_view: RuntimeView::Semantic,
            reason,
        });
    }
    widenings
}

/// Serde helper for Duration as milliseconds.
mod duration_ms {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::time::Duration;

    pub fn serialize<S: Serializer>(d: &Duration, s: S) -> Result<S::Ok, S::Error> {
        d.as_millis().serialize(s)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Duration, D::Error> {
        let ms = u64::deserialize(d)?;
        Ok(Duration::from_millis(ms))
    }
}
