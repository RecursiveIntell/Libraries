use recursive_kernel_core::*;
use stack_ids::{ConstraintId, KernelRunId, OperatorId, OracleSliceId, ScopeKey, SyndromeId};

// ---------------------------------------------------------------------------
// Helper: builds a valid OperatorMetadata for reuse across tests
// ---------------------------------------------------------------------------
fn valid_operator_metadata() -> OperatorMetadata {
    OperatorMetadata {
        operator_id: OperatorId::new("test.operator.v1"),
        name: "test_operator".to_string(),
        contract: OperatorContract {
            inputs: vec!["input_a".into()],
            outputs: vec!["output_b".into()],
            exactness: ExactnessClass::Exact,
            convergence: ConvergenceKind::AcyclicSinglePass,
            stop_rule: StopRule::AcyclicCompletion,
            degradation: DegradationBehavior::AdvisoryOnly,
        },
    }
}

// ---------------------------------------------------------------------------
// 1. OperatorMetadata validate: empty name fails
// ---------------------------------------------------------------------------
#[test]
fn test_operator_validate_empty_name_fails() {
    let mut meta = valid_operator_metadata();
    meta.name = "".to_string();
    let result = meta.validate();
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "operator name must not be empty");
}

// ---------------------------------------------------------------------------
// 2. OperatorMetadata validate: empty inputs fails
// ---------------------------------------------------------------------------
#[test]
fn test_operator_validate_empty_inputs_fails() {
    let mut meta = valid_operator_metadata();
    meta.contract.inputs = vec![];
    let result = meta.validate();
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "operator inputs must not be empty");
}

// ---------------------------------------------------------------------------
// 3. OperatorMetadata validate: empty outputs fails
// ---------------------------------------------------------------------------
#[test]
fn test_operator_validate_empty_outputs_fails() {
    let mut meta = valid_operator_metadata();
    meta.contract.outputs = vec![];
    let result = meta.validate();
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "operator outputs must not be empty");
}

// ---------------------------------------------------------------------------
// 4. OperatorMetadata validate: valid contract passes
// ---------------------------------------------------------------------------
#[test]
fn test_operator_validate_success() {
    let meta = valid_operator_metadata();
    assert!(meta.validate().is_ok());
}

// ---------------------------------------------------------------------------
// 5. KernelRun always returns NonAuthoritativeDerived
// ---------------------------------------------------------------------------
#[test]
fn test_kernel_run_authority_class() {
    let run_without_oracle = KernelRun {
        run_id: KernelRunId::new("run-alpha"),
        policy_version: "policy-v2".into(),
        scope_key: ScopeKey::namespace_only("ns-test"),
        operator_ids: vec![OperatorId::new(CONSTRAINT_COMPILER_OPERATOR_ID)],
        oracle_slice_id: None,
        degradation_active: false,
    };
    assert_eq!(
        run_without_oracle.authority_class(),
        ArtifactAuthorityClass::NonAuthoritativeDerived
    );

    let run_with_oracle = KernelRun {
        run_id: KernelRunId::new("run-beta"),
        policy_version: "policy-v3".into(),
        scope_key: ScopeKey::namespace_only("ns-test"),
        operator_ids: vec![
            OperatorId::new(CONSTRAINT_COMPILER_OPERATOR_ID),
            OperatorId::new(RECURSIVE_MESSAGE_PASSING_OPERATOR_ID),
        ],
        oracle_slice_id: Some(OracleSliceId::new("oracle-1")),
        degradation_active: true,
    };
    assert_eq!(
        run_with_oracle.authority_class(),
        ArtifactAuthorityClass::NonAuthoritativeDerived
    );
}

// ---------------------------------------------------------------------------
// 6. Syndrome serialization roundtrip
// ---------------------------------------------------------------------------
#[test]
fn test_syndrome_serde_roundtrip() {
    let syndrome = Syndrome {
        syndrome_id: SyndromeId::new("syn-001"),
        signature: "constraint_violation::node_42".to_string(),
        blocked_by_degradation: true,
    };
    let json = serde_json::to_string(&syndrome).unwrap();
    let deserialized: Syndrome = serde_json::from_str(&json).unwrap();
    assert_eq!(syndrome, deserialized);
}

// ---------------------------------------------------------------------------
// 7. ResidualArtifact with monotone flag
// ---------------------------------------------------------------------------
#[test]
fn test_residual_artifact_monotone_flag() {
    let residual_mono = ResidualArtifact {
        constraint_id: ConstraintId::new("mono"),
        residual_micros: 500,
        monotone_nonincreasing: true,
    };
    let json = serde_json::to_string(&residual_mono).unwrap();
    let deserialized: ResidualArtifact = serde_json::from_str(&json).unwrap();
    assert_eq!(residual_mono, deserialized);
    assert!(deserialized.monotone_nonincreasing);

    let residual_non_mono = ResidualArtifact {
        constraint_id: ConstraintId::new("nonmono"),
        residual_micros: 1200,
        monotone_nonincreasing: false,
    };
    let json = serde_json::to_string(&residual_non_mono).unwrap();
    let deserialized: ResidualArtifact = serde_json::from_str(&json).unwrap();
    assert!(!deserialized.monotone_nonincreasing);
}

// ---------------------------------------------------------------------------
// 8. WitnessArtifact with multiple supporting constraints
// ---------------------------------------------------------------------------
#[test]
fn test_witness_artifact_multiple_constraints() {
    let witness = WitnessArtifact {
        witness_id: "wit-multi".into(),
        target_node_id: "node-99".into(),
        supporting_constraint_ids: vec![
            ConstraintId::new("constraint:1"),
            ConstraintId::new("constraint:2"),
            ConstraintId::new("constraint:3"),
        ],
        belief_micros: 950_000,
    };
    let json = serde_json::to_string(&witness).unwrap();
    let deserialized: WitnessArtifact = serde_json::from_str(&json).unwrap();
    assert_eq!(witness, deserialized);
    assert_eq!(deserialized.supporting_constraint_ids.len(), 3);
    assert_eq!(
        deserialized.supporting_constraint_ids[0],
        ConstraintId::new("constraint:1")
    );
    assert_eq!(
        deserialized.supporting_constraint_ids[2],
        ConstraintId::new("constraint:3")
    );
}

// ---------------------------------------------------------------------------
// 9. CertificateArtifact with and without oracle slice
// ---------------------------------------------------------------------------
#[test]
fn test_certificate_with_oracle_slice() {
    let cert = CertificateArtifact {
        certificate_id: "cert-with-oracle".into(),
        certified_node_id: "node-certified".into(),
        satisfied_constraint_ids: vec![ConstraintId::new("sat1")],
        oracle_slice_id: Some(OracleSliceId::new("oracle-slice-42")),
    };
    let json = serde_json::to_string(&cert).unwrap();
    let deserialized: CertificateArtifact = serde_json::from_str(&json).unwrap();
    assert_eq!(cert, deserialized);
    assert!(deserialized.oracle_slice_id.is_some());
    assert_eq!(
        deserialized.oracle_slice_id.unwrap(),
        OracleSliceId::new("oracle-slice-42")
    );
}

#[test]
fn test_certificate_without_oracle_slice() {
    let cert = CertificateArtifact {
        certificate_id: "cert-no-oracle".into(),
        certified_node_id: "node-standalone".into(),
        satisfied_constraint_ids: vec![ConstraintId::new("sat1"), ConstraintId::new("sat2")],
        oracle_slice_id: None,
    };
    let json = serde_json::to_string(&cert).unwrap();
    let deserialized: CertificateArtifact = serde_json::from_str(&json).unwrap();
    assert_eq!(cert, deserialized);
    assert!(deserialized.oracle_slice_id.is_none());
    // Verify null-valued optional field is absent from serialized JSON
    let json_value: serde_json::Value = serde_json::from_str(&json).unwrap();
    // The field is present but null (serde default) or absent depending on serde config.
    // Either way, roundtrip equality is the binding property.
    if let Some(val) = json_value.get("oracle_slice_id") {
        assert!(val.is_null());
    }
}

// ---------------------------------------------------------------------------
// 10. KernelRefutationResult all three outcome variants
// ---------------------------------------------------------------------------
#[test]
fn test_refutation_result_flip_witness_found() {
    let result = KernelRefutationResult {
        target_node_id: "node-flip".into(),
        outcome: KernelRefutationOutcome::FlipWitnessFound,
        witness: Some(PerturbationWitness {
            changed_node_ids: vec!["node-a".into(), "node-b".into()],
            changed_constraint_ids: vec![ConstraintId::new("changed")],
            budget_used: 42,
        }),
        notes: vec!["flip detected at iteration 7".into()],
    };
    assert!(matches!(
        result.outcome,
        KernelRefutationOutcome::FlipWitnessFound
    ));
    assert!(result.witness.is_some());
    let witness = result.witness.as_ref().unwrap();
    assert_eq!(witness.changed_node_ids.len(), 2);
    assert_eq!(witness.budget_used, 42);
}

#[test]
fn test_refutation_result_no_flip_within_budget() {
    let result = KernelRefutationResult {
        target_node_id: "node-stable".into(),
        outcome: KernelRefutationOutcome::NoFlipFoundWithinBudget,
        witness: None,
        notes: vec!["exhausted budget of 100 perturbations".into()],
    };
    assert!(matches!(
        result.outcome,
        KernelRefutationOutcome::NoFlipFoundWithinBudget
    ));
    assert!(result.witness.is_none());
    assert_eq!(result.notes.len(), 1);
}

#[test]
fn test_refutation_result_unsupported() {
    let result = KernelRefutationResult {
        target_node_id: "node-unsupported".into(),
        outcome: KernelRefutationOutcome::Unsupported,
        witness: None,
        notes: vec![],
    };
    assert!(matches!(
        result.outcome,
        KernelRefutationOutcome::Unsupported
    ));
    assert!(result.witness.is_none());
    assert!(result.notes.is_empty());
}

// ---------------------------------------------------------------------------
// 11. PerturbationWitness serialization roundtrip
// ---------------------------------------------------------------------------
#[test]
fn test_perturbation_witness_serde_roundtrip() {
    let witness = PerturbationWitness {
        changed_node_ids: vec!["n1".into(), "n2".into(), "n3".into()],
        changed_constraint_ids: vec![ConstraintId::new("p1"), ConstraintId::new("p2")],
        budget_used: 17,
    };
    let json = serde_json::to_string(&witness).unwrap();
    let deserialized: PerturbationWitness = serde_json::from_str(&json).unwrap();
    assert_eq!(witness, deserialized);
}

// ---------------------------------------------------------------------------
// 12. CalibrationReport blocked flag behavior
// ---------------------------------------------------------------------------
#[test]
fn test_calibration_report_blocked() {
    let report = CalibrationReport {
        nuisance_node_ids: vec!["nuisance-1".into()],
        caveats: vec!["stale calibration data".into()],
        blocked: true,
    };
    let json = serde_json::to_string(&report).unwrap();
    let deserialized: CalibrationReport = serde_json::from_str(&json).unwrap();
    assert_eq!(report, deserialized);
    assert!(deserialized.blocked);
    assert_eq!(deserialized.caveats.len(), 1);
}

#[test]
fn test_calibration_report_not_blocked() {
    let report = CalibrationReport {
        nuisance_node_ids: vec![],
        caveats: vec![],
        blocked: false,
    };
    let json = serde_json::to_string(&report).unwrap();
    let deserialized: CalibrationReport = serde_json::from_str(&json).unwrap();
    assert_eq!(report, deserialized);
    assert!(!deserialized.blocked);
    assert!(deserialized.nuisance_node_ids.is_empty());
    assert!(deserialized.caveats.is_empty());
}

// ---------------------------------------------------------------------------
// 13. Whitespace-only name also fails validation
// ---------------------------------------------------------------------------
#[test]
fn test_operator_validate_whitespace_only_name_fails() {
    let mut meta = valid_operator_metadata();
    meta.name = "   ".to_string();
    let result = meta.validate();
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "operator name must not be empty");
}
