use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct TransparencyReceiptV1 {
    pub schema_version: String,
    pub transparency_receipt_id: String,
    pub attestation_envelope_id: String,
    pub registry_identity: String,
    pub inclusion_material: String,
    pub recorded_time: String,
    pub admissibility_judgment: String,
}
