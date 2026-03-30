use crate::error::PilotError;
use crate::observe::Observation;
use crate::orient::TargetCandidate;
use semantic_memory::MemoryStore;
use serde_json::json;
use verification_adjudication::AdjudicationResult;
use verification_control::{CheckMethod, PromotionClass, ReversibilityClass, VerificationCase};

pub(super) fn promotion_class_for_plan(
    plan: &crate::act::PlanKind,
    degraded: bool,
) -> PromotionClass {
    if degraded || matches!(plan, crate::act::PlanKind::AdvisoryOnlyVerificationPlan(_)) {
        PromotionClass::P0
    } else {
        match plan {
            crate::act::PlanKind::PairedPatch { .. } => PromotionClass::P2,
            crate::act::PlanKind::OracleExactBounded { .. }
            | crate::act::PlanKind::OracleConservative
            | crate::act::PlanKind::OracleDeltaParity { .. }
            | crate::act::PlanKind::OracleTemporalReplay { .. }
            | crate::act::PlanKind::OracleCausalRefuter { .. }
            | crate::act::PlanKind::OracleMinimalPerturbation { .. } => PromotionClass::P2,
            crate::act::PlanKind::AdvisoryOnlyVerificationPlan(_) => PromotionClass::P0,
        }
    }
}

pub(super) fn reversibility_class_for_plan(plan: &crate::act::PlanKind) -> ReversibilityClass {
    match plan {
        crate::act::PlanKind::PairedPatch { .. } => ReversibilityClass::RequiresSupersession,
        crate::act::PlanKind::AdvisoryOnlyVerificationPlan(_) => {
            ReversibilityClass::ReversibleLocal
        }
        _ => ReversibilityClass::ReversibleScoped,
    }
}

pub(super) fn currently_promoted(observation: &Observation, candidate: &TargetCandidate) -> bool {
    let Some(claim_version_id) = candidate.target.primary_claim_version_id() else {
        return false;
    };

    observation.claim_versions.iter().any(|claim| {
        claim.claim_version_id == claim_version_id
            && claim
                .metadata
                .as_ref()
                .and_then(|metadata| metadata.get("promotion_state"))
                .and_then(|promotion_state| promotion_state.get("state"))
                .and_then(|state| state.as_str())
                == Some("promoted")
    })
}

pub(super) async fn execute_rollback_invalidation(
    memory_store: &MemoryStore,
    case: &VerificationCase,
    adjudication: &AdjudicationResult,
) -> Result<Option<usize>, PilotError> {
    let Some(claim_version_id) = case.region.claim_version_id.as_ref() else {
        return Ok(Some(0));
    };
    let invalidated = memory_store
        .invalidate_derivations(
            "claim",
            claim_version_id.as_str(),
            "on_source_change",
            adjudication.rollback_plan.reason.as_str(),
        )
        .await
        .map_err(PilotError::Memory)?;
    Ok(Some(invalidated))
}

pub(super) fn map_check_method(plan: &crate::act::PlanKind) -> CheckMethod {
    match plan {
        crate::act::PlanKind::OracleExactBounded { .. } => CheckMethod::ExactBoundedOracle,
        crate::act::PlanKind::OracleConservative => CheckMethod::ConservativeOracle,
        crate::act::PlanKind::OracleDeltaParity { .. } => CheckMethod::DeltaParityOracle,
        crate::act::PlanKind::OracleTemporalReplay { .. } => CheckMethod::TemporalReplayOracle,
        crate::act::PlanKind::OracleCausalRefuter { .. } => CheckMethod::CausalRefuter,
        crate::act::PlanKind::OracleMinimalPerturbation { .. } => {
            CheckMethod::MinimalPerturbationOracle
        }
        crate::act::PlanKind::PairedPatch { .. } => CheckMethod::PairedPatch,
        crate::act::PlanKind::AdvisoryOnlyVerificationPlan(_) => CheckMethod::AdvisoryOnly,
    }
}

pub(super) fn plan_input(plan: &crate::act::PlanKind) -> serde_json::Value {
    match plan {
        crate::act::PlanKind::OracleExactBounded { oracle_slice_id } => {
            json!({ "oracle_slice_id": oracle_slice_id })
        }
        crate::act::PlanKind::OracleConservative => json!({ "mode": "conservative" }),
        crate::act::PlanKind::OracleDeltaParity {
            changed_node_ids,
            max_iterations,
        } => json!({
            "changed_node_ids": changed_node_ids,
            "max_iterations": max_iterations,
        }),
        crate::act::PlanKind::OracleTemporalReplay { cutoff_recorded_at } => {
            json!({ "cutoff_recorded_at": cutoff_recorded_at })
        }
        crate::act::PlanKind::OracleCausalRefuter {
            target_node_id,
            max_removed_nodes,
        }
        | crate::act::PlanKind::OracleMinimalPerturbation {
            target_node_id,
            max_removed_nodes,
        } => json!({
            "target_node_id": target_node_id,
            "max_removed_nodes": max_removed_nodes,
        }),
        crate::act::PlanKind::PairedPatch {
            fixture_path,
            description,
            ..
        } => json!({
            "fixture_path": fixture_path,
            "description": description,
        }),
        crate::act::PlanKind::AdvisoryOnlyVerificationPlan(plan) => {
            json!({ "description": plan.description })
        }
    }
}
