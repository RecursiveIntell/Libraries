#![allow(clippy::expect_used)]
#![allow(deprecated)]

//! TRUTH-002 authority-fence negative tests: the Forge -> Bridge boundary.
//!
//! Locks the crate-level do-nots (AGENTS.md / authority boundary):
//! - the bridge never stamps importer-owned `recorded_at` / `imported_at`;
//! - the bridge never synthesizes `supersedes_claim_version_id` from
//!   claim-level lineage;
//! - the bridge never promotes alias review/confirmation truth;
//! - the bridge copies exporter confidence verbatim (no recalculation).

use forge_memory_bridge::{
    transform_envelope, ClaimState, ImportProjectionRecord, MergeDecision, ProjectionImportBatchV1,
    ReviewState,
};
use semantic_memory_forge::{
    ExportClaim, ExportEntityAlias, ExportEnvelopeV1, ExportRecord, EXPORT_ENVELOPE_V1_SCHEMA,
};
use stack_ids::{ClaimId, ClaimVersionId, EntityId, EnvelopeId, ScopeKey, TraceCtx};

fn build_envelope(records: Vec<ExportRecord>) -> ExportEnvelopeV1 {
    let scope = ScopeKey::namespace_only("fence-ns");
    let digest = ExportEnvelopeV1::compute_digest("forge", &scope, &records)
        .expect("digest computation must succeed");
    ExportEnvelopeV1 {
        envelope_id: EnvelopeId::new("env-fence-001"),
        schema_version: EXPORT_ENVELOPE_V1_SCHEMA.into(),
        content_digest: digest,
        source_authority: "forge".into(),
        scope_key: scope,
        trace_ctx: Some(TraceCtx::generate()),
        exported_at: "2026-03-08T12:00:00Z".into(),
        records,
    }
}

fn claim_record(
    claim_version_id: Option<&str>,
    supersedes_claim_id: Option<ClaimId>,
    supersedes_claim_version_id: Option<ClaimVersionId>,
    confidence: f32,
) -> ExportRecord {
    ExportRecord::Claim(ExportClaim {
        claim_id: Some(ClaimId::new("claim-fence")),
        claim_version_id: claim_version_id.map(ClaimVersionId::new),
        subject_entity_id: EntityId::new("entity-fence"),
        predicate: "has_type".into(),
        object_anchor: serde_json::json!("function"),
        valid_from: Some("2026-01-01T00:00:00Z".into()),
        valid_to: None,
        confidence,
        content: "fence claim content".into(),
        projection_family: "forge_verification".into(),
        supersedes_claim_id,
        supersedes_claim_version_id,
        metadata: None,
    })
}

fn transform_v1(envelope: ExportEnvelopeV1) -> ProjectionImportBatchV1 {
    transform_envelope(&envelope).expect("transform must succeed")
}

fn find_claim<'a>(
    batch: &'a ProjectionImportBatchV1,
    claim_version_id: &str,
) -> &'a forge_memory_bridge::ImportClaimVersion {
    batch
        .records
        .iter()
        .find_map(|record| match record {
            ImportProjectionRecord::ClaimVersion(cv)
                if cv.claim_version_id.as_str() == claim_version_id =>
            {
                Some(cv)
            }
            _ => None,
        })
        .expect("claim version must be present in the transformed batch")
}

/// The bridge output must not carry importer-owned timestamps at all:
/// `recorded_at` / `imported_at` are assigned by semantic-memory only.
#[test]
fn bridge_never_emits_importer_recorded_at_or_imported_at() {
    let envelope = build_envelope(vec![claim_record(Some("claim-fence-v1"), None, None, 0.9)]);
    let batch = transform_v1(envelope);
    let json = serde_json::to_value(&batch).expect("serialize batch");
    let text = json.to_string();
    assert!(
        !text.contains("\"recorded_at\""),
        "bridge output must not contain recorded_at: {text}"
    );
    assert!(
        !text.contains("\"imported_at\""),
        "bridge output must not contain imported_at: {text}"
    );
    // Bridge-owned provenance timestamps remain present.
    assert!(text.contains("\"transformed_at\""));
    assert!(text.contains("\"source_exported_at\""));
}

/// Claim-level supersession must NOT be minted into a version pointer.
/// `supersedes_claim_id` is lineage metadata; the version pointer stays None.
#[test]
fn bridge_does_not_synthesize_version_supersession_from_claim_lineage() {
    let envelope = build_envelope(vec![claim_record(
        Some("claim-fence-v1"),
        Some(ClaimId::new("claim-old")),
        None,
        0.9,
    )]);
    let batch = transform_v1(envelope);
    let cv = find_claim(&batch, "claim-fence-v1");
    assert_eq!(
        cv.supersedes_claim_version_id, None,
        "bridge must not synthesize a version supersession pointer"
    );
}

/// Exporter-provided version lineage is preserved verbatim (provenance copy).
#[test]
fn bridge_copies_exporter_supersession_pointer_verbatim() {
    let prior = ClaimVersionId::new("claim-old-v3");
    let envelope = build_envelope(vec![claim_record(
        Some("claim-fence-v2"),
        None,
        Some(prior.clone()),
        0.9,
    )]);
    let batch = transform_v1(envelope);
    let cv = find_claim(&batch, "claim-fence-v2");
    assert_eq!(
        cv.supersedes_claim_version_id
            .as_ref()
            .map(|id| id.as_str()),
        Some(prior.as_str()),
        "exporter lineage must be copied, not altered"
    );
}

/// Alias review/confirmation truth is never promoted by the bridge:
/// all review fields default to pending/false regardless of source shape.
#[test]
fn bridge_never_promotes_alias_review_truth() {
    let envelope = build_envelope(vec![ExportRecord::EntityAlias(ExportEntityAlias {
        canonical_entity_id: EntityId::new("entity-fence"),
        alias_text: "alias fence".into(),
        alias_source: "forge_extraction".into(),
        match_evidence: Some(serde_json::json!({"score": 0.99})),
        confidence: 0.99,
        scope: None,
        superseded_by_entity_id: None,
        split_from_entity_id: None,
    })]);
    let batch = transform_v1(envelope);
    let alias = batch
        .records
        .iter()
        .find_map(|record| match record {
            ImportProjectionRecord::EntityAlias(ea) => Some(ea),
            _ => None,
        })
        .expect("alias must be present");
    assert_eq!(alias.merge_decision, MergeDecision::PendingReview);
    assert_eq!(alias.review_state, ReviewState::PendingReview);
    assert!(!alias.is_human_confirmed);
    assert!(!alias.is_human_confirmed_final);
}

/// Confidence is copied verbatim from the export — never recalculated or
/// promoted by the bridge.
#[test]
fn bridge_copies_confidence_without_recalculation() {
    let envelope = build_envelope(vec![claim_record(Some("claim-fence-v3"), None, None, 0.42)]);
    let batch = transform_v1(envelope);
    let cv = find_claim(&batch, "claim-fence-v3");
    assert_eq!(cv.confidence, 0.42, "confidence must be copied verbatim");
    assert_eq!(cv.claim_state, ClaimState::Active);
}
