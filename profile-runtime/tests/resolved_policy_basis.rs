#![allow(clippy::expect_used)]

use profile_runtime::{
    compose_profile_runtime, resolve_policy_basis, ApplicabilityContextV1,
    CompiledObligationKindV1, CompositionRuleSetV1, FoldClassV1, ObligationContributionV1,
    PolicyBasisStatusV1, PolicyBasisTaskContextV1, ProfileRefGroupV1, ProfileSetV1,
};
use stack_ids::ResidencyPolicyProfileId;

fn context() -> ApplicabilityContextV1 {
    ApplicabilityContextV1::new(
        "mission-space",
        None,
        "2026-09-05T20:00:00Z",
        "2026-09-05T20:00:01Z",
        "sealed_completion",
        "principal",
        vec!["role:principal".into()],
        None,
        None,
        Some("normal".into()),
    )
}

fn profile_set(context: &ApplicabilityContextV1) -> ProfileSetV1 {
    ProfileSetV1::new(
        context.applicability_context_id.clone(),
        ProfileRefGroupV1 {
            residency_policy_profile_id: Some(ResidencyPolicyProfileId::new("rpp_policy")),
            ..ProfileRefGroupV1::default()
        },
        vec![
            "authority:policy-owner:7".into(),
            "source:profile-runtime:5".into(),
        ],
    )
}

fn strings(
    family: &str,
    key: &str,
    kind: CompiledObligationKindV1,
    fold: FoldClassV1,
    values: &[&str],
) -> ObligationContributionV1 {
    ObligationContributionV1 {
        obligation_family: family.into(),
        obligation_key: key.into(),
        output_kind: kind,
        fold_class: fold,
        string_values: values.iter().map(|value| (*value).into()).collect(),
        numeric_value: None,
        expiry_at: None,
        blocking: false,
        source_profile_ref: "rpp_policy".into(),
        admissible_exception_classes: Vec::new(),
        explanation: format!("fixture {family}"),
    }
}

fn number(family: &str, key: &str, value: i64) -> ObligationContributionV1 {
    ObligationContributionV1 {
        obligation_family: family.into(),
        obligation_key: key.into(),
        output_kind: CompiledObligationKindV1::Effect,
        fold_class: FoldClassV1::MinOfMaxima,
        string_values: Vec::new(),
        numeric_value: Some(value),
        expiry_at: None,
        blocking: false,
        source_profile_ref: "rpp_policy".into(),
        admissible_exception_classes: Vec::new(),
        explanation: format!("fixture {family}"),
    }
}

fn contributions() -> Vec<ObligationContributionV1> {
    vec![
        strings(
            "egress.allowed_route_classes",
            "route",
            CompiledObligationKindV1::Effect,
            FoldClassV1::Intersection,
            &["local", "managed_cloud"],
        ),
        strings(
            "egress.allowed_route_classes",
            "route",
            CompiledObligationKindV1::Effect,
            FoldClassV1::Intersection,
            &["managed_cloud"],
        ),
        strings(
            "disclosure.allowed_classes",
            "classification",
            CompiledObligationKindV1::Disclosure,
            FoldClassV1::Intersection,
            &["public", "private"],
        ),
        strings(
            "disclosure.allowed_classes",
            "classification",
            CompiledObligationKindV1::Disclosure,
            FoldClassV1::Intersection,
            &["public"],
        ),
        strings(
            "effect.allowed_classes",
            "effect",
            CompiledObligationKindV1::Effect,
            FoldClassV1::Intersection,
            &["sealed_completion"],
        ),
        strings(
            "effect.required_preflight_checks",
            "checks",
            CompiledObligationKindV1::Check,
            FoldClassV1::Union,
            &["policy_current", "graph_obligation_current"],
        ),
        number("budget.max_input_tokens", "input", 4096),
        number("budget.max_input_tokens", "input", 2048),
        number("budget.max_output_tokens", "output", 512),
        number("budget.max_attempts", "attempts", 2),
        number("budget.max_concurrency", "concurrency", 1),
        number("budget.max_wall_time_ms", "wall_time", 30_000),
        number("budget.max_artifact_bytes", "artifact_bytes", 65_536),
        ObligationContributionV1 {
            obligation_family: "policy.not_after".into(),
            obligation_key: "expiry".into(),
            output_kind: CompiledObligationKindV1::Continuity,
            fold_class: FoldClassV1::EarliestExpiry,
            string_values: Vec::new(),
            numeric_value: None,
            expiry_at: Some("2026-09-05T21:00:00Z".into()),
            blocking: false,
            source_profile_ref: "rpp_policy".into(),
            admissible_exception_classes: Vec::new(),
            explanation: "fixture expiry".into(),
        },
    ]
}

fn task_context() -> PolicyBasisTaskContextV1 {
    PolicyBasisTaskContextV1 {
        mission_ref: "mission:1".into(),
        task_ref: "task:1".into(),
        instruction_ref: "instruction:7".into(),
        instruction_digest:
            "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".into(),
        source_revision: "source:7".into(),
        authority_snapshot_ref: "authority:snapshot:7".into(),
        unresolved_instruction_obligations: vec!["operator_clause:preserve_live_state".into()],
    }
}

#[test]
fn resolved_basis_is_a_digest_bound_projection_of_exact_owner_outputs() {
    let context = context();
    let profiles = profile_set(&context);
    let rules = CompositionRuleSetV1::reference_v1();
    let outcome = compose_profile_runtime(
        &context,
        &profiles,
        &rules,
        &contributions(),
        &[],
        "2026-09-05T20:00:02Z",
    )
    .expect("owner composition");

    let basis = resolve_policy_basis(&context, &profiles, &rules, &outcome, task_context())
        .expect("policy-basis projection");
    basis.validate().expect("basis digest and owner refs");

    assert_eq!(basis.status, PolicyBasisStatusV1::Admitted);
    assert_eq!(basis.allowed_route_classes, vec!["managed_cloud"]);
    assert_eq!(basis.allowed_disclosure_classes, vec!["public"]);
    assert_eq!(basis.allowed_effect_classes, vec!["sealed_completion"]);
    assert_eq!(basis.limits.max_input_tokens, 2048);
    assert_eq!(basis.limits.max_output_tokens, 512);
    assert_eq!(basis.not_before, context.valid_as_of);
    assert_eq!(basis.not_after, "2026-09-05T21:00:00Z");
    assert!(basis
        .mandatory_obligations
        .contains(&"operator_clause:preserve_live_state".into()));
    assert_eq!(
        basis.owner_refs.composition_receipt_ref,
        outcome.receipt.composition_receipt_id
    );
    assert!(!basis.basis_digest.to_string().is_empty());

    let round_trip: profile_runtime::ResolvedPolicyBasisV1 =
        serde_json::from_slice(&serde_json::to_vec(&basis).expect("serialize"))
            .expect("deserialize");
    assert_eq!(round_trip, basis);
    round_trip.validate().expect("round-trip validation");
}

#[test]
fn projection_preserves_blocks_and_rejects_cross_owner_reference_drift() {
    let context = context();
    let profiles = profile_set(&context);
    let rules = CompositionRuleSetV1::reference_v1();
    let mut outcome = compose_profile_runtime(
        &context,
        &profiles,
        &rules,
        &contributions(),
        &[],
        "2026-09-05T20:00:02Z",
    )
    .expect("owner composition");
    outcome
        .compiled_obligation_set
        .block_entries
        .push(profile_runtime::BlockEntryV1 {
            block_key: "operator:unresolved".into(),
            reason: "current instruction remains unresolved".into(),
            source_profile_refs: vec!["rpp_policy".into()],
            admissible_exception_classes: Vec::new(),
        });

    let blocked = resolve_policy_basis(&context, &profiles, &rules, &outcome, task_context())
        .expect("blocked policy remains evidence-bearing");
    assert_eq!(blocked.status, PolicyBasisStatusV1::Blocked);
    assert!(blocked
        .blocking_reasons
        .contains(&"operator:unresolved".into()));

    let wrong_profiles = profile_set(&context);
    let error = resolve_policy_basis(&context, &wrong_profiles, &rules, &outcome, task_context())
        .expect_err("different profile-set identity must fail");
    assert_eq!(error.kind(), "owner_reference_mismatch");
}

#[test]
fn pr11_policy_rejects_expired_malformed_and_inverted_windows() {
    for expiry in ["2026-09-05T19:59:59Z", "garbageZ", "2026-02-30T20:00:00Z"] {
        let context = context();
        let profiles = profile_set(&context);
        let rules = CompositionRuleSetV1::reference_v1();
        let mut entries = contributions();
        entries.last_mut().unwrap().expiry_at = Some(expiry.into());
        let outcome = compose_profile_runtime(
            &context,
            &profiles,
            &rules,
            &entries,
            &[],
            "2026-09-05T20:00:02Z",
        )
        .unwrap();
        assert!(
            resolve_policy_basis(&context, &profiles, &rules, &outcome, task_context()).is_err()
        );
    }
    for expiry in ["2026-09-05T20:00:00Z", "2026-09-05T16:00:00-04:00"] {
        let context = context();
        let profiles = profile_set(&context);
        let rules = CompositionRuleSetV1::reference_v1();
        let mut entries = contributions();
        entries.last_mut().unwrap().expiry_at = Some(expiry.into());
        let outcome = compose_profile_runtime(
            &context,
            &profiles,
            &rules,
            &entries,
            &[],
            "2026-09-05T20:00:02Z",
        )
        .unwrap();
        let mut basis =
            resolve_policy_basis(&context, &profiles, &rules, &outcome, task_context()).unwrap();
        assert_eq!(basis.status, PolicyBasisStatusV1::Admitted);
        basis.not_after = "2026-09-05T19:00:00Z".into();
        assert!(basis.validate().is_err());
    }
}

#[test]
fn pr12_expiry_fold_uses_instants_and_keeps_malformed_constraints_visible() {
    for expiry in ["2026-09-05T20:00:00-04:00", "invalidZ"] {
        let context = context();
        let profiles = profile_set(&context);
        let rules = CompositionRuleSetV1::reference_v1();
        let mut entries = contributions();
        let mut extra = entries.last().unwrap().clone();
        extra.expiry_at = Some(expiry.into());
        entries.push(extra);
        for _ in 0..2 {
            let outcome = compose_profile_runtime(
                &context,
                &profiles,
                &rules,
                &entries,
                &[],
                "2026-09-05T20:00:02Z",
            )
            .unwrap();
            let basis = resolve_policy_basis(&context, &profiles, &rules, &outcome, task_context());
            if expiry == "invalidZ" {
                assert!(basis.is_err());
            } else {
                assert_eq!(basis.unwrap().not_after, "2026-09-05T21:00:00Z");
            }
            entries.reverse();
        }
    }
}
