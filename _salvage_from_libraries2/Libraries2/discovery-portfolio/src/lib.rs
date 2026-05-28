//! Typed discovery portfolio surface crate for program, campaign, and budget artifacts.
//!
//! The crate keeps the compatibility name, but it only publishes typed
//! portfolio surfaces plus bounded prioritization helpers. It does not
//! implement a general experiment scheduler or promotion runtime.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use stack_ids::{
    CampaignDecisionTraceId, DiscoveryProgramId, ExperimentCampaignId, InformationValueEstimateId,
    PortfolioPlanId, SurfaceStatus, VerificationLoadBudgetId,
};
use std::cmp::Ordering;

pub const DISCOVERY_PROGRAM_V1_SCHEMA: &str = "discovery_program_v1";
pub const PORTFOLIO_PLAN_V1_SCHEMA: &str = "portfolio_plan_v1";
pub const EXPERIMENT_CAMPAIGN_V1_SCHEMA: &str = "experiment_campaign_v1";
pub const CAMPAIGN_DECISION_TRACE_V1_SCHEMA: &str = "campaign_decision_trace_v1";
pub const INFORMATION_VALUE_ESTIMATE_V1_SCHEMA: &str = "information_value_estimate_v1";
pub const VERIFICATION_LOAD_BUDGET_V1_SCHEMA: &str = "verification_load_budget_v1";
pub const PROGRAM_HYPOTHESIS_SET_V1_SCHEMA: &str = "program_hypothesis_set_v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct DiscoveryProgramV1 {
    pub schema_version: String,
    pub discovery_program_id: DiscoveryProgramId,
    pub program_name: String,
    pub canonical_owner: String,
    pub publication_status: SurfaceStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ProgramHypothesisSetV1 {
    pub schema_version: String,
    pub discovery_program_id: DiscoveryProgramId,
    #[serde(default)]
    pub hypothesis_refs: Vec<String>,
    pub horizon_only: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct InformationValueEstimateV1 {
    pub schema_version: String,
    pub information_value_estimate_id: InformationValueEstimateId,
    pub campaign_id: ExperimentCampaignId,
    pub expected_information_gain: u32,
    pub estimated_review_cost: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ExperimentCampaignV1 {
    pub schema_version: String,
    pub experiment_campaign_id: ExperimentCampaignId,
    pub campaign_name: String,
    pub utility_case: String,
    pub information_value_estimate_id: InformationValueEstimateId,
    pub required_review_slots: u32,
    pub advisory_only: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct PortfolioPlanV1 {
    pub schema_version: String,
    pub portfolio_plan_id: PortfolioPlanId,
    pub discovery_program_id: DiscoveryProgramId,
    #[serde(default)]
    pub campaign_ids: Vec<ExperimentCampaignId>,
    pub utility_rationale: String,
    pub review_load_rationale: String,
    pub advisory_only: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct VerificationLoadBudgetV1 {
    pub schema_version: String,
    pub verification_load_budget_id: VerificationLoadBudgetId,
    pub total_review_slots: u32,
    pub remaining_review_slots: u32,
    pub exhausted: bool,
    pub horizon_only: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum CampaignDecision {
    Launch,
    Defer,
    PauseBudgetExhausted,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct CampaignDecisionLineV1 {
    pub campaign_id: ExperimentCampaignId,
    pub decision: CampaignDecision,
    pub expected_information_gain: u32,
    pub estimated_review_cost: u32,
    pub budget_pressure: f64,
    #[serde(default)]
    pub hypothesis_refs: Vec<String>,
    pub rationale: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct CampaignDecisionTraceV1 {
    pub schema_version: String,
    pub campaign_decision_trace_id: CampaignDecisionTraceId,
    pub portfolio_plan_id: PortfolioPlanId,
    pub discovery_program_id: DiscoveryProgramId,
    pub verification_load_budget_id: VerificationLoadBudgetId,
    #[serde(default)]
    pub decisions: Vec<CampaignDecisionLineV1>,
    pub remaining_review_slots: u32,
    pub advisory_only: bool,
    pub degraded: bool,
    pub generated_at: String,
}

pub fn evaluate_portfolio_plan(
    program: &DiscoveryProgramV1,
    hypotheses: &ProgramHypothesisSetV1,
    plan: &PortfolioPlanV1,
    campaigns: &[ExperimentCampaignV1],
    value_estimates: &[InformationValueEstimateV1],
    budget: &VerificationLoadBudgetV1,
    generated_at: impl Into<String>,
) -> CampaignDecisionTraceV1 {
    let mut remaining = budget.remaining_review_slots;
    let mut degraded = false;
    let mut decisions = Vec::with_capacity(campaigns.len());
    let mut candidate_campaigns = campaigns
        .iter()
        .filter(|campaign| {
            plan.campaign_ids.is_empty()
                || plan.campaign_ids.contains(&campaign.experiment_campaign_id)
        })
        .collect::<Vec<_>>();

    let plan_order = plan
        .campaign_ids
        .iter()
        .enumerate()
        .map(|(index, id)| (id.as_str().to_string(), index))
        .collect::<std::collections::HashMap<_, _>>();
    let estimate_by_campaign = value_estimates
        .iter()
        .map(|estimate| (estimate.campaign_id.as_str().to_string(), estimate))
        .collect::<std::collections::HashMap<_, _>>();

    candidate_campaigns.sort_by(|left, right| {
        let left_estimate = estimate_by_campaign.get(left.experiment_campaign_id.as_str());
        let right_estimate = estimate_by_campaign.get(right.experiment_campaign_id.as_str());

        match (left_estimate, right_estimate) {
            (Some(left_estimate), Some(right_estimate)) => {
                let left_cost = left_estimate.estimated_review_cost.max(1);
                let right_cost = right_estimate.estimated_review_cost.max(1);
                let left_ratio = left_estimate.expected_information_gain * right_cost;
                let right_ratio = right_estimate.expected_information_gain * left_cost;
                right_ratio
                    .cmp(&left_ratio)
                    .then_with(|| {
                        right_estimate
                            .expected_information_gain
                            .cmp(&left_estimate.expected_information_gain)
                    })
                    .then_with(|| left.required_review_slots.cmp(&right.required_review_slots))
                    .then_with(|| {
                        let left_order = plan_order
                            .get(left.experiment_campaign_id.as_str())
                            .copied()
                            .unwrap_or(usize::MAX);
                        let right_order = plan_order
                            .get(right.experiment_campaign_id.as_str())
                            .copied()
                            .unwrap_or(usize::MAX);
                        left_order.cmp(&right_order)
                    })
            }
            (Some(_), None) => Ordering::Less,
            (None, Some(_)) => Ordering::Greater,
            (None, None) => Ordering::Equal,
        }
    });

    for campaign in candidate_campaigns {
        let estimate = estimate_by_campaign.get(campaign.experiment_campaign_id.as_str());
        let expected_information_gain = estimate
            .map(|estimate| estimate.expected_information_gain)
            .unwrap_or_default();
        let estimated_review_cost = estimate
            .map(|estimate| estimate.estimated_review_cost)
            .unwrap_or(campaign.required_review_slots);
        let budget_pressure = if remaining == 0 {
            1.0
        } else {
            campaign.required_review_slots as f64 / remaining as f64
        };
        let hypothesis_context = if hypotheses.hypothesis_refs.is_empty() {
            "no hypothesis refs were attached".to_string()
        } else {
            format!(
                "hypothesis set remains visible: {}",
                hypotheses.hypothesis_refs.join(", ")
            )
        };

        let (decision, rationale) = if estimate.is_none() {
            degraded = true;
            (
                CampaignDecision::Defer,
                format!(
                    "campaign is deferred because no typed information-value estimate was supplied; {hypothesis_context}"
                ),
            )
        } else if budget.exhausted || remaining == 0 {
            degraded = true;
            (
                CampaignDecision::PauseBudgetExhausted,
                format!(
                    "verification budget is exhausted, so the program pauses explicitly even though expected information gain is {expected_information_gain}; {hypothesis_context}"
                ),
            )
        } else if remaining >= campaign.required_review_slots {
            remaining -= campaign.required_review_slots;
            (
                CampaignDecision::Launch,
                format!(
                    "campaign launches because expected information gain {expected_information_gain} justifies review cost {estimated_review_cost} under budget pressure {:.2}; {hypothesis_context}",
                    budget_pressure
                ),
            )
        } else {
            degraded = true;
            (
                CampaignDecision::Defer,
                format!(
                    "campaign is deferred because review cost {estimated_review_cost} would exceed remaining review slots {remaining} under budget pressure {:.2}; {hypothesis_context}",
                    budget_pressure
                ),
            )
        };

        decisions.push(CampaignDecisionLineV1 {
            campaign_id: campaign.experiment_campaign_id.clone(),
            decision,
            expected_information_gain,
            estimated_review_cost,
            budget_pressure,
            hypothesis_refs: hypotheses.hypothesis_refs.clone(),
            rationale,
        });
    }

    CampaignDecisionTraceV1 {
        schema_version: CAMPAIGN_DECISION_TRACE_V1_SCHEMA.into(),
        campaign_decision_trace_id: CampaignDecisionTraceId::generate(),
        portfolio_plan_id: plan.portfolio_plan_id.clone(),
        discovery_program_id: program.discovery_program_id.clone(),
        verification_load_budget_id: budget.verification_load_budget_id.clone(),
        decisions,
        remaining_review_slots: remaining,
        advisory_only: true,
        degraded,
        generated_at: generated_at.into(),
    }
}
