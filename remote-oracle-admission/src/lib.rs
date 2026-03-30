//! Typed remote-oracle admission surface crate for lease, slice, replay, and
//! re-admission artifacts.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use stack_ids::{
    AttestationEnvelopeId, AttestationRevocationId, AttestationSupersessionId,
    CrossRuntimeReplayTicketId, DisputeBundleId, RemoteOracleLeaseId, RemoteSliceRequestId,
    RemoteSliceResultId, TrustRootSetId,
};

pub use stack_ids::V25ConstitutionCitation;

pub const ATTESTATION_REVOCATION_V1_SCHEMA: &str = "attestation_revocation_v1";
pub const ATTESTATION_SUPERSESSION_V1_SCHEMA: &str = "attestation_supersession_v1";
pub const REMOTE_ORACLE_LEASE_V1_SCHEMA: &str = "remote_oracle_lease_v1";
pub const REMOTE_SLICE_REQUEST_V1_SCHEMA: &str = "remote_slice_request_v1";
pub const REMOTE_SLICE_RESULT_V1_SCHEMA: &str = "remote_slice_result_v1";
pub const CROSS_RUNTIME_REPLAY_TICKET_V1_SCHEMA: &str = "cross_runtime_replay_ticket_v1";

// Canonical v25 citations embed ApplicabilityContextId, ProfileSetId,
// CompositionReceiptId, EffectiveConstitutionId, and CompiledObligationSetId.

fn require_non_empty(value: &str, field: &'static str) -> Result<(), &'static str> {
    if value.trim().is_empty() {
        return Err(field);
    }
    Ok(())
}

fn require_non_empty_slice<T>(values: &[T], field: &'static str) -> Result<(), &'static str> {
    if values.is_empty() {
        return Err(field);
    }
    Ok(())
}

fn require_schema_version(
    found: &str,
    expected: &'static str,
    field: &'static str,
) -> Result<(), &'static str> {
    if found != expected {
        return Err(field);
    }
    Ok(())
}

fn require_citation(citation: &V25ConstitutionCitation) -> Result<(), &'static str> {
    require_non_empty(
        citation.applicability_context_id.as_str(),
        "applicability_context_id",
    )?;
    require_non_empty(citation.profile_set_id.as_str(), "profile_set_id")?;
    require_non_empty(
        citation.composition_receipt_id.as_str(),
        "composition_receipt_id",
    )?;
    require_non_empty(
        citation.effective_constitution_id.as_str(),
        "effective_constitution_id",
    )?;
    require_non_empty(
        citation.compiled_obligation_set_id.as_str(),
        "compiled_obligation_set_id",
    )?;
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RemoteExactnessClassV1 {
    Exact,
    BoundedExact,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum RemoteDisclosureClassV1 {
    #[serde(rename = "non-sensitive")]
    NonSensitive,
    #[serde(rename = "redacted_structured_only")]
    RedactedStructuredOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RemoteReplayObligationV1 {
    MustReturnReplayTicketOrNonreplayableReason,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum AttestationReplayImpactV1 {
    #[serde(rename = "replay ticket unchanged")]
    ReplayTicketUnchanged,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum LocalAdmissionRecommendationV1 {
    Eligible,
    AdmitIfTransparencyReceiptPresent,
    AdmitWithDisclosureConstraints,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ReplayFailureBehaviorV1 {
    Retry,
    DowngradeToAdvisoryAndEmitDisputeIfMandatory,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct AttestationRevocationV1 {
    pub schema_version: String,
    pub attestation_revocation_id: AttestationRevocationId,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub affected_refs: Vec<String>,
    pub revocation_reason: String,
    pub effective_time: String,
    pub blast_radius: String,
    pub required_local_invalidation_behavior: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dispute_linkage: Option<DisputeBundleId>,
}

impl AttestationRevocationV1 {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        attestation_revocation_id: AttestationRevocationId,
        affected_refs: Vec<String>,
        revocation_reason: impl Into<String>,
        effective_time: impl Into<String>,
        blast_radius: impl Into<String>,
        required_local_invalidation_behavior: impl Into<String>,
        dispute_linkage: Option<DisputeBundleId>,
    ) -> Result<Self, &'static str> {
        let value = Self {
            schema_version: ATTESTATION_REVOCATION_V1_SCHEMA.to_string(),
            attestation_revocation_id,
            affected_refs,
            revocation_reason: revocation_reason.into(),
            effective_time: effective_time.into(),
            blast_radius: blast_radius.into(),
            required_local_invalidation_behavior: required_local_invalidation_behavior.into(),
            dispute_linkage,
        };
        value.validate()?;
        Ok(value)
    }

    pub fn validate(&self) -> Result<(), &'static str> {
        require_schema_version(
            &self.schema_version,
            ATTESTATION_REVOCATION_V1_SCHEMA,
            "schema_version",
        )?;
        require_non_empty(
            self.attestation_revocation_id.as_str(),
            "attestation_revocation_id",
        )?;
        require_non_empty(&self.revocation_reason, "revocation_reason")?;
        require_non_empty(&self.effective_time, "effective_time")?;
        require_non_empty(&self.blast_radius, "blast_radius")?;
        require_non_empty(
            &self.required_local_invalidation_behavior,
            "required_local_invalidation_behavior",
        )?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct AttestationSupersessionV1 {
    pub schema_version: String,
    pub attestation_supersession_id: AttestationSupersessionId,
    pub prior_ref: String,
    pub replacement_ref: String,
    pub semantic_delta_summary: String,
    pub effective_time: String,
    pub replay_impact: AttestationReplayImpactV1,
    pub requires_re_admission: bool,
}

impl AttestationSupersessionV1 {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        attestation_supersession_id: AttestationSupersessionId,
        prior_ref: impl Into<String>,
        replacement_ref: impl Into<String>,
        semantic_delta_summary: impl Into<String>,
        effective_time: impl Into<String>,
        replay_impact: AttestationReplayImpactV1,
        requires_re_admission: bool,
    ) -> Result<Self, &'static str> {
        let value = Self {
            schema_version: ATTESTATION_SUPERSESSION_V1_SCHEMA.to_string(),
            attestation_supersession_id,
            prior_ref: prior_ref.into(),
            replacement_ref: replacement_ref.into(),
            semantic_delta_summary: semantic_delta_summary.into(),
            effective_time: effective_time.into(),
            replay_impact,
            requires_re_admission,
        };
        value.validate()?;
        Ok(value)
    }

    pub fn validate(&self) -> Result<(), &'static str> {
        require_schema_version(
            &self.schema_version,
            ATTESTATION_SUPERSESSION_V1_SCHEMA,
            "schema_version",
        )?;
        require_non_empty(
            self.attestation_supersession_id.as_str(),
            "attestation_supersession_id",
        )?;
        require_non_empty(&self.prior_ref, "prior_ref")?;
        require_non_empty(&self.replacement_ref, "replacement_ref")?;
        require_non_empty(&self.semantic_delta_summary, "semantic_delta_summary")?;
        require_non_empty(&self.effective_time, "effective_time")?;
        if self.prior_ref == self.replacement_ref {
            return Err("replacement_ref");
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct RemoteOracleLeaseV1 {
    pub schema_version: String,
    pub remote_oracle_lease_id: RemoteOracleLeaseId,
    pub oracle_identity: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub allowed_artifact_families: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub allowed_graph_or_slice_kinds: Vec<String>,
    pub exactness_class_ceiling: RemoteExactnessClassV1,
    pub budget_ceiling: String,
    pub disclosure_ceiling: RemoteDisclosureClassV1,
    pub replay_obligation: RemoteReplayObligationV1,
    pub lease_expiry: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub policy_owner_refs: Vec<String>,
}

impl RemoteOracleLeaseV1 {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        remote_oracle_lease_id: RemoteOracleLeaseId,
        oracle_identity: impl Into<String>,
        allowed_artifact_families: Vec<String>,
        allowed_graph_or_slice_kinds: Vec<String>,
        exactness_class_ceiling: RemoteExactnessClassV1,
        budget_ceiling: impl Into<String>,
        disclosure_ceiling: RemoteDisclosureClassV1,
        replay_obligation: RemoteReplayObligationV1,
        lease_expiry: impl Into<String>,
        policy_owner_refs: Vec<String>,
    ) -> Result<Self, &'static str> {
        let value = Self {
            schema_version: REMOTE_ORACLE_LEASE_V1_SCHEMA.to_string(),
            remote_oracle_lease_id,
            oracle_identity: oracle_identity.into(),
            allowed_artifact_families,
            allowed_graph_or_slice_kinds,
            exactness_class_ceiling,
            budget_ceiling: budget_ceiling.into(),
            disclosure_ceiling,
            replay_obligation,
            lease_expiry: lease_expiry.into(),
            policy_owner_refs,
        };
        value.validate()?;
        Ok(value)
    }

    pub fn validate(&self) -> Result<(), &'static str> {
        require_schema_version(
            &self.schema_version,
            REMOTE_ORACLE_LEASE_V1_SCHEMA,
            "schema_version",
        )?;
        require_non_empty(
            self.remote_oracle_lease_id.as_str(),
            "remote_oracle_lease_id",
        )?;
        require_non_empty(&self.oracle_identity, "oracle_identity")?;
        require_non_empty_slice(&self.allowed_artifact_families, "allowed_artifact_families")?;
        require_non_empty_slice(
            &self.allowed_graph_or_slice_kinds,
            "allowed_graph_or_slice_kinds",
        )?;
        require_non_empty(&self.budget_ceiling, "budget_ceiling")?;
        require_non_empty(&self.lease_expiry, "lease_expiry")?;
        require_non_empty_slice(&self.policy_owner_refs, "policy_owner_refs")?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct RemoteSliceRequestV1 {
    pub schema_version: String,
    pub remote_slice_request_id: RemoteSliceRequestId,
    pub requested_slice_definition: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub required_artifact_refs: Vec<String>,
    pub allowed_disclosure_policy: String,
    pub exactness_target: RemoteExactnessClassV1,
    pub trust_root_set_id: TrustRootSetId,
    #[serde(flatten)]
    pub citation: V25ConstitutionCitation,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub challenge_expectations: Vec<String>,
    pub remote_oracle_lease_id: RemoteOracleLeaseId,
}

impl RemoteSliceRequestV1 {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        remote_slice_request_id: RemoteSliceRequestId,
        requested_slice_definition: impl Into<String>,
        required_artifact_refs: Vec<String>,
        allowed_disclosure_policy: impl Into<String>,
        exactness_target: RemoteExactnessClassV1,
        trust_root_set_id: TrustRootSetId,
        citation: V25ConstitutionCitation,
        challenge_expectations: Vec<String>,
        remote_oracle_lease_id: RemoteOracleLeaseId,
    ) -> Result<Self, &'static str> {
        let value = Self {
            schema_version: REMOTE_SLICE_REQUEST_V1_SCHEMA.to_string(),
            remote_slice_request_id,
            requested_slice_definition: requested_slice_definition.into(),
            required_artifact_refs,
            allowed_disclosure_policy: allowed_disclosure_policy.into(),
            exactness_target,
            trust_root_set_id,
            citation,
            challenge_expectations,
            remote_oracle_lease_id,
        };
        value.validate()?;
        Ok(value)
    }

    pub fn validate(&self) -> Result<(), &'static str> {
        require_schema_version(
            &self.schema_version,
            REMOTE_SLICE_REQUEST_V1_SCHEMA,
            "schema_version",
        )?;
        require_non_empty(
            self.remote_slice_request_id.as_str(),
            "remote_slice_request_id",
        )?;
        require_non_empty(
            &self.requested_slice_definition,
            "requested_slice_definition",
        )?;
        require_non_empty_slice(&self.required_artifact_refs, "required_artifact_refs")?;
        require_non_empty(&self.allowed_disclosure_policy, "allowed_disclosure_policy")?;
        require_non_empty(self.trust_root_set_id.as_str(), "trust_root_set_id")?;
        require_citation(&self.citation)?;
        require_non_empty(
            self.remote_oracle_lease_id.as_str(),
            "remote_oracle_lease_id",
        )?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct RemoteSliceResultV1 {
    pub schema_version: String,
    pub remote_slice_result_id: RemoteSliceResultId,
    pub remote_slice_request_id: RemoteSliceRequestId,
    #[serde(flatten)]
    pub citation: V25ConstitutionCitation,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub returned_artifact_refs: Vec<String>,
    pub exactness_class: RemoteExactnessClassV1,
    pub remote_execution_evidence: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub disclosure_markers: Vec<String>,
    pub replay_handle: String,
    pub local_admission_recommendation: LocalAdmissionRecommendationV1,
    pub attestation_envelope_id: AttestationEnvelopeId,
}

impl RemoteSliceResultV1 {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        remote_slice_result_id: RemoteSliceResultId,
        remote_slice_request_id: RemoteSliceRequestId,
        citation: V25ConstitutionCitation,
        returned_artifact_refs: Vec<String>,
        exactness_class: RemoteExactnessClassV1,
        remote_execution_evidence: impl Into<String>,
        disclosure_markers: Vec<String>,
        replay_handle: impl Into<String>,
        local_admission_recommendation: LocalAdmissionRecommendationV1,
        attestation_envelope_id: AttestationEnvelopeId,
    ) -> Result<Self, &'static str> {
        let value = Self {
            schema_version: REMOTE_SLICE_RESULT_V1_SCHEMA.to_string(),
            remote_slice_result_id,
            remote_slice_request_id,
            citation,
            returned_artifact_refs,
            exactness_class,
            remote_execution_evidence: remote_execution_evidence.into(),
            disclosure_markers,
            replay_handle: replay_handle.into(),
            local_admission_recommendation,
            attestation_envelope_id,
        };
        value.validate()?;
        Ok(value)
    }

    pub fn validate(&self) -> Result<(), &'static str> {
        require_schema_version(
            &self.schema_version,
            REMOTE_SLICE_RESULT_V1_SCHEMA,
            "schema_version",
        )?;
        require_non_empty(
            self.remote_slice_result_id.as_str(),
            "remote_slice_result_id",
        )?;
        require_non_empty(
            self.remote_slice_request_id.as_str(),
            "remote_slice_request_id",
        )?;
        require_citation(&self.citation)?;
        require_non_empty_slice(&self.returned_artifact_refs, "returned_artifact_refs")?;
        require_non_empty(&self.remote_execution_evidence, "remote_execution_evidence")?;
        require_non_empty(&self.replay_handle, "replay_handle")?;
        require_non_empty(
            self.attestation_envelope_id.as_str(),
            "attestation_envelope_id",
        )?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct CrossRuntimeReplayTicketV1 {
    pub schema_version: String,
    pub cross_runtime_replay_ticket_id: CrossRuntimeReplayTicketId,
    #[serde(flatten)]
    pub citation: V25ConstitutionCitation,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub artifact_refs: Vec<String>,
    pub time_coordinates: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub required_trust_roots: Vec<TrustRootSetId>,
    pub allowed_disclosure: RemoteDisclosureClassV1,
    pub lease_window: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub replay_expectations: Vec<String>,
    pub failure_behavior: ReplayFailureBehaviorV1,
}

impl CrossRuntimeReplayTicketV1 {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        cross_runtime_replay_ticket_id: CrossRuntimeReplayTicketId,
        citation: V25ConstitutionCitation,
        artifact_refs: Vec<String>,
        time_coordinates: impl Into<String>,
        required_trust_roots: Vec<TrustRootSetId>,
        allowed_disclosure: RemoteDisclosureClassV1,
        lease_window: impl Into<String>,
        replay_expectations: Vec<String>,
        failure_behavior: ReplayFailureBehaviorV1,
    ) -> Result<Self, &'static str> {
        let value = Self {
            schema_version: CROSS_RUNTIME_REPLAY_TICKET_V1_SCHEMA.to_string(),
            cross_runtime_replay_ticket_id,
            citation,
            artifact_refs,
            time_coordinates: time_coordinates.into(),
            required_trust_roots,
            allowed_disclosure,
            lease_window: lease_window.into(),
            replay_expectations,
            failure_behavior,
        };
        value.validate()?;
        Ok(value)
    }

    pub fn validate(&self) -> Result<(), &'static str> {
        require_schema_version(
            &self.schema_version,
            CROSS_RUNTIME_REPLAY_TICKET_V1_SCHEMA,
            "schema_version",
        )?;
        require_non_empty(
            self.cross_runtime_replay_ticket_id.as_str(),
            "cross_runtime_replay_ticket_id",
        )?;
        require_citation(&self.citation)?;
        require_non_empty_slice(&self.artifact_refs, "artifact_refs")?;
        require_non_empty(&self.time_coordinates, "time_coordinates")?;
        require_non_empty_slice(&self.required_trust_roots, "required_trust_roots")?;
        require_non_empty(&self.lease_window, "lease_window")?;
        Ok(())
    }
}
