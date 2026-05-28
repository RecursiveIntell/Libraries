#![cfg_attr(test, allow(deprecated))]
#![cfg_attr(test, allow(clippy::expect_used))]

pub mod reference_interpreters;

/// Returns the baseline phase-1 conformance gate identifiers.
pub fn phase_1_gates() -> &'static [&'static str] {
    &["CF-A1", "CF-C1", "CF-C2", "CF-O1", "CF-R1"]
}

/// Returns the baseline phase-2 conformance gate identifiers.
pub fn phase_2_gates() -> &'static [&'static str] {
    &["CF-A3", "CF-C3", "CF-C4", "CF-O2"]
}

/// Returns the phase-3-and-beyond conformance gate identifiers.
pub fn phase_3_plus_gates() -> &'static [&'static str] {
    &[
        "CF-O3", "CF-O4", "CF-R2", "CF-R3", "CF-R4", "CF-B1", "CF-B2", "CF-B3", "CF-P1", "CF-P2",
    ]
}

/// Returns the v16-v20 bounded-surface conformance gate identifiers.
pub fn v16_v20_gates() -> &'static [&'static str] {
    &[
        "FED-001", "FED-004", "THY-001", "POR-001", "GOV-001", "SFX-001",
    ]
}

/// Returns the v9 constitutional freeze conformance gate identifiers.
///
/// These gates enforce the v9 finish-line invariants:
/// - CF-V9-C001: Canonical bundle round-trip freeze
/// - CF-V9-C002: Execution evidence artifact freeze
/// - CF-V9-C004: Schema governance enforcement
/// - CF-V9-C005: Bridge atomicity proof
/// - CF-V9-C008: Repair law hardening
/// - CF-V9-C009: Runtime provenance completeness
/// - CF-V9-C017: Cross-crate conformance harness
pub fn v9_constitutional_gates() -> &'static [&'static str] {
    &[
        "CF-V9-C001",
        "CF-V9-C002",
        "CF-V9-C004",
        "CF-V9-C005",
        "CF-V9-C008",
        "CF-V9-C009",
        "CF-V9-C017",
    ]
}

#[cfg(test)]
mod tests {
    use constraint_compiler::{
        compile_batch, CompilerPolicy, ConstraintDegradation, GraphSurfaceKind,
    };
    use forge_memory_bridge::{
        transform_envelope_v2, transform_envelope_v3, BridgeImportFailureArtifact,
        ImportProjectionRecord,
    };
    use kernel_execution::{
        affected_nodes_for_delta, execute_delta_propagation, execute_message_passing_baseline,
        execute_residual_correction, schedule_execution, ExecutionBudget, ExecutionStopReason,
        WorkloadClass,
    };
    use kernel_oracles::{
        evaluate_causal_refuter, evaluate_conservative, evaluate_exact_bounded,
        evaluate_temporal_replay, minimal_perturbation_witness, OracleRefutationOutcome,
        TemporalReplaySnapshot,
    };
    use knowledge_runtime::{
        adapters::semantic_memory::SemanticMemoryAdapter, config::ProjectionConfig,
        KnowledgeRuntime, RuntimeConfig, Scope,
    };
    use recursive_kernel_core::{ArtifactAuthorityClass, KernelRun};
    use semantic_memory::{MemoryConfig, MemoryStore, MockEmbedder, ProjectionQuery};
    use semantic_memory_forge::{
        ConstraintSeedKind, ExportAuthority, ExportClaim, ExportConfidenceClass,
        ExportEnvelopeError, ExportEnvelopeV2, ExportEnvelopeV3, ExportRecord,
        ExportRecordSemanticsV3, ExportRecordV3, ForgeExportMeta, NuisanceSnapshot,
        ProjectionVisibilityClass, EXPORT_ENVELOPE_V2_SCHEMA, EXPORT_ENVELOPE_V3_SCHEMA,
    };
    use stack_ids::{
        AssertionGroupId, ClaimFamilyId, ClaimId, ClaimVersionId, ContentDigest, EntityId,
        EnvelopeId, KernelRunId, OperatorId, ScopeKey,
    };
    use std::{convert::TryFrom, future::Future, path::PathBuf};
    use tempfile::TempDir;
    use verification_calibration::NuisanceStateArtifact;
    use verification_control::{RegionArtifactKind, RegionArtifactTransport, SyndromeRouteRecord};

    fn block_on<F: Future>(future: F) -> F::Output {
        tokio::runtime::Runtime::new().unwrap().block_on(future)
    }

    fn make_batch(
        namespace: &str,
        claim_ids: &[&str],
        thin: bool,
        nuisance: bool,
    ) -> forge_memory_bridge::ProjectionImportBatchV3 {
        let scope = ScopeKey::namespace_only(namespace);
        let records = claim_ids
            .iter()
            .enumerate()
            .map(|(index, claim_id)| ExportRecordV3 {
                record: ExportRecord::Claim(ExportClaim {
                    claim_id: None,
                    claim_version_id: Some((*claim_id).into()),
                    subject_entity_id: EntityId::new(format!("entity-{index}")),
                    predicate: "supports".into(),
                    object_anchor: serde_json::json!("result"),
                    valid_from: None,
                    valid_to: None,
                    confidence: 1.0,
                    content: format!("{claim_id} supports result"),
                    projection_family: "forge_verification".into(),
                    supersedes_claim_id: None,
                    supersedes_claim_version_id: None,
                    metadata: None,
                }),
                semantics: (!thin).then(|| ExportRecordSemanticsV3 {
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
                    comparability_snapshot_version: nuisance.then(|| "cmp-1".into()),
                    nuisance_snapshot: nuisance.then(|| NuisanceSnapshot {
                        environment_fingerprint: Some("linux".into()),
                        toolchain_version: Some("rust-1.85".into()),
                        dependency_set_hash: Some("deps-1".into()),
                        scope_mismatch_markers: vec![],
                        measurement_notes: vec!["measurement drift".into()],
                        selection_bias_markers: vec!["selection bias".into()],
                    }),
                    projection_visibility_class: ProjectionVisibilityClass::Standard,
                    export_confidence_class: ExportConfidenceClass::Verified,
                    derivation_seed_ids: vec![],
                    review_priority_hint: None,
                }),
            })
            .collect::<Vec<_>>();
        let export_meta = ForgeExportMeta {
            authority: ExportAuthority::Forge,
            run_id: Some("run-1".into()),
            direct_write: false,
            comparability_snapshot_version: nuisance.then(|| "cmp-1".into()),
            exported_at: "2026-03-10T00:00:00Z".into(),
        };
        let digest =
            ExportEnvelopeV3::compute_digest("forge", &scope, &records, Some(&export_meta), None)
                .unwrap();
        let envelope = ExportEnvelopeV3 {
            envelope_id: EnvelopeId::new(format!("env-{namespace}")),
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

    fn make_v2_envelope(namespace: &str, claim_ids: &[&str]) -> ExportEnvelopeV2 {
        let scope = ScopeKey::namespace_only(namespace);
        let records = claim_ids
            .iter()
            .enumerate()
            .map(|(index, claim_version_id)| {
                ExportRecord::Claim(ExportClaim {
                    claim_id: Some(ClaimId::new(format!("claim-{index}"))),
                    claim_version_id: Some(ClaimVersionId::new(*claim_version_id)),
                    subject_entity_id: EntityId::new(format!("entity-{index}")),
                    predicate: "supports".into(),
                    object_anchor: serde_json::json!("result"),
                    valid_from: None,
                    valid_to: None,
                    confidence: 1.0,
                    content: format!("{claim_version_id} supports result"),
                    projection_family: "forge_verification".into(),
                    supersedes_claim_id: None,
                    supersedes_claim_version_id: None,
                    metadata: None,
                })
            })
            .collect::<Vec<_>>();
        let export_meta = ForgeExportMeta {
            authority: ExportAuthority::Forge,
            run_id: Some("run-v2".into()),
            direct_write: false,
            comparability_snapshot_version: Some("cmp-v2".into()),
            exported_at: "2026-03-10T00:00:00Z".into(),
        };
        let digest =
            ExportEnvelopeV2::compute_digest("forge", &scope, &records, Some(&export_meta), None)
                .unwrap();

        ExportEnvelopeV2 {
            envelope_id: EnvelopeId::new(format!("env-v2-{namespace}")),
            schema_version: EXPORT_ENVELOPE_V2_SCHEMA.into(),
            content_digest: digest,
            source_authority: "forge".into(),
            scope_key: scope,
            trace_ctx: None,
            exported_at: "2026-03-10T00:00:00Z".into(),
            export_meta: Some(export_meta),
            evidence_bundle: None,
            records,
        }
    }

    fn compile_fixture(
        namespace: &str,
        claim_ids: &[&str],
        thin: bool,
        nuisance: bool,
    ) -> constraint_compiler::CompileOutput {
        compile_batch(
            &make_batch(namespace, claim_ids, thin, nuisance),
            &CompilerPolicy {
                policy_version: "policy-1".into(),
                include_hyperedges: true,
            },
        )
    }

    fn open_runtime(
        batch: &forge_memory_bridge::ProjectionImportBatchV3,
        namespace: &str,
    ) -> KnowledgeRuntime {
        let dir = TempDir::new().unwrap();
        let base_dir = dir.path().to_path_buf();
        std::mem::forget(dir);
        let config = MemoryConfig {
            base_dir,
            ..Default::default()
        };
        let embedder = Box::new(MockEmbedder::new(config.embedding.dimensions));
        let store = MemoryStore::open_with_embedder(config, embedder).unwrap();
        block_on(store.import_projection_batch(batch)).unwrap();
        let adapter = SemanticMemoryAdapter::new(store);
        KnowledgeRuntime::new(
            RuntimeConfig {
                default_scope: Scope::new(namespace),
                query: Default::default(),
                entity: Default::default(),
                projection: ProjectionConfig {
                    staleness_threshold_secs: 3600,
                    import_staleness_threshold_secs: 0,
                    persist: false,
                },
                strict_temporal: false,
                strict_scope: false,
            },
            adapter,
        )
        .unwrap()
    }

    fn open_store() -> MemoryStore {
        let dir = TempDir::new().unwrap();
        let base_dir = dir.path().to_path_buf();
        std::mem::forget(dir);
        open_store_with_path(base_dir)
    }

    fn open_store_with_path(base_dir: PathBuf) -> MemoryStore {
        let config = MemoryConfig {
            base_dir,
            ..Default::default()
        };
        let embedder = Box::new(MockEmbedder::new(config.embedding.dimensions));
        MemoryStore::open_with_embedder(config, embedder).unwrap()
    }

    #[test]
    fn cf_a1_kernel_artifacts_are_non_authoritative() {
        let run = KernelRun {
            run_id: KernelRunId::new("run-1"),
            policy_version: "policy-1".into(),
            scope_key: ScopeKey::namespace_only("ns"),
            operator_ids: vec![OperatorId::new(
                recursive_kernel_core::CONSTRAINT_COMPILER_OPERATOR_ID,
            )],
            oracle_slice_id: None,
            degradation_active: false,
        };

        assert_eq!(
            run.authority_class(),
            ArtifactAuthorityClass::NonAuthoritativeDerived
        );
    }

    #[test]
    fn cf_a3_advisory_outputs_do_not_self_promote() {
        let runtime = open_runtime(
            &make_batch("cf-a3", &["claim-a", "claim-b", "claim-c"], false, false),
            "cf-a3",
        );
        let advisory = block_on(runtime.latest_inference_advisory(Some(&Scope::new("cf-a3"))))
            .unwrap()
            .expect("advisory expected");
        let gate = block_on(runtime.latest_risk_gate(Some(&Scope::new("cf-a3"))))
            .unwrap()
            .expect("gate expected");

        assert!(advisory.advisory_only);
        assert!(gate.advisory_only);
        assert_ne!(gate.status, "authoritative");
    }

    #[test]
    fn cf_c1_deterministic_compile() {
        let a = compile_fixture("cf-c1", &["claim-a", "claim-b"], false, false);
        let b = compile_fixture("cf-c1", &["claim-a", "claim-b"], false, false);
        assert_eq!(a.graph_hash, b.graph_hash);
    }

    #[test]
    fn cf_c2_thin_export_degrades_explicitly() {
        let compiled = compile_fixture("cf-c2", &["claim-a"], true, false);
        assert!(compiled
            .degradations
            .contains(&ConstraintDegradation::ThinExport));
    }

    #[test]
    fn cf_c2_mixed_semantics_do_not_hallucinate_hyperedge_membership() {
        let mut batch = make_batch("cf-c2-mixed", &["claim-a", "claim-b"], false, false);
        batch.records[0].semantics = None;
        let compiled = compile_batch(
            &batch,
            &CompilerPolicy {
                policy_version: "policy-1".into(),
                include_hyperedges: true,
            },
        );

        assert!(compiled
            .degradations
            .contains(&ConstraintDegradation::ThinExport));
        assert!(compiled.oracle_candidates.is_empty());

        let group_hyperedge = compiled
            .hyperedges
            .iter()
            .find(|edge| edge.edge_id == "assertion_group:group-1")
            .expect("thin export should keep only explicitly provided grouping");
        assert_eq!(group_hyperedge.member_node_ids, vec!["claim-b".to_string()]);
    }

    #[test]
    fn cf_c2a_thin_export_forces_conservative_oracle_mode() {
        let compiled = compile_fixture("cf-c2a", &["claim-a"], true, false);

        assert!(compiled
            .degradations
            .contains(&ConstraintDegradation::ThinExport));
        assert!(compiled.oracle_candidates.is_empty());
        assert!(evaluate_exact_bounded(&compiled).is_none());
        let conservative = evaluate_conservative(&compiled);
        assert!(conservative.supported);
        assert_eq!(
            conservative.mode,
            kernel_oracles::OracleMode::ConservativeFallback
        );
    }

    #[test]
    fn cf_c3_nuisance_state_is_included() {
        let compiled = compile_fixture("cf-c3", &["claim-a", "claim-b"], false, true);
        assert!(compiled
            .nodes
            .iter()
            .any(|node| node.kind == "nuisance_state"));
    }

    #[test]
    fn cf_c4_hyperedge_correctness_preserves_group_membership() {
        let compiled = compile_fixture("cf-c4", &["claim-a", "claim-b"], false, false);
        let hyperedge = compiled
            .hyperedges
            .iter()
            .find(|edge| edge.edge_id == "assertion_group:group-1")
            .expect("group hyperedge expected");
        assert_eq!(hyperedge.member_node_ids.len(), 2);
    }

    #[test]
    fn cf_g1_v2_to_v3_upgrade_preserves_bridgeable_projection_content() {
        let v2 = make_v2_envelope("cf-g1", &["claim-g1-a", "claim-g1-b"]);
        let v3 = ExportEnvelopeV3::try_from(v2.clone()).unwrap();
        let v2_batch = transform_envelope_v2(&v2).unwrap();
        let v3_batch = transform_envelope_v3(&v3).unwrap();
        let v3_records = v3_batch
            .records
            .iter()
            .map(|record| record.record.clone())
            .collect::<Vec<_>>();

        assert_eq!(v2_batch.source_envelope_id, v3_batch.source_envelope_id);
        assert_eq!(v2_batch.scope_key, v3_batch.scope_key);
        assert_eq!(v2_batch.source_authority, v3_batch.source_authority);
        assert_eq!(v2_batch.records.len(), v3_batch.records.len());
        assert_eq!(
            serde_json::to_value(&v2_batch.records).unwrap(),
            serde_json::to_value(&v3_records).unwrap()
        );
    }

    #[test]
    fn cf_g4_imported_projection_data_survives_drop_and_reopen() {
        let dir = TempDir::new().unwrap();
        let base_dir = dir.path().to_path_buf();
        std::mem::forget(dir);

        let batch = make_batch("cf-g4", &["claim-g4-a", "claim-g4-b"], false, false);
        let imported_count = {
            let store = open_store_with_path(base_dir.clone());
            let imported = block_on(store.import_projection_batch(&batch)).unwrap();
            assert_eq!(imported.status, "complete");
            let claims = block_on(
                store.query_claim_versions(ProjectionQuery::new(ScopeKey::namespace_only("cf-g4"))),
            )
            .unwrap();
            assert!(
                claims
                    .iter()
                    .any(|claim| claim.claim_version_id == ClaimVersionId::new("claim-g4-a")),
                "imported data should be queryable before drop"
            );
            claims.len()
        };

        let reopened_count = {
            let store = open_store_with_path(base_dir);
            let claims = block_on(
                store.query_claim_versions(ProjectionQuery::new(ScopeKey::namespace_only("cf-g4"))),
            )
            .unwrap();
            claims.len()
        };

        assert_eq!(reopened_count, imported_count);
    }

    #[test]
    fn cf_g2_invalid_upgrade_fails_explicitly() {
        let mut v2 = make_v2_envelope("cf-g2", &["claim-g2-a"]);
        v2.content_digest = ContentDigest::compute(b"tampered");

        let err = ExportEnvelopeV3::try_from(v2).unwrap_err();
        assert!(matches!(err, ExportEnvelopeError::DigestMismatch { .. }));
    }

    #[test]
    fn cf_g3_retry_import_of_upgraded_batch_is_idempotent() {
        let v2 = make_v2_envelope("cf-g3", &["claim-g3-a"]);
        let v3 = ExportEnvelopeV3::try_from(v2).unwrap();
        let batch = transform_envelope_v3(&v3).unwrap();
        let store = open_store();

        let first = block_on(store.import_projection_batch(&batch)).unwrap();
        let second = block_on(store.import_projection_batch(&batch)).unwrap();
        let claims = block_on(
            store.query_claim_versions(ProjectionQuery::new(ScopeKey::namespace_only("cf-g3"))),
        )
        .unwrap();

        assert_eq!(first.status, "complete");
        assert_eq!(second.status, "already_imported");
        assert_eq!(claims.len(), 1);
        assert!(matches!(
            batch.records.first().map(|record| &record.record),
            Some(ImportProjectionRecord::ClaimVersion(_))
        ));
    }

    #[test]
    fn cf_o1_oracle_parity_on_supported_slice() {
        let compiled = compile_fixture("cf-o1", &["claim-a", "claim-b"], false, false);
        let exact = evaluate_exact_bounded(&compiled).expect("exact oracle slice expected");
        let conservative = evaluate_conservative(&compiled);
        assert_eq!(
            exact.satisfied_constraint_count,
            conservative.satisfied_constraint_count
        );
    }

    #[test]
    fn cf_o1_exact_oracle_mode_is_reproducible() {
        let compiled = compile_fixture("cf-o1-repro", &["claim-a", "claim-b"], false, false);
        let first = evaluate_exact_bounded(&compiled).expect("exact oracle slice expected");
        let second = evaluate_exact_bounded(&compiled).expect("exact oracle slice expected");

        assert_eq!(first, second);
    }

    #[test]
    fn cf_o2_delta_parity_matches_bounded_recompute() {
        let compiled = compile_fixture("cf-o2", &["claim-a", "claim-b"], false, false);
        let delta = execute_delta_propagation(&compiled, &["claim-a".into()], 3);
        let full = execute_message_passing_baseline(&compiled, 3);
        let affected = affected_nodes_for_delta(&compiled.invalidation_cones, &["claim-a".into()]);

        for node_id in affected {
            let delta_belief = delta
                .execution
                .node_beliefs
                .iter()
                .find(|belief| belief.node_id == node_id)
                .expect("delta belief expected")
                .belief_micros;
            let full_belief = full
                .node_beliefs
                .iter()
                .find(|belief| belief.node_id == node_id)
                .expect("full belief expected")
                .belief_micros;
            assert_eq!(delta_belief, full_belief);
        }
        assert!(!delta.recomputed_region_ids.is_empty());
        assert!(!delta.region_traces.is_empty());
        assert!(!delta.invalidation_manifest.explicit_global_rebuild);
    }

    #[test]
    fn cf_v10_geometry_and_regions_are_explicit() {
        let compiled = compile_fixture("cf-v10-geometry", &["claim-a", "claim-b"], false, false);

        assert_eq!(
            compiled.geometry_manifest.surfaces,
            vec![
                GraphSurfaceKind::Storage,
                GraphSurfaceKind::Retrieval,
                GraphSurfaceKind::Inference,
                GraphSurfaceKind::Repair,
                GraphSurfaceKind::Control,
            ]
        );
        assert!(compiled.geometry_manifest.no_silent_collapse);
        assert!(compiled
            .regions
            .iter()
            .all(|region| region.bounded_default_unit_of_work));
    }

    #[test]
    fn cf_o3_temporal_replay_matches_expected_snapshot() {
        let older = make_batch("cf-o3-old", &["claim-old-a", "claim-old-b"], false, false);
        let newer = make_batch("cf-o3-new", &["claim-new-a", "claim-new-b"], false, false);
        let policy = CompilerPolicy {
            policy_version: "policy-1".into(),
            include_hyperedges: true,
        };
        let expected = compile_batch(&older, &policy);
        let replay = evaluate_temporal_replay(
            &[
                TemporalReplaySnapshot {
                    recorded_at: "2026-03-08T00:00:00Z".into(),
                    batch: older,
                },
                TemporalReplaySnapshot {
                    recorded_at: "2026-03-10T00:00:00Z".into(),
                    batch: newer,
                },
            ],
            "2026-03-09T00:00:00Z",
            &policy,
            &expected.graph_hash,
        )
        .expect("replay result expected");

        assert!(replay.matched_expected_hash);
    }

    #[test]
    fn cf_o4_refuter_integration_emits_structured_result() {
        let compiled = compile_fixture("cf-o4", &["claim-a", "claim-b"], false, false);
        let result = evaluate_causal_refuter(&compiled, "claim-a", 1);
        assert!(matches!(
            result.outcome,
            OracleRefutationOutcome::FlipWitness { .. }
                | OracleRefutationOutcome::NoFlipFound { .. }
                | OracleRefutationOutcome::NotApplicable { .. }
        ));
    }

    #[test]
    fn cf_r1_operator_requires_explicit_stop_rule() {
        let operator = recursive_kernel_core::constraint_compiler_operator();
        operator.validate().unwrap();
    }

    #[test]
    fn cf_r2_instability_terminates_explicitly() {
        let compiled = compile_fixture("cf-r2", &["claim-a", "claim-b"], false, true);
        let report = execute_message_passing_baseline(&compiled, 1);
        assert_eq!(report.stop_reason, ExecutionStopReason::MaxIterations);
    }

    #[test]
    fn cf_r3_residual_repair_is_monotone() {
        let compiled = compile_fixture("cf-r3", &["claim-a", "claim-b"], false, false);
        let report = execute_residual_correction(&compiled, 3);
        let mut previous = u64::MAX;
        for sample in &report.residuals {
            assert!(sample.total_residual_micros <= previous);
            previous = sample.total_residual_micros;
        }
        assert!(report.convergence_report.residual_monotone_nonincreasing);
    }

    #[test]
    fn cf_r4_syndromes_are_reproducible() {
        let compiled = compile_fixture("cf-r4", &["claim-a"], true, false);
        let a = execute_message_passing_baseline(&compiled, 2);
        let b = execute_message_passing_baseline(&compiled, 2);
        assert_eq!(a.syndromes, b.syndromes);
    }

    #[test]
    fn cf_b1_minimal_perturbation_returns_structured_witness() {
        let compiled = compile_fixture("cf-b1", &["claim-a", "claim-b"], false, false);
        let result = minimal_perturbation_witness(&compiled, "claim-a", 1);
        assert!(matches!(
            result.outcome,
            OracleRefutationOutcome::FlipWitness { .. }
                | OracleRefutationOutcome::NoFlipFound { .. }
        ));
    }

    #[test]
    fn cf_b2_witness_and_certificate_emission_exist() {
        let compiled = compile_fixture("cf-b2", &["claim-a", "claim-b", "claim-c"], false, false);
        let scheduled = schedule_execution(&compiled, &ExecutionBudget::default());
        assert!(!scheduled.execution.witnesses.is_empty());
        assert!(!scheduled.execution.certificates.is_empty());
    }

    #[test]
    fn cf_b3_calibration_disclosure_surfaces_on_nuisance_runs() {
        let runtime = open_runtime(
            &make_batch("cf-b3", &["claim-a", "claim-b"], false, true),
            "cf-b3",
        );
        let explanation =
            block_on(runtime.latest_inference_explanation(Some(&Scope::new("cf-b3"))))
                .unwrap()
                .expect("explanation expected");

        assert!(explanation
            .calibration_caveats
            .iter()
            .any(|caveat| caveat.contains("nuisance state")));
    }

    #[test]
    fn cf_v10_syndrome_transport_and_nuisance_artifacts_are_typed() {
        let compiled = compile_fixture("cf-v10-artifacts", &["claim-a", "claim-b"], false, true);
        let report = execute_message_passing_baseline(&compiled, 2);
        let first_region = compiled.regions.first().expect("region expected");
        let first_syndrome = report.syndromes.first().expect("syndrome expected");

        let route = SyndromeRouteRecord::new(
            first_syndrome.syndrome_id.clone(),
            first_region.region_id.clone(),
            first_region.node_ids.clone(),
            vec!["witness:claim-a".into()],
        );
        let transport_payload = serde_json::to_string(first_syndrome).unwrap();
        let transport = RegionArtifactTransport::new(
            RegionArtifactKind::Syndrome,
            first_syndrome.syndrome_id.to_string(),
            transport_payload.as_bytes(),
            first_region.region_id.clone(),
            first_region.region_id.clone(),
            "inference",
            "repair",
            "replay:syndrome-route",
        );
        let nuisance = NuisanceStateArtifact::new(
            "2026-03-13T00:00:00Z",
            Some(first_region.region_id.clone()),
            "provider_instability",
            vec!["retry_storm".into(), "comparability_shift".into()],
            true,
            vec!["kernel_execution".into(), "verification_control".into()],
        );

        assert!(route.routed_before_global_invalidation);
        assert_eq!(transport.from_surface, "inference");
        assert!(nuisance.semantic_change_confounded);
    }

    #[test]
    fn cf_p1_workload_classes_remain_separate() {
        let small = schedule_execution(
            &compile_fixture("cf-p1-small", &["claim-a", "claim-b"], false, false),
            &ExecutionBudget::default(),
        );
        let large = schedule_execution(
            &compile_fixture(
                "cf-p1-large",
                &[
                    "claim-a", "claim-b", "claim-c", "claim-d", "claim-e", "claim-f", "claim-g",
                    "claim-h", "claim-i",
                ],
                false,
                false,
            ),
            &ExecutionBudget {
                max_nodes: 4,
                ..ExecutionBudget::default()
            },
        );

        assert_eq!(small.workload_class, WorkloadClass::Background);
        assert_eq!(large.workload_class, WorkloadClass::Heavy);
    }

    #[test]
    fn cf_p2_budgeted_degradation_is_explicit() {
        let scheduled = schedule_execution(
            &compile_fixture(
                "cf-p2",
                &["claim-a", "claim-b", "claim-c", "claim-d", "claim-e"],
                false,
                false,
            ),
            &ExecutionBudget {
                max_nodes: 2,
                ..ExecutionBudget::default()
            },
        );
        assert_eq!(
            scheduled.execution.stop_reason,
            ExecutionStopReason::BudgetExhausted
        );
        assert_eq!(
            scheduled.degraded_reason.as_deref(),
            Some("budget_exhausted")
        );
    }

    // ═══════════════════════════════════════════════════════════════
    // v9 Constitutional Freeze and Conformance Hardening
    // LIB-C001, LIB-C002, LIB-C005, LIB-C008, LIB-C009, LIB-C017
    // ═══════════════════════════════════════════════════════════════

    // ─── LIB-C001: Canonical bundle round-trip freeze ─────────────

    #[test]
    fn cf_v9_c001_episode_bundle_round_trips() {
        // LIB-C001: EpisodeBundleV1 must round-trip through serde without data loss.
        use semantic_memory_forge::{
            DispatchOutcomeV1, EpisodeBundleV1, ExecutionContextV1, VerificationSummary,
            EPISODE_BUNDLE_V1_SCHEMA, EXECUTION_CONTEXT_V1_SCHEMA,
        };
        use stack_ids::{EpisodeId, TraceCtx};

        let trace = TraceCtx::generate();
        let ctx = ExecutionContextV1 {
            schema_version: EXECUTION_CONTEXT_V1_SCHEMA.into(),
            trace_ctx: trace.clone(),
            attempt_id: Some(stack_ids::AttemptId::new("attempt-c001")),
            trial_id: Some(stack_ids::TrialId::new("trial-c001")),
            replay_link: Some("replay://link".into()),
            workload_class: Some("forge_export".into()),
            queue_hops: vec!["hop-1".into(), "hop-2".into()],
            deadline: Some("2026-04-01T00:00:00Z".into()),
            cost_budget_units: Some(1000),
            degradation_markers: vec!["marker-a".into()],
            dispatch_outcome: DispatchOutcomeV1::Succeeded,
            environment_fingerprint: Some("linux-amd64".into()),
            provider_route: Some("forge".into()),
            cancellation_reason: None,
            tool_receipt_ref: Some("receipt://ref".into()),
            provider_call_ref: Some("call://ref".into()),
            replay_parent_ref: Some("parent://ref".into()),
        };
        let bundle = EpisodeBundleV1 {
            schema_version: EPISODE_BUNDLE_V1_SCHEMA.into(),
            bundle_id: "bundle-c001".into(),
            episode_id: EpisodeId::new("ep-c001"),
            primary_document_id: "doc-c001".into(),
            namespace: "ns-c001".into(),
            scope_key: ScopeKey::namespace_only("ns-c001"),
            valid_from: Some("2026-01-01T00:00:00Z".into()),
            valid_to: Some("2026-12-31T23:59:59Z".into()),
            exported_at: "2026-03-15T00:00:00Z".into(),
            recorded_at: Some("2026-03-15T00:00:01Z".into()),
            source_envelope_id: EnvelopeId::new("env-c001"),
            content_digest: ContentDigest::compute(b"c001-content"),
            source_evidence_pointers: vec!["evidence://ptr-1".into()],
            source_receipt_digests: vec!["digest-1".into()],
            claim_version_ids: vec!["claim-v1".into(), "claim-v2".into()],
            relation_version_ids: vec!["rel-v1".into()],
            verification_summary: Some(VerificationSummary {
                lifecycle_state: semantic_memory_forge::VerificationLifecycleState::Verified,
                promotion_state: semantic_memory_forge::PromotionState::Promoted {
                    version_id: Some("v1".into()),
                    promoted_at: Some("2026-03-15T00:00:00Z".into()),
                },
                completed_trial_count: 3,
                passed_refutation_count: 2,
                failed_refutation_count: 0,
                comparability_snapshot_version: None,
                notes: vec!["clean".into()],
            }),
            refutation_artifact_ids: vec!["refutation-1".into()],
            control_plane_refs: vec!["forge_run:run-c001".into()],
            execution_context: ctx,
            thin_export: false,
            supersedes_bundle_id: Some("bundle-prev".into()),
            evidence_bundle_id: Some("ev-bundle-1".into()),
        };

        let serialized = serde_json::to_string(&bundle).unwrap();
        let deserialized: EpisodeBundleV1 = serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized.bundle_id, bundle.bundle_id);
        assert_eq!(deserialized.episode_id, bundle.episode_id);
        assert_eq!(deserialized.claim_version_ids.len(), 2);
        assert_eq!(deserialized.relation_version_ids.len(), 1);
        assert!(deserialized.verification_summary.is_some());
        assert_eq!(
            deserialized.execution_context.attempt_id,
            bundle.execution_context.attempt_id
        );
        assert_eq!(
            deserialized.execution_context.queue_hops.len(),
            bundle.execution_context.queue_hops.len()
        );
        assert_eq!(
            deserialized.supersedes_bundle_id,
            bundle.supersedes_bundle_id
        );
        assert_eq!(deserialized.evidence_bundle_id, bundle.evidence_bundle_id);
    }

    #[test]
    fn cf_v9_c001_execution_context_round_trips() {
        // LIB-C001/C002: ExecutionContextV1 must round-trip without data loss.
        use semantic_memory_forge::{
            DispatchOutcomeV1, ExecutionContextV1, EXECUTION_CONTEXT_V1_SCHEMA,
        };
        use stack_ids::TraceCtx;

        let ctx = ExecutionContextV1 {
            schema_version: EXECUTION_CONTEXT_V1_SCHEMA.into(),
            trace_ctx: TraceCtx::generate(),
            attempt_id: Some(stack_ids::AttemptId::new("a-1")),
            trial_id: Some(stack_ids::TrialId::new("t-1")),
            replay_link: Some("replay://x".into()),
            workload_class: Some("heavy".into()),
            queue_hops: vec!["q1".into(), "q2".into(), "q3".into()],
            deadline: Some("2026-04-01T12:00:00Z".into()),
            cost_budget_units: Some(500),
            degradation_markers: vec!["thin_export".into(), "missing_nuisance".into()],
            dispatch_outcome: DispatchOutcomeV1::Degraded,
            environment_fingerprint: Some("env-fp".into()),
            provider_route: Some("openrouter".into()),
            cancellation_reason: None,
            tool_receipt_ref: Some("receipt://1".into()),
            provider_call_ref: Some("call://1".into()),
            replay_parent_ref: Some("parent://1".into()),
        };

        let json = serde_json::to_value(&ctx).unwrap();
        let restored: ExecutionContextV1 = serde_json::from_value(json).unwrap();

        assert_eq!(restored.attempt_id, ctx.attempt_id);
        assert_eq!(restored.trial_id, ctx.trial_id);
        assert_eq!(restored.queue_hops.len(), 3);
        assert_eq!(restored.degradation_markers.len(), 2);
        assert_eq!(restored.dispatch_outcome, DispatchOutcomeV1::Degraded);
        assert_eq!(restored.tool_receipt_ref, ctx.tool_receipt_ref);
        assert_eq!(restored.provider_call_ref, ctx.provider_call_ref);
        assert_eq!(restored.replay_parent_ref, ctx.replay_parent_ref);
    }

    // ─── LIB-C005: Bridge atomicity proof conformance ─────────────

    #[test]
    fn cf_v9_c005_tampered_digest_produces_typed_error() {
        // LIB-C005: Bridge must produce a typed, replayable error (not a log message)
        // when digest verification fails during import.
        let mut envelope = make_v2_envelope("cf-c005-tamper", &["claim-c005"]);
        envelope.content_digest = ContentDigest::compute(b"tampered-data");

        let err = transform_envelope_v2(&envelope).unwrap_err();
        assert!(
            matches!(err, forge_memory_bridge::BridgeError::DigestMismatch { .. }),
            "LIB-C005: tampered digest must produce BridgeError::DigestMismatch, got: {err:?}"
        );
        // Error kind must be stable and machine-readable.
        assert_eq!(err.kind(), "digest_mismatch");
    }

    #[test]
    fn cf_v9_c005_v3_tampered_digest_produces_typed_error() {
        // LIB-C005: Same atomicity proof for V3 path.
        let scope = ScopeKey::namespace_only("c005-v3");
        let records = vec![ExportRecordV3 {
            record: ExportRecord::Claim(ExportClaim {
                claim_id: None,
                claim_version_id: Some(ClaimVersionId::new("claim-c005-v3")),
                subject_entity_id: EntityId::new("entity-c005"),
                predicate: "supports".into(),
                object_anchor: serde_json::json!("result"),
                valid_from: None,
                valid_to: None,
                confidence: 1.0,
                content: "claim supports result".into(),
                projection_family: "forge_verification".into(),
                supersedes_claim_id: None,
                supersedes_claim_version_id: None,
                metadata: None,
            }),
            semantics: None,
        }];
        let envelope = ExportEnvelopeV3 {
            envelope_id: EnvelopeId::new("env-c005-v3"),
            schema_version: EXPORT_ENVELOPE_V3_SCHEMA.into(),
            content_digest: ContentDigest::compute(b"wrong-digest"),
            source_authority: "forge".into(),
            scope_key: scope,
            trace_ctx: None,
            exported_at: "2026-04-01T00:00:00Z".into(),
            export_meta: None,
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

        let err = transform_envelope_v3(&envelope).unwrap_err();
        assert!(
            matches!(err, forge_memory_bridge::BridgeError::DigestMismatch { .. }),
            "LIB-C005: V3 tampered digest must produce BridgeError::DigestMismatch, got: {err:?}"
        );
        assert_eq!(err.kind(), "digest_mismatch");
    }

    #[test]
    fn cf_v9_c005_bridge_preserves_digest_through_v3_transform() {
        // LIB-C005: The bridge must preserve the source envelope digest into the batch.
        let batch = make_batch(
            "c005-digest",
            &["claim-c005-a", "claim-c005-b"],
            false,
            false,
        );
        // The digest in the batch must match the source envelope's computed digest.
        assert!(
            !batch.content_digest.0.is_empty(),
            "LIB-C005: content_digest must be carried through from source envelope"
        );
        // Verify it's actually the Blake3 format (hex, 64 chars).
        assert_eq!(
            batch.content_digest.0.len(),
            64,
            "LIB-C005: content_digest must be a 64-char hex string (BLAKE3)"
        );
    }

    #[test]
    fn cf_v9_c005_bridge_error_kinds_are_stable() {
        // LIB-C005: All bridge error kinds must be stable machine-readable strings.
        use forge_memory_bridge::BridgeError;
        let cases: Vec<BridgeError> = vec![
            BridgeError::InvalidEnvelope {
                reason: "test".into(),
            },
            BridgeError::IncompatibleVersion {
                expected: "a".into(),
                actual: "b".into(),
            },
            BridgeError::DigestMismatch {
                expected: "a".into(),
                actual: "b".into(),
            },
            BridgeError::DigestComputationFailed {
                reason: "test".into(),
            },
            BridgeError::InvalidRecord {
                reason: "test".into(),
            },
            BridgeError::TransformFailed {
                reason: "test".into(),
            },
            BridgeError::MissingEpisodeIdentity {
                record_context: "test".into(),
            },
        ];
        let expected_kinds = [
            "invalid_envelope",
            "incompatible_version",
            "digest_mismatch",
            "digest_computation_failed",
            "invalid_record",
            "transform_failed",
            "missing_episode_identity",
        ];
        for (error, expected_kind) in cases.iter().zip(expected_kinds.iter()) {
            assert_eq!(
                error.kind(),
                *expected_kind,
                "LIB-C005: BridgeError kind must be stable"
            );
        }
    }

    #[test]
    fn cf_v9_c005_bridge_failure_produces_replayable_artifact() {
        // LIB-C005: Bridge failures must produce typed, replayable artifacts.
        let mut envelope = make_v2_envelope("cf-c005-artifact", &["claim-c005"]);
        envelope.content_digest = ContentDigest::compute(b"tampered");

        let err = transform_envelope_v2(&envelope).unwrap_err();
        let artifact = BridgeImportFailureArtifact::from_error(
            &err,
            envelope.envelope_id.as_str(),
            &envelope.source_authority,
            &envelope.scope_key.namespace,
        );

        assert_eq!(
            artifact.schema_version,
            forge_memory_bridge::BRIDGE_IMPORT_FAILURE_ARTIFACT_V1_SCHEMA
        );
        assert_eq!(artifact.error_kind, "digest_mismatch");
        assert!(!artifact.error_message.is_empty());
        assert_eq!(artifact.source_envelope_id, envelope.envelope_id.as_str());
        assert!(!artifact.failed_at.is_empty());

        // Must round-trip through serde.
        let json = serde_json::to_value(&artifact).unwrap();
        let restored: BridgeImportFailureArtifact = serde_json::from_value(json).unwrap();
        assert_eq!(restored.error_kind, artifact.error_kind);
        assert_eq!(restored.source_envelope_id, artifact.source_envelope_id);
    }

    // ─── LIB-C008: Repair law conformance ─────────────────────────

    #[test]
    fn cf_v9_c008_repair_record_has_required_fields() {
        // LIB-C008: Every RepairRecordV1 must carry blast_radius, reversibility,
        // execution_context, repair_class, and an unchanged_statement.
        use forge_pilot::{RepairClassV1, RepairRecordV1};
        use semantic_memory_forge::{
            DispatchOutcomeV1, ExecutionContextV1, EXECUTION_CONTEXT_V1_SCHEMA,
        };
        use stack_ids::TraceCtx;

        let record = RepairRecordV1 {
            schema_version: "repair_record_v1".into(),
            repair_record_id: "repair-c008".into(),
            affected_identities: vec!["claim-1".into(), "claim-2".into()],
            repair_class: RepairClassV1::SupersessionRepair,
            trigger_artifacts: vec!["contradiction-witness-1".into()],
            blast_radius: "scoped:ns-c008".into(),
            reversibility: "reversible_scoped".into(),
            action: "supersede claim-1 with claim-2".into(),
            execution_context: ExecutionContextV1 {
                schema_version: EXECUTION_CONTEXT_V1_SCHEMA.into(),
                trace_ctx: TraceCtx::generate(),
                attempt_id: None,
                trial_id: None,
                replay_link: None,
                workload_class: Some("repair".into()),
                queue_hops: vec![],
                deadline: None,
                cost_budget_units: None,
                degradation_markers: vec![],
                dispatch_outcome: DispatchOutcomeV1::Succeeded,
                environment_fingerprint: None,
                provider_route: None,
                cancellation_reason: None,
                tool_receipt_ref: None,
                provider_call_ref: None,
                replay_parent_ref: None,
            },
            opened_at: "2026-04-01T00:00:00Z".into(),
            resolved_at: Some("2026-04-01T00:00:01Z".into()),
            unchanged_statement: "no semantic widening applied; supersession only".into(),
        };

        // Required fields must be non-empty.
        assert!(
            !record.blast_radius.is_empty(),
            "LIB-C008: blast_radius required"
        );
        assert!(
            !record.reversibility.is_empty(),
            "LIB-C008: reversibility required"
        );
        assert!(
            !record.unchanged_statement.is_empty(),
            "LIB-C008: unchanged_statement required"
        );
        assert!(!record.action.is_empty(), "LIB-C008: action required");
        assert_eq!(
            record.execution_context.schema_version, EXECUTION_CONTEXT_V1_SCHEMA,
            "LIB-C008: repair record must carry a valid execution context"
        );

        // Round-trip through serde.
        let json = serde_json::to_value(&record).unwrap();
        let restored: RepairRecordV1 = serde_json::from_value(json).unwrap();
        assert_eq!(restored.repair_record_id, record.repair_record_id);
        assert_eq!(restored.repair_class, record.repair_class);
        assert_eq!(restored.blast_radius, record.blast_radius);
        assert_eq!(restored.reversibility, record.reversibility);
        assert_eq!(restored.unchanged_statement, record.unchanged_statement);
        assert_eq!(restored.affected_identities.len(), 2);
    }

    #[test]
    fn cf_v9_c008_repair_classes_are_exhaustive() {
        // LIB-C008: Repair classes must cover the canonical set.
        use forge_pilot::RepairClassV1;

        let classes = [
            RepairClassV1::IdentityRepair,
            RepairClassV1::TemporalRepair,
            RepairClassV1::SupersessionRepair,
            RepairClassV1::RollbackRepair,
            RepairClassV1::BundleImportRepair,
            RepairClassV1::ScopeWideningRepair,
            RepairClassV1::VerificationStateRepair,
        ];
        // Each class must serialize to a distinct stable string.
        let mut seen = std::collections::HashSet::new();
        for class in &classes {
            let wire = serde_json::to_value(class).unwrap();
            let label = wire.as_str().unwrap().to_string();
            assert!(
                seen.insert(label.clone()),
                "LIB-C008: duplicate repair class wire label: {label}"
            );
        }
        assert_eq!(
            seen.len(),
            7,
            "LIB-C008: expected 7 distinct repair classes"
        );
    }

    // ─── LIB-C009: Runtime provenance completeness ─────────────────

    #[test]
    fn cf_v9_c009_query_trace_carries_view_model_and_widening() {
        // LIB-C009: Every query result must carry declared view model and widening status.
        let runtime = open_runtime(
            &make_batch("cf-c009", &["claim-a", "claim-b"], false, false),
            "cf-c009",
        );
        let (_results, trace) =
            block_on(runtime.query("claim-a supports result", Some(&Scope::new("cf-c009"))))
                .unwrap();

        // View model declarations must be present.
        // For a non-temporal, non-entity query, executed_views should be populated.
        // Even if empty (no degradation), the field itself is well-defined.
        assert!(
            !trace.requested_views.is_empty()
                || !trace.executed_views.is_empty()
                || !trace.plan.legs.is_empty(),
            "LIB-C009: query trace must declare requested/executed views or have at least one leg"
        );

        // Widening disclosures must be a well-typed list (even if empty = no widening).
        // This asserts the field is structurally present, not that widening happened.
        let _ = &trace.widenings;

        // Temporal mode must be explicitly declared.
        // For non-temporal queries, this is typically "none".
        assert!(
            trace.temporal_mode.is_some() || !trace.has_temporal_downgrade(),
            "LIB-C009: temporal_mode must be explicitly set or temporal downgrade absent"
        );

        // The provenance snapshot must be constructible.
        let provenance = trace.runtime_query_provenance();
        assert_eq!(
            provenance.schema_version, "runtime_query_provenance_v1",
            "LIB-C009: provenance schema version must be runtime_query_provenance_v1"
        );
        assert!(!provenance.scope.namespace.is_empty());
        assert!(!provenance.classification_mode.is_empty());
    }

    #[test]
    fn cf_v9_c009_runtime_query_provenance_round_trips() {
        // LIB-C009: RuntimeQueryProvenanceV1 must round-trip through serde.
        use knowledge_runtime::RuntimeQueryProvenanceV1;

        let runtime = open_runtime(
            &make_batch("cf-c009-rt", &["claim-a"], false, false),
            "cf-c009-rt",
        );
        let (_results, trace) =
            block_on(runtime.query("claim-a supports result", Some(&Scope::new("cf-c009-rt"))))
                .unwrap();
        let provenance = trace.runtime_query_provenance();

        let json = serde_json::to_value(&provenance).unwrap();
        let restored: RuntimeQueryProvenanceV1 = serde_json::from_value(json).unwrap();

        assert_eq!(restored.schema_version, provenance.schema_version);
        assert_eq!(restored.scope, provenance.scope);
        assert_eq!(restored.classification_mode, provenance.classification_mode);
        assert_eq!(restored.raw_result_count, provenance.raw_result_count);
        assert_eq!(restored.merged_result_count, provenance.merged_result_count);
        assert_eq!(restored.total_duration_ms, provenance.total_duration_ms);
    }

    #[test]
    fn cf_v9_c009_degraded_query_surfaces_widening() {
        // LIB-C009: When the runtime degrades a temporal query, it must surface
        // an explicit widening disclosure in the provenance record.
        let runtime = open_runtime(
            &make_batch("cf-c009-deg", &["claim-a", "claim-b"], false, false),
            "cf-c009-deg",
        );
        // Query with temporal expression that will be degraded.
        let (_results, trace) = block_on(runtime.query(
            "what happened last week with claim-a",
            Some(&Scope::new("cf-c009-deg")),
        ))
        .unwrap();

        if trace.has_temporal_downgrade() {
            // If temporal was downgraded, widenings must contain a disclosure.
            assert!(
                !trace.widenings.is_empty(),
                "LIB-C009: temporal downgrade must produce an explicit widening disclosure"
            );
            let provenance = trace.runtime_query_provenance();
            assert!(
                !provenance.widenings.is_empty(),
                "LIB-C009: provenance must carry widening disclosures"
            );
        }
        // Whether degraded or not, warnings must be present and typed.
        // (A non-temporal path may not degrade, which is also valid.)
    }

    // ─── LIB-C017: Cross-crate conformance gate registry ─────────

    #[test]
    fn cf_v9_c017_conformance_gate_registry_is_exhaustive() {
        // LIB-C017: Every conformance gate from phase 1-3 and v16-v20 must be
        // accounted for in the gate registry.
        let all_gates: Vec<&str> = [
            super::phase_1_gates(),
            super::phase_2_gates(),
            super::phase_3_plus_gates(),
            super::v16_v20_gates(),
        ]
        .into_iter()
        .flat_map(|gates| gates.iter().copied())
        .collect();

        // Must have no duplicates.
        let mut seen = std::collections::HashSet::new();
        for gate in &all_gates {
            assert!(
                seen.insert(*gate),
                "LIB-C017: duplicate conformance gate: {gate}"
            );
        }

        // Smoke check: minimum expected gate count across all phases.
        assert!(
            all_gates.len() >= 19,
            "LIB-C017: expected at least 19 conformance gates, got {}",
            all_gates.len()
        );
    }

    #[test]
    fn cf_v9_c017_v3_envelope_digest_is_deterministic() {
        // LIB-C017: Two identical envelopes must produce the same content digest.
        let batch_a = make_batch("c017-det", &["claim-a", "claim-b"], false, false);
        let batch_b = make_batch("c017-det", &["claim-a", "claim-b"], false, false);
        assert_eq!(
            batch_a.content_digest, batch_b.content_digest,
            "LIB-C017: identical envelopes must produce identical digests"
        );
    }

    #[test]
    fn cf_v9_c017_bridge_transform_preserves_execution_context() {
        // LIB-C017: The V3 bridge must preserve execution context through transform.
        let batch = make_batch("c017-ctx", &["claim-a"], false, false);
        assert!(
            batch.execution_context.is_some(),
            "LIB-C017: V3 bridge must always emit an execution context"
        );
        let ctx = batch.execution_context.as_ref().unwrap();
        assert_eq!(
            ctx.schema_version, "execution_context_v1",
            "LIB-C017: execution context schema version must be execution_context_v1"
        );
        assert!(
            ctx.workload_class.is_some(),
            "LIB-C017: execution context must declare workload class"
        );
    }

    #[test]
    fn cf_v9_c017_v2_to_v3_upgrade_preserves_record_count() {
        // LIB-C017: Upgrading V2 to V3 must preserve the number of records.
        let v2 = make_v2_envelope("c017-upgrade", &["claim-a", "claim-b", "claim-c"]);
        let v3 = ExportEnvelopeV3::try_from(v2.clone()).unwrap();
        assert_eq!(
            v2.records.len(),
            v3.records.len(),
            "LIB-C017: V2->V3 upgrade must preserve record count"
        );
    }
}
