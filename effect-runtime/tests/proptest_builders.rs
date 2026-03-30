use effect_runtime::error::EffectRuntimeValidationError;
use effect_runtime::vocab::*;
use effect_runtime::*;
use proptest::prelude::*;
use stack_ids::{EffectIntentId, EffectWindowId};

fn arb_commit_atomicity() -> impl Strategy<Value = CommitAtomicityV1> {
    prop_oneof![
        Just(CommitAtomicityV1::BoundedMultiStep),
        Just(CommitAtomicityV1::ExactlyOnce),
        Just(CommitAtomicityV1::AtLeastOnce),
        Just(CommitAtomicityV1::BestEffort),
    ]
}

fn arb_retry_owner() -> impl Strategy<Value = RetryOwnerV1> {
    prop_oneof![
        Just(RetryOwnerV1::EffectRuntime),
        Just(RetryOwnerV1::ForgeOrchestration),
        Just(RetryOwnerV1::Verification),
        Just(RetryOwnerV1::Provider),
        Just(RetryOwnerV1::Operator),
    ]
}

fn arb_close_midflight() -> impl Strategy<Value = CloseMidflightBehaviorV1> {
    prop_oneof![
        Just(CloseMidflightBehaviorV1::AbortAndEmitReceipt),
        Just(CloseMidflightBehaviorV1::FailClosed),
        Just(CloseMidflightBehaviorV1::ContinueObservation),
        Just(CloseMidflightBehaviorV1::BestEffortCompensation),
    ]
}

fn arb_effect_class() -> impl Strategy<Value = EffectClassV1> {
    prop_oneof![
        Just(EffectClassV1::ExternalWrite),
        Just(EffectClassV1::ExternalRead),
        Just(EffectClassV1::ExternalDelete),
        Just(EffectClassV1::StateTransition),
        Just(EffectClassV1::Compensation),
    ]
}

fn arb_blast_radius() -> impl Strategy<Value = BlastRadiusCeilingV1> {
    prop_oneof![
        Just(BlastRadiusCeilingV1::SingleTenant),
        Just(BlastRadiusCeilingV1::SingleService),
        Just(BlastRadiusCeilingV1::Regional),
        Just(BlastRadiusCeilingV1::Global),
    ]
}

fn arb_reversibility() -> impl Strategy<Value = ReversibilityClassV1> {
    prop_oneof![
        Just(ReversibilityClassV1::Compensatable),
        Just(ReversibilityClassV1::Replayable),
        Just(ReversibilityClassV1::Irreversible),
    ]
}

fn arb_run_mode() -> impl Strategy<Value = RunModeV1> {
    prop_oneof![
        Just(RunModeV1::Simulated),
        Just(RunModeV1::DryRun),
        Just(RunModeV1::Replay),
    ]
}

fn arb_publication_status() -> impl Strategy<Value = PublicationStatusV1> {
    prop_oneof![
        Just(PublicationStatusV1::Required),
        Just(PublicationStatusV1::AdvisoryOnly),
        Just(PublicationStatusV1::Deferred),
        Just(PublicationStatusV1::Suppressed),
    ]
}

fn arb_rfc3339_timestamp() -> impl Strategy<Value = String> {
    (2020u32..2030, 1u32..13, 1u32..29, 0u32..24, 0u32..60, 0u32..60).prop_map(
        |(y, m, d, h, min, s)| {
            format!("{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z", y, m, d, h, min, s)
        },
    )
}

proptest! {
    #[test]
    fn effect_window_builder_validate_roundtrip(
        commit_atomicity in arb_commit_atomicity(),
        retry_owner in arb_retry_owner(),
        close_midflight in arb_close_midflight(),
        earliest in arb_rfc3339_timestamp(),
        latest in arb_rfc3339_timestamp(),
        timeout in arb_rfc3339_timestamp(),
    ) {
        let window_id = EffectWindowId::new("ew-proptest");
        let result = EffectWindowV1::builder(
            window_id,
            &earliest,
            &latest,
            &timeout,
            commit_atomicity,
            retry_owner,
            close_midflight,
        )
        .build();
        prop_assert!(result.is_ok(), "valid timestamps should build: {:?}", result.err());

        let window = result.unwrap();
        let json = serde_json::to_string(&window).unwrap();
        let roundtrip: EffectWindowV1 = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(roundtrip.earliest_start, window.earliest_start);
        prop_assert_eq!(roundtrip.timeout_at, window.timeout_at);
    }

    #[test]
    fn effect_window_rejects_invalid_timestamp(
        bad_stamp in "[a-z]{5,15}",
    ) {
        let window_id = EffectWindowId::new("ew-bad");
        let result = EffectWindowV1::builder(
            window_id,
            &bad_stamp,
            "2025-01-01T00:00:00Z",
            "2025-01-01T00:00:00Z",
            CommitAtomicityV1::ExactlyOnce,
            RetryOwnerV1::EffectRuntime,
            CloseMidflightBehaviorV1::FailClosed,
        )
        .build();
        prop_assert!(result.is_err());
        if let Err(EffectRuntimeValidationError::InvalidTimestamp { field, .. }) = &result {
            prop_assert_eq!(*field, "earliest_start");
        } else {
            prop_assert!(false, "expected InvalidTimestamp, got {:?}", result);
        }
    }

    #[test]
    fn effect_intent_builder_validate_roundtrip(
        effect_class in arb_effect_class(),
        blast_radius in arb_blast_radius(),
        reversibility in arb_reversibility(),
        run_mode in arb_run_mode(),
        publication_status in arb_publication_status(),
    ) {
        let intent_id = EffectIntentId::new("ei-proptest");
        let window_id = EffectWindowId::new("ew-proptest");
        let result = EffectIntentV1::builder(
            intent_id,
            effect_class,
            "target_surface_test",
            "intended_outcome_test",
            vec!["artifact-ref-1".into()],
            "capability_class_test",
            "scope_test",
            blast_radius,
            reversibility,
            "idempotency_key_test",
            window_id,
            vec!["obs-ref-1".into()],
            vec!["approval-ref-1".into()],
            run_mode,
            publication_status,
        )
        .build();
        prop_assert!(result.is_ok(), "valid intent should build: {:?}", result.err());

        let intent = result.unwrap();
        let json = serde_json::to_string(&intent).unwrap();
        let roundtrip: EffectIntentV1 = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(roundtrip.effect_class, intent.effect_class);
        prop_assert_eq!(roundtrip.blast_radius_ceiling, intent.blast_radius_ceiling);
        prop_assert_eq!(roundtrip.reversibility_class, intent.reversibility_class);
    }
}
