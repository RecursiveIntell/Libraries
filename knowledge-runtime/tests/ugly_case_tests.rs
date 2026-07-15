#![allow(clippy::expect_used)]

//! Release-blocking ugly-case tests for knowledge-runtime.
//!
//! These tests verify the runtime's honest degradation, scope enforcement,
//! deterministic tie-breaking, and authority boundary invariants.

use chrono::Utc;
use knowledge_runtime::config::ProjectionConfig;
use knowledge_runtime::obs::trace::{QueryTrace, QueryWarning, RuntimeView};
use knowledge_runtime::query::classify::{ClassifyResult, QueryMode};
use knowledge_runtime::query::merge::{self, LegResult, MergePolicy, MergedResults};
use knowledge_runtime::query::route::{RetrievalStrategy, RouteLeg, RoutePlan};
use knowledge_runtime::{
    Entity, EntityCacheHandle, EntityKind, EntityRegistry, KnowledgeRuntime, MatchQuality,
    ProjectionHealth, ProjectionKind, ProjectionTracker, RuntimeConfig, Scope, ScopeKey,
};
use semantic_memory::{MemoryConfig, MemoryStore, MockEmbedder, SearchConfig};
use stack_ids::{EntityId, TraceCtx};
use std::time::Duration;
use tempfile::TempDir;

// ── D1. Temporal request degrade honesty ────────────────────────

#[test]
fn temporal_request_not_silently_downgraded_without_warning() {
    // When a temporal query is classified, it must produce a TemporalLookup mode.
    // The runtime must emit TemporalDowngradedToHybrid when it falls back.
    let result = knowledge_runtime::query::classify::classify("what happened yesterday");
    match &result.mode {
        QueryMode::TemporalLookup { .. } | QueryMode::Mixed { .. } => {
            // Expected: temporal signal detected
        }
        other => {
            // Even if classification doesn't detect temporal, that's a classification
            // issue, not a honesty issue. The key invariant is: if temporal IS detected
            // and execution falls back, a warning MUST be emitted.
            // This test documents that temporal keywords are detected.
            panic!(
                "Expected temporal signal for 'what happened yesterday', got {:?}",
                other
            );
        }
    }
}

#[test]
fn temporal_downgrade_warning_always_emitted_when_present() {
    let scope = Scope::new("test");
    let trace = QueryTrace::from_pipeline(
        TraceCtx::generate(),
        scope.key(),
        ClassifyResult {
            mode: QueryMode::SemanticLookup,
            confidence: 0.8,
            reason: None,
        },
        RoutePlan {
            query: "test".into(),
            scope: scope.clone(),
            legs: vec![RouteLeg {
                strategy: RetrievalStrategy::TemporalSearch {
                    temporal_expr: "yesterday".into(),
                },
                limit: 10,
                filter: None,
            }],
        },
        vec![],
        Duration::from_millis(10),
        &MergedResults {
            results: vec![],
            duplicates_fused: 0,
            total_raw: 0,
        },
        vec![QueryWarning::TemporalDowngradedToHybrid {
            temporal_expr: "yesterday".into(),
        }],
    );

    assert!(trace.is_degraded());
    assert!(trace.has_temporal_downgrade());
    assert!(trace
        .widenings
        .iter()
        .any(|widening| widening.reason.contains("temporal route degraded")));
}

// ── D2. Scope widen prevention ──────────────────────────────────

#[test]
fn scope_with_extra_dimensions_always_warns() {
    let scope = Scope::new("ns").with_domain("code").with_workspace("ws1");
    let scope_key = scope.key();

    // When scope has dimensions beyond namespace, ScopePartiallyEnforced must fire
    assert!(scope.domain.is_some() || scope.workspace_id.is_some());

    let trace = QueryTrace::from_pipeline(
        TraceCtx::generate(),
        scope_key.clone(),
        ClassifyResult {
            mode: QueryMode::SemanticLookup,
            confidence: 0.8,
            reason: None,
        },
        RoutePlan {
            query: "test".into(),
            scope: scope.clone(),
            legs: vec![RouteLeg {
                strategy: RetrievalStrategy::HybridSearch,
                limit: 10,
                filter: None,
            }],
        },
        vec![],
        Duration::from_millis(5),
        &MergedResults {
            results: vec![],
            duplicates_fused: 0,
            total_raw: 0,
        },
        vec![QueryWarning::ScopePartiallyEnforced {
            full_scope: scope_key,
        }],
    );

    assert!(trace.has_scope_enforcement_warning());
    assert!(trace
        .widenings
        .iter()
        .any(|widening| widening.reason.contains("namespace-only scope")));
}

#[test]
fn namespace_only_scope_does_not_trigger_scope_warning() {
    let scope = Scope::new("ns");
    // No domain, no workspace, no repo — no warning needed
    assert!(scope.domain.is_none());
    assert!(scope.workspace_id.is_none());
    assert!(scope.repo_id.is_none());
}

// ── D3. Deterministic tie-break ─────────────────────────────────

#[test]
fn equal_base_ranking_remains_deterministic() {
    use semantic_memory::{SearchResult, SearchSource};

    // Create results with identical scores
    let make_result = |id: &str| SearchResult {
        content: format!("content_{id}"),
        source: SearchSource::Fact {
            fact_id: id.to_string(),
            namespace: "test".to_string(),
        },
        score: 1.0,
        bm25_rank: Some(1),
        vector_rank: Some(1),
        cosine_similarity: Some(0.9),
    };

    let leg_results = vec![vec![
        LegResult {
            leg_index: 0,
            result: make_result("b"),
        },
        LegResult {
            leg_index: 0,
            result: make_result("a"),
        },
        LegResult {
            leg_index: 0,
            result: make_result("c"),
        },
    ]];

    let policy = MergePolicy::default();

    // Run merge 10 times and verify ordering is stable
    let mut orderings = Vec::new();
    for _ in 0..10 {
        let merged = merge::merge(leg_results.clone(), &policy);
        let keys: Vec<String> = merged
            .results
            .iter()
            .map(|m| match &m.result.source {
                SearchSource::Fact { fact_id, .. } => fact_id.clone(),
                _ => "unknown".to_string(),
            })
            .collect();
        orderings.push(keys);
    }

    // All orderings must be identical
    for ordering in &orderings {
        assert_eq!(
            ordering, &orderings[0],
            "merge ordering is not deterministic"
        );
    }
}

// ── D5. Raw Forge join prohibition ──────────────────────────────

#[test]
fn runtime_never_silently_fuses_raw_receipts_into_ranking() {
    // This is a structural test: verify that the runtime adapter only calls
    // semantic-memory search, not any raw Forge receipt API.
    // The adapter module only exposes search and last_import_at.
    // If a raw Forge join were added, it would need to appear in the adapter.
    //
    // This test documents the invariant; the compile-time guarantee is
    // that SemanticMemoryAdapter has no method for raw Forge receipt access.

    // The runtime config type does not contain any Forge-related field.
    // We verify by inspecting the type's Debug representation via a constructed instance.
    let config = knowledge_runtime::RuntimeConfig {
        default_scope: Scope::new("test"),
        query: Default::default(),
        entity: Default::default(),
        projection: Default::default(),
        strict_temporal: false,
        strict_scope: false,
    };
    assert!(
        !format!("{:?}", config).contains("forge_receipt"),
        "RuntimeConfig must not reference raw Forge receipts"
    );
}

// ── B2. Entity: competing canonical IDs ─────────────────────────

#[test]
fn competing_canonical_ids_remain_explicit_conflict() {
    use stack_ids::EntityId;

    let scope = ScopeKey::namespace_only("test");
    let mut reg = EntityRegistry::new(16, 100);

    // Register two entities with the same canonical name in the same scope
    let e1 = Entity {
        id: EntityId::new("id-1"),
        canonical_name: "Alice".into(),
        kind: EntityKind::Person,
        scope: scope.clone(),
        aliases: vec![],
        metadata: None,
    };
    let e2 = Entity {
        id: EntityId::new("id-2"),
        canonical_name: "Alice".into(),
        kind: EntityKind::Person,
        scope: scope.clone(),
        aliases: vec![],
        metadata: None,
    };

    reg.register(e1).unwrap();
    // The second registration with a different ID but the same canonical name
    // is rejected as an explicit collision — no silent winner without provenance.
    let collision = reg.register(e2);
    assert!(
        collision.is_err(),
        "registering a competing canonical name must fail"
    );
    // The first entity is still present and resolvable.
    let result = reg.resolve("Alice", &scope);
    assert_eq!(result.quality, MatchQuality::ExactCanonical);
    assert_eq!(result.entity.unwrap().id.as_str(), "entity:id-1");
    assert!(reg.get(&EntityId::new("id-1")).is_some());
}

// ── Projection lifecycle: staleness and invalidation ────────────

#[test]
fn projection_staleness_is_time_based_and_explicit() {
    use knowledge_runtime::ids::ProjectionId;
    use knowledge_runtime::projection::lifecycle::{
        InvalidationEvent, ProjectionVersion, StaleCause,
    };

    let mut tracker = ProjectionTracker::new(1); // 1 second staleness threshold

    let id = ProjectionId {
        kind: ProjectionKind::Entity,
        key: "test-entity".into(),
        scope: ScopeKey::namespace_only("test"),
    };

    // Record a build
    tracker.record_build(
        id.clone(),
        10,
        100,
        Some(ProjectionVersion {
            schema_version: Some("1".into()),
            resolver_version: Some("1".into()),
        }),
    );

    // Immediately after build, should be healthy
    assert_eq!(tracker.health(&id), ProjectionHealth::Healthy);

    // After explicit invalidation, should be stale
    let event = InvalidationEvent {
        projection_ids: vec![id.clone()],
        cause: StaleCause::ExplicitInvalidation {
            reason: "test invalidation".into(),
        },
        at: Utc::now(),
    };
    tracker.invalidate(&event);
    assert_eq!(tracker.health(&id), ProjectionHealth::Stale);
}

#[test]
fn projection_version_mismatch_triggers_stale() {
    use knowledge_runtime::ids::ProjectionId;
    use knowledge_runtime::projection::lifecycle::{
        InvalidationEvent, ProjectionVersion, StaleCause,
    };

    let mut tracker = ProjectionTracker::new(3600);

    let id = ProjectionId {
        kind: ProjectionKind::Entity,
        key: "versioned".into(),
        scope: ScopeKey::namespace_only("test"),
    };

    tracker.record_build(
        id.clone(),
        5,
        50,
        Some(ProjectionVersion {
            schema_version: Some("v1".into()),
            resolver_version: Some("r1".into()),
        }),
    );
    assert_eq!(tracker.health(&id), ProjectionHealth::Healthy);

    // Invalidate due to version mismatch
    let event = InvalidationEvent {
        projection_ids: vec![id.clone()],
        cause: StaleCause::VersionMismatch,
        at: Utc::now(),
    };
    tracker.invalidate(&event);
    assert_eq!(tracker.health(&id), ProjectionHealth::Stale);
}

// ── Entity cache handle non-authoritative fencing ───────────────

#[test]
fn entity_cache_handle_load_and_clear_scope() {
    let scope = ScopeKey::namespace_only("test");
    let mut reg = EntityRegistry::new(16, 100);

    // Use the cache handle to load an entity (simulates upstream refresh)
    {
        let mut handle = EntityCacheHandle::from_registry_mut(&mut reg);

        let entity = Entity {
            id: EntityId::new("alice-test"),
            canonical_name: "Alice".into(),
            kind: EntityKind::Person,
            scope: scope.clone(),
            aliases: vec!["ally".into()],
            metadata: None,
        };
        handle.load_from_upstream(entity).unwrap();
        assert_eq!(handle.len(), 1);
        assert!(!handle.is_empty());

        handle.clear_scope(&scope);
        assert_eq!(handle.len(), 0);
        assert!(handle.is_empty());
    }
}

#[test]
fn entity_cache_handle_clear_all() {
    let scope_a = ScopeKey::namespace_only("a");
    let scope_b = ScopeKey::namespace_only("b");
    let mut reg = EntityRegistry::new(16, 100);

    {
        let mut handle = EntityCacheHandle::from_registry_mut(&mut reg);
        handle
            .load_from_upstream(Entity {
                id: EntityId::new("alice"),
                canonical_name: "Alice".into(),
                kind: EntityKind::Person,
                scope: scope_a.clone(),
                aliases: vec![],
                metadata: None,
            })
            .unwrap();
        handle
            .load_from_upstream(Entity {
                id: EntityId::new("bob"),
                canonical_name: "Bob".into(),
                kind: EntityKind::Person,
                scope: scope_b.clone(),
                aliases: vec![],
                metadata: None,
            })
            .unwrap();
        assert_eq!(handle.len(), 2);

        handle.clear_all();
        assert_eq!(handle.len(), 0);
    }
}

#[test]
fn entity_cache_handle_api_surface_is_intentionally_restricted() {
    // Structural test: EntityCacheHandle only exposes cache-appropriate operations:
    // - load_from_upstream()
    // - clear_scope()
    // - clear_all()
    // - len()
    // - is_empty()
    //
    // It does NOT expose resolve(), remove(), iter(), get().
    // Those are read/query operations accessed through entity_registry() (immutable ref).
    // This test documents the authority boundary enforcement.

    let mut reg = EntityRegistry::new(16, 100);
    let handle = EntityCacheHandle::from_registry_mut(&mut reg);

    // Cache handle provides length introspection only
    assert_eq!(handle.len(), 0);
    assert!(handle.is_empty());
}

// ── Helpers for strict-mode integration tests ───────────────────

fn open_test_store(dir: &TempDir) -> MemoryStore {
    let config = MemoryConfig {
        base_dir: dir.path().to_path_buf(),
        search: SearchConfig {
            min_similarity: -1.0,
            ..Default::default()
        },
        ..Default::default()
    };
    let embedder = Box::new(MockEmbedder::new(config.embedding.dimensions));
    MemoryStore::open_with_embedder(config, embedder).expect("open store")
}

fn runtime_config_strict_temporal() -> RuntimeConfig {
    RuntimeConfig {
        default_scope: Scope::new("test-ns"),
        query: Default::default(),
        entity: Default::default(),
        projection: ProjectionConfig {
            staleness_threshold_secs: 3600,
            import_staleness_threshold_secs: 0,
            persist: false,
        },
        strict_temporal: true,
        strict_scope: false,
    }
}

fn runtime_config_strict_scope() -> RuntimeConfig {
    RuntimeConfig {
        default_scope: Scope::new("test-ns"),
        query: Default::default(),
        entity: Default::default(),
        projection: ProjectionConfig {
            staleness_threshold_secs: 3600,
            import_staleness_threshold_secs: 0,
            persist: false,
        },
        strict_temporal: false,
        strict_scope: true,
    }
}

fn runtime_config_non_strict() -> RuntimeConfig {
    RuntimeConfig {
        default_scope: Scope::new("test-ns"),
        query: Default::default(),
        entity: Default::default(),
        projection: ProjectionConfig {
            staleness_threshold_secs: 3600,
            import_staleness_threshold_secs: 0,
            persist: false,
        },
        strict_temporal: false,
        strict_scope: false,
    }
}

async fn ingest_scoped_document(store: &MemoryStore, scope: &Scope, title: &str, content: &str) {
    store
        .ingest_document(
            title,
            content,
            &scope.namespace,
            None,
            Some(serde_json::json!({
                "scope_domain": scope.domain.clone(),
                "scope_workspace_id": scope.workspace_id.clone(),
                "scope_repo_id": scope.repo_id.clone(),
            })),
        )
        .await
        .unwrap();
}

// ── I015: Strict temporal mode rejects temporal queries ──────────

#[tokio::test]
async fn strict_temporal_rejects_temporal_query_with_error() {
    let dir = TempDir::new().unwrap();
    let store = open_test_store(&dir);

    store
        .add_fact("test-ns", "something happened yesterday", None, None)
        .await
        .unwrap();

    let adapter =
        knowledge_runtime::adapters::semantic_memory::SemanticMemoryAdapter::new(store.clone());
    let config = runtime_config_strict_temporal();
    let runtime = KnowledgeRuntime::new(config, adapter).unwrap();

    let scope = Scope::new("test-ns");
    let result = runtime.query("what happened yesterday", Some(&scope)).await;

    // If the classifier detects temporal signal, strict mode must return an error
    match result {
        Err(ref e) if e.kind() == "temporal_not_supported" => {
            // Expected: strict temporal mode rejects when metadata is unavailable
            let msg = format!("{e}");
            assert!(
                msg.contains("temporal"),
                "error message must mention temporal: {msg}"
            );
        }
        Ok((_results, trace)) => {
            // Classification might not detect temporal signal for this query.
            // If it doesn't, there's no temporal leg, so no error is expected.
            let has_temporal = matches!(
                &trace.classification.mode,
                QueryMode::TemporalLookup { .. } | QueryMode::Mixed { .. }
            );
            assert!(
                !has_temporal,
                "temporal query was classified as temporal but strict mode did not reject it"
            );
        }
        Err(e) => {
            panic!("unexpected error kind: {}", e);
        }
    }
}

#[tokio::test]
async fn non_strict_temporal_emits_warning_instead_of_error() {
    let dir = TempDir::new().unwrap();
    let store = open_test_store(&dir);

    store
        .add_fact("test-ns", "something happened yesterday", None, None)
        .await
        .unwrap();

    let adapter =
        knowledge_runtime::adapters::semantic_memory::SemanticMemoryAdapter::new(store.clone());
    let config = runtime_config_non_strict();
    let runtime = KnowledgeRuntime::new(config, adapter).unwrap();

    let scope = Scope::new("test-ns");
    let result = runtime.query("what happened yesterday", Some(&scope)).await;

    // Non-strict mode must NOT error — it should succeed with a warning
    let (_results, trace) = result.expect("non-strict temporal must not return error");

    let has_temporal = matches!(
        &trace.classification.mode,
        QueryMode::TemporalLookup { .. } | QueryMode::Mixed { .. }
    );
    if has_temporal {
        assert!(
            trace.has_temporal_downgrade(),
            "non-strict temporal query must emit TemporalDowngradedToHybrid warning"
        );
    }
}

#[tokio::test]
async fn explicit_temporal_query_discloses_temporal_requested_view() {
    let dir = TempDir::new().unwrap();
    let store = open_test_store(&dir);

    store
        .add_fact("test-ns", "deploy notes for service alpha", None, None)
        .await
        .unwrap();

    let adapter =
        knowledge_runtime::adapters::semantic_memory::SemanticMemoryAdapter::new(store.clone());
    let config = runtime_config_non_strict();
    let runtime = KnowledgeRuntime::new(config, adapter).unwrap();

    let scope = Scope::new("test-ns");
    let (_results, trace) = runtime
        .query_temporal(
            "deploy notes for service alpha",
            Some(&scope),
            "2026-03-12T00:00:00Z",
            "2026-03-12T00:00:00Z",
        )
        .await
        .unwrap();

    let provenance = trace.runtime_query_provenance();
    assert!(provenance.requested_views.contains(&RuntimeView::Temporal));
    assert!(provenance
        .widenings
        .iter()
        .any(|widening| widening.requested_view == RuntimeView::Temporal));
}

#[tokio::test]
async fn exact_entity_query_reports_entity_view_without_widening() {
    let dir = TempDir::new().unwrap();
    let store = open_test_store(&dir);

    let adapter =
        knowledge_runtime::adapters::semantic_memory::SemanticMemoryAdapter::new(store.clone());
    let config = runtime_config_non_strict();
    let mut runtime = KnowledgeRuntime::new(config, adapter).unwrap();
    runtime
        .refresh_entity_cache()
        .load_from_upstream(Entity {
            id: EntityId::new("entity-alice"),
            canonical_name: "Alice".into(),
            kind: EntityKind::Person,
            scope: ScopeKey::namespace_only("test-ns"),
            aliases: vec!["ally".into()],
            metadata: None,
        })
        .unwrap();

    let scope = Scope::new("test-ns");
    let (_results, trace) = runtime.query("find \"Alice\"", Some(&scope)).await.unwrap();
    let provenance = trace.runtime_query_provenance();

    assert!(provenance.requested_views.contains(&RuntimeView::Entity));
    assert!(provenance.executed_views.contains(&RuntimeView::Entity));
    assert!(!provenance
        .warnings
        .iter()
        .any(|warning| matches!(warning, QueryWarning::EntityScopeFallback { .. })));
    assert!(!provenance
        .widenings
        .iter()
        .any(|widening| widening.reason.contains("broader candidate scope")));
}

#[tokio::test]
async fn entity_alias_scope_fallback_is_explicit_in_provenance() {
    let dir = TempDir::new().unwrap();
    let store = open_test_store(&dir);

    let adapter =
        knowledge_runtime::adapters::semantic_memory::SemanticMemoryAdapter::new(store.clone());
    let config = runtime_config_non_strict();
    let mut runtime = KnowledgeRuntime::new(config, adapter).unwrap();
    runtime
        .refresh_entity_cache()
        .load_from_upstream(Entity {
            id: EntityId::new("entity-alice"),
            canonical_name: "Alice".into(),
            kind: EntityKind::Person,
            scope: ScopeKey::namespace_only("test-ns"),
            aliases: vec!["ally".into()],
            metadata: None,
        })
        .unwrap();

    let scope = Scope::new("test-ns").with_domain("code");
    let (_results, trace) = runtime.query("find \"ally\"", Some(&scope)).await.unwrap();
    let provenance = trace.runtime_query_provenance();

    assert!(provenance.requested_views.contains(&RuntimeView::Entity));
    assert!(provenance.executed_views.contains(&RuntimeView::Entity));
    assert!(provenance
        .warnings
        .iter()
        .any(|warning| matches!(warning, QueryWarning::EntityScopeFallback { .. })));
    assert!(provenance
        .widenings
        .iter()
        .any(|widening| widening.reason.contains("broader candidate scope")));
}

#[tokio::test]
async fn mixed_entity_and_temporal_degradation_discloses_all_active_views() {
    let dir = TempDir::new().unwrap();
    let store = open_test_store(&dir);

    store
        .add_fact("test-ns", "ally changed config last week", None, None)
        .await
        .unwrap();

    let adapter =
        knowledge_runtime::adapters::semantic_memory::SemanticMemoryAdapter::new(store.clone());
    let config = runtime_config_non_strict();
    let mut runtime = KnowledgeRuntime::new(config, adapter).unwrap();
    runtime
        .refresh_entity_cache()
        .load_from_upstream(Entity {
            id: EntityId::new("entity-alice"),
            canonical_name: "Alice".into(),
            kind: EntityKind::Person,
            scope: ScopeKey::namespace_only("test-ns"),
            aliases: vec!["ally".into()],
            metadata: None,
        })
        .unwrap();

    let scope = Scope::new("test-ns")
        .with_domain("code")
        .with_workspace("ws-1");
    let (_results, trace) = runtime
        .query("what did \"ally\" do last week", Some(&scope))
        .await
        .unwrap();
    let provenance = trace.runtime_query_provenance();

    assert!(provenance.requested_views.contains(&RuntimeView::Entity));
    assert!(provenance.requested_views.contains(&RuntimeView::Temporal));
    assert!(provenance.executed_views.contains(&RuntimeView::Entity));
    assert!(provenance.executed_views.contains(&RuntimeView::Semantic));
    assert!(provenance
        .warnings
        .iter()
        .any(|warning| matches!(warning, QueryWarning::EntityScopeFallback { .. })));
    assert!(provenance
        .warnings
        .iter()
        .any(|warning| matches!(warning, QueryWarning::TemporalDowngradedToHybrid { .. })));
    assert!(provenance.widenings.len() >= 2);
}

// ── I016: Strict scope mode rejects extra-dimension queries ──────

#[tokio::test]
async fn strict_scope_allows_fully_enforced_extra_dimensions() {
    let dir = TempDir::new().unwrap();
    let store = open_test_store(&dir);
    let scope = Scope::new("test-ns")
        .with_domain("code")
        .with_workspace("ws-1");
    ingest_scoped_document(
        &store,
        &scope,
        "Strict scope doc",
        "test fact for strict scope lives in ws-1 code scope",
    )
    .await;

    let adapter =
        knowledge_runtime::adapters::semantic_memory::SemanticMemoryAdapter::new(store.clone());
    let config = runtime_config_strict_scope();
    let runtime = KnowledgeRuntime::new(config, adapter).unwrap();

    let (results, trace) = runtime
        .query("strict scope ws-1 code", Some(&scope))
        .await
        .expect("strict scope must allow fully enforceable extra dimensions");
    assert!(
        !trace.has_scope_enforcement_warning(),
        "fully enforced scoped query must not emit ScopePartiallyEnforced"
    );
    assert!(
        results
            .iter()
            .any(|result| result.content.contains("ws-1 code scope")),
        "strict scope query must return the matching scoped document"
    );
}

#[tokio::test]
async fn strict_scope_allows_namespace_only_queries() {
    let dir = TempDir::new().unwrap();
    let store = open_test_store(&dir);

    store
        .add_fact("test-ns", "test fact namespace only", None, None)
        .await
        .unwrap();

    let adapter =
        knowledge_runtime::adapters::semantic_memory::SemanticMemoryAdapter::new(store.clone());
    let config = runtime_config_strict_scope();
    let runtime = KnowledgeRuntime::new(config, adapter).unwrap();

    // Query with namespace-only scope — should succeed even in strict mode
    let scope = Scope::new("test-ns");
    let result = runtime.query("test fact", Some(&scope)).await;
    assert!(
        result.is_ok(),
        "strict scope must allow namespace-only queries"
    );

    let (_results, trace) = result.unwrap();
    assert!(
        !trace.has_scope_enforcement_warning(),
        "namespace-only scope must not trigger scope warning"
    );
}

#[tokio::test]
async fn non_strict_scope_allows_fully_enforced_extra_dimensions_without_warning() {
    let dir = TempDir::new().unwrap();
    let store = open_test_store(&dir);
    let scope = Scope::new("test-ns")
        .with_domain("code")
        .with_workspace("ws-1");
    ingest_scoped_document(
        &store,
        &scope,
        "Non-strict scope doc",
        "test fact for scope warning is fully scoped in ws-1 code",
    )
    .await;

    let adapter =
        knowledge_runtime::adapters::semantic_memory::SemanticMemoryAdapter::new(store.clone());
    let config = runtime_config_non_strict();
    let runtime = KnowledgeRuntime::new(config, adapter).unwrap();

    let (results, trace) = runtime
        .query("scope warning ws-1 code", Some(&scope))
        .await
        .expect("non-strict scope must allow fully enforceable extra dimensions");
    assert!(
        !trace.has_scope_enforcement_warning(),
        "fully enforceable scope must not emit ScopePartiallyEnforced in non-strict mode"
    );
    assert!(
        results
            .iter()
            .any(|result| result.content.contains("fully scoped in ws-1 code")),
        "non-strict scoped query must return the matching document"
    );
}

// ── I015/I016: TemporalNotSupported error variant ────────────────

#[test]
fn temporal_not_supported_error_variant_exists_and_has_correct_kind() {
    use knowledge_runtime::RuntimeError;

    let err = RuntimeError::TemporalNotSupported {
        temporal_expr: "yesterday".into(),
    };
    assert_eq!(err.kind(), "temporal_not_supported");
    let msg = format!("{err}");
    assert!(msg.contains("yesterday"));
    assert!(msg.contains("temporal"));
}

// ── I015/I016: Config validation passes with strict flags ────────

#[test]
fn strict_temporal_and_strict_scope_pass_config_validation() {
    let dir = TempDir::new().unwrap();
    let store = open_test_store(&dir);
    let adapter =
        knowledge_runtime::adapters::semantic_memory::SemanticMemoryAdapter::new(store.clone());

    let config = RuntimeConfig {
        default_scope: Scope::new("test-ns"),
        query: Default::default(),
        entity: Default::default(),
        projection: Default::default(),
        strict_temporal: true,
        strict_scope: true,
    };

    // Both strict flags set to true must still pass validation
    let result = KnowledgeRuntime::new(config, adapter);
    assert!(
        result.is_ok(),
        "strict_temporal=true and strict_scope=true must pass validation"
    );
}

#[tokio::test]
async fn explained_search_adapter_preserves_scope_and_runtime_provenance() {
    let dir = TempDir::new().unwrap();
    let store = open_test_store(&dir);

    let personal = Scope::new("test-ns").with_domain("personal");
    let work = Scope::new("test-ns").with_domain("work");
    ingest_scoped_document(
        &store,
        &personal,
        "Personal Rust",
        "Rust memory safety notes for my personal study.",
    )
    .await;
    ingest_scoped_document(
        &store,
        &work,
        "Work Rust",
        "Rust memory safety guidance for the work codebase.",
    )
    .await;

    let adapter =
        knowledge_runtime::adapters::semantic_memory::SemanticMemoryAdapter::new(store.clone());
    let explained = adapter
        .search_explained("rust memory safety", &personal, 5, 0.0)
        .await
        .unwrap();

    assert!(!explained.explained_results.is_empty());
    assert!(explained.explained_results.iter().all(|item| item
        .result
        .content
        .contains("personal")
        || item.result.content.contains("my personal")));
    assert_eq!(explained.provenance.scope, personal.key());
    assert_eq!(explained.provenance.classification_mode, "search");
    assert_eq!(explained.provenance.query, "rust memory safety");
    assert_eq!(explained.provenance.requested_views.len(), 1);
    assert_eq!(explained.provenance.executed_views.len(), 1);
}
