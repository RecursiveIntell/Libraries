use crate::{ContextGovernorError, V3ProjectionError};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ContextGovernorFailureV1 {
    pub schema: String,
    pub operation: String,
    pub code: String,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub details: BTreeMap<String, Value>,
}

impl ContextGovernorFailureV1 {
    pub fn from_error(operation: impl Into<String>, error: &ContextGovernorError) -> Self {
        let mut details = BTreeMap::new();
        let code = match error {
            ContextGovernorError::V3Projection(error) => {
                let kind = match error {
                    V3ProjectionError::InvalidSource => "invalid_source",
                    V3ProjectionError::ExistingProjection => "existing_projection",
                    V3ProjectionError::OverlappingScope => "overlapping_scope",
                    V3ProjectionError::UnsafePath => "unsafe_path",
                    V3ProjectionError::ResourceLimit => "resource_limit",
                    V3ProjectionError::SourceChanged => "source_changed",
                    V3ProjectionError::SourceMismatch => "source_mismatch",
                    V3ProjectionError::UnsupportedContract => "unsupported_contract",
                    V3ProjectionError::InvalidOptions => "invalid_options",
                    V3ProjectionError::PublicationCollision => "publication_collision",
                    V3ProjectionError::LegacyAncestorUnsupported => "legacy_ancestor_unsupported",
                };
                details.insert("kind".into(), Value::String(kind.into()));
                "v3_projection_error"
            }
            ContextGovernorError::CanonicalActiveKeyMissing { .. } => {
                "canonical_active_key_missing"
            }
            ContextGovernorError::KeyUnreadable { .. } => "key_unreadable",
            ContextGovernorError::InvalidKeyLength { actual, .. } => {
                details.insert("actual".into(), Value::from(*actual));
                "invalid_key_length"
            }
            ContextGovernorError::InvalidKeyEncoding { .. } => "invalid_key_encoding",
            ContextGovernorError::InvalidKeyPermissions { .. } => "invalid_key_permissions",
            ContextGovernorError::WrongKeyOwner { .. } => "wrong_key_owner",
            ContextGovernorError::KeyPathEscape { .. } => "key_path_escape",
            ContextGovernorError::ComputedKeyIdMismatch { .. } => "computed_key_id_mismatch",
            ContextGovernorError::WrongConfiguredKeyId { .. } => "wrong_configured_key_id",
            ContextGovernorError::RequiredHistoricalKeyUnavailable { .. } => {
                "required_historical_key_unavailable"
            }
            ContextGovernorError::CompromisedKey { .. } => "compromised_key",
            ContextGovernorError::LegacyUnboundKeyUse { .. } => "legacy_unbound_key_use",
            ContextGovernorError::ConflictingActiveKeyState { .. } => {
                "conflicting_active_key_state"
            }
            ContextGovernorError::RollbackKeyMismatch { .. } => "rollback_key_mismatch",
            ContextGovernorError::ConfigurationPathOutsideCanonicalState { .. } => {
                "configuration_path_outside_canonical_state"
            }
            ContextGovernorError::EmptyMessages => "empty_messages",
            ContextGovernorError::Serialization(_) => "serialization_failed",
            ContextGovernorError::Io(error) => {
                details.insert(
                    "io_kind".into(),
                    Value::String(format!("{:?}", error.kind()).to_ascii_lowercase()),
                );
                "io_failed"
            }
            ContextGovernorError::ReceiptNotFound(_) => "receipt_not_found",
            ContextGovernorError::BudgetExceeded { target, actual } => {
                details.insert("target".into(), Value::from(*target));
                details.insert("actual".into(), Value::from(*actual));
                "budget_exceeded"
            }
            ContextGovernorError::CannotMeetTarget {
                target,
                minimum_safe,
                actual,
                reasons,
            } => {
                details.insert("target".into(), Value::from(*target));
                details.insert("minimum_safe".into(), Value::from(*minimum_safe));
                details.insert("actual".into(), Value::from(*actual));
                details.insert("reason_count".into(), Value::from(reasons.len()));
                "cannot_meet_target"
            }
            ContextGovernorError::Sqlite(_) => "sqlite_failed",
            ContextGovernorError::SummarySafetyFailed(_) => "summary_safety_failed",
            ContextGovernorError::SignedReceiptRequiresKey => "signed_receipt_requires_key",
            ContextGovernorError::CompactedTranscriptIntegrityMismatch { .. } => {
                "compacted_transcript_integrity_mismatch"
            }
            ContextGovernorError::ExactFallbackIntegrityMismatch { .. } => {
                "exact_fallback_integrity_mismatch"
            }
            ContextGovernorError::UnsupportedReceiptSchema(_) => "unsupported_receipt_schema",
            ContextGovernorError::ReceiptAlreadyExists(_) => "receipt_already_exists",
            ContextGovernorError::LineageIntegrityMismatch { .. } => "lineage_integrity_mismatch",
            ContextGovernorError::LineageMissingAncestor { .. } => "lineage_missing_ancestor",
            ContextGovernorError::AmbiguousLineageTip { receipt_ids, .. } => {
                details.insert("tip_count".into(), Value::from(receipt_ids.len()));
                "ambiguous_lineage_tip"
            }
            ContextGovernorError::AmbiguousLineageTarget(_) => "ambiguous_lineage_target",
            ContextGovernorError::ReceiptIntegrityMissing { .. } => "receipt_integrity_missing",
            ContextGovernorError::ReceiptIntegrityFailed { .. } => "receipt_integrity_failed",
            ContextGovernorError::ReceiptIntegrityUnavailable { .. } => {
                "receipt_integrity_unavailable"
            }
            ContextGovernorError::PendingReceiptNotFound(_) => "pending_receipt_not_found",
            ContextGovernorError::CommittedTranscriptMismatch(mismatch) => {
                details.insert(
                    "expected_count".into(),
                    Value::from(mismatch.expected_count),
                );
                details.insert("actual_count".into(), Value::from(mismatch.actual_count));
                "committed_transcript_mismatch"
            }
            ContextGovernorError::GenerationOverflow { .. } => "generation_overflow",
            ContextGovernorError::CompactionNoNetBenefit {
                before,
                after,
                minimum_savings,
            } => {
                details.insert("before".into(), Value::from(*before));
                details.insert("after".into(), Value::from(*after));
                details.insert("minimum_savings".into(), Value::from(*minimum_savings));
                "compaction_no_net_benefit"
            }
            ContextGovernorError::ProvenanceBudgetExceeded {
                actual_bytes,
                maximum_bytes,
            } => {
                details.insert("actual_bytes".into(), Value::from(*actual_bytes));
                details.insert("maximum_bytes".into(), Value::from(*maximum_bytes));
                "provenance_budget_exceeded"
            }
            ContextGovernorError::LineageGenerationLimit {
                generation,
                maximum_generation,
            } => {
                details.insert("generation".into(), Value::from(*generation));
                details.insert(
                    "maximum_generation".into(),
                    Value::from(*maximum_generation),
                );
                "lineage_generation_limit"
            }
            ContextGovernorError::LineageIndexRebuildRequired { .. } => {
                "lineage_index_rebuild_required"
            }
        };
        Self {
            schema: "ContextGovernorFailureV1".into(),
            operation: operation.into(),
            code: code.into(),
            details,
        }
    }
}
