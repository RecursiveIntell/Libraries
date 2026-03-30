/// Error types for the knowledge-runtime crate.
///
/// All errors flow through [`RuntimeError`]. Upstream `MemoryError` converts
/// automatically via `#[from]`.
#[derive(Debug, thiserror::Error)]
pub enum RuntimeError {
    /// Propagated from the upstream semantic-memory store.
    #[error("memory store error: {0}")]
    Memory(#[from] semantic_memory::MemoryError),

    /// Configuration is invalid.
    #[error("invalid config for '{field}': {reason}")]
    InvalidConfig { field: &'static str, reason: String },

    /// Entity not found in registry.
    #[error("entity not found: {id}")]
    EntityNotFound { id: String },

    /// Entity registry is full.
    #[error("entity registry full (capacity {capacity})")]
    RegistryFull { capacity: usize },

    /// COR-005: Entity name/alias collision with an existing entity.
    #[error("entity collision: name '{name}' already registered to {existing_id}, cannot assign to {new_id}")]
    EntityCollision {
        name: String,
        existing_id: crate::ids::EntityId,
        new_id: crate::ids::EntityId,
    },

    /// A projection is stale or missing and cannot serve the request.
    #[error("projection unavailable: {id} ({reason})")]
    ProjectionUnavailable { id: String, reason: String },

    /// Adapter failed to connect or communicate with the backing store.
    #[error("adapter error: {0}")]
    Adapter(String),

    /// Scope dimensions beyond namespace were requested but cannot be fully
    /// enforced by the upstream adapter. Results may include items outside
    /// the requested domain/workspace_id/repo_id.
    #[error("scope not fully enforced: upstream adapter only filters by namespace; unpushed dimensions: {unpushed_dimensions:?}")]
    ScopeNotFullyEnforced {
        /// Which scope dimensions could not be pushed to the upstream adapter.
        unpushed_dimensions: Vec<String>,
    },

    /// Temporal search was requested but results lack temporal metadata
    /// (valid_from/valid_to). In strict temporal mode, this is an error
    /// rather than a silent downgrade.
    #[error("temporal search not supported: results lack temporal metadata for expression '{temporal_expr}'")]
    TemporalNotSupported {
        /// The temporal expression from the query that could not be executed.
        temporal_expr: String,
    },
}

impl RuntimeError {
    /// Stable string discriminant for programmatic matching.
    pub fn kind(&self) -> &'static str {
        match self {
            Self::Memory(_) => "memory",
            Self::InvalidConfig { .. } => "invalid_config",
            Self::EntityNotFound { .. } => "entity_not_found",
            Self::RegistryFull { .. } => "registry_full",
            Self::ProjectionUnavailable { .. } => "projection_unavailable",
            Self::Adapter(_) => "adapter",
            Self::EntityCollision { .. } => "entity_collision",
            Self::ScopeNotFullyEnforced { .. } => "scope_not_fully_enforced",
            Self::TemporalNotSupported { .. } => "temporal_not_supported",
        }
    }
}
