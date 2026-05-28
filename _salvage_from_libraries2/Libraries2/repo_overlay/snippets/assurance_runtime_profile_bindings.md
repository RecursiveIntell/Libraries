
# Suggested `assurance-runtime` profile bindings

Add:
- `src/profile_p4_regulated.rs`
- `src/profile_p5_hazard.rs`

P4 should bind to:
- `DeploymentProfileV1`
- `AssuranceCaseV1`
- `ControlMappingV1`
- `ReleaseReadinessDecisionV1`
- `CertificationBundleV1`

P5 should bind to:
- `HazardRegisterV1`
- `FieldMonitoringPlanV1`
- `ServiceLevelProfileV1`
- `ContainmentDecisionV1`
- `RecoveryPlanV1`
