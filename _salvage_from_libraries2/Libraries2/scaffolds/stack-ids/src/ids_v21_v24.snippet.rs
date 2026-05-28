use serde::{Deserialize, Serialize};

macro_rules! string_id {
    ($name:ident) => {
        #[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
        pub struct $name(pub String);

        impl $name {
            pub fn new(value: impl Into<String>) -> Self { Self(value.into()) }
            pub fn as_str(&self) -> &str { &self.0 }
        }
    };
}
string_id!(EffectIntentId);
string_id!(EffectPreflightReportId);
string_id!(EffectWindowId);
string_id!(EffectCommitDecisionId);
string_id!(EffectExecutionReceiptId);
string_id!(EffectObservationBundleId);
string_id!(CompensationPlanId);
string_id!(CompensationExecutionReceiptId);
string_id!(ExternalEffectLedgerEntryId);
string_id!(CapabilityClassId);
string_id!(AuthorityLeaseId);
string_id!(DelegationBundleId);
string_id!(AuthorityChainId);
string_id!(SeparationOfDutiesPolicyId);
string_id!(DualControlApprovalId);
string_id!(BreakGlassGrantId);
string_id!(DelegationRevocationId);
string_id!(ActingOnBehalfReceiptId);
string_id!(ConflictDisclosureId);
string_id!(DeploymentProfileId);
string_id!(OperatingEnvelopeId);
string_id!(AssuranceCaseId);
string_id!(HazardRegisterId);
string_id!(ControlMappingId);
string_id!(ResidualRiskAcceptanceId);
string_id!(ReleaseReadinessDecisionId);
string_id!(FieldMonitoringPlanId);
string_id!(CertificationBundleId);
string_id!(RecertificationTriggerId);
string_id!(ServiceLevelProfileId);
string_id!(ErrorBudgetLedgerId);
string_id!(IncidentCaseId);
string_id!(ContainmentDecisionId);
string_id!(ForensicFreezeId);
string_id!(RecoveryPlanId);
string_id!(RecoveryReplaySliceId);
string_id!(ContinuityExceptionId);
string_id!(PostmortemBundleId);
string_id!(ResilienceExerciseId);
