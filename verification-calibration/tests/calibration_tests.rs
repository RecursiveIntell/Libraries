use verification_calibration::{CalibrationSnapshot, CALIBRATION_SNAPSHOT_V1_SCHEMA};

/// GIVEN uniform comparability version, no caveats, within threshold
/// WHEN CalibrationSnapshot::evaluate is called
/// THEN forces_advisory_only is false.
#[test]
fn test_snapshot_all_passing() {
    let snapshot = CalibrationSnapshot::evaluate(
        stack_ids::VerificationCaseId::new("case-all-pass"),
        "2026-03-25T00:00:00Z",
        true, // comparability_available
        true, // oracle_calibrated
        500_000,
        100_000,
        vec![],
    );

    assert!(!snapshot.forces_advisory_only);
    assert!(snapshot.comparability_available);
    assert!(snapshot.oracle_calibrated);
    assert_eq!(snapshot.schema_version, CALIBRATION_SNAPSHOT_V1_SCHEMA);
    assert!(snapshot.drift_markers.is_empty());
    assert!(snapshot.observed_risk_micros < snapshot.abstention_threshold_micros);
}

/// GIVEN calibration caveats present (drift markers)
/// WHEN CalibrationSnapshot::evaluate is called
/// THEN forces_advisory_only is true.
#[test]
fn test_snapshot_forces_advisory_on_caveats() {
    let snapshot = CalibrationSnapshot::evaluate(
        stack_ids::VerificationCaseId::new("case-caveat"),
        "2026-03-25T00:00:00Z",
        true,
        true,
        500_000,
        100_000,
        vec!["oracle_version_mismatch".into()],
    );

    assert!(snapshot.forces_advisory_only);
    assert_eq!(snapshot.drift_markers.len(), 1);
    assert_eq!(snapshot.drift_markers[0], "oracle_version_mismatch");
}

/// GIVEN multiple distinct comparability snapshot versions (drift markers)
/// WHEN CalibrationSnapshot::evaluate is called
/// THEN the snapshot reflects non-uniform state and forces advisory.
#[test]
fn test_snapshot_version_drift() {
    let drift = vec![
        "v1.0_to_v1.1_schema_drift".into(),
        "v1.1_to_v1.2_oracle_recalibration".into(),
        "v1.2_to_v1.3_threshold_shift".into(),
    ];
    let snapshot = CalibrationSnapshot::evaluate(
        stack_ids::VerificationCaseId::new("case-drift"),
        "2026-03-25T00:00:00Z",
        true,
        true,
        500_000,
        100_000,
        drift.clone(),
    );

    assert!(snapshot.forces_advisory_only);
    assert_eq!(snapshot.drift_markers.len(), 3);
    assert_eq!(snapshot.drift_markers, drift);
}

/// GIVEN abstention threshold set to 100_000 and observed risk exceeds it
/// WHEN CalibrationSnapshot::evaluate is called
/// THEN forces_advisory_only is true.
#[test]
fn test_snapshot_abstention_threshold_exceeded() {
    let snapshot = CalibrationSnapshot::evaluate(
        stack_ids::VerificationCaseId::new("case-exceeded"),
        "2026-03-25T00:00:00Z",
        true,
        true,
        100_000, // threshold
        200_000, // observed risk exceeds threshold
        vec![],
    );

    assert!(snapshot.forces_advisory_only);
    assert!(snapshot.observed_risk_micros > snapshot.abstention_threshold_micros);
}

/// GIVEN observed risk exactly equals abstention threshold
/// WHEN CalibrationSnapshot::evaluate is called
/// THEN forces_advisory_only is true (boundary: >= triggers advisory).
#[test]
fn test_snapshot_abstention_threshold_boundary() {
    let threshold = 100_000u64;
    let snapshot = CalibrationSnapshot::evaluate(
        stack_ids::VerificationCaseId::new("case-boundary"),
        "2026-03-25T00:00:00Z",
        true,
        true,
        threshold,
        threshold, // exactly at threshold
        vec![],
    );

    assert!(snapshot.forces_advisory_only);
    assert_eq!(
        snapshot.observed_risk_micros,
        snapshot.abstention_threshold_micros
    );

    // Confirm that one micro below threshold does NOT force advisory
    let snapshot_below = CalibrationSnapshot::evaluate(
        stack_ids::VerificationCaseId::new("case-boundary-below"),
        "2026-03-25T00:00:00Z",
        true,
        true,
        threshold,
        threshold - 1, // one micro below
        vec![],
    );

    assert!(!snapshot_below.forces_advisory_only);
}

/// GIVEN degradation markers are present
/// WHEN CalibrationSnapshot::evaluate is called
/// THEN degradation markers appear in the snapshot output and advisory is forced.
#[test]
fn test_snapshot_degraded_input() {
    let degradations = vec![
        "embedding_model_downgraded".into(),
        "retrieval_latency_elevated".into(),
    ];
    let snapshot = CalibrationSnapshot::evaluate(
        stack_ids::VerificationCaseId::new("case-degraded"),
        "2026-03-25T00:00:00Z",
        true,
        true,
        500_000,
        100_000,
        degradations.clone(),
    );

    assert!(snapshot.forces_advisory_only);
    assert_eq!(snapshot.drift_markers, degradations);
    assert!(snapshot
        .drift_markers
        .contains(&"embedding_model_downgraded".to_string()));
    assert!(snapshot
        .drift_markers
        .contains(&"retrieval_latency_elevated".to_string()));
}

/// GIVEN a CalibrationSnapshot with all fields populated
/// WHEN serialized to JSON and deserialized back
/// THEN the result equals the original.
#[test]
fn test_snapshot_serialization_roundtrip() {
    let original = CalibrationSnapshot::evaluate(
        stack_ids::VerificationCaseId::new("case-serde"),
        "2026-03-25T12:00:00Z",
        true,
        false,
        250_000,
        125_000,
        vec!["marker_alpha".into(), "marker_beta".into()],
    );

    let json = serde_json::to_string(&original).unwrap();
    let deserialized: CalibrationSnapshot = serde_json::from_str(&json).unwrap();

    assert_eq!(original, deserialized);
    assert_eq!(deserialized.schema_version, CALIBRATION_SNAPSHOT_V1_SCHEMA);
    assert_eq!(
        deserialized.case_id,
        stack_ids::VerificationCaseId::new("case-serde")
    );
    assert_eq!(deserialized.abstention_threshold_micros, 250_000);
    assert_eq!(deserialized.observed_risk_micros, 125_000);
    assert_eq!(deserialized.drift_markers.len(), 2);
    assert!(deserialized.forces_advisory_only); // oracle not calibrated
}

/// GIVEN no degradation markers and all other conditions passing
/// WHEN CalibrationSnapshot::evaluate is called
/// THEN degradation list is empty and forces_advisory_only depends only on
///      comparability, oracle calibration, and threshold.
#[test]
fn test_snapshot_empty_degradations() {
    // All passing with empty degradations: advisory should be false
    let snapshot_ok = CalibrationSnapshot::evaluate(
        stack_ids::VerificationCaseId::new("case-empty-degrad-ok"),
        "2026-03-25T00:00:00Z",
        true,
        true,
        500_000,
        100_000,
        vec![],
    );

    assert!(snapshot_ok.drift_markers.is_empty());
    assert!(!snapshot_ok.forces_advisory_only);

    // No degradations but oracle not calibrated: advisory should be true
    let snapshot_uncal = CalibrationSnapshot::evaluate(
        stack_ids::VerificationCaseId::new("case-empty-degrad-uncal"),
        "2026-03-25T00:00:00Z",
        true,
        false, // oracle not calibrated
        500_000,
        100_000,
        vec![],
    );

    assert!(snapshot_uncal.drift_markers.is_empty());
    assert!(snapshot_uncal.forces_advisory_only);

    // No degradations but comparability unavailable: advisory should be true
    let snapshot_nocomp = CalibrationSnapshot::evaluate(
        stack_ids::VerificationCaseId::new("case-empty-degrad-nocomp"),
        "2026-03-25T00:00:00Z",
        false, // comparability unavailable
        true,
        500_000,
        100_000,
        vec![],
    );

    assert!(snapshot_nocomp.drift_markers.is_empty());
    assert!(snapshot_nocomp.forces_advisory_only);
}
