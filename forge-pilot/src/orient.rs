//! Orient phase of the OODA loop.
//!
//! Extracts verification targets from an observation snapshot, scores
//! them by urgency, and selects an execution plan for each candidate.

use crate::act::{AdvisoryPlan, PlanKind};
use crate::config::LoopConfig;
use crate::history::PilotHistory;
use crate::observe::{Observation, ObservationDisposition};
use crate::targets::TargetKind;
use crate::types::TargetNormalization;
use std::cmp::Ordering;
use std::collections::hash_map::DefaultHasher;
use std::collections::BTreeSet;
use std::hash::{Hash, Hasher};

/// A scored verification target with its selected execution plan.
#[derive(Debug, Clone)]
pub struct TargetCandidate {
    pub target: TargetKind,
    pub stable_key: String,
    pub urgency: f64,
    pub rationale: String,
    pub check_hints: Vec<String>,
    pub plan: PlanKind,
    pub normalization: TargetNormalization,
}

/// Extracts target kinds worth considering from an observation snapshot.
pub fn extract_targets(observation: &Observation, config: &LoopConfig) -> Vec<TargetKind> {
    let mut targets = Vec::new();

    if let Some(scheduled) = &observation.scheduled {
        for syndrome in &scheduled.execution.syndromes {
            targets.push(TargetKind::ActiveSyndrome {
                signature: syndrome.signature.clone(),
            });
        }

        for belief in &scheduled.execution.node_beliefs {
            if belief.node_id.starts_with("nuisance:") {
                continue;
            }
            if belief.belief_micros < 900_000 {
                targets.push(TargetKind::FragileNode {
                    node_id: belief.node_id.clone(),
                    belief_micros: belief.belief_micros,
                });
            }
        }
    }

    if let Some(compiled) = &observation.compiled {
        for marker in &compiled.degradations {
            if matches!(
                marker,
                constraint_compiler::ConstraintDegradation::ThinExport
            ) {
                targets.push(TargetKind::ThinExport {
                    marker: "thin_export".into(),
                });
            }
        }
    }

    for degradation in &observation.degradations {
        if degradation.kind == "thin_export" {
            targets.push(TargetKind::ThinExport {
                marker: degradation.detail.clone(),
            });
        }
        if degradation.kind == "scope_stale" {
            targets.push(TargetKind::ScopeStale {
                last_import_at: observation
                    .import_log
                    .as_ref()
                    .map(|row| row.imported_at.clone()),
            });
        }
    }

    if matches!(
        observation.status.disposition,
        ObservationDisposition::ImportRequired
    ) {
        targets.push(TargetKind::ScopeStale {
            last_import_at: None,
        });
    }

    for claim in &observation.claim_versions {
        if claim.claim_state == "pending_review" {
            targets.push(TargetKind::UnverifiedClaimVersion {
                claim_version_id: claim.claim_version_id.clone(),
            });
        }
        if let Some(supersedes_claim_version_id) = &claim.supersedes_claim_version_id {
            targets.push(TargetKind::SupersessionVerification {
                claim_version_id: claim.claim_version_id.clone(),
                supersedes_claim_version_id: supersedes_claim_version_id.clone(),
            });
        }
    }

    if let Some(explanation) = &observation.explanation {
        if !matches!(
            explanation.causal_refutation_outcome.as_str(),
            "flip_witness" | "supported"
        ) {
            let promoted_claim_version_id = observation
                .claim_versions
                .iter()
                .find(|claim| {
                    claim
                        .metadata
                        .as_ref()
                        .and_then(|metadata| metadata.get("promotion_state"))
                        .and_then(|promotion_state| promotion_state.get("state"))
                        .and_then(|state| state.as_str())
                        == Some("promoted")
                })
                .map(|claim| claim.claim_version_id.clone());
            for node_id in &explanation.witness_node_ids {
                targets.push(TargetKind::RefutationGap {
                    target_node_id: node_id.clone(),
                    claim_version_id: promoted_claim_version_id.clone(),
                });
            }
        }
        for marker in &explanation.calibration_caveats {
            targets.push(TargetKind::CalibrationCaveat {
                marker: marker.clone(),
            });
        }
    }

    let comparability_versions = observation
        .recent_imports
        .iter()
        .filter_map(|row| row.comparability_snapshot_version.clone())
        .collect::<BTreeSet<_>>();
    if comparability_versions.len() > 1 {
        targets.push(TargetKind::ComparabilityDrift {
            detail: comparability_versions
                .into_iter()
                .collect::<Vec<_>>()
                .join(","),
        });
    }

    dedupe_targets(targets, config)
}

/// Scores concrete target candidates from an observation and pilot history.
pub fn score_targets(
    observation: &Observation,
    history: &PilotHistory,
    config: &LoopConfig,
) -> Vec<TargetCandidate> {
    let mut candidates = extract_targets(observation, config)
        .into_iter()
        .map(|target| {
            let stable_key = target.stable_key();
            let prior_attempts = history.retry_count(&stable_key);
            let plan = choose_plan(observation, config, &target);
            let mut urgency = score_target(observation, &target, prior_attempts);
            if matches!(plan, PlanKind::PairedPatch { .. }) {
                // An explicit patch seed is an operator-selected execution path and should
                // outrank generic oracle fallback candidates for the same observation.
                urgency = 1.1;
            }
            TargetCandidate {
                rationale: build_rationale(observation, &target, prior_attempts),
                check_hints: build_check_hints(&target),
                normalization: normalize_target(observation, &target),
                target,
                stable_key,
                urgency,
                plan,
            }
        })
        .collect::<Vec<_>>();

    candidates.sort_by(compare_candidates);
    candidates
}

fn normalize_target(observation: &Observation, target: &TargetKind) -> TargetNormalization {
    TargetNormalization {
        stable_target_key: target.stable_key(),
        canonical_case_class: target.canonical_case_class(),
        bounded_region_digest: bounded_region_digest(observation, target),
        required_artifact_families: target.required_artifact_families(),
        budget_class: target.budget_class(),
        comparability_required: matches!(target, TargetKind::ComparabilityDrift { .. }),
        nuisance_sensitive: observation
            .compiled
            .as_ref()
            .map(|compiled| {
                compiled
                    .nodes
                    .iter()
                    .any(|node| node.kind == "nuisance_state")
            })
            .unwrap_or(false),
        missing_falsifier: matches!(
            target,
            TargetKind::RefutationGap {
                claim_version_id: None,
                ..
            }
        ),
    }
}

fn bounded_region_digest(observation: &Observation, target: &TargetKind) -> String {
    let mut hasher = DefaultHasher::new();
    observation.scope_key.hash(&mut hasher);
    target.stable_key().hash(&mut hasher);
    observation
        .import_log
        .as_ref()
        .map(|row| row.imported_at.as_str())
        .hash(&mut hasher);
    observation
        .claim_versions
        .iter()
        .map(|claim| claim.claim_version_id.as_str())
        .collect::<Vec<_>>()
        .hash(&mut hasher);
    format!("region:{:016x}", hasher.finish())
}

fn dedupe_targets(targets: Vec<TargetKind>, _config: &LoopConfig) -> Vec<TargetKind> {
    let mut seen = BTreeSet::new();
    let mut deduped = Vec::new();
    for target in targets {
        let key = target.stable_key();
        if seen.insert(key) {
            deduped.push(target);
        }
    }
    deduped
}

fn score_target(observation: &Observation, target: &TargetKind, prior_attempts: u32) -> f64 {
    let mut urgency = match target {
        TargetKind::ActiveSyndrome { .. } => 0.95,
        TargetKind::FragileNode { belief_micros, .. } => {
            1.0 - (*belief_micros as f64 / 1_000_000.0)
        }
        TargetKind::ThinExport { .. } => 0.85,
        TargetKind::UnverifiedClaimVersion { .. } => 0.75,
        TargetKind::RefutationGap {
            claim_version_id: Some(_),
            ..
        } => 0.92,
        TargetKind::RefutationGap { .. } => 0.72,
        TargetKind::SupersessionVerification { .. } => 0.68,
        TargetKind::ComparabilityDrift { .. } => 0.60,
        TargetKind::CalibrationCaveat { .. } => 0.50,
        TargetKind::ScopeStale { last_import_at } => {
            if last_import_at.is_none() {
                0.40
            } else {
                0.25
            }
        }
    };

    if observation
        .risk_gate
        .as_ref()
        .is_some_and(|risk_gate| risk_gate.status == "blocked")
    {
        urgency += 0.05;
    }
    if matches!(target, TargetKind::ThinExport { .. }) && observation.thin_export_active() {
        urgency += 0.05;
    }
    if observation.explanation.as_ref().is_some_and(|explanation| {
        explanation.residual_count > 0 && explanation.certificate_count == 0
    }) {
        urgency += 0.03;
    }
    if prior_attempts > 0 {
        urgency -= 0.5_f64.powi(prior_attempts as i32);
    }
    urgency.clamp(0.0, 1.0)
}

fn compare_candidates(left: &TargetCandidate, right: &TargetCandidate) -> Ordering {
    right
        .urgency
        .partial_cmp(&left.urgency)
        .unwrap_or(Ordering::Equal)
        .then_with(|| left.target.priority().cmp(&right.target.priority()))
        .then_with(|| left.stable_key.cmp(&right.stable_key))
}

fn choose_plan(observation: &Observation, config: &LoopConfig, target: &TargetKind) -> PlanKind {
    let stable_key = target.stable_key();
    if let Some(seed) = config.patch_seed_for(&stable_key) {
        return PlanKind::PairedPatch {
            fixture_path: seed.fixture_path.clone(),
            patch: seed.patch.clone(),
            experiment_config: seed.experiment_config.clone(),
            description: if seed.description.is_empty() {
                format!("paired patch plan for {}", target.stable_key())
            } else {
                seed.description.clone()
            },
        };
    }

    match target {
        TargetKind::ActiveSyndrome { .. } => {
            if let Some(oracle) = &observation.oracle {
                PlanKind::OracleExactBounded {
                    oracle_slice_id: oracle.slice_id.clone(),
                }
            } else {
                PlanKind::OracleConservative
            }
        }
        TargetKind::UnverifiedClaimVersion { claim_version_id } => {
            if claim_is_promoted(observation, claim_version_id) {
                PlanKind::OracleCausalRefuter {
                    target_node_id: observation
                        .explanation
                        .as_ref()
                        .and_then(|explanation| explanation.witness_node_ids.first().cloned())
                        .or_else(|| claim_subject_entity_id(observation, claim_version_id))
                        .unwrap_or_else(|| claim_version_id.as_str().to_string()),
                    max_removed_nodes: config.minimal_perturbation_budget,
                }
            } else if let Some(oracle) = &observation.oracle {
                PlanKind::OracleExactBounded {
                    oracle_slice_id: oracle.slice_id.clone(),
                }
            } else {
                PlanKind::OracleConservative
            }
        }
        TargetKind::FragileNode { node_id, .. } => PlanKind::OracleMinimalPerturbation {
            target_node_id: node_id.clone(),
            max_removed_nodes: config.minimal_perturbation_budget,
        },
        TargetKind::RefutationGap { target_node_id, .. } => PlanKind::OracleCausalRefuter {
            target_node_id: target_node_id.clone(),
            max_removed_nodes: config.minimal_perturbation_budget,
        },
        TargetKind::SupersessionVerification { .. } | TargetKind::ComparabilityDrift { .. } => {
            PlanKind::OracleTemporalReplay {
                cutoff_recorded_at: observation
                    .import_log
                    .as_ref()
                    .map(|row| row.imported_at.clone())
                    .unwrap_or_else(|| chrono::Utc::now().to_rfc3339()),
            }
        }
        TargetKind::CalibrationCaveat { .. } => {
            let changed_node_ids = observation
                .explanation
                .as_ref()
                .map(|explanation| explanation.witness_node_ids.clone())
                .unwrap_or_default();
            PlanKind::OracleDeltaParity {
                changed_node_ids,
                max_iterations: config.oracle_max_iterations,
            }
        }
        TargetKind::ThinExport { marker } => PlanKind::AdvisoryOnlyVerificationPlan(AdvisoryPlan {
            description: format!("advisory-only plan for {marker}"),
        }),
        TargetKind::ScopeStale { last_import_at } => {
            PlanKind::AdvisoryOnlyVerificationPlan(AdvisoryPlan {
                description: if last_import_at.is_none()
                    && matches!(
                        observation.status.disposition,
                        ObservationDisposition::ImportRequired
                    ) {
                    format!("bootstrap advisory: {}", observation.status.exact_next_step)
                } else {
                    format!(
                        "advisory-only plan for stale scope {}",
                        last_import_at.as_deref().unwrap_or("never")
                    )
                },
            })
        }
    }
}

fn claim_is_promoted(
    observation: &Observation,
    claim_version_id: &stack_ids::ClaimVersionId,
) -> bool {
    observation.claim_versions.iter().any(|claim| {
        claim.claim_version_id == *claim_version_id
            && claim
                .metadata
                .as_ref()
                .and_then(|metadata| metadata.get("promotion_state"))
                .and_then(|promotion_state| promotion_state.get("state"))
                .and_then(|state| state.as_str())
                == Some("promoted")
    })
}

fn claim_subject_entity_id(
    observation: &Observation,
    claim_version_id: &stack_ids::ClaimVersionId,
) -> Option<String> {
    observation
        .claim_versions
        .iter()
        .find(|claim| claim.claim_version_id == *claim_version_id)
        .map(|claim| claim.subject_entity_id.as_str().to_string())
}

fn build_rationale(observation: &Observation, target: &TargetKind, prior_attempts: u32) -> String {
    let degradation_hint = if observation.thin_export_active() {
        "degradation active"
    } else if matches!(
        observation.status.disposition,
        ObservationDisposition::ImportRequired
    ) {
        "canonical import missing"
    } else {
        "full kernel payload available"
    };
    format!(
        "target={} prior_attempts={} {}",
        target.stable_key(),
        prior_attempts,
        degradation_hint
    )
}

fn build_check_hints(target: &TargetKind) -> Vec<String> {
    match target {
        TargetKind::ActiveSyndrome { signature } => vec![format!("inspect syndrome {signature}")],
        TargetKind::FragileNode { node_id, .. } => vec![format!("perturb node {node_id}")],
        TargetKind::ThinExport { marker } => vec![format!("inspect export thinness {marker}")],
        TargetKind::UnverifiedClaimVersion { claim_version_id } => {
            vec![format!("verify claim {}", claim_version_id.as_str())]
        }
        TargetKind::RefutationGap { target_node_id, .. } => {
            vec![format!("run refuter for {target_node_id}")]
        }
        TargetKind::SupersessionVerification {
            claim_version_id,
            supersedes_claim_version_id,
        } => vec![format!(
            "replay supersession {} -> {}",
            claim_version_id.as_str(),
            supersedes_claim_version_id.as_str()
        )],
        TargetKind::ComparabilityDrift { detail } => {
            vec![format!("inspect comparability versions {detail}")]
        }
        TargetKind::CalibrationCaveat { marker } => vec![format!("review calibration {marker}")],
        TargetKind::ScopeStale { last_import_at } => vec![format!(
            "scope import stale at {}",
            last_import_at.as_deref().unwrap_or("never")
        )],
    }
}
