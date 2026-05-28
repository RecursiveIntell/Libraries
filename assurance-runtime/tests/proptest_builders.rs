#![allow(clippy::expect_used)]
#![allow(dead_code)]

use assurance_runtime::{AssuranceCaseV1, ControlCoverageStateV1, ControlMappingV1};
use proptest::prelude::*;

fn arb_non_empty_string() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9_]{2,20}".prop_map(|s| s)
}

fn arb_string_vec(min: usize) -> impl Strategy<Value = Vec<String>> {
    prop::collection::vec(arb_non_empty_string(), min..=5)
}

fn arb_coverage_state() -> impl Strategy<Value = ControlCoverageStateV1> {
    prop_oneof![
        Just(ControlCoverageStateV1::Partial),
        Just(ControlCoverageStateV1::GapPresent),
    ]
}

proptest! {
    #[test]
    fn assurance_case_valid_inputs_build(
        case_id in arb_non_empty_string(),
        claim in arb_non_empty_string(),
        evidence in arb_string_vec(1),
        argument in arb_non_empty_string(),
        obligations in arb_string_vec(0),
        ready in proptest::bool::ANY,
    ) {
        let result = AssuranceCaseV1::new(
            &case_id,
            &claim,
            evidence,
            &argument,
            obligations,
            ready,
        );
        prop_assert!(result.is_ok(), "valid inputs should build: {:?}", result.err());

        let case = result.unwrap();
        let json = serde_json::to_string(&case).unwrap();
        let roundtrip: AssuranceCaseV1 = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(roundtrip.assurance_case_id, case.assurance_case_id);
        prop_assert_eq!(roundtrip.ready_for_release, case.ready_for_release);
    }

    #[test]
    fn assurance_case_rejects_empty_case_id(
        claim in arb_non_empty_string(),
        evidence in arb_string_vec(1),
        argument in arb_non_empty_string(),
    ) {
        let result = AssuranceCaseV1::new("", &claim, evidence, &argument, vec![], false);
        prop_assert!(result.is_err());
    }

    #[test]
    fn assurance_case_rejects_empty_evidence(
        case_id in arb_non_empty_string(),
        claim in arb_non_empty_string(),
        argument in arb_non_empty_string(),
    ) {
        let result = AssuranceCaseV1::new(&case_id, &claim, vec![], &argument, vec![], false);
        prop_assert!(result.is_err());
    }

    #[test]
    fn control_mapping_valid_inputs_build(
        mapping_id in arb_non_empty_string(),
        hazard_id in arb_non_empty_string(),
        controls in arb_string_vec(1),
        coverage in arb_coverage_state(),
    ) {
        let result = ControlMappingV1::new(
            &mapping_id,
            &hazard_id,
            controls,
            coverage,
            vec!["gap-1".into()],
        );
        prop_assert!(result.is_ok(), "valid inputs should build: {:?}", result.err());
    }

    #[test]
    fn covered_mapping_rejects_gaps(
        mapping_id in arb_non_empty_string(),
        hazard_id in arb_non_empty_string(),
        controls in arb_string_vec(1),
        gaps in arb_string_vec(1),
    ) {
        let result = ControlMappingV1::new(
            &mapping_id,
            &hazard_id,
            controls,
            ControlCoverageStateV1::Covered,
            gaps,
        );
        prop_assert!(result.is_err(), "covered mapping with gaps must fail");
    }
}
