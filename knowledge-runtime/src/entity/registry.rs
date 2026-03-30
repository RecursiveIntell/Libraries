//! Scope-aware in-memory entity registry with alias resolution and fallback.

use crate::error::RuntimeError;
use crate::ids::{EntityId, ScopeKey};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Entity kind discriminant.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EntityKind {
    /// A person (contributor, user, author).
    Person,
    /// A code entity (function, type, module, etc.).
    Code,
    /// A project, product, or system.
    Project,
    /// A concept or topic.
    Concept,
    /// Untyped / caller-defined.
    Custom(String),
}

/// A registered entity with aliases and metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    /// Canonical identity.
    pub id: EntityId,
    /// Human-readable canonical name.
    pub canonical_name: String,
    /// Entity kind.
    pub kind: EntityKind,
    /// Scope this entity belongs to.
    pub scope: ScopeKey,
    /// Alternative names that resolve to this entity.
    pub aliases: Vec<String>,
    /// Optional structured metadata.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

/// Match quality when resolving a mention to an entity.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MatchQuality {
    /// Exact canonical name or ID match within the requested scope.
    ExactCanonical,
    /// Matched through an alias within the requested scope.
    ExactAlias,
    /// Matched in a broader/different scope as a fallback.
    /// The `actual_scope` field in `ResolveResult` shows where it was found.
    ScopedFallback,
    /// No registered match — the mention is an unresolved candidate.
    Unresolved,
}

/// Result of resolving a mention against the registry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolveResult {
    /// The primary resolved entity, if found.
    pub entity: Option<Entity>,
    /// How the match was made.
    pub quality: MatchQuality,
    /// The original mention that was resolved.
    pub mention: String,
    /// Additional candidates if the mention was ambiguous across scopes.
    pub alternatives: Vec<Entity>,
    /// The scope that was queried.
    pub queried_scope: ScopeKey,
}

/// Composite key for scope-partitioned entity indexes.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct ScopedName {
    scope: ScopeKey,
    lower_name: String,
}

/// Scope-aware in-memory entity registry.
///
/// Entities are partitioned by scope. Resolution first tries the exact scope,
/// then falls back to namespace-only scope if the exact scope yields nothing.
/// Cross-scope collisions are avoided by using `ScopeKey` in all indexes.
///
/// This registry owns entity data as a derived projection — it does NOT
/// own source truth. Entities can be rebuilt from upstream stores.
#[derive(Debug)]
pub struct EntityRegistry {
    /// Entities by ID.
    entities: BTreeMap<EntityId, Entity>,
    /// Index: (scope, lowercase canonical name) -> entity ID.
    name_index: BTreeMap<ScopedName, EntityId>,
    /// Index: (scope, lowercase alias) -> entity ID.
    alias_index: BTreeMap<ScopedName, EntityId>,
    /// Maximum aliases per entity.
    max_aliases: usize,
    /// Maximum total entities across all scopes.
    max_entities: usize,
}

impl EntityRegistry {
    /// Create a new registry with bounded alias and entity limits.
    pub fn new(max_aliases: usize, max_entities: usize) -> Self {
        Self {
            entities: BTreeMap::new(),
            name_index: BTreeMap::new(),
            alias_index: BTreeMap::new(),
            max_aliases,
            max_entities,
        }
    }

    /// Register a new entity. Returns error if the registry is full.
    pub fn register(&mut self, entity: Entity) -> Result<(), RuntimeError> {
        if self.entities.len() >= self.max_entities && !self.entities.contains_key(&entity.id) {
            return Err(RuntimeError::RegistryFull {
                capacity: self.max_entities,
            });
        }

        if entity.aliases.len() > self.max_aliases {
            return Err(RuntimeError::InvalidConfig {
                field: "entity.aliases",
                reason: format!(
                    "entity has {} aliases, max is {}",
                    entity.aliases.len(),
                    self.max_aliases
                ),
            });
        }

        // Remove old index entries if updating
        if let Some(old) = self.entities.remove(&entity.id) {
            let old_key = ScopedName {
                scope: old.scope.clone(),
                lower_name: old.canonical_name.to_lowercase(),
            };
            self.name_index.remove(&old_key);
            for alias in &old.aliases {
                let alias_key = ScopedName {
                    scope: old.scope.clone(),
                    lower_name: alias.to_lowercase(),
                };
                self.alias_index.remove(&alias_key);
            }
        }

        // COR-005: Check for canonical/alias collisions before inserting.
        // Reject if another entity (different ID) already owns this name or alias.
        let name_key = ScopedName {
            scope: entity.scope.clone(),
            lower_name: entity.canonical_name.to_lowercase(),
        };
        if let Some(existing_id) = self.name_index.get(&name_key) {
            if existing_id != &entity.id {
                return Err(RuntimeError::EntityCollision {
                    name: entity.canonical_name.clone(),
                    existing_id: existing_id.clone(),
                    new_id: entity.id.clone(),
                });
            }
        }
        for alias in &entity.aliases {
            let alias_key = ScopedName {
                scope: entity.scope.clone(),
                lower_name: alias.to_lowercase(),
            };
            if let Some(existing_id) = self.alias_index.get(&alias_key) {
                if existing_id != &entity.id {
                    return Err(RuntimeError::EntityCollision {
                        name: alias.clone(),
                        existing_id: existing_id.clone(),
                        new_id: entity.id.clone(),
                    });
                }
            }
            // Also check if alias collides with another entity's canonical name
            if let Some(existing_id) = self.name_index.get(&alias_key) {
                if existing_id != &entity.id {
                    return Err(RuntimeError::EntityCollision {
                        name: alias.clone(),
                        existing_id: existing_id.clone(),
                        new_id: entity.id.clone(),
                    });
                }
            }
        }

        // Insert index entries (collision-free at this point)
        self.name_index.insert(name_key, entity.id.clone());

        for alias in &entity.aliases {
            let alias_key = ScopedName {
                scope: entity.scope.clone(),
                lower_name: alias.to_lowercase(),
            };
            self.alias_index.insert(alias_key, entity.id.clone());
        }

        self.entities.insert(entity.id.clone(), entity);
        Ok(())
    }

    /// Remove an entity by ID.
    pub fn remove(&mut self, id: &EntityId) -> Option<Entity> {
        if let Some(entity) = self.entities.remove(id) {
            let name_key = ScopedName {
                scope: entity.scope.clone(),
                lower_name: entity.canonical_name.to_lowercase(),
            };
            self.name_index.remove(&name_key);
            for alias in &entity.aliases {
                let alias_key = ScopedName {
                    scope: entity.scope.clone(),
                    lower_name: alias.to_lowercase(),
                };
                self.alias_index.remove(&alias_key);
            }
            Some(entity)
        } else {
            None
        }
    }

    /// Look up an entity by ID.
    pub fn get(&self, id: &EntityId) -> Option<&Entity> {
        self.entities.get(id)
    }

    /// Resolve a raw mention within a specific scope.
    ///
    /// Resolution order:
    /// 1. Exact canonical name match in the given scope -> `ExactCanonical`
    /// 2. Exact alias match in the given scope -> `ExactAlias`
    /// 3. Namespace-only fallback (if scope has more dimensions) -> `ScopedFallback`
    /// 4. Unresolved
    ///
    /// Alternatives from other scopes with the same namespace are included
    /// in `ResolveResult::alternatives` when the primary match is `Unresolved`
    /// or `ScopedFallback`, so callers can see potential ambiguity.
    pub fn resolve(&self, mention: &str, scope: &ScopeKey) -> ResolveResult {
        let lower = mention.to_lowercase();

        // 1. Exact name match in given scope
        let exact_name_key = ScopedName {
            scope: scope.clone(),
            lower_name: lower.clone(),
        };
        if let Some(id) = self.name_index.get(&exact_name_key) {
            if let Some(entity) = self.entities.get(id) {
                return ResolveResult {
                    entity: Some(entity.clone()),
                    quality: MatchQuality::ExactCanonical,
                    mention: mention.to_string(),
                    alternatives: Vec::new(),
                    queried_scope: scope.clone(),
                };
            }
        }

        // 2. Exact alias match in given scope
        let exact_alias_key = ScopedName {
            scope: scope.clone(),
            lower_name: lower.clone(),
        };
        if let Some(id) = self.alias_index.get(&exact_alias_key) {
            if let Some(entity) = self.entities.get(id) {
                return ResolveResult {
                    entity: Some(entity.clone()),
                    quality: MatchQuality::ExactAlias,
                    mention: mention.to_string(),
                    alternatives: Vec::new(),
                    queried_scope: scope.clone(),
                };
            }
        }

        // 3. Namespace-only fallback if scope has more dimensions
        let has_extra_dims =
            scope.domain.is_some() || scope.workspace_id.is_some() || scope.repo_id.is_some();

        if has_extra_dims {
            let ns_only = ScopeKey::namespace_only(&scope.namespace);
            let ns_name_key = ScopedName {
                scope: ns_only.clone(),
                lower_name: lower.clone(),
            };
            if let Some(id) = self.name_index.get(&ns_name_key) {
                if let Some(entity) = self.entities.get(id) {
                    let alternatives = self.find_alternatives(mention, scope, Some(id));
                    return ResolveResult {
                        entity: Some(entity.clone()),
                        quality: MatchQuality::ScopedFallback,
                        mention: mention.to_string(),
                        alternatives,
                        queried_scope: scope.clone(),
                    };
                }
            }
            let ns_alias_key = ScopedName {
                scope: ns_only,
                lower_name: lower.clone(),
            };
            if let Some(id) = self.alias_index.get(&ns_alias_key) {
                if let Some(entity) = self.entities.get(id) {
                    let alternatives = self.find_alternatives(mention, scope, Some(id));
                    return ResolveResult {
                        entity: Some(entity.clone()),
                        quality: MatchQuality::ScopedFallback,
                        mention: mention.to_string(),
                        alternatives,
                        queried_scope: scope.clone(),
                    };
                }
            }
        }

        // 4. Unresolved — gather alternatives from same namespace
        let alternatives = self.find_alternatives(mention, scope, None);

        ResolveResult {
            entity: None,
            quality: MatchQuality::Unresolved,
            mention: mention.to_string(),
            alternatives,
            queried_scope: scope.clone(),
        }
    }

    /// Find entities matching `mention` in the same namespace but different
    /// sub-scopes, excluding `exclude_id`.
    fn find_alternatives(
        &self,
        mention: &str,
        scope: &ScopeKey,
        exclude_id: Option<&EntityId>,
    ) -> Vec<Entity> {
        let lower = mention.to_lowercase();
        let mut seen = std::collections::HashSet::new();
        let mut alts = Vec::new();

        for (key, id) in self.name_index.iter().chain(self.alias_index.iter()) {
            if key.lower_name == lower
                && key.scope.namespace == scope.namespace
                && key.scope != *scope
                && (exclude_id != Some(id))
                && seen.insert(id.clone())
            {
                if let Some(entity) = self.entities.get(id) {
                    alts.push(entity.clone());
                }
            }
        }
        alts
    }

    /// Number of registered entities.
    pub fn len(&self) -> usize {
        self.entities.len()
    }

    /// Whether the registry is empty.
    pub fn is_empty(&self) -> bool {
        self.entities.is_empty()
    }

    /// Iterate over all entities.
    pub fn iter(&self) -> impl Iterator<Item = &Entity> {
        self.entities.values()
    }

    /// Iterate over entities in a specific scope.
    pub fn iter_scope<'a>(&'a self, scope: &'a ScopeKey) -> impl Iterator<Item = &'a Entity> + 'a {
        self.entities.values().filter(move |e| &e.scope == scope)
    }

    /// Remove all entities across all scopes (for full cache rebuild).
    pub fn clear_all(&mut self) {
        self.entities.clear();
        self.name_index.clear();
        self.alias_index.clear();
    }

    /// Remove all entities in a specific scope.
    pub fn clear_scope(&mut self, scope: &ScopeKey) {
        let ids_to_remove: Vec<EntityId> = self
            .entities
            .values()
            .filter(|e| &e.scope == scope)
            .map(|e| e.id.clone())
            .collect();
        for id in ids_to_remove {
            self.remove(&id);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_entity(name: &str, scope: ScopeKey, aliases: Vec<&str>) -> Entity {
        Entity {
            id: EntityId::new(format!("{}@{}", name.to_lowercase(), scope)),
            canonical_name: name.to_string(),
            kind: EntityKind::Person,
            scope,
            aliases: aliases.into_iter().map(String::from).collect(),
            metadata: None,
        }
    }

    #[test]
    fn exact_name_match_in_scope() {
        let scope = ScopeKey::namespace_only("test");
        let mut reg = EntityRegistry::new(16, 100);
        reg.register(make_entity("Alice", scope.clone(), vec![]))
            .unwrap();

        let result = reg.resolve("alice", &scope);
        assert_eq!(result.quality, MatchQuality::ExactCanonical);
        assert!(result.entity.is_some());
    }

    #[test]
    fn alias_match_in_scope() {
        let scope = ScopeKey::namespace_only("test");
        let mut reg = EntityRegistry::new(16, 100);
        reg.register(make_entity("Alice", scope.clone(), vec!["ally"]))
            .unwrap();

        let result = reg.resolve("ally", &scope);
        assert_eq!(result.quality, MatchQuality::ExactAlias);
    }

    #[test]
    fn unresolved_mention() {
        let scope = ScopeKey::namespace_only("test");
        let reg = EntityRegistry::new(16, 100);
        let result = reg.resolve("unknown", &scope);
        assert_eq!(result.quality, MatchQuality::Unresolved);
        assert!(result.entity.is_none());
    }

    #[test]
    fn same_mention_different_scopes_resolves_differently() {
        let scope_a = ScopeKey {
            namespace: "ns".into(),
            domain: None,
            workspace_id: None,
            repo_id: Some("repo-a".into()),
        };
        let scope_b = ScopeKey {
            namespace: "ns".into(),
            domain: None,
            workspace_id: None,
            repo_id: Some("repo-b".into()),
        };

        let mut reg = EntityRegistry::new(16, 100);
        reg.register(Entity {
            id: EntityId::new("alice-a"),
            canonical_name: "Alice".into(),
            kind: EntityKind::Person,
            scope: scope_a.clone(),
            aliases: vec![],
            metadata: None,
        })
        .unwrap();
        reg.register(Entity {
            id: EntityId::new("alice-b"),
            canonical_name: "Alice".into(),
            kind: EntityKind::Person,
            scope: scope_b.clone(),
            aliases: vec![],
            metadata: None,
        })
        .unwrap();

        let result_a = reg.resolve("Alice", &scope_a);
        let result_b = reg.resolve("Alice", &scope_b);

        assert_eq!(result_a.quality, MatchQuality::ExactCanonical);
        assert_eq!(result_b.quality, MatchQuality::ExactCanonical);
        assert_eq!(result_a.entity.unwrap().id.as_str(), "alice-a");
        assert_eq!(result_b.entity.unwrap().id.as_str(), "alice-b");
    }

    #[test]
    fn alias_only_works_in_correct_scope() {
        let scope_a = ScopeKey {
            namespace: "ns".into(),
            domain: None,
            workspace_id: None,
            repo_id: Some("repo-a".into()),
        };
        let scope_b = ScopeKey {
            namespace: "ns".into(),
            domain: None,
            workspace_id: None,
            repo_id: Some("repo-b".into()),
        };

        let mut reg = EntityRegistry::new(16, 100);
        reg.register(Entity {
            id: EntityId::new("alice-a"),
            canonical_name: "Alice".into(),
            kind: EntityKind::Person,
            scope: scope_a.clone(),
            aliases: vec!["ally".into()],
            metadata: None,
        })
        .unwrap();

        // "ally" resolves in scope_a
        let result = reg.resolve("ally", &scope_a);
        assert_eq!(result.quality, MatchQuality::ExactAlias);

        // "ally" does NOT resolve in scope_b
        let result = reg.resolve("ally", &scope_b);
        assert_eq!(result.quality, MatchQuality::Unresolved);
        // But it appears as an alternative
        assert_eq!(result.alternatives.len(), 1);
        assert_eq!(result.alternatives[0].id.as_str(), "alice-a");
    }

    #[test]
    fn scoped_fallback_to_namespace_only() {
        let ns_scope = ScopeKey::namespace_only("ns");
        let narrow_scope = ScopeKey {
            namespace: "ns".into(),
            domain: Some("code".into()),
            workspace_id: None,
            repo_id: None,
        };

        let mut reg = EntityRegistry::new(16, 100);
        reg.register(Entity {
            id: EntityId::new("alice-ns"),
            canonical_name: "Alice".into(),
            kind: EntityKind::Person,
            scope: ns_scope.clone(),
            aliases: vec![],
            metadata: None,
        })
        .unwrap();

        // Querying narrow scope falls back to namespace-only
        let result = reg.resolve("Alice", &narrow_scope);
        assert_eq!(result.quality, MatchQuality::ScopedFallback);
        assert!(result.entity.is_some());
    }

    #[test]
    fn registry_full_rejects() {
        let scope = ScopeKey::namespace_only("test");
        let mut reg = EntityRegistry::new(16, 1);
        reg.register(make_entity("Alice", scope.clone(), vec![]))
            .unwrap();
        let err = reg.register(make_entity("Bob", scope, vec![])).unwrap_err();
        assert_eq!(err.kind(), "registry_full");
    }

    #[test]
    fn update_replaces_old_indexes() {
        let scope = ScopeKey::namespace_only("test");
        let mut reg = EntityRegistry::new(16, 100);
        reg.register(make_entity("Alice", scope.clone(), vec!["ally"]))
            .unwrap();
        // Re-register with different alias
        let updated = Entity {
            id: EntityId::new(format!("alice@{}", scope)),
            canonical_name: "Alice".into(),
            kind: EntityKind::Person,
            scope: scope.clone(),
            aliases: vec!["al".into()],
            metadata: None,
        };
        reg.register(updated).unwrap();

        // Old alias should no longer resolve
        assert_eq!(
            reg.resolve("ally", &scope).quality,
            MatchQuality::Unresolved
        );
        // New alias should resolve
        assert_eq!(reg.resolve("al", &scope).quality, MatchQuality::ExactAlias);
    }

    #[test]
    fn clear_scope_removes_only_targeted_scope() {
        let scope_a = ScopeKey::namespace_only("a");
        let scope_b = ScopeKey::namespace_only("b");
        let mut reg = EntityRegistry::new(16, 100);
        reg.register(make_entity("Alice", scope_a.clone(), vec![]))
            .unwrap();
        reg.register(make_entity("Bob", scope_b.clone(), vec![]))
            .unwrap();

        reg.clear_scope(&scope_a);
        assert_eq!(reg.len(), 1);
        assert_eq!(
            reg.resolve("Alice", &scope_a).quality,
            MatchQuality::Unresolved
        );
        assert_eq!(
            reg.resolve("Bob", &scope_b).quality,
            MatchQuality::ExactCanonical
        );
    }

    #[test]
    fn clear_all_removes_everything() {
        let scope_a = ScopeKey::namespace_only("a");
        let scope_b = ScopeKey::namespace_only("b");
        let mut reg = EntityRegistry::new(16, 100);
        reg.register(make_entity("Alice", scope_a.clone(), vec!["ally"]))
            .unwrap();
        reg.register(make_entity("Bob", scope_b.clone(), vec!["bobby"]))
            .unwrap();

        assert_eq!(reg.len(), 2);
        reg.clear_all();
        assert_eq!(reg.len(), 0);
        assert!(reg.is_empty());

        // All indexes must be cleared — no resolution should succeed
        assert_eq!(
            reg.resolve("Alice", &scope_a).quality,
            MatchQuality::Unresolved
        );
        assert_eq!(
            reg.resolve("ally", &scope_a).quality,
            MatchQuality::Unresolved
        );
        assert_eq!(
            reg.resolve("Bob", &scope_b).quality,
            MatchQuality::Unresolved
        );
        assert_eq!(
            reg.resolve("bobby", &scope_b).quality,
            MatchQuality::Unresolved
        );
    }

    #[test]
    fn clear_all_allows_re_registration() {
        let scope = ScopeKey::namespace_only("test");
        let mut reg = EntityRegistry::new(16, 2);
        reg.register(make_entity("Alice", scope.clone(), vec![]))
            .unwrap();
        reg.register(make_entity("Bob", scope.clone(), vec![]))
            .unwrap();

        // Registry is now full
        assert!(reg
            .register(make_entity("Charlie", scope.clone(), vec![]))
            .is_err());

        // Clear and re-register
        reg.clear_all();
        reg.register(make_entity("Charlie", scope.clone(), vec![]))
            .unwrap();
        assert_eq!(reg.len(), 1);
        assert_eq!(
            reg.resolve("Charlie", &scope).quality,
            MatchQuality::ExactCanonical
        );
    }
}
