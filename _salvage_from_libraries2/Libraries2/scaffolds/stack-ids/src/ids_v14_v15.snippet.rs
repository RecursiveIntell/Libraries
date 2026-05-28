use serde::{Deserialize, Serialize};

macro_rules! string_id {
    ($name:ident) => {
        #[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
        pub struct $name(pub String);

        impl $name {
            pub fn new(value: impl Into<String>) -> Self {
                Self(value.into())
            }

            pub fn as_str(&self) -> &str {
                &self.0
            }
        }
    };
}
string_id!(InterventionId);
string_id!(OutcomeSchemaId);
string_id!(ExperimentCaseId);
string_id!(CohortContractId);
string_id!(ComparabilityMatrixId);
string_id!(CounterfactualSliceId);
string_id!(DecisionTraceId);
string_id!(RefuterSuiteId);
string_id!(RefuterResultId);
string_id!(RolloutDecisionId);
string_id!(RollbackDecisionId);
string_id!(ExperimentBudgetId);
string_id!(AttestationEnvelopeId);
string_id!(TrustRootSetId);
string_id!(ArtifactAdmissionPolicyId);
string_id!(TransparencyReceiptId);
string_id!(AttestationRevocationId);
string_id!(AttestationSupersessionId);
string_id!(RemoteOracleLeaseId);
string_id!(RemoteSliceRequestId);
string_id!(RemoteSliceResultId);
string_id!(CrossRuntimeReplayTicketId);
string_id!(DisputeBundleId);
string_id!(DisclosurePolicyId);
string_id!(DisclosureBudgetId);
