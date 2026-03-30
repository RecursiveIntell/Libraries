//! Property-based tests for mechanism-runtime artifact types.
//!
//! Validates serde roundtrip fidelity and schema_version constant usage
//! for top-level artifact families. FitRunV1 contains an f64 field
//! (fit_score), so roundtrip comparisons use JSON string equality.

use mechanism_runtime::*;
use proptest::prelude::*;
use stack_ids::{
    FitRunId, MechanismBundleId, RolloutStabilityReportId, SimulationContractId, SurfaceStatus,
    TheoryRefuterSuiteId, TheoryVersionId,
};

// --- Generators ---

fn arb_string() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9_-]{0,30}".prop_map(|s| s)
}

fn arb_id() -> impl Strategy<Value = String> {
    "[a-z]{3}-[a-z0-9]{8}".prop_map(|s| s)
}

fn arb_vec_string() -> impl Strategy<Value = Vec<String>> {
    prop::collection::vec(arb_string(), 0..3)
}

fn arb_surface_status() -> impl Strategy<Value = SurfaceStatus> {
    prop_oneof![
        Just(SurfaceStatus::AdvisoryOnly),
        Just(SurfaceStatus::NonAdmitted),
        Just(SurfaceStatus::Degraded),
        Just(SurfaceStatus::HorizonOnly),
    ]
}

fn arb_mechanism_bundle_id() -> impl Strategy<Value = MechanismBundleId> {
    arb_id().prop_map(MechanismBundleId::new)
}

fn arb_theory_version_id() -> impl Strategy<Value = TheoryVersionId> {
    arb_id().prop_map(TheoryVersionId::new)
}

fn arb_simulation_contract_id() -> impl Strategy<Value = SimulationContractId> {
    arb_id().prop_map(SimulationContractId::new)
}

fn arb_fit_run_id() -> impl Strategy<Value = FitRunId> {
    arb_id().prop_map(FitRunId::new)
}

fn arb_theory_refuter_suite_id() -> impl Strategy<Value = TheoryRefuterSuiteId> {
    arb_id().prop_map(TheoryRefuterSuiteId::new)
}

fn arb_rollout_stability_report_id() -> impl Strategy<Value = RolloutStabilityReportId> {
    arb_id().prop_map(RolloutStabilityReportId::new)
}

fn arb_fit_disposition() -> impl Strategy<Value = FitDisposition> {
    prop_oneof![
        Just(FitDisposition::AdvisoryFitOnly),
        Just(FitDisposition::PromotionBlockedMissingRefuter),
        Just(FitDisposition::PromotionBlockedFailingRefuter),
        Just(FitDisposition::PromotionBlockedStabilityRisk),
        Just(FitDisposition::EligibleForLocalReview),
    ]
}

// --- Top-level artifact generators ---

fn arb_mechanism_bundle_v1() -> impl Strategy<Value = MechanismBundleV1> {
    (
        arb_mechanism_bundle_id(),
        arb_string(),
        arb_string(),
        arb_surface_status(),
    )
        .prop_map(
            |(mechanism_bundle_id, mechanism_name, canonical_owner, publication_status)| {
                MechanismBundleV1 {
                    schema_version: MECHANISM_BUNDLE_V1_SCHEMA.into(),
                    mechanism_bundle_id,
                    mechanism_name,
                    canonical_owner,
                    publication_status,
                }
            },
        )
}

fn arb_theory_refuter_suite_v1() -> impl Strategy<Value = TheoryRefuterSuiteV1> {
    (
        arb_theory_refuter_suite_id(),
        arb_theory_version_id(),
        arb_vec_string(),
        arb_vec_string(),
        arb_vec_string(),
        prop::bool::ANY,
    )
        .prop_map(
            |(
                theory_refuter_suite_id,
                theory_version_id,
                required_refuters,
                available_refuters,
                failing_refuters,
                horizon_only,
            )| {
                TheoryRefuterSuiteV1 {
                    schema_version: THEORY_REFUTER_SUITE_V1_SCHEMA.into(),
                    theory_refuter_suite_id,
                    theory_version_id,
                    required_refuters,
                    available_refuters,
                    failing_refuters,
                    horizon_only,
                }
            },
        )
}

fn arb_fit_run_v1() -> impl Strategy<Value = FitRunV1> {
    (
        (
            arb_fit_run_id(),
            arb_mechanism_bundle_id(),
            arb_theory_version_id(),
            arb_simulation_contract_id(),
            0.0..1.0f64,
            arb_fit_disposition(),
            prop::bool::ANY,
        ),
        (
            prop::bool::ANY,
            prop::bool::ANY,
            prop::bool::ANY,
            arb_theory_refuter_suite_id(),
            arb_rollout_stability_report_id(),
            arb_vec_string(),
            arb_string(),
        ),
    )
        .prop_map(
            |(
                (
                    fit_run_id,
                    mechanism_bundle_id,
                    theory_version_id,
                    simulation_contract_id,
                    fit_score,
                    disposition,
                    advisory_only,
                ),
                (
                    degraded,
                    refuter_ready,
                    stability_clear,
                    theory_refuter_suite_id,
                    rollout_stability_report_id,
                    notes,
                    generated_at,
                ),
            )| {
                FitRunV1 {
                    schema_version: FIT_RUN_V1_SCHEMA.into(),
                    fit_run_id,
                    mechanism_bundle_id,
                    theory_version_id,
                    simulation_contract_id,
                    fit_score,
                    disposition,
                    advisory_only,
                    degraded,
                    refuter_ready,
                    stability_clear,
                    theory_refuter_suite_id,
                    rollout_stability_report_id,
                    notes,
                    generated_at,
                }
            },
        )
}

// --- Property tests ---

proptest! {
    #[test]
    fn mechanism_bundle_v1_serde_roundtrip(val in arb_mechanism_bundle_v1()) {
        let json = serde_json::to_string(&val).unwrap();
        let back: MechanismBundleV1 = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(&val, &back);
    }

    #[test]
    fn mechanism_bundle_v1_schema_version_constant(val in arb_mechanism_bundle_v1()) {
        prop_assert_eq!(val.schema_version.as_str(), MECHANISM_BUNDLE_V1_SCHEMA);
    }

    #[test]
    fn theory_refuter_suite_v1_serde_roundtrip(val in arb_theory_refuter_suite_v1()) {
        let json = serde_json::to_string(&val).unwrap();
        let back: TheoryRefuterSuiteV1 = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(&val, &back);
    }

    #[test]
    fn theory_refuter_suite_v1_schema_version_constant(val in arb_theory_refuter_suite_v1()) {
        prop_assert_eq!(val.schema_version.as_str(), THEORY_REFUTER_SUITE_V1_SCHEMA);
    }

    /// FitRunV1 contains f64 (fit_score), so roundtrip compares fields individually.
    #[test]
    fn fit_run_v1_serde_roundtrip(val in arb_fit_run_v1()) {
        let json = serde_json::to_string(&val).unwrap();
        let back: FitRunV1 = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(&val.schema_version, &back.schema_version);
        prop_assert_eq!(&val.fit_run_id, &back.fit_run_id);
        prop_assert_eq!(&val.mechanism_bundle_id, &back.mechanism_bundle_id);
        prop_assert_eq!(&val.disposition, &back.disposition);
        prop_assert!((val.fit_score - back.fit_score).abs() < 1e-10);
        prop_assert_eq!(val.advisory_only, back.advisory_only);
        prop_assert_eq!(val.degraded, back.degraded);
        prop_assert_eq!(val.refuter_ready, back.refuter_ready);
        prop_assert_eq!(val.stability_clear, back.stability_clear);
    }

    #[test]
    fn fit_run_v1_schema_version_constant(val in arb_fit_run_v1()) {
        prop_assert_eq!(val.schema_version.as_str(), FIT_RUN_V1_SCHEMA);
    }
}
