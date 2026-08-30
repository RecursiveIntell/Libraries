//! DR-05 deterministic 100-case admission pack.
//!
//! The static manifest is the declared inventory. Every entry names a semantic
//! scenario; no case ID or integer selector influences production behavior.
//! Each execution opens a fresh SQLite store, observes an owner-backed result,
//! normalizes only volatile identifiers/timestamps, and compares the resulting
//! ordered witness sequence across two complete runs.
#![allow(clippy::expect_used)]

use bitemporal_runtime::{append_supersede, as_of_query, BitemporalRecord};
use chrono::{DateTime, TimeZone, Utc};
use forge_memory_bridge::{
    ClaimState, ContradictionStatus, ImportClaimVersion, ImportProjectionRecord,
    ImportProjectionRecordV3, ProjectionFreshness as BridgeProjectionFreshness,
    ProjectionImportBatchV3, PROJECTION_IMPORT_BATCH_V3_SCHEMA,
};
use semantic_memory::{
    evaluate_governed_access_v1, AssertionDraftV1, AuthorityFaultStage, AuthorityPermit,
    AuthorityScopeV1, AuthorityScopesV1, CallerPrincipalV1, DelegationElevationLeaseV1,
    ElevationRequirementV1, EpisodeMeta, EpisodeOutcome, ForgettingClosureRequestV1,
    GovernedAccessPurposeV1, GovernedAccessRequestV1, MemoryConfig, MemoryError, MemoryStore,
    MemoryTransitionCandidateV1, MemoryTransitionOutcomeV1, MockEmbedder, NamespaceScopeV1,
    OriginAuthorityLabelV1, OriginClassV1, OriginDerivationKindV1, OriginRiskV1, ProjectionQuery,
    ReceiptMode, RevocationStatusV1, SearchContext, SourceArtifactV1, SourceSpanRefV1,
    StateDependencyEdgeV1, StateView, SubjectPrincipalV1, TransitionDisposition,
    TransitionOperation, VerificationStatus,
};
use semantic_memory_forge::{
    BilatticeTruthV1, ClaimStateV13, ContradictionWitnessV1, DegradationKindV1,
    EvidenceAdmissibilityV1, ExactnessLevelV1, QualityVectorV1, RetractionRecordV1, SemanticViewV1,
    SupportExprV1, SupportPolarityV1, SupportProvenanceKindV1, SupportSetV1, SupportTokenV1,
    CLAIM_STATE_V13_SCHEMA, CONTRADICTION_WITNESS_V1_SCHEMA, RETRACTION_RECORD_V1_SCHEMA,
    SUPPORT_SET_V1_SCHEMA,
};
use stack_ids::{
    ClaimId, ClaimStateId, ClaimVersionId, ContentDigest, ContradictionWitnessId, EntityId,
    EnvelopeId, RetractionRecordId, ScopeKey, SemanticsProfileId, SupportSetId,
};
use std::collections::{BTreeMap, BTreeSet};
use tempfile::TempDir;

const EARLY: i64 = 500;
const T0: i64 = 1_000;
const MID: i64 = 1_500;
const T1: i64 = 2_000;
const T2: i64 = 3_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Family {
    Temporal,
    Contradiction,
    Origin,
    Forgetting,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ExpectedOutcome {
    Applied,
    Allowed,
    Denied,
    Refused,
    RolledBack,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum WitnessCategory {
    AsOfState,
    ProjectionReceipt,
    AuthorityState,
    AccessDecision,
    ClosureState,
    TypedError,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum BasicProbe {
    BeforeValid,
    BeforeRecorded,
    InitialVisible,
    SupersededVisible,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ProgressionProbe {
    BeforeAny,
    First,
    Second,
    Third,
    RecordedCutoffRetainsSecond,
    ValidCutoffRetainsSecond,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RetroactiveProbe {
    BeforeOriginalValid,
    OriginalVisible,
    CorrectionAtEarlyValid,
    CorrectionAtLateValid,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RecordedReversalProbe {
    RootBeforeBackdate,
    BackdatedWinner,
    LaterRecordedWinner,
    BetweenRecordedCutoff,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ChainProbe {
    First,
    Second,
    Third,
    HistoricalFirst,
    HistoricalSecond,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum InterleaveProbe {
    OnlyFirstFamily,
    BothFamiliesBeforeUpdate,
    BothFamiliesAfterUpdate,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TieProbe {
    RootCutoff,
    FirstInsertedTieWinner,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TemporalScenario {
    BeforeFirst(BasicProbe),
    ValidProgression(ProgressionProbe),
    RetroactiveCorrection(RetroactiveProbe),
    RecordedReversal(RecordedReversalProbe),
    HistoricalChain(ChainProbe),
    MultiIdInterleave(InterleaveProbe),
    TwoWayTie(TieProbe),
    BranchTie,
    ThreeWayTie,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ContradictionScenario {
    SupportSetRoundTrip,
    ContradictionWitnessRoundTrip,
    RetractionRecordRoundTrip,
    ClaimStateRoundTrip,
    BothRemainsNonActionable,
    UnknownSupportSchemaRefused,
    MalformedV3MissingFieldRefused,
    MissingSupportTokensRefused,
    MissingClaimStateTxFromRefused,
    PreferredOpenConflict,
    InvalidTemporalOrder,
    OverlappingPreferredIntervals,
    AppendSupersedeCurrentHistorical,
    AppendRedactCurrentHistorical,
    SourceBackedQuarantine,
    AuthorityChangedPayloadConflict,
    ProjectionIdempotentRepeat,
    ProjectionChangedPayloadConflict,
    SourceBackedRetraction,
    TransitionFaultRollback,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum OriginScenario {
    NoOriginAppendDenied,
    SummaryNonElevation,
    RephraseNonElevation,
    ToolEchoNonElevation,
    CorroborationNonElevation,
    DirectIdDenied,
    SearchDenied,
    CachedSearchDenied,
    ExportDenied,
    ReplayDenied,
    RecallBoundaryAllowed,
    AssertionBoundaryDenied,
    ActionBoundaryDenied,
    OriginImmutable,
    RevocationAllPurposes,
    WrongAudienceDenied,
    WrongSubjectDenied,
    WrongNamespaceDenied,
    WrongPrincipalDenied,
    CallerLeaseReplayDenied,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ForgettingScenario {
    RootClosure,
    TransitiveClosure,
    CycleClosure,
    SharedClosure,
    UnrelatedSurvives,
    ZeroBudget,
    InsufficientBudget,
    WrongScope,
    WrongPrincipal,
    ExactRetry,
    ConflictingRetry,
    FaultBeforeMutation,
    FaultAfterMutation,
    FaultBeforeReceipt,
    FaultAfterReceipt,
    RawTombstone,
    GovernedRemoval,
    SearchRemoval,
    CacheRemoval,
    ReplayRemoval,
    ExportRemoval,
    CurrentViewRemoval,
    SupersededViewRemoval,
    HistoricalViewRemoval,
    ProjectionRemoval,
    EmbeddingRemoval,
    GraphRemoval,
    ReceiptNoPlaintext,
    OwnerRebuildNonResurrection,
    UnsupportedDerivedKindRefused,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Scenario {
    Temporal(TemporalScenario),
    Contradiction(ContradictionScenario),
    Origin(OriginScenario),
    Forgetting(ForgettingScenario),
}

#[derive(Clone, Copy, Debug)]
struct Case {
    id: &'static str,
    family: Family,
    owner_seam: &'static str,
    scenario: Scenario,
    expected_outcome: ExpectedOutcome,
    expected_witness: WitnessCategory,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct CaseWitness {
    id: &'static str,
    family: Family,
    owner_seam: &'static str,
    outcome: ExpectedOutcome,
    category: WitnessCategory,
    observed: Vec<String>,
}

macro_rules! case {
    ($id:literal, Temporal, $scenario:expr, $outcome:ident, $witness:ident) => {
        Case {
            id: $id,
            family: Family::Temporal,
            owner_seam: "bitemporal-runtime::as_of_query + MemoryStore::episode_as_of",
            scenario: Scenario::Temporal($scenario),
            expected_outcome: ExpectedOutcome::$outcome,
            expected_witness: WitnessCategory::$witness,
        }
    };
    ($id:literal, Contradiction, $scenario:expr, $outcome:ident, $witness:ident) => {
        Case {
            id: $id,
            family: Family::Contradiction,
            owner_seam: "Forge V13 + bridge V3 projection + append-only authority",
            scenario: Scenario::Contradiction($scenario),
            expected_outcome: ExpectedOutcome::$outcome,
            expected_witness: WitnessCategory::$witness,
        }
    };
    ($id:literal, Origin, $scenario:expr, $outcome:ident, $witness:ident) => {
        Case {
            id: $id,
            family: Family::Origin,
            owner_seam: "OriginAuthorityLabelV1 + governed access",
            scenario: Scenario::Origin($scenario),
            expected_outcome: ExpectedOutcome::$outcome,
            expected_witness: WitnessCategory::$witness,
        }
    };
    ($id:literal, Forgetting, $scenario:expr, $outcome:ident, $witness:ident) => {
        Case {
            id: $id,
            family: Family::Forgetting,
            owner_seam: "MemoryAuthority::forget dependency closure",
            scenario: Scenario::Forgetting($scenario),
            expected_outcome: ExpectedOutcome::$outcome,
            expected_witness: WitnessCategory::$witness,
        }
    };
}

const CASES: &[Case] = &[
    case!(
        "T01",
        Temporal,
        TemporalScenario::BeforeFirst(BasicProbe::BeforeValid),
        Applied,
        AsOfState
    ),
    case!(
        "T02",
        Temporal,
        TemporalScenario::BeforeFirst(BasicProbe::BeforeRecorded),
        Applied,
        AsOfState
    ),
    case!(
        "T03",
        Temporal,
        TemporalScenario::BeforeFirst(BasicProbe::InitialVisible),
        Applied,
        AsOfState
    ),
    case!(
        "T04",
        Temporal,
        TemporalScenario::BeforeFirst(BasicProbe::SupersededVisible),
        Applied,
        AsOfState
    ),
    case!(
        "T05",
        Temporal,
        TemporalScenario::ValidProgression(ProgressionProbe::BeforeAny),
        Applied,
        AsOfState
    ),
    case!(
        "T06",
        Temporal,
        TemporalScenario::ValidProgression(ProgressionProbe::First),
        Applied,
        AsOfState
    ),
    case!(
        "T07",
        Temporal,
        TemporalScenario::ValidProgression(ProgressionProbe::Second),
        Applied,
        AsOfState
    ),
    case!(
        "T08",
        Temporal,
        TemporalScenario::ValidProgression(ProgressionProbe::Third),
        Applied,
        AsOfState
    ),
    case!(
        "T09",
        Temporal,
        TemporalScenario::ValidProgression(ProgressionProbe::RecordedCutoffRetainsSecond),
        Applied,
        AsOfState
    ),
    case!(
        "T10",
        Temporal,
        TemporalScenario::ValidProgression(ProgressionProbe::ValidCutoffRetainsSecond),
        Applied,
        AsOfState
    ),
    case!(
        "T11",
        Temporal,
        TemporalScenario::RetroactiveCorrection(RetroactiveProbe::BeforeOriginalValid),
        Applied,
        AsOfState
    ),
    case!(
        "T12",
        Temporal,
        TemporalScenario::RetroactiveCorrection(RetroactiveProbe::OriginalVisible),
        Applied,
        AsOfState
    ),
    case!(
        "T13",
        Temporal,
        TemporalScenario::RetroactiveCorrection(RetroactiveProbe::CorrectionAtEarlyValid),
        Applied,
        AsOfState
    ),
    case!(
        "T14",
        Temporal,
        TemporalScenario::RetroactiveCorrection(RetroactiveProbe::CorrectionAtLateValid),
        Applied,
        AsOfState
    ),
    case!(
        "T15",
        Temporal,
        TemporalScenario::RecordedReversal(RecordedReversalProbe::RootBeforeBackdate),
        Applied,
        AsOfState
    ),
    case!(
        "T16",
        Temporal,
        TemporalScenario::RecordedReversal(RecordedReversalProbe::BackdatedWinner),
        Applied,
        AsOfState
    ),
    case!(
        "T17",
        Temporal,
        TemporalScenario::RecordedReversal(RecordedReversalProbe::LaterRecordedWinner),
        Applied,
        AsOfState
    ),
    case!(
        "T18",
        Temporal,
        TemporalScenario::RecordedReversal(RecordedReversalProbe::BetweenRecordedCutoff),
        Applied,
        AsOfState
    ),
    case!(
        "T19",
        Temporal,
        TemporalScenario::HistoricalChain(ChainProbe::First),
        Applied,
        AsOfState
    ),
    case!(
        "T20",
        Temporal,
        TemporalScenario::HistoricalChain(ChainProbe::Second),
        Applied,
        AsOfState
    ),
    case!(
        "T21",
        Temporal,
        TemporalScenario::HistoricalChain(ChainProbe::Third),
        Applied,
        AsOfState
    ),
    case!(
        "T22",
        Temporal,
        TemporalScenario::HistoricalChain(ChainProbe::HistoricalFirst),
        Applied,
        AsOfState
    ),
    case!(
        "T23",
        Temporal,
        TemporalScenario::HistoricalChain(ChainProbe::HistoricalSecond),
        Applied,
        AsOfState
    ),
    case!(
        "T24",
        Temporal,
        TemporalScenario::MultiIdInterleave(InterleaveProbe::OnlyFirstFamily),
        Applied,
        AsOfState
    ),
    case!(
        "T25",
        Temporal,
        TemporalScenario::MultiIdInterleave(InterleaveProbe::BothFamiliesBeforeUpdate),
        Applied,
        AsOfState
    ),
    case!(
        "T26",
        Temporal,
        TemporalScenario::MultiIdInterleave(InterleaveProbe::BothFamiliesAfterUpdate),
        Applied,
        AsOfState
    ),
    case!(
        "T27",
        Temporal,
        TemporalScenario::TwoWayTie(TieProbe::RootCutoff),
        Applied,
        AsOfState
    ),
    case!(
        "T28",
        Temporal,
        TemporalScenario::TwoWayTie(TieProbe::FirstInsertedTieWinner),
        Applied,
        AsOfState
    ),
    case!(
        "T29",
        Temporal,
        TemporalScenario::BranchTie,
        Applied,
        AsOfState
    ),
    case!(
        "T30",
        Temporal,
        TemporalScenario::ThreeWayTie,
        Applied,
        AsOfState
    ),
    case!(
        "C01",
        Contradiction,
        ContradictionScenario::SupportSetRoundTrip,
        Applied,
        ProjectionReceipt
    ),
    case!(
        "C02",
        Contradiction,
        ContradictionScenario::ContradictionWitnessRoundTrip,
        Applied,
        ProjectionReceipt
    ),
    case!(
        "C03",
        Contradiction,
        ContradictionScenario::RetractionRecordRoundTrip,
        Applied,
        ProjectionReceipt
    ),
    case!(
        "C04",
        Contradiction,
        ContradictionScenario::ClaimStateRoundTrip,
        Applied,
        ProjectionReceipt
    ),
    case!(
        "C05",
        Contradiction,
        ContradictionScenario::BothRemainsNonActionable,
        Denied,
        ProjectionReceipt
    ),
    case!(
        "C06",
        Contradiction,
        ContradictionScenario::UnknownSupportSchemaRefused,
        Refused,
        TypedError
    ),
    case!(
        "C07",
        Contradiction,
        ContradictionScenario::MissingSupportTokensRefused,
        Refused,
        TypedError
    ),
    case!(
        "C08",
        Contradiction,
        ContradictionScenario::MalformedV3MissingFieldRefused,
        Refused,
        TypedError
    ),
    case!(
        "C09",
        Contradiction,
        ContradictionScenario::MissingClaimStateTxFromRefused,
        Refused,
        TypedError
    ),
    case!(
        "C10",
        Contradiction,
        ContradictionScenario::PreferredOpenConflict,
        Refused,
        TypedError
    ),
    case!(
        "C11",
        Contradiction,
        ContradictionScenario::InvalidTemporalOrder,
        Refused,
        TypedError
    ),
    case!(
        "C12",
        Contradiction,
        ContradictionScenario::OverlappingPreferredIntervals,
        Refused,
        TypedError
    ),
    case!(
        "C13",
        Contradiction,
        ContradictionScenario::AppendSupersedeCurrentHistorical,
        Applied,
        AuthorityState
    ),
    case!(
        "C14",
        Contradiction,
        ContradictionScenario::AppendRedactCurrentHistorical,
        Applied,
        AuthorityState
    ),
    case!(
        "C15",
        Contradiction,
        ContradictionScenario::SourceBackedQuarantine,
        Refused,
        AuthorityState
    ),
    case!(
        "C16",
        Contradiction,
        ContradictionScenario::AuthorityChangedPayloadConflict,
        Refused,
        TypedError
    ),
    case!(
        "C17",
        Contradiction,
        ContradictionScenario::ProjectionIdempotentRepeat,
        Applied,
        ProjectionReceipt
    ),
    case!(
        "C18",
        Contradiction,
        ContradictionScenario::ProjectionChangedPayloadConflict,
        Refused,
        TypedError
    ),
    case!(
        "C19",
        Contradiction,
        ContradictionScenario::SourceBackedRetraction,
        Applied,
        AuthorityState
    ),
    case!(
        "C20",
        Contradiction,
        ContradictionScenario::TransitionFaultRollback,
        RolledBack,
        TypedError
    ),
    case!(
        "O01",
        Origin,
        OriginScenario::NoOriginAppendDenied,
        Denied,
        TypedError
    ),
    case!(
        "O02",
        Origin,
        OriginScenario::SummaryNonElevation,
        Denied,
        AccessDecision
    ),
    case!(
        "O03",
        Origin,
        OriginScenario::RephraseNonElevation,
        Denied,
        AccessDecision
    ),
    case!(
        "O04",
        Origin,
        OriginScenario::ToolEchoNonElevation,
        Denied,
        AccessDecision
    ),
    case!(
        "O05",
        Origin,
        OriginScenario::CorroborationNonElevation,
        Denied,
        AccessDecision
    ),
    case!(
        "O06",
        Origin,
        OriginScenario::DirectIdDenied,
        Denied,
        AccessDecision
    ),
    case!(
        "O07",
        Origin,
        OriginScenario::SearchDenied,
        Denied,
        AccessDecision
    ),
    case!(
        "O08",
        Origin,
        OriginScenario::CachedSearchDenied,
        Denied,
        AccessDecision
    ),
    case!(
        "O09",
        Origin,
        OriginScenario::ExportDenied,
        Denied,
        AccessDecision
    ),
    case!(
        "O10",
        Origin,
        OriginScenario::ReplayDenied,
        Denied,
        AccessDecision
    ),
    case!(
        "O11",
        Origin,
        OriginScenario::RecallBoundaryAllowed,
        Allowed,
        AccessDecision
    ),
    case!(
        "O12",
        Origin,
        OriginScenario::AssertionBoundaryDenied,
        Denied,
        AccessDecision
    ),
    case!(
        "O13",
        Origin,
        OriginScenario::ActionBoundaryDenied,
        Denied,
        AccessDecision
    ),
    case!(
        "O14",
        Origin,
        OriginScenario::OriginImmutable,
        Applied,
        AuthorityState
    ),
    case!(
        "O15",
        Origin,
        OriginScenario::RevocationAllPurposes,
        Denied,
        AccessDecision
    ),
    case!(
        "O16",
        Origin,
        OriginScenario::WrongAudienceDenied,
        Denied,
        AccessDecision
    ),
    case!(
        "O17",
        Origin,
        OriginScenario::WrongSubjectDenied,
        Denied,
        AccessDecision
    ),
    case!(
        "O18",
        Origin,
        OriginScenario::WrongNamespaceDenied,
        Denied,
        AccessDecision
    ),
    case!(
        "O19",
        Origin,
        OriginScenario::WrongPrincipalDenied,
        Denied,
        AccessDecision
    ),
    case!(
        "O20",
        Origin,
        OriginScenario::CallerLeaseReplayDenied,
        Denied,
        AccessDecision
    ),
    case!(
        "F01",
        Forgetting,
        ForgettingScenario::RootClosure,
        Applied,
        ClosureState
    ),
    case!(
        "F02",
        Forgetting,
        ForgettingScenario::TransitiveClosure,
        Applied,
        ClosureState
    ),
    case!(
        "F03",
        Forgetting,
        ForgettingScenario::CycleClosure,
        Applied,
        ClosureState
    ),
    case!(
        "F04",
        Forgetting,
        ForgettingScenario::SharedClosure,
        Applied,
        ClosureState
    ),
    case!(
        "F05",
        Forgetting,
        ForgettingScenario::UnrelatedSurvives,
        Applied,
        ClosureState
    ),
    case!(
        "F06",
        Forgetting,
        ForgettingScenario::ZeroBudget,
        Refused,
        TypedError
    ),
    case!(
        "F07",
        Forgetting,
        ForgettingScenario::InsufficientBudget,
        Refused,
        TypedError
    ),
    case!(
        "F08",
        Forgetting,
        ForgettingScenario::WrongScope,
        Refused,
        TypedError
    ),
    case!(
        "F09",
        Forgetting,
        ForgettingScenario::WrongPrincipal,
        Refused,
        TypedError
    ),
    case!(
        "F10",
        Forgetting,
        ForgettingScenario::ExactRetry,
        Applied,
        ClosureState
    ),
    case!(
        "F11",
        Forgetting,
        ForgettingScenario::ConflictingRetry,
        Refused,
        TypedError
    ),
    case!(
        "F12",
        Forgetting,
        ForgettingScenario::FaultBeforeMutation,
        RolledBack,
        TypedError
    ),
    case!(
        "F13",
        Forgetting,
        ForgettingScenario::FaultAfterMutation,
        RolledBack,
        TypedError
    ),
    case!(
        "F14",
        Forgetting,
        ForgettingScenario::FaultBeforeReceipt,
        RolledBack,
        TypedError
    ),
    case!(
        "F15",
        Forgetting,
        ForgettingScenario::FaultAfterReceipt,
        RolledBack,
        TypedError
    ),
    case!(
        "F16",
        Forgetting,
        ForgettingScenario::RawTombstone,
        Applied,
        ClosureState
    ),
    case!(
        "F17",
        Forgetting,
        ForgettingScenario::GovernedRemoval,
        Applied,
        ClosureState
    ),
    case!(
        "F18",
        Forgetting,
        ForgettingScenario::SearchRemoval,
        Applied,
        ClosureState
    ),
    case!(
        "F19",
        Forgetting,
        ForgettingScenario::CacheRemoval,
        Applied,
        ClosureState
    ),
    case!(
        "F20",
        Forgetting,
        ForgettingScenario::ReplayRemoval,
        Applied,
        ClosureState
    ),
    case!(
        "F21",
        Forgetting,
        ForgettingScenario::ExportRemoval,
        Applied,
        ClosureState
    ),
    case!(
        "F22",
        Forgetting,
        ForgettingScenario::CurrentViewRemoval,
        Applied,
        ClosureState
    ),
    case!(
        "F23",
        Forgetting,
        ForgettingScenario::SupersededViewRemoval,
        Applied,
        ClosureState
    ),
    case!(
        "F24",
        Forgetting,
        ForgettingScenario::HistoricalViewRemoval,
        Applied,
        ClosureState
    ),
    case!(
        "F25",
        Forgetting,
        ForgettingScenario::ProjectionRemoval,
        Applied,
        ClosureState
    ),
    case!(
        "F26",
        Forgetting,
        ForgettingScenario::EmbeddingRemoval,
        Applied,
        ClosureState
    ),
    case!(
        "F27",
        Forgetting,
        ForgettingScenario::GraphRemoval,
        Applied,
        ClosureState
    ),
    case!(
        "F28",
        Forgetting,
        ForgettingScenario::ReceiptNoPlaintext,
        Applied,
        ClosureState
    ),
    case!(
        "F29",
        Forgetting,
        ForgettingScenario::OwnerRebuildNonResurrection,
        Applied,
        ClosureState
    ),
    case!(
        "F30",
        Forgetting,
        ForgettingScenario::UnsupportedDerivedKindRefused,
        Refused,
        TypedError
    ),
];

const EXPECTED_IDS: &[&str] = &[
    "T01", "T02", "T03", "T04", "T05", "T06", "T07", "T08", "T09", "T10", "T11", "T12", "T13",
    "T14", "T15", "T16", "T17", "T18", "T19", "T20", "T21", "T22", "T23", "T24", "T25", "T26",
    "T27", "T28", "T29", "T30", "C01", "C02", "C03", "C04", "C05", "C06", "C07", "C08", "C09",
    "C10", "C11", "C12", "C13", "C14", "C15", "C16", "C17", "C18", "C19", "C20", "O01", "O02",
    "O03", "O04", "O05", "O06", "O07", "O08", "O09", "O10", "O11", "O12", "O13", "O14", "O15",
    "O16", "O17", "O18", "O19", "O20", "F01", "F02", "F03", "F04", "F05", "F06", "F07", "F08",
    "F09", "F10", "F11", "F12", "F13", "F14", "F15", "F16", "F17", "F18", "F19", "F20", "F21",
    "F22", "F23", "F24", "F25", "F26", "F27", "F28", "F29", "F30",
];

fn validate_manifest(cases: &[Case]) -> Result<(), String> {
    if cases.len() != 100 {
        return Err(format!("expected 100 cases, found {}", cases.len()));
    }
    let ids = cases.iter().map(|case| case.id).collect::<Vec<_>>();
    if ids != EXPECTED_IDS {
        return Err(format!("manifest IDs/order differ: {ids:?}"));
    }
    if ids.iter().copied().collect::<BTreeSet<_>>().len() != 100 {
        return Err("manifest contains duplicate IDs".into());
    }
    let mut counts = BTreeMap::new();
    for case in cases {
        *counts.entry(case.family).or_insert(0usize) += 1;
        let actual = match case.scenario {
            Scenario::Temporal(_) => Family::Temporal,
            Scenario::Contradiction(_) => Family::Contradiction,
            Scenario::Origin(_) => Family::Origin,
            Scenario::Forgetting(_) => Family::Forgetting,
        };
        if actual != case.family {
            return Err(format!("{} scenario/family mismatch", case.id));
        }
    }
    for (family, expected) in [
        (Family::Temporal, 30),
        (Family::Contradiction, 20),
        (Family::Origin, 20),
        (Family::Forgetting, 30),
    ] {
        if counts.get(&family) != Some(&expected) {
            return Err(format!("wrong {family:?} count: {:?}", counts.get(&family)));
        }
    }
    Ok(())
}

fn test_store() -> (MemoryStore, TempDir) {
    let temp = TempDir::new().expect("tempdir");
    let config = MemoryConfig {
        base_dir: temp.path().to_path_buf(),
        ..Default::default()
    };
    let store = MemoryStore::open_with_embedder(
        config.clone(),
        Box::new(MockEmbedder::new(config.embedding.dimensions)),
    )
    .expect("open isolated store");
    (store, temp)
}

fn at(seconds: i64) -> DateTime<Utc> {
    Utc.timestamp_opt(seconds, 0).single().expect("fixed time")
}

#[derive(Clone)]
struct TemporalVersion {
    valid: i64,
    recorded: i64,
    value: &'static str,
}
#[derive(Clone)]
struct TemporalFamily {
    id: &'static str,
    versions: Vec<TemporalVersion>,
}
struct TemporalPlan {
    families: Vec<TemporalFamily>,
    valid_cutoff: i64,
    recorded_cutoff: i64,
    expected: &'static [(&'static str, &'static str)],
}

fn temporal_plan(scenario: TemporalScenario) -> TemporalPlan {
    let basic = || TemporalFamily {
        id: "basic",
        versions: vec![
            TemporalVersion {
                valid: T0,
                recorded: T0,
                value: "alpha",
            },
            TemporalVersion {
                valid: T0,
                recorded: T1,
                value: "beta",
            },
        ],
    };
    let progression = || TemporalFamily {
        id: "progression",
        versions: vec![
            TemporalVersion {
                valid: T0,
                recorded: T0,
                value: "a",
            },
            TemporalVersion {
                valid: T1,
                recorded: T1,
                value: "b",
            },
            TemporalVersion {
                valid: T2,
                recorded: T2,
                value: "c",
            },
        ],
    };
    let retro = || TemporalFamily {
        id: "retro",
        versions: vec![
            TemporalVersion {
                valid: T2,
                recorded: T0,
                value: "original",
            },
            TemporalVersion {
                valid: T0,
                recorded: T1,
                value: "correction",
            },
        ],
    };
    let reversal = || TemporalFamily {
        id: "reversal",
        versions: vec![
            TemporalVersion {
                valid: T0,
                recorded: T0,
                value: "root",
            },
            TemporalVersion {
                valid: T1,
                recorded: T2,
                value: "later",
            },
            TemporalVersion {
                valid: T1,
                recorded: T1,
                value: "backdated",
            },
        ],
    };
    let chain = || TemporalFamily {
        id: "chain",
        versions: vec![
            TemporalVersion {
                valid: T0,
                recorded: T0,
                value: "a",
            },
            TemporalVersion {
                valid: T1,
                recorded: T1,
                value: "b",
            },
            TemporalVersion {
                valid: T2,
                recorded: T2,
                value: "c",
            },
        ],
    };
    match scenario {
        TemporalScenario::BeforeFirst(probe) => {
            let (valid_cutoff, recorded_cutoff, expected) = match probe {
                BasicProbe::BeforeValid => (EARLY, T2, &[][..]),
                BasicProbe::BeforeRecorded => (T0, EARLY, &[][..]),
                BasicProbe::InitialVisible => (T0, T0, &[("basic", "alpha")][..]),
                BasicProbe::SupersededVisible => (T0, T1, &[("basic", "beta")][..]),
            };
            TemporalPlan {
                families: vec![basic()],
                valid_cutoff,
                recorded_cutoff,
                expected,
            }
        }
        TemporalScenario::ValidProgression(probe) => {
            let (valid_cutoff, recorded_cutoff, expected) = match probe {
                ProgressionProbe::BeforeAny => (EARLY, EARLY, &[][..]),
                ProgressionProbe::First => (T0, T0, &[("progression", "a")][..]),
                ProgressionProbe::Second => (T1, T1, &[("progression", "b")][..]),
                ProgressionProbe::Third => (T2, T2, &[("progression", "c")][..]),
                ProgressionProbe::RecordedCutoffRetainsSecond => {
                    (T2, T1, &[("progression", "b")][..])
                }
                ProgressionProbe::ValidCutoffRetainsSecond => (T1, T2, &[("progression", "b")][..]),
            };
            TemporalPlan {
                families: vec![progression()],
                valid_cutoff,
                recorded_cutoff,
                expected,
            }
        }
        TemporalScenario::RetroactiveCorrection(probe) => {
            let (valid_cutoff, recorded_cutoff, expected) = match probe {
                RetroactiveProbe::BeforeOriginalValid => (T0, T0, &[][..]),
                RetroactiveProbe::OriginalVisible => (T2, T0, &[("retro", "original")][..]),
                RetroactiveProbe::CorrectionAtEarlyValid => {
                    (T0, T1, &[("retro", "correction")][..])
                }
                RetroactiveProbe::CorrectionAtLateValid => (T2, T1, &[("retro", "correction")][..]),
            };
            TemporalPlan {
                families: vec![retro()],
                valid_cutoff,
                recorded_cutoff,
                expected,
            }
        }
        TemporalScenario::RecordedReversal(probe) => {
            let (valid_cutoff, recorded_cutoff, expected) = match probe {
                RecordedReversalProbe::RootBeforeBackdate => (T0, T1, &[("reversal", "root")][..]),
                RecordedReversalProbe::BackdatedWinner => {
                    (T1, T1, &[("reversal", "backdated")][..])
                }
                RecordedReversalProbe::LaterRecordedWinner => {
                    (T1, T2, &[("reversal", "later")][..])
                }
                RecordedReversalProbe::BetweenRecordedCutoff => {
                    (T1, MID, &[("reversal", "root")][..])
                }
            };
            TemporalPlan {
                families: vec![reversal()],
                valid_cutoff,
                recorded_cutoff,
                expected,
            }
        }
        TemporalScenario::HistoricalChain(probe) => {
            let (valid_cutoff, recorded_cutoff, expected) = match probe {
                ChainProbe::First => (T0, T0, &[("chain", "a")][..]),
                ChainProbe::Second => (T1, T1, &[("chain", "b")][..]),
                ChainProbe::Third => (T2, T2, &[("chain", "c")][..]),
                ChainProbe::HistoricalFirst => (T2, T0, &[("chain", "a")][..]),
                ChainProbe::HistoricalSecond => (T2, T1, &[("chain", "b")][..]),
            };
            TemporalPlan {
                families: vec![chain()],
                valid_cutoff,
                recorded_cutoff,
                expected,
            }
        }
        TemporalScenario::MultiIdInterleave(probe) => {
            let families = vec![
                TemporalFamily {
                    id: "left",
                    versions: vec![
                        TemporalVersion {
                            valid: T0,
                            recorded: T0,
                            value: "left-1",
                        },
                        TemporalVersion {
                            valid: T1,
                            recorded: T2,
                            value: "left-2",
                        },
                    ],
                },
                TemporalFamily {
                    id: "right",
                    versions: vec![TemporalVersion {
                        valid: T1,
                        recorded: T1,
                        value: "right-1",
                    }],
                },
            ];
            let (valid_cutoff, recorded_cutoff, expected) = match probe {
                InterleaveProbe::OnlyFirstFamily => (T0, T0, &[("left", "left-1")][..]),
                InterleaveProbe::BothFamiliesBeforeUpdate => {
                    (T1, T1, &[("left", "left-1"), ("right", "right-1")][..])
                }
                InterleaveProbe::BothFamiliesAfterUpdate => {
                    (T1, T2, &[("left", "left-2"), ("right", "right-1")][..])
                }
            };
            TemporalPlan {
                families,
                valid_cutoff,
                recorded_cutoff,
                expected,
            }
        }
        TemporalScenario::TwoWayTie(probe) => {
            let family = TemporalFamily {
                id: "tie",
                versions: vec![
                    TemporalVersion {
                        valid: T0,
                        recorded: T0,
                        value: "root",
                    },
                    TemporalVersion {
                        valid: T1,
                        recorded: T1,
                        value: "first",
                    },
                    TemporalVersion {
                        valid: T1,
                        recorded: T1,
                        value: "second",
                    },
                ],
            };
            let (valid_cutoff, recorded_cutoff, expected) = match probe {
                TieProbe::RootCutoff => (T0, T1, &[("tie", "root")][..]),
                TieProbe::FirstInsertedTieWinner => (T1, T1, &[("tie", "first")][..]),
            };
            TemporalPlan {
                families: vec![family],
                valid_cutoff,
                recorded_cutoff,
                expected,
            }
        }
        TemporalScenario::BranchTie => TemporalPlan {
            families: vec![TemporalFamily {
                id: "branch",
                versions: vec![
                    TemporalVersion {
                        valid: T0,
                        recorded: T0,
                        value: "root",
                    },
                    TemporalVersion {
                        valid: T1,
                        recorded: T1,
                        value: "left",
                    },
                    TemporalVersion {
                        valid: T1,
                        recorded: T1,
                        value: "right",
                    },
                ],
            }],
            valid_cutoff: T1,
            recorded_cutoff: T2,
            expected: &[("branch", "left")],
        },
        TemporalScenario::ThreeWayTie => TemporalPlan {
            families: vec![TemporalFamily {
                id: "three-way",
                versions: vec![
                    TemporalVersion {
                        valid: T0,
                        recorded: T0,
                        value: "root",
                    },
                    TemporalVersion {
                        valid: T1,
                        recorded: T1,
                        value: "first",
                    },
                    TemporalVersion {
                        valid: T1,
                        recorded: T1,
                        value: "second",
                    },
                    TemporalVersion {
                        valid: T1,
                        recorded: T1,
                        value: "third",
                    },
                ],
            }],
            valid_cutoff: T1,
            recorded_cutoff: T1,
            expected: &[("three-way", "first")],
        },
    }
}

fn episode_meta(value: &str, valid: i64) -> EpisodeMeta {
    EpisodeMeta {
        cause_ids: vec![format!("cause-{value}")],
        effect_type: value.into(),
        outcome: EpisodeOutcome::Pending,
        confidence: 0.5,
        verification_status: VerificationStatus::Unverified,
        experiment_id: None,
        valid_time: Some(at(valid)),
        fact_digest: None,
    }
}

async fn execute_temporal(scenario: TemporalScenario) -> Vec<String> {
    let plan = temporal_plan(scenario);
    let (store, _temp) = test_store();
    let mut reference = Vec::new();
    let mut document_to_family = BTreeMap::new();
    for family in &plan.families {
        let document_id = store
            .ingest_document(
                family.id,
                "DR-05 temporal fixture",
                "dr05-temporal",
                None,
                None,
            )
            .await
            .expect("temporal document");
        document_to_family.insert(document_id.clone(), family.id);
        let mut predecessor = None;
        for (position, version) in family.versions.iter().enumerate() {
            let episode_id = format!("{}-{position}", family.id);
            store
                .append_episode_version(
                    &episode_id,
                    predecessor.as_deref(),
                    &document_id,
                    &episode_meta(version.value, version.valid),
                    Some(at(version.recorded)),
                )
                .await
                .expect("temporal append");
            predecessor = Some(episode_id);
            append_supersede(
                &mut reference,
                BitemporalRecord {
                    id: family.id.to_string(),
                    valid_time: at(version.valid),
                    recorded_time: at(version.recorded),
                    value: version.value.to_string(),
                },
            )
            .expect("reference append");
        }
    }
    let expected = plan
        .expected
        .iter()
        .map(|(id, value)| ((*id).to_string(), (*value).to_string()))
        .collect::<BTreeMap<_, _>>();
    let reference_winners =
        as_of_query(&reference, at(plan.valid_cutoff), at(plan.recorded_cutoff))
            .into_iter()
            .map(|row| (row.id, row.value))
            .collect::<BTreeMap<_, _>>();
    assert_eq!(reference_winners, expected, "hand-pinned reference winner");
    let (rows, receipt) = store
        .episode_as_of(at(plan.valid_cutoff), at(plan.recorded_cutoff))
        .await
        .expect("SQLite as-of");
    let sqlite_winners = rows
        .iter()
        .map(|row| {
            (
                document_to_family[&row.document_id].to_string(),
                row.meta.effect_type.clone(),
            )
        })
        .collect::<BTreeMap<_, _>>();
    assert_eq!(sqlite_winners, expected, "hand-pinned SQLite winner");
    assert_eq!(receipt.episode_count, rows.len());
    vec![
        format!("winners={sqlite_winners:?}"),
        format!("episode_count={}", receipt.episode_count),
        format!("excluded={}", receipt.excluded_superseded),
        format!("reference={reference_winners:?}"),
    ]
}

fn operator_permit(principal: &str, capability: &str) -> AuthorityPermit {
    AuthorityPermit::operator_system(principal, "dr05-pack", capability)
}

fn support_set() -> SupportSetV1 {
    SupportSetV1 {
        schema_version: SUPPORT_SET_V1_SCHEMA.into(),
        support_set_id: SupportSetId::new("support-dr05"),
        claim_id: ClaimId::new("claim-dr05"),
        semantics_profile_id: SemanticsProfileId::new("profile-dr05"),
        support_tokens: vec![
            SupportTokenV1 {
                token_id: "supports".into(),
                kind: SupportProvenanceKindV1::EvidenceRef,
                reference: "evidence:one".into(),
                polarity: SupportPolarityV1::Supports,
            },
            SupportTokenV1 {
                token_id: "refutes".into(),
                kind: SupportProvenanceKindV1::ClaimVersion,
                reference: "claim-version:prior".into(),
                polarity: SupportPolarityV1::Refutes,
            },
        ],
        support_expr: SupportExprV1::AnyOf {
            children: vec![
                SupportExprV1::Token {
                    token_id: "supports".into(),
                },
                SupportExprV1::Token {
                    token_id: "refutes".into(),
                },
            ],
        },
        content_digest: ContentDigest::compute(b"support-dr05"),
    }
}

fn contradiction_witness() -> ContradictionWitnessV1 {
    ContradictionWitnessV1 {
        schema_version: CONTRADICTION_WITNESS_V1_SCHEMA.into(),
        contradiction_witness_id: ContradictionWitnessId::new("witness-dr05"),
        claim_id: ClaimId::new("claim-dr05"),
        conflicting_token_ids: vec!["supports".into(), "refutes".into()],
        summary: Some("support and refutation coexist".into()),
    }
}

fn retraction_record() -> RetractionRecordV1 {
    RetractionRecordV1 {
        schema_version: RETRACTION_RECORD_V1_SCHEMA.into(),
        retraction_record_id: RetractionRecordId::new("retraction-dr05"),
        claim_id: ClaimId::new("claim-dr05"),
        retracted_claim_version_id: ClaimVersionId::new("claim-dr05-v0"),
        superseded_by_claim_version_id: Some(ClaimVersionId::new("claim-dr05-v1")),
        effective_recorded_at: "2026-08-28T00:00:00Z".into(),
        reason: "source-backed correction".into(),
        cascade_required: true,
        delta_summary: Some("rebuild dependent projections".into()),
    }
}

fn claim_state(truth: BilatticeTruthV1) -> ClaimStateV13 {
    let support = support_set();
    ClaimStateV13 {
        schema_version: CLAIM_STATE_V13_SCHEMA.into(),
        claim_state_id: ClaimStateId::new("claim-state-dr05"),
        claim_id: ClaimId::new("claim-dr05"),
        claim_version_id: Some(ClaimVersionId::new("claim-dr05-v1")),
        semantics_profile_id: SemanticsProfileId::new("profile-dr05"),
        view: SemanticViewV1::Canonical,
        bilattice_truth: truth,
        support_set_id: Some(support.support_set_id),
        support_set_digest: Some(support.content_digest),
        quality_vector: QualityVectorV1 {
            exactness: ExactnessLevelV1::Conservative,
            degradation: vec![DegradationKindV1::ExactnessDowngraded],
            freshness: Some("current".into()),
            replay_limited: false,
            execution_contaminated: false,
        },
        evidence_admissibility: EvidenceAdmissibilityV1::Admissible,
        contradiction_witness_id: Some(ContradictionWitnessId::new("witness-dr05")),
        valid_from: Some("2026-08-01T00:00:00Z".into()),
        valid_to: None,
        tx_from: "2026-08-28T00:00:00Z".into(),
        tx_to: None,
        proof_obligations_remaining: vec!["resolve contradiction".into()],
        policy_action_allowed: false,
    }
}

fn claim_record(tag: &str) -> ImportProjectionRecordV3 {
    ImportProjectionRecordV3 {
        record: ImportProjectionRecord::ClaimVersion(ImportClaimVersion {
            claim_id: ClaimId::new(format!("claim-{tag}")),
            claim_version_id: ClaimVersionId::new(format!("claim-{tag}-v1")),
            claim_state: ClaimState::Active,
            projection_family: "forge_verification".into(),
            subject_entity_id: EntityId::new(format!("entity-{tag}")),
            predicate: "supports".into(),
            object_anchor: serde_json::json!("compiler"),
            scope_key: ScopeKey::namespace_only("dr05-projection"),
            valid_from: None,
            valid_to: None,
            preferred_open: true,
            source_envelope_id: EnvelopeId::new(format!("envelope-{tag}")),
            source_authority: "forge".into(),
            trace_ctx: None,
            freshness: BridgeProjectionFreshness::Current,
            contradiction_status: ContradictionStatus::None,
            supersedes_claim_version_id: None,
            content: format!("projection content {tag}"),
            confidence: 0.9,
            metadata: None,
        }),
        semantics: None,
    }
}

fn v3_batch(tag: &str) -> ProjectionImportBatchV3 {
    ProjectionImportBatchV3 {
        source_envelope_id: EnvelopeId::new(format!("envelope-{tag}")),
        schema_version: PROJECTION_IMPORT_BATCH_V3_SCHEMA.into(),
        export_schema_version: Some("export_envelope_v3".into()),
        content_digest: ContentDigest::compute(tag.as_bytes()),
        source_authority: "forge".into(),
        scope_key: ScopeKey::namespace_only("dr05-projection"),
        trace_ctx: None,
        source_exported_at: "2026-08-28T00:00:00Z".into(),
        transformed_at: "2026-08-28T00:00:01Z".into(),
        export_meta: None,
        evidence_bundle: None,
        episode_bundle: None,
        execution_context: None,
        support_sets: vec![],
        contradiction_witnesses: vec![],
        retraction_records: vec![],
        claim_states_v13: vec![],
        intervention_bundles_v14: vec![],
        outcome_schemas_v14: vec![],
        cohort_contracts_v14: vec![],
        counterfactual_slices_v14: vec![],
        experiment_cases_v14: vec![],
        comparability_matrices_v14: vec![],
        decision_traces_v14: vec![],
        refuter_suites_v14: vec![],
        refuter_results_v14: vec![],
        experiment_budgets_v14: vec![],
        rollout_decisions_v14: vec![],
        rollback_decisions_v14: vec![],
        attestation_envelopes_v15: vec![],
        trust_root_sets_v15: vec![],
        artifact_admission_policies_v15: vec![],
        transparency_receipts_v15: vec![],
        attestation_revocations_v15: vec![],
        attestation_supersessions_v15: vec![],
        remote_oracle_leases_v15: vec![],
        remote_slice_requests_v15: vec![],
        remote_slice_results_v15: vec![],
        cross_runtime_replay_tickets_v15: vec![],
        dispute_bundles_v15: vec![],
        disclosure_policies_v15: vec![],
        disclosure_budgets_v15: vec![],
        records: vec![claim_record(tag)],
    }
}

async fn rebuilt_v3(
    store: &MemoryStore,
    batch: &ProjectionImportBatchV3,
) -> ProjectionImportBatchV3 {
    let result = store
        .import_projection_batch(batch)
        .await
        .expect("V3 import");
    assert_eq!(result.status, "complete");
    let logs = store
        .query_projection_imports(Some("dr05-projection"), 10)
        .await
        .expect("import logs");
    let log = logs
        .iter()
        .find(|entry| entry.source_envelope_id == batch.source_envelope_id.as_str())
        .expect("V3 log");
    log.rebuildable_kernel_batch_v3()
        .expect("rebuild receipt decode")
        .expect("V3 payload present")
}

fn transition_candidate(assertion: &str) -> MemoryTransitionCandidateV1 {
    let artifact = SourceArtifactV1::new("artifact:dr05", assertion).expect("artifact");
    let span = SourceSpanRefV1::new("artifact:dr05", 0, assertion.len()).expect("span");
    MemoryTransitionCandidateV1::new(
        "candidate-dr05",
        vec![artifact],
        vec![span.clone()],
        vec![
            AssertionDraftV1::new("assertion-dr05", "general", assertion, vec![span], vec![])
                .expect("assertion"),
        ],
        TransitionOperation::Append {
            assertion_id: "assertion-dr05".into(),
        },
        vec![],
    )
    .expect("candidate")
}

async fn execute_contradiction(scenario: ContradictionScenario) -> Vec<String> {
    let (store, _temp) = test_store();
    match scenario {
        ContradictionScenario::SupportSetRoundTrip => {
            let mut batch = v3_batch("support");
            batch.support_sets = vec![support_set()];
            let rebuilt = rebuilt_v3(&store, &batch).await;
            assert_eq!(rebuilt.support_sets, batch.support_sets);
            vec![
                format!("support_sets={}", rebuilt.support_sets.len()),
                format!("tokens={}", rebuilt.support_sets[0].support_tokens.len()),
            ]
        }
        ContradictionScenario::ContradictionWitnessRoundTrip => {
            let mut batch = v3_batch("witness");
            batch.contradiction_witnesses = vec![contradiction_witness()];
            let rebuilt = rebuilt_v3(&store, &batch).await;
            assert_eq!(
                rebuilt.contradiction_witnesses,
                batch.contradiction_witnesses
            );
            vec![
                format!("witnesses={}", rebuilt.contradiction_witnesses.len()),
                format!(
                    "conflicts={}",
                    rebuilt.contradiction_witnesses[0]
                        .conflicting_token_ids
                        .len()
                ),
            ]
        }
        ContradictionScenario::RetractionRecordRoundTrip => {
            let mut batch = v3_batch("retraction");
            batch.retraction_records = vec![retraction_record()];
            let rebuilt = rebuilt_v3(&store, &batch).await;
            assert_eq!(rebuilt.retraction_records, batch.retraction_records);
            vec![
                format!("retractions={}", rebuilt.retraction_records.len()),
                format!("cascade={}", rebuilt.retraction_records[0].cascade_required),
            ]
        }
        ContradictionScenario::ClaimStateRoundTrip => {
            let mut batch = v3_batch("state");
            batch.claim_states_v13 = vec![claim_state(BilatticeTruthV1::TrueOnly)];
            let rebuilt = rebuilt_v3(&store, &batch).await;
            assert_eq!(rebuilt.claim_states_v13, batch.claim_states_v13);
            vec![
                format!("states={}", rebuilt.claim_states_v13.len()),
                format!("truth={:?}", rebuilt.claim_states_v13[0].bilattice_truth),
            ]
        }
        ContradictionScenario::BothRemainsNonActionable => {
            let mut batch = v3_batch("both");
            batch.claim_states_v13 = vec![claim_state(BilatticeTruthV1::Both)];
            let rebuilt = rebuilt_v3(&store, &batch).await;
            let state = &rebuilt.claim_states_v13[0];
            assert_eq!(state.bilattice_truth, BilatticeTruthV1::Both);
            assert!(!state.policy_action_allowed, "Both must not elect a winner");
            vec![
                "truth=Both".into(),
                format!("policy_action_allowed={}", state.policy_action_allowed),
            ]
        }
        ContradictionScenario::UnknownSupportSchemaRefused => {
            let mut invalid = support_set();
            invalid.schema_version = "support_set_v99".into();
            let error = invalid
                .validate()
                .expect_err("unknown V13 schema must refuse");
            assert!(error.contains("schema_version mismatch"));
            assert!(store
                .query_projection_imports(None, 10)
                .await
                .expect("logs")
                .is_empty());
            vec!["error=schema_version_mismatch".into(), "imports=0".into()]
        }
        ContradictionScenario::MissingSupportTokensRefused => {
            let mut invalid = support_set();
            invalid.support_tokens.clear();
            let error = invalid
                .validate()
                .expect_err("missing support tokens must refuse");
            assert!(error.contains("support_tokens"));
            assert!(store
                .query_claim_versions(ProjectionQuery::new(ScopeKey::namespace_only(
                    "dr05-projection"
                )))
                .await
                .expect("claims")
                .is_empty());
            vec!["error=missing_support_tokens".into(), "claims=0".into()]
        }
        ContradictionScenario::MalformedV3MissingFieldRefused => {
            let mut encoded = serde_json::to_value(v3_batch("malformed-v3")).expect("encode V3");
            encoded
                .as_object_mut()
                .expect("V3 object")
                .remove("source_authority");
            let error = serde_json::from_value::<ProjectionImportBatchV3>(encoded)
                .expect_err("missing V3 source_authority must refuse");
            assert!(error.to_string().contains("source_authority"));
            assert!(store
                .query_projection_imports(None, 10)
                .await
                .expect("logs")
                .is_empty());
            vec![
                "error=missing_v3_source_authority".into(),
                "imports=0".into(),
            ]
        }
        ContradictionScenario::MissingClaimStateTxFromRefused => {
            let mut invalid = claim_state(BilatticeTruthV1::Unknown);
            invalid.tx_from.clear();
            let error = invalid.validate().expect_err("missing tx_from must refuse");
            assert!(error.contains("tx_from"));
            assert!(store
                .query_projection_imports(None, 10)
                .await
                .expect("logs")
                .is_empty());
            vec!["error=missing_tx_from".into(), "imports=0".into()]
        }
        ContradictionScenario::PreferredOpenConflict => {
            let first = v3_batch("preferred-first");
            store
                .import_projection_batch(&first)
                .await
                .expect("first preferred");
            let mut second = v3_batch("preferred-second");
            let ImportProjectionRecord::ClaimVersion(record) = &mut second.records[0].record else {
                unreachable!()
            };
            record.claim_id = ClaimId::new("claim-preferred-first");
            let error = store
                .import_projection_batch(&second)
                .await
                .expect_err("second preferred must conflict");
            assert_eq!(error.kind(), "import_invalid");
            let claims = store
                .query_claim_versions(ProjectionQuery::new(ScopeKey::namespace_only(
                    "dr05-projection",
                )))
                .await
                .expect("claims");
            assert_eq!(claims.len(), 1);
            vec![
                format!("error={}", error.kind()),
                format!("claims={}", claims.len()),
            ]
        }
        ContradictionScenario::InvalidTemporalOrder => {
            let mut batch = v3_batch("bad-time");
            let ImportProjectionRecord::ClaimVersion(record) = &mut batch.records[0].record else {
                unreachable!()
            };
            record.valid_from = Some("2026-09-01T00:00:00Z".into());
            record.valid_to = Some("2026-08-01T00:00:00Z".into());
            let error = store
                .import_projection_batch(&batch)
                .await
                .expect_err("invalid interval");
            assert_eq!(error.kind(), "import_invalid");
            assert!(store
                .query_claim_versions(ProjectionQuery::new(ScopeKey::namespace_only(
                    "dr05-projection"
                )))
                .await
                .expect("claims")
                .is_empty());
            vec![format!("error={}", error.kind()), "claims=0".into()]
        }
        ContradictionScenario::OverlappingPreferredIntervals => {
            let mut batch = v3_batch("overlap");
            let ImportProjectionRecord::ClaimVersion(first) = &mut batch.records[0].record else {
                unreachable!()
            };
            first.valid_from = Some("2026-08-01T00:00:00Z".into());
            first.valid_to = Some("2026-08-20T00:00:00Z".into());
            let mut second = batch.records[0].clone();
            let ImportProjectionRecord::ClaimVersion(record) = &mut second.record else {
                unreachable!()
            };
            record.claim_version_id = ClaimVersionId::new("claim-overlap-v2");
            record.valid_from = Some("2026-08-10T00:00:00Z".into());
            record.valid_to = Some("2026-08-30T00:00:00Z".into());
            batch.records.push(second);
            let error = store
                .import_projection_batch(&batch)
                .await
                .expect_err("overlap must refuse");
            assert_eq!(error.kind(), "import_invalid");
            assert!(store
                .query_claim_versions(ProjectionQuery::new(ScopeKey::namespace_only(
                    "dr05-projection"
                )))
                .await
                .expect("claims")
                .is_empty());
            vec![format!("error={}", error.kind()), "claims=0".into()]
        }
        ContradictionScenario::AppendSupersedeCurrentHistorical => {
            let authority = store.authority();
            let first = authority
                .append(
                    operator_permit("principal:dr05", AuthorityPermit::APPEND_CAPABILITY),
                    "append-current".into(),
                    "general".into(),
                    "old assertion".into(),
                    None,
                )
                .await
                .expect("append");
            let replacement = authority
                .supersede(
                    operator_permit("principal:dr05", AuthorityPermit::SUPERSEDE_CAPABILITY),
                    "supersede-current".into(),
                    first.affected_ids[0].clone(),
                    "new assertion".into(),
                    None,
                )
                .await
                .expect("supersede");
            let current = store.list_facts("general", 10, 0).await.expect("current");
            let historical = store
                .list_facts_with_view("general", 10, 0, StateView::IncludeSuperseded)
                .await
                .expect("historical");
            assert_eq!(
                current
                    .iter()
                    .map(|fact| fact.content.as_str())
                    .collect::<Vec<_>>(),
                vec!["new assertion"]
            );
            assert_eq!(historical.len(), 2);
            vec![
                format!("operation={:?}", replacement.operation_kind),
                format!("current={}", current.len()),
                format!("historical={}", historical.len()),
            ]
        }
        ContradictionScenario::AppendRedactCurrentHistorical => {
            let authority = store.authority();
            let first = authority
                .append(
                    operator_permit("principal:dr05", AuthorityPermit::APPEND_CAPABILITY),
                    "append-redact".into(),
                    "general".into(),
                    "sensitive assertion".into(),
                    None,
                )
                .await
                .expect("append");
            let receipt = authority
                .redact(
                    operator_permit("principal:dr05", AuthorityPermit::REDACT_CAPABILITY),
                    "redact-current".into(),
                    first.affected_ids[0].clone(),
                    "privacy request".into(),
                )
                .await
                .expect("redact");
            let current = store.list_facts("general", 10, 0).await.expect("current");
            let historical = store
                .list_facts_with_view("general", 10, 0, StateView::IncludeSuperseded)
                .await
                .expect("historical");
            assert_eq!(current[0].content, "[REDACTED]");
            assert_eq!(historical.len(), 2);
            vec![
                format!("operation={:?}", receipt.operation_kind),
                format!("current_content={}", current[0].content),
                format!("historical={}", historical.len()),
            ]
        }
        ContradictionScenario::SourceBackedQuarantine => {
            let authority = store.authority();
            let artifact = SourceArtifactV1::new("artifact:quarantine", "supported statement")
                .expect("artifact");
            let span = SourceSpanRefV1::new("artifact:quarantine", 0, 19).expect("span");
            let candidate = MemoryTransitionCandidateV1::new(
                "candidate-quarantine",
                vec![artifact],
                vec![span.clone()],
                vec![AssertionDraftV1::new(
                    "assertion-quarantine",
                    "general",
                    "supported statement plus invention",
                    vec![span],
                    vec![],
                )
                .expect("assertion")],
                TransitionOperation::Append {
                    assertion_id: "assertion-quarantine".into(),
                },
                vec![],
            )
            .expect("candidate");
            let outcome = authority
                .verify_and_commit(
                    operator_permit("principal:dr05", AuthorityPermit::APPEND_CAPABILITY),
                    "source-quarantine".into(),
                    candidate,
                )
                .await
                .expect("quarantine outcome");
            let MemoryTransitionOutcomeV1::Quarantined { record } = outcome else {
                panic!("unsupported assertion must quarantine")
            };
            assert_eq!(
                record.verification.disposition,
                TransitionDisposition::Quarantine
            );
            assert!(!record.verification.unsupported_spans.is_empty());
            let facts = store.list_facts("general", 10, 0).await.expect("facts");
            assert!(facts.is_empty());
            vec![
                format!("disposition={:?}", record.verification.disposition),
                format!(
                    "unsupported={}",
                    record.verification.unsupported_spans.len()
                ),
                format!("facts={}", facts.len()),
            ]
        }
        ContradictionScenario::AuthorityChangedPayloadConflict => {
            let authority = store.authority();
            authority
                .append(
                    operator_permit("principal:dr05", AuthorityPermit::APPEND_CAPABILITY),
                    "authority-conflict".into(),
                    "general".into(),
                    "first payload".into(),
                    None,
                )
                .await
                .expect("first");
            let error = authority
                .append(
                    operator_permit("principal:dr05", AuthorityPermit::APPEND_CAPABILITY),
                    "authority-conflict".into(),
                    "general".into(),
                    "changed payload".into(),
                    None,
                )
                .await
                .expect_err("changed payload conflict");
            assert!(matches!(
                error,
                MemoryError::AuthorityIdempotencyConflict { .. }
            ));
            assert_eq!(
                store
                    .list_facts("general", 10, 0)
                    .await
                    .expect("facts")
                    .len(),
                1
            );
            vec![format!("error={}", error.kind()), "facts=1".into()]
        }
        ContradictionScenario::ProjectionIdempotentRepeat => {
            let batch = v3_batch("projection-repeat");
            let first = store.import_projection_batch(&batch).await.expect("first");
            let repeat = store.import_projection_batch(&batch).await.expect("repeat");
            assert_eq!(first.status, "complete");
            assert_eq!(repeat.status, "already_imported");
            assert!(repeat.was_duplicate);
            vec![
                format!("first={}", first.status),
                format!("repeat={}", repeat.status),
                format!("duplicate={}", repeat.was_duplicate),
            ]
        }
        ContradictionScenario::ProjectionChangedPayloadConflict => {
            let first = v3_batch("projection-conflict");
            store.import_projection_batch(&first).await.expect("first");
            let mut changed = first.clone();
            changed.content_digest = ContentDigest::compute(b"changed-projection-payload");
            let error = store
                .import_projection_batch(&changed)
                .await
                .expect_err("changed digest conflict");
            assert_eq!(error.kind(), "import_migration_required");
            let claims = store
                .query_claim_versions(ProjectionQuery::new(ScopeKey::namespace_only(
                    "dr05-projection",
                )))
                .await
                .expect("claims");
            assert_eq!(claims.len(), 1);
            vec![
                format!("error={}", error.kind()),
                format!("claims={}", claims.len()),
            ]
        }
        ContradictionScenario::SourceBackedRetraction => {
            let authority = store.authority();
            let seeded = authority
                .append(
                    operator_permit("principal:dr05", AuthorityPermit::APPEND_CAPABILITY),
                    "seed-retraction".into(),
                    "general".into(),
                    "sensitive assertion".into(),
                    None,
                )
                .await
                .expect("seed");
            let evidence = "operator-approved retraction";
            let span =
                SourceSpanRefV1::new("artifact:retraction", 0, evidence.len()).expect("span");
            let candidate = MemoryTransitionCandidateV1::new(
                "candidate-retraction",
                vec![SourceArtifactV1::new("artifact:retraction", evidence).expect("artifact")],
                vec![span.clone()],
                vec![],
                TransitionOperation::Retract {
                    target_fact_id: seeded.affected_ids[0].clone(),
                    reason: evidence.into(),
                    source_spans: vec![span],
                },
                vec![],
            )
            .expect("candidate");
            let outcome = authority
                .verify_and_commit(
                    operator_permit("principal:dr05", AuthorityPermit::REDACT_CAPABILITY),
                    "source-retraction".into(),
                    candidate,
                )
                .await
                .expect("transition");
            let MemoryTransitionOutcomeV1::Committed {
                verification,
                authority_receipt,
                ..
            } = outcome
            else {
                panic!("retraction must commit")
            };
            assert_eq!(verification.disposition, TransitionDisposition::Commit);
            assert_eq!(
                store.list_facts("general", 10, 0).await.expect("facts")[0].content,
                "[REDACTED]"
            );
            vec![
                format!("disposition={:?}", verification.disposition),
                format!("operation={:?}", authority_receipt.operation_kind),
                "current=[REDACTED]".into(),
            ]
        }
        ContradictionScenario::TransitionFaultRollback => {
            let authority = store.authority();
            authority.set_fault(Some(AuthorityFaultStage::AfterAppend));
            let error = authority
                .verify_and_commit(
                    operator_permit("principal:dr05", AuthorityPermit::APPEND_CAPABILITY),
                    "transition-fault".into(),
                    transition_candidate("source-backed fact"),
                )
                .await
                .expect_err("fault");
            assert!(matches!(
                error,
                MemoryError::AuthorityFaultInjected {
                    stage: AuthorityFaultStage::AfterAppend
                }
            ));
            assert!(store
                .list_facts("general", 10, 0)
                .await
                .expect("facts")
                .is_empty());
            assert!(authority
                .get_transition_by_idempotency_key("transition-fault")
                .await
                .expect("transition lookup")
                .is_none());
            vec![
                format!("error={}", error.kind()),
                "facts=0".into(),
                "transition=absent".into(),
            ]
        }
    }
}

fn restricted_origin() -> OriginAuthorityLabelV1 {
    OriginAuthorityLabelV1::new(
        OriginClassV1::ExternalEvidence,
        "principal:alice",
        "dr05-source",
        "blake3:dr05-source",
        OriginRiskV1::High,
        AuthorityScopesV1 {
            recall: AuthorityScopeV1::Audience,
            assertion: AuthorityScopeV1::Denied,
            action: AuthorityScopeV1::Denied,
        },
        ElevationRequirementV1::Never,
        None,
        RevocationStatusV1::Active,
        vec!["principal:alice".into(), "team:dr05".into()],
    )
    .expect("restricted origin")
}

fn origin_permit(label: OriginAuthorityLabelV1) -> AuthorityPermit {
    AuthorityPermit::with_evidence(
        "principal:alice",
        "dr05-pack",
        AuthorityPermit::APPEND_CAPABILITY,
        vec![format!("blake3:{}", "a".repeat(64))],
    )
    .with_origin(label)
}

fn request_for(
    principal: &str,
    purpose: GovernedAccessPurposeV1,
    namespace: &str,
) -> GovernedAccessRequestV1 {
    GovernedAccessRequestV1::new(principal, principal, purpose, namespace)
}

async fn append_restricted_origin(store: &MemoryStore) -> String {
    store
        .authority()
        .append(
            origin_permit(restricted_origin()),
            "origin-append".into(),
            "general".into(),
            "origin governed sentinel".into(),
            None,
        )
        .await
        .expect("origin append")
        .affected_ids[0]
        .clone()
}

fn scoped_origin_label() -> OriginAuthorityLabelV1 {
    restricted_origin()
        .with_subject_principal(SubjectPrincipalV1::new("principal:patient").expect("subject"))
        .with_resource_scope(NamespaceScopeV1::exact("medical"))
}

fn scoped_request() -> GovernedAccessRequestV1 {
    GovernedAccessRequestV1::for_principals(
        CallerPrincipalV1::new("principal:alice").expect("caller"),
        SubjectPrincipalV1::new("principal:patient").expect("subject"),
        vec!["principal:alice".into(), "team:dr05".into()],
        GovernedAccessPurposeV1::Recall,
        NamespaceScopeV1::exact("medical"),
    )
}

fn normalize_decision(decision: &semantic_memory::OriginAuthorityDecisionV1) -> Vec<String> {
    vec![
        format!("allowed={}", decision.allowed),
        format!("outcome={:?}", decision.outcome),
        format!("purpose={:?}", decision.purpose),
        format!("reasons={:?}", decision.reasons),
        format!("revoked={}", decision.revocation_reference.is_some()),
    ]
}

async fn execute_origin(scenario: OriginScenario) -> Vec<String> {
    let (store, _temp) = test_store();
    match scenario {
        OriginScenario::NoOriginAppendDenied => {
            let error = store
                .authority()
                .append(
                    AuthorityPermit::with_evidence(
                        "principal:alice",
                        "dr05",
                        AuthorityPermit::APPEND_CAPABILITY,
                        vec![format!("blake3:{}", "b".repeat(64))],
                    ),
                    "no-origin".into(),
                    "general".into(),
                    "must not persist".into(),
                    None,
                )
                .await
                .expect_err("no-origin append must deny");
            assert!(matches!(error, MemoryError::OriginAuthorityRejected { .. }));
            let raw = store
                .add_fact_raw_compat("general", "raw compatibility boundary", None, None, None)
                .await
                .expect("raw compat");
            let governed = store
                .authority()
                .get_fact_governed(
                    &raw.id,
                    request_for(
                        "principal:alice",
                        GovernedAccessPurposeV1::Recall,
                        "general",
                    ),
                )
                .await
                .expect("governed raw");
            assert!(
                !governed.decision.allowed,
                "raw compatibility is not authorization"
            );
            vec![
                format!("error={}", error.kind()),
                "canonical_writes=0".into(),
                format!("raw_governed_allowed={}", governed.decision.allowed),
            ]
        }
        OriginScenario::SummaryNonElevation
        | OriginScenario::RephraseNonElevation
        | OriginScenario::ToolEchoNonElevation
        | OriginScenario::CorroborationNonElevation => {
            let kind = match scenario {
                OriginScenario::SummaryNonElevation => OriginDerivationKindV1::Summary,
                OriginScenario::RephraseNonElevation => OriginDerivationKindV1::Rephrase,
                OriginScenario::ToolEchoNonElevation => OriginDerivationKindV1::TrustedToolEcho,
                OriginScenario::CorroborationNonElevation => OriginDerivationKindV1::Corroboration,
                _ => unreachable!(),
            };
            let strong = OriginAuthorityLabelV1::operator_system("principal:alice", "operator");
            let derived = OriginAuthorityLabelV1::derive(
                &[restricted_origin(), strong],
                kind,
                "blake3:derived",
            )
            .expect("derive");
            assert_eq!(derived.risk, OriginRiskV1::High);
            assert_eq!(derived.scopes.recall, AuthorityScopeV1::Audience);
            assert_eq!(derived.scopes.assertion, AuthorityScopeV1::Denied);
            assert_eq!(derived.scopes.action, AuthorityScopeV1::Denied);
            assert_eq!(derived.elevation, ElevationRequirementV1::Never);
            vec![
                format!("kind={kind:?}"),
                format!("risk={:?}", derived.risk),
                format!("recall={:?}", derived.scopes.recall),
                format!("assertion={:?}", derived.scopes.assertion),
                format!("action={:?}", derived.scopes.action),
                format!("elevation={:?}", derived.elevation),
            ]
        }
        OriginScenario::DirectIdDenied => {
            let id = append_restricted_origin(&store).await;
            let result = store
                .authority()
                .get_fact_governed(
                    &id,
                    request_for("principal:bob", GovernedAccessPurposeV1::Recall, "general"),
                )
                .await
                .expect("direct access");
            assert!(result.fact.is_none());
            normalize_decision(&result.decision)
        }
        OriginScenario::SearchDenied => {
            append_restricted_origin(&store).await;
            let result = store
                .authority()
                .search_governed(
                    "origin governed sentinel",
                    Some(8),
                    request_for("principal:bob", GovernedAccessPurposeV1::Recall, "general"),
                )
                .await
                .expect("search");
            assert!(result.results.is_empty());
            let decision = result
                .decisions
                .iter()
                .find(|decision| !decision.allowed)
                .expect("denial");
            let mut witness = normalize_decision(decision);
            witness.push(format!("results={}", result.results.len()));
            witness
        }
        OriginScenario::CachedSearchDenied => {
            append_restricted_origin(&store).await;
            let request = request_for("principal:bob", GovernedAccessPurposeV1::Recall, "general");
            let first = store
                .authority()
                .search_governed("origin governed sentinel", Some(8), request.clone())
                .await
                .expect("first search");
            let cached = store
                .authority()
                .search_governed("origin governed sentinel", Some(8), request)
                .await
                .expect("cached search");
            assert!(first.results.is_empty() && cached.results.is_empty());
            assert!(first.decisions.iter().all(|decision| !decision.allowed));
            assert!(cached.decisions.iter().all(|decision| !decision.allowed));
            vec![
                format!("first_results={}", first.results.len()),
                format!("cached_results={}", cached.results.len()),
                format!("cached_denials={}", cached.decisions.len()),
            ]
        }
        OriginScenario::ExportDenied => {
            let id = append_restricted_origin(&store).await;
            let result = store
                .authority()
                .export_fact_governed(
                    &id,
                    request_for("principal:bob", GovernedAccessPurposeV1::Recall, "general"),
                )
                .await
                .expect("export");
            assert!(result.fact.is_none());
            assert_eq!(result.decision.purpose, GovernedAccessPurposeV1::Export);
            normalize_decision(&result.decision)
        }
        OriginScenario::ReplayDenied => {
            append_restricted_origin(&store).await;
            let mut context = SearchContext::default_now();
            context.receipt_mode = ReceiptMode::ReturnReceipt;
            let receipt = store
                .search_with_context(
                    "origin governed sentinel",
                    Some(8),
                    Some(&["general"]),
                    None,
                    context,
                )
                .await
                .expect("source search")
                .receipt
                .expect("receipt");
            let result = store
                .authority()
                .replay_search_receipt_governed(
                    &receipt.receipt_id,
                    "origin governed sentinel",
                    Some(8),
                    request_for("principal:bob", GovernedAccessPurposeV1::Recall, "general"),
                )
                .await
                .expect("replay");
            assert!(result.allowed_result_ids.is_empty());
            let decision = result
                .decisions
                .iter()
                .find(|decision| !decision.allowed)
                .expect("denial");
            let mut witness = normalize_decision(decision);
            witness.push(format!(
                "allowed_results={}",
                result.allowed_result_ids.len()
            ));
            witness
        }
        OriginScenario::RecallBoundaryAllowed => {
            let id = append_restricted_origin(&store).await;
            let result = store
                .authority()
                .get_fact_governed(
                    &id,
                    request_for(
                        "principal:alice",
                        GovernedAccessPurposeV1::Recall,
                        "general",
                    ),
                )
                .await
                .expect("recall");
            assert!(result.fact.is_some());
            assert!(result.decision.allowed);
            normalize_decision(&result.decision)
        }
        OriginScenario::AssertionBoundaryDenied => {
            let id = append_restricted_origin(&store).await;
            let result = store
                .authority()
                .get_fact_governed(
                    &id,
                    request_for(
                        "principal:alice",
                        GovernedAccessPurposeV1::Assertion,
                        "general",
                    ),
                )
                .await
                .expect("assertion");
            assert!(result.fact.is_none());
            normalize_decision(&result.decision)
        }
        OriginScenario::ActionBoundaryDenied => {
            let id = append_restricted_origin(&store).await;
            let result = store
                .authority()
                .get_fact_governed(
                    &id,
                    request_for(
                        "principal:alice",
                        GovernedAccessPurposeV1::Action,
                        "general",
                    ),
                )
                .await
                .expect("action");
            assert!(result.fact.is_none());
            normalize_decision(&result.decision)
        }
        OriginScenario::OriginImmutable => {
            let id = append_restricted_origin(&store).await;
            let before = store
                .authority()
                .get_origin_authority(&id)
                .await
                .expect("origin")
                .expect("origin exists");
            let update = store
                .raw_execute(
                    "UPDATE origin_authority_labels SET label_digest = 'forged' WHERE fact_id = ?1",
                    vec![id.clone()],
                )
                .await;
            assert!(
                update.is_err(),
                "immutable origin trigger must reject update"
            );
            let after = store
                .authority()
                .get_origin_authority(&id)
                .await
                .expect("origin after")
                .expect("origin exists");
            assert_eq!(before, after);
            vec![
                format!("label_unchanged={}", before == after),
                format!("update_refused={}", update.is_err()),
                format!(
                    "digest_matches={}",
                    before.label_digest == after.label_digest
                ),
            ]
        }
        OriginScenario::RevocationAllPurposes => {
            let id = append_restricted_origin(&store).await;
            let before = store
                .authority()
                .get_origin_authority(&id)
                .await
                .expect("origin")
                .expect("origin");
            store
                .authority()
                .revoke_origin(
                    AuthorityPermit::operator_system(
                        "principal:alice",
                        "dr05",
                        AuthorityPermit::REVOKE_ORIGIN_CAPABILITY,
                    ),
                    "revoke-all".into(),
                    &id,
                    "revocation:dr05".into(),
                )
                .await
                .expect("revoke");
            let after = store
                .authority()
                .get_origin_authority(&id)
                .await
                .expect("origin")
                .expect("origin");
            assert_eq!(before, after, "write-time label remains immutable");
            let mut denied = Vec::new();
            for purpose in [
                GovernedAccessPurposeV1::Recall,
                GovernedAccessPurposeV1::Assertion,
                GovernedAccessPurposeV1::Action,
                GovernedAccessPurposeV1::Export,
                GovernedAccessPurposeV1::Replay,
                GovernedAccessPurposeV1::Admin,
            ] {
                let result = store
                    .authority()
                    .get_fact_governed(&id, request_for("principal:alice", purpose, "general"))
                    .await
                    .expect("revoked access");
                assert!(!result.decision.allowed);
                assert!(result.decision.revocation_reference.is_some());
                denied.push(format!("{purpose:?}"));
            }
            vec![
                format!("denied={denied:?}"),
                "label_unchanged=true".into(),
                "revocation_reference=true".into(),
            ]
        }
        OriginScenario::WrongAudienceDenied => {
            let request = scoped_request().with_audiences(vec!["team:other".into()]);
            let decision = evaluate_governed_access_v1(
                "fact:patient",
                Some("medical"),
                Some(&scoped_origin_label()),
                None,
                &request,
            );
            assert!(!decision.allowed);
            assert!(decision
                .reasons
                .iter()
                .any(|reason| reason == "audience_intersection_empty"));
            normalize_decision(&decision)
        }
        OriginScenario::WrongSubjectDenied => {
            let request = GovernedAccessRequestV1::for_principals(
                CallerPrincipalV1::new("principal:alice").expect("caller"),
                SubjectPrincipalV1::new("principal:other-patient").expect("subject"),
                vec!["principal:alice".into(), "team:dr05".into()],
                GovernedAccessPurposeV1::Recall,
                NamespaceScopeV1::exact("medical"),
            );
            let decision = evaluate_governed_access_v1(
                "fact:patient",
                Some("medical"),
                Some(&scoped_origin_label()),
                None,
                &request,
            );
            assert!(!decision.allowed);
            normalize_decision(&decision)
        }
        OriginScenario::WrongNamespaceDenied => {
            let mut request = scoped_request();
            request.scope = NamespaceScopeV1::exact("other");
            let decision = evaluate_governed_access_v1(
                "fact:patient",
                Some("medical"),
                Some(&scoped_origin_label()),
                None,
                &request,
            );
            assert!(!decision.allowed);
            assert!(decision
                .reasons
                .iter()
                .any(|reason| reason == "request_policy_digest_mismatch"
                    || reason == "namespace_scope_mismatch"));
            normalize_decision(&decision)
        }
        OriginScenario::WrongPrincipalDenied => {
            let request = GovernedAccessRequestV1::for_principals(
                CallerPrincipalV1::new("principal:bob").expect("caller"),
                SubjectPrincipalV1::new("principal:patient").expect("subject"),
                vec!["principal:bob".into()],
                GovernedAccessPurposeV1::Recall,
                NamespaceScopeV1::exact("medical"),
            );
            let decision = evaluate_governed_access_v1(
                "fact:patient",
                Some("medical"),
                Some(&scoped_origin_label()),
                None,
                &request,
            );
            assert!(!decision.allowed);
            normalize_decision(&decision)
        }
        OriginScenario::CallerLeaseReplayDenied => {
            let request = scoped_request().with_delegation_or_elevation(
                DelegationElevationLeaseV1::delegation(
                    "lease:caller-carried",
                    "principal:patient",
                    "principal:alice",
                    vec![GovernedAccessPurposeV1::Recall],
                    NamespaceScopeV1::exact("medical"),
                    vec!["team:dr05".into()],
                    "2999-01-01T00:00:00Z",
                ),
            );
            let first = evaluate_governed_access_v1(
                "fact:patient",
                Some("medical"),
                Some(&scoped_origin_label()),
                None,
                &request,
            );
            let replay = evaluate_governed_access_v1(
                "fact:patient",
                Some("medical"),
                Some(&scoped_origin_label()),
                None,
                &request,
            );
            assert!(!first.allowed && !replay.allowed);
            assert!(first
                .reasons
                .iter()
                .any(|reason| reason == "untrusted_caller_carried_lease"));
            assert_eq!(first.decision_digest, replay.decision_digest);
            vec![
                format!("allowed={}", first.allowed),
                format!("outcome={:?}", first.outcome),
                "reason=untrusted_caller_carried_lease".into(),
                format!(
                    "replay_stable={}",
                    first.decision_digest == replay.decision_digest
                ),
            ]
        }
    }
}

async fn append_forgetting(
    store: &MemoryStore,
    principal: &str,
    key: &str,
    content: &str,
) -> String {
    store
        .authority()
        .append(
            operator_permit(principal, AuthorityPermit::APPEND_CAPABILITY),
            key.into(),
            "private".into(),
            content.into(),
            Some("dr05-forgetting".into()),
        )
        .await
        .expect("forgetting append")
        .affected_ids[0]
        .clone()
}

fn forgetting_request(
    root: &str,
    namespace: &str,
    reason: &str,
    budget: usize,
) -> ForgettingClosureRequestV1 {
    ForgettingClosureRequestV1::new(vec![root.to_string()], namespace, reason, budget)
}

fn forgetting_access() -> GovernedAccessRequestV1 {
    request_for("principal:dr05", GovernedAccessPurposeV1::Recall, "private")
}

async fn add_dependency(store: &MemoryStore, derived: &str, source: &str) {
    store
        .add_state_dependency_edge(
            StateDependencyEdgeV1::derived_from_state(
                format!("fact:{derived}"),
                format!("fact:{source}"),
            ),
            1.0,
        )
        .await
        .expect("dependency");
}

async fn forget_root(
    store: &MemoryStore,
    key: &str,
    root: &str,
    reason: &str,
    budget: usize,
) -> semantic_memory::ForgettingClosureReceiptV1 {
    store
        .authority()
        .forget(
            operator_permit("principal:dr05", AuthorityPermit::FORGET_CAPABILITY),
            key.into(),
            forgetting_request(root, "private", reason, budget),
        )
        .await
        .expect("forget")
}

fn normalized_closure(receipt: &semantic_memory::ForgettingClosureReceiptV1) -> Vec<String> {
    let removed = receipt
        .removed_surfaces
        .iter()
        .map(|surface| surface.surface.clone())
        .collect::<BTreeSet<_>>();
    let invalidated = receipt
        .invalidated_surfaces
        .iter()
        .map(|surface| surface.surface.clone())
        .collect::<BTreeSet<_>>();
    vec![
        format!("affected={}", receipt.affected_canonical_ids.len()),
        format!("removed={removed:?}"),
        format!("invalidated={invalidated:?}"),
        format!(
            "verified={}",
            receipt.verification.iter().all(|check| check.passed)
        ),
        format!(
            "epoch_delta={}",
            receipt.after_epoch.0 - receipt.before_epoch.0
        ),
    ]
}

async fn execute_forgetting(scenario: ForgettingScenario) -> Vec<String> {
    let (store, _temp) = test_store();
    match scenario {
        ForgettingScenario::RootClosure => {
            let root =
                append_forgetting(&store, "principal:dr05", "root-only", "root-only canary").await;
            let receipt = forget_root(&store, "forget-root", &root, "erase root", 8).await;
            assert_eq!(receipt.affected_canonical_ids, vec![root]);
            normalized_closure(&receipt)
        }
        ForgettingScenario::TransitiveClosure => {
            let root = append_forgetting(
                &store,
                "principal:dr05",
                "transitive-root",
                "transitive root",
            )
            .await;
            let child = append_forgetting(
                &store,
                "principal:dr05",
                "transitive-child",
                "transitive child",
            )
            .await;
            let grandchild = append_forgetting(
                &store,
                "principal:dr05",
                "transitive-grandchild",
                "transitive grandchild",
            )
            .await;
            add_dependency(&store, &child, &root).await;
            add_dependency(&store, &grandchild, &child).await;
            let receipt =
                forget_root(&store, "forget-transitive", &root, "erase transitive", 8).await;
            assert_eq!(
                receipt
                    .affected_canonical_ids
                    .iter()
                    .cloned()
                    .collect::<BTreeSet<_>>(),
                [root, child, grandchild].into_iter().collect()
            );
            normalized_closure(&receipt)
        }
        ForgettingScenario::CycleClosure => {
            let root =
                append_forgetting(&store, "principal:dr05", "cycle-root", "cycle root").await;
            let left =
                append_forgetting(&store, "principal:dr05", "cycle-left", "cycle left").await;
            let right =
                append_forgetting(&store, "principal:dr05", "cycle-right", "cycle right").await;
            add_dependency(&store, &left, &root).await;
            add_dependency(&store, &right, &left).await;
            add_dependency(&store, &left, &right).await;
            let receipt = forget_root(&store, "forget-cycle", &root, "erase cycle", 8).await;
            assert_eq!(
                receipt
                    .affected_canonical_ids
                    .iter()
                    .cloned()
                    .collect::<BTreeSet<_>>(),
                [root, left, right].into_iter().collect()
            );
            normalized_closure(&receipt)
        }
        ForgettingScenario::SharedClosure => {
            let root =
                append_forgetting(&store, "principal:dr05", "shared-root", "shared root").await;
            let other_root = append_forgetting(
                &store,
                "principal:dr05",
                "shared-other",
                "other root survives",
            )
            .await;
            let shared =
                append_forgetting(&store, "principal:dr05", "shared-derived", "shared derived")
                    .await;
            add_dependency(&store, &shared, &root).await;
            add_dependency(&store, &shared, &other_root).await;
            let receipt = forget_root(&store, "forget-shared", &root, "erase shared", 8).await;
            assert!(receipt.affected_canonical_ids.contains(&shared));
            assert_eq!(
                store
                    .get_fact_raw_compat(&other_root)
                    .await
                    .expect("other")
                    .expect("other exists")
                    .content,
                "other root survives"
            );
            let mut witness = normalized_closure(&receipt);
            witness.push("other_root=survives".into());
            witness
        }
        ForgettingScenario::UnrelatedSurvives => {
            let root = append_forgetting(
                &store,
                "principal:dr05",
                "unrelated-root",
                "erase this root",
            )
            .await;
            let unrelated = append_forgetting(
                &store,
                "principal:dr05",
                "unrelated-control",
                "unrelated survives",
            )
            .await;
            let receipt = forget_root(&store, "forget-unrelated", &root, "erase root", 8).await;
            let control = store
                .get_fact_raw_compat(&unrelated)
                .await
                .expect("control")
                .expect("control exists");
            assert_eq!(control.content, "unrelated survives");
            let mut witness = normalized_closure(&receipt);
            witness.push(format!("control={}", control.content));
            witness
        }
        ForgettingScenario::ZeroBudget => {
            let root = append_forgetting(
                &store,
                "principal:dr05",
                "zero-budget",
                "zero budget canary",
            )
            .await;
            let error = store
                .authority()
                .forget(
                    operator_permit("principal:dr05", AuthorityPermit::FORGET_CAPABILITY),
                    "forget-zero".into(),
                    forgetting_request(&root, "private", "zero budget", 0),
                )
                .await
                .expect_err("zero budget");
            assert!(matches!(
                error,
                MemoryError::ForgettingBudgetExceeded { budget: 0, .. }
            ));
            let content = store
                .get_fact_raw_compat(&root)
                .await
                .expect("root")
                .expect("root exists")
                .content;
            assert_eq!(content, "zero budget canary");
            vec![
                format!("error={}", error.kind()),
                format!("content={content}"),
            ]
        }
        ForgettingScenario::InsufficientBudget => {
            let root = append_forgetting(
                &store,
                "principal:dr05",
                "small-budget-root",
                "small budget root",
            )
            .await;
            let child = append_forgetting(
                &store,
                "principal:dr05",
                "small-budget-child",
                "small budget child",
            )
            .await;
            add_dependency(&store, &child, &root).await;
            let error = store
                .authority()
                .forget(
                    operator_permit("principal:dr05", AuthorityPermit::FORGET_CAPABILITY),
                    "forget-small".into(),
                    forgetting_request(&root, "private", "small budget", 1),
                )
                .await
                .expect_err("insufficient budget");
            assert!(matches!(
                error,
                MemoryError::ForgettingBudgetExceeded { .. }
            ));
            assert_eq!(
                store
                    .get_fact_raw_compat(&root)
                    .await
                    .expect("root")
                    .expect("root exists")
                    .content,
                "small budget root"
            );
            assert_eq!(
                store
                    .get_fact_raw_compat(&child)
                    .await
                    .expect("child")
                    .expect("child exists")
                    .content,
                "small budget child"
            );
            vec![
                format!("error={}", error.kind()),
                "root=unchanged".into(),
                "child=unchanged".into(),
            ]
        }
        ForgettingScenario::WrongScope => {
            let root = append_forgetting(
                &store,
                "principal:dr05",
                "wrong-scope",
                "wrong scope canary",
            )
            .await;
            let error = store
                .authority()
                .forget(
                    operator_permit("principal:dr05", AuthorityPermit::FORGET_CAPABILITY),
                    "forget-wrong-scope".into(),
                    forgetting_request(&root, "other", "wrong scope", 8),
                )
                .await
                .expect_err("wrong scope");
            assert!(matches!(
                error,
                MemoryError::ForgettingClosureIncomplete { .. }
            ));
            assert_eq!(
                store
                    .get_fact_raw_compat(&root)
                    .await
                    .expect("root")
                    .expect("root exists")
                    .content,
                "wrong scope canary"
            );
            vec![
                format!("error={}", error.kind()),
                "content=unchanged".into(),
            ]
        }
        ForgettingScenario::WrongPrincipal => {
            let root = append_forgetting(
                &store,
                "principal:other",
                "wrong-principal",
                "wrong principal canary",
            )
            .await;
            let error = store
                .authority()
                .forget(
                    operator_permit("principal:dr05", AuthorityPermit::FORGET_CAPABILITY),
                    "forget-wrong-principal".into(),
                    forgetting_request(&root, "private", "wrong principal", 8),
                )
                .await
                .expect_err("wrong principal");
            assert!(matches!(
                error,
                MemoryError::ForgettingClosureIncomplete { .. }
            ));
            assert_eq!(
                store
                    .get_fact_raw_compat(&root)
                    .await
                    .expect("root")
                    .expect("root exists")
                    .content,
                "wrong principal canary"
            );
            vec![
                format!("error={}", error.kind()),
                "content=unchanged".into(),
            ]
        }
        ForgettingScenario::ExactRetry => {
            let root =
                append_forgetting(&store, "principal:dr05", "retry-root", "retry canary").await;
            let request = forgetting_request(&root, "private", "exact retry", 8);
            let first = store
                .authority()
                .forget(
                    operator_permit("principal:dr05", AuthorityPermit::FORGET_CAPABILITY),
                    "forget-retry".into(),
                    request.clone(),
                )
                .await
                .expect("first");
            let replay = store
                .authority()
                .forget(
                    operator_permit("principal:dr05", AuthorityPermit::FORGET_CAPABILITY),
                    "forget-retry".into(),
                    request,
                )
                .await
                .expect("replay");
            assert_eq!(first, replay);
            let mut witness = normalized_closure(&replay);
            witness.push("same_receipt=true".into());
            witness
        }
        ForgettingScenario::ConflictingRetry => {
            let root =
                append_forgetting(&store, "principal:dr05", "conflict-root", "conflict canary")
                    .await;
            store
                .authority()
                .forget(
                    operator_permit("principal:dr05", AuthorityPermit::FORGET_CAPABILITY),
                    "forget-conflict".into(),
                    forgetting_request(&root, "private", "first reason", 8),
                )
                .await
                .expect("first");
            let error = store
                .authority()
                .forget(
                    operator_permit("principal:dr05", AuthorityPermit::FORGET_CAPABILITY),
                    "forget-conflict".into(),
                    forgetting_request(&root, "private", "changed reason", 8),
                )
                .await
                .expect_err("conflict");
            assert!(matches!(
                error,
                MemoryError::AuthorityIdempotencyConflict { .. }
            ));
            vec![format!("error={}", error.kind()), "receipt_count=1".into()]
        }
        ForgettingScenario::FaultBeforeMutation
        | ForgettingScenario::FaultAfterMutation
        | ForgettingScenario::FaultBeforeReceipt
        | ForgettingScenario::FaultAfterReceipt => {
            let (stage, label) = match scenario {
                ForgettingScenario::FaultBeforeMutation => (
                    AuthorityFaultStage::BeforeForgettingMutation,
                    "before_mutation",
                ),
                ForgettingScenario::FaultAfterMutation => (
                    AuthorityFaultStage::AfterForgettingMutation,
                    "after_mutation",
                ),
                ForgettingScenario::FaultBeforeReceipt => (
                    AuthorityFaultStage::BeforeForgettingReceipt,
                    "before_receipt",
                ),
                ForgettingScenario::FaultAfterReceipt => {
                    (AuthorityFaultStage::AfterForgettingReceipt, "after_receipt")
                }
                _ => unreachable!(),
            };
            let root = append_forgetting(
                &store,
                "principal:dr05",
                "fault-root",
                "fault rollback canary",
            )
            .await;
            store.authority().set_fault(Some(stage));
            let error = store
                .authority()
                .forget(
                    operator_permit("principal:dr05", AuthorityPermit::FORGET_CAPABILITY),
                    "forget-fault".into(),
                    forgetting_request(&root, "private", "fault rollback", 8),
                )
                .await
                .expect_err("fault");
            assert!(
                matches!(error, MemoryError::AuthorityFaultInjected { stage: actual } if actual == stage)
            );
            assert_eq!(
                store
                    .get_fact_raw_compat(&root)
                    .await
                    .expect("root")
                    .expect("root exists")
                    .content,
                "fault rollback canary"
            );
            assert!(store
                .authority()
                .get_forgetting_receipt_by_idempotency_key("forget-fault")
                .await
                .expect("receipt lookup")
                .is_none());
            vec![
                format!("stage={label}"),
                format!("error={}", error.kind()),
                "content=unchanged".into(),
                "receipt=absent".into(),
            ]
        }
        ForgettingScenario::RawTombstone => {
            let root =
                append_forgetting(&store, "principal:dr05", "raw-root", "raw surface canary").await;
            let receipt = forget_root(&store, "forget-raw", &root, "erase raw", 8).await;
            let raw = store
                .get_fact_raw_compat(&root)
                .await
                .expect("raw")
                .expect("raw row");
            assert_eq!(raw.content, "[FORGOTTEN]");
            let mut witness = normalized_closure(&receipt);
            witness.push(format!("raw={}", raw.content));
            witness
        }
        ForgettingScenario::GovernedRemoval => {
            let root = append_forgetting(
                &store,
                "principal:dr05",
                "governed-root",
                "governed surface canary",
            )
            .await;
            let receipt = forget_root(&store, "forget-governed", &root, "erase governed", 8).await;
            let result = store
                .authority()
                .get_fact_governed(&root, forgetting_access())
                .await
                .expect("governed");
            assert!(result.fact.is_none());
            let mut witness = normalized_closure(&receipt);
            witness.push(format!("governed_present={}", result.fact.is_some()));
            witness.push(format!("allowed={}", result.decision.allowed));
            witness
        }
        ForgettingScenario::SearchRemoval => {
            let root = append_forgetting(
                &store,
                "principal:dr05",
                "search-root",
                "search removal canary",
            )
            .await;
            let receipt = forget_root(&store, "forget-search", &root, "erase search", 8).await;
            let results = store
                .search("search removal canary", Some(8), Some(&["private"]), None)
                .await
                .expect("search");
            assert!(results
                .iter()
                .all(|result| !result.content.contains("search removal canary")));
            let mut witness = normalized_closure(&receipt);
            witness.push(format!("search_results={}", results.len()));
            witness
        }
        ForgettingScenario::CacheRemoval => {
            let root = append_forgetting(
                &store,
                "principal:dr05",
                "cache-root",
                "cache removal canary",
            )
            .await;
            let before = store
                .search("cache removal canary", Some(8), None, None)
                .await
                .expect("prime cache");
            assert!(!before.is_empty());
            let receipt = forget_root(&store, "forget-cache", &root, "erase cache", 8).await;
            let after = store
                .search("cache removal canary", Some(8), None, None)
                .await
                .expect("cached search");
            assert!(after
                .iter()
                .all(|result| !result.content.contains("cache removal canary")));
            let mut witness = normalized_closure(&receipt);
            witness.push(format!("before={}", before.len()));
            witness.push(format!("after={}", after.len()));
            witness
        }
        ForgettingScenario::ReplayRemoval => {
            let root = append_forgetting(
                &store,
                "principal:dr05",
                "replay-root",
                "replay removal canary",
            )
            .await;
            let mut context = SearchContext::default_now();
            context.receipt_mode = ReceiptMode::ReturnReceipt;
            let search_receipt = store
                .search_with_context(
                    "replay removal canary",
                    Some(8),
                    Some(&["private"]),
                    None,
                    context,
                )
                .await
                .expect("search")
                .receipt
                .expect("receipt");
            let receipt = forget_root(&store, "forget-replay", &root, "erase replay", 8).await;
            let replay = store
                .replay_search_receipt(
                    &search_receipt.receipt_id,
                    "replay removal canary",
                    Some(8),
                    Some(&["private"]),
                    None,
                )
                .await;
            assert!(matches!(
                replay,
                Err(MemoryError::ForgettingClosureIncomplete { .. })
            ));
            let mut witness = normalized_closure(&receipt);
            witness.push(format!(
                "replay_error={}",
                replay.expect_err("replay refused").kind()
            ));
            witness
        }
        ForgettingScenario::ExportRemoval => {
            let root = append_forgetting(
                &store,
                "principal:dr05",
                "export-root",
                "export removal canary",
            )
            .await;
            let receipt = forget_root(&store, "forget-export", &root, "erase export", 8).await;
            let export = store
                .authority()
                .export_fact_governed(&root, forgetting_access())
                .await
                .expect("export");
            assert!(export.fact.is_none());
            let mut witness = normalized_closure(&receipt);
            witness.push(format!("export_present={}", export.fact.is_some()));
            witness.push(format!("allowed={}", export.decision.allowed));
            witness
        }
        ForgettingScenario::CurrentViewRemoval
        | ForgettingScenario::SupersededViewRemoval
        | ForgettingScenario::HistoricalViewRemoval => {
            let (view, label) = match scenario {
                ForgettingScenario::CurrentViewRemoval => (StateView::Current, "current"),
                ForgettingScenario::SupersededViewRemoval => {
                    (StateView::IncludeSuperseded, "include_superseded")
                }
                ForgettingScenario::HistoricalViewRemoval => (
                    StateView::HistoricalAt("2999-01-01T00:00:00Z".into()),
                    "historical",
                ),
                _ => unreachable!(),
            };
            let root =
                append_forgetting(&store, "principal:dr05", "view-root", "view removal canary")
                    .await;
            let receipt = forget_root(&store, "forget-view", &root, "erase view", 8).await;
            let results = store
                .search_with_view(
                    "view removal canary",
                    Some(8),
                    Some(&["private"]),
                    None,
                    view,
                )
                .await
                .expect("view search");
            assert!(results
                .iter()
                .all(|result| !result.content.contains("view removal canary")));
            let mut witness = normalized_closure(&receipt);
            witness.push(format!("view={label}"));
            witness.push(format!("results={}", results.len()));
            witness
        }
        ForgettingScenario::ProjectionRemoval => {
            let root = append_forgetting(
                &store,
                "principal:dr05",
                "projection-root",
                "projection ancestor",
            )
            .await;
            store.raw_execute(
                "INSERT INTO claim_versions (claim_version_id, claim_id, projection_family, subject_entity_id, predicate, object_anchor, scope_namespace, source_envelope_id, source_authority, content) VALUES (?1, ?2, 'test', 'entity', 'contains', ?3, 'private', 'env', 'test', ?4)",
                vec!["claim-v1".into(), "claim-1".into(), "\"projection ancestor\"".into(), "derived projection canary".into()],
            ).await.expect("projection insert");
            store.raw_execute(
                "INSERT INTO derivation_edges (source_kind, source_id, target_kind, target_id, derivation_type) VALUES ('fact', ?1, 'claim_version', 'claim-v1', 'derived_from_fact')",
                vec![root.clone()],
            ).await.expect("derivation insert");
            assert_eq!(
                store
                    .query_claim_versions(ProjectionQuery::new(ScopeKey::namespace_only("private")))
                    .await
                    .expect("claims")
                    .len(),
                1
            );
            let receipt =
                forget_root(&store, "forget-projection", &root, "erase projection", 16).await;
            let claims = store
                .query_claim_versions(ProjectionQuery::new(ScopeKey::namespace_only("private")))
                .await
                .expect("claims");
            assert!(claims.is_empty());
            let mut witness = normalized_closure(&receipt);
            witness.push(format!("claims={}", claims.len()));
            witness
        }
        ForgettingScenario::EmbeddingRemoval => {
            let root = append_forgetting(
                &store,
                "principal:dr05",
                "embedding-root",
                "embedding removal canary",
            )
            .await;
            assert!(store
                .get_fact_embedding(&root)
                .await
                .expect("embedding")
                .is_some());
            let receipt =
                forget_root(&store, "forget-embedding", &root, "erase embedding", 8).await;
            let embedding = store
                .get_fact_embedding(&root)
                .await
                .expect("embedding after");
            assert!(embedding.is_none());
            let mut witness = normalized_closure(&receipt);
            witness.push(format!("embedding_present={}", embedding.is_some()));
            witness
        }
        ForgettingScenario::GraphRemoval => {
            let root = append_forgetting(
                &store,
                "principal:dr05",
                "graph-root",
                "graph removal canary",
            )
            .await;
            let child = append_forgetting(
                &store,
                "principal:dr05",
                "graph-child",
                "graph child canary",
            )
            .await;
            add_dependency(&store, &child, &root).await;
            assert!(!store
                .list_graph_edges_for_node(&format!("fact:{root}"))
                .await
                .expect("edges")
                .is_empty());
            let receipt = forget_root(&store, "forget-graph", &root, "erase graph", 8).await;
            let root_edges = store
                .list_graph_edges_for_node(&format!("fact:{root}"))
                .await
                .expect("root edges");
            let child_edges = store
                .list_graph_edges_for_node(&format!("fact:{child}"))
                .await
                .expect("child edges");
            assert!(root_edges.is_empty() && child_edges.is_empty());
            let mut witness = normalized_closure(&receipt);
            witness.push(format!("root_edges={}", root_edges.len()));
            witness.push(format!("child_edges={}", child_edges.len()));
            witness
        }
        ForgettingScenario::ReceiptNoPlaintext => {
            let canary = "plaintext-secret-dr05-9917";
            let reason = "plaintext-erasure-reason-dr05-114";
            let root = append_forgetting(&store, "principal:dr05", "plaintext-root", canary).await;
            let receipt = forget_root(&store, "forget-plaintext", &root, reason, 8).await;
            let json = serde_json::to_string(&receipt).expect("receipt JSON");
            assert!(!json.contains(canary));
            assert!(!json.contains(reason));
            let mut witness = normalized_closure(&receipt);
            witness.push(format!("contains_canary={}", json.contains(canary)));
            witness.push(format!("contains_reason={}", json.contains(reason)));
            witness
        }
        ForgettingScenario::OwnerRebuildNonResurrection => {
            let root = append_forgetting(
                &store,
                "principal:dr05",
                "rebuild-root",
                "rebuild resurrection canary",
            )
            .await;
            let receipt =
                forget_root(&store, "forget-rebuild", &root, "erase before rebuild", 8).await;
            let rebuilt = store.reembed_all().await.expect("owner-supported rebuild");
            let results = store
                .search(
                    "rebuild resurrection canary",
                    Some(8),
                    Some(&["private"]),
                    None,
                )
                .await
                .expect("search after rebuild");
            assert!(results
                .iter()
                .all(|result| !result.content.contains("rebuild resurrection canary")));
            assert_eq!(
                store
                    .get_fact_raw_compat(&root)
                    .await
                    .expect("raw")
                    .expect("row")
                    .content,
                "[FORGOTTEN]"
            );
            let mut witness = normalized_closure(&receipt);
            witness.push(format!("reembedded={rebuilt}"));
            witness.push(format!("resurrected_results={}", results.len()));
            witness
        }
        ForgettingScenario::UnsupportedDerivedKindRefused => {
            let root = append_forgetting(
                &store,
                "principal:dr05",
                "unsupported-root",
                "unsupported derived canary",
            )
            .await;
            store.raw_execute(
                "INSERT INTO derivation_edges (source_kind, source_id, target_kind, target_id, derivation_type) VALUES ('fact', ?1, 'unsupported_artifact', 'artifact-1', 'derived_from_fact')",
                vec![root.clone()],
            ).await.expect("unsupported derivation");
            let error = store
                .authority()
                .forget(
                    operator_permit("principal:dr05", AuthorityPermit::FORGET_CAPABILITY),
                    "forget-unsupported".into(),
                    forgetting_request(&root, "private", "unsupported adapter", 8),
                )
                .await
                .expect_err("unsupported kind must refuse");
            assert!(matches!(
                error,
                MemoryError::ForgettingClosureIncomplete { .. }
            ));
            assert_eq!(
                store
                    .get_fact_raw_compat(&root)
                    .await
                    .expect("root")
                    .expect("root exists")
                    .content,
                "unsupported derived canary"
            );
            vec![
                format!("error={}", error.kind()),
                "content=unchanged".into(),
                "receipt=absent".into(),
            ]
        }
    }
}

fn contradiction_outcome(scenario: ContradictionScenario) -> (ExpectedOutcome, WitnessCategory) {
    match scenario {
        ContradictionScenario::SupportSetRoundTrip
        | ContradictionScenario::ContradictionWitnessRoundTrip
        | ContradictionScenario::RetractionRecordRoundTrip
        | ContradictionScenario::ClaimStateRoundTrip
        | ContradictionScenario::ProjectionIdempotentRepeat => {
            (ExpectedOutcome::Applied, WitnessCategory::ProjectionReceipt)
        }
        ContradictionScenario::BothRemainsNonActionable => {
            (ExpectedOutcome::Denied, WitnessCategory::ProjectionReceipt)
        }
        ContradictionScenario::UnknownSupportSchemaRefused
        | ContradictionScenario::MissingSupportTokensRefused
        | ContradictionScenario::MalformedV3MissingFieldRefused
        | ContradictionScenario::MissingClaimStateTxFromRefused
        | ContradictionScenario::PreferredOpenConflict
        | ContradictionScenario::InvalidTemporalOrder
        | ContradictionScenario::OverlappingPreferredIntervals
        | ContradictionScenario::AuthorityChangedPayloadConflict
        | ContradictionScenario::ProjectionChangedPayloadConflict => {
            (ExpectedOutcome::Refused, WitnessCategory::TypedError)
        }
        ContradictionScenario::AppendSupersedeCurrentHistorical
        | ContradictionScenario::AppendRedactCurrentHistorical
        | ContradictionScenario::SourceBackedRetraction => {
            (ExpectedOutcome::Applied, WitnessCategory::AuthorityState)
        }
        ContradictionScenario::SourceBackedQuarantine => {
            (ExpectedOutcome::Refused, WitnessCategory::AuthorityState)
        }
        ContradictionScenario::TransitionFaultRollback => {
            (ExpectedOutcome::RolledBack, WitnessCategory::TypedError)
        }
    }
}

fn origin_outcome(scenario: OriginScenario) -> (ExpectedOutcome, WitnessCategory) {
    match scenario {
        OriginScenario::NoOriginAppendDenied => {
            (ExpectedOutcome::Denied, WitnessCategory::TypedError)
        }
        OriginScenario::RecallBoundaryAllowed => {
            (ExpectedOutcome::Allowed, WitnessCategory::AccessDecision)
        }
        OriginScenario::OriginImmutable => {
            (ExpectedOutcome::Applied, WitnessCategory::AuthorityState)
        }
        _ => (ExpectedOutcome::Denied, WitnessCategory::AccessDecision),
    }
}

fn forgetting_outcome(scenario: ForgettingScenario) -> (ExpectedOutcome, WitnessCategory) {
    match scenario {
        ForgettingScenario::ZeroBudget
        | ForgettingScenario::InsufficientBudget
        | ForgettingScenario::WrongScope
        | ForgettingScenario::WrongPrincipal
        | ForgettingScenario::ConflictingRetry
        | ForgettingScenario::UnsupportedDerivedKindRefused => {
            (ExpectedOutcome::Refused, WitnessCategory::TypedError)
        }
        ForgettingScenario::FaultBeforeMutation
        | ForgettingScenario::FaultAfterMutation
        | ForgettingScenario::FaultBeforeReceipt
        | ForgettingScenario::FaultAfterReceipt => {
            (ExpectedOutcome::RolledBack, WitnessCategory::TypedError)
        }
        _ => (ExpectedOutcome::Applied, WitnessCategory::ClosureState),
    }
}

async fn execute_case(case: Case) -> CaseWitness {
    let (outcome, category, observed) = match case.scenario {
        Scenario::Temporal(scenario) => (
            ExpectedOutcome::Applied,
            WitnessCategory::AsOfState,
            execute_temporal(scenario).await,
        ),
        Scenario::Contradiction(scenario) => {
            let (outcome, category) = contradiction_outcome(scenario);
            (outcome, category, execute_contradiction(scenario).await)
        }
        Scenario::Origin(scenario) => {
            let (outcome, category) = origin_outcome(scenario);
            (outcome, category, execute_origin(scenario).await)
        }
        Scenario::Forgetting(scenario) => {
            let (outcome, category) = forgetting_outcome(scenario);
            (outcome, category, execute_forgetting(scenario).await)
        }
    };
    assert_eq!(
        outcome, case.expected_outcome,
        "{} outcome at {}",
        case.id, case.owner_seam
    );
    assert_eq!(
        category, case.expected_witness,
        "{} witness category at {}",
        case.id, case.owner_seam
    );
    assert!(
        !observed.is_empty(),
        "{} must emit source-derived stable fields",
        case.id
    );
    CaseWitness {
        id: case.id,
        family: case.family,
        owner_seam: case.owner_seam,
        outcome,
        category,
        observed,
    }
}

async fn execute_manifest() -> Vec<CaseWitness> {
    let mut witnesses = Vec::with_capacity(CASES.len());
    for case in CASES.iter().copied() {
        witnesses.push(execute_case(case).await);
    }
    let executed_ids = witnesses
        .iter()
        .map(|witness| witness.id)
        .collect::<Vec<_>>();
    let manifest_ids = CASES.iter().map(|case| case.id).collect::<Vec<_>>();
    assert_eq!(executed_ids, manifest_ids, "exact ordered execution");
    assert_eq!(executed_ids, EXPECTED_IDS, "exact declared inventory");
    assert_eq!(
        executed_ids.iter().copied().collect::<BTreeSet<_>>().len(),
        100,
        "unique executions"
    );
    assert_eq!(witnesses.len(), 100, "one witness per case");
    witnesses
}

#[test]
fn dr05_manifest_is_exactly_100_typed_cases() {
    validate_manifest(CASES).expect("DR-05 manifest shape");
}

#[tokio::test]
async fn dr05_all_100_cases_are_source_backed_and_repeatable() {
    validate_manifest(CASES).expect("DR-05 manifest shape");
    let first = execute_manifest().await;
    let second = execute_manifest().await;
    assert_eq!(first, second, "normalized fresh-store witness sequence");
}

/// Companion temporal regression: two sibling families sharing a document
/// remain distinct. It intentionally does not contribute to T01-T30.
#[tokio::test]
async fn dr05_temporal_sibling_families_do_not_merge() {
    let (store, _temp) = test_store();
    let document_id = store
        .ingest_document(
            "siblings",
            "independent families",
            "dr05-temporal",
            None,
            None,
        )
        .await
        .expect("document");
    store
        .append_episode_version(
            "a0",
            None,
            &document_id,
            &episode_meta("a0", T0),
            Some(at(T0)),
        )
        .await
        .expect("a0");
    store
        .append_episode_version(
            "b0",
            None,
            &document_id,
            &episode_meta("b0", T0),
            Some(at(T0)),
        )
        .await
        .expect("b0");
    store
        .append_episode_version(
            "a1",
            Some("a0"),
            &document_id,
            &episode_meta("a1", T1),
            Some(at(T1)),
        )
        .await
        .expect("a1");
    store
        .append_episode_version(
            "b1",
            Some("b0"),
            &document_id,
            &episode_meta("b1", T1),
            Some(at(T2)),
        )
        .await
        .expect("b1");
    let (rows, receipt) = store.episode_as_of(at(T1), at(T2)).await.expect("as of");
    let winners = rows
        .iter()
        .map(|row| (row.episode_id.as_str(), row.meta.effect_type.as_str()))
        .collect::<BTreeMap<_, _>>();
    assert_eq!(winners, BTreeMap::from([("a1", "a1"), ("b1", "b1")]));
    assert_eq!(receipt.episode_count, 2);
    assert_eq!(receipt.excluded_superseded, 2);
}
