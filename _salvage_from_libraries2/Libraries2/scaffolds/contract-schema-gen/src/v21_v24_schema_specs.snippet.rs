// Draft v21/v24 schema registrations.
// Adjust module paths only if final owner names differ.

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
