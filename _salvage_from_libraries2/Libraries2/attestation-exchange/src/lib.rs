use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use stack_ids::{
    ArtifactAdmissionPolicyId, AttestationEnvelopeId, ContentDigest, DisclosurePolicyId,
    TransparencyReceiptId, TrustRootSetId,
};

pub mod profile_p6_vendor;
pub use profile_p6_vendor::*;

pub const ATTESTATION_ENVELOPE_V1_SCHEMA: &str = "attestation_envelope_v1";
pub const TRUST_ROOT_SET_V1_SCHEMA: &str = "trust_root_set_v1";
pub const TRANSPARENCY_RECEIPT_V1_SCHEMA: &str = "transparency_receipt_v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct AttestationEnvelopeV1 {
    pub schema_version: String,
    pub attestation_envelope_id: AttestationEnvelopeId,
    pub artifact_family: String,
    pub artifact_version: String,
    pub content_digest: ContentDigest,
    pub schema_identity: String,
    pub signer_identity: String,
    pub signing_time: String,
    pub trust_root_set_id: TrustRootSetId,
    pub provenance_summary: String,
    pub disclosure_policy_id: DisclosurePolicyId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub artifact_admission_policy_id: Option<ArtifactAdmissionPolicyId>,
    pub replayability_class: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub revocation_refs: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub supersession_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct TrustRootSetV1 {
    pub schema_version: String,
    pub trust_root_set_id: TrustRootSetId,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub trust_root_identities: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub allowed_signer_classes: Vec<String>,
    pub expiration_policy: String,
    pub rotation_policy: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub allowed_artifact_families: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub revocation_sources: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub policy_owner_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct TransparencyReceiptV1 {
    pub schema_version: String,
    pub transparency_receipt_id: TransparencyReceiptId,
    pub attestation_envelope_id: AttestationEnvelopeId,
    pub registry_identity: String,
    pub inclusion_material: String,
    pub recorded_time: String,
    pub admissibility_judgment: String,
}
