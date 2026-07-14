//! Typed treaty and settlement surface crate for shared-view artifact families.
//!
//! The crate keeps the compatibility name, but it is not a network runtime.
//! It publishes settlement-facing schemas and bounded evaluators that keep
//! degraded and dissent states explicit.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use stack_ids::{
    CrossRuntimeEquivalenceBundleId, LocalDissentId, RuntimeIdentitySetId, SettlementCaseId,
    SettlementReceiptId, SharedDispositionId, SharedDivergenceReportId, SharedReplaySliceId,
    SharedViewDowngradeId, TreatyBundleId, TreatySuspensionId,
};

pub use stack_ids::{SurfaceStatus, V25ConstitutionCitation};

pub const TREATY_BUNDLE_V1_SCHEMA: &str = "treaty_bundle_v1";
pub const RUNTIME_IDENTITY_SET_V1_SCHEMA: &str = "runtime_identity_set_v1";
pub const CROSS_RUNTIME_EQUIVALENCE_BUNDLE_V1_SCHEMA: &str = "cross_runtime_equivalence_bundle_v1";
pub const SETTLEMENT_CASE_V1_SCHEMA: &str = "settlement_case_v1";
pub const SHARED_DISPOSITION_V1_SCHEMA: &str = "shared_disposition_v1";
pub const LOCAL_DISSENT_RECORD_V1_SCHEMA: &str = "local_dissent_record_v1";
pub const SHARED_VIEW_DOWNGRADE_V1_SCHEMA: &str = "shared_view_downgrade_v1";
pub const SETTLEMENT_RECEIPT_V1_SCHEMA: &str = "settlement_receipt_v1";
pub const SHARED_REPLAY_SLICE_V1_SCHEMA: &str = "shared_replay_slice_v1";
pub const SHARED_DIVERGENCE_REPORT_V1_SCHEMA: &str = "shared_divergence_report_v1";
pub const TREATY_SUSPENSION_V1_SCHEMA: &str = "treaty_suspension_v1";

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum SettlementValidationError {
    #[error("invalid artifact state: {0}")]
    InvalidState(&'static str),
    #[error("invalid artifact linkage: {0}")]
    InvalidLinkage(&'static str),
    #[error("duplicate runtime identity: {0}")]
    DuplicateRuntime(String),
}

pub type SettlementValidationResult = Result<(), SettlementValidationError>;

/// Validates treaty, identity, equivalence, and case linkage before settlement.
/// This remains a pure artifact check; it does not create global authority.
pub fn validate_settlement_inputs(
    treaty: &TreatyBundleV1,
    identities: &RuntimeIdentitySetV1,
    equivalence: &CrossRuntimeEquivalenceBundleV1,
    case: &SettlementCaseV1,
) -> SettlementValidationResult {
    if treaty.schema_version != TREATY_BUNDLE_V1_SCHEMA
        || identities.schema_version != RUNTIME_IDENTITY_SET_V1_SCHEMA
        || equivalence.schema_version != CROSS_RUNTIME_EQUIVALENCE_BUNDLE_V1_SCHEMA
        || case.schema_version != SETTLEMENT_CASE_V1_SCHEMA
    {
        return Err(SettlementValidationError::InvalidState(
            "unsupported settlement artifact schema version",
        ));
    }
    if treaty.treaty_bundle_id != identities.treaty_bundle_id
        || treaty.treaty_bundle_id != equivalence.treaty_bundle_id
        || treaty.treaty_bundle_id != case.treaty_bundle_id
    {
        return Err(SettlementValidationError::InvalidLinkage(
            "treaty linkage is inconsistent across settlement artifacts",
        ));
    }
    if identities.runtime_identity_set_id != equivalence.runtime_identity_set_id {
        return Err(SettlementValidationError::InvalidLinkage(
            "equivalence bundle does not refer to the runtime identity set",
        ));
    }
    if equivalence.equivalence_bundle_id != case.equivalence_bundle_id {
        return Err(SettlementValidationError::InvalidLinkage(
            "settlement case does not refer to the equivalence bundle",
        ));
    }
    let mut party_names = std::collections::BTreeSet::new();
    for party in &treaty.parties {
        if party.runtime_name.trim().is_empty()
            || party.local_authority.trim().is_empty()
            || party.admission_lane.trim().is_empty()
        {
            return Err(SettlementValidationError::InvalidState(
                "treaty party contains an empty admission field",
            ));
        }
        if !party_names.insert(party.runtime_name.clone()) {
            return Err(SettlementValidationError::DuplicateRuntime(
                party.runtime_name.clone(),
            ));
        }
    }
    let mut identity_names = std::collections::BTreeSet::new();
    for identity in &identities.identities {
        if !party_names.contains(&identity.runtime_name)
            || identity.local_subject_ref.trim().is_empty()
            || identity.shared_subject_ref.trim().is_empty()
            || identity.evidence_ref.trim().is_empty()
        {
            return Err(SettlementValidationError::InvalidLinkage(
                "runtime identity is not admitted by the treaty or lacks evidence",
            ));
        }
        if !identity_names.insert(identity.runtime_name.clone()) {
            return Err(SettlementValidationError::DuplicateRuntime(
                identity.runtime_name.clone(),
            ));
        }
    }
    if identity_names != party_names {
        return Err(SettlementValidationError::InvalidLinkage(
            "treaty parties and runtime identities do not match",
        ));
    }
    for evidence in &equivalence.evidence {
        if evidence.local_claim_ref.trim().is_empty()
            || evidence.shared_claim_ref.trim().is_empty()
            || evidence.evidence_refs.is_empty()
        {
            return Err(SettlementValidationError::InvalidState(
                "equivalence evidence is incomplete",
            ));
        }
    }
    let mut replay_names = std::collections::BTreeSet::new();
    for requirement in &case.replay_requirements {
        if !identity_names.contains(&requirement.runtime_name)
            || !replay_names.insert(requirement.runtime_name.clone())
        {
            return Err(SettlementValidationError::InvalidLinkage(
                "replay requirements must name each admitted runtime at most once",
            ));
        }
    }
    for dissent in &case.local_dissent {
        if !identity_names.contains(&dissent.runtime_name) {
            return Err(SettlementValidationError::InvalidLinkage(
                "local dissent names a runtime outside the admitted identity set",
            ));
        }
    }
    Ok(())
}
// CompositionReceiptId, EffectiveConstitutionId, and CompiledObligationSetId.

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct TreatyPartyV1 {
    pub runtime_name: String,
    pub local_authority: String,
    pub admission_lane: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct TreatyBundleV1 {
    pub schema_version: String,
    pub treaty_bundle_id: TreatyBundleId,
    pub treaty_name: String,
    #[serde(default)]
    pub parties: Vec<TreatyPartyV1>,
    pub canonical_owner: String,
    pub publication_status: SurfaceStatus,
    pub advisory_only: bool,
    pub non_authoritative: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct RuntimeIdentityRecordV1 {
    pub runtime_name: String,
    pub local_subject_ref: String,
    pub shared_subject_ref: String,
    pub evidence_ref: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct RuntimeIdentitySetV1 {
    pub schema_version: String,
    pub runtime_identity_set_id: RuntimeIdentitySetId,
    pub treaty_bundle_id: TreatyBundleId,
    #[serde(default)]
    pub identities: Vec<RuntimeIdentityRecordV1>,
    pub canonical_owner: String,
    pub publication_status: SurfaceStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct EquivalenceEvidenceV1 {
    pub local_claim_ref: String,
    pub shared_claim_ref: String,
    #[serde(default)]
    pub evidence_refs: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct CrossRuntimeEquivalenceBundleV1 {
    pub schema_version: String,
    pub equivalence_bundle_id: CrossRuntimeEquivalenceBundleId,
    pub treaty_bundle_id: TreatyBundleId,
    pub runtime_identity_set_id: RuntimeIdentitySetId,
    pub equivalence_scope: String,
    #[serde(default)]
    pub evidence: Vec<EquivalenceEvidenceV1>,
    pub canonical_owner: String,
    pub publication_status: SurfaceStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct SharedDispositionV1 {
    pub schema_version: String,
    pub shared_disposition_id: SharedDispositionId,
    pub disposition_label: String,
    pub treatment: String,
    pub advisory_only: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct LocalDissentRecordV1 {
    pub schema_version: String,
    pub local_dissent_id: LocalDissentId,
    pub runtime_name: String,
    pub reason: String,
    pub blocks_shared_settlement: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ReplayRequirementV1 {
    pub runtime_name: String,
    pub required: bool,
    pub replay_available: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SettlementDisposition {
    SharedDispositionIssued,
    AdvisoryOnly,
    DegradedSharedView,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum DowngradeReason {
    MissingRequiredReplay,
    NonAdmittedSurface,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct SharedViewDowngradeV1 {
    pub schema_version: String,
    pub shared_view_downgrade_id: SharedViewDowngradeId,
    pub reason: DowngradeReason,
    pub detail: String,
    pub preserves_local_dissent: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct SettlementCaseV1 {
    pub schema_version: String,
    pub settlement_case_id: SettlementCaseId,
    pub treaty_bundle_id: TreatyBundleId,
    pub equivalence_bundle_id: CrossRuntimeEquivalenceBundleId,
    #[serde(flatten)]
    pub citation: V25ConstitutionCitation,
    pub shared_disposition: SharedDispositionV1,
    #[serde(default)]
    pub replay_requirements: Vec<ReplayRequirementV1>,
    #[serde(default)]
    pub local_dissent: Vec<LocalDissentRecordV1>,
    pub advisory_only: bool,
    pub non_admitted: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct SettlementReceiptV1 {
    pub schema_version: String,
    pub settlement_receipt_id: SettlementReceiptId,
    pub settlement_case_id: SettlementCaseId,
    #[serde(flatten)]
    pub citation: V25ConstitutionCitation,
    pub disposition: SettlementDisposition,
    pub settlement_allowed: bool,
    pub advisory_only: bool,
    pub non_admitted: bool,
    pub degraded: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shared_disposition_id: Option<SharedDispositionId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub downgrade: Option<SharedViewDowngradeV1>,
    #[serde(default)]
    pub local_dissent: Vec<LocalDissentRecordV1>,
    pub generated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct SharedReplaySliceV1 {
    pub schema_version: String,
    pub shared_replay_slice_id: SharedReplaySliceId,
    pub settlement_case_id: SettlementCaseId,
    pub treaty_bundle_id: TreatyBundleId,
    #[serde(flatten)]
    pub citation: V25ConstitutionCitation,
    #[serde(default)]
    pub replay_requirements: Vec<ReplayRequirementV1>,
    #[serde(default)]
    pub missing_required_runtime_names: Vec<String>,
    pub all_required_replays_available: bool,
    pub degraded: bool,
    pub advisory_only: bool,
    pub generated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct SharedDivergenceReportV1 {
    pub schema_version: String,
    pub shared_divergence_report_id: SharedDivergenceReportId,
    pub settlement_case_id: SettlementCaseId,
    pub treaty_bundle_id: TreatyBundleId,
    pub shared_replay_slice_id: SharedReplaySliceId,
    #[serde(flatten)]
    pub citation: V25ConstitutionCitation,
    #[serde(default)]
    pub blocking_runtime_names: Vec<String>,
    pub divergence_summary: String,
    pub requires_local_dissent_preservation: bool,
    pub recommends_suspension: bool,
    pub advisory_only: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct TreatySuspensionV1 {
    pub schema_version: String,
    pub treaty_suspension_id: TreatySuspensionId,
    pub treaty_bundle_id: TreatyBundleId,
    pub shared_divergence_report_id: SharedDivergenceReportId,
    pub suspension_scope: String,
    pub suspension_reason: String,
    #[serde(default)]
    pub blocked_runtime_names: Vec<String>,
    #[serde(default)]
    pub resumption_requirements: Vec<String>,
    pub advisory_only: bool,
    pub generated_at: String,
}

pub fn evaluate_shared_replay(
    case: &SettlementCaseV1,
    generated_at: impl Into<String>,
) -> SharedReplaySliceV1 {
    let missing_required_runtime_names = case
        .replay_requirements
        .iter()
        .filter(|requirement| requirement.required && !requirement.replay_available)
        .map(|requirement| requirement.runtime_name.clone())
        .collect::<Vec<_>>();

    SharedReplaySliceV1 {
        schema_version: SHARED_REPLAY_SLICE_V1_SCHEMA.into(),
        shared_replay_slice_id: SharedReplaySliceId::generate(),
        settlement_case_id: case.settlement_case_id.clone(),
        treaty_bundle_id: case.treaty_bundle_id.clone(),
        citation: case.citation.clone(),
        replay_requirements: case.replay_requirements.clone(),
        all_required_replays_available: missing_required_runtime_names.is_empty(),
        degraded: !missing_required_runtime_names.is_empty(),
        missing_required_runtime_names,
        advisory_only: true,
        generated_at: generated_at.into(),
    }
}

pub fn evaluate_divergence_or_suspension(
    case: &SettlementCaseV1,
    replay: &SharedReplaySliceV1,
    divergence_summary: impl Into<String>,
    generated_at: impl Into<String>,
) -> (SharedDivergenceReportV1, Option<TreatySuspensionV1>) {
    let generated_at = generated_at.into();
    let divergence_summary = divergence_summary.into();
    let mut blocking_runtime_names = case
        .local_dissent
        .iter()
        .filter(|dissent| dissent.blocks_shared_settlement)
        .map(|dissent| dissent.runtime_name.clone())
        .collect::<Vec<_>>();

    for runtime_name in &replay.missing_required_runtime_names {
        if !blocking_runtime_names.contains(runtime_name) {
            blocking_runtime_names.push(runtime_name.clone());
        }
    }

    let recommends_suspension = case.non_admitted
        || !blocking_runtime_names.is_empty()
        || (replay.degraded
            && case
                .local_dissent
                .iter()
                .any(|dissent| dissent.blocks_shared_settlement));

    let report = SharedDivergenceReportV1 {
        schema_version: SHARED_DIVERGENCE_REPORT_V1_SCHEMA.into(),
        shared_divergence_report_id: SharedDivergenceReportId::generate(),
        settlement_case_id: case.settlement_case_id.clone(),
        treaty_bundle_id: case.treaty_bundle_id.clone(),
        citation: case.citation.clone(),
        shared_replay_slice_id: replay.shared_replay_slice_id.clone(),
        blocking_runtime_names: blocking_runtime_names.clone(),
        divergence_summary,
        requires_local_dissent_preservation: !case.local_dissent.is_empty(),
        recommends_suspension,
        advisory_only: true,
    };

    let suspension = recommends_suspension.then(|| TreatySuspensionV1 {
        schema_version: TREATY_SUSPENSION_V1_SCHEMA.into(),
        treaty_suspension_id: TreatySuspensionId::generate(),
        treaty_bundle_id: case.treaty_bundle_id.clone(),
        shared_divergence_report_id: report.shared_divergence_report_id.clone(),
        suspension_scope: "bounded-settlement-family".into(),
        suspension_reason:
            "shared settlement remains blocked until replay gaps or blocking dissent are resolved"
                .into(),
        blocked_runtime_names: blocking_runtime_names,
        resumption_requirements: vec![
            "publish a replay-complete shared replay slice".into(),
            "preserve local dissent visibility in the next settlement review".into(),
            "re-run settlement evaluation before any admission action".into(),
        ],
        advisory_only: true,
        generated_at,
    });

    (report, suspension)
}

pub fn evaluate_settlement(
    case: &SettlementCaseV1,
    generated_at: impl Into<String>,
) -> SettlementReceiptV1 {
    let generated_at = generated_at.into();
    let replay = evaluate_shared_replay(case, generated_at.clone());

    if case.non_admitted {
        return SettlementReceiptV1 {
            schema_version: SETTLEMENT_RECEIPT_V1_SCHEMA.into(),
            settlement_receipt_id: SettlementReceiptId::generate(),
            settlement_case_id: case.settlement_case_id.clone(),
            citation: case.citation.clone(),
            disposition: SettlementDisposition::AdvisoryOnly,
            settlement_allowed: false,
            advisory_only: true,
            non_admitted: true,
            degraded: false,
            shared_disposition_id: None,
            downgrade: Some(SharedViewDowngradeV1 {
                schema_version: SHARED_VIEW_DOWNGRADE_V1_SCHEMA.into(),
                shared_view_downgrade_id: SharedViewDowngradeId::generate(),
                reason: DowngradeReason::NonAdmittedSurface,
                detail: "settlement remains non-admitted and cannot create shared authority".into(),
                preserves_local_dissent: true,
            }),
            local_dissent: case.local_dissent.clone(),
            generated_at,
        };
    }

    if replay.degraded {
        return SettlementReceiptV1 {
            schema_version: SETTLEMENT_RECEIPT_V1_SCHEMA.into(),
            settlement_receipt_id: SettlementReceiptId::generate(),
            settlement_case_id: case.settlement_case_id.clone(),
            citation: case.citation.clone(),
            disposition: SettlementDisposition::DegradedSharedView,
            settlement_allowed: false,
            advisory_only: true,
            non_admitted: false,
            degraded: true,
            shared_disposition_id: None,
            downgrade: Some(SharedViewDowngradeV1 {
                schema_version: SHARED_VIEW_DOWNGRADE_V1_SCHEMA.into(),
                shared_view_downgrade_id: SharedViewDowngradeId::generate(),
                reason: DowngradeReason::MissingRequiredReplay,
                detail: "missing replay on one side forces explicit downgrade instead of silent settlement".into(),
                preserves_local_dissent: true,
            }),
            local_dissent: case.local_dissent.clone(),
            generated_at,
        };
    }

    SettlementReceiptV1 {
        schema_version: SETTLEMENT_RECEIPT_V1_SCHEMA.into(),
        settlement_receipt_id: SettlementReceiptId::generate(),
        settlement_case_id: case.settlement_case_id.clone(),
        citation: case.citation.clone(),
        disposition: if case.advisory_only {
            SettlementDisposition::AdvisoryOnly
        } else {
            SettlementDisposition::SharedDispositionIssued
        },
        settlement_allowed: !case.advisory_only,
        advisory_only: case.advisory_only,
        non_admitted: false,
        degraded: false,
        shared_disposition_id: Some(case.shared_disposition.shared_disposition_id.clone()),
        downgrade: None,
        local_dissent: case.local_dissent.clone(),
        generated_at,
    }
}
