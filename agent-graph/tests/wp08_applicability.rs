use agent_graph::applicability::{
    ApplicabilityEngine, ApplicabilityState, Artifact, Basis, Dependency, DependencyKind,
    DependencyPrecision, PublicationMode, Reason, Scope,
};

fn scope(paths: &[&str]) -> Scope {
    Scope::new(paths.iter().copied())
}

fn artifact(id: &str, paths: &[&str], version: &str) -> Artifact {
    Artifact::new(
        id,
        scope(paths),
        Basis::new(version, format!("digest-{version}")),
    )
}

#[test]
fn dep_01_authorization_source_change_invalidates_only_authorization_descendants() {
    let artifacts = vec![
        artifact("authorization", &["repo"], "auth-v1"),
        artifact("authorized-analysis", &["repo/private"], "analysis-v1"),
        artifact("authorized-closure", &["repo/private"], "closure-v1"),
        artifact("parser", &["repo/parser.rs"], "parser-v1"),
    ];
    let dependencies = vec![
        Dependency::exact(
            "authorization",
            "authorized-analysis",
            DependencyKind::Authorization,
            Basis::new("auth-v1", "digest-auth-v1"),
        ),
        Dependency::exact(
            "authorized-analysis",
            "authorized-closure",
            DependencyKind::Analysis,
            Basis::new("analysis-v1", "digest-analysis-v1"),
        ),
    ];
    let mut engine = ApplicabilityEngine::new(artifacts, dependencies).unwrap();

    let affected = engine
        .update_basis(
            "authorization",
            Basis::new("auth-v2", "digest-auth-v2"),
            Reason::AuthorizationChanged,
        )
        .unwrap();

    assert_eq!(
        affected,
        vec!["authorization", "authorized-analysis", "authorized-closure"]
    );
    assert_eq!(
        engine
            .evaluate("authorized-closure", &scope(&["repo/private"]))
            .state,
        ApplicabilityState::Revalidate
    );
    assert_eq!(
        engine.evaluate("parser", &scope(&["repo/parser.rs"])).state,
        ApplicabilityState::Reusable
    );
}

#[test]
fn dep_02_absence_finding_depends_on_query_universe_inventory() {
    let artifacts = vec![
        artifact("inventory", &["repo/**/*.rs"], "inventory-v1"),
        artifact("absence", &["repo/**/*.rs"], "absence-v1")
            .with_universal_claim(true)
            .with_full_enumeration_coverage(true),
    ];
    let dependencies = vec![Dependency::exact(
        "inventory",
        "absence",
        DependencyKind::QueryUniverse,
        Basis::new("inventory-v1", "digest-inventory-v1"),
    )];
    let mut engine = ApplicabilityEngine::new(artifacts, dependencies).unwrap();

    engine
        .update_basis(
            "inventory",
            Basis::new("inventory-v2", "digest-new-alternate-file"),
            Reason::QueryUniverseChanged,
        )
        .unwrap();

    let decision = engine.evaluate("absence", &scope(&["repo/**/*.rs"]));
    assert_eq!(decision.state, ApplicabilityState::Revalidate);
    assert!(decision.reasons.contains(&Reason::QueryUniverseChanged));
}

#[test]
fn dep_03_source_deletion_rejects_current_reuse_but_retains_history() {
    let source = artifact("source", &["repo/input.md"], "source-v1")
        .with_historical_identity("sha256:source-v1");
    let mut engine = ApplicabilityEngine::new(vec![source], vec![]).unwrap();

    engine.set_present("source", false).unwrap();

    let decision = engine.evaluate("source", &scope(&["repo/input.md"]));
    assert_eq!(decision.state, ApplicabilityState::HistoricalOnly);
    assert_eq!(
        decision.historical_identity.as_deref(),
        Some("sha256:source-v1")
    );
    assert!(!decision.payload_reveal_allowed);
    assert!(decision.reasons.contains(&Reason::SourceMissing));
}

#[test]
fn dep_04_broader_requested_scope_cannot_reuse_narrower_proof() {
    let proof = artifact("proof", &["repo/src/a.rs"], "proof-v1");
    let engine = ApplicabilityEngine::new(vec![proof], vec![]).unwrap();

    assert_eq!(
        engine.evaluate("proof", &scope(&["repo/src/a.rs"])).state,
        ApplicabilityState::Reusable
    );
    let broader = engine.evaluate("proof", &scope(&["repo/src/a.rs", "repo/src/b.rs"]));
    assert_eq!(broader.state, ApplicabilityState::Revalidate);
    assert!(broader.reasons.contains(&Reason::ScopeNotCovered));
}

#[test]
fn dep_05_parent_requirement_revision_reopens_all_mapped_descendants() {
    let artifacts = vec![
        artifact("requirement", &["REQ-1"], "req-v1"),
        artifact("analysis", &["REQ-1"], "analysis-v1"),
        artifact("check", &["REQ-1"], "check-v1"),
        artifact("closure", &["REQ-1"], "closure-v1"),
    ];
    let dependencies = vec![
        Dependency::exact(
            "requirement",
            "analysis",
            DependencyKind::Requirement,
            Basis::new("req-v1", "digest-req-v1"),
        ),
        Dependency::exact(
            "analysis",
            "check",
            DependencyKind::Analysis,
            Basis::new("analysis-v1", "digest-analysis-v1"),
        ),
        Dependency::exact(
            "check",
            "closure",
            DependencyKind::Closure,
            Basis::new("check-v1", "digest-check-v1"),
        ),
    ];
    let mut engine = ApplicabilityEngine::new(artifacts, dependencies).unwrap();

    let affected = engine
        .update_basis(
            "requirement",
            Basis::new("req-v2", "digest-req-v2"),
            Reason::RequirementRevised,
        )
        .unwrap();

    assert_eq!(
        affected,
        vec!["requirement", "analysis", "check", "closure"]
    );
    for id in ["analysis", "check", "closure"] {
        assert_eq!(
            engine.evaluate(id, &scope(&["REQ-1"])).state,
            ApplicabilityState::Revalidate
        );
    }
}

#[test]
fn dep_06_retrieval_stack_version_change_creates_new_basis() {
    let artifacts = vec![
        artifact(
            "retrieval-basis",
            &["index"],
            "embed-1/index-1/chunk-1/rank-1",
        ),
        artifact("query-result", &["query:q"], "result-v1"),
    ];
    let old_basis = Basis::new(
        "embed-1/index-1/chunk-1/rank-1",
        "digest-embed-1/index-1/chunk-1/rank-1",
    );
    let dependencies = vec![Dependency::exact(
        "retrieval-basis",
        "query-result",
        DependencyKind::RetrievalBasis,
        old_basis.clone(),
    )];
    let mut engine = ApplicabilityEngine::new(artifacts, dependencies).unwrap();
    let new_basis = Basis::new(
        "embed-2/index-1/chunk-1/rank-1",
        "digest-embed-2/index-1/chunk-1/rank-1",
    );

    engine
        .update_basis("retrieval-basis", new_basis.clone(), Reason::BasisChanged)
        .unwrap();

    assert_ne!(old_basis, new_basis);
    let decision = engine.evaluate("query-result", &scope(&["query:q"]));
    assert_eq!(decision.state, ApplicabilityState::Revalidate);
    assert!(decision.reasons.contains(&Reason::BasisChanged));
}

#[test]
fn dep_07_top_k_miss_cannot_establish_universal_absence() {
    let absence = artifact("top-k-absence", &["corpus"], "query-v1")
        .with_universal_claim(true)
        .with_full_enumeration_coverage(false);
    let engine = ApplicabilityEngine::new(vec![absence], vec![]).unwrap();

    let decision = engine.evaluate("top-k-absence", &scope(&["corpus"]));
    assert_eq!(decision.state, ApplicabilityState::Blocked);
    assert!(decision.reasons.contains(&Reason::InsufficientCoverage));
}

#[test]
fn dep_09_undeclared_read_footprint_conservatively_invalidates_on_scope_change() {
    let artifacts = vec![
        artifact("repo-snapshot", &["repo"], "snapshot-v1"),
        artifact("analysis", &["repo/src/a.rs"], "analysis-v1").with_dependency_complete(false),
    ];
    let dependencies = vec![Dependency::enclosing(
        "repo-snapshot",
        "analysis",
        DependencyKind::ReadFootprint,
        Basis::new("snapshot-v1", "digest-snapshot-v1"),
        scope(&["repo"]),
    )];
    let mut engine = ApplicabilityEngine::new(artifacts, dependencies).unwrap();

    let affected = engine.invalidate_relevant_scope(&scope(&["repo/src/new.rs"]));

    assert_eq!(affected, vec!["analysis"]);
    let decision = engine.evaluate("analysis", &scope(&["repo/src/a.rs"]));
    assert_eq!(decision.state, ApplicabilityState::Revalidate);
    assert!(decision.reasons.contains(&Reason::DependencyIncomplete));
}

#[test]
fn dep_10_mutual_dependency_scc_revalidates_together_and_cannot_self_support() {
    let artifacts = vec![
        artifact("claim-a", &["claims"], "a-v1").with_external_support(false),
        artifact("claim-b", &["claims"], "b-v1").with_external_support(false),
    ];
    let dependencies = vec![
        Dependency::exact(
            "claim-a",
            "claim-b",
            DependencyKind::Support,
            Basis::new("a-v1", "digest-a-v1"),
        ),
        Dependency::exact(
            "claim-b",
            "claim-a",
            DependencyKind::Support,
            Basis::new("b-v1", "digest-b-v1"),
        ),
    ];
    let mut engine = ApplicabilityEngine::new(artifacts, dependencies).unwrap();

    assert_eq!(
        engine.strongly_connected_components(),
        vec![vec!["claim-a", "claim-b"]]
    );
    assert_eq!(
        engine.evaluate("claim-a", &scope(&["claims"])).state,
        ApplicabilityState::Blocked
    );
    assert!(engine
        .evaluate("claim-b", &scope(&["claims"]))
        .reasons
        .contains(&Reason::CircularSelfSupport));

    let affected = engine
        .invalidate("claim-a", Reason::DependencyChanged)
        .unwrap();
    assert_eq!(affected, vec!["claim-a", "claim-b"]);
    for id in ["claim-a", "claim-b"] {
        assert_eq!(
            engine.evaluate(id, &scope(&["claims"])).state,
            ApplicabilityState::Revalidate
        );
    }
}

#[test]
fn dep_11_superseded_conclusion_is_historical_with_separate_replacement() {
    let old = artifact("conclusion-v1", &["REQ-1"], "conclusion-v1")
        .with_historical_identity("sha256:conclusion-v1")
        .with_superseded_by("conclusion-v2");
    let new = artifact("conclusion-v2", &["REQ-1"], "conclusion-v2");
    let engine = ApplicabilityEngine::new(vec![old, new], vec![]).unwrap();

    let old_decision = engine.evaluate("conclusion-v1", &scope(&["REQ-1"]));
    assert_eq!(old_decision.state, ApplicabilityState::HistoricalOnly);
    assert_eq!(old_decision.replacement.as_deref(), Some("conclusion-v2"));
    assert!(old_decision.reasons.contains(&Reason::Superseded));
    assert_eq!(
        engine.evaluate("conclusion-v2", &scope(&["REQ-1"])).state,
        ApplicabilityState::Reusable
    );
}

#[test]
fn dep_13_revoked_authorization_blocks_payload_despite_matching_hashes() {
    let artifacts = vec![
        artifact("authorization", &["repo/private"], "auth-v1"),
        artifact("payload", &["repo/private"], "payload-v1"),
    ];
    let dependencies = vec![Dependency::exact(
        "authorization",
        "payload",
        DependencyKind::Authorization,
        Basis::new("auth-v1", "digest-auth-v1"),
    )];
    let mut engine = ApplicabilityEngine::new(artifacts, dependencies).unwrap();

    engine.set_authorized("authorization", false).unwrap();

    let decision = engine.evaluate("payload", &scope(&["repo/private"]));
    assert_eq!(decision.state, ApplicabilityState::Blocked);
    assert!(!decision.payload_reveal_allowed);
    assert!(decision.reasons.contains(&Reason::AuthorizationRevoked));
}

#[test]
fn dep_14_unsupported_precision_uses_enclosing_snapshot_and_recomputation_reason() {
    let dependency = Dependency::enclosing(
        "snapshot",
        "analysis",
        DependencyKind::ReadFootprint,
        Basis::new("snapshot-v1", "digest-snapshot-v1"),
        scope(&["repo/module"]),
    );
    assert_eq!(dependency.precision, DependencyPrecision::EnclosingSnapshot);
    let artifacts = vec![
        artifact("snapshot", &["repo/module"], "snapshot-v1"),
        artifact("analysis", &["repo/module/a.rs"], "analysis-v1"),
    ];
    let mut engine = ApplicabilityEngine::new(artifacts, vec![dependency]).unwrap();

    engine.invalidate_relevant_scope(&scope(&["repo/module/b.rs"]));

    let decision = engine.evaluate("analysis", &scope(&["repo/module/a.rs"]));
    assert_eq!(decision.state, ApplicabilityState::Revalidate);
    assert!(decision.reasons.contains(&Reason::EnclosingSnapshotChanged));
    assert!(decision
        .reasons
        .contains(&Reason::ExtraRecomputationRequired));
}

#[test]
fn join_10_publication_barrier_detects_post_join_dependency_change() {
    let artifacts = vec![
        artifact("left", &["join"], "left-v1"),
        artifact("right", &["join"], "right-v1"),
    ];
    let mut engine = ApplicabilityEngine::new(artifacts, vec![]).unwrap();
    let joined = engine.capture_join(["right", "left"]).unwrap();

    engine
        .update_basis(
            "right",
            Basis::new("right-v2", "digest-right-v2"),
            Reason::DependencyChanged,
        )
        .unwrap();

    let current = engine.publication_decision(&joined, PublicationMode::CurrentOnly);
    assert_eq!(current.state, ApplicabilityState::Blocked);
    assert!(current
        .reasons
        .contains(&Reason::DependencyChangedAfterJoin));

    let historical = engine.publication_decision(&joined, PublicationMode::AllowHistorical);
    assert_eq!(historical.state, ApplicabilityState::HistoricalOnly);
    assert!(historical
        .reasons
        .contains(&Reason::DependencyChangedAfterJoin));
}
