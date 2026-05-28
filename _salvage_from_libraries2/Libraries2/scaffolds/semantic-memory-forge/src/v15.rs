//! Draft v15 artifact family scaffold.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct AttestationEnvelopeV1 {
    pub schema_version: String,
    pub attestation_envelope_id: String,
    pub artifact_family: String,
    pub artifact_version: String,
    pub content_digest: String,
    pub schema_identity: String,
    pub signer_identity: String,
    pub signing_time: String,
    pub trust_root_set_id: String,
    pub provenance_summary: String,
    pub disclosure_policy_id: String,
    pub replayability_class: String,
    pub revocation_refs: Vec<String>,
    pub supersession_refs: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct TrustRootSetV1 {
    pub schema_version: String,
    pub trust_root_set_id: String,
    pub trust_root_identities: Vec<String>,
    pub allowed_signer_classes: Vec<String>,
    pub expiration_policy: String,
    pub rotation_policy: String,
    pub allowed_artifact_families: Vec<String>,
    pub revocation_sources: Vec<String>,
    pub policy_owner_refs: Vec<String>,
}

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

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct AttestationRevocationV1 {
    pub schema_version: String,
    pub attestation_revocation_id: String,
    pub affected_refs: Vec<String>,
    pub revocation_reason: String,
    pub effective_time: String,
    pub blast_radius: String,
    pub required_local_invalidation_behavior: String,
    pub dispute_linkage: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct AttestationSupersessionV1 {
    pub schema_version: String,
    pub attestation_supersession_id: String,
    pub prior_ref: String,
    pub replacement_ref: String,
    pub semantic_delta_summary: String,
    pub effective_time: String,
    pub replay_impact: String,
    pub requires_re_admission: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct RemoteOracleLeaseV1 {
    pub schema_version: String,
    pub remote_oracle_lease_id: String,
    pub oracle_identity: String,
    pub allowed_artifact_families: Vec<String>,
    pub allowed_graph_or_slice_kinds: Vec<String>,
    pub exactness_class_ceiling: String,
    pub budget_ceiling: String,
    pub disclosure_ceiling: String,
    pub replay_obligation: String,
    pub lease_expiry: String,
    pub policy_owner_refs: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct RemoteSliceRequestV1 {
    pub schema_version: String,
    pub remote_slice_request_id: String,
    pub requested_slice_definition: String,
    pub required_artifact_refs: Vec<String>,
    pub allowed_disclosure_policy: String,
    pub exactness_target: String,
    pub trust_root_set_id: String,
    pub challenge_expectations: Vec<String>,
    pub remote_oracle_lease_id: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct RemoteSliceResultV1 {
    pub schema_version: String,
    pub remote_slice_result_id: String,
    pub remote_slice_request_id: String,
    pub returned_artifact_refs: Vec<String>,
    pub exactness_class: String,
    pub remote_execution_evidence: String,
    pub disclosure_markers: Vec<String>,
    pub replay_handle: String,
    pub local_admission_recommendation: String,
    pub attestation_envelope_id: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct CrossRuntimeReplayTicketV1 {
    pub schema_version: String,
    pub cross_runtime_replay_ticket_id: String,
    pub artifact_refs: Vec<String>,
    pub time_coordinates: String,
    pub required_trust_roots: Vec<String>,
    pub allowed_disclosure: String,
    pub lease_window: String,
    pub replay_expectations: Vec<String>,
    pub failure_behavior: String,
}
