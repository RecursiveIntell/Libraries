//! Property-based tests for constitutional-memory artifact types.
//!
//! Validates serde roundtrip fidelity and schema_version constant usage
//! for top-level artifact families.

use constitutional_memory::*;
use proptest::prelude::*;
use stack_ids::{
    AmendmentDecisionId, AmendmentProposalId, ArchiveManifestId, CharterBundleId,
    CompactionReceiptId, DeprecationBundleId, DoctrineSnapshotId, HistoricalQueryGuaranteeId,
    RetirementBundleId, SemanticDiffId, SurfaceStatus,
};

// --- Generators ---

fn arb_string() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9_-]{0,30}".prop_map(|s| s)
}

fn arb_id() -> impl Strategy<Value = String> {
    "[a-z]{3}-[a-z0-9]{8}".prop_map(|s| s)
}

fn arb_vec_string() -> impl Strategy<Value = Vec<String>> {
    prop::collection::vec(arb_string(), 0..3)
}

fn arb_surface_status() -> impl Strategy<Value = SurfaceStatus> {
    prop_oneof![
        Just(SurfaceStatus::AdvisoryOnly),
        Just(SurfaceStatus::NonAdmitted),
        Just(SurfaceStatus::Degraded),
        Just(SurfaceStatus::HorizonOnly),
    ]
}

fn arb_charter_bundle_id() -> impl Strategy<Value = CharterBundleId> {
    arb_id().prop_map(CharterBundleId::new)
}

fn arb_doctrine_snapshot_id() -> impl Strategy<Value = DoctrineSnapshotId> {
    arb_id().prop_map(DoctrineSnapshotId::new)
}

fn arb_amendment_proposal_id() -> impl Strategy<Value = AmendmentProposalId> {
    arb_id().prop_map(AmendmentProposalId::new)
}

fn arb_amendment_decision_id() -> impl Strategy<Value = AmendmentDecisionId> {
    arb_id().prop_map(AmendmentDecisionId::new)
}

fn arb_archive_manifest_id() -> impl Strategy<Value = ArchiveManifestId> {
    arb_id().prop_map(ArchiveManifestId::new)
}

fn arb_compaction_receipt_id() -> impl Strategy<Value = CompactionReceiptId> {
    arb_id().prop_map(CompactionReceiptId::new)
}

fn arb_historical_query_guarantee_id() -> impl Strategy<Value = HistoricalQueryGuaranteeId> {
    arb_id().prop_map(HistoricalQueryGuaranteeId::new)
}

fn arb_deprecation_bundle_id() -> impl Strategy<Value = DeprecationBundleId> {
    arb_id().prop_map(DeprecationBundleId::new)
}

fn arb_retirement_bundle_id() -> impl Strategy<Value = RetirementBundleId> {
    arb_id().prop_map(RetirementBundleId::new)
}

fn arb_semantic_diff_id() -> impl Strategy<Value = SemanticDiffId> {
    arb_id().prop_map(SemanticDiffId::new)
}

fn arb_amendment_disposition() -> impl Strategy<Value = AmendmentDisposition> {
    prop_oneof![
        Just(AmendmentDisposition::ApprovedWithRollback),
        Just(AmendmentDisposition::BlockedMissingRollback),
        Just(AmendmentDisposition::BlockedMissingSemanticDiff),
        Just(AmendmentDisposition::BlockedMissingArchiveGuarantee),
        Just(AmendmentDisposition::AdvisoryOnly),
    ]
}

// --- Top-level artifact generators ---

fn arb_charter_bundle_v1() -> impl Strategy<Value = CharterBundleV1> {
    (
        arb_charter_bundle_id(),
        arb_string(),
        arb_string(),
        arb_surface_status(),
    )
        .prop_map(
            |(charter_bundle_id, charter_name, canonical_owner, publication_status)| {
                CharterBundleV1 {
                    schema_version: CHARTER_BUNDLE_V1_SCHEMA.into(),
                    charter_bundle_id,
                    charter_name,
                    canonical_owner,
                    publication_status,
                }
            },
        )
}

fn arb_amendment_decision_v1() -> impl Strategy<Value = AmendmentDecisionV1> {
    (
        arb_amendment_decision_id(),
        arb_amendment_proposal_id(),
        arb_amendment_disposition(),
        prop::option::of(arb_semantic_diff_id()),
        prop::option::of(arb_archive_manifest_id()),
        prop::option::of(arb_historical_query_guarantee_id()),
        arb_string(),
        arb_vec_string(),
        prop::bool::ANY,
        prop::bool::ANY,
        arb_string(),
    )
        .prop_map(
            |(
                amendment_decision_id,
                amendment_proposal_id,
                disposition,
                semantic_diff_id,
                archive_manifest_id,
                historical_query_guarantee_id,
                archive_consequence_summary,
                unresolved_obligations,
                advisory_only,
                rollback_ready,
                decided_at,
            )| {
                AmendmentDecisionV1 {
                    schema_version: AMENDMENT_DECISION_V1_SCHEMA.into(),
                    amendment_decision_id,
                    amendment_proposal_id,
                    disposition,
                    semantic_diff_id,
                    archive_manifest_id,
                    historical_query_guarantee_id,
                    archive_consequence_summary,
                    unresolved_obligations,
                    advisory_only,
                    rollback_ready,
                    decided_at,
                }
            },
        )
}

fn arb_archive_manifest_v1() -> impl Strategy<Value = ArchiveManifestV1> {
    (
        arb_archive_manifest_id(),
        arb_charter_bundle_id(),
        arb_vec_string(),
        arb_string(),
    )
        .prop_map(
            |(archive_manifest_id, charter_bundle_id, preserved_refs, replay_guarantee)| {
                ArchiveManifestV1 {
                    schema_version: ARCHIVE_MANIFEST_V1_SCHEMA.into(),
                    archive_manifest_id,
                    charter_bundle_id,
                    preserved_refs,
                    replay_guarantee,
                }
            },
        )
}

// --- Property tests ---

proptest! {
    #[test]
    fn charter_bundle_v1_serde_roundtrip(val in arb_charter_bundle_v1()) {
        let json = serde_json::to_string(&val).unwrap();
        let back: CharterBundleV1 = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(&val, &back);
    }

    #[test]
    fn charter_bundle_v1_schema_version_constant(val in arb_charter_bundle_v1()) {
        prop_assert_eq!(val.schema_version.as_str(), CHARTER_BUNDLE_V1_SCHEMA);
    }

    #[test]
    fn amendment_decision_v1_serde_roundtrip(val in arb_amendment_decision_v1()) {
        let json = serde_json::to_string(&val).unwrap();
        let back: AmendmentDecisionV1 = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(&val, &back);
    }

    #[test]
    fn amendment_decision_v1_schema_version_constant(val in arb_amendment_decision_v1()) {
        prop_assert_eq!(val.schema_version.as_str(), AMENDMENT_DECISION_V1_SCHEMA);
    }

    #[test]
    fn archive_manifest_v1_serde_roundtrip(val in arb_archive_manifest_v1()) {
        let json = serde_json::to_string(&val).unwrap();
        let back: ArchiveManifestV1 = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(&val, &back);
    }

    #[test]
    fn archive_manifest_v1_schema_version_constant(val in arb_archive_manifest_v1()) {
        prop_assert_eq!(val.schema_version.as_str(), ARCHIVE_MANIFEST_V1_SCHEMA);
    }
}
