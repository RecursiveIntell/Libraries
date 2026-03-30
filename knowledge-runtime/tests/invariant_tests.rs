//! Tests proving runtime invariants introduced by the invariant-enforcement phase.
//!
//! Covers:
//! - Deterministic merge behavior (tie-breaking)
//! - Projection health transitions (including import-aware states)
//! - Warning emission for degraded paths
//! - Trace/provenance preservation across merge

use knowledge_runtime::query::merge::{merge, LegResult, MergePolicy, ScoreNormalization};
use knowledge_runtime::{
    InvalidationEvent, ProjectionHealth, ProjectionId, ProjectionKind, ProjectionTracker,
    ProjectionVersion, QueryWarning, ScopeKey, StaleCause,
};
use semantic_memory::{SearchResult, SearchSource};

// ─── Helpers ───────────────────────────────────────────────────

fn make_fact(id: &str, score: f64) -> SearchResult {
    SearchResult {
        content: format!("fact {id}"),
        source: SearchSource::Fact {
            fact_id: id.to_string(),
            namespace: "test".to_string(),
        },
        score,
        bm25_rank: None,
        vector_rank: None,
        cosine_similarity: None,
    }
}

fn make_chunk(id: &str, doc_id: &str, score: f64) -> SearchResult {
    SearchResult {
        content: format!("chunk {id}"),
        source: SearchSource::Chunk {
            chunk_id: id.to_string(),
            document_id: doc_id.to_string(),
            document_title: "doc".to_string(),
            chunk_index: 0,
        },
        score,
        bm25_rank: None,
        vector_rank: None,
        cosine_similarity: None,
    }
}

fn test_scope() -> ScopeKey {
    ScopeKey::namespace_only("test")
}

fn test_pid(key: &str) -> ProjectionId {
    ProjectionId::new(ProjectionKind::Entity, key, test_scope())
}

// ─── Merge Determinism Tests ───────────────────────────────────

#[test]
fn merge_tie_breaking_is_deterministic_across_runs() {
    // Two facts with identical scores should always sort in the same order
    let policy = MergePolicy {
        limit: 10,
        normalization: ScoreNormalization::None,
        multi_leg_boost: 0.0,
    };

    let make_legs = || {
        vec![vec![
            LegResult {
                leg_index: 0,
                result: make_fact("aaa", 0.5),
            },
            LegResult {
                leg_index: 0,
                result: make_fact("bbb", 0.5),
            },
            LegResult {
                leg_index: 0,
                result: make_fact("ccc", 0.5),
            },
        ]]
    };

    let r1 = merge(make_legs(), &policy);
    let r2 = merge(make_legs(), &policy);
    let r3 = merge(make_legs(), &policy);

    // All three runs should produce identical ordering
    let order1: Vec<_> = r1
        .results
        .iter()
        .map(|r| r.result.content.as_str())
        .collect();
    let order2: Vec<_> = r2
        .results
        .iter()
        .map(|r| r.result.content.as_str())
        .collect();
    let order3: Vec<_> = r3
        .results
        .iter()
        .map(|r| r.result.content.as_str())
        .collect();

    assert_eq!(order1, order2, "merge ordering must be deterministic");
    assert_eq!(order2, order3, "merge ordering must be deterministic");
}

#[test]
fn merge_tie_breaking_favors_multi_leg_support() {
    // When scores are equal after normalization, the item with more source legs wins
    let policy = MergePolicy {
        limit: 10,
        normalization: ScoreNormalization::None,
        multi_leg_boost: 0.0, // disable boost to test pure tie-breaking
    };

    let legs = vec![
        vec![
            LegResult {
                leg_index: 0,
                result: make_fact("single", 0.5),
            },
            LegResult {
                leg_index: 0,
                result: make_fact("multi", 0.5),
            },
        ],
        vec![LegResult {
            leg_index: 1,
            result: make_fact("multi", 0.5),
        }],
    ];

    let merged = merge(legs, &policy);
    // "multi" appeared in 2 legs and should rank first in tie-breaking
    assert_eq!(merged.results[0].result.content, "fact multi");
    assert_eq!(merged.results[0].source_legs.len(), 2);
}

#[test]
fn merge_tie_breaking_uses_stable_key_after_leg_count() {
    // When leg count is also tied, use lexicographic key ordering
    let policy = MergePolicy {
        limit: 10,
        normalization: ScoreNormalization::None,
        multi_leg_boost: 0.0,
    };

    let legs = vec![vec![
        LegResult {
            leg_index: 0,
            result: make_fact("zzz", 0.5),
        },
        LegResult {
            leg_index: 0,
            result: make_fact("aaa", 0.5),
        },
        LegResult {
            leg_index: 0,
            result: make_fact("mmm", 0.5),
        },
    ]];

    let merged = merge(legs, &policy);
    let keys: Vec<_> = merged
        .results
        .iter()
        .map(|r| r.result.content.as_str())
        .collect();
    // Should be sorted lexicographically by identity key: fact:aaa < fact:mmm < fact:zzz
    assert_eq!(keys, vec!["fact aaa", "fact mmm", "fact zzz"]);
}

#[test]
fn merge_provenance_survives_fusion() {
    // Verify that fused results retain provenance from all contributing legs
    let policy = MergePolicy {
        limit: 10,
        normalization: ScoreNormalization::None,
        multi_leg_boost: 0.1,
    };

    let legs = vec![
        vec![
            LegResult {
                leg_index: 0,
                result: make_fact("shared", 0.8),
            },
            LegResult {
                leg_index: 0,
                result: make_fact("only-0", 0.7),
            },
        ],
        vec![
            LegResult {
                leg_index: 1,
                result: make_fact("shared", 0.6),
            },
            LegResult {
                leg_index: 1,
                result: make_fact("only-1", 0.5),
            },
        ],
        vec![LegResult {
            leg_index: 2,
            result: make_fact("shared", 0.4),
        }],
    ];

    let merged = merge(legs, &policy);

    // Find the fused "shared" result
    let shared = merged
        .results
        .iter()
        .find(|r| r.result.content == "fact shared")
        .expect("shared result should exist");

    // Should have provenance from all 3 legs
    assert!(shared.source_legs.contains(&0));
    assert!(shared.source_legs.contains(&1));
    assert!(shared.source_legs.contains(&2));
    assert_eq!(shared.per_leg_scores.len(), 3);

    // Should use the best score (0.8)
    assert_eq!(shared.result.score, 0.8);
}

#[test]
fn merge_reports_accurate_fusion_count() {
    let policy = MergePolicy::default();

    let legs = vec![
        vec![
            LegResult {
                leg_index: 0,
                result: make_fact("dup", 0.9),
            },
            LegResult {
                leg_index: 0,
                result: make_fact("unique-a", 0.8),
            },
        ],
        vec![
            LegResult {
                leg_index: 1,
                result: make_fact("dup", 0.7),
            },
            LegResult {
                leg_index: 1,
                result: make_fact("unique-b", 0.6),
            },
        ],
    ];

    let merged = merge(legs, &policy);
    assert_eq!(merged.total_raw, 4);
    assert_eq!(merged.duplicates_fused, 1);
    assert_eq!(merged.results.len(), 3);
}

// ─── Projection Health Transition Tests ────────────────────────

#[test]
fn import_lagging_state_is_distinct() {
    let mut tracker = ProjectionTracker::new(3600);
    let id = test_pid("import-lag");

    tracker.record_build(id.clone(), 10, 50, None);
    assert_eq!(tracker.health(&id), ProjectionHealth::Healthy);

    // Simulate import lag via invalidation
    let event = InvalidationEvent {
        projection_ids: vec![id.clone()],
        cause: StaleCause::ImportLag {
            last_import_at: Some("2024-01-01T00:00:00Z".to_string()),
        },
        at: chrono::Utc::now(),
    };
    tracker.invalidate(&event);
    assert_eq!(tracker.health(&id), ProjectionHealth::Stale);

    // Verify the cause is preserved
    let meta = tracker.get(&id).unwrap();
    assert!(matches!(
        meta.stale_cause,
        Some(StaleCause::ImportLag { .. })
    ));
}

#[test]
fn import_failure_cause_is_tracked() {
    let mut tracker = ProjectionTracker::new(3600);
    let id = test_pid("import-fail");

    tracker.record_build(id.clone(), 10, 50, None);

    let event = InvalidationEvent {
        projection_ids: vec![id.clone()],
        cause: StaleCause::ImportFailure {
            error: "network timeout".into(),
        },
        at: chrono::Utc::now(),
    };
    tracker.invalidate(&event);

    let meta = tracker.get(&id).unwrap();
    assert_eq!(meta.health, ProjectionHealth::Stale);
    match &meta.stale_cause {
        Some(StaleCause::ImportFailure { error }) => {
            assert_eq!(error, "network timeout");
        }
        other => panic!("expected ImportFailure, got {:?}", other),
    }
}

#[test]
fn version_mismatch_triggers_stale() {
    let mut tracker = ProjectionTracker::new(3600);
    let id = test_pid("ver-test");

    let v1 = ProjectionVersion {
        schema_version: Some("1.0".into()),
        resolver_version: Some("1.0".into()),
    };
    tracker.record_build(id.clone(), 10, 50, Some(v1));
    assert_eq!(tracker.health(&id), ProjectionHealth::Healthy);

    let event = InvalidationEvent {
        projection_ids: vec![id.clone()],
        cause: StaleCause::VersionMismatch,
        at: chrono::Utc::now(),
    };
    tracker.invalidate(&event);
    assert_eq!(tracker.health(&id), ProjectionHealth::Stale);
}

// ─── Warning Type Tests ────────────────────────────────────────

#[test]
fn projection_import_stale_warning_carries_metadata() {
    let scope = ScopeKey::namespace_only("test");
    let warning = QueryWarning::ProjectionImportStale {
        scope: scope.clone(),
        last_import_at: Some("2024-01-01T00:00:00Z".into()),
    };

    // Verify it serializes cleanly
    let json = serde_json::to_string(&warning).unwrap();
    assert!(json.contains("ProjectionImportStale"));
    assert!(json.contains("2024-01-01"));

    // Verify it deserializes cleanly
    let deserialized: QueryWarning = serde_json::from_str(&json).unwrap();
    assert_eq!(warning, deserialized);
}

#[test]
fn all_warning_variants_serialize() {
    let warnings = vec![
        QueryWarning::TemporalDowngradedToHybrid {
            temporal_expr: "last week".into(),
        },
        QueryWarning::ScopePartiallyEnforced {
            full_scope: ScopeKey::namespace_only("ns"),
        },
        QueryWarning::EntityScopeFallback {
            mention: "@user".into(),
            queried_scope: ScopeKey::namespace_only("ns"),
        },
        QueryWarning::ProjectionImportStale {
            scope: ScopeKey::namespace_only("ns"),
            last_import_at: None,
        },
    ];

    for w in &warnings {
        let json = serde_json::to_string(w).unwrap();
        let round_tripped: QueryWarning = serde_json::from_str(&json).unwrap();
        assert_eq!(w, &round_tripped);
    }
}

// ─── StaleCause Tests ──────────────────────────────────────────

#[test]
fn all_stale_causes_serialize() {
    let causes = vec![
        StaleCause::TimeThreshold,
        StaleCause::ExplicitInvalidation {
            reason: "manual".into(),
        },
        StaleCause::SourceChanged,
        StaleCause::VersionMismatch,
        StaleCause::ImportLag {
            last_import_at: Some("2024-06-15T12:00:00Z".into()),
        },
        StaleCause::ImportLag {
            last_import_at: None,
        },
        StaleCause::ImportFailure {
            error: "connection refused".into(),
        },
    ];

    for cause in &causes {
        let json = serde_json::to_string(cause).unwrap();
        let round_tripped: StaleCause = serde_json::from_str(&json).unwrap();
        assert_eq!(cause, &round_tripped);
    }
}

// ─── QueryTrace Helper Tests ───────────────────────────────────

#[test]
fn query_trace_helpers_detect_degradation() {
    use knowledge_runtime::obs::trace::QueryTrace;
    use knowledge_runtime::query::classify::{ClassifyResult, QueryMode};
    use knowledge_runtime::query::merge::MergedResults;
    use knowledge_runtime::query::route::RoutePlan;
    use knowledge_runtime::Scope;
    use stack_ids::TraceCtx;
    use std::time::Duration;

    let scope = Scope::new("test");
    let trace_no_warnings = QueryTrace::from_pipeline(
        TraceCtx::from_trace_id("t1"),
        scope.key(),
        ClassifyResult {
            mode: QueryMode::SemanticLookup,
            confidence: 0.8,
            reason: None,
        },
        RoutePlan {
            query: "test".into(),
            scope: scope.clone(),
            legs: vec![],
        },
        vec![],
        Duration::from_millis(10),
        &MergedResults {
            results: vec![],
            duplicates_fused: 0,
            total_raw: 0,
        },
        vec![],
    );

    assert!(!trace_no_warnings.is_degraded());
    assert!(!trace_no_warnings.has_temporal_downgrade());
    assert!(!trace_no_warnings.has_scope_enforcement_warning());
    assert!(!trace_no_warnings.has_import_staleness_warning());

    let trace_with_warnings = QueryTrace::from_pipeline(
        TraceCtx::from_trace_id("t2"),
        scope.key(),
        ClassifyResult {
            mode: QueryMode::SemanticLookup,
            confidence: 0.8,
            reason: None,
        },
        RoutePlan {
            query: "test".into(),
            scope: scope.clone(),
            legs: vec![],
        },
        vec![],
        Duration::from_millis(10),
        &MergedResults {
            results: vec![],
            duplicates_fused: 0,
            total_raw: 0,
        },
        vec![
            QueryWarning::TemporalDowngradedToHybrid {
                temporal_expr: "last week".into(),
            },
            QueryWarning::ScopePartiallyEnforced {
                full_scope: ScopeKey::namespace_only("ns"),
            },
            QueryWarning::ProjectionImportStale {
                scope: ScopeKey::namespace_only("ns"),
                last_import_at: None,
            },
        ],
    );

    assert!(trace_with_warnings.is_degraded());
    assert!(trace_with_warnings.has_temporal_downgrade());
    assert!(trace_with_warnings.has_scope_enforcement_warning());
    assert!(trace_with_warnings.has_import_staleness_warning());
}

// ─── Cross-type Merge Determinism Tests ────────────────────────

#[test]
fn merge_cross_type_determinism() {
    // Mixing facts, chunks, and episodes should still produce deterministic ordering
    let policy = MergePolicy {
        limit: 10,
        normalization: ScoreNormalization::None,
        multi_leg_boost: 0.0,
    };

    let make_legs = || {
        vec![vec![
            LegResult {
                leg_index: 0,
                result: make_fact("f1", 0.5),
            },
            LegResult {
                leg_index: 0,
                result: make_chunk("c1", "d1", 0.5),
            },
        ]]
    };

    let r1 = merge(make_legs(), &policy);
    let r2 = merge(make_legs(), &policy);

    let order1: Vec<_> = r1
        .results
        .iter()
        .map(|r| r.result.content.as_str())
        .collect();
    let order2: Vec<_> = r2
        .results
        .iter()
        .map(|r| r.result.content.as_str())
        .collect();
    assert_eq!(order1, order2, "cross-type merge must be deterministic");
}

// ─── Scope Enforcement Boundary Tests ──────────────────────────

#[test]
fn scope_key_with_extra_dimensions_is_distinct() {
    let ns_only = ScopeKey::namespace_only("prod");
    let with_domain = ScopeKey {
        namespace: "prod".into(),
        domain: Some("code".into()),
        workspace_id: None,
        repo_id: None,
    };
    let with_repo = ScopeKey {
        namespace: "prod".into(),
        domain: Some("code".into()),
        workspace_id: None,
        repo_id: Some("my-repo".into()),
    };

    assert_ne!(ns_only, with_domain);
    assert_ne!(with_domain, with_repo);
    assert_ne!(ns_only, with_repo);
}

// ─── Projection Tracker Import-Aware Transitions ──────────────

#[test]
fn record_import_failure_then_recover() {
    let mut tracker = ProjectionTracker::new(3600);
    let id = test_pid("import-recovery");

    // Start healthy
    tracker.record_build(id.clone(), 10, 50, None);
    assert_eq!(tracker.health(&id), ProjectionHealth::Healthy);

    // Fail
    let event = InvalidationEvent {
        projection_ids: vec![id.clone()],
        cause: StaleCause::ImportFailure {
            error: "timeout".into(),
        },
        at: chrono::Utc::now(),
    };
    tracker.invalidate(&event);
    assert_eq!(tracker.health(&id), ProjectionHealth::Stale);
    assert!(matches!(
        tracker.get(&id).unwrap().stale_cause,
        Some(StaleCause::ImportFailure { .. })
    ));

    // Recover with successful rebuild
    tracker.record_build(id.clone(), 15, 30, None);
    assert_eq!(tracker.health(&id), ProjectionHealth::Healthy);
    assert!(tracker.get(&id).unwrap().stale_cause.is_none());
}

// ─── ProjectionHealth Tests ────────────────────────────────────

#[test]
fn all_health_variants_serialize() {
    let variants = vec![
        ProjectionHealth::Healthy,
        ProjectionHealth::Stale,
        ProjectionHealth::Missing,
        ProjectionHealth::Rebuilding,
        ProjectionHealth::ImportLagging,
        ProjectionHealth::ImportFailed,
    ];

    for v in &variants {
        let json = serde_json::to_string(v).unwrap();
        let round_tripped: ProjectionHealth = serde_json::from_str(&json).unwrap();
        assert_eq!(v, &round_tripped);
    }
}
