use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct VendorCertificationAdapterV1 {
    pub schema_version: String,
    pub vendor_certification_adapter_id: String,
    pub vendor_name: String,
    pub product_surface: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub covered_artifact_families: Vec<String>,
    pub translation_mode: String,
    pub support_window: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct VendorEvidenceTranslationV1 {
    pub schema_version: String,
    pub vendor_evidence_translation_id: String,
    pub vendor_certification_adapter_id: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub source_shapes: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub canonical_targets: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub lossy_fields: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub required_caveats: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct VendorTrustRootBindingV1 {
    pub schema_version: String,
    pub vendor_trust_root_binding_id: String,
    pub vendor_certification_adapter_id: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub trust_root_refs: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub signer_classes: Vec<String>,
    pub rotation_channel: String,
    pub revocation_channel: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct VendorRevocationHandlingV1 {
    pub schema_version: String,
    pub vendor_revocation_handling_id: String,
    pub vendor_certification_adapter_id: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub revocation_inputs: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub local_invalidation_actions: Vec<String>,
    pub replay_impact: String,
    pub admission_impact: String,
}
