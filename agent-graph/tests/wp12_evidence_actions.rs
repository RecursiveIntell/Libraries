use agent_graph::evidence_actions::{
    ActionAuthority, ActionCalibration, EvidenceActionCandidate, EvidenceActionSelector, Freshness,
    LoopDisposition, NoProgressGuard, Reconciliation, SelectionMode, SourceCheckpoint,
    WakeDependency, WakeDispatch, WakeEvent, WakeIngest, WakeRequestKind, WakeupReconciler,
};

fn candidate(
    action_id: &str,
    obligation_ref: &str,
    mandatory: bool,
    rank_score: f64,
    authority: ActionAuthority,
) -> EvidenceActionCandidate {
    EvidenceActionCandidate::new(action_id, obligation_ref, mandatory, rank_score)
        .with_authority_grant(authority)
}

#[test]
fn eval_06_mandatory_action_survives_optimizer_ranking() {
    let selection = EvidenceActionSelector::select(
        vec![
            candidate(
                "required-check",
                "proof-obligation",
                true,
                -100.0,
                ActionAuthority::Provider,
            ),
            candidate(
                "ranked-suggestion",
                "optional-analysis",
                false,
                1_000.0,
                ActionAuthority::None,
            ),
        ],
        ActionCalibration::in_distribution(),
    );

    assert_eq!(selection.ordered_actions[0].action_id, "required-check");
    assert_eq!(
        selection.mandatory_obligation_refs,
        vec!["proof-obligation"]
    );
    assert!(selection
        .ordered_actions
        .iter()
        .any(|action| action.action_id == "required-check"));
    assert_eq!(selection.dispatchable_action_ids(), vec!["required-check"]);
    assert!(!selection
        .dispatchable_action_ids()
        .contains(&"ranked-suggestion"));
}

#[test]
fn eval_07_off_distribution_calibration_becomes_conservative_advisory_with_uncertainty() {
    let selection = EvidenceActionSelector::select(
        vec![candidate(
            "provider-proposal",
            "optional-analysis",
            false,
            0.99,
            ActionAuthority::Provider,
        )],
        ActionCalibration::off_distribution(0.73),
    );

    assert_eq!(selection.mode, SelectionMode::ConservativeAdvisory);
    assert_eq!(selection.uncertainty, Some(0.73));
    assert_eq!(selection.dispatchable_action_ids(), Vec::<&str>::new());
    assert_eq!(
        selection.ordered_actions[0].authority_grant,
        ActionAuthority::Provider
    );
}

#[test]
fn eval_08_no_progress_loop_terminates_escalates_and_preserves_gaps() {
    let mut guard = NoProgressGuard::new(3);

    assert_eq!(
        guard.observe(false, ["gap-evidence"]),
        LoopDisposition::Continue
    );
    assert_eq!(
        guard.observe(false, ["gap-verifier"]),
        LoopDisposition::Continue
    );
    assert_eq!(
        guard.observe(false, ["gap-evidence"]),
        LoopDisposition::Escalate
    );
    assert!(guard.terminated());
    assert_eq!(
        guard.unresolved_gap_refs(),
        vec!["gap-evidence", "gap-verifier"]
    );
    assert_eq!(
        guard.observe(true, std::iter::empty::<&str>()),
        LoopDisposition::Escalate
    );
}

#[test]
fn evt_01_operating_condition_version_reopens_only_dependents() {
    let mut reconciler = WakeupReconciler::new(
        vec![
            WakeDependency::new("condition-a", "obligation-a"),
            WakeDependency::new("condition-a", "obligation-b"),
            WakeDependency::new("condition-b", "obligation-c"),
        ],
        4,
    );

    let outcome = reconciler.ingest(
        WakeEvent::new("condition-a", "source-v2", "basis-v2", 2),
        WakeRequestKind::Revalidate,
    );

    assert_eq!(outcome, WakeIngest::Queued);
    let intents = reconciler.pending_intents();
    assert_eq!(intents.len(), 1);
    assert_eq!(
        intents[0].obligation_refs,
        vec!["obligation-a", "obligation-b"]
    );
    assert!(!intents[0]
        .obligation_refs
        .contains(&"obligation-c".to_owned()));
}

#[test]
fn evt_02_duplicate_or_out_of_order_event_yields_one_intent_and_stale_cannot_overwrite() {
    let mut reconciler =
        WakeupReconciler::new(vec![WakeDependency::new("condition", "obligation")], 4);
    let current = WakeEvent::new("condition", "source-v2", "basis-v2", 2);

    assert_eq!(
        reconciler.ingest(current.clone(), WakeRequestKind::Revalidate),
        WakeIngest::Queued
    );
    assert_eq!(
        reconciler.ingest(current, WakeRequestKind::Revalidate),
        WakeIngest::Duplicate
    );
    assert_eq!(
        reconciler.ingest(
            WakeEvent::new("condition", "source-v1", "basis-v1", 1),
            WakeRequestKind::Revalidate,
        ),
        WakeIngest::Stale
    );

    assert_eq!(reconciler.pending_len(), 1);
    assert_eq!(reconciler.source_basis("condition"), Some("basis-v2"));
    assert_eq!(reconciler.source_watermark("condition"), Some(2));
}

#[test]
fn evt_03_watermark_reconciliation_detects_lost_notification_before_reuse_and_unknown_blocks() {
    let mut reconciler =
        WakeupReconciler::new(vec![WakeDependency::new("condition", "obligation")], 4);
    reconciler.ingest(
        WakeEvent::new("condition", "source-v1", "basis-v1", 1),
        WakeRequestKind::Revalidate,
    );

    let outcome = reconciler.reconcile_source(
        "condition",
        Some(SourceCheckpoint::new("source-v3", "basis-v3", 3)),
    );
    assert_eq!(outcome, Reconciliation::LostNotificationDetected);
    assert!(!reconciler.reuse_allowed("condition"));
    assert_eq!(reconciler.source_watermark("condition"), Some(3));
    assert_eq!(reconciler.source_basis("condition"), Some("basis-v3"));

    assert_eq!(
        reconciler.reconcile_source("condition", None),
        Reconciliation::FreshnessUnknown
    );
    assert_eq!(reconciler.freshness("condition"), Freshness::Unknown);
    assert!(!reconciler.reuse_allowed("condition"));
}

#[test]
fn evt_04_storm_coalesces_to_bounded_latest_basis_queue_with_complete_obligation_coverage_no_polling(
) {
    let mut reconciler = WakeupReconciler::new(
        vec![
            WakeDependency::new("condition", "obligation-a"),
            WakeDependency::new("condition", "obligation-b"),
            WakeDependency::new("condition", "obligation-c"),
        ],
        2,
    );

    for watermark in 1..=100 {
        let event = WakeEvent::new(
            "condition",
            format!("source-v{watermark}"),
            format!("basis-v{watermark}"),
            watermark,
        );
        assert!(matches!(
            reconciler.ingest(event, WakeRequestKind::Revalidate),
            WakeIngest::Queued | WakeIngest::Coalesced
        ));
    }

    assert!(reconciler.pending_len() <= 2);
    let intents = reconciler.pending_intents();
    assert_eq!(intents.len(), 1);
    assert_eq!(
        intents[0]
            .basis_versions
            .get("condition")
            .map(String::as_str),
        Some("basis-v100")
    );
    assert_eq!(
        intents[0].obligation_refs,
        vec!["obligation-a", "obligation-b", "obligation-c"]
    );
    assert_eq!(reconciler.poll_count(), 0);
}

#[test]
fn evt_05_safe_mode_records_and_defers_request_with_zero_provider_or_effect_dispatch() {
    let mut reconciler =
        WakeupReconciler::new(vec![WakeDependency::new("condition", "obligation")], 4);
    reconciler.set_safe_mode(true);

    assert_eq!(
        reconciler.ingest(
            WakeEvent::new("condition", "source-v1", "basis-v1", 1),
            WakeRequestKind::Provider,
        ),
        WakeIngest::DeferredSafeMode
    );
    assert_eq!(
        reconciler.ingest(
            WakeEvent::new("condition", "source-v2", "basis-v2", 2),
            WakeRequestKind::Effect,
        ),
        WakeIngest::DeferredSafeMode
    );
    assert_eq!(reconciler.dispatch_next(), WakeDispatch::DeferredSafeMode);
    assert_eq!(reconciler.provider_dispatch_count(), 0);
    assert_eq!(reconciler.effect_dispatch_count(), 0);
    assert_eq!(reconciler.records().len(), 2);
    assert!(reconciler.records().iter().all(|record| record.deferred));
    assert!(reconciler.pending_len() > 0);
}

#[test]
fn pr11_safe_mode_exit_requires_reconciliation_then_releases_queue() {
    let mut reconciler =
        WakeupReconciler::new(vec![WakeDependency::new("condition", "obligation")], 4);
    reconciler.set_safe_mode(true);
    reconciler.ingest(
        WakeEvent::new("condition", "source-v1", "basis-v1", 1),
        WakeRequestKind::Provider,
    );
    assert_eq!(reconciler.dispatch_next(), WakeDispatch::DeferredSafeMode);
    reconciler.set_safe_mode(false);
    assert_eq!(
        reconciler.dispatch_next(),
        WakeDispatch::BlockedUnknownFreshness
    );
    reconciler.reconcile_source(
        "condition",
        Some(SourceCheckpoint {
            source_version: "source-v1".into(),
            basis_version: "basis-v1".into(),
            watermark: 1,
        }),
    );
    assert_eq!(reconciler.dispatch_next(), WakeDispatch::Dispatched);
    assert_eq!(reconciler.dispatch_next(), WakeDispatch::Empty);
    assert_eq!(reconciler.provider_dispatch_count(), 1);
}
