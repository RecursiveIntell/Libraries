#![allow(clippy::expect_used)]

//! Cross-crate end-to-end proof suite.
//!
//! Proves the canonical normal path:
//!   Forge export → bridge transform → memory import → runtime query
//!
//! Also proves:
//! - Import rejection for malformed/incompatible batches
//! - Projection-backed temporal and scope semantics
//! - Evidence-ref import with explicit-only semantics
//! - Idempotent re-import
//! - Derivation edge correctness (claim_version target)
//!
//! The V3 export lane is the canonical normal path. Retained V1/V2 fixtures in
//! this file are compatibility coverage unless a test explicitly says otherwise.

#![allow(deprecated, unused_mut, clippy::too_many_arguments)]

use chrono::{Duration, Utc};
use forge_memory_bridge::{
    transform_envelope_v3, ImportClaimVersion, ImportProjectionRecord, ImportProjectionRecordV3,
    ProjectionImportBatchV3, PROJECTION_IMPORT_BATCH_V2_SCHEMA, PROJECTION_IMPORT_BATCH_V3_SCHEMA,
};
use knowledge_runtime::config::ProjectionConfig;
use knowledge_runtime::entity::registry::{Entity, EntityKind};
use knowledge_runtime::{
    KnowledgeRuntime, ProjectedPromotionState, ProjectedVerificationLifecycle, RuntimeConfig, Scope,
};
use semantic_memory::{MemoryConfig, MemoryStore, MockEmbedder, SearchConfig, SearchSource};
use semantic_memory_forge::{
    CausalRoleHint, ConstraintSeedKind, ExportClaim, ExportConfidenceClass, ExportEnvelopeV1,
    ExportEnvelopeV2, ExportEnvelopeV3, ExportEvidenceRef, ExportRecord, ExportRecordSemanticsV3,
    NuisanceSnapshot, ProjectionVisibilityClass, EXPORT_ENVELOPE_V1_SCHEMA,
    EXPORT_ENVELOPE_V3_SCHEMA,
};
use stack_ids::{
    AssertionGroupId, ClaimFamilyId, ClaimId, ClaimVersionId, ContentDigest, EntityId, EnvelopeId,
    EpisodeId, RelationVersionId, TraceCtx,
};
use tempfile::TempDir;

// ── Helpers ──────────────────────────────────────────────────────

fn open_store(dir: &TempDir) -> MemoryStore {
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

fn test_runtime_config() -> RuntimeConfig {
    RuntimeConfig {
        default_scope: Scope::new("test-ns"),
        query: Default::default(),
        entity: Default::default(),
        projection: ProjectionConfig {
            staleness_threshold_secs: 3600,
            import_staleness_threshold_secs: 0, // disable for most tests
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

fn make_claim_envelope(ns: &str) -> ExportEnvelopeV1 {
    make_claim_envelope_with_scope(
        stack_ids::ScopeKey::namespace_only(ns),
        "env-001",
        "claim-1",
        "ent-1",
        "Entity ent-1 is a function",
        Some("2026-01-01T00:00:00Z".into()),
        None,
    )
}

fn make_claim_envelope_v3(ns: &str) -> ExportEnvelopeV3 {
    ExportEnvelopeV3::try_from(ExportEnvelopeV2::from(make_claim_envelope(ns)))
        .expect("v3 enrichment should succeed for the bounded canonical proof fixture")
}

fn canonical_batch_from_v1(envelope: &ExportEnvelopeV1) -> ProjectionImportBatchV3 {
    let envelope_v3 = ExportEnvelopeV3::try_from(ExportEnvelopeV2::from(envelope.clone()))
        .expect("v3 enrichment should succeed for bounded canonical proof fixtures");
    transform_envelope_v3(&envelope_v3).expect("canonical V3 bridge transform must succeed")
}

fn make_claim_envelope_with_scope(
    scope: stack_ids::ScopeKey,
    envelope_id: &str,
    claim_id: &str,
    subject_entity_id: &str,
    content: &str,
    valid_from: Option<String>,
    valid_to: Option<String>,
) -> ExportEnvelopeV1 {
    let records = vec![ExportRecord::Claim(ExportClaim {
        claim_id: Some(ClaimId::new(claim_id)),
        claim_version_id: Some(ClaimVersionId::new(format!(
            "fixture-version-{envelope_id}"
        ))),
        subject_entity_id: EntityId::new(subject_entity_id),
        predicate: "has_type".into(),
        object_anchor: serde_json::json!("function"),
        valid_from,
        valid_to,
        confidence: 0.95,
        content: content.into(),
        projection_family: "forge_verification".into(),
        supersedes_claim_id: None,
        supersedes_claim_version_id: None,
        metadata: None,
    })];
    let digest = ExportEnvelopeV1::compute_digest("forge", &scope, &records).unwrap();

    ExportEnvelopeV1 {
        envelope_id: EnvelopeId::new(envelope_id),
        schema_version: EXPORT_ENVELOPE_V1_SCHEMA.into(),
        content_digest: digest,
        source_authority: "forge".into(),
        scope_key: scope,
        trace_ctx: Some(TraceCtx::generate()),
        exported_at: "2026-03-07T00:00:00Z".into(),
        records,
    }
}

fn make_claim_envelope_with_scope_and_metadata(
    scope: stack_ids::ScopeKey,
    envelope_id: &str,
    claim_id: &str,
    claim_version_id: &str,
    subject_entity_id: &str,
    content: &str,
    metadata: Option<serde_json::Value>,
    valid_from: Option<String>,
    valid_to: Option<String>,
) -> ExportEnvelopeV1 {
    let records = vec![ExportRecord::Claim(ExportClaim {
        claim_id: Some(ClaimId::new(claim_id)),
        claim_version_id: Some(ClaimVersionId::new(claim_version_id)),
        subject_entity_id: EntityId::new(subject_entity_id),
        predicate: "has_type".into(),
        object_anchor: serde_json::json!("function"),
        valid_from,
        valid_to,
        confidence: 0.95,
        content: content.into(),
        projection_family: "forge_verification".into(),
        supersedes_claim_id: None,
        supersedes_claim_version_id: None,
        metadata,
    })];
    let digest = ExportEnvelopeV1::compute_digest("forge", &scope, &records).unwrap();

    ExportEnvelopeV1 {
        envelope_id: EnvelopeId::new(envelope_id),
        schema_version: EXPORT_ENVELOPE_V1_SCHEMA.into(),
        content_digest: digest,
        source_authority: "forge".into(),
        scope_key: scope,
        trace_ctx: Some(TraceCtx::generate()),
        exported_at: "2026-03-07T00:00:00Z".into(),
        records,
    }
}

fn make_verification_summary_metadata(
    lifecycle_state: &str,
    promotion_state: serde_json::Value,
    completed_trial_count: u32,
    passed_refutation_count: u32,
    failed_refutation_count: u32,
    comparability_snapshot_version: Option<&str>,
    notes: &[&str],
) -> serde_json::Value {
    serde_json::json!({
        "verification_summary": {
            "lifecycle_state": lifecycle_state,
            "promotion_state": promotion_state,
            "completed_trial_count": completed_trial_count,
            "passed_refutation_count": passed_refutation_count,
            "failed_refutation_count": failed_refutation_count,
            "comparability_snapshot_version": comparability_snapshot_version,
            "notes": notes,
        }
    })
}

fn assert_contains_projection_kind(
    results: &[semantic_memory::SearchResult],
    projection_kind: &str,
) {
    assert!(
        results.iter().any(|result| matches!(
            &result.source,
            SearchSource::Projection { projection_kind: kind, .. } if kind == projection_kind
        )),
        "expected at least one {projection_kind} projection result, got {results:?}"
    );
}

fn make_claim_with_evidence_envelope(ns: &str) -> ExportEnvelopeV1 {
    let records = vec![
        ExportRecord::Claim(ExportClaim {
            claim_id: Some(ClaimId::new("claim-ev")),
            claim_version_id: Some(ClaimVersionId::new(format!("fixture-version-v{}", line!()))),
            subject_entity_id: EntityId::new("ent-ev"),
            predicate: "is_tested".into(),
            object_anchor: serde_json::json!(true),
            valid_from: None,
            valid_to: None,
            confidence: 0.9,
            content: "Entity ent-ev has test coverage".into(),
            projection_family: "forge_verification".into(),
            supersedes_claim_id: None,
            supersedes_claim_version_id: None,
            metadata: None,
        }),
        ExportRecord::EvidenceRef(ExportEvidenceRef {
            claim_id: ClaimId::new("claim-ev"),
            claim_version_id: Some(ClaimVersionId::new(format!("fixture-version-v{}", line!()))),
            fetch_handle: "forge://evidence/run-42/artifact-7".into(),
            source_authority: "forge".into(),
            metadata: None,
        }),
    ];
    let scope = stack_ids::ScopeKey::namespace_only(ns);
    let digest = ExportEnvelopeV1::compute_digest("forge", &scope, &records).unwrap();

    ExportEnvelopeV1 {
        envelope_id: EnvelopeId::new("env-with-evidence"),
        schema_version: EXPORT_ENVELOPE_V1_SCHEMA.into(),
        content_digest: digest,
        source_authority: "forge".into(),
        scope_key: scope,
        trace_ctx: Some(TraceCtx::generate()),
        exported_at: "2026-03-07T00:00:00Z".into(),
        records,
    }
}

fn make_kernel_v3_batch(ns: &str) -> ProjectionImportBatchV3 {
    ProjectionImportBatchV3 {
        source_envelope_id: EnvelopeId::new("env-runtime-kernel-v3"),
        schema_version: PROJECTION_IMPORT_BATCH_V3_SCHEMA.into(),
        export_schema_version: Some("export_envelope_v3".into()),
        content_digest: ContentDigest::compute(b"runtime-kernel-v3"),
        source_authority: "forge".into(),
        scope_key: stack_ids::ScopeKey::namespace_only(ns),
        trace_ctx: Some(TraceCtx::generate()),
        source_exported_at: "2026-03-10T00:00:00Z".into(),
        transformed_at: "2026-03-10T00:01:00Z".into(),
        export_meta: None,
        evidence_bundle: None,
        episode_bundle: None,
        execution_context: None,
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
        records: vec![ImportProjectionRecordV3 {
            record: ImportProjectionRecord::ClaimVersion(ImportClaimVersion {
                claim_id: ClaimId::new("claim-runtime-kernel-v3"),
                claim_version_id: ClaimVersionId::new("claim-version-runtime-kernel-v3"),
                claim_state: forge_memory_bridge::ClaimState::Active,
                projection_family: "forge_verification".into(),
                subject_entity_id: EntityId::new("entity-runtime-kernel-v3"),
                predicate: "supports".into(),
                object_anchor: serde_json::json!("kernel"),
                scope_key: stack_ids::ScopeKey::namespace_only(ns),
                valid_from: Some("2026-03-10T00:00:00Z".into()),
                valid_to: None,
                preferred_open: true,
                source_envelope_id: EnvelopeId::new("env-runtime-kernel-v3"),
                source_authority: "forge".into(),
                trace_ctx: None,
                freshness: forge_memory_bridge::ProjectionFreshness::Current,
                contradiction_status: forge_memory_bridge::ContradictionStatus::None,
                supersedes_claim_version_id: None,
                content: "runtime kernel claim".into(),
                confidence: 0.99,
                metadata: None,
            }),
            semantics: Some(ExportRecordSemanticsV3 {
                claim_family_id: Some(ClaimFamilyId::new("family-runtime-kernel-v3")),
                assertion_group_id: Some(AssertionGroupId::new("group-runtime-kernel-v3")),
                relation_group_id: None,
                joint_evidence_group_id: None,
                constraint_seed_kind: Some(ConstraintSeedKind::Hyperedge),
                treatment_hint: Some(CausalRoleHint::Treatment),
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
                derivation_seed_ids: vec!["seed-runtime-kernel-v3".into()],
                review_priority_hint: None,
            }),
        }],
    }
}

fn make_scheduler_degraded_kernel_v3_batch(ns: &str) -> ProjectionImportBatchV3 {
    let mut batch = make_kernel_v3_batch(ns);
    for index in 0..8 {
        let mut record = batch.records[0].clone();
        if let ImportProjectionRecord::ClaimVersion(claim) = &mut record.record {
            claim.claim_id = ClaimId::new(format!("claim-runtime-kernel-v3-extra-{index}"));
            claim.claim_version_id =
                ClaimVersionId::new(format!("claim-version-runtime-kernel-v3-extra-{index}"));
            claim.subject_entity_id =
                EntityId::new(format!("entity-runtime-kernel-v3-extra-{index}"));
            claim.source_envelope_id = EnvelopeId::new("env-runtime-kernel-v3-scheduler");
            claim.content = format!("runtime kernel extra claim {index}");
        }
        if let Some(semantics) = &mut record.semantics {
            semantics.claim_family_id = Some(ClaimFamilyId::new(format!(
                "family-runtime-kernel-v3-extra-{index}"
            )));
            semantics.assertion_group_id = Some(AssertionGroupId::new(format!(
                "group-runtime-kernel-v3-extra-{index}"
            )));
            semantics.derivation_seed_ids = vec![format!("seed-runtime-kernel-v3-extra-{index}")];
        }
        batch.records.push(record);
    }
    batch
}

fn make_causal_projection_envelope(ns: &str) -> ExportEnvelopeV1 {
    let records = vec![
        ExportRecord::Claim(ExportClaim {
            claim_id: Some(ClaimId::new("claim-causal")),
            claim_version_id: Some(ClaimVersionId::new(format!("fixture-version-v{}", line!()))),
            subject_entity_id: EntityId::new("ent-causal"),
            predicate: "changes_outcome".into(),
            object_anchor: serde_json::json!("verified"),
            valid_from: None,
            valid_to: None,
            confidence: 0.93,
            content: "Patch caused targeted behavior change".into(),
            projection_family: "forge_verification".into(),
            supersedes_claim_id: None,
            supersedes_claim_version_id: None,
            metadata: Some(serde_json::json!({
                "source": "causal_projection_test",
            })),
        }),
        ExportRecord::Episode(semantic_memory_forge::ExportEpisode {
            episode_id: Some(EpisodeId::new("ep-causal-query")),
            document_id: "doc-causal-query".into(),
            cause_ids: vec!["cause:claim-causal".into(), "run:causal-query".into()],
            effect_type: "causal_inference".into(),
            outcome: "verification_bundle".into(),
            confidence: 0.93,
            experiment_id: Some("exp-causal".into()),
            metadata: Some(serde_json::json!({
                "source": "runtime_causal_projection_test",
            })),
        }),
        ExportRecord::Relation(semantic_memory_forge::ExportRelation {
            relation_version_id: Some(RelationVersionId::new(format!(
                "relation-fixture-{}",
                line!()
            ))),
            subject_entity_id: EntityId::new("ent-causal"),
            predicate: "causes".into(),
            object_anchor: serde_json::json!({
                "effect": "behavioral_change",
                "severity": "high",
            }),
            valid_from: None,
            valid_to: None,
            confidence: 0.84,
            projection_family: "forge_verification".into(),
            source_claim_id: Some(ClaimId::new("claim-causal")),
            source_episode_id: Some(EpisodeId::new("ep-causal-query")),
            supersedes_relation_version_id: None,
            metadata: Some(serde_json::json!({
                "source": "runtime_causal_projection_test",
            })),
        }),
        ExportRecord::EvidenceRef(ExportEvidenceRef {
            claim_id: ClaimId::new("claim-causal"),
            claim_version_id: Some(ClaimVersionId::new(format!("fixture-version-v{}", line!()))),
            fetch_handle: "forge://evidence/run:causal-query/artifact-7".into(),
            source_authority: "forge".into(),
            metadata: Some(serde_json::json!({
                "case": "causal_query",
            })),
        }),
    ];

    let scope = stack_ids::ScopeKey::namespace_only(ns);
    let digest = ExportEnvelopeV1::compute_digest("forge", &scope, &records).unwrap();

    ExportEnvelopeV1 {
        envelope_id: EnvelopeId::new("env-causal-query"),
        schema_version: EXPORT_ENVELOPE_V1_SCHEMA.into(),
        content_digest: digest,
        source_authority: "forge".into(),
        scope_key: scope,
        trace_ctx: Some(TraceCtx::generate()),
        exported_at: "2026-03-08T00:00:00Z".into(),
        records,
    }
}

fn make_verification_projection_envelope(ns: &str) -> ExportEnvelopeV1 {
    let records = vec![
        ExportRecord::Claim(ExportClaim {
            claim_id: Some(ClaimId::new("claim-verification")),
            claim_version_id: Some(ClaimVersionId::new(format!("fixture-version-v{}", line!()))),
            subject_entity_id: EntityId::new("ent-verification"),
            predicate: "verification_bundle_has_trials".into(),
            object_anchor: serde_json::json!("verification trial evidence"),
            valid_from: None,
            valid_to: None,
            confidence: 0.96,
            content: "Verification bundle has paired trial records".into(),
            projection_family: "forge_verification".into(),
            supersedes_claim_id: None,
            supersedes_claim_version_id: None,
            metadata: Some(serde_json::json!({
                "bundle_id": "verification_bundle_1",
                "attempt_id": "attempt-1",
            })),
        }),
        ExportRecord::Relation(semantic_memory_forge::ExportRelation {
            relation_version_id: Some(RelationVersionId::new(format!(
                "relation-fixture-{}",
                line!()
            ))),
            subject_entity_id: EntityId::new("ent-verification"),
            predicate: "verification_trial_baseline".into(),
            object_anchor: serde_json::json!({
                "trial_id": "trial-baseline-1",
                "attempt_id": "attempt-1",
                "baseline_or_patch": "Baseline",
                "completed": true,
            }),
            valid_from: None,
            valid_to: None,
            confidence: 0.91,
            projection_family: "forge_verification".into(),
            source_claim_id: Some(ClaimId::new("claim-verification")),
            source_episode_id: None,
            supersedes_relation_version_id: None,
            metadata: Some(serde_json::json!({
                "bundle_id": "verification_bundle_1",
                "attempt_id": "attempt-1",
                "trial_id": "trial-baseline-1",
                "baseline_or_patch": "Baseline",
            })),
        }),
        ExportRecord::Relation(semantic_memory_forge::ExportRelation {
            relation_version_id: Some(RelationVersionId::new(format!(
                "relation-fixture-{}",
                line!()
            ))),
            subject_entity_id: EntityId::new("ent-verification"),
            predicate: "verification_trial_patched".into(),
            object_anchor: serde_json::json!({
                "trial_id": "trial-patched-1",
                "attempt_id": "attempt-1",
                "baseline_or_patch": "Patched",
                "completed": true,
            }),
            valid_from: None,
            valid_to: None,
            confidence: 0.93,
            projection_family: "forge_verification".into(),
            source_claim_id: Some(ClaimId::new("claim-verification")),
            source_episode_id: None,
            supersedes_relation_version_id: None,
            metadata: Some(serde_json::json!({
                "bundle_id": "verification_bundle_1",
                "attempt_id": "attempt-1",
                "trial_id": "trial-patched-1",
                "baseline_or_patch": "Patched",
            })),
        }),
        ExportRecord::Relation(semantic_memory_forge::ExportRelation {
            relation_version_id: Some(RelationVersionId::new(format!(
                "relation-fixture-{}",
                line!()
            ))),
            subject_entity_id: EntityId::new("ent-verification"),
            predicate: "verification_refutation_placebo".into(),
            object_anchor: serde_json::json!({
                "artifact_id": "ref-placebo-1",
                "artifact_type": "Placebo",
                "outcome": "passed",
                "attempt_id": "attempt-1",
                "trial_id": "trial-baseline-1",
            }),
            valid_from: None,
            valid_to: None,
            confidence: 0.97,
            projection_family: "forge_verification".into(),
            source_claim_id: Some(ClaimId::new("claim-verification")),
            source_episode_id: None,
            supersedes_relation_version_id: None,
            metadata: Some(serde_json::json!({
                "bundle_id": "verification_bundle_1",
                "artifact_id": "ref-placebo-1",
                "outcome": "passed",
            })),
        }),
        ExportRecord::Relation(semantic_memory_forge::ExportRelation {
            relation_version_id: Some(RelationVersionId::new(format!(
                "relation-fixture-{}",
                line!()
            ))),
            subject_entity_id: EntityId::new("ent-verification"),
            predicate: "verification_refutation_dummy_outcome".into(),
            object_anchor: serde_json::json!({
                "artifact_id": "ref-dummy-outcome-1",
                "artifact_type": "DummyOutcome",
                "outcome": "inconclusive",
                "attempt_id": "attempt-1",
                "trial_id": "trial-patched-1",
            }),
            valid_from: None,
            valid_to: None,
            confidence: 0.95,
            projection_family: "forge_verification".into(),
            source_claim_id: Some(ClaimId::new("claim-verification")),
            source_episode_id: None,
            supersedes_relation_version_id: None,
            metadata: Some(serde_json::json!({
                "bundle_id": "verification_bundle_1",
                "artifact_id": "ref-dummy-outcome-1",
                "outcome": "inconclusive",
            })),
        }),
        ExportRecord::Relation(semantic_memory_forge::ExportRelation {
            relation_version_id: Some(RelationVersionId::new(format!(
                "relation-fixture-{}",
                line!()
            ))),
            subject_entity_id: EntityId::new("ent-verification"),
            predicate: "verification_refutation_subsample_stability".into(),
            object_anchor: serde_json::json!({
                "artifact_id": "ref-subsample-1",
                "artifact_type": "SubsampleStability",
                "outcome": "failed",
                "attempt_id": "attempt-1",
                "trial_id": "trial-patched-1",
            }),
            valid_from: None,
            valid_to: None,
            confidence: 0.9,
            projection_family: "forge_verification".into(),
            source_claim_id: Some(ClaimId::new("claim-verification")),
            source_episode_id: None,
            supersedes_relation_version_id: None,
            metadata: Some(serde_json::json!({
                "bundle_id": "verification_bundle_1",
                "artifact_id": "ref-subsample-1",
                "outcome": "failed",
            })),
        }),
    ];

    let scope = stack_ids::ScopeKey::namespace_only(ns);
    let digest = ExportEnvelopeV1::compute_digest("forge", &scope, &records).unwrap();

    ExportEnvelopeV1 {
        envelope_id: EnvelopeId::new("env-verification-runtime"),
        schema_version: EXPORT_ENVELOPE_V1_SCHEMA.into(),
        content_digest: digest,
        source_authority: "forge".into(),
        scope_key: scope,
        trace_ctx: Some(TraceCtx::generate()),
        exported_at: "2026-03-09T00:00:00Z".into(),
        records,
    }
}

// ── 1. Canonical import path ─────────────────────────────────────

#[tokio::test]
async fn canonical_path_forge_export_bridge_transform_memory_import() {
    let dir = TempDir::new().unwrap();
    let store = open_store(&dir);

    // Step 1: Create the canonical V3 export envelope (simulating Forge output)
    let envelope = make_claim_envelope_v3("test-ns");
    let trace_id = envelope.trace_ctx.as_ref().unwrap().trace_id.clone();

    // Step 2: Bridge transform
    let batch = transform_envelope_v3(&envelope).unwrap();
    assert_eq!(batch.source_envelope_id, envelope.envelope_id);
    assert_eq!(batch.source_authority, "forge");
    assert!(!batch.transformed_at.is_empty());
    assert_eq!(batch.schema_version, PROJECTION_IMPORT_BATCH_V3_SCHEMA);
    assert_eq!(
        batch.export_schema_version.as_deref(),
        Some(EXPORT_ENVELOPE_V3_SCHEMA)
    );

    // Verify record was transformed correctly
    match &batch.records[0].record {
        ImportProjectionRecord::ClaimVersion(cv) => {
            assert_eq!(cv.claim_id.as_str(), "claim:claim-1");
            assert!(!cv.claim_version_id.is_empty());
            assert_eq!(cv.predicate, "has_type");
            // Trace context preserved through bridge
            assert_eq!(cv.trace_ctx.as_ref().unwrap().trace_id, trace_id);
        }
        _ => panic!("expected ClaimVersion"),
    }
    assert!(
        batch.records[0].semantics.is_none(),
        "thin canonical V3 exports must preserve absent semantics rather than invent them"
    );

    // Step 3: Import the canonical typed bridge batch into semantic-memory
    let result = store.import_projection_batch(&batch).await.unwrap();
    assert_eq!(result.source_envelope_id, "envelope:env-001");
    assert_eq!(result.status, "complete");
    assert_eq!(result.record_count, 1);
    assert!(!result.was_duplicate);

    let logs = store
        .query_projection_imports(Some("test-ns"), 10)
        .await
        .unwrap();
    let entry = logs
        .iter()
        .find(|entry| entry.source_envelope_id == "envelope:env-001")
        .unwrap();
    assert_eq!(entry.schema_version, PROJECTION_IMPORT_BATCH_V2_SCHEMA);
    assert_eq!(
        entry.export_schema_version.as_deref(),
        Some(EXPORT_ENVELOPE_V3_SCHEMA)
    );
}

// ── 2. Import rejection path ─────────────────────────────────────

#[tokio::test]
async fn import_rejects_unsupported_schema_version() {
    let dir = TempDir::new().unwrap();
    let store = open_store(&dir);

    let envelope = make_claim_envelope("test-ns");
    let batch = canonical_batch_from_v1(&envelope);
    let mut batch_json = serde_json::to_value(&batch).unwrap();
    batch_json["schema_version"] = serde_json::Value::String("unknown_v99".into());

    // Unsupported schema versions are rejected at the JSON import boundary.
    let err = store
        .import_projection_batch_json_compat(&serde_json::to_string(&batch_json).unwrap())
        .await
        .unwrap_err();
    let msg = format!("{err}");
    assert!(
        msg.contains("unsupported schema_version"),
        "expected version-law rejection, got: {msg}"
    );
}

#[tokio::test]
async fn import_rejects_malformed_json() {
    let dir = TempDir::new().unwrap();
    let store = open_store(&dir);

    let err = store
        .import_projection_batch_json_compat("{\"garbage\": true}")
        .await
        .unwrap_err();
    let msg = format!("{err}");
    assert!(
        msg.contains("invalid batch JSON"),
        "expected decode rejection, got: {msg}"
    );
}

// ── 3. Idempotent re-import ──────────────────────────────────────

#[tokio::test]
async fn idempotent_reimport_is_safe_noop() {
    let dir = TempDir::new().unwrap();
    let store = open_store(&dir);

    let envelope = make_claim_envelope("test-ns");
    let batch = canonical_batch_from_v1(&envelope);

    // First import
    let r1 = store.import_projection_batch(&batch).await.unwrap();
    assert!(!r1.was_duplicate);

    // Second import of same batch — must be idempotent no-op
    let r2 = store.import_projection_batch(&batch).await.unwrap();
    assert!(r2.was_duplicate);
    assert_eq!(r2.source_envelope_id, r1.source_envelope_id);
}

// ── 4. Evidence import with explicit-only semantics ──────────────

#[tokio::test]
async fn evidence_ref_imported_as_opaque_handle() {
    let dir = TempDir::new().unwrap();
    let store = open_store(&dir);

    let envelope = make_claim_with_evidence_envelope("test-ns");
    let batch = canonical_batch_from_v1(&envelope);

    // Verify evidence ref is present in batch with opaque handle
    let has_evidence = batch.records.iter().any(|r| {
        matches!(
            &r.record,
            ImportProjectionRecord::EvidenceRef(ev) if ev.fetch_handle == "forge://evidence/run-42/artifact-7"
        )
    });
    assert!(
        has_evidence,
        "evidence ref with opaque fetch handle must be in batch"
    );

    let result = store.import_projection_batch(&batch).await.unwrap();
    assert_eq!(result.record_count, 2); // claim + evidence ref
}

// ── I020: query_with_trace accepts caller-supplied TraceCtx ──────

#[tokio::test]
async fn query_with_trace_uses_caller_supplied_trace_ctx() {
    let dir = TempDir::new().unwrap();
    let store = open_store(&dir);

    let batch = canonical_batch_from_v1(&make_claim_envelope("test-ns"));
    store.import_projection_batch(&batch).await.unwrap();

    let adapter =
        knowledge_runtime::adapters::semantic_memory::SemanticMemoryAdapter::new(store.clone());
    let config = test_runtime_config();
    let runtime = KnowledgeRuntime::new(config, adapter).unwrap();

    let caller_trace = TraceCtx::from_trace_id("caller-supplied-trace-id");
    let scope = Scope::new("test-ns");
    let (_results, trace) = runtime
        .query_with_trace("function entity", Some(&scope), Some(caller_trace))
        .await
        .unwrap();

    // The trace must use the caller-supplied trace_id, NOT a fresh one
    assert_eq!(
        trace.trace_ctx.trace_id, "caller-supplied-trace-id",
        "query_with_trace must use the caller-supplied TraceCtx"
    );
}

#[tokio::test]
async fn query_with_trace_none_generates_fresh_trace_ctx() {
    let dir = TempDir::new().unwrap();
    let store = open_store(&dir);

    let batch = canonical_batch_from_v1(&make_claim_envelope("test-ns"));
    store.import_projection_batch(&batch).await.unwrap();

    let adapter =
        knowledge_runtime::adapters::semantic_memory::SemanticMemoryAdapter::new(store.clone());
    let config = test_runtime_config();
    let runtime = KnowledgeRuntime::new(config, adapter).unwrap();

    let scope = Scope::new("test-ns");
    let (_results, trace) = runtime
        .query_with_trace("function entity", Some(&scope), None)
        .await
        .unwrap();

    // With None, a fresh trace is generated — it must be non-empty
    assert!(
        !trace.trace_ctx.trace_id.is_empty(),
        "query_with_trace(None) must generate a fresh TraceCtx"
    );
}

#[tokio::test]
async fn query_convenience_wrapper_still_works() {
    let dir = TempDir::new().unwrap();
    let store = open_store(&dir);

    let batch = canonical_batch_from_v1(&make_claim_envelope("test-ns"));
    store.import_projection_batch(&batch).await.unwrap();

    let adapter =
        knowledge_runtime::adapters::semantic_memory::SemanticMemoryAdapter::new(store.clone());
    let config = test_runtime_config();
    let runtime = KnowledgeRuntime::new(config, adapter).unwrap();

    let scope = Scope::new("test-ns");
    // query() must still work as before (delegates to query_with_trace internally)
    let (_results, trace) = runtime
        .query("function entity", Some(&scope))
        .await
        .unwrap();
    assert!(!trace.trace_ctx.trace_id.is_empty());
}

// ── I023: persist=true is rejected at config validation ──────────

#[test]
fn persist_true_rejected_at_config_validation() {
    use knowledge_runtime::config::ProjectionConfig;

    let config = RuntimeConfig {
        default_scope: Scope::new("test-ns"),
        query: Default::default(),
        entity: Default::default(),
        projection: ProjectionConfig {
            staleness_threshold_secs: 3600,
            import_staleness_threshold_secs: 0,
            persist: true, // MUST be rejected
        },
        strict_temporal: false,
        strict_scope: false,
    };

    let dir = TempDir::new().unwrap();
    let store = open_store(&dir);
    let adapter =
        knowledge_runtime::adapters::semantic_memory::SemanticMemoryAdapter::new(store.clone());

    let result = KnowledgeRuntime::new(config, adapter);
    assert!(result.is_err(), "persist=true must be rejected");
    let err = result.unwrap_err();
    assert_eq!(err.kind(), "invalid_config");
    let msg = format!("{err}");
    assert!(
        msg.contains("persist"),
        "error message must mention persist: {msg}"
    );
}

// ── I027: entity_registry_mut removed ────────────────────────────

#[test]
fn entity_registry_mut_is_not_public() {
    // Structural proof: KnowledgeRuntime does not expose entity_registry_mut().
    // If someone re-adds it, this test documents the design intent.
    // The fact that this codebase compiles without any entity_registry_mut()
    // callers proves removal is safe.
    //
    // The non-authoritative cache is accessed via refresh_entity_cache() only.

    let dir = TempDir::new().unwrap();
    let store = open_store(&dir);
    let adapter =
        knowledge_runtime::adapters::semantic_memory::SemanticMemoryAdapter::new(store.clone());
    let config = test_runtime_config();
    let mut runtime = KnowledgeRuntime::new(config, adapter).unwrap();

    // Can access entity registry immutably
    let _reg = runtime.entity_registry();

    // Can access entity cache handle for non-authoritative mutation
    let handle = runtime.refresh_entity_cache();
    assert!(handle.is_empty());

    // entity_registry_mut() no longer exists — this is a compile-time guarantee.
    // If you uncomment the next line, it MUST fail to compile:
    // let _bad = runtime.entity_registry_mut();
}

// ── I021: ScopeNotFullyEnforced error variant exists ─────────────

#[test]
fn scope_not_fully_enforced_error_variant_exists() {
    use knowledge_runtime::RuntimeError;

    let err = RuntimeError::ScopeNotFullyEnforced {
        unpushed_dimensions: vec!["domain".into(), "repo_id".into()],
    };
    assert_eq!(err.kind(), "scope_not_fully_enforced");
    let msg = format!("{err}");
    assert!(msg.contains("domain"));
    assert!(msg.contains("repo_id"));
}

// ── 5. Runtime query after import ────────────────────────────────

#[tokio::test]
async fn runtime_query_sees_imported_data() {
    let dir = TempDir::new().unwrap();
    let store = open_store(&dir);

    // Import data through canonical path. No side-loaded fact/chunk/message rows
    // are added here; the runtime must answer directly from imported projections.
    let envelope = make_claim_envelope("test-ns");
    let batch = canonical_batch_from_v1(&envelope);
    store.import_projection_batch(&batch).await.unwrap();

    // Build runtime
    let adapter =
        knowledge_runtime::adapters::semantic_memory::SemanticMemoryAdapter::new(store.clone());
    let config = test_runtime_config();
    let runtime = KnowledgeRuntime::new(config, adapter).unwrap();

    // Query
    let scope = Scope::new("test-ns");
    let (results, trace) = runtime
        .query("function entity", Some(&scope))
        .await
        .unwrap();

    // Imported projections must now be the real retrieval substrate.
    assert!(!trace.trace_ctx.trace_id.is_empty());
    assert!(
        results
            .iter()
            .any(|result| result.content.contains("Entity ent-1 is a function")),
        "runtime must surface imported claim content directly"
    );
    assert_contains_projection_kind(&results, "claim_version");
}

#[tokio::test]
async fn kr104_runtime_query_consumes_imported_causal_episodes() {
    let dir = TempDir::new().unwrap();
    let store = open_store(&dir);

    let envelope = make_causal_projection_envelope("test-ns");
    let batch = canonical_batch_from_v1(&envelope);
    store.import_projection_batch(&batch).await.unwrap();

    let adapter =
        knowledge_runtime::adapters::semantic_memory::SemanticMemoryAdapter::new(store.clone());
    let config = test_runtime_config();
    let runtime = KnowledgeRuntime::new(config, adapter).unwrap();

    let scope = Scope::new("test-ns");
    let (results, trace) = runtime
        .query("verification_bundle", Some(&scope))
        .await
        .unwrap();

    assert!(!trace.trace_ctx.trace_id.is_empty());
    assert_contains_projection_kind(&results, "episode");

    assert!(
        results
            .iter()
            .any(|result| result.content.contains("doc-causal-query")),
        "causal episode content should be surfaced from imported rows"
    );

    for result in &results {
        if let SearchSource::Projection {
            projection_kind, ..
        } = &result.source
        {
            if projection_kind == "episode" {
                assert!(
                    !result.content.contains("forge://evidence/"),
                    "evidence handle must stay opaque in projection-facing results"
                );
            }
        }
    }
}

#[tokio::test]
async fn kr104_runtime_query_consumes_verification_relation_records() {
    let dir = TempDir::new().unwrap();
    let store = open_store(&dir);

    let envelope = make_verification_projection_envelope("test-ns");
    let batch = canonical_batch_from_v1(&envelope);
    store.import_projection_batch(&batch).await.unwrap();

    let adapter =
        knowledge_runtime::adapters::semantic_memory::SemanticMemoryAdapter::new(store.clone());
    let config = test_runtime_config();
    let runtime = KnowledgeRuntime::new(config, adapter).unwrap();

    let scope = Scope::new("test-ns");
    let (results, trace) = runtime.query("verification", Some(&scope)).await.unwrap();

    assert!(!trace.trace_ctx.trace_id.is_empty());
    assert_contains_projection_kind(&results, "relation_version");

    let relation_contents: Vec<&str> = results
        .iter()
        .filter_map(|result| {
            if let SearchSource::Projection {
                projection_kind, ..
            } = &result.source
            {
                if projection_kind == "relation_version" {
                    Some(result.content.as_str())
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect();

    assert!(
        relation_contents
            .iter()
            .any(|content| content.contains("verification_trial_baseline")),
        "baseline trial relation must be query-visible"
    );
    assert!(
        relation_contents
            .iter()
            .any(|content| content.contains("verification_trial_patched")),
        "patched trial relation must be query-visible"
    );
    assert!(
        relation_contents
            .iter()
            .any(|content| content.contains("verification_refutation_placebo")),
        "placebo refutation relation must be query-visible"
    );
    assert!(
        relation_contents
            .iter()
            .any(|content| content.contains("verification_refutation_dummy_outcome")),
        "dummy outcome refutation relation must be query-visible"
    );
    assert!(
        relation_contents
            .iter()
            .any(|content| content.contains("verification_refutation_subsample_stability")),
        "subsample stability refutation relation must be query-visible"
    );
}

// ── 6. Projection-backed temporal execution ──────────────────────

#[tokio::test]
async fn query_temporal_respects_recorded_at_cutoff_on_imported_projection_rows() {
    let dir = TempDir::new().unwrap();
    let store = open_store(&dir);

    let scope_key = stack_ids::ScopeKey::namespace_only("test-ns");

    let first_records = vec![ExportRecord::Claim(ExportClaim {
        claim_id: Some(ClaimId::new("claim-legacy")),
        claim_version_id: Some(ClaimVersionId::new(format!("fixture-version-v{}", line!()))),
        subject_entity_id: EntityId::new("ent-temporal-recorded"),
        predicate: "status".into(),
        object_anchor: serde_json::json!("archived"),
        valid_from: Some("2026-01-01T00:00:00Z".into()),
        valid_to: None,
        confidence: 0.8,
        content: "deployment status was baseline".into(),
        projection_family: "forge_verification".into(),
        supersedes_claim_id: None,
        supersedes_claim_version_id: None,
        metadata: None,
    })];

    let first_digest =
        ExportEnvelopeV1::compute_digest("forge", &scope_key, &first_records).unwrap();
    let first_envelope = ExportEnvelopeV1 {
        envelope_id: EnvelopeId::new("env-temporal-recorded-old"),
        schema_version: EXPORT_ENVELOPE_V1_SCHEMA.into(),
        content_digest: first_digest,
        source_authority: "forge".into(),
        scope_key: scope_key.clone(),
        trace_ctx: Some(TraceCtx::generate()),
        exported_at: "2026-03-07T00:00:00Z".into(),
        records: first_records,
    };
    let first_batch = canonical_batch_from_v1(&first_envelope);
    store.import_projection_batch(&first_batch).await.unwrap();

    tokio::time::sleep(std::time::Duration::from_secs(1)).await;

    let second_records = vec![ExportRecord::Claim(ExportClaim {
        claim_id: Some(ClaimId::new("claim-current")),
        claim_version_id: Some(ClaimVersionId::new(format!("fixture-version-v{}", line!()))),
        subject_entity_id: EntityId::new("ent-temporal-recorded"),
        predicate: "status".into(),
        object_anchor: serde_json::json!("current"),
        valid_from: Some("2026-02-01T00:00:00Z".into()),
        valid_to: None,
        confidence: 0.95,
        content: "deployment status was updated".into(),
        projection_family: "forge_verification".into(),
        supersedes_claim_id: None,
        supersedes_claim_version_id: None,
        metadata: None,
    })];

    let second_digest =
        ExportEnvelopeV1::compute_digest("forge", &scope_key, &second_records).unwrap();
    let second_envelope = ExportEnvelopeV1 {
        envelope_id: EnvelopeId::new("env-temporal-recorded-new"),
        schema_version: EXPORT_ENVELOPE_V1_SCHEMA.into(),
        content_digest: second_digest,
        source_authority: "forge".into(),
        scope_key,
        trace_ctx: Some(TraceCtx::generate()),
        exported_at: "2026-03-08T00:00:00Z".into(),
        records: second_records,
    };
    let second_batch = canonical_batch_from_v1(&second_envelope);
    store.import_projection_batch(&second_batch).await.unwrap();

    let import_log = store
        .query_projection_imports(Some("test-ns"), 10)
        .await
        .unwrap();
    assert!(import_log.len() >= 2, "expected two projection imports");
    let oldest_imported_at = import_log
        .iter()
        .map(|entry| entry.imported_at.clone())
        .min()
        .expect("at least one import log row");
    let latest_imported_at = import_log
        .iter()
        .map(|entry| entry.imported_at.clone())
        .max()
        .expect("at least one import log row");

    let adapter =
        knowledge_runtime::adapters::semantic_memory::SemanticMemoryAdapter::new(store.clone());
    let config = test_runtime_config();
    let runtime = KnowledgeRuntime::new(config, adapter).unwrap();

    let scope = Scope::new("test-ns");
    let valid_at = "2026-03-15T00:00:00Z";

    let (historical_results, historical_trace) = runtime
        .query_temporal(
            "what is the deployment status today?",
            Some(&scope),
            valid_at,
            &oldest_imported_at,
        )
        .await
        .unwrap();
    assert!(
        !historical_trace.has_temporal_downgrade(),
        "explicit recorded-time cutoff on projection path should not downgrade temporal semantics"
    );
    assert!(
        historical_results
            .iter()
            .any(|result| result.content.contains("baseline")),
        "earliest cutoff must include the first imported state"
    );
    assert!(
        !historical_results
            .iter()
            .any(|result| result.content.contains("updated")),
        "earliest cutoff must exclude later projection import"
    );

    let (current_results, _current_trace) = runtime
        .query_temporal(
            "what is the deployment status today?",
            Some(&scope),
            valid_at,
            &latest_imported_at,
        )
        .await
        .unwrap();
    assert!(
        current_results
            .iter()
            .any(|result| result.content.contains("baseline")),
        "latest cutoff should still show first state"
    );
    assert!(
        current_results
            .iter()
            .any(|result| result.content.contains("updated")),
        "latest cutoff must include the second imported state"
    );
}

#[tokio::test]
async fn query_temporal_explicitly_filters_hybrid_route_by_recorded_at() {
    let dir = TempDir::new().unwrap();
    let store = open_store(&dir);

    let scope_key = stack_ids::ScopeKey::namespace_only("test-ns");

    let first_records = vec![ExportRecord::Claim(ExportClaim {
        claim_id: Some(ClaimId::new("claim-hybrid-baseline")),
        claim_version_id: Some(ClaimVersionId::new(format!("fixture-version-v{}", line!()))),
        subject_entity_id: EntityId::new("ent-temporal-hybrid"),
        predicate: "status".into(),
        object_anchor: serde_json::json!("baseline"),
        valid_from: Some("2026-01-01T00:00:00Z".into()),
        valid_to: None,
        confidence: 0.8,
        content: "hybrid baseline status".into(),
        projection_family: "forge_verification".into(),
        supersedes_claim_id: None,
        supersedes_claim_version_id: None,
        metadata: None,
    })];

    let first_digest =
        ExportEnvelopeV1::compute_digest("forge", &scope_key, &first_records).unwrap();
    let first_envelope = ExportEnvelopeV1 {
        envelope_id: EnvelopeId::new("env-temporal-hybrid-old"),
        schema_version: EXPORT_ENVELOPE_V1_SCHEMA.into(),
        content_digest: first_digest,
        source_authority: "forge".into(),
        scope_key: scope_key.clone(),
        trace_ctx: Some(TraceCtx::generate()),
        exported_at: "2026-03-07T00:00:00Z".into(),
        records: first_records,
    };
    let first_batch = canonical_batch_from_v1(&first_envelope);
    store.import_projection_batch(&first_batch).await.unwrap();

    tokio::time::sleep(std::time::Duration::from_secs(1)).await;

    let second_records = vec![ExportRecord::Claim(ExportClaim {
        claim_id: Some(ClaimId::new("claim-hybrid-current")),
        claim_version_id: Some(ClaimVersionId::new(format!("fixture-version-v{}", line!()))),
        subject_entity_id: EntityId::new("ent-temporal-hybrid"),
        predicate: "status".into(),
        object_anchor: serde_json::json!("current"),
        valid_from: Some("2026-02-01T00:00:00Z".into()),
        valid_to: None,
        confidence: 0.9,
        content: "hybrid current status".into(),
        projection_family: "forge_verification".into(),
        supersedes_claim_id: None,
        supersedes_claim_version_id: None,
        metadata: None,
    })];

    let second_digest =
        ExportEnvelopeV1::compute_digest("forge", &scope_key, &second_records).unwrap();
    let second_envelope = ExportEnvelopeV1 {
        envelope_id: EnvelopeId::new("env-temporal-hybrid-new"),
        schema_version: EXPORT_ENVELOPE_V1_SCHEMA.into(),
        content_digest: second_digest,
        source_authority: "forge".into(),
        scope_key,
        trace_ctx: Some(TraceCtx::generate()),
        exported_at: "2026-03-08T00:00:00Z".into(),
        records: second_records,
    };
    let second_batch = canonical_batch_from_v1(&second_envelope);
    store.import_projection_batch(&second_batch).await.unwrap();

    let import_log = store
        .query_projection_imports(Some("test-ns"), 10)
        .await
        .unwrap();
    assert!(import_log.len() >= 2, "expected two projection imports");
    let oldest_imported_at = import_log
        .iter()
        .map(|entry| entry.imported_at.clone())
        .min()
        .expect("at least one import log row");
    let latest_imported_at = import_log
        .iter()
        .map(|entry| entry.imported_at.clone())
        .max()
        .expect("at least one import log row");

    let adapter =
        knowledge_runtime::adapters::semantic_memory::SemanticMemoryAdapter::new(store.clone());
    let config = test_runtime_config();
    let runtime = KnowledgeRuntime::new(config, adapter).unwrap();

    let scope = Scope::new("test-ns");
    let valid_at = "2026-03-15T00:00:00Z";

    let (historical_results, historical_trace) = runtime
        .query_temporal(
            "deployment status",
            Some(&scope),
            valid_at,
            &oldest_imported_at,
        )
        .await
        .unwrap();

    assert!(
        !historical_trace.has_temporal_downgrade(),
        "explicit bitemporal filters on hybrid projection routes should not downgrade"
    );
    assert!(
        historical_results
            .iter()
            .any(|result| result.content.contains("baseline")),
        "earliest cutoff must include first hybrid state"
    );
    assert!(
        !historical_results
            .iter()
            .any(|result| result.content.contains("current")),
        "earliest cutoff must exclude later hybrid state"
    );

    let (current_results, _current_trace) = runtime
        .query_temporal(
            "deployment status",
            Some(&scope),
            valid_at,
            &latest_imported_at,
        )
        .await
        .unwrap();
    assert!(
        current_results
            .iter()
            .any(|result| result.content.contains("baseline")),
        "latest cutoff should still show first hybrid state"
    );
    assert!(
        current_results
            .iter()
            .any(|result| result.content.contains("current")),
        "latest cutoff must include the second hybrid state"
    );
}

#[tokio::test]
async fn query_temporal_explicitly_filters_entity_route_by_recorded_at() {
    let dir = TempDir::new().unwrap();
    let store = open_store(&dir);

    let scope_key = stack_ids::ScopeKey::namespace_only("test-ns");
    let scope = Scope::new("test-ns");

    let first_records = vec![ExportRecord::Claim(ExportClaim {
        claim_id: Some(ClaimId::new("claim-entity-route-baseline")),
        claim_version_id: Some(ClaimVersionId::new(format!("fixture-version-v{}", line!()))),
        subject_entity_id: EntityId::new("ent-temporal-entity-route"),
        predicate: "status".into(),
        object_anchor: serde_json::json!("baseline"),
        valid_from: Some("2026-01-01T00:00:00Z".into()),
        valid_to: None,
        confidence: 0.82,
        content: "entity route baseline status".into(),
        projection_family: "forge_verification".into(),
        supersedes_claim_id: None,
        supersedes_claim_version_id: None,
        metadata: None,
    })];

    let first_digest =
        ExportEnvelopeV1::compute_digest("forge", &scope_key, &first_records).unwrap();
    let first_envelope = ExportEnvelopeV1 {
        envelope_id: EnvelopeId::new("env-entity-temporal-old"),
        schema_version: EXPORT_ENVELOPE_V1_SCHEMA.into(),
        content_digest: first_digest,
        source_authority: "forge".into(),
        scope_key: scope_key.clone(),
        trace_ctx: Some(TraceCtx::generate()),
        exported_at: "2026-03-07T00:00:00Z".into(),
        records: first_records,
    };
    let first_batch = canonical_batch_from_v1(&first_envelope);
    store.import_projection_batch(&first_batch).await.unwrap();

    tokio::time::sleep(std::time::Duration::from_secs(1)).await;

    let second_records = vec![ExportRecord::Claim(ExportClaim {
        claim_id: Some(ClaimId::new("claim-entity-route-current")),
        claim_version_id: Some(ClaimVersionId::new(format!("fixture-version-v{}", line!()))),
        subject_entity_id: EntityId::new("ent-temporal-entity-route"),
        predicate: "status".into(),
        object_anchor: serde_json::json!("current"),
        valid_from: Some("2026-02-01T00:00:00Z".into()),
        valid_to: None,
        confidence: 0.91,
        content: "entity route current status".into(),
        projection_family: "forge_verification".into(),
        supersedes_claim_id: None,
        supersedes_claim_version_id: None,
        metadata: None,
    })];

    let second_digest =
        ExportEnvelopeV1::compute_digest("forge", &scope_key, &second_records).unwrap();
    let second_envelope = ExportEnvelopeV1 {
        envelope_id: EnvelopeId::new("env-entity-temporal-new"),
        schema_version: EXPORT_ENVELOPE_V1_SCHEMA.into(),
        content_digest: second_digest,
        source_authority: "forge".into(),
        scope_key,
        trace_ctx: Some(TraceCtx::generate()),
        exported_at: "2026-03-08T00:00:00Z".into(),
        records: second_records,
    };
    let second_batch = canonical_batch_from_v1(&second_envelope);
    store.import_projection_batch(&second_batch).await.unwrap();

    let import_log = store
        .query_projection_imports(Some("test-ns"), 10)
        .await
        .unwrap();
    let oldest_imported_at = import_log
        .iter()
        .map(|entry| entry.imported_at.clone())
        .min()
        .expect("at least one import log row");
    let latest_imported_at = import_log
        .iter()
        .map(|entry| entry.imported_at.clone())
        .max()
        .expect("at least one import log row");

    let adapter =
        knowledge_runtime::adapters::semantic_memory::SemanticMemoryAdapter::new(store.clone());
    let mut runtime = KnowledgeRuntime::new(test_runtime_config(), adapter).unwrap();
    {
        let mut handle = runtime.refresh_entity_cache();
        handle
            .load_from_upstream(Entity {
                id: EntityId::new("ent-temporal-entity-route"),
                canonical_name: "ent-temporal-entity-route".to_string(),
                kind: EntityKind::Concept,
                scope: scope.key(),
                aliases: vec![],
                metadata: None,
            })
            .unwrap();
    }

    let valid_at = "2026-03-15T00:00:00Z";
    let (historical_results, historical_trace) = runtime
        .query_temporal(
            "\"ent-temporal-entity-route\" status",
            Some(&scope),
            valid_at,
            &oldest_imported_at,
        )
        .await
        .unwrap();
    assert!(
        !historical_trace.has_temporal_downgrade(),
        "explicit bitemporal filters on entity projection routes should not downgrade"
    );
    assert!(
        historical_results
            .iter()
            .any(|result| result.content.contains("baseline")),
        "earliest cutoff should include baseline entity state"
    );
    assert!(
        !historical_results
            .iter()
            .any(|result| result.content.contains("current")),
        "earliest cutoff should exclude later entity state"
    );

    let (current_results, _current_trace) = runtime
        .query_temporal(
            "\"ent-temporal-entity-route\" status",
            Some(&scope),
            valid_at,
            &latest_imported_at,
        )
        .await
        .unwrap();
    assert!(
        current_results
            .iter()
            .any(|result| result.content.contains("current")),
        "latest cutoff must include the newer entity state"
    );
}

#[tokio::test]
async fn query_temporal_explicitly_filters_mixed_route_by_recorded_at() {
    let dir = TempDir::new().unwrap();
    let store = open_store(&dir);

    let scope_key = stack_ids::ScopeKey::namespace_only("test-ns");
    let scope = Scope::new("test-ns");

    let first_records = vec![ExportRecord::Claim(ExportClaim {
        claim_id: Some(ClaimId::new("claim-mixed-baseline")),
        claim_version_id: Some(ClaimVersionId::new(format!("fixture-version-v{}", line!()))),
        subject_entity_id: EntityId::new("ent-temporal-mixed-route"),
        predicate: "status".into(),
        object_anchor: serde_json::json!("baseline"),
        valid_from: Some("2026-01-01T00:00:00Z".into()),
        valid_to: None,
        confidence: 0.84,
        content: "ent-temporal-mixed-route baseline status".into(),
        projection_family: "forge_verification".into(),
        supersedes_claim_id: None,
        supersedes_claim_version_id: None,
        metadata: None,
    })];

    let first_digest =
        ExportEnvelopeV1::compute_digest("forge", &scope_key, &first_records).unwrap();
    let first_envelope = ExportEnvelopeV1 {
        envelope_id: EnvelopeId::new("env-mixed-temporal-old"),
        schema_version: EXPORT_ENVELOPE_V1_SCHEMA.into(),
        content_digest: first_digest,
        source_authority: "forge".into(),
        scope_key: scope_key.clone(),
        trace_ctx: Some(TraceCtx::generate()),
        exported_at: "2026-03-07T00:00:00Z".into(),
        records: first_records,
    };
    let first_batch = canonical_batch_from_v1(&first_envelope);
    store.import_projection_batch(&first_batch).await.unwrap();

    tokio::time::sleep(std::time::Duration::from_secs(1)).await;

    let second_records = vec![ExportRecord::Claim(ExportClaim {
        claim_id: Some(ClaimId::new("claim-mixed-current")),
        claim_version_id: Some(ClaimVersionId::new(format!("fixture-version-v{}", line!()))),
        subject_entity_id: EntityId::new("ent-temporal-mixed-route"),
        predicate: "status".into(),
        object_anchor: serde_json::json!("current"),
        valid_from: Some("2026-02-01T00:00:00Z".into()),
        valid_to: None,
        confidence: 0.91,
        content: "ent-temporal-mixed-route current status".into(),
        projection_family: "forge_verification".into(),
        supersedes_claim_id: None,
        supersedes_claim_version_id: None,
        metadata: None,
    })];

    let second_digest =
        ExportEnvelopeV1::compute_digest("forge", &scope_key, &second_records).unwrap();
    let second_envelope = ExportEnvelopeV1 {
        envelope_id: EnvelopeId::new("env-mixed-temporal-new"),
        schema_version: EXPORT_ENVELOPE_V1_SCHEMA.into(),
        content_digest: second_digest,
        source_authority: "forge".into(),
        scope_key,
        trace_ctx: Some(TraceCtx::generate()),
        exported_at: "2026-03-08T00:00:00Z".into(),
        records: second_records,
    };
    let second_batch = canonical_batch_from_v1(&second_envelope);
    store.import_projection_batch(&second_batch).await.unwrap();

    let import_log = store
        .query_projection_imports(Some("test-ns"), 10)
        .await
        .unwrap();
    assert!(import_log.len() >= 2, "expected two projection imports");
    let oldest_imported_at = import_log
        .iter()
        .map(|entry| entry.imported_at.clone())
        .min()
        .expect("at least one import log row");
    let latest_imported_at = import_log
        .iter()
        .map(|entry| entry.imported_at.clone())
        .max()
        .expect("at least one import log row");

    let adapter =
        knowledge_runtime::adapters::semantic_memory::SemanticMemoryAdapter::new(store.clone());
    let mut runtime = KnowledgeRuntime::new(test_runtime_config(), adapter).unwrap();
    {
        let mut handle = runtime.refresh_entity_cache();
        handle
            .load_from_upstream(Entity {
                id: EntityId::new("ent-temporal-mixed-route"),
                canonical_name: "ent-temporal-mixed-route".to_string(),
                kind: EntityKind::Concept,
                scope: scope.key(),
                aliases: vec![],
                metadata: None,
            })
            .unwrap();
    }

    let valid_at = "2026-03-15T00:00:00Z";
    let (historical_results, historical_trace) = runtime
        .query_temporal(
            "\"ent-temporal-mixed-route\" status yesterday",
            Some(&scope),
            valid_at,
            &oldest_imported_at,
        )
        .await
        .unwrap();
    assert!(
        !historical_trace.has_temporal_downgrade(),
        "explicit bitemporal filters on mixed routes should not downgrade"
    );
    assert!(
        historical_results
            .iter()
            .any(|result| result.content.contains("baseline")),
        "mixed explicit bitemporal query should include baseline with historical cutoff"
    );
    assert!(
        !historical_results
            .iter()
            .any(|result| result.content.contains("current")),
        "mixed explicit bitemporal query should exclude later imported state with historical cutoff"
    );

    let (current_results, _current_trace) = runtime
        .query_temporal(
            "\"ent-temporal-mixed-route\" status yesterday",
            Some(&scope),
            valid_at,
            &latest_imported_at,
        )
        .await
        .unwrap();
    assert!(
        current_results
            .iter()
            .any(|result| result.content.contains("baseline")),
        "mixed explicit bitemporal query should include first imported state at latest cutoff"
    );
    assert!(
        current_results
            .iter()
            .any(|result| result.content.contains("current")),
        "mixed explicit bitemporal query should include second imported state at latest cutoff"
    );
}

#[tokio::test]
async fn temporal_query_answers_from_imported_projection_rows() {
    let dir = TempDir::new().unwrap();
    let store = open_store(&dir);

    let now = Utc::now();
    let records = vec![
        ExportRecord::Claim(ExportClaim {
            claim_id: Some(ClaimId::new("claim-old")),
            claim_version_id: Some(ClaimVersionId::new(format!("fixture-version-v{}", line!()))),
            subject_entity_id: EntityId::new("ent-temporal"),
            predicate: "status".into(),
            object_anchor: serde_json::json!("archived"),
            valid_from: Some((now - Duration::days(7)).to_rfc3339()),
            valid_to: Some((now - Duration::days(2)).to_rfc3339()),
            confidence: 0.8,
            content: "deployment status was archived".into(),
            projection_family: "forge_verification".into(),
            supersedes_claim_id: None,
            supersedes_claim_version_id: None,
            metadata: None,
        }),
        ExportRecord::Claim(ExportClaim {
            claim_id: Some(ClaimId::new("claim-current")),
            claim_version_id: Some(ClaimVersionId::new(format!("fixture-version-v{}", line!()))),
            subject_entity_id: EntityId::new("ent-temporal"),
            predicate: "status".into(),
            object_anchor: serde_json::json!("current"),
            valid_from: Some((now - Duration::hours(6)).to_rfc3339()),
            valid_to: None,
            confidence: 0.95,
            content: "deployment status is current today".into(),
            projection_family: "forge_verification".into(),
            supersedes_claim_id: None,
            supersedes_claim_version_id: None,
            metadata: None,
        }),
    ];
    let scope_key = stack_ids::ScopeKey::namespace_only("test-ns");
    let digest = ExportEnvelopeV1::compute_digest("forge", &scope_key, &records).unwrap();
    let envelope = ExportEnvelopeV1 {
        envelope_id: EnvelopeId::new("env-temporal"),
        schema_version: EXPORT_ENVELOPE_V1_SCHEMA.into(),
        content_digest: digest,
        source_authority: "forge".into(),
        scope_key,
        trace_ctx: Some(TraceCtx::generate()),
        exported_at: now.to_rfc3339(),
        records,
    };
    let batch = canonical_batch_from_v1(&envelope);
    store.import_projection_batch(&batch).await.unwrap();

    let adapter =
        knowledge_runtime::adapters::semantic_memory::SemanticMemoryAdapter::new(store.clone());
    let config = test_runtime_config();
    let runtime = KnowledgeRuntime::new(config, adapter).unwrap();

    let scope = Scope::new("test-ns");
    let (_results, trace) = runtime
        .query("what is the deployment status today", Some(&scope))
        .await
        .unwrap();

    let (results, _trace) = runtime
        .query("what is the deployment status today", Some(&scope))
        .await
        .unwrap();
    assert!(
        !trace.has_temporal_downgrade(),
        "projection-backed temporal route must not degrade to hybrid"
    );
    assert!(
        results
            .iter()
            .any(|result| result.content.contains("current today")),
        "temporal route must return the currently valid imported claim"
    );
    assert!(
        results
            .iter()
            .all(|result| !result.content.contains("archived")),
        "temporal route must filter out imported claims outside the valid interval"
    );
    assert_contains_projection_kind(&results, "claim_version");
}

// ── 7. Scope enforcement transparency ────────────────────────────

#[tokio::test]
async fn scope_with_extra_dimensions_emits_warning() {
    let dir = TempDir::new().unwrap();
    let store = open_store(&dir);

    let scope = Scope::new("test-ns")
        .with_domain("code")
        .with_workspace("ws-1");
    ingest_scoped_document(
        &store,
        &scope,
        "Scoped search doc",
        "test fact for scope lives in code ws-1 document scope",
    )
    .await;

    let adapter =
        knowledge_runtime::adapters::semantic_memory::SemanticMemoryAdapter::new(store.clone());
    let config = test_runtime_config();
    let runtime = KnowledgeRuntime::new(config, adapter).unwrap();

    let (results, trace) = runtime
        .query("code ws-1 document scope", Some(&scope))
        .await
        .unwrap();

    assert!(
        !trace.has_scope_enforcement_warning(),
        "scoped hybrid query with document scope evidence must not emit ScopePartiallyEnforced"
    );
    assert!(
        results
            .iter()
            .any(|result| result.content.contains("code ws-1 document scope")),
        "scoped hybrid query must return the matching scoped document"
    );
}

#[tokio::test]
async fn query_temporal_explicit_temporal_enforces_full_scope_on_projection_routes() {
    let dir = TempDir::new().unwrap();
    let store = open_store(&dir);

    let matching_scope = Scope::new("test-ns")
        .with_domain("code")
        .with_workspace("ws-1")
        .with_repo("repo-1");
    let non_matching_scope = Scope::new("test-ns")
        .with_domain("docs")
        .with_workspace("ws-1")
        .with_repo("repo-2");

    let matching_envelope = make_claim_envelope_with_scope(
        matching_scope.key(),
        "env-temporal-scope-hit",
        "claim-temporal-scope-hit",
        "ent-temporal-scope-hit",
        "scoped temporal projection result",
        Some("2026-01-01T00:00:00Z".into()),
        None,
    );
    let non_matching_envelope = make_claim_envelope_with_scope(
        non_matching_scope.key(),
        "env-temporal-scope-miss",
        "claim-temporal-scope-miss",
        "ent-temporal-scope-miss",
        "scoped temporal projection from another scope",
        Some("2026-01-01T00:00:00Z".into()),
        None,
    );

    for envelope in [matching_envelope, non_matching_envelope] {
        let batch = canonical_batch_from_v1(&envelope);
        store.import_projection_batch(&batch).await.unwrap();
    }

    let adapter =
        knowledge_runtime::adapters::semantic_memory::SemanticMemoryAdapter::new(store.clone());
    let config = RuntimeConfig {
        default_scope: matching_scope.clone(),
        strict_scope: false,
        ..test_runtime_config()
    };
    let runtime = KnowledgeRuntime::new(config, adapter).unwrap();

    let recorded_cutoff = runtime
        .adapter()
        .last_import_at("test-ns")
        .await
        .unwrap()
        .expect("import timestamp should exist");

    let (results, trace) = runtime
        .query_temporal(
            "scoped temporal projection",
            Some(&matching_scope),
            "2026-01-01T12:00:00Z",
            &recorded_cutoff,
        )
        .await
        .unwrap();

    assert!(
        !trace.has_scope_enforcement_warning(),
        "projection-backed explicit temporal query must enforce full scope without partial-scope warning"
    );
    assert!(
        results
            .iter()
            .any(|result| result.content == "scoped temporal projection result"),
        "explicit temporal query must return the matching imported row"
    );
    assert!(
        results
            .iter()
            .all(|result| !result.content.contains("another scope")),
        "explicit temporal query must exclude rows from non-matching scope dimensions"
    );
    assert_contains_projection_kind(&results, "claim_version");
}

#[tokio::test]
async fn query_temporal_strict_scope_enforces_full_scope_without_fallback() {
    let dir = TempDir::new().unwrap();
    let store = open_store(&dir);

    let matching_scope = Scope::new("test-ns")
        .with_domain("code")
        .with_workspace("ws-1")
        .with_repo("repo-1");
    let non_matching_scope = Scope::new("test-ns")
        .with_domain("docs")
        .with_workspace("ws-1")
        .with_repo("repo-2");

    let matching_envelope = make_claim_envelope_with_scope(
        matching_scope.key(),
        "env-temporal-scope-hit-strict",
        "claim-temporal-scope-hit-strict",
        "ent-temporal-scope-hit-strict",
        "strict scoped temporal projection result",
        Some("2026-01-01T00:00:00Z".into()),
        None,
    );
    let non_matching_envelope = make_claim_envelope_with_scope(
        non_matching_scope.key(),
        "env-temporal-scope-miss-strict",
        "claim-temporal-scope-miss-strict",
        "ent-temporal-scope-miss-strict",
        "scoped temporal projection from another scope",
        Some("2026-01-01T00:00:00Z".into()),
        None,
    );

    for envelope in [matching_envelope, non_matching_envelope] {
        let batch = canonical_batch_from_v1(&envelope);
        store.import_projection_batch(&batch).await.unwrap();
    }

    let adapter =
        knowledge_runtime::adapters::semantic_memory::SemanticMemoryAdapter::new(store.clone());
    let config = RuntimeConfig {
        default_scope: matching_scope.clone(),
        strict_scope: true,
        strict_temporal: true,
        ..test_runtime_config()
    };
    let runtime = KnowledgeRuntime::new(config, adapter).unwrap();

    let recorded_cutoff = runtime
        .adapter()
        .last_import_at("test-ns")
        .await
        .unwrap()
        .expect("import timestamp should exist");

    let (results, trace) = runtime
        .query_temporal(
            "scoped temporal projection",
            Some(&matching_scope),
            "2026-01-01T12:00:00Z",
            &recorded_cutoff,
        )
        .await
        .unwrap();

    assert!(
        !trace.has_scope_enforcement_warning(),
        "strict scope must still enforce full scope without fallback on projection temporal routes"
    );
    assert!(
        !trace.has_temporal_downgrade(),
        "strict temporal route with explicit temporal filters should not downgrade"
    );
    assert!(
        results
            .iter()
            .any(|result| result.content == "strict scoped temporal projection result"),
        "strict-temporal query must return the matching imported row"
    );
    assert!(
        results
            .iter()
            .all(|result| !result.content.contains("another scope")),
        "strict-temporal query must exclude rows from non-matching scope dimensions"
    );
    assert_contains_projection_kind(&results, "claim_version");
}

// ── 8. Bridge rejects tampered export ────────────────────────────

#[test]
fn bridge_rejects_tampered_digest() {
    let mut envelope = make_claim_envelope("test-ns");
    let mut envelope = ExportEnvelopeV2::from(envelope);
    let mut envelope = ExportEnvelopeV3::try_from(envelope)
        .expect("canonical V3 conversion must succeed for mismatch fixture");
    envelope.content_digest = stack_ids::ContentDigest::compute(b"tampered");

    let err = transform_envelope_v3(&envelope).unwrap_err();
    assert!(
        matches!(err, forge_memory_bridge::BridgeError::DigestMismatch { .. }),
        "bridge must reject tampered envelope digest"
    );
}

#[test]
fn bridge_rejects_incompatible_export_version() {
    let mut envelope = make_claim_envelope("test-ns");
    let envelope_v2 = ExportEnvelopeV2::from(envelope);
    let mut envelope = ExportEnvelopeV3::try_from(envelope_v2)
        .expect("canonical V3 conversion must succeed for mismatch fixture");
    envelope.schema_version = "export_envelope_v99".into();

    let err = transform_envelope_v3(&envelope).unwrap_err();
    assert!(
        matches!(
            err,
            forge_memory_bridge::BridgeError::IncompatibleVersion { .. }
        ),
        "bridge must reject incompatible export version"
    );
}

// ── 9. Trace context preserved end-to-end ────────────────────────

#[tokio::test]
async fn trace_context_preserved_through_full_path() {
    let dir = TempDir::new().unwrap();
    let store = open_store(&dir);

    let envelope = make_claim_envelope("test-ns");
    let source_trace_id = envelope.trace_ctx.as_ref().unwrap().trace_id.clone();

    // Bridge preserves trace
    let batch = canonical_batch_from_v1(&envelope);
    assert_eq!(
        batch.trace_ctx.as_ref().unwrap().trace_id,
        source_trace_id,
        "bridge must preserve source trace_id"
    );

    // Import preserves provenance (envelope_id carries through)
    let result = store.import_projection_batch(&batch).await.unwrap();
    assert_eq!(result.source_envelope_id, "envelope:env-001");
}

// ── KR-003: Rebuild driver trait proof ────────────────────────────

#[tokio::test]
async fn kr003_rebuild_driver_trait_drives_stale_projection_rebuild() {
    use knowledge_runtime::projection::lifecycle::{
        InvalidationEvent, ProjectionVersion, StaleCause,
    };
    use knowledge_runtime::projection::rebuild::{rebuild_stale, RebuildDriver, RebuildOutcome};
    use knowledge_runtime::{ProjectionHealth, ProjectionKind, ProjectionTracker, RuntimeError};

    // Create a mock rebuild driver
    struct MockDriver;
    impl RebuildDriver for MockDriver {
        async fn rebuild(
            &self,
            _id: &knowledge_runtime::ProjectionId,
        ) -> Result<RebuildOutcome, RuntimeError> {
            Ok(RebuildOutcome {
                source_count: 42,
                build_duration_ms: 100,
                version: Some(ProjectionVersion {
                    schema_version: Some("v2".into()),
                    resolver_version: Some("r1".into()),
                }),
            })
        }
        fn can_rebuild(&self, _id: &knowledge_runtime::ProjectionId) -> bool {
            true
        }
    }

    let mut tracker = ProjectionTracker::new(3600);
    let id = knowledge_runtime::ProjectionId {
        kind: ProjectionKind::Entity,
        key: "test-entity".into(),
        scope: knowledge_runtime::ScopeKey::namespace_only("test"),
    };

    // Build, then invalidate to make stale
    tracker.record_build(id.clone(), 10, 50, None);
    assert_eq!(tracker.health(&id), ProjectionHealth::Healthy);

    let event = InvalidationEvent {
        projection_ids: vec![id.clone()],
        cause: StaleCause::SourceChanged,
        at: chrono::Utc::now(),
    };
    tracker.invalidate(&event);
    assert_eq!(tracker.health(&id), ProjectionHealth::Stale);

    // Drive rebuild via trait
    let driver = MockDriver;
    let rebuilt = rebuild_stale(&mut tracker, &driver).await.unwrap();
    assert_eq!(rebuilt, 1);

    // Tracker state transitions back to Healthy
    assert_eq!(tracker.health(&id), ProjectionHealth::Healthy);
}

#[tokio::test]
async fn kr003_rebuild_driver_can_decline_projection() {
    use knowledge_runtime::projection::lifecycle::{InvalidationEvent, StaleCause};
    use knowledge_runtime::projection::rebuild::{rebuild_stale, RebuildDriver, RebuildOutcome};
    use knowledge_runtime::{ProjectionHealth, ProjectionKind, ProjectionTracker, RuntimeError};

    struct DeclineAllDriver;
    impl RebuildDriver for DeclineAllDriver {
        async fn rebuild(
            &self,
            _id: &knowledge_runtime::ProjectionId,
        ) -> Result<RebuildOutcome, RuntimeError> {
            unreachable!("should not be called when can_rebuild returns false");
        }
        fn can_rebuild(&self, _id: &knowledge_runtime::ProjectionId) -> bool {
            false
        }
    }

    let mut tracker = ProjectionTracker::new(3600);
    let id = knowledge_runtime::ProjectionId {
        kind: ProjectionKind::Entity,
        key: "unsupported".into(),
        scope: knowledge_runtime::ScopeKey::namespace_only("test"),
    };

    tracker.record_build(id.clone(), 5, 25, None);
    tracker.invalidate(&InvalidationEvent {
        projection_ids: vec![id.clone()],
        cause: StaleCause::ExplicitInvalidation {
            reason: "test".into(),
        },
        at: chrono::Utc::now(),
    });

    let driver = DeclineAllDriver;
    let rebuilt = rebuild_stale(&mut tracker, &driver).await.unwrap();
    assert_eq!(rebuilt, 0, "driver declined all projections");
    assert_eq!(
        tracker.health(&id),
        ProjectionHealth::Stale,
        "declined projection remains stale"
    );
}

// ── KR-005: Expanded cross-crate proof tests ─────────────────────

#[tokio::test]
async fn kr005_strict_scope_accepts_fully_enforced_domain_in_cross_crate_path() {
    // Prove that strict_scope mode accepts domain/repo scope once the runtime
    // can fully enforce scoped hybrid search against document metadata.
    let dir = TempDir::new().unwrap();
    let store = open_store(&dir);
    let scope = Scope::new("test-ns")
        .with_domain("code")
        .with_repo("repo-1");
    ingest_scoped_document(
        &store,
        &scope,
        "Strict scoped cross-crate doc",
        "fact for scope proof in repo-1 code scope",
    )
    .await;

    let adapter =
        knowledge_runtime::adapters::semantic_memory::SemanticMemoryAdapter::new(store.clone());
    let config = RuntimeConfig {
        default_scope: Scope::new("test-ns"),
        strict_scope: true,
        strict_temporal: false,
        ..test_runtime_config()
    };
    let runtime = KnowledgeRuntime::new(config, adapter).unwrap();
    let (results, trace) = runtime
        .query("repo-1 code scope proof", Some(&scope))
        .await
        .expect("strict_scope must accept fully enforceable scoped hybrid search");
    assert!(
        !trace.has_scope_enforcement_warning(),
        "strict_scope must not warn when scope is fully enforced"
    );
    assert!(
        results
            .iter()
            .any(|result| result.content.contains("repo-1 code scope")),
        "strict scoped cross-crate query must return the matching document"
    );
}

#[tokio::test]
async fn kr005_strict_temporal_succeeds_on_projection_backed_route() {
    // Prove strict temporal mode succeeds when imported projections provide
    // a real temporal substrate.
    let dir = TempDir::new().unwrap();
    let store = open_store(&dir);

    let now = Utc::now();
    let envelope = make_claim_envelope_with_scope(
        stack_ids::ScopeKey::namespace_only("test-ns"),
        "env-temporal-strict",
        "claim-strict-temporal",
        "ent-strict-temporal",
        "deployment status is current today",
        Some((now - Duration::hours(1)).to_rfc3339()),
        None,
    );
    let batch = canonical_batch_from_v1(&envelope);
    store.import_projection_batch(&batch).await.unwrap();

    let adapter =
        knowledge_runtime::adapters::semantic_memory::SemanticMemoryAdapter::new(store.clone());
    let config = RuntimeConfig {
        strict_temporal: true,
        ..test_runtime_config()
    };
    let runtime = KnowledgeRuntime::new(config, adapter).unwrap();

    let scope = Scope::new("test-ns");
    let (results, trace) = runtime
        .query("what is the deployment status today", Some(&scope))
        .await
        .unwrap();

    assert!(
        !trace.has_temporal_downgrade(),
        "strict temporal mode must not degrade when projection-backed temporal execution exists"
    );
    assert!(
        results
            .iter()
            .any(|result| result.content.contains("current today")),
        "strict temporal query must surface imported projection results"
    );
    assert_contains_projection_kind(&results, "claim_version");
}

#[tokio::test]
async fn kr005_evidence_remains_opaque_in_query_results() {
    // Prove that evidence refs imported through the canonical path
    // do not leak raw evidence data into search results.
    let dir = TempDir::new().unwrap();
    let store = open_store(&dir);

    let envelope = make_claim_with_evidence_envelope("test-ns");
    let batch = canonical_batch_from_v1(&envelope);
    store.import_projection_batch(&batch).await.unwrap();

    let adapter =
        knowledge_runtime::adapters::semantic_memory::SemanticMemoryAdapter::new(store.clone());
    let config = test_runtime_config();
    let runtime = KnowledgeRuntime::new(config, adapter).unwrap();

    let scope = Scope::new("test-ns");
    let (results, _trace) = runtime.query("test coverage", Some(&scope)).await.unwrap();

    assert_contains_projection_kind(&results, "claim_version");
    // Search results must NOT contain raw evidence fetch handles
    for result in &results {
        assert!(
            !result.content.contains("forge://evidence/"),
            "KR-005: raw evidence fetch handle must not appear in search results: {}",
            result.content
        );
        assert!(
            !matches!(
                &result.source,
                SearchSource::Projection { projection_kind, .. } if projection_kind == "evidence_ref"
            ),
            "KR-005: evidence refs must remain opaque during normal retrieval"
        );
    }
}

#[tokio::test]
async fn kr005_evidence_refs_are_only_exposed_via_explain_path() {
    let dir = TempDir::new().unwrap();
    let store = open_store(&dir);

    let envelope = make_claim_with_evidence_envelope("test-ns");
    let batch = canonical_batch_from_v1(&envelope);
    let _result = store.import_projection_batch(&batch).await.unwrap();

    let adapter =
        knowledge_runtime::adapters::semantic_memory::SemanticMemoryAdapter::new(store.clone());
    let runtime = KnowledgeRuntime::new(test_runtime_config(), adapter).unwrap();

    let scope = Scope::new("test-ns");
    let (search_results, _trace) = runtime.query("test coverage", Some(&scope)).await.unwrap();
    assert!(
        search_results
            .iter()
            .any(|r| r.content.contains("Entity ent-ev has test coverage")),
        "expected imported claim content in query results"
    );
    for result in &search_results {
        assert!(
            !result.content.contains("forge://evidence/"),
            "KR-005: search must not leak raw evidence handles"
        );
        assert!(
            !matches!(
                &result.source,
                SearchSource::Projection {
                    projection_kind,
                    ..
                } if projection_kind == "evidence_ref"
            ),
            "KR-005: evidence refs must not appear in search result provenance"
        );
    }

    let evidence_refs = runtime
        .query_evidence_refs_for_claim("claim-ev", None, Some(&scope), 10)
        .await
        .unwrap();
    assert_eq!(
        evidence_refs.len(),
        1,
        "expected one evidence ref for claim-ev"
    );
    assert_eq!(evidence_refs[0].claim_id.as_str(), "claim:claim-ev");
    assert!(
        evidence_refs[0]
            .fetch_handle
            .starts_with("forge://evidence/"),
        "explain path must surface opaque fetch handle"
    );

    let older_cutoff = runtime
        .query_evidence_refs_for_claim_as_of(
            "claim-ev",
            None,
            Some(&scope),
            "1970-01-01T00:00:00Z",
            10,
        )
        .await
        .unwrap();
    assert!(
        older_cutoff.is_empty(),
        "cutoff before latest import should yield no evidence rows"
    );

    let none_found = runtime
        .query_evidence_refs_for_claim("missing-claim", None, Some(&scope), 10)
        .await
        .unwrap();
    assert!(
        none_found.is_empty(),
        "non-existent claim must return no evidence refs"
    );
}

#[tokio::test]
async fn kr005_trace_ctx_continuity_through_import_and_query() {
    // Prove trace context is preserved from Forge export through
    // bridge, import, and runtime query.
    let dir = TempDir::new().unwrap();
    let store = open_store(&dir);

    let envelope = make_claim_envelope("test-ns");
    let export_trace_id = envelope.trace_ctx.as_ref().unwrap().trace_id.clone();

    // Bridge preserves trace
    let batch = canonical_batch_from_v1(&envelope);
    assert_eq!(
        batch.trace_ctx.as_ref().unwrap().trace_id,
        export_trace_id,
        "bridge must preserve export trace_id"
    );

    // Import carries provenance
    let import_result = store.import_projection_batch(&batch).await.unwrap();
    assert_eq!(import_result.source_envelope_id, "envelope:env-001");

    // Runtime query with correlated trace
    let adapter =
        knowledge_runtime::adapters::semantic_memory::SemanticMemoryAdapter::new(store.clone());
    let config = test_runtime_config();
    let runtime = KnowledgeRuntime::new(config, adapter).unwrap();

    let query_trace = TraceCtx::from_trace_id(&export_trace_id);
    let scope = Scope::new("test-ns");
    let (_results, trace) = runtime
        .query_with_trace("function entity", Some(&scope), Some(query_trace))
        .await
        .unwrap();

    // Query trace preserves the correlated trace_id
    assert_eq!(
        trace.trace_ctx.trace_id, export_trace_id,
        "KR-005: trace_id must be continuous from export through query"
    );
}

#[tokio::test]
async fn kr104_causal_query_answers_from_imported_episode_projection() {
    use semantic_memory_forge::ExportEpisode;

    let dir = TempDir::new().unwrap();
    let store = open_store(&dir);

    let records = vec![ExportRecord::Episode(ExportEpisode {
        episode_id: Some(stack_ids::EpisodeId::generate()),
        document_id: "doc-causal".into(),
        cause_ids: vec!["build-1".into()],
        effect_type: "test_failure".into(),
        outcome: "regression".into(),
        confidence: 0.88,
        experiment_id: Some("exp-causal".into()),
        metadata: None,
    })];
    let scope_key = stack_ids::ScopeKey::namespace_only("test-ns");
    let digest = ExportEnvelopeV1::compute_digest("forge", &scope_key, &records).unwrap();
    let envelope = ExportEnvelopeV1 {
        envelope_id: EnvelopeId::new("env-causal"),
        schema_version: EXPORT_ENVELOPE_V1_SCHEMA.into(),
        content_digest: digest,
        source_authority: "forge".into(),
        scope_key,
        trace_ctx: Some(TraceCtx::generate()),
        exported_at: "2026-03-08T00:00:00Z".into(),
        records,
    };
    let batch = canonical_batch_from_v1(&envelope);
    store.import_projection_batch(&batch).await.unwrap();

    let adapter =
        knowledge_runtime::adapters::semantic_memory::SemanticMemoryAdapter::new(store.clone());
    let runtime = KnowledgeRuntime::new(test_runtime_config(), adapter).unwrap();
    let scope = Scope::new("test-ns");
    let (results, _trace) = runtime
        .query("test_failure regression", Some(&scope))
        .await
        .unwrap();

    assert_contains_projection_kind(&results, "episode");
    assert!(
        results.iter().any(|result| result
            .content
            .contains("doc-causal test_failure regression")),
        "runtime must surface imported episode projections on causal queries"
    );
}

#[tokio::test]
async fn kr105_bounded_entity_candidate_expansion_uses_imported_aliases() {
    use semantic_memory_forge::ExportEntityAlias;

    let dir = TempDir::new().unwrap();
    let store = open_store(&dir);

    let records = vec![
        ExportRecord::Claim(ExportClaim {
            claim_id: Some(ClaimId::new("claim-alias")),
            claim_version_id: Some(ClaimVersionId::new(format!("fixture-version-v{}", line!()))),
            subject_entity_id: EntityId::new("ent-alias"),
            predicate: "owns".into(),
            object_anchor: serde_json::json!("pipeline"),
            valid_from: Some("2026-01-01T00:00:00Z".into()),
            valid_to: None,
            confidence: 0.9,
            content: "Entity alias target owns the pipeline".into(),
            projection_family: "forge_verification".into(),
            supersedes_claim_id: None,
            supersedes_claim_version_id: None,
            metadata: None,
        }),
        ExportRecord::EntityAlias(ExportEntityAlias {
            canonical_entity_id: EntityId::new("ent-alias"),
            alias_text: "Entity One".into(),
            alias_source: "forge_extraction".into(),
            match_evidence: None,
            confidence: 0.9,
            scope: None,
            superseded_by_entity_id: None,
            split_from_entity_id: None,
        }),
    ];
    let scope_key = stack_ids::ScopeKey::namespace_only("test-ns");
    let digest = ExportEnvelopeV1::compute_digest("forge", &scope_key, &records).unwrap();
    let envelope = ExportEnvelopeV1 {
        envelope_id: EnvelopeId::new("env-alias"),
        schema_version: EXPORT_ENVELOPE_V1_SCHEMA.into(),
        content_digest: digest,
        source_authority: "forge".into(),
        scope_key,
        trace_ctx: Some(TraceCtx::generate()),
        exported_at: "2026-03-08T00:00:00Z".into(),
        records,
    };
    let batch = canonical_batch_from_v1(&envelope);
    store.import_projection_batch(&batch).await.unwrap();

    let adapter =
        knowledge_runtime::adapters::semantic_memory::SemanticMemoryAdapter::new(store.clone());
    let runtime = KnowledgeRuntime::new(test_runtime_config(), adapter).unwrap();
    let scope = Scope::new("test-ns");
    let (results, _trace) = runtime
        .query("what does \"Entty One\" own", Some(&scope))
        .await
        .unwrap();

    assert_contains_projection_kind(&results, "claim_version");
    assert!(
        results
            .iter()
            .any(|result| result.content.contains("owns the pipeline")),
        "bounded candidate expansion must recover imported alias matches without side-loaded entities"
    );
}

// ── E2E-001: Architecture-closure proof suite ────────────────────
//
// This section is the dedicated architecture-closure proof matrix.
// It exercises the canonical path and asserts:
// - Evidence opacity
// - Lineage continuity
// - Scope/temporal truthfulness
// - Idempotent import
// - Bridge-assigned defaults vs exporter truth

#[tokio::test]
async fn e2e001_canonical_path_with_all_record_types() {
    use semantic_memory_forge::{ExportEntityAlias, ExportEpisode, ExportRelation};
    use stack_ids::ScopeKey;

    let dir = TempDir::new().unwrap();
    let store = open_store(&dir);

    // Build an envelope with all record types
    let records = vec![
        ExportRecord::Claim(ExportClaim {
            claim_id: Some(ClaimId::new("claim-e2e")),
            claim_version_id: Some(ClaimVersionId::new(format!("fixture-version-v{}", line!()))),
            subject_entity_id: EntityId::new("ent-e2e"),
            predicate: "has_type".into(),
            object_anchor: serde_json::json!("function"),
            valid_from: Some("2026-01-01T00:00:00Z".into()),
            valid_to: None,
            confidence: 0.95,
            content: "Entity ent-e2e is a function for testing".into(),
            projection_family: "forge_verification".into(),
            supersedes_claim_id: None,
            supersedes_claim_version_id: None,
            metadata: None,
        }),
        ExportRecord::Relation(ExportRelation {
            relation_version_id: Some(RelationVersionId::new(format!(
                "relation-fixture-{}",
                line!()
            ))),
            subject_entity_id: EntityId::new("ent-e2e"),
            predicate: "depends_on".into(),
            object_anchor: serde_json::json!("ent-dep"),
            valid_from: None,
            valid_to: None,
            confidence: 0.8,
            projection_family: "forge_verification".into(),
            source_claim_id: Some(ClaimId::new("claim-e2e")),
            source_episode_id: None,
            supersedes_relation_version_id: None,
            metadata: None,
        }),
        ExportRecord::EvidenceRef(ExportEvidenceRef {
            claim_id: ClaimId::new("claim-e2e"),
            claim_version_id: Some(ClaimVersionId::new(format!("fixture-version-v{}", line!()))),
            fetch_handle: "forge://evidence/run-99/artifact-1".into(),
            source_authority: "forge".into(),
            metadata: None,
        }),
        ExportRecord::EntityAlias(ExportEntityAlias {
            canonical_entity_id: EntityId::new("ent-e2e"),
            alias_text: "E2E Test Entity".into(),
            alias_source: "forge_extraction".into(),
            match_evidence: None,
            confidence: 0.9,
            scope: None,
            superseded_by_entity_id: Some(EntityId::new("ent-old")),
            split_from_entity_id: Some(EntityId::new("ent-split")),
        }),
        ExportRecord::Episode(ExportEpisode {
            episode_id: Some(stack_ids::EpisodeId::generate()),
            document_id: "doc-e2e".into(),
            cause_ids: vec!["cause-1".into()],
            effect_type: "code_change".into(),
            outcome: "success".into(),
            confidence: 0.85,
            experiment_id: Some("exp-e2e".into()),
            metadata: None,
        }),
    ];
    let scope = ScopeKey::namespace_only("e2e-ns");
    let digest = ExportEnvelopeV1::compute_digest("forge", &scope, &records).unwrap();
    let trace = TraceCtx::generate();

    let envelope = ExportEnvelopeV1 {
        envelope_id: EnvelopeId::new("env-e2e-full"),
        schema_version: EXPORT_ENVELOPE_V1_SCHEMA.into(),
        content_digest: digest,
        source_authority: "forge".into(),
        scope_key: scope,
        trace_ctx: Some(trace.clone()),
        exported_at: "2026-03-08T00:00:00Z".into(),
        records,
    };

    // Bridge transform
    let batch = canonical_batch_from_v1(&envelope);
    assert_eq!(batch.records.len(), 5, "all 5 record types must transform");
    assert_eq!(batch.trace_ctx.as_ref().unwrap().trace_id, trace.trace_id);

    // Verify bridge defaults vs exporter truth
    for record in &batch.records {
        match &record.record {
            ImportProjectionRecord::ClaimVersion(cv) => {
                assert_eq!(
                    cv.freshness,
                    forge_memory_bridge::ProjectionFreshness::Current
                );
                assert_eq!(cv.claim_state, forge_memory_bridge::ClaimState::Active);
                assert!(
                    cv.supersedes_claim_version_id.is_none(),
                    "version-level supersession must be None (deferred)"
                );
            }
            ImportProjectionRecord::RelationVersion(rv) => {
                assert_eq!(
                    rv.freshness,
                    forge_memory_bridge::ProjectionFreshness::Current
                );
            }
            ImportProjectionRecord::EntityAlias(ea) => {
                assert!(
                    !ea.is_human_confirmed_final,
                    "automated flow must not set human_confirmed_final"
                );
                assert_eq!(
                    ea.review_state,
                    forge_memory_bridge::ReviewState::PendingReview
                );
            }
            ImportProjectionRecord::EvidenceRef(ev) => {
                assert!(
                    ev.fetch_handle.starts_with("forge://"),
                    "evidence handle must be opaque reference"
                );
            }
            ImportProjectionRecord::Episode(_) => {}
        }
        assert!(
            record.semantics.is_none(),
            "bounded canonical fixture should stay thin rather than inventing V3 semantics"
        );
    }

    // Import into memory
    let result = store.import_projection_batch(&batch).await.unwrap();
    assert_eq!(result.status, "complete");
    assert_eq!(result.record_count, 5);
    assert!(!result.was_duplicate);

    // Idempotent re-import
    let result2 = store.import_projection_batch(&batch).await.unwrap();
    assert!(result2.was_duplicate, "re-import must be idempotent");

    let adapter =
        knowledge_runtime::adapters::semantic_memory::SemanticMemoryAdapter::new(store.clone());
    let config = RuntimeConfig {
        default_scope: Scope::new("e2e-ns"),
        ..test_runtime_config()
    };
    let runtime = KnowledgeRuntime::new(config, adapter).unwrap();

    let scope = Scope::new("e2e-ns");
    let (results, query_trace) = runtime
        .query_with_trace(
            "function entity",
            Some(&scope),
            Some(TraceCtx::from_trace_id(&trace.trace_id)),
        )
        .await
        .unwrap();

    assert_contains_projection_kind(&results, "claim_version");
    // Trace continuity
    assert_eq!(
        query_trace.trace_ctx.trace_id, trace.trace_id,
        "E2E-001: trace_id must be continuous end-to-end"
    );

    assert!(
        results
            .iter()
            .any(|result| result.content.contains("function for testing")),
        "E2E-001: imported claims must be query-visible without side-loaded facts"
    );

    // Evidence opacity: raw fetch handles must NOT appear in search results
    for r in &results {
        assert!(
            !r.content.contains("forge://evidence/"),
            "E2E-001: evidence must remain opaque in search results"
        );
    }
}

#[tokio::test]
async fn e2e001_scope_truthfulness_strict_and_non_strict() {
    let dir = TempDir::new().unwrap();
    let store = open_store(&dir);
    let scoped_scope = Scope::new("e2e-ns")
        .with_domain("code")
        .with_workspace("ws-1")
        .with_repo("repo-1");
    let other_scope = Scope::new("e2e-ns")
        .with_domain("docs")
        .with_workspace("ws-2")
        .with_repo("repo-2");

    let matching = make_claim_envelope_with_scope(
        scoped_scope.key(),
        "env-scope-hit",
        "claim-scope-hit",
        "ent-scope-hit",
        "scoped projection result",
        Some("2026-01-01T00:00:00Z".into()),
        None,
    );
    let non_matching = make_claim_envelope_with_scope(
        other_scope.key(),
        "env-scope-miss",
        "claim-scope-miss",
        "ent-scope-miss",
        "scoped projection result from another scope",
        Some("2026-01-01T00:00:00Z".into()),
        None,
    );
    for envelope in [matching, non_matching] {
        let batch = canonical_batch_from_v1(&envelope);
        store.import_projection_batch(&batch).await.unwrap();
    }

    // Non-strict: full scope is enforced on projection-backed retrieval, so the
    // query succeeds without a partial-enforcement warning.
    {
        let adapter =
            knowledge_runtime::adapters::semantic_memory::SemanticMemoryAdapter::new(store.clone());
        let config = RuntimeConfig {
            default_scope: Scope::new("e2e-ns"),
            strict_scope: false,
            ..test_runtime_config()
        };
        let runtime = KnowledgeRuntime::new(config, adapter).unwrap();
        let (results, trace) = runtime
            .query("scoped projection", Some(&scoped_scope))
            .await
            .unwrap();
        assert!(
            !trace.has_scope_enforcement_warning(),
            "E2E-001: projection-backed scoped query must not warn about partial scope enforcement"
        );
        assert_contains_projection_kind(&results, "claim_version");
        assert!(
            results
                .iter()
                .any(|result| result.content == "scoped projection result"),
            "E2E-001: full-scope query must return the matching imported row"
        );
        assert!(
            results
                .iter()
                .all(|result| !result.content.contains("another scope")),
            "E2E-001: full-scope query must exclude rows from other scopes"
        );
    }

    // Strict: the same projection-backed route still succeeds.
    {
        let adapter =
            knowledge_runtime::adapters::semantic_memory::SemanticMemoryAdapter::new(store.clone());
        let config = RuntimeConfig {
            default_scope: Scope::new("e2e-ns"),
            strict_scope: true,
            ..test_runtime_config()
        };
        let runtime = KnowledgeRuntime::new(config, adapter).unwrap();
        let (results, trace) = runtime
            .query("scoped projection", Some(&scoped_scope))
            .await
            .unwrap();
        assert!(
            !trace.has_scope_enforcement_warning(),
            "E2E-001: strict scope must pass on projection-backed routes"
        );
        assert_contains_projection_kind(&results, "claim_version");
    }
}

#[tokio::test]
async fn projected_verification_summary_distinguishes_verification_states() {
    let dir = TempDir::new().unwrap();
    let store = open_store(&dir);
    let scope = Scope::new("test-ns");

    let envelopes = vec![
        make_claim_envelope_with_scope_and_metadata(
            scope.key(),
            "env-verified",
            "claim-verified",
            "claim-verified-v1",
            "ent-verified",
            "verified projection claim",
            Some(make_verification_summary_metadata(
                "verified",
                serde_json::json!({
                    "state": "promoted",
                    "version_id": "basis-42",
                    "promoted_at": "2026-03-07T00:00:00Z",
                }),
                2,
                1,
                0,
                Some("cmp-verified-1"),
                &["paired verification clean"],
            )),
            Some("2026-01-01T00:00:00Z".into()),
            None,
        ),
        make_claim_envelope_with_scope_and_metadata(
            scope.key(),
            "env-contradicted",
            "claim-contradicted",
            "claim-contradicted-v1",
            "ent-contradicted",
            "contradicted projection claim",
            Some(make_verification_summary_metadata(
                "contradicted",
                serde_json::json!({
                    "state": "blocked",
                    "reason": "placebo_failed",
                }),
                2,
                0,
                1,
                Some("cmp-contradicted-1"),
                &["placebo failed"],
            )),
            Some("2026-01-01T00:00:00Z".into()),
            None,
        ),
        make_claim_envelope_with_scope_and_metadata(
            scope.key(),
            "env-superseded",
            "claim-superseded",
            "claim-superseded-v1",
            "ent-superseded",
            "superseded projection claim",
            Some(make_verification_summary_metadata(
                "superseded",
                serde_json::json!({
                    "state": "not_promoted",
                }),
                1,
                0,
                0,
                Some("cmp-superseded-1"),
                &[],
            )),
            Some("2026-01-01T00:00:00Z".into()),
            Some("2026-02-01T00:00:00Z".into()),
        ),
        make_claim_envelope_with_scope_and_metadata(
            scope.key(),
            "env-unverified",
            "claim-unverified",
            "claim-unverified-v1",
            "ent-unverified",
            "unverified projection claim",
            None,
            Some("2026-01-01T00:00:00Z".into()),
            None,
        ),
    ];

    for envelope in envelopes {
        let batch = canonical_batch_from_v1(&envelope);
        store.import_projection_batch(&batch).await.unwrap();
    }

    let adapter =
        knowledge_runtime::adapters::semantic_memory::SemanticMemoryAdapter::new(store.clone());
    let runtime = KnowledgeRuntime::new(test_runtime_config(), adapter).unwrap();

    let verified = runtime
        .query_verification_summary_for_claim(
            "claim-verified",
            Some("claim-verified-v1"),
            Some(&scope),
        )
        .await
        .unwrap()
        .expect("verified summary should exist");
    assert_eq!(
        verified.lifecycle_state,
        ProjectedVerificationLifecycle::Verified
    );
    assert_eq!(
        verified.promotion_state,
        ProjectedPromotionState::Promoted {
            version_id: Some("basis-42".into()),
            promoted_at: Some("2026-03-07T00:00:00Z".into()),
        }
    );
    assert_eq!(verified.completed_trial_count, 2);
    assert_eq!(verified.passed_refutation_count, 1);
    assert_eq!(verified.failed_refutation_count, 0);
    assert_eq!(
        verified.comparability_snapshot_version.as_deref(),
        Some("cmp-verified-1")
    );
    assert_eq!(verified.notes, vec!["paired verification clean"]);

    let contradicted = runtime
        .query_verification_summary_for_claim(
            "claim-contradicted",
            Some("claim-contradicted-v1"),
            Some(&scope),
        )
        .await
        .unwrap()
        .expect("contradicted summary should exist");
    assert_eq!(
        contradicted.lifecycle_state,
        ProjectedVerificationLifecycle::Contradicted
    );
    assert_eq!(contradicted.claim_state, "disputed");
    assert!(
        contradicted.contradiction_status.contains("placebo failed"),
        "projected contradiction state should carry exported notes"
    );

    let superseded = runtime
        .query_verification_summary_for_claim(
            "claim-superseded",
            Some("claim-superseded-v1"),
            Some(&scope),
        )
        .await
        .unwrap()
        .expect("superseded summary should exist");
    assert_eq!(
        superseded.lifecycle_state,
        ProjectedVerificationLifecycle::Superseded
    );
    assert_eq!(superseded.freshness, "superseded");

    let unverified = runtime
        .query_verification_summary_for_claim(
            "claim-unverified",
            Some("claim-unverified-v1"),
            Some(&scope),
        )
        .await
        .unwrap()
        .expect("unverified summary should exist");
    assert_eq!(
        unverified.lifecycle_state,
        ProjectedVerificationLifecycle::Unverified
    );
    assert_eq!(
        unverified.promotion_state,
        ProjectedPromotionState::NotPromoted
    );
}

#[tokio::test]
async fn projected_verification_summary_as_of_uses_recorded_time_cutoff() {
    let dir = TempDir::new().unwrap();
    let store = open_store(&dir);
    let scope = Scope::new("test-ns");

    let first = make_claim_envelope_with_scope_and_metadata(
        scope.key(),
        "env-history-v1",
        "claim-history",
        "claim-history-v1",
        "ent-history",
        "historical verification claim",
        Some(make_verification_summary_metadata(
            "unverified",
            serde_json::json!({ "state": "not_promoted" }),
            0,
            0,
            0,
            Some("cmp-history-v1"),
            &["awaiting paired replay"],
        )),
        Some("2026-01-01T00:00:00Z".into()),
        Some("2026-02-01T00:00:00Z".into()),
    );
    let first_batch = canonical_batch_from_v1(&first);
    store.import_projection_batch(&first_batch).await.unwrap();
    let cutoff = store
        .last_import_at("test-ns")
        .await
        .unwrap()
        .expect("first import should record timestamp");

    tokio::time::sleep(std::time::Duration::from_millis(15)).await;

    let second = make_claim_envelope_with_scope_and_metadata(
        scope.key(),
        "env-history-v2",
        "claim-history",
        "claim-history-v2",
        "ent-history",
        "current verification claim",
        Some(make_verification_summary_metadata(
            "verified",
            serde_json::json!({ "state": "eligible" }),
            2,
            1,
            0,
            Some("cmp-history-v2"),
            &["paired trials completed"],
        )),
        Some("2026-02-01T00:00:00Z".into()),
        None,
    );
    let second_batch = canonical_batch_from_v1(&second);
    store.import_projection_batch(&second_batch).await.unwrap();

    let adapter =
        knowledge_runtime::adapters::semantic_memory::SemanticMemoryAdapter::new(store.clone());
    let runtime = KnowledgeRuntime::new(test_runtime_config(), adapter).unwrap();

    let historical = runtime
        .query_verification_summary_for_claim_as_of("claim-history", None, Some(&scope), &cutoff)
        .await
        .unwrap()
        .expect("historical summary should exist");
    assert_eq!(
        historical.claim_version_id,
        "claim-version:claim-history-v1"
    );
    assert_eq!(
        historical.lifecycle_state,
        ProjectedVerificationLifecycle::Unverified
    );

    let current = runtime
        .query_verification_summary_for_claim("claim-history", None, Some(&scope))
        .await
        .unwrap()
        .expect("current summary should exist");
    assert_eq!(
        current.claim_version_id,
        "claim-version:claim-history-v2"
    );
    assert_eq!(
        current.lifecycle_state,
        ProjectedVerificationLifecycle::Verified
    );
}

#[tokio::test]
async fn runtime_exposes_non_authoritative_inference_advisory_from_latest_v3_import() {
    let dir = TempDir::new().unwrap();
    let store = open_store(&dir);
    let batch = make_kernel_v3_batch("kernel-runtime-ns");
    store.import_projection_batch(&batch).await.unwrap();

    let adapter =
        knowledge_runtime::adapters::semantic_memory::SemanticMemoryAdapter::new(store.clone());
    let runtime = KnowledgeRuntime::new(test_runtime_config(), adapter).unwrap();

    let advisory = runtime
        .latest_inference_advisory(Some(&Scope::new("kernel-runtime-ns")))
        .await
        .unwrap()
        .expect("expected inference advisory");

    assert!(advisory.advisory_only);
    assert_eq!(advisory.execution_mode, "message_passing_baseline");
    assert!(advisory.iteration_count >= 1);
    assert!(advisory.message_count >= 1);
    assert_eq!(advisory.oracle_mode, "exact_bounded");
    assert!(advisory.oracle_supported);
    assert_eq!(advisory.satisfied_constraint_count, 1);
    assert_eq!(advisory.degraded_reason, None);
    assert!(
        advisory.degradation_markers.is_empty(),
        "rich V3 import should not degrade on the bounded fixture"
    );
}

#[tokio::test]
async fn runtime_surfaces_thin_export_as_conservative_advisory() {
    let dir = TempDir::new().unwrap();
    let store = open_store(&dir);
    let mut batch = make_kernel_v3_batch("kernel-runtime-thin");
    batch.records[0].semantics = None;
    store.import_projection_batch(&batch).await.unwrap();

    let adapter =
        knowledge_runtime::adapters::semantic_memory::SemanticMemoryAdapter::new(store.clone());
    let runtime = KnowledgeRuntime::new(test_runtime_config(), adapter).unwrap();

    let advisory = runtime
        .latest_inference_advisory(Some(&Scope::new("kernel-runtime-thin")))
        .await
        .unwrap()
        .expect("expected thin-export inference advisory");

    assert!(advisory.advisory_only);
    assert_eq!(advisory.oracle_mode, "conservative_fallback");
    assert_eq!(advisory.degraded_reason, None);
    assert!(
        advisory
            .degradation_markers
            .contains(&"thin_export".to_string()),
        "thin export must degrade explicitly instead of hallucinating structure"
    );
}

#[tokio::test]
async fn runtime_selects_kernel_receipt_by_exact_scope_not_namespace_only() {
    let dir = TempDir::new().unwrap();
    let store = open_store(&dir);

    let mut repo_a = make_kernel_v3_batch("kernel-shared-ns");
    repo_a.source_envelope_id = EnvelopeId::new("env-runtime-kernel-v3-repo-a");
    repo_a.content_digest = ContentDigest::compute(b"runtime-kernel-v3-repo-a");
    repo_a.scope_key = stack_ids::ScopeKey {
        namespace: "kernel-shared-ns".into(),
        domain: None,
        workspace_id: None,
        repo_id: Some("repo-a".into()),
    };
    if let ImportProjectionRecord::ClaimVersion(claim) = &mut repo_a.records[0].record {
        claim.claim_id = ClaimId::new("claim-runtime-kernel-v3-repo-a");
        claim.claim_version_id = ClaimVersionId::new("claim-version-runtime-kernel-v3-repo-a");
        claim.subject_entity_id = EntityId::new("entity-runtime-kernel-v3-repo-a");
        claim.source_envelope_id = EnvelopeId::new("env-runtime-kernel-v3-repo-a");
        claim.scope_key = stack_ids::ScopeKey {
            namespace: "kernel-shared-ns".into(),
            domain: None,
            workspace_id: None,
            repo_id: Some("repo-a".into()),
        };
        claim.content = "repo-a runtime kernel claim".into();
    }
    if let Some(semantics) = &mut repo_a.records[0].semantics {
        semantics.claim_family_id = Some(ClaimFamilyId::new("family-runtime-kernel-v3-repo-a"));
        semantics.assertion_group_id =
            Some(AssertionGroupId::new("group-runtime-kernel-v3-repo-a"));
        semantics.derivation_seed_ids = vec!["seed-runtime-kernel-v3-repo-a".into()];
    }

    let mut repo_b = make_kernel_v3_batch("kernel-shared-ns");
    repo_b.source_envelope_id = EnvelopeId::new("env-runtime-kernel-v3-repo-b");
    repo_b.content_digest = ContentDigest::compute(b"runtime-kernel-v3-repo-b");
    repo_b.scope_key = stack_ids::ScopeKey {
        namespace: "kernel-shared-ns".into(),
        domain: None,
        workspace_id: None,
        repo_id: Some("repo-b".into()),
    };
    if let ImportProjectionRecord::ClaimVersion(claim) = &mut repo_b.records[0].record {
        claim.claim_id = ClaimId::new("claim-runtime-kernel-v3-repo-b");
        claim.claim_version_id = ClaimVersionId::new("claim-version-runtime-kernel-v3-repo-b");
        claim.subject_entity_id = EntityId::new("entity-runtime-kernel-v3-repo-b");
        claim.source_envelope_id = EnvelopeId::new("env-runtime-kernel-v3-repo-b");
        claim.scope_key = stack_ids::ScopeKey {
            namespace: "kernel-shared-ns".into(),
            domain: None,
            workspace_id: None,
            repo_id: Some("repo-b".into()),
        };
        claim.content = "repo-b runtime kernel claim".into();
    }
    repo_b.records[0].semantics = None;

    store.import_projection_batch(&repo_a).await.unwrap();
    store.import_projection_batch(&repo_b).await.unwrap();

    let adapter =
        knowledge_runtime::adapters::semantic_memory::SemanticMemoryAdapter::new(store.clone());
    let runtime = KnowledgeRuntime::new(
        RuntimeConfig {
            default_scope: Scope::new("kernel-shared-ns").with_repo("repo-a"),
            ..test_runtime_config()
        },
        adapter,
    )
    .unwrap();

    let advisory = runtime
        .latest_inference_advisory(Some(&Scope::new("kernel-shared-ns").with_repo("repo-a")))
        .await
        .unwrap()
        .expect("expected scope-specific inference advisory");

    assert_eq!(
        advisory.source_envelope_id,
        "envelope:env-runtime-kernel-v3-repo-a"
    );
    assert!(
        advisory.degradation_markers.is_empty(),
        "repo-a advisory should not inherit repo-b thin-export degradation"
    );
}

#[tokio::test]
async fn runtime_exposes_inference_explanation_and_allows_clean_risk_gate() {
    let dir = TempDir::new().unwrap();
    let store = open_store(&dir);
    let mut batch = make_kernel_v3_batch("kernel-runtime-clean");
    let mut peer = batch.records[0].clone();
    if let ImportProjectionRecord::ClaimVersion(claim) = &mut peer.record {
        claim.claim_id = ClaimId::new("claim-runtime-kernel-v3-peer");
        claim.claim_version_id = ClaimVersionId::new("claim-version-runtime-kernel-v3-peer");
        claim.subject_entity_id = EntityId::new("entity-runtime-kernel-v3-peer");
        claim.content = "runtime kernel peer claim".into();
    }
    if let Some(semantics) = &mut peer.semantics {
        semantics.claim_family_id = Some(ClaimFamilyId::new("family-runtime-kernel-v3-peer"));
        semantics.derivation_seed_ids = vec!["seed-runtime-kernel-v3-peer".into()];
    }
    batch.records.push(peer);
    store.import_projection_batch(&batch).await.unwrap();

    let adapter =
        knowledge_runtime::adapters::semantic_memory::SemanticMemoryAdapter::new(store.clone());
    let runtime = KnowledgeRuntime::new(test_runtime_config(), adapter).unwrap();

    let explanation = runtime
        .latest_inference_explanation(Some(&Scope::new("kernel-runtime-clean")))
        .await
        .unwrap()
        .expect("expected inference explanation");
    assert!(explanation.advisory_only);
    assert_eq!(explanation.degraded_reason, None);
    assert_eq!(explanation.execution_mode, "message_passing_baseline");
    assert_eq!(explanation.refutation_outcome, "flip_witness_found");
    assert!(explanation.witness_count > 0);
    assert!(explanation.certificate_count > 0);
    assert!(explanation.calibration_caveats.is_empty());

    let gate = runtime
        .latest_risk_gate(Some(&Scope::new("kernel-runtime-clean")))
        .await
        .unwrap()
        .expect("expected risk gate");
    assert_eq!(gate.status, "allowed");
    assert!(gate.reasons.is_empty());
    assert!(gate.advisory_only);
    assert_eq!(gate.degraded_reason, None);
}

#[tokio::test]
async fn query_path_can_attach_kernel_explanation_without_promoting_truth() {
    let dir = TempDir::new().unwrap();
    let store = open_store(&dir);
    let batch = make_kernel_v3_batch("kernel-runtime-query");
    store.import_projection_batch(&batch).await.unwrap();

    let adapter =
        knowledge_runtime::adapters::semantic_memory::SemanticMemoryAdapter::new(store.clone());
    let runtime = KnowledgeRuntime::new(test_runtime_config(), adapter).unwrap();

    let (results, _trace, explanation) = runtime
        .query_with_inference_explanation(
            "runtime kernel",
            Some(&Scope::new("kernel-runtime-query")),
            None,
        )
        .await
        .unwrap();

    assert!(!results.is_empty());
    let explanation = explanation.expect("expected attached kernel explanation");
    assert!(explanation.advisory_only);
    assert_eq!(explanation.degraded_reason, None);
    assert_eq!(explanation.execution_mode, "message_passing_baseline");
}

#[tokio::test]
async fn runtime_blocks_risk_gate_when_nuisance_calibration_is_active() {
    let dir = TempDir::new().unwrap();
    let store = open_store(&dir);
    let mut batch = make_kernel_v3_batch("kernel-runtime-nuisance");
    let semantics = batch.records[0]
        .semantics
        .as_mut()
        .expect("semantics required for nuisance test");
    semantics.comparability_snapshot_version = Some("cmp-nuisance".into());
    semantics.nuisance_snapshot = Some(NuisanceSnapshot {
        environment_fingerprint: Some("linux".into()),
        toolchain_version: Some("rust-1.85".into()),
        dependency_set_hash: Some("deps-risk".into()),
        scope_mismatch_markers: vec![],
        measurement_notes: vec!["measurement drift".into()],
        selection_bias_markers: vec!["selection bias".into()],
    });
    store.import_projection_batch(&batch).await.unwrap();

    let adapter =
        knowledge_runtime::adapters::semantic_memory::SemanticMemoryAdapter::new(store.clone());
    let runtime = KnowledgeRuntime::new(test_runtime_config(), adapter).unwrap();

    let explanation = runtime
        .latest_inference_explanation(Some(&Scope::new("kernel-runtime-nuisance")))
        .await
        .unwrap()
        .expect("expected inference explanation");
    assert!(explanation
        .calibration_caveats
        .iter()
        .any(|caveat| caveat.contains("nuisance state")));

    let gate = runtime
        .latest_risk_gate(Some(&Scope::new("kernel-runtime-nuisance")))
        .await
        .unwrap()
        .expect("expected risk gate");
    assert_eq!(gate.status, "blocked");
    assert!(gate
        .reasons
        .contains(&"calibration_caveat_active".to_string()));
}

#[tokio::test]
async fn runtime_exposes_degraded_kernel_failure_artifacts_without_log_spelunking() {
    let dir = TempDir::new().unwrap();
    let store = open_store(&dir);
    let mut batch = make_kernel_v3_batch("kernel-runtime-degraded");
    batch.records[0].semantics = None;
    store.import_projection_batch(&batch).await.unwrap();

    let adapter =
        knowledge_runtime::adapters::semantic_memory::SemanticMemoryAdapter::new(store.clone());
    let runtime = KnowledgeRuntime::new(test_runtime_config(), adapter).unwrap();

    let explanation = runtime
        .latest_inference_explanation(Some(&Scope::new("kernel-runtime-degraded")))
        .await
        .unwrap()
        .expect("expected degraded inference explanation");
    assert!(explanation.advisory_only);
    assert_eq!(explanation.degraded_reason, None);
    assert!(explanation
        .degradation_markers
        .contains(&"thin_export".to_string()));
    assert!(!explanation.stop_reason.is_empty());
    assert!(!explanation.residual_micros.is_empty());
    assert!(!explanation.syndrome_signatures.is_empty());

    let gate = runtime
        .latest_risk_gate(Some(&Scope::new("kernel-runtime-degraded")))
        .await
        .unwrap()
        .expect("expected degraded risk gate");
    assert_eq!(gate.status, "blocked");
    assert_eq!(gate.degraded_reason, None);
    assert!(gate.reasons.contains(&"degradation_active".to_string()));
}

#[tokio::test]
async fn runtime_surfaces_scheduler_degraded_reason_through_query_provenance() {
    let dir = TempDir::new().unwrap();
    let store = open_store(&dir);
    let batch = make_scheduler_degraded_kernel_v3_batch("kernel-runtime-scheduler-degraded");
    store.import_projection_batch(&batch).await.unwrap();

    let adapter =
        knowledge_runtime::adapters::semantic_memory::SemanticMemoryAdapter::new(store.clone());
    let runtime = KnowledgeRuntime::new(test_runtime_config(), adapter).unwrap();

    let advisory = runtime
        .latest_inference_advisory(Some(&Scope::new("kernel-runtime-scheduler-degraded")))
        .await
        .unwrap()
        .expect("expected scheduler-degraded advisory");
    assert_eq!(
        advisory.degraded_reason.as_deref(),
        Some("explicit_changed_nodes_required_for_delta")
    );

    let explanation = runtime
        .latest_inference_explanation(Some(&Scope::new("kernel-runtime-scheduler-degraded")))
        .await
        .unwrap()
        .expect("expected scheduler-degraded explanation");
    assert_eq!(
        explanation.degraded_reason.as_deref(),
        Some("explicit_changed_nodes_required_for_delta")
    );

    let gate = runtime
        .latest_risk_gate(Some(&Scope::new("kernel-runtime-scheduler-degraded")))
        .await
        .unwrap()
        .expect("expected scheduler-degraded risk gate");
    assert_eq!(
        gate.degraded_reason.as_deref(),
        Some("explicit_changed_nodes_required_for_delta")
    );
    assert!(gate
        .reasons
        .contains(&"scheduled_degraded:explicit_changed_nodes_required_for_delta".to_string()));

    let (_results, trace, attached_explanation) = runtime
        .query_with_inference_explanation(
            "runtime kernel",
            Some(&Scope::new("kernel-runtime-scheduler-degraded")),
            None,
        )
        .await
        .unwrap();
    let attached_explanation = attached_explanation.expect("expected attached explanation");
    assert_eq!(
        attached_explanation.degraded_reason.as_deref(),
        Some("explicit_changed_nodes_required_for_delta")
    );
    assert_eq!(
        trace.kernel_degraded_reason.as_deref(),
        Some("explicit_changed_nodes_required_for_delta")
    );
    let provenance = trace.runtime_query_provenance();
    assert_eq!(
        provenance.kernel_degraded_reason.as_deref(),
        Some("explicit_changed_nodes_required_for_delta")
    );
}

#[tokio::test]
async fn runtime_makes_oracle_parity_downgrade_visible_between_rich_and_thin_batches() {
    let dir = TempDir::new().unwrap();
    let store = open_store(&dir);

    let rich = make_kernel_v3_batch("kernel-runtime-rich");
    let mut thin = make_kernel_v3_batch("kernel-runtime-thin-compare");
    thin.source_envelope_id = EnvelopeId::new("env-runtime-kernel-v3-thin-compare");
    thin.content_digest = ContentDigest::compute(b"runtime-kernel-v3-thin-compare");
    if let ImportProjectionRecord::ClaimVersion(claim) = &mut thin.records[0].record {
        claim.claim_id = ClaimId::new("claim-runtime-kernel-v3-thin-compare");
        claim.claim_version_id =
            ClaimVersionId::new("claim-version-runtime-kernel-v3-thin-compare");
        claim.subject_entity_id = EntityId::new("entity-runtime-kernel-v3-thin-compare");
        claim.source_envelope_id = EnvelopeId::new("env-runtime-kernel-v3-thin-compare");
        claim.content = "runtime kernel thin compare claim".into();
    }
    thin.records[0].semantics = None;
    store.import_projection_batch(&rich).await.unwrap();
    store.import_projection_batch(&thin).await.unwrap();

    let adapter =
        knowledge_runtime::adapters::semantic_memory::SemanticMemoryAdapter::new(store.clone());
    let runtime = KnowledgeRuntime::new(test_runtime_config(), adapter).unwrap();

    let rich_advisory = runtime
        .latest_inference_advisory(Some(&Scope::new("kernel-runtime-rich")))
        .await
        .unwrap()
        .expect("expected rich advisory");
    let thin_advisory = runtime
        .latest_inference_advisory(Some(&Scope::new("kernel-runtime-thin-compare")))
        .await
        .unwrap()
        .expect("expected thin advisory");

    assert!(rich_advisory.advisory_only);
    assert!(thin_advisory.advisory_only);
    assert_eq!(rich_advisory.oracle_mode, "exact_bounded");
    assert_eq!(thin_advisory.oracle_mode, "conservative_fallback");
    assert!(rich_advisory.degradation_markers.is_empty());
    assert!(thin_advisory
        .degradation_markers
        .contains(&"thin_export".to_string()));
}

#[tokio::test]
async fn query_temporal_explicit_degrades_with_temporal_warning_when_projection_missing() {
    let dir = TempDir::new().unwrap();
    let store = open_store(&dir);
    let adapter =
        knowledge_runtime::adapters::semantic_memory::SemanticMemoryAdapter::new(store.clone());
    let runtime = KnowledgeRuntime::new(test_runtime_config(), adapter).unwrap();

    let scope = Scope::new("no-proj-ns");
    let (results, trace) = runtime
        .query_temporal(
            "when was this claim last updated",
            Some(&scope),
            "2026-03-10T00:00:00Z",
            "2026-03-10T00:00:00Z",
        )
        .await
        .unwrap();

    assert!(
        trace.has_temporal_downgrade(),
        "explicit temporal query without projection imports must downgrade to hybrid"
    );
    assert!(
        results.is_empty(),
        "explicit temporal query on namespace without imports should return empty results"
    );
}

#[tokio::test]
async fn query_temporal_strict_mode_rejects_temporal_without_projection() {
    let dir = TempDir::new().unwrap();
    let store = open_store(&dir);
    let adapter =
        knowledge_runtime::adapters::semantic_memory::SemanticMemoryAdapter::new(store.clone());
    let runtime = KnowledgeRuntime::new(
        RuntimeConfig {
            strict_temporal: true,
            ..test_runtime_config()
        },
        adapter,
    )
    .unwrap();

    let scope = Scope::new("no-proj-ns");
    let err = runtime
        .query_temporal(
            "what happened yesterday",
            Some(&scope),
            "2026-03-10T00:00:00Z",
            "2026-03-10T00:00:00Z",
        )
        .await
        .unwrap_err();

    assert_eq!(
        err.kind(),
        "temporal_not_supported",
        "strict temporal without projection support must fail explicitly"
    );
}

// ── 10. Import log records timestamp for freshness checks ────────

#[tokio::test]
async fn import_records_timestamp_for_staleness_check() {
    let dir = TempDir::new().unwrap();
    let store = open_store(&dir);

    // Before import, no timestamp
    let ts_before = store.last_import_at("test-ns").await.unwrap();
    assert!(ts_before.is_none());

    // Import
    let envelope = make_claim_envelope("test-ns");
    let batch = canonical_batch_from_v1(&envelope);
    store.import_projection_batch(&batch).await.unwrap();

    // After import, timestamp should exist
    let ts_after = store.last_import_at("test-ns").await.unwrap();
    assert!(
        ts_after.is_some(),
        "import must record timestamp for freshness tracking"
    );
}

// ── LIB-CRIT-001: Domain-scoped queries must fall back to hybrid search ────

#[tokio::test]
async fn lib_crit_001_domain_scoped_query_uses_hybrid_with_full_scope_filtering() {
    // Regression test for LIB-CRIT-001: when scope has extra dimensions
    // (domain, workspace, repo), scope_requires_pushdown is true. Before
    // the fix, this caused the projection-only substring-matching path to
    // be the ONLY retrieval path, suppressing the hybrid search fallback.
    //
    // This test imports a projection (so has_projection_imports = true),
    // adds a fact to the semantic store (reachable via hybrid search),
    // and queries with domain scope using content that WON'T substring-match
    // the projection but WILL match via hybrid (FTS5/HNSW).

    let dir = TempDir::new().unwrap();
    let store = open_store(&dir);

    // Step 1: Add a scoped document reachable via hybrid search (FTS5/HNSW).
    let scope = Scope::new("test-ns").with_domain("code");
    ingest_scoped_document(
        &store,
        &scope,
        "JWT auth",
        "the authentication service validates JWT tokens on every request",
    )
    .await;

    // Step 2: Import a projection so has_projection_imports = true.
    // The projection content is about something unrelated to our query.
    let scope_key = stack_ids::ScopeKey {
        namespace: "test-ns".into(),
        domain: Some("code".into()),
        workspace_id: None,
        repo_id: None,
    };
    let envelope = make_claim_envelope_with_scope(
        scope_key,
        "env-crit-001",
        "claim-crit-001",
        "ent-crit-001",
        "Entity ent-crit-001 is a database migration",
        Some("2026-01-01T00:00:00Z".into()),
        None,
    );
    let batch = canonical_batch_from_v1(&envelope);
    store.import_projection_batch(&batch).await.unwrap();

    // Step 3: Build runtime with non-strict scope (Recall's configuration).
    let adapter =
        knowledge_runtime::adapters::semantic_memory::SemanticMemoryAdapter::new(store.clone());
    let config = RuntimeConfig {
        default_scope: Scope::new("test-ns"),
        strict_scope: false,
        ..test_runtime_config()
    };
    let runtime = KnowledgeRuntime::new(config, adapter).unwrap();

    // Step 4: Query with domain scope. The query is about "JWT authentication"
    // which won't substring-match the projection (about "database migration"),
    // but SHOULD be found via hybrid search while still honoring the domain filter.
    let (results, _trace) = runtime
        .query("JWT authentication token validation", Some(&scope))
        .await
        .unwrap();

    // Before LIB-CRIT-001 fix: results would be empty because the projection
    // substring-match returns nothing and hybrid fallback was suppressed.
    // After fix: hybrid search fallback kicks in when projection results < limit.
    assert!(
        !results.is_empty(),
        "LIB-CRIT-001: domain-scoped query must use hybrid search \
         when projection results are insufficient"
    );
}
