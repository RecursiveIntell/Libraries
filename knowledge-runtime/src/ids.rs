// Re-export shared identity types from stack-ids (single source of truth).
//
// EntityId, Scope, and ScopeKey are defined in `stack-ids` and re-exported
// here so that all knowledge-runtime internal code can continue to use
// `crate::ids::EntityId` etc. without change.
//
// ProjectionId and ProjectionKind remain runtime-owned because they carry
// runtime-specific structure (kind enum + key + scope) that goes beyond
// the opaque string ID in stack-ids.

pub use stack_ids::{EntityId, Scope, ScopeKey};

use serde::{Deserialize, Serialize};

// --- Projection Identity ---

/// Identifies a specific projection instance.
///
/// A projection is identified by its kind, a key within the kind, and
/// the scope it belongs to. Two projections with the same kind and key
/// but different scopes are distinct.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ProjectionId {
    /// Projection kind.
    pub kind: ProjectionKind,
    /// Unique key within the kind (often an `EntityId` or scope key).
    pub key: String,
    /// Scope this projection belongs to.
    pub scope: ScopeKey,
}

impl ProjectionId {
    pub fn new(kind: ProjectionKind, key: impl Into<String>, scope: ScopeKey) -> Self {
        Self {
            kind,
            key: key.into(),
            scope,
        }
    }
}

impl std::fmt::Display for ProjectionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}@{}", self.kind.as_str(), self.key, self.scope)
    }
}

/// Discriminant for projection types.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectionKind {
    /// Entity registry projection.
    Entity,
    /// Temporal claim projection.
    Temporal,
    /// Route statistics projection.
    RouteStats,
    /// Caller-defined projection kind.
    Custom(String),
}

impl ProjectionKind {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Entity => "entity",
            Self::Temporal => "temporal",
            Self::RouteStats => "route_stats",
            Self::Custom(s) => s.as_str(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scope_key_equality() {
        let s1 = Scope::new("ns").with_repo("repo-a");
        let s2 = Scope::new("ns").with_repo("repo-a");
        assert_eq!(s1.key(), s2.key());
    }

    #[test]
    fn scope_key_inequality_different_repo() {
        let s1 = Scope::new("ns").with_repo("repo-a");
        let s2 = Scope::new("ns").with_repo("repo-b");
        assert_ne!(s1.key(), s2.key());
    }

    #[test]
    fn scope_key_display() {
        let s = Scope::new("prod")
            .with_domain("code")
            .with_workspace("ws1")
            .with_repo("myrepo");
        assert_eq!(s.key().to_string(), "prod/code@ws1#myrepo");
    }

    #[test]
    fn projection_id_includes_scope() {
        let sk = ScopeKey::namespace_only("ns");
        let pid = ProjectionId::new(ProjectionKind::Entity, "test", sk);
        let display = pid.to_string();
        assert!(display.contains("entity"));
        assert!(display.contains("test"));
        assert!(display.contains("ns"));
    }
}
