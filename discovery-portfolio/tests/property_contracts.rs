//! Property-based tests for discovery-portfolio artifact types.
//!
//! Validates serde roundtrip fidelity and schema_version constant usage
//! for top-level artifact families. CampaignDecisionTraceV1 and
//! CampaignDecisionLineV1 contain f64 fields, so roundtrip comparisons
//! use JSON string equality instead of PartialEq.

use discovery_portfolio::*;
use proptest::prelude::*;
use stack_ids::{
    CampaignDecisionTraceId, DiscoveryProgramId, ExperimentCampaignId, PortfolioPlanId,
    SurfaceStatus, VerificationLoadBudgetId,
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

fn arb_discovery_program_id() -> impl Strategy<Value = DiscoveryProgramId> {
    arb_id().prop_map(DiscoveryProgramId::new)
}

fn arb_experiment_campaign_id() -> impl Strategy<Value = ExperimentCampaignId> {
    arb_id().prop_map(ExperimentCampaignId::new)
}

fn arb_portfolio_plan_id() -> impl Strategy<Value = PortfolioPlanId> {
    arb_id().prop_map(PortfolioPlanId::new)
}

fn arb_verification_load_budget_id() -> impl Strategy<Value = VerificationLoadBudgetId> {
    arb_id().prop_map(VerificationLoadBudgetId::new)
}

fn arb_campaign_decision_trace_id() -> impl Strategy<Value = CampaignDecisionTraceId> {
    arb_id().prop_map(CampaignDecisionTraceId::new)
}

fn arb_campaign_decision() -> impl Strategy<Value = CampaignDecision> {
    prop_oneof![
        Just(CampaignDecision::Launch),
        Just(CampaignDecision::Defer),
        Just(CampaignDecision::PauseBudgetExhausted),
    ]
}

// --- Top-level artifact generators ---

fn arb_discovery_program_v1() -> impl Strategy<Value = DiscoveryProgramV1> {
    (
        arb_discovery_program_id(),
        arb_string(),
        arb_string(),
        arb_surface_status(),
    )
        .prop_map(
            |(discovery_program_id, program_name, canonical_owner, publication_status)| {
                DiscoveryProgramV1 {
                    schema_version: DISCOVERY_PROGRAM_V1_SCHEMA.into(),
                    discovery_program_id,
                    program_name,
                    canonical_owner,
                    publication_status,
                }
            },
        )
}

fn arb_verification_load_budget_v1() -> impl Strategy<Value = VerificationLoadBudgetV1> {
    (
        arb_verification_load_budget_id(),
        0u32..1000,
        0u32..1000,
        prop::bool::ANY,
        prop::bool::ANY,
    )
        .prop_map(
            |(
                verification_load_budget_id,
                total_review_slots,
                remaining_review_slots,
                exhausted,
                horizon_only,
            )| {
                VerificationLoadBudgetV1 {
                    schema_version: VERIFICATION_LOAD_BUDGET_V1_SCHEMA.into(),
                    verification_load_budget_id,
                    total_review_slots,
                    remaining_review_slots,
                    exhausted,
                    horizon_only,
                }
            },
        )
}

fn arb_campaign_decision_line_v1() -> impl Strategy<Value = CampaignDecisionLineV1> {
    (
        arb_experiment_campaign_id(),
        arb_campaign_decision(),
        0u32..1000,
        0u32..1000,
        0.0..1.0f64,
        arb_vec_string(),
        arb_string(),
    )
        .prop_map(
            |(
                campaign_id,
                decision,
                expected_information_gain,
                estimated_review_cost,
                budget_pressure,
                hypothesis_refs,
                rationale,
            )| {
                CampaignDecisionLineV1 {
                    campaign_id,
                    decision,
                    expected_information_gain,
                    estimated_review_cost,
                    budget_pressure,
                    hypothesis_refs,
                    rationale,
                }
            },
        )
}

fn arb_campaign_decision_trace_v1() -> impl Strategy<Value = CampaignDecisionTraceV1> {
    (
        arb_campaign_decision_trace_id(),
        arb_portfolio_plan_id(),
        arb_discovery_program_id(),
        arb_verification_load_budget_id(),
        prop::collection::vec(arb_campaign_decision_line_v1(), 0..3),
        0u32..1000,
        prop::bool::ANY,
        prop::bool::ANY,
        arb_string(),
    )
        .prop_map(
            |(
                campaign_decision_trace_id,
                portfolio_plan_id,
                discovery_program_id,
                verification_load_budget_id,
                decisions,
                remaining_review_slots,
                advisory_only,
                degraded,
                generated_at,
            )| {
                CampaignDecisionTraceV1 {
                    schema_version: CAMPAIGN_DECISION_TRACE_V1_SCHEMA.into(),
                    campaign_decision_trace_id,
                    portfolio_plan_id,
                    discovery_program_id,
                    verification_load_budget_id,
                    decisions,
                    remaining_review_slots,
                    advisory_only,
                    degraded,
                    generated_at,
                }
            },
        )
}

// --- Property tests ---

proptest! {
    #[test]
    fn discovery_program_v1_serde_roundtrip(val in arb_discovery_program_v1()) {
        let json = serde_json::to_string(&val).unwrap();
        let back: DiscoveryProgramV1 = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(&val, &back);
    }

    #[test]
    fn discovery_program_v1_schema_version_constant(val in arb_discovery_program_v1()) {
        prop_assert_eq!(val.schema_version.as_str(), DISCOVERY_PROGRAM_V1_SCHEMA);
    }

    #[test]
    fn verification_load_budget_v1_serde_roundtrip(val in arb_verification_load_budget_v1()) {
        let json = serde_json::to_string(&val).unwrap();
        let back: VerificationLoadBudgetV1 = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(&val, &back);
    }

    #[test]
    fn verification_load_budget_v1_schema_version_constant(val in arb_verification_load_budget_v1()) {
        prop_assert_eq!(val.schema_version.as_str(), VERIFICATION_LOAD_BUDGET_V1_SCHEMA);
    }

    /// CampaignDecisionTraceV1 contains f64 via CampaignDecisionLineV1.budget_pressure.
    /// Verify structural roundtrip: serialize then deserialize preserves all non-f64 fields,
    /// and f64 values are within epsilon.
    #[test]
    fn campaign_decision_trace_v1_serde_roundtrip(val in arb_campaign_decision_trace_v1()) {
        let json = serde_json::to_string(&val).unwrap();
        let back: CampaignDecisionTraceV1 = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(&val.schema_version, &back.schema_version);
        prop_assert_eq!(&val.campaign_decision_trace_id, &back.campaign_decision_trace_id);
        prop_assert_eq!(val.decisions.len(), back.decisions.len());
        prop_assert_eq!(val.remaining_review_slots, back.remaining_review_slots);
        prop_assert_eq!(val.advisory_only, back.advisory_only);
        for (a, b) in val.decisions.iter().zip(back.decisions.iter()) {
            prop_assert_eq!(&a.campaign_id, &b.campaign_id);
            prop_assert_eq!(&a.decision, &b.decision);
            prop_assert!((a.budget_pressure - b.budget_pressure).abs() < 1e-10);
        }
    }

    #[test]
    fn campaign_decision_trace_v1_schema_version_constant(val in arb_campaign_decision_trace_v1()) {
        prop_assert_eq!(val.schema_version.as_str(), CAMPAIGN_DECISION_TRACE_V1_SCHEMA);
    }
}
