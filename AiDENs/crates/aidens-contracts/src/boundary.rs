//! Boundary compiler and repair receipts for parser/treatment integrity.
//!
//! These are AiDENs-local receipt envelopes around boundary behavior. Canonical
//! schema and boundary semantics remain with their owner crates.

use super::*;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct BoundaryCompilerProfileV1 {
    pub profile_id: ArtifactId,
    pub language: String,
    pub dialect: String,
    pub schema_identity: String,
    pub duplicate_key_policy: String,
    pub unknown_field_policy: String,
    pub canonicalization_profile: String,
    pub repair_policy: String,
    pub treatment_integrity_required: bool,
    pub max_bytes: u64,
    pub max_depth: u32,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reason_codes: Vec<String>,
}

impl BoundaryCompilerProfileV1 {
    pub fn strict_json(schema_identity: impl Into<String>) -> Self {
        let schema_identity = schema_identity.into();
        Self {
            profile_id: generated_artifact_id_from_material(
                "boundary-profile",
                &format!("strict-json|{schema_identity}"),
            ),
            language: "json".into(),
            dialect: "json-schema-2020-12-subset".into(),
            schema_identity,
            duplicate_key_policy: "reject".into(),
            unknown_field_policy: "schema-controlled".into(),
            canonicalization_profile: "stack-ids-json-c14n-v1".into(),
            repair_policy: "receipt-required-no-treatment-change".into(),
            treatment_integrity_required: true,
            max_bytes: 1_048_576,
            max_depth: 64,
            reason_codes: vec!["boundary-profile-strict-json".into()],
        }
    }

    pub fn rejects_duplicate_keys(&self) -> bool {
        self.duplicate_key_policy == "reject"
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct TreatmentIntegrityReceiptV1 {
    pub receipt_id: ArtifactId,
    pub profile_id: ArtifactId,
    pub before_digest: DisplayDigestV1,
    pub after_digest: DisplayDigestV1,
    pub treatment_critical_fields: Vec<String>,
    pub treatment_changed: bool,
    pub hard_failed: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reason_codes: Vec<String>,
    pub recorded_at: DateTime<Utc>,
}

impl TreatmentIntegrityReceiptV1 {
    pub fn compare(
        profile: &BoundaryCompilerProfileV1,
        before: &serde_json::Value,
        after: &serde_json::Value,
        treatment_critical_fields: Vec<String>,
    ) -> Self {
        let treatment_changed = treatment_critical_fields
            .iter()
            .any(|field| before.pointer(field) != after.pointer(field));
        let hard_failed = treatment_changed && profile.treatment_integrity_required;
        let material = serde_json::json!({
            "profile": profile.profile_id,
            "before": before,
            "after": after,
            "fields": treatment_critical_fields,
        });
        Self {
            receipt_id: generated_artifact_id_from_material(
                "treatment-integrity",
                &non_authoritative_json_display_digest(&material),
            ),
            profile_id: profile.profile_id.clone(),
            before_digest: DisplayDigestV1::for_json_value(before),
            after_digest: DisplayDigestV1::for_json_value(after),
            treatment_critical_fields,
            treatment_changed,
            hard_failed,
            reason_codes: if hard_failed {
                vec!["treatment-integrity-hard-fail".into()]
            } else if treatment_changed {
                vec!["treatment-integrity-degraded".into()]
            } else {
                vec!["treatment-integrity-preserved".into()]
            },
            recorded_at: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct BoundaryRepairReceiptV1 {
    pub receipt_id: ArtifactId,
    pub profile_id: ArtifactId,
    pub repair_kind: String,
    pub before_digest: DisplayDigestV1,
    pub after_digest: DisplayDigestV1,
    pub treatment_integrity_receipt_id: ArtifactId,
    pub accepted: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reason_codes: Vec<String>,
    pub recorded_at: DateTime<Utc>,
}

impl BoundaryRepairReceiptV1 {
    pub fn new(
        profile: &BoundaryCompilerProfileV1,
        repair_kind: impl Into<String>,
        before: &serde_json::Value,
        after: &serde_json::Value,
        treatment: &TreatmentIntegrityReceiptV1,
    ) -> Self {
        let repair_kind = repair_kind.into();
        let accepted = !treatment.hard_failed;
        let material = format!(
            "{}|{}|{}",
            profile.profile_id.0, repair_kind, treatment.receipt_id.0
        );
        Self {
            receipt_id: generated_artifact_id_from_material("boundary-repair", &material),
            profile_id: profile.profile_id.clone(),
            repair_kind,
            before_digest: DisplayDigestV1::for_json_value(before),
            after_digest: DisplayDigestV1::for_json_value(after),
            treatment_integrity_receipt_id: treatment.receipt_id.clone(),
            accepted,
            reason_codes: if accepted {
                vec!["boundary-repair-receipted".into()]
            } else {
                vec!["boundary-repair-rejected-treatment-change".into()]
            },
            recorded_at: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct BoundaryCompileReceiptV1 {
    pub receipt_id: ArtifactId,
    pub profile_id: ArtifactId,
    pub input_digest: DisplayDigestV1,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_digest: Option<DisplayDigestV1>,
    pub accepted: bool,
    pub duplicate_keys_rejected: bool,
    pub schema_validated: bool,
    pub canonicalized: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repair_receipt_id: Option<ArtifactId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub treatment_integrity_receipt_id: Option<ArtifactId>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reason_codes: Vec<String>,
    pub recorded_at: DateTime<Utc>,
}

impl BoundaryCompileReceiptV1 {
    pub fn accepted(
        profile: &BoundaryCompilerProfileV1,
        input: &serde_json::Value,
        output: &serde_json::Value,
        repair_receipt_id: Option<ArtifactId>,
        treatment_integrity_receipt_id: Option<ArtifactId>,
    ) -> Self {
        let input_digest = DisplayDigestV1::for_json_value(input);
        let output_digest = DisplayDigestV1::for_json_value(output);
        let material = format!(
            "{}|{}|{}",
            profile.profile_id.0, input_digest.digest, output_digest.digest
        );
        Self {
            receipt_id: generated_artifact_id_from_material("boundary-compile", &material),
            profile_id: profile.profile_id.clone(),
            input_digest,
            output_digest: Some(output_digest),
            accepted: true,
            duplicate_keys_rejected: profile.rejects_duplicate_keys(),
            schema_validated: true,
            canonicalized: true,
            repair_receipt_id,
            treatment_integrity_receipt_id,
            reason_codes: vec!["boundary-compile-accepted".into()],
            recorded_at: Utc::now(),
        }
    }

    pub fn rejected_duplicate_key(
        profile: &BoundaryCompilerProfileV1,
        input: &serde_json::Value,
    ) -> Self {
        let input_digest = DisplayDigestV1::for_json_value(input);
        let material = format!(
            "{}|{}|duplicate-key",
            profile.profile_id.0, input_digest.digest
        );
        Self {
            receipt_id: generated_artifact_id_from_material("boundary-compile", &material),
            profile_id: profile.profile_id.clone(),
            input_digest,
            output_digest: None,
            accepted: false,
            duplicate_keys_rejected: true,
            schema_validated: false,
            canonicalized: false,
            repair_receipt_id: None,
            treatment_integrity_receipt_id: None,
            reason_codes: vec!["duplicate-json-object-key".into()],
            recorded_at: Utc::now(),
        }
    }
}
