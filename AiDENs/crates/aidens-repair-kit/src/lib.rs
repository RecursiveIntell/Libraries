//! Thin repair facade over canonical verification and Forge repair artifacts.

pub mod canonical_stack {
    pub use semantic_memory_forge::{RetractionRecordV1, RETRACTION_RECORD_V1_SCHEMA};
    pub use verification_control::{
        BoundaryArtifactKind, BoundaryRepairRecord, BOUNDARY_REPAIR_RECORD_V1_SCHEMA,
    };
}

pub use canonical_stack::BoundaryRepairRecord as CanonicalBoundaryRepairRecord;
pub use canonical_stack::RetractionRecordV1 as CanonicalRetractionRecordV1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoundaryRepairAdmissionDisposition {
    Accepted,
    Rejected,
    Quarantined,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoundaryRepairAdmissionReceipt {
    pub disposition: BoundaryRepairAdmissionDisposition,
    pub accepted: bool,
    pub replay_required: bool,
    pub schema_version: String,
    pub reason_codes: Vec<String>,
}

impl BoundaryRepairAdmissionReceipt {
    pub fn accepted(record: &CanonicalBoundaryRepairRecord) -> Self {
        Self {
            disposition: BoundaryRepairAdmissionDisposition::Accepted,
            accepted: true,
            replay_required: true,
            schema_version: record.schema_version.clone(),
            reason_codes: vec![
                "boundary-repair-record-admitted".into(),
                "repair-replay-required".into(),
            ],
        }
    }

    pub fn rejected(reason: impl Into<String>) -> Self {
        Self {
            disposition: BoundaryRepairAdmissionDisposition::Rejected,
            accepted: false,
            replay_required: true,
            schema_version: canonical_stack::BOUNDARY_REPAIR_RECORD_V1_SCHEMA.into(),
            reason_codes: vec!["boundary-repair-record-rejected".into(), reason.into()],
        }
    }

    pub fn quarantined(reason: impl Into<String>) -> Self {
        Self {
            disposition: BoundaryRepairAdmissionDisposition::Quarantined,
            accepted: false,
            replay_required: true,
            schema_version: canonical_stack::BOUNDARY_REPAIR_RECORD_V1_SCHEMA.into(),
            reason_codes: vec!["boundary-repair-record-quarantined".into(), reason.into()],
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CanonicalRepairAdapter;

impl CanonicalRepairAdapter {
    #[allow(clippy::too_many_arguments)]
    pub fn boundary_repair_record(
        &self,
        artifact_kind: canonical_stack::BoundaryArtifactKind,
        artifact_schema_version: impl Into<String>,
        repair_kind: impl Into<String>,
        field_path: impl Into<String>,
        original_value: Option<serde_json::Value>,
        repaired_value: serde_json::Value,
        rationale: impl Into<String>,
    ) -> CanonicalBoundaryRepairRecord {
        canonical_stack::BoundaryRepairRecord::new(
            artifact_kind,
            artifact_schema_version,
            repair_kind,
            field_path,
            original_value,
            repaired_value,
            rationale,
        )
    }

    pub fn validate_retraction(&self, record: &CanonicalRetractionRecordV1) -> Result<(), String> {
        record.validate()
    }

    pub fn admit_boundary_repair_record(
        &self,
        record: &CanonicalBoundaryRepairRecord,
    ) -> BoundaryRepairAdmissionReceipt {
        if record.schema_version != canonical_stack::BOUNDARY_REPAIR_RECORD_V1_SCHEMA {
            return BoundaryRepairAdmissionReceipt::rejected("schema-version-mismatch");
        }
        if record.field_path.trim().is_empty() || record.rationale.trim().is_empty() {
            return BoundaryRepairAdmissionReceipt::quarantined(
                "repair-record-missing-material-field",
            );
        }
        BoundaryRepairAdmissionReceipt::accepted(record)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_canonical_boundary_repair_record() {
        let record = CanonicalRepairAdapter.boundary_repair_record(
            canonical_stack::BoundaryArtifactKind::ControlReceipt,
            "control_receipt_v1",
            "field_normalization",
            "$.actor",
            Some(serde_json::json!("")),
            serde_json::json!("forge-pilot"),
            "canonical verification-control repair record",
        );

        assert_eq!(
            record.schema_version,
            canonical_stack::BOUNDARY_REPAIR_RECORD_V1_SCHEMA
        );
    }

    #[test]
    fn boundary_repair_admission_accepts_rejects_and_quarantines() {
        let accepted = CanonicalRepairAdapter.boundary_repair_record(
            canonical_stack::BoundaryArtifactKind::ControlReceipt,
            "control_receipt_v1",
            "field_normalization",
            "$.actor",
            Some(serde_json::json!("")),
            serde_json::json!("forge-pilot"),
            "canonical verification-control repair record",
        );
        let accepted_receipt = CanonicalRepairAdapter.admit_boundary_repair_record(&accepted);
        assert_eq!(
            accepted_receipt.disposition,
            BoundaryRepairAdmissionDisposition::Accepted
        );
        assert!(accepted_receipt.accepted);
        assert!(accepted_receipt.replay_required);

        let mut rejected = accepted.clone();
        rejected.schema_version = "wrong-schema".into();
        let rejected_receipt = CanonicalRepairAdapter.admit_boundary_repair_record(&rejected);
        assert_eq!(
            rejected_receipt.disposition,
            BoundaryRepairAdmissionDisposition::Rejected
        );
        assert!(!rejected_receipt.accepted);

        let mut quarantined = accepted;
        quarantined.field_path = String::new();
        let quarantined_receipt = CanonicalRepairAdapter.admit_boundary_repair_record(&quarantined);
        assert_eq!(
            quarantined_receipt.disposition,
            BoundaryRepairAdmissionDisposition::Quarantined
        );
        assert!(!quarantined_receipt.accepted);
        assert!(quarantined_receipt
            .reason_codes
            .contains(&"repair-record-missing-material-field".into()));
    }
}
