// Draft v14/v15 schema registrations.
// Adjust module paths to match the final owner crates before landing.

SchemaSpec {
    name: "intervention-bundle-v1.schema.json",
    writer: write_schema::<semantic_memory_forge::InterventionBundleV1>,
},
SchemaSpec {
    name: "outcome-schema-v1.schema.json",
    writer: write_schema::<semantic_memory_forge::OutcomeSchemaV1>,
},
SchemaSpec {
    name: "experiment-case-v1.schema.json",
    writer: write_schema::<verification_control::ExperimentCaseV1>,
},
SchemaSpec {
    name: "cohort-contract-v1.schema.json",
    writer: write_schema::<semantic_memory_forge::CohortContractV1>,
},
SchemaSpec {
    name: "comparability-matrix-v1.schema.json",
    writer: write_schema::<verification_control::ComparabilityMatrixV1>,
},
SchemaSpec {
    name: "counterfactual-slice-v1.schema.json",
    writer: write_schema::<semantic_memory_forge::CounterfactualSliceV1>,
},
SchemaSpec {
    name: "decision-trace-v1.schema.json",
    writer: write_schema::<verification_control::DecisionTraceV1>,
},
SchemaSpec {
    name: "refuter-suite-v1.schema.json",
    writer: write_schema::<verification_policy::RefuterSuiteV1>,
},
SchemaSpec {
    name: "refuter-result-v1.schema.json",
    writer: write_schema::<verification_control::RefuterResultV1>,
},
SchemaSpec {
    name: "rollout-decision-v1.schema.json",
    writer: write_schema::<verification_adjudication::RolloutDecisionV1>,
},
SchemaSpec {
    name: "rollback-decision-v1.schema.json",
    writer: write_schema::<verification_adjudication::RollbackDecisionV1>,
},
SchemaSpec {
    name: "experiment-budget-v1.schema.json",
    writer: write_schema::<verification_policy::ExperimentBudgetV1>,
},
SchemaSpec {
    name: "attestation-envelope-v1.schema.json",
    writer: write_schema::<attestation_exchange::AttestationEnvelopeV1>,
},
SchemaSpec {
    name: "trust-root-set-v1.schema.json",
    writer: write_schema::<attestation_exchange::TrustRootSetV1>,
},
SchemaSpec {
    name: "artifact-admission-policy-v1.schema.json",
    writer: write_schema::<verification_policy::ArtifactAdmissionPolicyV1>,
},
SchemaSpec {
    name: "transparency-receipt-v1.schema.json",
    writer: write_schema::<attestation_exchange::TransparencyReceiptV1>,
},
SchemaSpec {
    name: "attestation-revocation-v1.schema.json",
    writer: write_schema::<remote_oracle_admission::AttestationRevocationV1>,
},
SchemaSpec {
    name: "attestation-supersession-v1.schema.json",
    writer: write_schema::<remote_oracle_admission::AttestationSupersessionV1>,
},
SchemaSpec {
    name: "remote-oracle-lease-v1.schema.json",
    writer: write_schema::<remote_oracle_admission::RemoteOracleLeaseV1>,
},
SchemaSpec {
    name: "remote-slice-request-v1.schema.json",
    writer: write_schema::<remote_oracle_admission::RemoteSliceRequestV1>,
},
SchemaSpec {
    name: "remote-slice-result-v1.schema.json",
    writer: write_schema::<remote_oracle_admission::RemoteSliceResultV1>,
},
SchemaSpec {
    name: "cross-runtime-replay-ticket-v1.schema.json",
    writer: write_schema::<remote_oracle_admission::CrossRuntimeReplayTicketV1>,
},
SchemaSpec {
    name: "dispute-bundle-v1.schema.json",
    writer: write_schema::<verification_control::DisputeBundleV1>,
},
SchemaSpec {
    name: "disclosure-policy-v1.schema.json",
    writer: write_schema::<verification_policy::DisclosurePolicyV1>,
},
SchemaSpec {
    name: "disclosure-budget-v1.schema.json",
    writer: write_schema::<verification_policy::DisclosureBudgetV1>,
},
