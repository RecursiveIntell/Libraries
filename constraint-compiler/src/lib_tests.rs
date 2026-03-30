use super::*;
use forge_memory_bridge::{
    transform_envelope_v3, ImportEntityAlias, ImportEvidenceRef, ImportProjectionRecordV3,
    MergeDecision, ReviewState,
};
use semantic_memory_forge::{
    ConstraintSeedKind, ExportAuthority, ExportClaim, ExportConfidenceClass, ExportEntityAlias,
    ExportEnvelopeV3, ExportEpisode, ExportEvidenceRef, ExportRecord, ExportRecordSemanticsV3,
    ExportRecordV3, ExportRelation, ForgeExportMeta, NuisanceSnapshot, ProjectionVisibilityClass,
    EXPORT_ENVELOPE_V3_SCHEMA,
};
use stack_ids::{
    AssertionGroupId, ClaimFamilyId, ClaimId, ClaimVersionId, EntityId, EnvelopeId, EpisodeId,
    ScopeKey,
};

fn rich_batch() -> ProjectionImportBatchV3 {
    let scope = ScopeKey::namespace_only("compiler");
    let semantics = ExportRecordSemanticsV3 {
        claim_family_id: Some(ClaimFamilyId::new("family-1")),
        assertion_group_id: Some(AssertionGroupId::new("group-1")),
        relation_group_id: None,
        joint_evidence_group_id: None,
        constraint_seed_kind: Some(ConstraintSeedKind::Hyperedge),
        treatment_hint: None,
        outcome_hint: None,
        confounder_hint: None,
        instrument_hint: None,
        effect_modifier_hint: None,
        contradiction_candidate_group_id: None,
        mutual_exclusion_group_id: None,
        comparability_snapshot_version: Some("cmp-1".into()),
        nuisance_snapshot: Some(NuisanceSnapshot {
            environment_fingerprint: Some("linux".into()),
            toolchain_version: Some("rust-1.85".into()),
            dependency_set_hash: Some("deps-1".into()),
            scope_mismatch_markers: vec![],
            measurement_notes: vec!["measurement drift".into()],
            selection_bias_markers: vec!["selection bias".into()],
        }),
        projection_visibility_class: ProjectionVisibilityClass::Standard,
        export_confidence_class: ExportConfidenceClass::Verified,
        derivation_seed_ids: vec!["seed-1".into()],
        review_priority_hint: None,
    };
    let records = vec![
        ExportRecordV3 {
            record: ExportRecord::Claim(ExportClaim {
                claim_id: None,
                claim_version_id: Some("claim-version-1".into()),
                subject_entity_id: EntityId::new("entity-1"),
                predicate: "supports".into(),
                object_anchor: serde_json::json!("result"),
                valid_from: None,
                valid_to: None,
                confidence: 1.0,
                content: "entity supports result".into(),
                projection_family: "forge_verification".into(),
                supersedes_claim_id: None,
                supersedes_claim_version_id: None,
                metadata: None,
            }),
            semantics: Some(semantics.clone()),
        },
        ExportRecordV3 {
            record: ExportRecord::Claim(ExportClaim {
                claim_id: None,
                claim_version_id: Some("claim-version-2".into()),
                subject_entity_id: EntityId::new("entity-2"),
                predicate: "supports".into(),
                object_anchor: serde_json::json!("result"),
                valid_from: None,
                valid_to: None,
                confidence: 0.95,
                content: "entity 2 supports result".into(),
                projection_family: "forge_verification".into(),
                supersedes_claim_id: None,
                supersedes_claim_version_id: None,
                metadata: None,
            }),
            semantics: Some(semantics),
        },
    ];
    let export_meta = ForgeExportMeta {
        authority: ExportAuthority::Forge,
        run_id: Some("run-1".into()),
        direct_write: false,
        comparability_snapshot_version: Some("cmp-1".into()),
        exported_at: "2026-03-10T00:00:00Z".into(),
    };
    let digest =
        ExportEnvelopeV3::compute_digest("forge", &scope, &records, Some(&export_meta), None)
            .unwrap();
    let envelope = ExportEnvelopeV3 {
        envelope_id: EnvelopeId::new("env-1"),
        schema_version: EXPORT_ENVELOPE_V3_SCHEMA.into(),
        content_digest: digest,
        source_authority: "forge".into(),
        scope_key: scope,
        trace_ctx: None,
        exported_at: "2026-03-10T00:00:00Z".into(),
        export_meta: Some(export_meta),
        evidence_bundle: None,
        support_sets: vec![],
        contradiction_witnesses: vec![],
        retraction_records: vec![],
        claim_states_v13: vec![],
        intervention_bundles_v14: vec![],
        outcome_schemas_v14: vec![],
        cohort_contracts_v14: vec![],
        counterfactual_slices_v14: vec![],
        experiment_cases_v14: vec![],
        comparability_matrices_v14: vec![],
        decision_traces_v14: vec![],
        refuter_suites_v14: vec![],
        refuter_results_v14: vec![],
        experiment_budgets_v14: vec![],
        rollout_decisions_v14: vec![],
        rollback_decisions_v14: vec![],
        attestation_envelopes_v15: vec![],
        trust_root_sets_v15: vec![],
        artifact_admission_policies_v15: vec![],
        transparency_receipts_v15: vec![],
        attestation_revocations_v15: vec![],
        attestation_supersessions_v15: vec![],
        remote_oracle_leases_v15: vec![],
        remote_slice_requests_v15: vec![],
        remote_slice_results_v15: vec![],
        cross_runtime_replay_tickets_v15: vec![],
        dispute_bundles_v15: vec![],
        disclosure_policies_v15: vec![],
        disclosure_budgets_v15: vec![],
        records,
    };
    transform_envelope_v3(&envelope).unwrap()
}

#[test]
fn compilation_is_deterministic() {
    let batch = rich_batch();
    let policy = CompilerPolicy {
        policy_version: "policy-1".into(),
        include_hyperedges: true,
    };

    let a = compile_batch(&batch, &policy);
    let b = compile_batch(&batch, &policy);
    assert_eq!(a.graph_hash, b.graph_hash);
    assert_eq!(a.hyperedges, b.hyperedges);
}

#[test]
fn compilation_is_deterministic_under_record_reordering() {
    let batch = rich_batch();
    let mut shuffled = batch.clone();
    shuffled.records.reverse();
    let policy = CompilerPolicy {
        policy_version: "policy-1".into(),
        include_hyperedges: true,
    };

    let normal = compile_batch(&batch, &policy);
    let shuffled_output = compile_batch(&shuffled, &policy);

    assert_eq!(normal.graph_hash, shuffled_output.graph_hash);
    assert_eq!(normal.nodes, shuffled_output.nodes);
    assert_eq!(normal.constraints, shuffled_output.constraints);
    assert_eq!(normal.degradations, shuffled_output.degradations);
}

#[test]
fn hyperedge_compilation_groups_records_by_assertion_group() {
    let batch = rich_batch();
    let output = compile_batch(
        &batch,
        &CompilerPolicy {
            policy_version: "policy-1".into(),
            include_hyperedges: true,
        },
    );

    let group = output
        .hyperedges
        .iter()
        .find(|edge| edge.edge_id == "assertion_group:group-1")
        .expect("assertion hyperedge expected");
    assert_eq!(
        group.member_node_ids,
        vec!["claim-version-1".to_string(), "claim-version-2".to_string()]
    );
}

#[test]
fn nuisance_state_is_materialized_in_hyperedges_by_default() {
    let batch = rich_batch();
    let output = compile_batch(
        &batch,
        &CompilerPolicy {
            policy_version: "policy-1".into(),
            include_hyperedges: true,
        },
    );

    assert!(output
        .nodes
        .iter()
        .any(|node| node.kind == "nuisance_state"));
    assert!(output
        .hyperedges
        .iter()
        .any(|edge| edge.edge_id.starts_with("nuisance_edge:")));
    assert!(output
        .constraints
        .iter()
        .any(|constraint| constraint.kind == "nuisance_disclosure"));
}

#[test]
fn nuisance_state_is_materialized_explicitly_when_hyperedges_disabled() {
    let batch = rich_batch();
    let output = compile_batch(
        &batch,
        &CompilerPolicy {
            policy_version: "policy-1".into(),
            include_hyperedges: false,
        },
    );

    assert!(output
        .constraints
        .iter()
        .any(|constraint| constraint.kind == "nuisance_disclosure"));
}

#[test]
fn invalidation_cones_cover_peer_nodes_in_hyperedge() {
    let batch = rich_batch();
    let output = compile_batch(
        &batch,
        &CompilerPolicy {
            policy_version: "policy-1".into(),
            include_hyperedges: true,
        },
    );

    let cone = output
        .invalidation_cones
        .iter()
        .find(|cone| cone.source_node_id == "claim-version-1")
        .expect("expected cone for first claim");
    assert!(cone
        .affected_node_ids
        .contains(&"claim-version-2".to_string()));
    assert!(cone
        .affected_hyperedge_ids
        .contains(&"assertion_group:group-1".to_string()));
}

#[test]
fn thin_export_yields_explicit_degradation() {
    let mut batch = rich_batch();
    batch.records[0].semantics = None;
    let output = compile_batch(
        &batch,
        &CompilerPolicy {
            policy_version: "policy-1".into(),
            include_hyperedges: true,
        },
    );

    assert!(output.degradations.contains(&ConstraintDegradation::ThinExport));
    assert!(output.oracle_candidates.is_empty());
}

#[test]
fn thin_export_does_not_invent_assertion_hyperedges_without_group() {
    let mut batch = rich_batch();
    batch.records[1].semantics = None;
    let output = compile_batch(
        &batch,
        &CompilerPolicy {
            policy_version: "policy-1".into(),
            include_hyperedges: true,
        },
    );

    assert!(output.degradations.contains(&ConstraintDegradation::ThinExport));
    let group_hyperedge = output
        .hyperedges
        .iter()
        .find(|edge| edge.edge_id == "assertion_group:group-1")
        .expect("explicitly semanticized records should keep their provided grouping");
    assert_eq!(
        group_hyperedge.member_node_ids,
        vec!["claim-version-1".to_string()]
    );
}

#[test]
fn alias_and_evidence_refs_without_semantics_do_not_force_thin_export() {
    let mut batch = rich_batch();
    batch.records.push(ImportProjectionRecordV3 {
        record: ImportProjectionRecord::EntityAlias(ImportEntityAlias {
            canonical_entity_id: EntityId::new("entity-alias"),
            alias_text: "alias-name".into(),
            alias_source: "manual".into(),
            match_evidence: None,
            confidence: 0.8,
            merge_decision: MergeDecision::PendingReview,
            scope: ScopeKey::namespace_only("compiler"),
            review_state: ReviewState::Unreviewed,
            is_human_confirmed: false,
            is_human_confirmed_final: false,
            superseded_by_entity_id: None,
            split_from_entity_id: None,
            source_envelope_id: EnvelopeId::new("env-alias-1"),
        }),
        semantics: None,
    });
    batch.records.push(ImportProjectionRecordV3 {
        record: ImportProjectionRecord::EvidenceRef(ImportEvidenceRef {
            claim_id: ClaimId::new("claim-ref-1"),
            claim_version_id: Some(ClaimVersionId::new("claim-version-ref-1")),
            fetch_handle: "fetch://evidence/ref-1".into(),
            source_authority: "forge".into(),
            source_envelope_id: EnvelopeId::new("env-ref-1"),
            metadata: None,
        }),
        semantics: None,
    });

    let output = compile_batch(
        &batch,
        &CompilerPolicy {
            policy_version: "policy-1".into(),
            include_hyperedges: true,
        },
    );

    assert!(
        !output.degradations.contains(&ConstraintDegradation::ThinExport),
        "alias/evidence-ref records intentionally omit semantics and must not down-classify a rich export"
    );
}

#[test]
fn alias_and_evidence_refs_preserve_thin_export_for_unreviewed_batches() {
    let mut batch = rich_batch();
    for record in &mut batch.records {
        if let Some(semantics) = record.semantics.as_mut() {
            semantics.export_confidence_class = ExportConfidenceClass::ThinExport;
        }
    }
    batch.records.push(ImportProjectionRecordV3 {
        record: ImportProjectionRecord::EntityAlias(ImportEntityAlias {
            canonical_entity_id: EntityId::new("entity-alias"),
            alias_text: "alias-name".into(),
            alias_source: "manual".into(),
            match_evidence: None,
            confidence: 0.8,
            merge_decision: MergeDecision::PendingReview,
            scope: ScopeKey::namespace_only("compiler"),
            review_state: ReviewState::Unreviewed,
            is_human_confirmed: false,
            is_human_confirmed_final: false,
            superseded_by_entity_id: None,
            split_from_entity_id: None,
            source_envelope_id: EnvelopeId::new("env-alias-thin"),
        }),
        semantics: None,
    });

    let output = compile_batch(
        &batch,
        &CompilerPolicy {
            policy_version: "policy-1".into(),
            include_hyperedges: true,
        },
    );

    assert!(
        output.degradations.contains(&ConstraintDegradation::ThinExport),
        "auxiliary alias/evidence records must keep explicitly thin exports degraded"
    );
}

#[test]
fn missing_claim_family_marker_is_explicit_without_hallucinating_thin_structure() {
    let mut batch = rich_batch();
    let mut semantics = batch.records[0]
        .semantics
        .clone()
        .expect("claim semantics should exist");
    semantics.claim_family_id = None;
    batch.records[0].semantics = Some(semantics);

    let output = compile_batch(
        &batch,
        &CompilerPolicy {
            policy_version: "policy-1".into(),
            include_hyperedges: true,
        },
    );

    assert!(output
        .degradations
        .contains(&ConstraintDegradation::MissingClaimFamily));
    assert!(
        output
            .degradations
            .iter()
            .filter(|d| **d == ConstraintDegradation::ThinExport)
            .count()
            == 0
    );
}

fn mixed_inferential_batch() -> ProjectionImportBatchV3 {
    let scope = ScopeKey::namespace_only("compiler");
    let semantics_no_group = ExportRecordSemanticsV3 {
        constraint_seed_kind: Some(ConstraintSeedKind::Hyperedge),
        ..Default::default()
    };
    let records = vec![
        ExportRecordV3 {
            record: ExportRecord::Claim(ExportClaim {
                claim_id: None,
                claim_version_id: Some("claim-version-mixed-1".into()),
                subject_entity_id: EntityId::new("entity-mixed-1"),
                predicate: "supports".into(),
                object_anchor: serde_json::json!("result"),
                valid_from: None,
                valid_to: None,
                confidence: 1.0,
                content: "entity supports result".into(),
                projection_family: "forge_verification".into(),
                supersedes_claim_id: None,
                supersedes_claim_version_id: None,
                metadata: None,
            }),
            semantics: Some(ExportRecordSemanticsV3 {
                claim_family_id: Some(ClaimFamilyId::new("mixed-family")),
                assertion_group_id: None,
                relation_group_id: None,
                ..Default::default()
            }),
        },
        ExportRecordV3 {
            record: ExportRecord::Relation(ExportRelation {
                relation_version_id: Some("relation-version-mixed-2".into()),
                subject_entity_id: EntityId::new("entity-mixed-a"),
                predicate: "causes".into(),
                object_anchor: serde_json::json!("result"),
                valid_from: None,
                valid_to: None,
                confidence: 1.0,
                projection_family: "forge_verification".into(),
                source_claim_id: None,
                source_episode_id: None,
                supersedes_relation_version_id: None,
                metadata: None,
            }),
            semantics: Some(ExportRecordSemanticsV3 {
                relation_group_id: None,
                ..Default::default()
            }),
        },
        ExportRecordV3 {
            record: ExportRecord::Episode(ExportEpisode {
                episode_id: Some(EpisodeId::new("episode-mixed-1")),
                document_id: "document-mixed-1".into(),
                cause_ids: vec!["cause-1".into()],
                effect_type: "association".into(),
                outcome: "outcome".into(),
                confidence: 0.9,
                experiment_id: None,
                metadata: None,
            }),
            semantics: Some(semantics_no_group.clone()),
        },
        ExportRecordV3 {
            record: ExportRecord::EntityAlias(ExportEntityAlias {
                canonical_entity_id: EntityId::new("entity-alias"),
                alias_text: "alias-name".into(),
                alias_source: "manual".into(),
                match_evidence: None,
                confidence: 0.8,
                scope: None,
                superseded_by_entity_id: None,
                split_from_entity_id: None,
            }),
            semantics: Some(semantics_no_group.clone()),
        },
        ExportRecordV3 {
            record: ExportRecord::EvidenceRef(ExportEvidenceRef {
                claim_id: ClaimId::new("claim-ref-1"),
                claim_version_id: Some(ClaimVersionId::new("claim-version-ref-1")),
                fetch_handle: "fetch://evidence/ref-1".into(),
                source_authority: "forge".into(),
                metadata: None,
            }),
            semantics: Some(semantics_no_group.clone()),
        },
    ];

    let digest = ExportEnvelopeV3::compute_digest(
        "forge",
        &scope,
        &records,
        Some(&ForgeExportMeta {
            authority: ExportAuthority::Forge,
            run_id: Some("run-mixed".into()),
            direct_write: false,
            comparability_snapshot_version: None,
            exported_at: "2026-03-10T00:00:00Z".into(),
        }),
        None,
    )
    .unwrap();
    let envelope = ExportEnvelopeV3 {
        envelope_id: EnvelopeId::new("env-mixed-1"),
        schema_version: EXPORT_ENVELOPE_V3_SCHEMA.into(),
        content_digest: digest,
        source_authority: "forge".into(),
        scope_key: scope,
        trace_ctx: None,
        exported_at: "2026-03-10T00:00:00Z".into(),
        export_meta: Some(ForgeExportMeta {
            authority: ExportAuthority::Forge,
            run_id: Some("run-mixed".into()),
            direct_write: false,
            comparability_snapshot_version: None,
            exported_at: "2026-03-10T00:00:00Z".into(),
        }),
        evidence_bundle: None,
        support_sets: vec![],
        contradiction_witnesses: vec![],
        retraction_records: vec![],
        claim_states_v13: vec![],
        intervention_bundles_v14: vec![],
        outcome_schemas_v14: vec![],
        cohort_contracts_v14: vec![],
        counterfactual_slices_v14: vec![],
        experiment_cases_v14: vec![],
        comparability_matrices_v14: vec![],
        decision_traces_v14: vec![],
        refuter_suites_v14: vec![],
        refuter_results_v14: vec![],
        experiment_budgets_v14: vec![],
        rollout_decisions_v14: vec![],
        rollback_decisions_v14: vec![],
        attestation_envelopes_v15: vec![],
        trust_root_sets_v15: vec![],
        artifact_admission_policies_v15: vec![],
        transparency_receipts_v15: vec![],
        attestation_revocations_v15: vec![],
        attestation_supersessions_v15: vec![],
        remote_oracle_leases_v15: vec![],
        remote_slice_requests_v15: vec![],
        remote_slice_results_v15: vec![],
        cross_runtime_replay_tickets_v15: vec![],
        dispute_bundles_v15: vec![],
        disclosure_policies_v15: vec![],
        disclosure_budgets_v15: vec![],
        records,
    };
    transform_envelope_v3(&envelope).unwrap()
}

#[test]
fn bug_001_nuisance_constraints_are_not_duplicated() {
    let output = compile_batch(
        &rich_batch(),
        &CompilerPolicy {
            policy_version: "policy-1".into(),
            include_hyperedges: true,
        },
    );
    let mut ids = output
        .constraints
        .iter()
        .map(|constraint| constraint.constraint_id.as_str())
        .collect::<Vec<_>>();
    let mut dedup = ids.clone();
    ids.sort();
    dedup.sort();
    dedup.dedup();
    assert_eq!(ids.len(), dedup.len());

    let explicit = compile_batch(
        &rich_batch(),
        &CompilerPolicy {
            policy_version: "policy-1".into(),
            include_hyperedges: false,
        },
    );
    let nuisance_constraints = explicit
        .constraints
        .iter()
        .filter(|constraint| constraint.kind == "nuisance_disclosure")
        .collect::<Vec<_>>();
    assert_eq!(nuisance_constraints.len(), 2);
}

#[test]
fn kern_001_nuisance_snapshot_hash_is_order_insensitive() {
    let forward = ExportRecordSemanticsV3 {
        nuisance_snapshot: Some(semantic_memory_forge::NuisanceSnapshot {
            environment_fingerprint: Some("env-a".into()),
            toolchain_version: Some("rust-1.87".into()),
            dependency_set_hash: Some("deps-a".into()),
            scope_mismatch_markers: vec!["scope-b".into(), "scope-a".into()],
            measurement_notes: vec!["note-2".into(), "note-1".into()],
            selection_bias_markers: vec!["bias-z".into(), "bias-a".into()],
        }),
        ..Default::default()
    };
    let reversed = ExportRecordSemanticsV3 {
        nuisance_snapshot: Some(semantic_memory_forge::NuisanceSnapshot {
            environment_fingerprint: Some("env-a".into()),
            toolchain_version: Some("rust-1.87".into()),
            dependency_set_hash: Some("deps-a".into()),
            scope_mismatch_markers: vec!["scope-a".into(), "scope-b".into()],
            measurement_notes: vec!["note-1".into(), "note-2".into()],
            selection_bias_markers: vec!["bias-a".into(), "bias-z".into()],
        }),
        ..Default::default()
    };

    assert_eq!(nuisance_node_id(&forward), nuisance_node_id(&reversed));
}

#[test]
fn kern_002_non_inferential_records_do_not_mint_nodes() {
    let output = compile_batch(
        &mixed_inferential_batch(),
        &CompilerPolicy {
            policy_version: "policy-1".into(),
            include_hyperedges: true,
        },
    );
    let node_ids = output
        .nodes
        .iter()
        .map(|node| node.node_id.as_str())
        .collect::<Vec<_>>();

    assert!(node_ids.contains(&"claim-version-mixed-1"));
    assert!(node_ids.contains(&"relation-version-mixed-2"));
    assert!(!node_ids.contains(&"episode-mixed-1"));
    assert!(!node_ids.contains(&"entity-alias"));
    assert!(!node_ids.contains(&"claim-version-ref-1"));
}

#[test]
fn bug_005_missing_group_is_inferential_only() {
    let output = compile_batch(
        &mixed_inferential_batch(),
        &CompilerPolicy {
            policy_version: "policy-1".into(),
            include_hyperedges: true,
        },
    );

    let missing_assertion_group = output
        .degradations
        .iter()
        .filter(|d| **d == ConstraintDegradation::MissingAssertionGroup)
        .count();
    let missing_relation_group = output
        .degradations
        .iter()
        .filter(|d| **d == ConstraintDegradation::MissingRelationGroup)
        .count();

    assert_eq!(missing_assertion_group, 1);
    assert_eq!(missing_relation_group, 1);
    assert!(!output.degradations.contains(&ConstraintDegradation::ThinExport));
}

#[test]
fn kern_003_oracle_candidates_exclude_nuisance_nodes() {
    let output = compile_batch(
        &rich_batch(),
        &CompilerPolicy {
            policy_version: "policy-1".into(),
            include_hyperedges: true,
        },
    );

    let candidate = output
        .oracle_candidates
        .first()
        .expect("supported batches should produce an oracle candidate");
    assert!(!candidate.node_ids.is_empty());
    assert!(candidate
        .node_ids
        .iter()
        .all(|node_id| !node_id.starts_with("nuisance:")));
}

#[test]
fn v10_geometry_manifest_keeps_graph_roles_explicitly_separate() {
    let output = compile_batch(
        &rich_batch(),
        &CompilerPolicy {
            policy_version: "policy-v10".into(),
            include_hyperedges: true,
        },
    );

    assert_eq!(
        output.geometry_manifest.surfaces,
        vec![
            GraphSurfaceKind::Storage,
            GraphSurfaceKind::Retrieval,
            GraphSurfaceKind::Inference,
            GraphSurfaceKind::Repair,
            GraphSurfaceKind::Control,
        ]
    );
    assert!(output.geometry_manifest.no_silent_collapse);
    assert!(output
        .geometry_manifest
        .compilation_boundaries
        .iter()
        .any(|boundary| {
            boundary.from_surface == GraphSurfaceKind::Inference
                && boundary.to_surface == GraphSurfaceKind::Repair
                && boundary
                    .artifact_families
                    .iter()
                    .any(|family| family == "syndrome")
        }));
}

#[test]
fn v10_regions_are_stable_and_default_bounded_units_of_work() {
    let batch = rich_batch();
    let a = compile_batch(
        &batch,
        &CompilerPolicy {
            policy_version: "policy-v10".into(),
            include_hyperedges: true,
        },
    );
    let b = compile_batch(
        &batch,
        &CompilerPolicy {
            policy_version: "policy-v10".into(),
            include_hyperedges: true,
        },
    );

    assert_eq!(a.regions, b.regions);
    assert!(!a.regions.is_empty());
    assert!(a
        .regions
        .iter()
        .all(|region| region.bounded_default_unit_of_work));
    assert!(a.regions.iter().all(|region| !region.node_ids.is_empty()));
}
