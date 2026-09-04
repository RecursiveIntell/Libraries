//! Semantic exactness, view disclosure, and degradation records.
//!
//! These local envelopes label AiDENs reports. They do not replace canonical
//! memory, runtime view, or degradation truth systems.

use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum SemanticExactnessV1 {
    Exact,
    Degraded,
    Partial,
    Refuted,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct SemanticStateV1 {
    pub state_id: ArtifactId,
    pub artifact_ref: ArtifactId,
    pub provenance_carrier: String,
    pub truth_carrier: String,
    pub bitemporal_carrier: String,
    pub identity_carrier: String,
    pub execution_carrier: String,
    pub view_carrier: String,
    pub exactness: SemanticExactnessV1,
    pub proof_carrier: String,
    pub nuisance_or_degradation_carrier: String,
    pub governance_carrier: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub degradation_record_ids: Vec<ArtifactId>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub view_disclosure_ids: Vec<ArtifactId>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub contradiction_record_ids: Vec<ArtifactId>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub execution_contamination_ids: Vec<ArtifactId>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reason_codes: Vec<String>,
}

impl SemanticStateV1 {
    pub fn exact_supported(artifact_ref: ArtifactId, proof_carrier: impl Into<String>) -> Self {
        Self {
            state_id: generated_artifact_id_from_material("semantic-state", artifact_ref.as_str()),
            artifact_ref,
            provenance_carrier: "declared-input-manifest".into(),
            truth_carrier: "canonical-owner-or-admitted-facade".into(),
            bitemporal_carrier: "recorded-time-declared".into(),
            identity_carrier: "stack-id-or-local-artifact-ref".into(),
            execution_carrier: "execution-context-envelope".into(),
            view_carrier: "view-disclosure-required".into(),
            exactness: SemanticExactnessV1::Exact,
            proof_carrier: proof_carrier.into(),
            nuisance_or_degradation_carrier: "none".into(),
            governance_carrier: "no-waiver".into(),
            degradation_record_ids: Vec::new(),
            view_disclosure_ids: Vec::new(),
            contradiction_record_ids: Vec::new(),
            execution_contamination_ids: Vec::new(),
            reason_codes: vec!["semantic-state-exact".into()],
        }
    }

    pub fn with_degradation(mut self, degradation: &LocalDegradationRecordV1) -> Self {
        self.exactness = SemanticExactnessV1::Degraded;
        self.nuisance_or_degradation_carrier = degradation.reason_code.clone();
        self.degradation_record_ids
            .push(degradation.degradation_id.clone());
        self.reason_codes.push("semantic-state-degraded".into());
        self.reason_codes.sort();
        self.reason_codes.dedup();
        self
    }

    pub fn with_view_disclosure(mut self, disclosure: &ViewDisclosureV1) -> Self {
        self.view_disclosure_ids
            .push(disclosure.disclosure_id.clone());
        self.reason_codes
            .push("semantic-state-view-disclosed".into());
        self.reason_codes.sort();
        self.reason_codes.dedup();
        self
    }

    pub fn with_contradiction(mut self, contradiction: &SemanticContradictionRecordV1) -> Self {
        self.exactness = SemanticExactnessV1::Refuted;
        self.truth_carrier = "contradiction-disclosed".into();
        self.contradiction_record_ids
            .push(contradiction.contradiction_id.clone());
        self.reason_codes
            .push("semantic-state-contradiction-disclosed".into());
        self.reason_codes.sort();
        self.reason_codes.dedup();
        self
    }

    pub fn with_execution_contamination(
        mut self,
        contamination: &ExecutionContaminationRecordV1,
    ) -> Self {
        self.exactness = SemanticExactnessV1::Degraded;
        self.execution_carrier = contamination.reason_code.clone();
        self.execution_contamination_ids
            .push(contamination.contamination_id.clone());
        self.reason_codes
            .push("semantic-state-execution-contamination-disclosed".into());
        self.reason_codes.sort();
        self.reason_codes.dedup();
        self
    }

    pub fn can_answer_as_exact(&self) -> bool {
        self.exactness == SemanticExactnessV1::Exact
            && self.degradation_record_ids.is_empty()
            && self.contradiction_record_ids.is_empty()
            && self.execution_contamination_ids.is_empty()
    }

    pub fn blocks_promotion(&self) -> bool {
        !self.can_answer_as_exact()
            || !self.contradiction_record_ids.is_empty()
            || !self.execution_contamination_ids.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct SemanticContradictionRecordV1 {
    pub contradiction_id: ArtifactId,
    pub artifact_ref: ArtifactId,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub witness_ids: Vec<ArtifactId>,
    pub reason_code: String,
    pub exactness_after: SemanticExactnessV1,
    pub promotion_blocking: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub canonical_backpointers: Vec<CanonicalBackpointerV1>,
    pub recorded_at: DateTime<Utc>,
}

impl SemanticContradictionRecordV1 {
    pub fn new(
        artifact_ref: ArtifactId,
        witness_ids: Vec<ArtifactId>,
        reason_code: impl Into<String>,
    ) -> Self {
        let reason_code = reason_code.into();
        let material = format!(
            "{}|{}|{}",
            artifact_ref.as_str(),
            witness_ids
                .iter()
                .map(|id| id.as_str().to_string())
                .collect::<Vec<_>>()
                .join("|"),
            reason_code
        );
        Self {
            contradiction_id: generated_artifact_id_from_material(
                "semantic-contradiction",
                &material,
            ),
            artifact_ref,
            witness_ids,
            reason_code,
            exactness_after: SemanticExactnessV1::Refuted,
            promotion_blocking: true,
            canonical_backpointers: canonical_owner_backpointer(
                "verification-adjudication",
                "VerificationDisposition",
                "canonical-contradiction-adjudication-owner",
            ),
            recorded_at: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ExecutionContaminationRecordV1 {
    pub contamination_id: ArtifactId,
    pub artifact_ref: ArtifactId,
    pub execution_context_ref: ArtifactId,
    pub reason_code: String,
    pub domain_truth_contaminated: bool,
    pub exactness_after: SemanticExactnessV1,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub canonical_backpointers: Vec<CanonicalBackpointerV1>,
    pub recorded_at: DateTime<Utc>,
}

impl ExecutionContaminationRecordV1 {
    pub fn new(
        artifact_ref: ArtifactId,
        execution_context_ref: ArtifactId,
        reason_code: impl Into<String>,
    ) -> Self {
        let reason_code = reason_code.into();
        Self {
            contamination_id: generated_artifact_id_from_material(
                "execution-contamination",
                &format!(
                    "{}|{}|{}",
                    artifact_ref.as_str(),
                    execution_context_ref.as_str(),
                    reason_code
                ),
            ),
            artifact_ref,
            execution_context_ref,
            reason_code,
            domain_truth_contaminated: true,
            exactness_after: SemanticExactnessV1::Degraded,
            canonical_backpointers: canonical_owner_backpointer(
                "llm-tool-runtime",
                "ExecutionContext",
                "canonical-execution-context-owner",
            ),
            recorded_at: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ViewDisclosureV1 {
    pub disclosure_id: ArtifactId,
    pub view_family: String,
    pub widening: bool,
    pub support_label: String,
    pub exactness: SemanticExactnessV1,
    pub source_report_id: Option<ArtifactId>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reason_codes: Vec<String>,
    pub recorded_at: DateTime<Utc>,
}

impl ViewDisclosureV1 {
    pub fn widening(
        view_family: impl Into<String>,
        support_label: impl Into<String>,
        source_report_id: Option<ArtifactId>,
    ) -> Self {
        let view_family = view_family.into();
        let support_label = support_label.into();
        Self {
            disclosure_id: generated_artifact_id_from_material(
                "view-disclosure",
                &format!("{view_family}|{support_label}|widening"),
            ),
            view_family,
            widening: true,
            support_label,
            exactness: SemanticExactnessV1::Degraded,
            source_report_id,
            reason_codes: vec!["view-widening-disclosed".into()],
            recorded_at: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct LocalDegradationRecordV1 {
    pub degradation_id: ArtifactId,
    pub artifact_ref: ArtifactId,
    pub reason_code: String,
    pub guarantee_weakened: String,
    pub exactness_after: SemanticExactnessV1,
    pub waived: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reason_codes: Vec<String>,
    pub recorded_at: DateTime<Utc>,
}

impl LocalDegradationRecordV1 {
    pub fn new(
        artifact_ref: ArtifactId,
        reason_code: impl Into<String>,
        guarantee_weakened: impl Into<String>,
    ) -> Self {
        let reason_code = reason_code.into();
        let guarantee_weakened = guarantee_weakened.into();
        Self {
            degradation_id: generated_artifact_id_from_material(
                "degradation-record",
                &format!(
                    "{}|{}|{}",
                    artifact_ref.as_str(),
                    reason_code,
                    guarantee_weakened
                ),
            ),
            artifact_ref,
            reason_code,
            guarantee_weakened,
            exactness_after: SemanticExactnessV1::Degraded,
            waived: false,
            reason_codes: vec!["degradation-record-emitted".into()],
            recorded_at: Utc::now(),
        }
    }

    pub fn blocks_readiness_without_waiver(&self) -> bool {
        !self.waived && self.exactness_after != SemanticExactnessV1::Exact
    }
}
