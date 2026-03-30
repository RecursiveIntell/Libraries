use crate::ids::{ProjectionId, ProjectionKind, ScopeKey};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Health status of a projection.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectionHealth {
    /// Projection is current and healthy.
    Healthy,
    /// Projection exists but may be stale.
    Stale,
    /// Projection is missing or failed to build.
    Missing,
    /// Reserved compatibility state for external rebuild orchestration.
    ///
    /// The in-memory tracker does not currently emit this variant on its own.
    Rebuilding,
    /// Reserved compatibility state for import-aware callers.
    ///
    /// The runtime currently surfaces import freshness as `ProjectionImportStale`
    /// warnings in query traces rather than transitioning tracker health here.
    ImportLagging,
    /// Reserved compatibility state for import-aware callers.
    ///
    /// The tracker does not currently transition into this state.
    ImportFailed,
}

/// Why a projection became stale.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StaleCause {
    /// Staleness threshold exceeded (time-based).
    TimeThreshold,
    /// Explicitly invalidated via API.
    ExplicitInvalidation { reason: String },
    /// Source data changed upstream.
    SourceChanged,
    /// Schema or resolver version changed.
    VersionMismatch,
    /// Upstream import pipeline is lagging behind source.
    ImportLag {
        /// ISO 8601 timestamp of the last successful import, if any.
        last_import_at: Option<String>,
    },
    /// The last import attempt failed.
    ImportFailure {
        /// Error from the failed import.
        error: String,
    },
}

/// Version metadata for a projection.
///
/// Tracks schema and resolver versions so that version changes can trigger
/// invalidation. Fields are opaque strings — the runtime does not interpret
/// them, just compares for equality.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectionVersion {
    /// Schema version of the projection data format.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema_version: Option<String>,
    /// Version of the resolver/adapter that built this projection.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolver_version: Option<String>,
}

/// Metadata about a projection's lifecycle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectionMeta {
    /// Projection identity.
    pub id: ProjectionId,
    /// Current health.
    pub health: ProjectionHealth,
    /// Why the projection is stale (if applicable).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stale_cause: Option<StaleCause>,
    /// When the projection was last built.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub built_at: Option<DateTime<Utc>>,
    /// When the projection was last invalidated.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub invalidated_at: Option<DateTime<Utc>>,
    /// How long the last build took, in milliseconds.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub build_duration_ms: Option<u64>,
    /// Number of source items that fed the projection.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_count: Option<usize>,
    /// Last error if the projection is unhealthy.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_error: Option<String>,
    /// Version metadata.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<ProjectionVersion>,
}

/// An invalidation signal that marks projections as stale.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvalidationEvent {
    /// Which projections are affected.
    pub projection_ids: Vec<ProjectionId>,
    /// Why they are being invalidated.
    pub cause: StaleCause,
    /// When the invalidation was issued.
    pub at: DateTime<Utc>,
}

/// Result of a projection lifecycle action.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectionActionResult {
    /// How many projections were affected.
    pub affected_count: usize,
    /// IDs of affected projections.
    pub affected_ids: Vec<ProjectionId>,
}

/// Projection lifecycle tracker.
///
/// Tracks the health and staleness metadata of all projections.
/// Projections are identified by `ProjectionId` which includes scope.
///
/// ## Authority: Observability only, NOT orchestration
///
/// This tracker is a **passive bookkeeping / observability component**.
/// It records state transitions (build, invalidation, failure) so that
/// callers can inspect projection health and decide whether to trigger
/// rebuilds. Current methods emit `Healthy`, `Stale`, and `Missing`;
/// other `ProjectionHealth` variants remain reserved for compatibility
/// and external orchestration surfaces. The tracker does NOT:
///
/// - Schedule or execute projection rebuilds
/// - Retry failed builds automatically
/// - Communicate with upstream import pipelines
/// - Own any source of truth
///
/// Callers are responsible for executing rebuilds and reporting
/// outcomes via `record_build()` / `record_failure()`.
#[derive(Debug)]
pub struct ProjectionTracker {
    /// Projection metadata by ID.
    projections: BTreeMap<ProjectionId, ProjectionMeta>,
    /// Staleness threshold in seconds.
    staleness_threshold_secs: u64,
}

impl ProjectionTracker {
    pub fn new(staleness_threshold_secs: u64) -> Self {
        Self {
            projections: BTreeMap::new(),
            staleness_threshold_secs,
        }
    }

    /// Record that a projection was successfully built.
    ///
    /// **This is a bookkeeping method, not a rebuild trigger.** The tracker
    /// records that a build occurred but does not execute, schedule, or
    /// orchestrate any rebuild work. External callers must drive the actual
    /// rebuild and call this method to report the outcome.
    pub fn record_build(
        &mut self,
        id: ProjectionId,
        source_count: usize,
        build_duration_ms: u64,
        version: Option<ProjectionVersion>,
    ) {
        self.projections.insert(
            id.clone(),
            ProjectionMeta {
                id,
                health: ProjectionHealth::Healthy,
                stale_cause: None,
                built_at: Some(Utc::now()),
                invalidated_at: None,
                build_duration_ms: Some(build_duration_ms),
                source_count: Some(source_count),
                last_error: None,
                version,
            },
        );
    }

    /// Record that a projection build failed.
    ///
    /// **This is a bookkeeping method, not a retry trigger.** The tracker
    /// records the failure but does not retry, schedule, or orchestrate any
    /// rebuild attempt. External callers must decide whether and when to
    /// retry, and call `record_build()` or `record_failure()` to report
    /// the next outcome.
    pub fn record_failure(&mut self, id: ProjectionId, error: String) {
        let existing = self.projections.get(&id);
        self.projections.insert(
            id.clone(),
            ProjectionMeta {
                id,
                health: ProjectionHealth::Missing,
                stale_cause: None,
                built_at: existing.and_then(|p| p.built_at),
                invalidated_at: None,
                build_duration_ms: None,
                source_count: existing.and_then(|p| p.source_count),
                last_error: Some(error),
                version: existing.and_then(|p| p.version.clone()),
            },
        );
    }

    /// Process an invalidation event, marking affected projections as stale.
    pub fn invalidate(&mut self, event: &InvalidationEvent) -> ProjectionActionResult {
        let mut affected = Vec::new();
        let now = Utc::now();

        for pid in &event.projection_ids {
            if let Some(meta) = self.projections.get_mut(pid) {
                meta.health = ProjectionHealth::Stale;
                meta.stale_cause = Some(event.cause.clone());
                meta.invalidated_at = Some(now);
                affected.push(pid.clone());
            }
        }

        ProjectionActionResult {
            affected_count: affected.len(),
            affected_ids: affected,
        }
    }

    /// Invalidate all projections of a given kind within a scope.
    pub fn invalidate_by_kind_and_scope(
        &mut self,
        kind: &ProjectionKind,
        scope: &ScopeKey,
        cause: StaleCause,
    ) -> ProjectionActionResult {
        let now = Utc::now();
        let mut affected = Vec::new();

        for (pid, meta) in self.projections.iter_mut() {
            if &pid.kind == kind && &pid.scope == scope {
                meta.health = ProjectionHealth::Stale;
                meta.stale_cause = Some(cause.clone());
                meta.invalidated_at = Some(now);
                affected.push(pid.clone());
            }
        }

        ProjectionActionResult {
            affected_count: affected.len(),
            affected_ids: affected,
        }
    }

    /// Invalidate all projections within a scope (all kinds).
    pub fn invalidate_scope(
        &mut self,
        scope: &ScopeKey,
        cause: StaleCause,
    ) -> ProjectionActionResult {
        let now = Utc::now();
        let mut affected = Vec::new();

        for (pid, meta) in self.projections.iter_mut() {
            if &pid.scope == scope {
                meta.health = ProjectionHealth::Stale;
                meta.stale_cause = Some(cause.clone());
                meta.invalidated_at = Some(now);
                affected.push(pid.clone());
            }
        }

        ProjectionActionResult {
            affected_count: affected.len(),
            affected_ids: affected,
        }
    }

    /// Get the health of a specific projection (with time-based staleness check).
    pub fn health(&self, id: &ProjectionId) -> ProjectionHealth {
        self.projections
            .get(id)
            .map(|m| {
                if m.health == ProjectionHealth::Healthy {
                    if let Some(built_at) = m.built_at {
                        let age = Utc::now()
                            .signed_duration_since(built_at)
                            .num_seconds()
                            .unsigned_abs();
                        if age > self.staleness_threshold_secs {
                            return ProjectionHealth::Stale;
                        }
                    }
                }
                m.health.clone()
            })
            .unwrap_or(ProjectionHealth::Missing)
    }

    /// Get metadata for a specific projection.
    pub fn get(&self, id: &ProjectionId) -> Option<&ProjectionMeta> {
        self.projections.get(id)
    }

    /// Iterate over all tracked projection IDs.
    ///
    /// Useful for external rebuild drivers that need to scan for stale projections.
    pub fn all_ids(&self) -> impl Iterator<Item = &ProjectionId> {
        self.projections.keys()
    }

    /// Query status of all projections matching a kind and/or scope filter.
    pub fn query_status(
        &self,
        kind: Option<&ProjectionKind>,
        scope: Option<&ScopeKey>,
    ) -> Vec<&ProjectionMeta> {
        self.projections
            .values()
            .filter(|m| {
                kind.map_or(true, |k| &m.id.kind == k) && scope.map_or(true, |s| &m.id.scope == s)
            })
            .collect()
    }

    /// List all tracked projections.
    pub fn list(&self) -> Vec<&ProjectionMeta> {
        self.projections.values().collect()
    }

    /// Remove a projection from tracking (forces recomputation on next access).
    pub fn remove(&mut self, id: &ProjectionId) -> Option<ProjectionMeta> {
        self.projections.remove(id)
    }

    /// Clear all projections within a scope.
    pub fn clear_scope(&mut self, scope: &ScopeKey) -> ProjectionActionResult {
        let ids_to_remove: Vec<ProjectionId> = self
            .projections
            .keys()
            .filter(|pid| &pid.scope == scope)
            .cloned()
            .collect();

        let affected_count = ids_to_remove.len();
        for id in &ids_to_remove {
            self.projections.remove(id);
        }

        ProjectionActionResult {
            affected_count,
            affected_ids: ids_to_remove,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_scope() -> ScopeKey {
        ScopeKey::namespace_only("test")
    }

    fn test_pid(key: &str) -> ProjectionId {
        ProjectionId::new(ProjectionKind::Entity, key, test_scope())
    }

    #[test]
    fn missing_by_default() {
        let tracker = ProjectionTracker::new(3600);
        assert_eq!(tracker.health(&test_pid("x")), ProjectionHealth::Missing);
    }

    #[test]
    fn healthy_after_build() {
        let mut tracker = ProjectionTracker::new(3600);
        let id = test_pid("test");
        tracker.record_build(id.clone(), 10, 50, None);
        assert_eq!(tracker.health(&id), ProjectionHealth::Healthy);
    }

    #[test]
    fn stale_after_invalidation() {
        let mut tracker = ProjectionTracker::new(3600);
        let id = test_pid("test");
        tracker.record_build(id.clone(), 10, 50, None);

        let event = InvalidationEvent {
            projection_ids: vec![id.clone()],
            cause: StaleCause::SourceChanged,
            at: Utc::now(),
        };
        let result = tracker.invalidate(&event);
        assert_eq!(result.affected_count, 1);
        assert_eq!(tracker.health(&id), ProjectionHealth::Stale);

        // Verify stale cause is recorded
        let meta = tracker.get(&id).unwrap();
        assert_eq!(meta.stale_cause, Some(StaleCause::SourceChanged));
        assert!(meta.invalidated_at.is_some());
    }

    #[test]
    fn invalidate_by_kind_and_scope() {
        let mut tracker = ProjectionTracker::new(3600);
        let scope = test_scope();
        let other_scope = ScopeKey::namespace_only("other");

        let id1 = ProjectionId::new(ProjectionKind::Entity, "a", scope.clone());
        let id2 = ProjectionId::new(ProjectionKind::Entity, "b", scope.clone());
        let id3 = ProjectionId::new(ProjectionKind::Entity, "c", other_scope.clone());
        let id4 = ProjectionId::new(ProjectionKind::Temporal, "d", scope.clone());

        tracker.record_build(id1.clone(), 5, 10, None);
        tracker.record_build(id2.clone(), 5, 10, None);
        tracker.record_build(id3.clone(), 5, 10, None);
        tracker.record_build(id4.clone(), 5, 10, None);

        let result = tracker.invalidate_by_kind_and_scope(
            &ProjectionKind::Entity,
            &scope,
            StaleCause::ExplicitInvalidation {
                reason: "test".into(),
            },
        );

        assert_eq!(result.affected_count, 2);
        assert_eq!(tracker.health(&id1), ProjectionHealth::Stale);
        assert_eq!(tracker.health(&id2), ProjectionHealth::Stale);
        assert_eq!(tracker.health(&id3), ProjectionHealth::Healthy); // different scope
        assert_eq!(tracker.health(&id4), ProjectionHealth::Healthy); // different kind
    }

    #[test]
    fn invalidate_scope_affects_all_kinds() {
        let mut tracker = ProjectionTracker::new(3600);
        let scope = test_scope();

        let id1 = ProjectionId::new(ProjectionKind::Entity, "a", scope.clone());
        let id2 = ProjectionId::new(ProjectionKind::Temporal, "b", scope.clone());

        tracker.record_build(id1.clone(), 5, 10, None);
        tracker.record_build(id2.clone(), 5, 10, None);

        let result = tracker.invalidate_scope(&scope, StaleCause::SourceChanged);
        assert_eq!(result.affected_count, 2);
        assert_eq!(tracker.health(&id1), ProjectionHealth::Stale);
        assert_eq!(tracker.health(&id2), ProjectionHealth::Stale);
    }

    #[test]
    fn query_status_filters() {
        let mut tracker = ProjectionTracker::new(3600);
        let scope = test_scope();

        let id1 = ProjectionId::new(ProjectionKind::Entity, "a", scope.clone());
        let id2 = ProjectionId::new(ProjectionKind::Temporal, "b", scope.clone());
        tracker.record_build(id1.clone(), 5, 10, None);
        tracker.record_build(id2.clone(), 5, 10, None);

        let entities = tracker.query_status(Some(&ProjectionKind::Entity), None);
        assert_eq!(entities.len(), 1);

        let in_scope = tracker.query_status(None, Some(&scope));
        assert_eq!(in_scope.len(), 2);
    }

    #[test]
    fn failure_recorded() {
        let mut tracker = ProjectionTracker::new(3600);
        let id = test_pid("test");
        tracker.record_failure(id.clone(), "timeout".into());
        assert_eq!(tracker.health(&id), ProjectionHealth::Missing);
        assert!(tracker.get(&id).unwrap().last_error.is_some());
    }

    #[test]
    fn remove_forces_missing() {
        let mut tracker = ProjectionTracker::new(3600);
        let id = test_pid("test");
        tracker.record_build(id.clone(), 5, 20, None);
        tracker.remove(&id);
        assert_eq!(tracker.health(&id), ProjectionHealth::Missing);
    }

    #[test]
    fn clear_scope_removes_only_targeted() {
        let mut tracker = ProjectionTracker::new(3600);
        let scope_a = ScopeKey::namespace_only("a");
        let scope_b = ScopeKey::namespace_only("b");

        let id_a = ProjectionId::new(ProjectionKind::Entity, "x", scope_a.clone());
        let id_b = ProjectionId::new(ProjectionKind::Entity, "y", scope_b.clone());

        tracker.record_build(id_a.clone(), 5, 10, None);
        tracker.record_build(id_b.clone(), 5, 10, None);

        let result = tracker.clear_scope(&scope_a);
        assert_eq!(result.affected_count, 1);
        assert_eq!(tracker.health(&id_a), ProjectionHealth::Missing);
        assert_eq!(tracker.health(&id_b), ProjectionHealth::Healthy);
    }

    #[test]
    fn version_metadata_preserved() {
        let mut tracker = ProjectionTracker::new(3600);
        let id = test_pid("test");
        let version = ProjectionVersion {
            schema_version: Some("1.0".into()),
            resolver_version: Some("2.0".into()),
        };
        tracker.record_build(id.clone(), 5, 10, Some(version.clone()));

        let meta = tracker.get(&id).unwrap();
        assert_eq!(
            meta.version.as_ref().unwrap().schema_version,
            Some("1.0".into())
        );
    }
}
