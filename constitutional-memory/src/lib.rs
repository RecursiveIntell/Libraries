//! Typed charter and archive surface crate for constitutional artifact families.
//!
//! The compatibility name stays in place, but the crate is not a hidden
//! governance runtime. It publishes typed constitutional surfaces and bounded
//! reconstruction checks for amendment and archive flows.
//!
//! ## Integration Points
//!
//! - **forge-pilot observe phase:** `governance_gate.rs` (`#[cfg(feature = "governance")]`) reads
//!   charter bundle publication status and doctrine snapshot advisory state from semantic-memory
//!   projections.
//! - **forge-pilot act phase:** amendment decision dispositions (blocked vs. approved) and
//!   archive manifest replay guarantees constrain constitutional change execution.
//! - **verification-control:** amendment proposal and amendment decision case types are affected;
//!   archive compaction triggers historical query guarantee verification cases.
//! - **Stack Arena scenarios:** governed lane exercises governance observation of charter
//!   bundle state and amendment decision dispositions.
//!
//! ## Artifact Families
//!
//! | Artifact | Schema Version | Owner |
//! |----------|---------------|-------|
//! | [`CharterBundleV1`] | `charter_bundle_v1` | this crate |
//! | [`DoctrineSnapshotV1`] | `doctrine_snapshot_v1` | this crate |
//! | [`AmendmentProposalV1`] | `amendment_proposal_v1` | this crate |
//! | [`AmendmentDecisionV1`] | `amendment_decision_v1` | this crate |
//! | [`ArchiveManifestV1`] | `archive_manifest_v1` | this crate |
//! | [`CompactionReceiptV1`] | `compaction_receipt_v1` | this crate |
//! | [`HistoricalQueryGuaranteeV1`] | `historical_query_guarantee_v1` | this crate |
//! | [`DeprecationBundleV1`] | `deprecation_bundle_v1` | this crate |
//! | [`RetirementBundleV1`] | `retirement_bundle_v1` | this crate |

pub mod error;
pub use error::*;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use stack_ids::{
    AmendmentDecisionId, AmendmentProposalId, ArchiveManifestId, CharterBundleId,
    CompactionReceiptId, DeprecationBundleId, DoctrineSnapshotId, HistoricalQueryGuaranteeId,
    RetirementBundleId, SemanticDiffId, SurfaceStatus,
};

pub const CHARTER_BUNDLE_V1_SCHEMA: &str = "charter_bundle_v1";
pub const DOCTRINE_SNAPSHOT_V1_SCHEMA: &str = "doctrine_snapshot_v1";
pub const AMENDMENT_PROPOSAL_V1_SCHEMA: &str = "amendment_proposal_v1";
pub const AMENDMENT_DECISION_V1_SCHEMA: &str = "amendment_decision_v1";
pub const ARCHIVE_MANIFEST_V1_SCHEMA: &str = "archive_manifest_v1";
pub const COMPACTION_RECEIPT_V1_SCHEMA: &str = "compaction_receipt_v1";
pub const HISTORICAL_QUERY_GUARANTEE_V1_SCHEMA: &str = "historical_query_guarantee_v1";
pub const DEPRECATION_BUNDLE_V1_SCHEMA: &str = "deprecation_bundle_v1";
pub const RETIREMENT_BUNDLE_V1_SCHEMA: &str = "retirement_bundle_v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct CharterBundleV1 {
    pub schema_version: String,
    pub charter_bundle_id: CharterBundleId,
    pub charter_name: String,
    pub canonical_owner: String,
    pub publication_status: SurfaceStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct DoctrineSnapshotV1 {
    pub schema_version: String,
    pub doctrine_snapshot_id: DoctrineSnapshotId,
    pub charter_bundle_id: CharterBundleId,
    pub doctrine_version: String,
    pub advisory_only: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct AmendmentProposalV1 {
    pub schema_version: String,
    pub amendment_proposal_id: AmendmentProposalId,
    pub charter_bundle_id: CharterBundleId,
    pub doctrine_snapshot_id: DoctrineSnapshotId,
    pub proposal_title: String,
    #[serde(default)]
    pub migration_obligations: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub semantic_diff_id: Option<SemanticDiffId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expected_archive_action: Option<String>,
    #[serde(default)]
    pub required_historical_query_modes: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rollback_handle: Option<String>,
    pub advisory_only: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AmendmentDisposition {
    ApprovedWithRollback,
    BlockedMissingRollback,
    BlockedMissingSemanticDiff,
    BlockedMissingArchiveGuarantee,
    AdvisoryOnly,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct AmendmentDecisionV1 {
    pub schema_version: String,
    pub amendment_decision_id: AmendmentDecisionId,
    pub amendment_proposal_id: AmendmentProposalId,
    pub disposition: AmendmentDisposition,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub semantic_diff_id: Option<SemanticDiffId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub archive_manifest_id: Option<ArchiveManifestId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub historical_query_guarantee_id: Option<HistoricalQueryGuaranteeId>,
    pub archive_consequence_summary: String,
    #[serde(default)]
    pub unresolved_obligations: Vec<String>,
    pub advisory_only: bool,
    pub rollback_ready: bool,
    pub decided_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ArchiveManifestV1 {
    pub schema_version: String,
    pub archive_manifest_id: ArchiveManifestId,
    pub charter_bundle_id: CharterBundleId,
    #[serde(default)]
    pub preserved_refs: Vec<String>,
    pub replay_guarantee: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct CompactionReceiptV1 {
    pub schema_version: String,
    pub compaction_receipt_id: CompactionReceiptId,
    pub archive_manifest_id: ArchiveManifestId,
    #[serde(default)]
    pub dropped_detail_refs: Vec<String>,
    pub queryable_degradation_note: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct HistoricalQueryGuaranteeV1 {
    pub schema_version: String,
    pub historical_query_guarantee_id: HistoricalQueryGuaranteeId,
    pub archive_manifest_id: ArchiveManifestId,
    pub guarantee_text: String,
    pub horizon_only: bool,
    #[serde(default)]
    pub guaranteed_query_modes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct DeprecationBundleV1 {
    pub schema_version: String,
    pub deprecation_bundle_id: DeprecationBundleId,
    pub charter_bundle_id: CharterBundleId,
    pub target_surface: String,
    pub replacement_surface: String,
    pub advisory_only: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct RetirementBundleV1 {
    pub schema_version: String,
    pub retirement_bundle_id: RetirementBundleId,
    pub deprecation_bundle_id: DeprecationBundleId,
    pub horizon_only: bool,
}

pub fn evaluate_amendment(
    proposal: &AmendmentProposalV1,
    migration_obligations_satisfied: bool,
    archive_manifest: Option<&ArchiveManifestV1>,
    historical_query_guarantee: Option<&HistoricalQueryGuaranteeV1>,
    decided_at: impl Into<String>,
) -> AmendmentDecisionV1 {
    let rollback_ready = proposal.rollback_handle.is_some();
    let semantic_diff_ready = proposal.semantic_diff_id.is_some();
    let archive_ready = archive_manifest.is_some() && historical_query_guarantee.is_some();
    let disposition = if proposal.advisory_only {
        AmendmentDisposition::AdvisoryOnly
    } else if !rollback_ready {
        AmendmentDisposition::BlockedMissingRollback
    } else if !semantic_diff_ready {
        AmendmentDisposition::BlockedMissingSemanticDiff
    } else if !archive_ready {
        AmendmentDisposition::BlockedMissingArchiveGuarantee
    } else if migration_obligations_satisfied {
        AmendmentDisposition::ApprovedWithRollback
    } else {
        AmendmentDisposition::BlockedMissingRollback
    };

    let mut unresolved_obligations =
        if matches!(disposition, AmendmentDisposition::ApprovedWithRollback) {
            vec![]
        } else {
            proposal.migration_obligations.clone()
        };
    if !rollback_ready {
        unresolved_obligations.push("rollback handle must be present".into());
    }
    if !semantic_diff_ready {
        unresolved_obligations.push("semantic diff linkage must remain explicit".into());
    }
    if !archive_ready {
        unresolved_obligations
            .push("archive manifest and historical query guarantee must be emitted".into());
    }

    AmendmentDecisionV1 {
        schema_version: AMENDMENT_DECISION_V1_SCHEMA.into(),
        amendment_decision_id: AmendmentDecisionId::generate(),
        amendment_proposal_id: proposal.amendment_proposal_id.clone(),
        disposition,
        semantic_diff_id: proposal.semantic_diff_id.clone(),
        archive_manifest_id: archive_manifest.map(|manifest| manifest.archive_manifest_id.clone()),
        historical_query_guarantee_id: historical_query_guarantee
            .map(|guarantee| guarantee.historical_query_guarantee_id.clone()),
        archive_consequence_summary: proposal
            .expected_archive_action
            .clone()
            .unwrap_or_else(|| "archive consequences were not described".into()),
        unresolved_obligations,
        advisory_only: !matches!(disposition, AmendmentDisposition::ApprovedWithRollback),
        rollback_ready,
        decided_at: decided_at.into(),
    }
}

pub fn evaluate_archive_compaction(
    charter_bundle_id: CharterBundleId,
    preserved_refs: Vec<String>,
    dropped_detail_refs: Vec<String>,
    guaranteed_query_modes: Vec<String>,
) -> (
    ArchiveManifestV1,
    HistoricalQueryGuaranteeV1,
    CompactionReceiptV1,
) {
    let archive_manifest = ArchiveManifestV1 {
        schema_version: ARCHIVE_MANIFEST_V1_SCHEMA.into(),
        archive_manifest_id: ArchiveManifestId::generate(),
        charter_bundle_id,
        preserved_refs,
        replay_guarantee: "historical replay remains reconstructable through the archive manifest"
            .into(),
    };
    let historical_query_guarantee = HistoricalQueryGuaranteeV1 {
        schema_version: HISTORICAL_QUERY_GUARANTEE_V1_SCHEMA.into(),
        historical_query_guarantee_id: HistoricalQueryGuaranteeId::generate(),
        archive_manifest_id: archive_manifest.archive_manifest_id.clone(),
        guarantee_text:
            "compaction may drop detail refs but historical constitutional queries stay answerable"
                .into(),
        horizon_only: false,
        guaranteed_query_modes,
    };
    let compaction_receipt = CompactionReceiptV1 {
        schema_version: COMPACTION_RECEIPT_V1_SCHEMA.into(),
        compaction_receipt_id: CompactionReceiptId::generate(),
        archive_manifest_id: archive_manifest.archive_manifest_id.clone(),
        dropped_detail_refs,
        queryable_degradation_note:
            "detail refs were compacted, but the archive manifest and historical query guarantee remain authoritative for replay".into(),
    };

    (
        archive_manifest,
        historical_query_guarantee,
        compaction_receipt,
    )
}
