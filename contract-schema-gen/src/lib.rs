use schemars::schema_for;
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};

type SchemaWriter = fn(&Path, &str) -> Result<(), Box<dyn std::error::Error>>;

#[derive(Clone, Copy)]
struct SchemaSpec {
    name: &'static str,
    writer: SchemaWriter,
}

fn write_schema<T: schemars::JsonSchema>(
    dir: &Path,
    name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let schema = schema_for!(T);
    let value: Value = serde_json::to_value(&schema)?;
    let path = dir.join(name);
    fs::write(path, serde_json::to_string_pretty(&value)? + "\n")?;
    Ok(())
}

const SCHEMA_SPECS: &[SchemaSpec] = &[
    SchemaSpec {
        name: "export-envelope-v3.schema.json",
        writer: write_schema::<semantic_memory_forge::ExportEnvelopeV3>,
    },
    SchemaSpec {
        name: "projection-import-batch-v3.schema.json",
        writer: write_schema::<forge_memory_bridge::ProjectionImportBatchV3>,
    },
    SchemaSpec {
        name: "compile-output.schema.json",
        writer: write_schema::<constraint_compiler::CompileOutput>,
    },
    SchemaSpec {
        name: "execution-report.schema.json",
        writer: write_schema::<kernel_execution::ExecutionReport>,
    },
    SchemaSpec {
        name: "oracle-assessment.schema.json",
        writer: write_schema::<kernel_oracles::OracleAssessment>,
    },
    SchemaSpec {
        name: "forge-tool-receipt-v2.schema.json",
        writer: write_schema::<semantic_memory_forge::ForgeToolReceiptV2>,
    },
    SchemaSpec {
        name: "episode-bundle-v1.schema.json",
        writer: write_schema::<semantic_memory_forge::EpisodeBundleV1>,
    },
    SchemaSpec {
        name: "evidence-bundle-v1.schema.json",
        writer: write_schema::<semantic_memory_forge::EvidenceBundle>,
    },
    SchemaSpec {
        name: "execution-context-v1.schema.json",
        writer: write_schema::<semantic_memory_forge::ExecutionContextV1>,
    },
    SchemaSpec {
        name: "semantics-profile-v1.schema.json",
        writer: write_schema::<semantic_memory_forge::SemanticsProfileV1>,
    },
    SchemaSpec {
        name: "claim-state-v1.schema.json",
        writer: write_schema::<semantic_memory_forge::ClaimStateV1>,
    },
    SchemaSpec {
        name: "support-set-v1.schema.json",
        writer: write_schema::<semantic_memory_forge::SupportSetV1>,
    },
    SchemaSpec {
        name: "contradiction-witness-v1.schema.json",
        writer: write_schema::<semantic_memory_forge::ContradictionWitnessV1>,
    },
    SchemaSpec {
        name: "retraction-record-v1.schema.json",
        writer: write_schema::<semantic_memory_forge::RetractionRecordV1>,
    },
    SchemaSpec {
        name: "claim-state-v13.schema.json",
        writer: write_schema::<semantic_memory_forge::ClaimStateV13>,
    },
    SchemaSpec {
        name: "witness-artifact-v1.schema.json",
        writer: write_schema::<semantic_memory_forge::WitnessArtifactV1>,
    },
    SchemaSpec {
        name: "certificate-artifact-v1.schema.json",
        writer: write_schema::<semantic_memory_forge::CertificateArtifactV1>,
    },
    SchemaSpec {
        name: "refutation-artifact-v1.schema.json",
        writer: write_schema::<semantic_memory_forge::RefutationArtifactV1>,
    },
    SchemaSpec {
        name: "semantic-diff-v1.schema.json",
        writer: write_schema::<semantic_memory_forge::SemanticDiffV1>,
    },
    SchemaSpec {
        name: "oracle-slice-contract-v1.schema.json",
        writer: write_schema::<semantic_memory_forge::OracleSliceContractV1>,
    },
    SchemaSpec {
        name: "causal-attribution-bundle-v1.schema.json",
        writer: write_schema::<semantic_memory_forge::CausalAttributionBundleV1>,
    },
    SchemaSpec {
        name: "degradation-record-v1.schema.json",
        writer: write_schema::<semantic_memory_forge::DegradationRecordV1>,
    },
    SchemaSpec {
        name: "exactness-budget-v1.schema.json",
        writer: write_schema::<semantic_memory_forge::ExactnessBudgetV1>,
    },
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
        name: "effect-intent-v1.schema.json",
        writer: write_schema::<effect_runtime::EffectIntentV1>,
    },
    SchemaSpec {
        name: "effect-preflight-report-v1.schema.json",
        writer: write_schema::<effect_runtime::EffectPreflightReportV1>,
    },
    SchemaSpec {
        name: "effect-window-v1.schema.json",
        writer: write_schema::<effect_runtime::EffectWindowV1>,
    },
    SchemaSpec {
        name: "effect-commit-decision-v1.schema.json",
        writer: write_schema::<effect_runtime::EffectCommitDecisionV1>,
    },
    SchemaSpec {
        name: "effect-execution-receipt-v1.schema.json",
        writer: write_schema::<effect_runtime::EffectExecutionReceiptV1>,
    },
    SchemaSpec {
        name: "effect-observation-bundle-v1.schema.json",
        writer: write_schema::<effect_runtime::EffectObservationBundleV1>,
    },
    SchemaSpec {
        name: "compensation-plan-v1.schema.json",
        writer: write_schema::<effect_runtime::CompensationPlanV1>,
    },
    SchemaSpec {
        name: "compensation-execution-receipt-v1.schema.json",
        writer: write_schema::<effect_runtime::CompensationExecutionReceiptV1>,
    },
    SchemaSpec {
        name: "external-effect-ledger-entry-v1.schema.json",
        writer: write_schema::<effect_runtime::ExternalEffectLedgerEntryV1>,
    },
    SchemaSpec {
        name: "capability-class-v1.schema.json",
        writer: write_schema::<authority_delegation::CapabilityClassV1>,
    },
    SchemaSpec {
        name: "authority-lease-v1.schema.json",
        writer: write_schema::<authority_delegation::AuthorityLeaseV1>,
    },
    SchemaSpec {
        name: "delegation-bundle-v1.schema.json",
        writer: write_schema::<authority_delegation::DelegationBundleV1>,
    },
    SchemaSpec {
        name: "authority-chain-v1.schema.json",
        writer: write_schema::<authority_delegation::AuthorityChainV1>,
    },
    SchemaSpec {
        name: "separation-of-duties-policy-v1.schema.json",
        writer: write_schema::<authority_delegation::SeparationOfDutiesPolicyV1>,
    },
    SchemaSpec {
        name: "dual-control-approval-v1.schema.json",
        writer: write_schema::<authority_delegation::DualControlApprovalV1>,
    },
    SchemaSpec {
        name: "break-glass-grant-v1.schema.json",
        writer: write_schema::<authority_delegation::BreakGlassGrantV1>,
    },
    SchemaSpec {
        name: "delegation-revocation-v1.schema.json",
        writer: write_schema::<authority_delegation::DelegationRevocationV1>,
    },
    SchemaSpec {
        name: "acting-on-behalf-receipt-v1.schema.json",
        writer: write_schema::<authority_delegation::ActingOnBehalfReceiptV1>,
    },
    SchemaSpec {
        name: "conflict-disclosure-v1.schema.json",
        writer: write_schema::<authority_delegation::ConflictDisclosureV1>,
    },
    SchemaSpec {
        name: "deployment-profile-v1.schema.json",
        writer: write_schema::<assurance_runtime::DeploymentProfileV1>,
    },
    SchemaSpec {
        name: "operating-envelope-v1.schema.json",
        writer: write_schema::<assurance_runtime::OperatingEnvelopeV1>,
    },
    SchemaSpec {
        name: "assurance-case-v1.schema.json",
        writer: write_schema::<assurance_runtime::AssuranceCaseV1>,
    },
    SchemaSpec {
        name: "hazard-register-v1.schema.json",
        writer: write_schema::<assurance_runtime::HazardRegisterV1>,
    },
    SchemaSpec {
        name: "control-mapping-v1.schema.json",
        writer: write_schema::<assurance_runtime::ControlMappingV1>,
    },
    SchemaSpec {
        name: "residual-risk-acceptance-v1.schema.json",
        writer: write_schema::<assurance_runtime::ResidualRiskAcceptanceV1>,
    },
    SchemaSpec {
        name: "release-readiness-decision-v1.schema.json",
        writer: write_schema::<assurance_runtime::ReleaseReadinessDecisionV1>,
    },
    SchemaSpec {
        name: "field-monitoring-plan-v1.schema.json",
        writer: write_schema::<assurance_runtime::FieldMonitoringPlanV1>,
    },
    SchemaSpec {
        name: "certification-bundle-v1.schema.json",
        writer: write_schema::<assurance_runtime::CertificationBundleV1>,
    },
    SchemaSpec {
        name: "recertification-trigger-v1.schema.json",
        writer: write_schema::<assurance_runtime::RecertificationTriggerV1>,
    },
    SchemaSpec {
        name: "service-level-profile-v1.schema.json",
        writer: write_schema::<continuity_runtime::ServiceLevelProfileV1>,
    },
    SchemaSpec {
        name: "error-budget-ledger-v1.schema.json",
        writer: write_schema::<continuity_runtime::ErrorBudgetLedgerV1>,
    },
    SchemaSpec {
        name: "incident-case-v1.schema.json",
        writer: write_schema::<continuity_runtime::IncidentCaseV1>,
    },
    SchemaSpec {
        name: "containment-decision-v1.schema.json",
        writer: write_schema::<continuity_runtime::ContainmentDecisionV1>,
    },
    SchemaSpec {
        name: "forensic-freeze-v1.schema.json",
        writer: write_schema::<continuity_runtime::ForensicFreezeV1>,
    },
    SchemaSpec {
        name: "recovery-plan-v1.schema.json",
        writer: write_schema::<continuity_runtime::RecoveryPlanV1>,
    },
    SchemaSpec {
        name: "recovery-replay-slice-v1.schema.json",
        writer: write_schema::<continuity_runtime::RecoveryReplaySliceV1>,
    },
    SchemaSpec {
        name: "continuity-exception-v1.schema.json",
        writer: write_schema::<continuity_runtime::ContinuityExceptionV1>,
    },
    SchemaSpec {
        name: "postmortem-bundle-v1.schema.json",
        writer: write_schema::<continuity_runtime::PostmortemBundleV1>,
    },
    SchemaSpec {
        name: "resilience-exercise-v1.schema.json",
        writer: write_schema::<continuity_runtime::ResilienceExerciseV1>,
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
    SchemaSpec {
        name: "pilot-loop-receipt-v1.schema.json",
        writer: write_schema::<forge_pilot::LoopReceipt>,
    },
    SchemaSpec {
        name: "verification-case-v1.schema.json",
        writer: write_schema::<verification_control::VerificationCase>,
    },
    SchemaSpec {
        name: "check-plan-v1.schema.json",
        writer: write_schema::<verification_control::CheckPlan>,
    },
    SchemaSpec {
        name: "verification-attempt-v1.schema.json",
        writer: write_schema::<verification_control::VerificationAttempt>,
    },
    SchemaSpec {
        name: "effect-review-case-v1.schema.json",
        writer: write_schema::<verification_control::EffectReviewCaseV1>,
    },
    SchemaSpec {
        name: "effect-block-receipt-v1.schema.json",
        writer: write_schema::<verification_control::EffectBlockReceiptV1>,
    },
    SchemaSpec {
        name: "delegation-review-case-v1.schema.json",
        writer: write_schema::<verification_control::DelegationReviewCaseV1>,
    },
    SchemaSpec {
        name: "release-gate-case-v1.schema.json",
        writer: write_schema::<verification_control::ReleaseGateCaseV1>,
    },
    SchemaSpec {
        name: "continuity-review-case-v1.schema.json",
        writer: write_schema::<verification_control::ContinuityReviewCaseV1>,
    },
    SchemaSpec {
        name: "control-receipt-v1.schema.json",
        writer: write_schema::<verification_control::ControlReceipt>,
    },
    SchemaSpec {
        name: "ledger-entry-v1.schema.json",
        writer: write_schema::<verification_control::LedgerEntry>,
    },
    SchemaSpec {
        name: "scheduler-decision-v1.schema.json",
        writer: write_schema::<verification_control::SchedulerDecision>,
    },
    SchemaSpec {
        name: "boundary-repair-record-v1.schema.json",
        writer: write_schema::<verification_control::BoundaryRepairRecord>,
    },
    SchemaSpec {
        name: "region-artifact-transport-v1.schema.json",
        writer: write_schema::<verification_control::RegionArtifactTransport>,
    },
    SchemaSpec {
        name: "syndrome-route-record-v1.schema.json",
        writer: write_schema::<verification_control::SyndromeRouteRecord>,
    },
    SchemaSpec {
        name: "policy-snapshot-v1.schema.json",
        writer: write_schema::<verification_policy::PolicySnapshot>,
    },
    SchemaSpec {
        name: "approval-record-v1.schema.json",
        writer: write_schema::<verification_policy::ApprovalRecord>,
    },
    SchemaSpec {
        name: "policy-decision-v1.schema.json",
        writer: write_schema::<verification_policy::PolicyDecision>,
    },
    SchemaSpec {
        name: "calibration-snapshot-v1.schema.json",
        writer: write_schema::<verification_calibration::CalibrationSnapshot>,
    },
    SchemaSpec {
        name: "nuisance-state-artifact-v1.schema.json",
        writer: write_schema::<verification_calibration::NuisanceStateArtifact>,
    },
    SchemaSpec {
        name: "promotion-decision-v1.schema.json",
        writer: write_schema::<verification_adjudication::PromotionDecision>,
    },
    SchemaSpec {
        name: "verification-disposition-v1.schema.json",
        writer: write_schema::<verification_adjudication::VerificationDisposition>,
    },
    SchemaSpec {
        name: "refutation-decision-v1.schema.json",
        writer: write_schema::<verification_adjudication::RefutationDecision>,
    },
    SchemaSpec {
        name: "rollback-plan-v1.schema.json",
        writer: write_schema::<verification_adjudication::RollbackPlan>,
    },
    SchemaSpec {
        name: "effect-adjudication-receipt-v1.schema.json",
        writer: write_schema::<verification_adjudication::EffectAdjudicationReceiptV1>,
    },
    SchemaSpec {
        name: "release-rollback-decision-v1.schema.json",
        writer: write_schema::<verification_adjudication::ReleaseRollbackDecisionV1>,
    },
    SchemaSpec {
        name: "adjudication-result-v1.schema.json",
        writer: write_schema::<verification_adjudication::AdjudicationResult>,
    },
    SchemaSpec {
        name: "loop-iteration-report-v1.schema.json",
        writer: write_schema::<forge_pilot::LoopIterationReport>,
    },
    SchemaSpec {
        name: "loop-report-v1.schema.json",
        writer: write_schema::<forge_pilot::LoopReport>,
    },
    SchemaSpec {
        name: "verification-plan-artifact-v1.schema.json",
        writer: write_schema::<forge_pilot::VerificationPlanArtifact>,
    },
    SchemaSpec {
        name: "repair-record-v1.schema.json",
        writer: write_schema::<forge_pilot::RepairRecordV1>,
    },
    SchemaSpec {
        name: "runtime-query-provenance-v1.schema.json",
        writer: write_schema::<knowledge_runtime::RuntimeQueryProvenanceV1>,
    },
    SchemaSpec {
        name: "inference-advisory-v1.schema.json",
        writer: write_schema::<knowledge_runtime::InferenceAdvisory>,
    },
    SchemaSpec {
        name: "inference-explanation-v1.schema.json",
        writer: write_schema::<knowledge_runtime::InferenceExplanation>,
    },
    SchemaSpec {
        name: "risk-gate-decision-v1.schema.json",
        writer: write_schema::<knowledge_runtime::RiskGateDecision>,
    },
    SchemaSpec {
        name: "import-failure-record-v1.schema.json",
        writer: write_schema::<semantic_memory::ProjectionImportFailureReceiptEntry>,
    },
    SchemaSpec {
        name: "treaty-bundle-v1.schema.json",
        writer: write_schema::<federated_settlement::TreatyBundleV1>,
    },
    SchemaSpec {
        name: "runtime-identity-set-v1.schema.json",
        writer: write_schema::<federated_settlement::RuntimeIdentitySetV1>,
    },
    SchemaSpec {
        name: "cross-runtime-equivalence-bundle-v1.schema.json",
        writer: write_schema::<federated_settlement::CrossRuntimeEquivalenceBundleV1>,
    },
    SchemaSpec {
        name: "shared-disposition-v1.schema.json",
        writer: write_schema::<federated_settlement::SharedDispositionV1>,
    },
    SchemaSpec {
        name: "local-dissent-record-v1.schema.json",
        writer: write_schema::<federated_settlement::LocalDissentRecordV1>,
    },
    SchemaSpec {
        name: "shared-view-downgrade-v1.schema.json",
        writer: write_schema::<federated_settlement::SharedViewDowngradeV1>,
    },
    SchemaSpec {
        name: "settlement-case-v1.schema.json",
        writer: write_schema::<federated_settlement::SettlementCaseV1>,
    },
    SchemaSpec {
        name: "settlement-receipt-v1.schema.json",
        writer: write_schema::<federated_settlement::SettlementReceiptV1>,
    },
    SchemaSpec {
        name: "shared-replay-slice-v1.schema.json",
        writer: write_schema::<federated_settlement::SharedReplaySliceV1>,
    },
    SchemaSpec {
        name: "shared-divergence-report-v1.schema.json",
        writer: write_schema::<federated_settlement::SharedDivergenceReportV1>,
    },
    SchemaSpec {
        name: "treaty-suspension-v1.schema.json",
        writer: write_schema::<federated_settlement::TreatySuspensionV1>,
    },
    SchemaSpec {
        name: "mechanism-bundle-v1.schema.json",
        writer: write_schema::<mechanism_runtime::MechanismBundleV1>,
    },
    SchemaSpec {
        name: "theory-version-v1.schema.json",
        writer: write_schema::<mechanism_runtime::TheoryVersionV1>,
    },
    SchemaSpec {
        name: "theory-library-v1.schema.json",
        writer: write_schema::<mechanism_runtime::TheoryLibraryV1>,
    },
    SchemaSpec {
        name: "hypothesis-library-v1.schema.json",
        writer: write_schema::<mechanism_runtime::HypothesisLibraryV1>,
    },
    SchemaSpec {
        name: "simulation-contract-v1.schema.json",
        writer: write_schema::<mechanism_runtime::SimulationContractV1>,
    },
    SchemaSpec {
        name: "fit-run-v1.schema.json",
        writer: write_schema::<mechanism_runtime::FitRunV1>,
    },
    SchemaSpec {
        name: "theory-refuter-suite-v1.schema.json",
        writer: write_schema::<mechanism_runtime::TheoryRefuterSuiteV1>,
    },
    SchemaSpec {
        name: "rollout-stability-report-v1.schema.json",
        writer: write_schema::<mechanism_runtime::RolloutStabilityReportV1>,
    },
    SchemaSpec {
        name: "discovery-program-v1.schema.json",
        writer: write_schema::<discovery_portfolio::DiscoveryProgramV1>,
    },
    SchemaSpec {
        name: "program-hypothesis-set-v1.schema.json",
        writer: write_schema::<discovery_portfolio::ProgramHypothesisSetV1>,
    },
    SchemaSpec {
        name: "portfolio-plan-v1.schema.json",
        writer: write_schema::<discovery_portfolio::PortfolioPlanV1>,
    },
    SchemaSpec {
        name: "experiment-campaign-v1.schema.json",
        writer: write_schema::<discovery_portfolio::ExperimentCampaignV1>,
    },
    SchemaSpec {
        name: "campaign-decision-trace-v1.schema.json",
        writer: write_schema::<discovery_portfolio::CampaignDecisionTraceV1>,
    },
    SchemaSpec {
        name: "information-value-estimate-v1.schema.json",
        writer: write_schema::<discovery_portfolio::InformationValueEstimateV1>,
    },
    SchemaSpec {
        name: "verification-load-budget-v1.schema.json",
        writer: write_schema::<discovery_portfolio::VerificationLoadBudgetV1>,
    },
    SchemaSpec {
        name: "charter-bundle-v1.schema.json",
        writer: write_schema::<constitutional_memory::CharterBundleV1>,
    },
    SchemaSpec {
        name: "doctrine-snapshot-v1.schema.json",
        writer: write_schema::<constitutional_memory::DoctrineSnapshotV1>,
    },
    SchemaSpec {
        name: "amendment-proposal-v1.schema.json",
        writer: write_schema::<constitutional_memory::AmendmentProposalV1>,
    },
    SchemaSpec {
        name: "amendment-decision-v1.schema.json",
        writer: write_schema::<constitutional_memory::AmendmentDecisionV1>,
    },
    SchemaSpec {
        name: "archive-manifest-v1.schema.json",
        writer: write_schema::<constitutional_memory::ArchiveManifestV1>,
    },
    SchemaSpec {
        name: "compaction-receipt-v1.schema.json",
        writer: write_schema::<constitutional_memory::CompactionReceiptV1>,
    },
    SchemaSpec {
        name: "historical-query-guarantee-v1.schema.json",
        writer: write_schema::<constitutional_memory::HistoricalQueryGuaranteeV1>,
    },
    SchemaSpec {
        name: "deprecation-bundle-v1.schema.json",
        writer: write_schema::<constitutional_memory::DeprecationBundleV1>,
    },
    SchemaSpec {
        name: "retirement-bundle-v1.schema.json",
        writer: write_schema::<constitutional_memory::RetirementBundleV1>,
    },
    SchemaSpec {
        name: "spec-bundle-v1.schema.json",
        writer: write_schema::<spec_execution::SpecBundleV1>,
    },
    SchemaSpec {
        name: "normative-ast-v1.schema.json",
        writer: write_schema::<spec_execution::NormativeASTV1>,
    },
    SchemaSpec {
        name: "generated-schema-bundle-v1.schema.json",
        writer: write_schema::<spec_execution::GeneratedSchemaBundleV1>,
    },
    SchemaSpec {
        name: "generated-interpreter-bundle-v1.schema.json",
        writer: write_schema::<spec_execution::GeneratedInterpreterBundleV1>,
    },
    SchemaSpec {
        name: "generated-conformance-corpus-v1.schema.json",
        writer: write_schema::<spec_execution::GeneratedConformanceCorpusV1>,
    },
    SchemaSpec {
        name: "generated-migration-plan-v1.schema.json",
        writer: write_schema::<spec_execution::GeneratedMigrationPlanV1>,
    },
    SchemaSpec {
        name: "proof-obligation-set-v1.schema.json",
        writer: write_schema::<spec_execution::ProofObligationSetV1>,
    },
    SchemaSpec {
        name: "proof-evaluation-receipt-v1.schema.json",
        writer: write_schema::<spec_execution::ProofEvaluationReceiptV1>,
    },
    SchemaSpec {
        name: "human-veto-bundle-v1.schema.json",
        writer: write_schema::<spec_execution::HumanVetoBundleV1>,
    },
    SchemaSpec {
        name: "meta-challenge-bundle-v1.schema.json",
        writer: write_schema::<spec_execution::MetaChallengeBundleV1>,
    },
    SchemaSpec {
        name: "self-hosting-build-receipt-v1.schema.json",
        writer: write_schema::<spec_execution::SelfHostingBuildReceiptV1>,
    },
    SchemaSpec {
        name: "effect-policy-profile-v1.schema.json",
        writer: write_schema::<verification_policy::EffectPolicyProfileV1>,
    },
    SchemaSpec {
        name: "delegation-policy-profile-v1.schema.json",
        writer: write_schema::<verification_policy::DelegationPolicyProfileV1>,
    },
    SchemaSpec {
        name: "release-policy-profile-v1.schema.json",
        writer: write_schema::<verification_policy::ReleasePolicyProfileV1>,
    },
    SchemaSpec {
        name: "continuity-policy-profile-v1.schema.json",
        writer: write_schema::<verification_policy::ContinuityPolicyProfileV1>,
    },
    SchemaSpec {
        name: "applicability-context-v1.schema.json",
        writer: write_schema::<profile_runtime::ApplicabilityContextV1>,
    },
    SchemaSpec {
        name: "profile-set-v1.schema.json",
        writer: write_schema::<profile_runtime::ProfileSetV1>,
    },
    SchemaSpec {
        name: "composition-rule-set-v1.schema.json",
        writer: write_schema::<profile_runtime::CompositionRuleSetV1>,
    },
    SchemaSpec {
        name: "composition-receipt-v1.schema.json",
        writer: write_schema::<profile_runtime::CompositionReceiptV1>,
    },
    SchemaSpec {
        name: "effective-constitution-v1.schema.json",
        writer: write_schema::<profile_runtime::EffectiveConstitutionV1>,
    },
    SchemaSpec {
        name: "compiled-obligation-set-v1.schema.json",
        writer: write_schema::<profile_runtime::CompiledObligationSetV1>,
    },
    SchemaSpec {
        name: "composition-conflict-set-v1.schema.json",
        writer: write_schema::<profile_runtime::CompositionConflictSetV1>,
    },
    SchemaSpec {
        name: "profile-exception-bundle-v1.schema.json",
        writer: write_schema::<profile_runtime::ProfileExceptionBundleV1>,
    },
    SchemaSpec {
        name: "policy-impact-diff-v1.schema.json",
        writer: write_schema::<profile_runtime::PolicyImpactDiffV1>,
    },
    SchemaSpec {
        name: "access-purpose-matrix-v1.schema.json",
        writer: write_schema::<verification_policy::AccessPurposeMatrixV1>,
    },
    SchemaSpec {
        name: "approval-matrix-v1.schema.json",
        writer: write_schema::<authority_delegation::ApprovalMatrixV1>,
    },
    SchemaSpec {
        name: "audit-extraction-policy-v1.schema.json",
        writer: write_schema::<verification_policy::AuditExtractionPolicyV1>,
    },
    SchemaSpec {
        name: "conflict-class-catalog-v1.schema.json",
        writer: write_schema::<authority_delegation::ConflictClassCatalogV1>,
    },
    SchemaSpec {
        name: "cross-boundary-transfer-class-v1.schema.json",
        writer: write_schema::<verification_policy::CrossBoundaryTransferClassV1>,
    },
    SchemaSpec {
        name: "delegation-matrix-v1.schema.json",
        writer: write_schema::<authority_delegation::DelegationMatrixV1>,
    },
    SchemaSpec {
        name: "escalation-clock-policy-v1.schema.json",
        writer: write_schema::<continuity_runtime::EscalationClockPolicyV1>,
    },
    SchemaSpec {
        name: "evidence-collection-plan-v1.schema.json",
        writer: write_schema::<assurance_runtime::EvidenceCollectionPlanV1>,
    },
    SchemaSpec {
        name: "hazard-library-v1.schema.json",
        writer: write_schema::<assurance_runtime::HazardLibraryV1>,
    },
    SchemaSpec {
        name: "hazard-scenario-v1.schema.json",
        writer: write_schema::<assurance_runtime::HazardScenarioV1>,
    },
    SchemaSpec {
        name: "incident-taxonomy-v1.schema.json",
        writer: write_schema::<continuity_runtime::IncidentTaxonomyV1>,
    },
    SchemaSpec {
        name: "locality-exception-v1.schema.json",
        writer: write_schema::<verification_policy::LocalityExceptionV1>,
    },
    SchemaSpec {
        name: "mitigation-playbook-v1.schema.json",
        writer: write_schema::<assurance_runtime::MitigationPlaybookV1>,
    },
    SchemaSpec {
        name: "monitor-catalog-v1.schema.json",
        writer: write_schema::<assurance_runtime::MonitorCatalogV1>,
    },
    SchemaSpec {
        name: "pager-route-profile-v1.schema.json",
        writer: write_schema::<continuity_runtime::PagerRouteProfileV1>,
    },
    SchemaSpec {
        name: "privacy-retention-profile-v1.schema.json",
        writer: write_schema::<verification_policy::PrivacyRetentionProfileV1>,
    },
    SchemaSpec {
        name: "recertification-schedule-v1.schema.json",
        writer: write_schema::<assurance_runtime::RecertificationScheduleV1>,
    },
    SchemaSpec {
        name: "redaction-rule-set-v1.schema.json",
        writer: write_schema::<verification_policy::RedactionRuleSetV1>,
    },
    SchemaSpec {
        name: "regulatory-regime-profile-v1.schema.json",
        writer: write_schema::<assurance_runtime::RegulatoryRegimeProfileV1>,
    },
    SchemaSpec {
        name: "requirement-control-map-v1.schema.json",
        writer: write_schema::<assurance_runtime::RequirementControlMapV1>,
    },
    SchemaSpec {
        name: "residency-policy-profile-v1.schema.json",
        writer: write_schema::<verification_policy::ResidencyPolicyProfileV1>,
    },
    SchemaSpec {
        name: "role-catalog-v1.schema.json",
        writer: write_schema::<authority_delegation::RoleCatalogV1>,
    },
    SchemaSpec {
        name: "severity-matrix-v1.schema.json",
        writer: write_schema::<continuity_runtime::SeverityMatrixV1>,
    },
    SchemaSpec {
        name: "tenant-boundary-profile-v1.schema.json",
        writer: write_schema::<verification_policy::TenantBoundaryProfileV1>,
    },
    SchemaSpec {
        name: "vendor-certification-adapter-v1.schema.json",
        writer: write_schema::<attestation_exchange::VendorCertificationAdapterV1>,
    },
    SchemaSpec {
        name: "vendor-evidence-translation-v1.schema.json",
        writer: write_schema::<attestation_exchange::VendorEvidenceTranslationV1>,
    },
    SchemaSpec {
        name: "vendor-revocation-handling-v1.schema.json",
        writer: write_schema::<attestation_exchange::VendorRevocationHandlingV1>,
    },
    SchemaSpec {
        name: "vendor-trust-root-binding-v1.schema.json",
        writer: write_schema::<attestation_exchange::VendorTrustRootBindingV1>,
    },
    SchemaSpec {
        name: "bridge-import-failure-artifact-v1.schema.json",
        writer: write_schema::<forge_memory_bridge::BridgeImportFailureArtifact>,
    },
];

/// Generates every registered schema into the supplied output directory.
pub fn generate_schemas(out_dir: &Path) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
    fs::create_dir_all(out_dir)?;

    let mut written = Vec::with_capacity(SCHEMA_SPECS.len());
    for spec in SCHEMA_SPECS {
        (spec.writer)(out_dir, spec.name)?;
        written.push(out_dir.join(spec.name));
    }

    Ok(written)
}

/// Returns the canonical checked-in schema directory.
pub fn canonical_schema_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../schemas")
}

/// Verifies that a directory exactly matches the generated canonical schema set.
pub fn check_against_dir(expected_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    if !expected_dir.is_dir() {
        return Err(format!(
            "schema compatibility check failed: missing canonical schemas directory '{}'",
            expected_dir.display()
        )
        .into());
    }

    let temp_dir = tempfile::tempdir()?;
    let generated = generate_schemas(temp_dir.path())?;

    for generated_path in generated {
        let file_name = generated_path.file_name().ok_or_else(|| {
            format!(
                "generated schema path '{}' has no filename",
                generated_path.display()
            )
        })?;
        let expected_path = expected_dir.join(file_name);
        if !expected_path.is_file() {
            return Err(format!("missing canonical schema '{}'", expected_path.display()).into());
        }

        let expected = fs::read_to_string(&expected_path)?;
        let actual = fs::read_to_string(&generated_path)?;
        if expected != actual {
            return Err(format!("schema drift detected for '{}'", expected_path.display()).into());
        }
    }

    let mut canonical_entries = fs::read_dir(expected_dir)?
        .map(|entry| entry.map(|entry| entry.path()))
        .collect::<Result<Vec<_>, _>>()?;
    canonical_entries.sort();

    if canonical_entries.len() != SCHEMA_SPECS.len() {
        return Err(format!(
            "schema directory '{}' contains {} files but generator owns {}",
            expected_dir.display(),
            canonical_entries.len(),
            SCHEMA_SPECS.len()
        )
        .into());
    }

    for expected in canonical_entries {
        if !SCHEMA_SPECS
            .iter()
            .any(|spec| expected.ends_with(spec.name))
        {
            return Err(format!("unexpected canonical schema '{}'", expected.display()).into());
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generates_expected_schema_set() {
        let temp = tempfile::tempdir().unwrap();
        let written = generate_schemas(temp.path()).unwrap();

        assert_eq!(written.len(), SCHEMA_SPECS.len());
        for spec in SCHEMA_SPECS {
            assert!(
                temp.path().join(spec.name).is_file(),
                "expected '{}' to be generated",
                spec.name
            );
        }
    }

    #[test]
    fn canonical_schemas_match_generator_output() {
        check_against_dir(&canonical_schema_dir()).unwrap();
    }

    #[test]
    fn check_fails_when_schema_file_drifts() {
        let temp = tempfile::tempdir().unwrap();
        generate_schemas(temp.path()).unwrap();
        let drifted = temp.path().join(SCHEMA_SPECS[0].name);
        fs::write(&drifted, "{\n  \"drifted\": true\n}\n").unwrap();

        let err = check_against_dir(temp.path()).unwrap_err();
        assert!(err.to_string().contains("schema drift detected"));
    }

    #[test]
    fn execution_evidence_schema_family_is_registered() {
        let required = [
            "forge-tool-receipt-v2.schema.json",
            "episode-bundle-v1.schema.json",
            "execution-context-v1.schema.json",
            "control-receipt-v1.schema.json",
            "loop-iteration-report-v1.schema.json",
            "verification-plan-artifact-v1.schema.json",
        ];

        for name in required {
            assert!(
                SCHEMA_SPECS.iter().any(|spec| spec.name == name),
                "missing execution-evidence schema registration for {name}"
            );
        }
    }

    // ─── LIB-C004: v9 constitutional artifact families must all have registered schemas ───

    #[test]
    fn v9_canonical_bundle_schemas_are_registered() {
        // LIB-C001: Freeze canonical episode/claim/evidence package law.
        // Every wire-visible v9 bundle artifact must be governed by contract-schema-gen.
        let required = [
            "episode-bundle-v1.schema.json",
            "evidence-bundle-v1.schema.json",
            "execution-context-v1.schema.json",
            "export-envelope-v3.schema.json",
            "projection-import-batch-v3.schema.json",
        ];
        for name in required {
            assert!(
                SCHEMA_SPECS.iter().any(|spec| spec.name == name),
                "LIB-C001: missing canonical bundle schema registration for {name}"
            );
        }
    }

    #[test]
    fn v9_execution_evidence_schemas_are_complete() {
        // LIB-C002: Freeze execution evidence artifacts.
        // All execution-evidence families must be governed.
        let required = [
            "forge-tool-receipt-v2.schema.json",
            "episode-bundle-v1.schema.json",
            "execution-context-v1.schema.json",
            "control-receipt-v1.schema.json",
            "scheduler-decision-v1.schema.json",
            "loop-iteration-report-v1.schema.json",
            "loop-report-v1.schema.json",
            "verification-plan-artifact-v1.schema.json",
            "repair-record-v1.schema.json",
            "runtime-query-provenance-v1.schema.json",
            "import-failure-record-v1.schema.json",
        ];
        for name in required {
            assert!(
                SCHEMA_SPECS.iter().any(|spec| spec.name == name),
                "LIB-C002: missing execution-evidence schema registration for {name}"
            );
        }
    }

    #[test]
    fn v9_repair_law_schema_is_registered() {
        // LIB-C008: Repair law hardening.
        // RepairRecordV1 must be a governed schema, not folkloric.
        assert!(
            SCHEMA_SPECS
                .iter()
                .any(|spec| spec.name == "repair-record-v1.schema.json"),
            "LIB-C008: RepairRecordV1 must be a governed schema"
        );
    }

    #[test]
    fn v9_runtime_provenance_schema_is_registered() {
        // LIB-C009: Runtime provenance completeness.
        // RuntimeQueryProvenanceV1 must be a governed schema.
        assert!(
            SCHEMA_SPECS
                .iter()
                .any(|spec| spec.name == "runtime-query-provenance-v1.schema.json"),
            "LIB-C009: RuntimeQueryProvenanceV1 must be a governed schema"
        );
    }

    #[test]
    fn v9_bridge_atomicity_schemas_are_registered() {
        // LIB-C005: Bridge atomicity proofs.
        // Bridge failure and region transport artifacts must be governed schemas.
        let required = [
            "import-failure-record-v1.schema.json",
            "boundary-repair-record-v1.schema.json",
            "region-artifact-transport-v1.schema.json",
            "syndrome-route-record-v1.schema.json",
            "bridge-import-failure-artifact-v1.schema.json",
        ];
        for name in required {
            assert!(
                SCHEMA_SPECS.iter().any(|spec| spec.name == name),
                "LIB-C005: missing bridge atomicity schema registration for {name}"
            );
        }
    }

    #[test]
    fn v9_governance_review_schemas_are_registered() {
        // LIB-C004: Schema governance enforcement.
        // All governance review case families must be governed schemas.
        let required = [
            "effect-review-case-v1.schema.json",
            "effect-block-receipt-v1.schema.json",
            "delegation-review-case-v1.schema.json",
            "release-gate-case-v1.schema.json",
            "continuity-review-case-v1.schema.json",
            "verification-case-v1.schema.json",
            "check-plan-v1.schema.json",
            "verification-attempt-v1.schema.json",
        ];
        for name in required {
            assert!(
                SCHEMA_SPECS.iter().any(|spec| spec.name == name),
                "LIB-C004: missing governance review schema registration for {name}"
            );
        }
    }
}
