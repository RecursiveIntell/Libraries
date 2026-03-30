//! SEM-001: Golden-fixture semantic oracle tests.
//!
//! These tests ensure that the highest-risk semantic seams produce
//! deterministic results for known inputs.
#![allow(deprecated)]

use forge_memory_bridge::{transform_envelope, ImportProjectionRecord};
use semantic_memory_forge::{
    ExportClaim, ExportEnvelopeV1, ExportEpisode, ExportRecord, EXPORT_ENVELOPE_V1_SCHEMA,
};
use stack_ids::{
    ClaimId, ClaimVersionId, ContentDigest, EntityId, EnvelopeId, EpisodeId, ScopeKey, TraceCtx,
};

fn golden_v1_envelope() -> ExportEnvelopeV1 {
    let scope = ScopeKey::from_legacy_namespace("semantic-oracle-test");
    let records = vec![
        ExportRecord::Claim(ExportClaim {
            claim_version_id: Some(ClaimVersionId::new("cv-oracle-1")),
            claim_id: Some(ClaimId::new("claim-oracle-1")),
            subject_entity_id: EntityId::new("entity-oracle-1"),
            predicate: "test_predicate".into(),
            object_anchor: serde_json::json!("anchor-value"),
            content: "The oracle must produce deterministic output".into(),
            projection_family: "forge_verification".into(),
            confidence: 0.95,
            valid_from: None,
            valid_to: None,
            supersedes_claim_id: None,
            supersedes_claim_version_id: None,
            metadata: None,
        }),
        ExportRecord::Episode(ExportEpisode {
            episode_id: Some(EpisodeId::new("ep-oracle-1")),
            document_id: "doc-oracle-1".into(),
            cause_ids: vec!["cause-build-1".into()],
            effect_type: "test_observation".into(),
            outcome: "confirmed_deterministic".into(),
            confidence: 0.99,
            experiment_id: Some("exp-oracle-1".into()),
            metadata: None,
        }),
    ];
    let digest =
        ExportEnvelopeV1::compute_digest("test-authority", &scope, &records).expect("digest");
    ExportEnvelopeV1 {
        envelope_id: EnvelopeId::new("env-oracle-golden"),
        schema_version: EXPORT_ENVELOPE_V1_SCHEMA.to_string(),
        content_digest: digest,
        source_authority: "test-authority".into(),
        scope_key: scope,
        trace_ctx: Some(TraceCtx::generate()),
        exported_at: "2026-03-30T00:00:00Z".into(),
        records,
    }
}

#[test]
fn golden_fixture_transform_deterministic() {
    let envelope = golden_v1_envelope();
    let batch = transform_envelope(&envelope).expect("golden fixture must transform");
    assert_eq!(batch.source_envelope_id, envelope.envelope_id);
    assert_eq!(batch.records.len(), 2);
    assert_eq!(batch.scope_key.namespace, "semantic-oracle-test");
}

#[test]
fn golden_fixture_episode_identity_preserved() {
    let envelope = golden_v1_envelope();
    let batch = transform_envelope(&envelope).expect("transform");
    let ep = batch
        .records
        .iter()
        .find_map(|r| match r {
            ImportProjectionRecord::Episode(ep) => Some(ep),
            _ => None,
        })
        .expect("episode present");
    assert_eq!(ep.episode_id.as_str(), "ep-oracle-1");
    assert_eq!(ep.document_id, "doc-oracle-1");
}

#[test]
fn golden_fixture_claim_content_preserved() {
    let envelope = golden_v1_envelope();
    let batch = transform_envelope(&envelope).expect("transform");
    let cv = batch
        .records
        .iter()
        .find_map(|r| match r {
            ImportProjectionRecord::ClaimVersion(cv) => Some(cv),
            _ => None,
        })
        .expect("claim present");
    assert_eq!(cv.content, "The oracle must produce deterministic output");
    assert_eq!(cv.predicate, "test_predicate");
}

#[test]
fn golden_fixture_missing_episode_id_rejected() {
    let scope = ScopeKey::from_legacy_namespace("oracle-reject");
    let records = vec![ExportRecord::Episode(ExportEpisode {
        episode_id: None,
        document_id: "doc-no-id".into(),
        cause_ids: vec![],
        effect_type: "test".into(),
        outcome: "unknown".into(),
        confidence: 0.5,
        experiment_id: Some("exp-reject".into()),
        metadata: None,
    })];
    let digest = ExportEnvelopeV1::compute_digest("test", &scope, &records).expect("digest");
    let envelope = ExportEnvelopeV1 {
        envelope_id: EnvelopeId::new("env-reject"),
        schema_version: EXPORT_ENVELOPE_V1_SCHEMA.to_string(),
        content_digest: digest,
        source_authority: "test".into(),
        scope_key: scope,
        trace_ctx: None,
        exported_at: "2026-03-30T00:00:00Z".into(),
        records,
    };
    let err = transform_envelope(&envelope).expect_err("must reject missing episode_id");
    assert_eq!(err.kind(), "missing_episode_identity");
}
