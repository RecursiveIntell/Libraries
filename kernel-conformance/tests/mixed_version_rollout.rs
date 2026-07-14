use constraint_compiler::{compile_batch, CompilerPolicy};
use forge_memory_bridge::{transform_envelope_v3, ProjectionImportBatchV3};
use semantic_memory_forge::{
    ConstraintSeedKind, ExportAuthority, ExportClaim, ExportConfidenceClass, ExportEnvelopeV2,
    ExportEnvelopeV3, ExportRecord, ExportRecordSemanticsV3, ExportRecordV3, ForgeExportMeta,
    ProjectionVisibilityClass, EXPORT_ENVELOPE_V2_SCHEMA, EXPORT_ENVELOPE_V3_SCHEMA,
};
use stack_ids::{
    AssertionGroupId, ClaimFamilyId, ClaimId, ClaimVersionId, EntityId, EnvelopeId, ScopeKey,
};

fn sample_v2_envelope() -> ExportEnvelopeV2 {
    let scope = ScopeKey::namespace_only("mixed-version");
    let export_meta = ForgeExportMeta {
        authority: ExportAuthority::Forge,
        run_id: Some("mixed-v2".into()),
        direct_write: false,
        comparability_snapshot_version: None,
        exported_at: "2026-03-12T00:00:00Z".into(),
    };
    let records = vec![ExportRecord::Claim(ExportClaim {
        claim_id: Some(ClaimId::new("claim-1")),
        claim_version_id: Some(ClaimVersionId::new("claim-v1")),
        subject_entity_id: EntityId::new("entity-1"),
        predicate: "supports".into(),
        object_anchor: serde_json::json!("result"),
        valid_from: None,
        valid_to: None,
        confidence: 1.0,
        content: "claim-v1 supports result".into(),
        projection_family: "forge_verification".into(),
        supersedes_claim_id: None,
        supersedes_claim_version_id: None,
        metadata: None,
    })];
    let digest =
        ExportEnvelopeV2::compute_digest("forge", &scope, &records, Some(&export_meta), None)
            .unwrap();
    ExportEnvelopeV2 {
        envelope_id: EnvelopeId::new("env-v2"),
        schema_version: EXPORT_ENVELOPE_V2_SCHEMA.into(),
        content_digest: digest,
        source_authority: "forge".into(),
        scope_key: scope,
        trace_ctx: None,
        exported_at: "2026-03-12T00:00:00Z".into(),
        export_meta: Some(export_meta),
        evidence_bundle: None,
        records,
    }
}

fn sample_v3_envelope() -> ExportEnvelopeV3 {
    let scope = ScopeKey::namespace_only("mixed-version");
    let export_meta = ForgeExportMeta {
        authority: ExportAuthority::Forge,
        run_id: Some("mixed-v3".into()),
        direct_write: false,
        comparability_snapshot_version: None,
        exported_at: "2026-03-12T00:00:00Z".into(),
    };
    let records = vec![ExportRecordV3 {
        record: ExportRecord::Claim(ExportClaim {
            claim_id: Some(ClaimId::new("claim-1")),
            claim_version_id: Some(ClaimVersionId::new("claim-v1")),
            subject_entity_id: EntityId::new("entity-1"),
            predicate: "supports".into(),
            object_anchor: serde_json::json!("result"),
            valid_from: None,
            valid_to: None,
            confidence: 1.0,
            content: "claim-v1 supports result".into(),
            projection_family: "forge_verification".into(),
            supersedes_claim_id: None,
            supersedes_claim_version_id: None,
            metadata: None,
        }),
        semantics: Some(ExportRecordSemanticsV3 {
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
            comparability_snapshot_version: None,
            nuisance_snapshot: None,
            projection_visibility_class: ProjectionVisibilityClass::Standard,
            export_confidence_class: ExportConfidenceClass::Verified,
            derivation_seed_ids: vec![],
            review_priority_hint: None,
        }),
    }];
    let digest =
        ExportEnvelopeV3::compute_digest("forge", &scope, &records, Some(&export_meta), None)
            .unwrap();
    ExportEnvelopeV3 {
        envelope_id: EnvelopeId::new("env-v3"),
        schema_version: EXPORT_ENVELOPE_V3_SCHEMA.into(),
        content_digest: digest,
        source_authority: "forge".into(),
        scope_key: scope,
        trace_ctx: None,
        exported_at: "2026-03-12T00:00:00Z".into(),
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
    }
}

fn compile(batch: &ProjectionImportBatchV3) -> constraint_compiler::CompileOutput {
    compile_batch(
        batch,
        &CompilerPolicy {
            policy_version: "mixed-version".into(),
            include_hyperedges: true,
        },
    )
}

#[test]
fn v2_upgrade_and_native_v3_are_both_bridgeable_into_canonical_v3_lane() {
    let upgraded_v3 = ExportEnvelopeV3::try_from(sample_v2_envelope()).unwrap();
    let native_v3 = sample_v3_envelope();

    let upgraded_batch = transform_envelope_v3(&upgraded_v3).unwrap();
    let native_batch = transform_envelope_v3(&native_v3).unwrap();

    assert_eq!(
        upgraded_batch.schema_version,
        forge_memory_bridge::PROJECTION_IMPORT_BATCH_V3_SCHEMA
    );
    assert_eq!(
        native_batch.schema_version,
        forge_memory_bridge::PROJECTION_IMPORT_BATCH_V3_SCHEMA
    );
    assert_eq!(upgraded_batch.records.len(), native_batch.records.len());
}

#[test]
fn upgraded_v2_and_native_v3_produce_compatible_kernel_graphs() {
    let upgraded_v3 = ExportEnvelopeV3::try_from(sample_v2_envelope()).unwrap();
    let native_v3 = sample_v3_envelope();

    let upgraded_batch = transform_envelope_v3(&upgraded_v3).unwrap();
    let native_batch = transform_envelope_v3(&native_v3).unwrap();

    let upgraded_compiled = compile(&upgraded_batch);
    let native_compiled = compile(&native_batch);

    assert_eq!(upgraded_compiled.nodes.len(), native_compiled.nodes.len());
    assert!(upgraded_compiled.constraints.len() <= native_compiled.constraints.len());
    assert_eq!(native_compiled.oracle_candidates.len(), 1);
    assert!(upgraded_compiled.oracle_candidates.len() <= native_compiled.oracle_candidates.len());
    assert!(
        upgraded_compiled.degradations.len() >= native_compiled.degradations.len(),
        "upgraded v2 path may degrade more, but must remain explicit"
    );
    assert!(
        upgraded_compiled
            .degradations
            .contains(&constraint_compiler::ConstraintDegradation::ThinExport),
        "upgraded v2 compatibility path must disclose thin-export degradation"
    );
}
