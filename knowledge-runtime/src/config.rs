use crate::error::RuntimeError;
use crate::ids::Scope;
use serde::{Deserialize, Serialize};

/// Runtime-level hint for semantic-memory candidate backend selection.
///
/// This is intentionally a hint/trace value only. Knowledge Runtime does not
/// construct or depend on proveKV/poly-kv backends directly.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeCandidateBackendHint {
    #[default]
    SemanticMemoryDefault,
    ProveKvPoolCandidate,
}

/// Top-level runtime configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeConfig {
    /// Default scope for queries that don't supply one.
    pub default_scope: Scope,

    /// Query pipeline settings.
    #[serde(default)]
    pub query: QueryConfig,

    /// Entity registry settings.
    #[serde(default)]
    pub entity: EntityConfig,

    /// Projection lifecycle settings.
    #[serde(default)]
    pub projection: ProjectionConfig,

    /// When true, temporal queries that cannot be executed on a supported
    /// projection-backed route return `RuntimeError::TemporalNotSupported`
    /// instead of downgrading to hybrid retrieval with a warning.
    ///
    /// Default: `false` (downgrade with warning).
    #[serde(default)]
    pub strict_temporal: bool,

    /// When true, queries whose scope includes dimensions beyond namespace
    /// (domain, workspace_id, repo_id) return `RuntimeError::ScopeNotFullyEnforced`
    /// instead of proceeding with namespace-only hybrid fallback.
    ///
    /// Default: `false` (warn but proceed).
    #[serde(default)]
    pub strict_scope: bool,
}

impl RuntimeConfig {
    /// Validate and normalize configuration. Returns error on invalid values.
    pub fn normalize_and_validate(&mut self) -> Result<(), RuntimeError> {
        if self.default_scope.namespace.is_empty() {
            return Err(RuntimeError::InvalidConfig {
                field: "default_scope.namespace",
                reason: "namespace must not be empty".into(),
            });
        }
        self.query.normalize_and_validate()?;
        self.projection.normalize_and_validate()?;
        Ok(())
    }
}

/// Query pipeline tuning.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryConfig {
    /// Maximum number of search results per route leg.
    pub max_results_per_leg: usize,
    /// Maximum number of route legs.
    pub max_route_legs: usize,
    /// Default result limit when caller does not specify.
    pub default_limit: usize,
    /// Semantic-memory candidate backend hint to disclose in runtime traces.
    ///
    /// The owning semantic-memory store remains authoritative for actual
    /// backend configuration and exact rerank enforcement.
    #[serde(default)]
    pub candidate_backend_hint: RuntimeCandidateBackendHint,
}

impl Default for QueryConfig {
    fn default() -> Self {
        Self {
            max_results_per_leg: 20,
            max_route_legs: 4,
            default_limit: 10,
            candidate_backend_hint: RuntimeCandidateBackendHint::default(),
        }
    }
}

impl QueryConfig {
    fn normalize_and_validate(&mut self) -> Result<(), RuntimeError> {
        if self.max_results_per_leg == 0 {
            self.max_results_per_leg = 20;
        }
        if self.max_route_legs == 0 {
            self.max_route_legs = 4;
        }
        if self.default_limit == 0 {
            self.default_limit = 10;
        }
        Ok(())
    }
}

/// Entity registry settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityConfig {
    /// Maximum number of aliases per entity.
    pub max_aliases: usize,
    /// Maximum number of entities in the registry (across all scopes).
    pub max_entities: usize,
}

impl Default for EntityConfig {
    fn default() -> Self {
        Self {
            max_aliases: 16,
            max_entities: 10_000,
        }
    }
}

/// Projection lifecycle settings.
///
/// **Note**: Projection persistence is not implemented in this crate today.
/// The `persist` field is retained solely for serde backward compatibility
/// with existing config files; setting it to `true` is rejected at validation
/// time because runtime projections remain derived and rebuildable.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectionConfig {
    /// Staleness threshold in seconds. Projections older than this are
    /// candidates for rebuild.
    pub staleness_threshold_secs: u64,
    /// Import staleness threshold in seconds. If the last successful import
    /// for a namespace is older than this, the runtime emits a
    /// `ProjectionImportStale` warning during query execution.
    /// Set to 0 to disable import staleness checking.
    pub import_staleness_threshold_secs: u64,
    /// Whether to persist projections to SQLite.
    ///
    /// Not implemented in this crate today. The runtime's projections
    /// are derived, non-authoritative, and rebuildable from upstream
    /// `semantic-memory` state, so enabling durable persistence here is
    /// outside the current contract.
    ///
    /// This field is retained only for serde backward compatibility with
    /// config files that already include it. Setting `persist: true` will
    /// cause config validation to return `InvalidConfig`.
    #[serde(default)]
    pub persist: bool,
}

impl Default for ProjectionConfig {
    fn default() -> Self {
        Self {
            staleness_threshold_secs: 3600,
            import_staleness_threshold_secs: 3600,
            persist: false,
        }
    }
}

impl ProjectionConfig {
    fn normalize_and_validate(&mut self) -> Result<(), RuntimeError> {
        if self.persist {
            return Err(RuntimeError::InvalidConfig {
                field: "projection.persist",
                reason: "projection persistence is not implemented in this crate today; \
                         projections are derived, non-authoritative, and rebuildable \
                         from upstream semantic-memory state; \
                         set persist=false or omit the field"
                    .into(),
            });
        }
        if self.staleness_threshold_secs == 0 {
            self.staleness_threshold_secs = 3600;
        }
        Ok(())
    }
}
