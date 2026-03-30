//! # knowledge-runtime
//!
//! Hardened scaffold / bounded first-pass orchestration layer for `semantic-memory`.
//!
//! This crate provides intent classification, route planning, entity resolution,
//! result merging, and projection lifecycle tracking on top of `semantic-memory`.
//!
//! ## Authority Model
//!
//! This crate NEVER owns source truth. All facts, episodes, documents, and
//! conversation messages live in `semantic-memory`. This crate owns only:
//! - Derived projections (entity registry, projection tracker)
//! - Query pipeline logic (classification, planning, merge)
//! - Observability traces
//!
//! Deleting any projection forces recomputation but causes no data loss.
//!
//! ## Query Pipeline
//!
//! ```text
//! query text
//!   -> classify (QueryMode: semantic | entity | temporal | mixed)
//!   -> plan (RoutePlan with legs)
//!   -> execute (per-leg retrieval via semantic-memory adapter)
//!   -> merge (fuse duplicates with provenance, normalize, boost, rank, truncate)
//!   -> results + QueryTrace (including degradation warnings)
//! ```
//!
//! ## Implemented Now
//!
//! - **Rule-based intent classification**: heuristic extraction of entity mentions
//!   (`@name`, `"quoted"`), temporal keywords, and mixed signals.
//! - **Route planning**: translates classified query modes into retrieval legs.
//! - **Projection-backed retrieval for imported knowledge**: imported claim,
//!   relation, and episode rows are queried directly through `semantic-memory`'s
//!   public projection APIs.
//! - **Hybrid fallback where projection substrate is absent**: namespace-only
//!   BM25/vector retrieval remains available when a supported projection route
//!   is unavailable for the queried leg.
//! - **Entity search**: scope-aware registry resolution plus bounded imported
//!   alias candidate expansion before fallback.
//! - **Deterministic result merge with provenance fusion**: duplicates across legs are
//!   fused (not discarded), retaining union of source legs and per-leg scores. Tie-breaking
//!   is three-tier: score > source leg count > lexicographic identity key. Multi-leg
//!   support gets a configurable score boost.
//! - **Scope-aware entity registry**: entities are partitioned by `ScopeKey` (namespace +
//!   domain + workspace_id + repo_id). Resolution falls back from narrow scope to
//!   namespace-only when needed, with fallback visible in results.
//! - **Scope enforcement transparency**: projection-backed routes enforce full
//!   scope directly. `ScopePartiallyEnforced` is emitted only when execution
//!   must fall back to namespace-only hybrid retrieval.
//! - **Scope-aware code identity**: code entity IDs include repo_id/workspace_id/namespace
//!   so the same qualified path in different repos yields different IDs.
//! - **Projection lifecycle observability**: health, staleness (time-based + explicit),
//!   invalidation by scope/kind/id, stale cause, and version metadata. The tracker
//!   currently emits `Healthy`, `Stale`, and `Missing`; reserved compatibility
//!   variants remain in the public enum but are not emitted by the tracker today.
//! - **Import staleness detection**: the runtime checks `semantic-memory`'s last import
//!   timestamp for the queried namespace and emits `ProjectionImportStale` warnings when
//!   data may be outdated. Import freshness is surfaced as a query warning today,
//!   not as a dedicated tracker health transition.
//! - **Query traces with degradation warnings**: unsupported temporal downgrade,
//!   partial scope enforcement on hybrid fallback, entity scope fallback, and
//!   import staleness are all surfaced in `QueryTrace::warnings`. Helper methods
//!   (`is_degraded()`, `has_temporal_downgrade()`, etc.) make warnings inspectable.
//! - **Strict enforcement modes (KR-001/KR-002)**: `strict_temporal` and `strict_scope`
//!   config flags convert degradation warnings into hard errors for callers that
//!   require guaranteed temporal/scope semantics rather than best-effort.
//! - **Rebuild driver interface (KR-003)**: the [`RebuildDriver`] trait provides a
//!   defined contract for external callers to drive projection rebuilds. The
//!   [`rebuild_stale`] convenience function polls the tracker and drives rebuilds
//!   automatically. The tracker remains observability-only.
//!
//! ## Not Real Yet (Deferred)
//!
//! - **Range temporal expressions**: projection-backed temporal execution is
//!   implemented for concrete as-of markers such as `today`, `yesterday`,
//!   `last week`, and `this month`. Range-style phrases such as `before`,
//!   `after`, `since`, and `between` still degrade explicitly.
//! - **Namespace-only hybrid scope**: `semantic-memory` hybrid search only
//!   pushes namespace. When runtime must use that path for a scoped query, the
//!   limitation is surfaced via `QueryWarning::ScopePartiallyEnforced` or
//!   `RuntimeError::ScopeNotFullyEnforced`.
//! - **Projection persistence**: the `persist` config flag is rejected at
//!   validation time. Durable runtime projections are not implemented in this
//!   crate today.
//! - **Projection rebuild execution**: the tracker is observability-only; it records
//!   build/failure events but does not schedule, execute, or retry rebuilds.
//!   External callers implement the [`RebuildDriver`] trait to drive actual rebuilds
//!   and report outcomes back to the tracker (KR-003).
//! - **LLM-based intent classification**: classifier is rule-based heuristic.
//! - **Evidence handle external dereference**: this layer does not resolve
//!   evidence handles against external stores.
//! - **Advanced fuzzy entity resolution**: bounded candidate expansion exists,
//!   but there is no embedding-based or graph-ML entity resolver.

pub mod adapters;
pub mod config;
pub mod entity;
pub mod error;
pub mod evidence;
pub mod ids;
mod inference;
pub mod obs;
pub mod projection;
pub mod query;
pub mod runtime;
pub mod temporal;
pub mod views;

// Re-export primary public types.
pub use config::RuntimeConfig;
pub use error::RuntimeError;
pub use ids::{EntityId, ProjectionId, ProjectionKind, Scope, ScopeKey};
pub use inference::{InferenceAdvisory, InferenceExplanation, RiskGateDecision};
pub use runtime::{
    EntityCacheHandle, KnowledgeRuntime, ProjectedPromotionState, ProjectedVerificationLifecycle,
    ProjectedVerificationSummary,
};

// Re-export commonly used inner types.
pub use entity::code_ids::{CodeEntity, CodeEntityKind};
pub use entity::registry::{Entity, EntityKind, EntityRegistry, MatchQuality, ResolveResult};
pub use evidence::support::{AnswerBundle, EvidenceItem, EvidenceRelevance, SearchEvidenceBundle};
pub use obs::trace::{
    ProjectionAction, ProjectionTrace, QueryTrace, QueryWarning, RuntimeQueryProvenanceV1,
};
pub use projection::lifecycle::{
    InvalidationEvent, ProjectionActionResult, ProjectionHealth, ProjectionMeta, ProjectionTracker,
    ProjectionVersion, StaleCause,
};
pub use projection::rebuild::{rebuild_stale, RebuildDriver, RebuildOutcome};
pub use query::classify::{ClassifyResult, QueryMode};
pub use query::merge::{MergePolicy, MergedItem, MergedResults};
pub use query::route::{LegFilter, RetrievalStrategy, RouteLeg, RoutePlan};
pub use temporal::claims::{TemporalClaim, TemporalContradictionStatus};
pub use views::{
    CompiledObligationRuntimeViewV1, CompositionConflictRuntimeViewV1, ContinuityRuntimeViewV1,
    DelegationRuntimeViewV1, DeployabilityRuntimeViewV1, EffectRuntimeViewV1,
    EffectiveConstitutionViewV1, PolicyImpactDiffRuntimeViewV1,
};
