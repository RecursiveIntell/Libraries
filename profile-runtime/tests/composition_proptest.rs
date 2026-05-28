#![allow(clippy::expect_used)]

//! LIB-LOW-003: Proptest coverage for profile composition fold class invariants.

use profile_runtime::{
    compose_profile_runtime, ApplicabilityContextV1, CompiledObligationKindV1,
    CompositionRuleSetV1, FoldClassV1, ObligationContributionV1, ProfileRefGroupV1, ProfileSetV1,
};
use proptest::prelude::*;

fn test_context() -> ApplicabilityContextV1 {
    ApplicabilityContextV1::new(
        "test-ns",
        None,
        "2026-01-01T00:00:00Z",
        "2026-01-01T00:00:00Z",
        "test",
        "test",
        vec!["role:test".into()],
        None,
        None,
        None,
    )
}

fn test_profile_set(context: &ApplicabilityContextV1) -> ProfileSetV1 {
    ProfileSetV1::new(
        context.applicability_context_id.clone(),
        ProfileRefGroupV1::default(),
        vec!["test-source:v1".into()],
    )
}

fn make_contribution(
    fold_class: FoldClassV1,
    string_values: Vec<String>,
    numeric_value: Option<i64>,
    source: &str,
) -> ObligationContributionV1 {
    ObligationContributionV1 {
        obligation_family: "test.family".into(),
        obligation_key: "test_key".into(),
        output_kind: CompiledObligationKindV1::Check,
        fold_class,
        string_values,
        numeric_value,
        expiry_at: None,
        blocking: false,
        source_profile_ref: source.into(),
        admissible_exception_classes: Vec::new(),
        explanation: "proptest contribution".into(),
    }
}

proptest! {
    #[test]
    fn union_fold_contains_all_inputs(
        values in prop::collection::vec("[a-z]{3,8}", 1..5)
    ) {
        let context = test_context();
        let profile_set = test_profile_set(&context);
        let rule_set = CompositionRuleSetV1::reference_v1();

        let contributions: Vec<_> = values
            .iter()
            .enumerate()
            .map(|(i, v)| make_contribution(
                FoldClassV1::Union,
                vec![v.clone()],
                None,
                &format!("source-{i}"),
            ))
            .collect();

        let outcome = compose_profile_runtime(
            &context,
            &profile_set,
            &rule_set,
            &contributions,
            &[],
            "2026-01-01T00:00:01Z",
        )
        .expect("union composition");

        // Union fold: every input value must appear in some entry's string_values
        let all_compiled_values: Vec<String> = outcome
            .compiled_obligation_set
            .obligation_entries
            .iter()
            .flat_map(|e| e.string_values.iter().cloned())
            .collect();
        for v in &values {
            prop_assert!(
                all_compiled_values.contains(v),
                "Union result missing value '{}'; compiled: {:?}",
                v,
                all_compiled_values
            );
        }
    }

    #[test]
    fn intersection_fold_is_subset_of_every_input(
        base in prop::collection::vec("[a-z]{3,8}", 2..5),
        extra_idx in 0..4usize
    ) {
        let context = test_context();
        let profile_set = test_profile_set(&context);
        let rule_set = CompositionRuleSetV1::reference_v1();

        // Ensure at least one common element
        let common = base[0].clone();
        let mut contributions = vec![
            make_contribution(
                FoldClassV1::Intersection,
                base.clone(),
                None,
                "source-0",
            ),
        ];
        // Second contribution is a subset of base (common + maybe one more)
        let mut second_values = vec![common.clone()];
        if extra_idx < base.len() {
            second_values.push(base[extra_idx].clone());
        }
        second_values.sort();
        second_values.dedup();
        contributions.push(make_contribution(
            FoldClassV1::Intersection,
            second_values.clone(),
            None,
            "source-1",
        ));

        let outcome = compose_profile_runtime(
            &context,
            &profile_set,
            &rule_set,
            &contributions,
            &[],
            "2026-01-01T00:00:01Z",
        )
        .expect("intersection composition");

        let compiled_values: Vec<String> = outcome
            .compiled_obligation_set
            .obligation_entries
            .iter()
            .flat_map(|e| e.string_values.iter().cloned())
            .collect();

        // Intersection: every compiled value must be in every contribution's string_values
        let base_set: std::collections::BTreeSet<_> = base.iter().cloned().collect();
        let second_set: std::collections::BTreeSet<_> = second_values.iter().cloned().collect();
        for v in &compiled_values {
            prop_assert!(
                base_set.contains(v) && second_set.contains(v),
                "Intersection result '{}' not in all inputs",
                v
            );
        }
    }

    #[test]
    fn min_of_maxima_is_at_most_every_input(
        values in prop::collection::vec(1i64..1000, 2..5)
    ) {
        let context = test_context();
        let profile_set = test_profile_set(&context);
        let rule_set = CompositionRuleSetV1::reference_v1();

        let contributions: Vec<_> = values
            .iter()
            .enumerate()
            .map(|(i, &v)| make_contribution(
                FoldClassV1::MinOfMaxima,
                Vec::new(),
                Some(v),
                &format!("source-{i}"),
            ))
            .collect();

        let outcome = compose_profile_runtime(
            &context,
            &profile_set,
            &rule_set,
            &contributions,
            &[],
            "2026-01-01T00:00:01Z",
        )
        .expect("min_of_maxima composition");

        for entry in &outcome.compiled_obligation_set.obligation_entries {
            if let Some(compiled_numeric) = entry.numeric_value {
                for &v in &values {
                    prop_assert!(
                        compiled_numeric <= v,
                        "MinOfMaxima {} > input {}",
                        compiled_numeric,
                        v
                    );
                }
            }
        }
    }

    #[test]
    fn max_of_minima_is_at_least_every_input(
        values in prop::collection::vec(1i64..1000, 2..5)
    ) {
        let context = test_context();
        let profile_set = test_profile_set(&context);
        let rule_set = CompositionRuleSetV1::reference_v1();

        let contributions: Vec<_> = values
            .iter()
            .enumerate()
            .map(|(i, &v)| make_contribution(
                FoldClassV1::MaxOfMinima,
                Vec::new(),
                Some(v),
                &format!("source-{i}"),
            ))
            .collect();

        let outcome = compose_profile_runtime(
            &context,
            &profile_set,
            &rule_set,
            &contributions,
            &[],
            "2026-01-01T00:00:01Z",
        )
        .expect("max_of_minima composition");

        for entry in &outcome.compiled_obligation_set.obligation_entries {
            if let Some(compiled_numeric) = entry.numeric_value {
                for &v in &values {
                    prop_assert!(
                        compiled_numeric >= v,
                        "MaxOfMinima {} < input {}",
                        compiled_numeric,
                        v
                    );
                }
            }
        }
    }
}
