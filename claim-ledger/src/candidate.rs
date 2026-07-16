//! Candidate-discovery provenance for semantic-memory/proveKV assisted claim workflows.
//!
//! Candidate discovery is not ledger authority. It can help assemble review packets,
//! but it cannot mutate support state, promotion state, or verification state.

use serde::{Deserialize, Serialize};

pub const SIMILAR_CLAIM_CANDIDATE_V1_SCHEMA: &str = "similar_claim_candidate_v1";
pub const PROOF_PACKET_CANDIDATE_PROVENANCE_V1_SCHEMA: &str =
    "proof_packet_candidate_provenance_v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SimilarClaimCandidateV1 {
    pub claim_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub claim_version_id: Option<String>,
    pub retrieval_backend: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub candidate_backend: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub generation_id: Option<String>,
    pub exact_rerank: bool,
    pub candidate_only: bool,
    pub mutates_verification_state: bool,
}

impl SimilarClaimCandidateV1 {
    pub fn validate_boundary(&self) -> Result<(), String> {
        if self.claim_id.trim().is_empty() {
            return Err("claim_id must not be empty".into());
        }
        if self.retrieval_backend.trim().is_empty() {
            return Err("retrieval_backend must not be empty".into());
        }
        if !self.candidate_only {
            return Err("similar claim discovery must stay candidate_only".into());
        }
        if !self.exact_rerank {
            return Err("semantic-memory candidate discovery requires exact rerank".into());
        }
        if self.mutates_verification_state {
            return Err("candidate discovery cannot mutate verification state".into());
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProofPacketCandidateProvenanceV1 {
    pub schema_version: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub candidate_discovery: Vec<SimilarClaimCandidateV1>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub verified_evidence_refs: Vec<String>,
}

impl ProofPacketCandidateProvenanceV1 {
    pub fn validate_for_verification_gate(&self) -> Result<(), String> {
        if self.schema_version != PROOF_PACKET_CANDIDATE_PROVENANCE_V1_SCHEMA {
            return Err(format!(
                "unsupported schema_version '{}'",
                self.schema_version
            ));
        }
        for candidate in &self.candidate_discovery {
            candidate.validate_boundary()?;
        }
        if self.verified_evidence_refs.is_empty() {
            return Err(
                "proof packet has candidate discovery but no verified evidence refs".into(),
            );
        }
        Ok(())
    }
}
