use profile_runtime::{
    compose_profile_runtime, diff_policy_impact, from_residency_policy_profile,
    ApplicabilityContextV1, CompiledObligationKindV1, CompositionRuleSetV1, FoldClassV1,
    ObligationContributionV1, ProfileExceptionBundleV1, ProfileRefGroupV1, ProfileSetV1,
};
use stack_ids::{ProfileExceptionBundleId, ResidencyPolicyProfileId};
use verification_policy::ResidencyPolicyProfileV1;

#[test]
fn block_dominant_path_blocks_without_exception_and_admits_with_exception() {
    let mut context = ApplicabilityContextV1::new(
        "prod/payments",
        None,
        "2026-03-16T14:00:00Z",
        "2026-03-16T14:00:05Z",
        "replay_export",
        "auditor",
        vec!["role:auditor".into()],
        None,
        None,
        Some("incident".into()),
    );

    let refs = ProfileRefGroupV1 {
        residency_policy_profile_id: Some(ResidencyPolicyProfileId::new("rpp_001")),
        ..ProfileRefGroupV1::default()
    };
    let profile_set = ProfileSetV1::new(
        context.applicability_context_id.clone(),
        refs,
        vec!["constitutional-memory:v25".into()],
    );
    let rule_set = CompositionRuleSetV1::reference_v1();

    let residency = ResidencyPolicyProfileV1 {
        schema_version: "ResidencyPolicyProfileV1".into(),
        residency_policy_profile_id: "rpp_001".into(),
        allowed_storage_regions: vec!["us-central".into()],
        allowed_execution_regions: vec!["us-central".into()],
        allowed_replay_regions: vec!["us-central".into()],
        forbidden_transfer_classes: vec!["customer_pii_to_non_us".into()],
        default_exception_path: "locality-review-board".into(),
    };

    let contributions = from_residency_policy_profile(&residency);
    let blocked = compose_profile_runtime(
        &context,
        &profile_set,
        &rule_set,
        &contributions,
        &[],
        "2026-03-16T14:00:06Z",
    )
    .expect("baseline composition");

    assert_eq!(
        blocked.effective_constitution.current_mode_classification,
        profile_runtime::ConstitutionModeV1::Blocked
    );
    assert_eq!(blocked.compiled_obligation_set.block_entries.len(), 1);
    assert!(blocked.conflict_set.is_none());

    let exception = ProfileExceptionBundleV1 {
        schema_version: "ProfileExceptionBundleV1".into(),
        profile_exception_bundle_id: ProfileExceptionBundleId::new("profile-exception-bundle-1"),
        affected_context_refs: vec![context.applicability_context_id.clone()],
        affected_profile_refs: vec!["rpp_001".into()],
        exception_class: "locality_exception".into(),
        reason: "incident-scoped replay export".into(),
        scope: "incident/replay/eu-west".into(),
        starts_at: "2026-03-16T13:30:00Z".into(),
        expires_at: "2026-03-17T13:30:00Z".into(),
        approval_refs: vec!["approval:privacy-board-1".into()],
        residual_obligations: vec!["encrypt_export".into()],
        added_monitoring_obligations: vec!["cross_region_replay_monitor".into()],
        added_post_hoc_review_obligations: vec!["locality_exception_post_review".into()],
        replay_expectations: vec!["full_replay_provenance".into()],
        cleanup_obligations: vec!["delete_temp_copy_after_review".into()],
        compensation_obligations: Vec::new(),
        recorded_at: "2026-03-16T13:31:00Z".into(),
    };
    context.admitted_exception_refs = vec![exception.profile_exception_bundle_id.clone()];

    let admitted = compose_profile_runtime(
        &context,
        &profile_set,
        &rule_set,
        &contributions,
        &[exception],
        "2026-03-16T14:00:07Z",
    )
    .expect("exception composition");

    assert_ne!(
        admitted.effective_constitution.current_mode_classification,
        profile_runtime::ConstitutionModeV1::Blocked
    );
    assert!(admitted.compiled_obligation_set.block_entries.is_empty());

    let diff = diff_policy_impact(
        &blocked.effective_constitution,
        &blocked.compiled_obligation_set,
        &admitted.effective_constitution,
        &admitted.compiled_obligation_set,
        false,
    );

    assert_eq!(diff.newly_blocked_paths, Vec::<String>::new());
    assert!(diff
        .newly_admitted_paths
        .iter()
        .any(|path| path == "residency.forbidden_transfer_classes:customer_pii_to_non_us"));
}

#[test]
fn conflict_if_different_emits_conflict_set() {
    let context = ApplicabilityContextV1::new(
        "prod/payments",
        None,
        "2026-03-16T15:00:00Z",
        "2026-03-16T15:00:02Z",
        "audit_export",
        "compliance_auditor",
        vec!["role:compliance-auditor".into()],
        None,
        None,
        Some("normal".into()),
    );
    let profile_set = ProfileSetV1::new(
        context.applicability_context_id.clone(),
        ProfileRefGroupV1::default(),
        vec!["constitutional-memory:v25".into()],
    );
    let rule_set = CompositionRuleSetV1::reference_v1();

    let contributions = vec![
        ObligationContributionV1::simple(
            "disclosure.export_format",
            "export_package_format",
            CompiledObligationKindV1::Disclosure,
            FoldClassV1::ConflictIfDifferent,
            vec!["bundle_v2".into()],
            "aep_001",
            "audit extraction format",
        ),
        ObligationContributionV1::simple(
            "disclosure.export_format",
            "export_package_format",
            CompiledObligationKindV1::Disclosure,
            FoldClassV1::ConflictIfDifferent,
            vec!["vendor_portal_zip".into()],
            "vet_001",
            "vendor translation format",
        ),
    ];

    let outcome = compose_profile_runtime(
        &context,
        &profile_set,
        &rule_set,
        &contributions,
        &[],
        "2026-03-16T15:00:03Z",
    )
    .expect("conflict composition");

    let conflict_set = outcome.conflict_set.expect("conflict set emitted");
    assert!(conflict_set.blocking);
    assert_eq!(conflict_set.conflicts.len(), 1);
    assert_eq!(
        conflict_set.conflicts[0].conflict_class,
        "incomparable_values"
    );
}
