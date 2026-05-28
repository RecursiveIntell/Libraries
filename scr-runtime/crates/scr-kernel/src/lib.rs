//! SCR-P0A canonical kernel types.
//!
//! These types define the deterministic, receipt-bearing evaluation surface for
//! proposed actions. External identifiers remain opaque adapter references; this
//! crate does not claim ownership over upstream artifact, evidence, provenance,
//! policy, permit, or repository truth.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use thiserror::Error;

pub const CONTROL_EVALUATION_INPUT_V1_SCHEMA: &str = "control_evaluation_input_v1";
pub const CONTROL_DECISION_RECEIPT_V1_SCHEMA: &str = "control_decision_receipt_v1";

/// A fixed-point score in basis points, where `0` is 0% and `10000` is 100%.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
#[serde(try_from = "u16", into = "u16")]
#[serde(deny_unknown_fields)]
#[schemars(range(min = "0", max = "10000"))]
pub struct ScoreBps(u16);

impl ScoreBps {
    pub const MIN: u16 = 0;
    pub const MAX: u16 = 10_000;

    /// Builds a score after enforcing the `0..=10000` basis-point range.
    pub fn new(value: u16) -> Result<Self, ScrError> {
        if value <= Self::MAX {
            Ok(Self(value))
        } else {
            Err(ScrError::ScoreOutOfRange {
                field: "score_bps",
                value,
                max: Self::MAX,
            })
        }
    }

    /// Returns the raw basis-point value.
    pub fn value(self) -> u16 {
        self.0
    }
}

impl TryFrom<u16> for ScoreBps {
    type Error = ScrError;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<ScoreBps> for u16 {
    fn from(value: ScoreBps) -> Self {
        value.0
    }
}

/// A fixed-point weight in basis points, where `0` is 0% and `10000` is 100%.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
#[serde(try_from = "u16", into = "u16")]
#[serde(deny_unknown_fields)]
#[schemars(range(min = "0", max = "10000"))]
pub struct WeightBps(u16);

impl WeightBps {
    pub const MIN: u16 = 0;
    pub const MAX: u16 = 10_000;

    /// Builds a weight after enforcing the `0..=10000` basis-point range.
    pub fn new(value: u16) -> Result<Self, ScrError> {
        if value <= Self::MAX {
            Ok(Self(value))
        } else {
            Err(ScrError::ScoreOutOfRange {
                field: "weight_bps",
                value,
                max: Self::MAX,
            })
        }
    }

    /// Returns the raw basis-point value.
    pub fn value(self) -> u16 {
        self.0
    }
}

impl TryFrom<u16> for WeightBps {
    type Error = ScrError;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<WeightBps> for u16 {
    fn from(value: WeightBps) -> Self {
        value.0
    }
}

/// Opaque reference to an external owner boundary.
///
/// This is an adapter reference only. It does not assert that SCR owns,
/// canonicalizes, fetches, validates, or mutates the referenced object.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ExternalArtifactRef {
    pub ref_kind: String,
    pub ref_value: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner_hint: Option<String>,
}

impl ExternalArtifactRef {
    pub fn new(
        ref_kind: impl Into<String>,
        ref_value: impl Into<String>,
    ) -> Result<Self, ScrError> {
        let value = Self {
            ref_kind: ref_kind.into(),
            ref_value: ref_value.into(),
            owner_hint: None,
        };
        value.validate()?;
        Ok(value)
    }

    pub fn with_owner_hint(
        ref_kind: impl Into<String>,
        ref_value: impl Into<String>,
        owner_hint: impl Into<String>,
    ) -> Result<Self, ScrError> {
        let value = Self {
            ref_kind: ref_kind.into(),
            ref_value: ref_value.into(),
            owner_hint: Some(owner_hint.into()),
        };
        value.validate()?;
        Ok(value)
    }

    pub fn validate(&self) -> Result<(), ScrError> {
        require_non_empty(&self.ref_kind, "ref_kind")?;
        require_non_empty(&self.ref_value, "ref_value")?;
        if let Some(owner_hint) = &self.owner_hint {
            require_non_empty(owner_hint, "owner_hint")?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Domain {
    Code,
    Release,
    Policy,
    Artifact,
    Audit,
    Operations,
    Other(String),
}

impl Domain {
    pub fn policy_key(&self) -> &str {
        match self {
            Self::Code => "code",
            Self::Release => "release",
            Self::Policy => "policy",
            Self::Artifact => "artifact",
            Self::Audit => "audit",
            Self::Operations => "operations",
            Self::Other(value) => value.as_str(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ProposedAction {
    Read,
    Analyze,
    Verify,
    GenerateRepairPacket,
    MutateArtifact,
    Release,
    Quarantine,
    Other(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RequestedEffect {
    NoMutation,
    AdvisoryOnly,
    PreparePatch,
    ApplyPatch,
    GenerateReleaseArtifact,
    BlockRelease,
    Other(String),
}

/// Canonical input evaluated by SCR-P0A.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ControlEvaluationInputV1 {
    pub schema_version: String,
    pub input_id: String,
    pub actor_ref: ExternalArtifactRef,
    pub permit_ref: ExternalArtifactRef,
    pub subject_ref: ExternalArtifactRef,
    pub domain: Domain,
    pub proposed_action: ProposedAction,
    pub requested_effect: RequestedEffect,
    #[serde(default)]
    pub evidence_refs: Vec<ExternalArtifactRef>,
    pub environment_ref: ExternalArtifactRef,
    pub valid_time_basis: String,
    pub recorded_time: String,
}

impl ControlEvaluationInputV1 {
    pub const SCHEMA_VERSION: &'static str = CONTROL_EVALUATION_INPUT_V1_SCHEMA;

    pub fn validate(&self) -> Result<(), ScrError> {
        require_schema_version(
            Self::SCHEMA_VERSION,
            &self.schema_version,
            "ControlEvaluationInputV1",
        )?;
        require_non_empty(&self.input_id, "input_id")?;
        self.actor_ref.validate()?;
        self.permit_ref.validate()?;
        self.subject_ref.validate()?;
        self.environment_ref.validate()?;
        for evidence_ref in &self.evidence_refs {
            evidence_ref.validate()?;
        }
        require_non_empty(&self.valid_time_basis, "valid_time_basis")?;
        require_non_empty(&self.recorded_time, "recorded_time")?;
        Ok(())
    }
}

/// Separate control axes. These must not be collapsed into a truth score.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ScoreAxesV1 {
    pub hazard: ScoreBps,
    pub evidence_confidence: ScoreBps,
    pub uncertainty: ScoreBps,
    pub authority: ScoreBps,
    pub containment: ScoreBps,
    pub integrity_risk: ScoreBps,
}

/// Derived deterministic pressures used by the action resolver.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DerivedPressuresV1 {
    pub autonomy_pressure: ScoreBps,
    pub verification_pressure: ScoreBps,
    pub repair_priority: ScoreBps,
    pub quarantine_pressure: ScoreBps,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ControlAction {
    AllowWithReceipt,
    Backlog,
    RequireSourceBasis,
    RequireVerification,
    RequireApproval,
    GenerateRepairPacket,
    RequireOwnerResolution,
    BlockMutation,
    BlockRelease,
    QuarantineArtifact,
}

impl ControlAction {
    pub fn name(&self) -> &'static str {
        match self {
            Self::AllowWithReceipt => "allow_with_receipt",
            Self::Backlog => "backlog",
            Self::RequireSourceBasis => "require_source_basis",
            Self::RequireVerification => "require_verification",
            Self::RequireApproval => "require_approval",
            Self::GenerateRepairPacket => "generate_repair_packet",
            Self::RequireOwnerResolution => "require_owner_resolution",
            Self::BlockMutation => "block_mutation",
            Self::BlockRelease => "block_release",
            Self::QuarantineArtifact => "quarantine_artifact",
        }
    }

    pub fn all_names() -> &'static [&'static str] {
        &[
            "backlog",
            "allow_with_receipt",
            "require_source_basis",
            "require_verification",
            "require_owner_resolution",
            "require_approval",
            "generate_repair_packet",
            "quarantine_artifact",
            "block_mutation",
            "block_release",
        ]
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RejectedActionV1 {
    pub action: ControlAction,
    pub reason_codes: Vec<ReasonCode>,
}

impl RejectedActionV1 {
    pub fn new(action: ControlAction, reason_codes: Vec<ReasonCode>) -> Result<Self, ScrError> {
        if reason_codes.is_empty() {
            return Err(ScrError::MissingField("rejected_action.reason_codes"));
        }
        Ok(Self {
            action,
            reason_codes,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ReasonCode(pub String);

impl ReasonCode {
    pub fn new(value: impl Into<String>) -> Result<Self, ScrError> {
        let value = value.into();
        require_non_empty(&value, "reason_code")?;
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct HardRuleResultV1 {
    pub rule_id: String,
    pub checked: bool,
    pub triggered: bool,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

impl HardRuleResultV1 {
    pub fn validate(&self) -> Result<(), ScrError> {
        require_non_empty(&self.rule_id, "hard_rule.rule_id")?;
        if self.triggered && self.reason_codes.is_empty() {
            return Err(ScrError::MissingField("hard_rule.reason_codes"));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AuthorityBasisV1 {
    pub actor_ref: ExternalArtifactRef,
    pub permit_ref: ExternalArtifactRef,
    pub authority_result: String,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

impl AuthorityBasisV1 {
    pub fn validate(&self) -> Result<(), ScrError> {
        self.actor_ref.validate()?;
        self.permit_ref.validate()?;
        require_non_empty(&self.authority_result, "authority_result")?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EvidenceBasisV1 {
    #[serde(default)]
    pub evidence_refs: Vec<ExternalArtifactRef>,
    pub evidence_result: String,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

impl EvidenceBasisV1 {
    pub fn validate(&self) -> Result<(), ScrError> {
        for evidence_ref in &self.evidence_refs {
            evidence_ref.validate()?;
        }
        require_non_empty(&self.evidence_result, "evidence_result")?;
        Ok(())
    }
}

/// Replayable SCR decision receipt.
///
/// This is scoped to SCR-P0A evaluation and is not a replacement for upstream
/// domain receipts.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ControlDecisionReceiptV1 {
    pub schema_version: String,
    pub input_hash: String,
    pub canonical_policy_hash: String,
    pub evaluator_algorithm_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub evaluator_algorithm_hash: Option<String>,
    #[serde(default)]
    pub hard_rules_checked: Vec<String>,
    #[serde(default)]
    pub hard_rules_triggered: Vec<String>,
    #[serde(default)]
    pub minimum_action_floors_applied: Vec<String>,
    #[serde(default)]
    pub hard_rule_results: Vec<HardRuleResultV1>,
    pub axes: ScoreAxesV1,
    pub derived_pressures: DerivedPressuresV1,
    pub chosen_action: ControlAction,
    #[serde(default)]
    pub rejected_actions: Vec<RejectedActionV1>,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
    pub authority_basis: AuthorityBasisV1,
    pub evidence_basis: EvidenceBasisV1,
    pub valid_time_basis: String,
    pub recorded_time: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub supersession_ref: Option<ExternalArtifactRef>,
}

impl ControlDecisionReceiptV1 {
    pub const SCHEMA_VERSION: &'static str = CONTROL_DECISION_RECEIPT_V1_SCHEMA;

    pub fn validate(&self) -> Result<(), ScrError> {
        require_schema_version(
            Self::SCHEMA_VERSION,
            &self.schema_version,
            "ControlDecisionReceiptV1",
        )?;
        require_non_empty(&self.input_hash, "input_hash")?;
        require_non_empty(&self.canonical_policy_hash, "canonical_policy_hash")?;
        require_non_empty(&self.evaluator_algorithm_id, "evaluator_algorithm_id")?;
        if let Some(hash) = &self.evaluator_algorithm_hash {
            require_non_empty(hash, "evaluator_algorithm_hash")?;
        }
        for result in &self.hard_rule_results {
            result.validate()?;
        }
        for floor in &self.minimum_action_floors_applied {
            require_non_empty(floor, "minimum_action_floors_applied")?;
        }
        for rejected_action in &self.rejected_actions {
            if rejected_action.reason_codes.is_empty() {
                return Err(ScrError::MissingField("rejected_action.reason_codes"));
            }
        }
        if self.reason_codes.is_empty() {
            return Err(ScrError::MissingField("reason_codes"));
        }
        self.authority_basis.validate()?;
        self.evidence_basis.validate()?;
        require_non_empty(&self.valid_time_basis, "valid_time_basis")?;
        require_non_empty(&self.recorded_time, "recorded_time")?;
        if let Some(supersession_ref) = &self.supersession_ref {
            supersession_ref.validate()?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ScrError {
    #[error("score out of range for {field}: {value}; maximum is {max}")]
    ScoreOutOfRange {
        field: &'static str,
        value: u16,
        max: u16,
    },
    #[error("missing required field: {0}")]
    MissingField(&'static str),
    #[error("invalid schema version for {artifact}: expected {expected}, found {found}")]
    InvalidSchemaVersion {
        artifact: &'static str,
        expected: &'static str,
        found: String,
    },
    #[error("evaluation is not implemented for this phase: {0}")]
    EvaluationUnavailable(&'static str),
    #[error("policy parse failed: {0}")]
    PolicyParseFailed(String),
    #[error("policy validation failed: {0}")]
    PolicyValidationFailed(String),
    #[error("serialization failed: {0}")]
    SerializationFailed(String),
}

impl ScrError {
    /// Returns a stable machine-readable error kind.
    pub fn kind(&self) -> &'static str {
        match self {
            Self::ScoreOutOfRange { .. } => "score_out_of_range",
            Self::MissingField(..) => "missing_field",
            Self::InvalidSchemaVersion { .. } => "invalid_schema_version",
            Self::EvaluationUnavailable(..) => "evaluation_unavailable",
            Self::PolicyParseFailed(..) => "policy_parse_failed",
            Self::PolicyValidationFailed(..) => "policy_validation_failed",
            Self::SerializationFailed(..) => "serialization_failed",
        }
    }
}

fn require_non_empty(value: &str, field: &'static str) -> Result<(), ScrError> {
    if value.trim().is_empty() {
        Err(ScrError::MissingField(field))
    } else {
        Ok(())
    }
}

fn require_schema_version(
    expected: &'static str,
    found: &str,
    artifact: &'static str,
) -> Result<(), ScrError> {
    if found == expected {
        Ok(())
    } else {
        Err(ScrError::InvalidSchemaVersion {
            artifact,
            expected,
            found: found.to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ref_value(value: &str) -> ExternalArtifactRef {
        ExternalArtifactRef::new("opaque_ref", value).unwrap()
    }

    fn score(value: u16) -> ScoreBps {
        ScoreBps::new(value).unwrap()
    }

    fn reason(value: &str) -> ReasonCode {
        ReasonCode::new(value).unwrap()
    }

    fn axes() -> ScoreAxesV1 {
        ScoreAxesV1 {
            hazard: score(8000),
            evidence_confidence: score(5000),
            uncertainty: score(7000),
            authority: score(9000),
            containment: score(3000),
            integrity_risk: score(6000),
        }
    }

    fn pressures() -> DerivedPressuresV1 {
        DerivedPressuresV1 {
            autonomy_pressure: score(8000),
            verification_pressure: score(8500),
            repair_priority: score(4000),
            quarantine_pressure: score(2000),
        }
    }

    fn receipt() -> ControlDecisionReceiptV1 {
        ControlDecisionReceiptV1 {
            schema_version: ControlDecisionReceiptV1::SCHEMA_VERSION.to_string(),
            input_hash: "hash_input_001".to_string(),
            canonical_policy_hash: "hash_policy_001".to_string(),
            evaluator_algorithm_id: "scr-p0a-reference-v1".to_string(),
            evaluator_algorithm_hash: Some("hash_algorithm_001".to_string()),
            hard_rules_checked: vec!["schema_validity".to_string()],
            hard_rules_triggered: vec!["requires_verification".to_string()],
            minimum_action_floors_applied: vec!["source_truth_drift".to_string()],
            hard_rule_results: vec![HardRuleResultV1 {
                rule_id: "requires_verification".to_string(),
                checked: true,
                triggered: true,
                reason_codes: vec![reason("high_hazard_uncertain")],
            }],
            axes: axes(),
            derived_pressures: pressures(),
            chosen_action: ControlAction::RequireVerification,
            rejected_actions: vec![RejectedActionV1::new(
                ControlAction::AllowWithReceipt,
                vec![reason("verification_pressure_exceeds_allow")],
            )
            .unwrap()],
            reason_codes: vec![reason("require_verification")],
            authority_basis: AuthorityBasisV1 {
                actor_ref: ref_value("actor_001"),
                permit_ref: ref_value("permit_001"),
                authority_result: "permit_valid_for_evaluation".to_string(),
                reason_codes: vec![reason("authority_present")],
            },
            evidence_basis: EvidenceBasisV1 {
                evidence_refs: vec![ref_value("evidence_001")],
                evidence_result: "evidence_basis_recorded".to_string(),
                reason_codes: vec![reason("evidence_refs_present")],
            },
            valid_time_basis: "2026-05-13T00:00:00Z".to_string(),
            recorded_time: "2026-05-13T00:00:00Z".to_string(),
            supersession_ref: None,
        }
    }

    #[test]
    fn score_bound_validation_accepts_edges() {
        assert_eq!(ScoreBps::new(0).unwrap().value(), 0);
        assert_eq!(ScoreBps::new(10_000).unwrap().value(), 10_000);
        assert_eq!(WeightBps::new(0).unwrap().value(), 0);
        assert_eq!(WeightBps::new(10_000).unwrap().value(), 10_000);
    }

    #[test]
    fn invalid_scores_are_rejected() {
        assert_eq!(
            ScoreBps::new(10_001).unwrap_err().kind(),
            "score_out_of_range"
        );
        assert_eq!(
            WeightBps::new(10_001).unwrap_err().kind(),
            "score_out_of_range"
        );
    }

    #[test]
    fn score_deserialization_rejects_invalid_value() {
        let err = serde_json::from_str::<ScoreBps>("10001").unwrap_err();
        assert!(err.to_string().contains("score out of range"));
    }

    #[test]
    fn input_serialization_round_trip() {
        let input = ControlEvaluationInputV1 {
            schema_version: ControlEvaluationInputV1::SCHEMA_VERSION.to_string(),
            input_id: "input_001".to_string(),
            actor_ref: ref_value("actor_001"),
            permit_ref: ref_value("permit_001"),
            subject_ref: ref_value("subject_001"),
            domain: Domain::Code,
            proposed_action: ProposedAction::Analyze,
            requested_effect: RequestedEffect::AdvisoryOnly,
            evidence_refs: vec![ref_value("evidence_001")],
            environment_ref: ref_value("environment_001"),
            valid_time_basis: "2026-05-13T00:00:00Z".to_string(),
            recorded_time: "2026-05-13T00:00:00Z".to_string(),
        };
        input.validate().unwrap();

        let encoded = serde_json::to_string(&input).unwrap();
        let decoded: ControlEvaluationInputV1 = serde_json::from_str(&encoded).unwrap();

        assert_eq!(decoded, input);
        decoded.validate().unwrap();
    }

    #[test]
    fn receipt_required_shape_validates() {
        let receipt = receipt();
        receipt.validate().unwrap();

        let encoded = serde_json::to_string(&receipt).unwrap();
        let decoded: ControlDecisionReceiptV1 = serde_json::from_str(&encoded).unwrap();

        assert_eq!(decoded, receipt);
        decoded.validate().unwrap();
    }

    #[test]
    fn receipt_requires_reason_codes() {
        let mut receipt = receipt();
        receipt.reason_codes.clear();

        assert_eq!(receipt.validate().unwrap_err().kind(), "missing_field");
    }
}
