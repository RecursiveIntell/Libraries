use std::cmp::Ordering;

use crate::attribution::{edit_op_node_id, effect_node_id};
use crate::calibration;
use crate::graph::{CausalGraph, CausalNode};
use crate::types::{CausalPrediction, EditOpSignature, RiskFlag};
use check_runner::EffectSignature;

const ZERO_SHOT_MIN_EFFECTIVE_SAMPLES: f64 = 5.0;
const RISK_MATCH_COVERAGE_FLOOR: f64 = 0.90;

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct PredictionConfig {
    pub risk_confidence_threshold: f64,
    pub zero_shot_coverage_threshold: f64,
    pub fuzzy_top_k: usize,
    /// Minimum sample units per signature to treat confidence as fully evidence-backed.
    pub min_samples_per_signature: usize,
}

impl Default for PredictionConfig {
    fn default() -> Self {
        Self {
            risk_confidence_threshold: 0.65,
            zero_shot_coverage_threshold: 0.6,
            fuzzy_top_k: 3,
            min_samples_per_signature: calibration::MIN_SAMPLES_PER_SIGNATURE,
        }
    }
}

pub fn predict(
    signatures: &[EditOpSignature],
    graph: &CausalGraph,
    risk_confidence_threshold: f64,
    zero_shot_coverage_threshold: f64,
) -> CausalPrediction {
    let config = PredictionConfig {
        risk_confidence_threshold,
        zero_shot_coverage_threshold,
        ..PredictionConfig::default()
    };
    predict_with_config(signatures, graph, &config)
}

pub fn predict_with_config(
    signatures: &[EditOpSignature],
    graph: &CausalGraph,
    config: &PredictionConfig,
) -> CausalPrediction {
    if signatures.is_empty() {
        return CausalPrediction {
            predicted_correctness: 0.5,
            predicted_novelty: 1.0,
            confidence: 0.0,
            coverage_fraction: 0.0,
            risk_flags: Vec::new(),
            zero_shot_eligible: false,
        };
    }

    let mut positive = 0.0;
    let mut negative = 0.0;
    let mut coverage_total = 0.0;
    let mut risk_candidates = Vec::new();
    let mut sample_evidence = 0.0;
    let mut signature_confidences = Vec::new();

    for signature in signatures {
        let matches = resolve_signature_matches(signature, graph, config.fuzzy_top_k);
        let signature_coverage = matches
            .iter()
            .map(|candidate| candidate.coverage_weight)
            .fold(0.0, f64::max);
        coverage_total += signature_coverage;
        if matches.is_empty() {
            continue;
        }

        let mut signature_confidence = 1.0_f64;

        for matched in matches {
            for (target_index, edge) in graph.outgoing_edges(matched.node_index) {
                let Some(CausalNode::Effect(effect_signature)) =
                    graph.graph.node_weight(target_index)
                else {
                    continue;
                };

                let edge_observations = edge.stats.observations as f64;
                let match_coverage = matched.coverage_weight;
                let edge_conservative_confidence = calibration::advisory_confidence(
                    calibration::conservative_reliability(edge.stats.alpha, edge.stats.beta),
                    match_coverage,
                    edge_observations,
                    1,
                    config.min_samples_per_signature,
                );
                signature_confidence = signature_confidence.min(edge_conservative_confidence);

                let signal = edge.weight.max(0.0) * edge.stats.mean() * match_coverage;
                if signal <= f64::EPSILON {
                    continue;
                }

                sample_evidence += edge_observations * match_coverage;

                if effect_signature.outcome == "pass" {
                    positive += signal;
                } else {
                    negative += signal;
                    risk_candidates.push(RawRiskCandidate {
                        op_signature: signature.clone(),
                        predicted_effect: effect_signature.clone(),
                        raw_confidence: edge_conservative_confidence,
                        coverage_weight: matched.coverage_weight,
                        effective_sample_size: calibration::effective_sample_size(
                            edge_observations,
                        ),
                        historical_weight: edge.weight,
                    });
                }
            }
        }

        signature_confidences.push(signature_confidence);
    }

    let coverage_fraction = (coverage_total / signatures.len() as f64).clamp(0.0, 1.0);
    let total_signal = positive + negative;
    let modeled_correctness = if total_signal.abs() < f64::EPSILON {
        0.5
    } else {
        (positive / total_signal).clamp(0.0, 1.0)
    };
    let blended_correctness = modeled_correctness;
    let signature_confidence = if signature_confidences.is_empty() {
        0.0
    } else {
        signature_confidences.into_iter().fold(1.0, f64::min)
    };
    let confidence = calibration::advisory_confidence(
        signature_confidence,
        1.0,
        sample_evidence,
        signatures.len(),
        config.min_samples_per_signature,
    );

    let mut risk_flags = Vec::new();
    for candidate in risk_candidates {
        if candidate.raw_confidence >= config.risk_confidence_threshold
            && candidate.coverage_weight >= RISK_MATCH_COVERAGE_FLOOR
            && candidate.effective_sample_size >= ZERO_SHOT_MIN_EFFECTIVE_SAMPLES
        {
            risk_flags.push(RiskFlag {
                op_signature: candidate.op_signature,
                predicted_effect: candidate.predicted_effect,
                confidence: candidate.raw_confidence,
                historical_weight: candidate.historical_weight,
            });
        }
    }

    let zero_shot_eligible = coverage_fraction >= config.zero_shot_coverage_threshold
        && confidence >= config.risk_confidence_threshold
        && calibration::effective_sample_size(sample_evidence) >= ZERO_SHOT_MIN_EFFECTIVE_SAMPLES
        && risk_flags.is_empty();

    risk_flags.sort_by(|left, right| {
        right
            .confidence
            .partial_cmp(&left.confidence)
            .unwrap_or(Ordering::Equal)
            .then_with(|| {
                effect_node_id(&left.predicted_effect).cmp(&effect_node_id(&right.predicted_effect))
            })
            .then_with(|| {
                edit_op_node_id(&left.op_signature).cmp(&edit_op_node_id(&right.op_signature))
            })
    });

    CausalPrediction {
        predicted_correctness: blended_correctness.clamp(0.0, 1.0),
        predicted_novelty: (1.0 - coverage_fraction).clamp(0.0, 1.0),
        confidence,
        coverage_fraction,
        risk_flags,
        zero_shot_eligible,
    }
}

#[derive(Debug, Clone)]
struct MatchCandidate {
    node_index: petgraph::graph::NodeIndex,
    coverage_weight: f64,
}

#[derive(Debug, Clone)]
struct RawRiskCandidate {
    op_signature: EditOpSignature,
    predicted_effect: EffectSignature,
    raw_confidence: f64,
    coverage_weight: f64,
    effective_sample_size: f64,
    historical_weight: f64,
}

fn resolve_signature_matches(
    signature: &EditOpSignature,
    graph: &CausalGraph,
    fuzzy_top_k: usize,
) -> Vec<MatchCandidate> {
    let node_id = edit_op_node_id(signature);
    if let Some(node_index) = graph.node_index_map.get(&node_id) {
        return vec![MatchCandidate {
            node_index: *node_index,
            coverage_weight: 1.0,
        }];
    }

    let mut fuzzy = graph
        .cause_nodes()
        .into_iter()
        .filter_map(|(node_index, candidate)| {
            let similarity = heuristic_similarity(signature, candidate);
            (similarity > 0.0).then_some((node_index, similarity))
        })
        .collect::<Vec<_>>();

    fuzzy.sort_by(|left, right| {
        right
            .1
            .partial_cmp(&left.1)
            .unwrap_or(Ordering::Equal)
            .then_with(|| left.0.index().cmp(&right.0.index()))
    });
    fuzzy.truncate(fuzzy_top_k.max(1));

    let total_similarity = fuzzy.iter().map(|(_, similarity)| *similarity).sum::<f64>();
    if total_similarity <= f64::EPSILON {
        return Vec::new();
    }

    fuzzy
        .into_iter()
        .map(|(node_index, similarity)| MatchCandidate {
            node_index,
            coverage_weight: similarity,
        })
        .collect()
}

fn heuristic_similarity(left: &EditOpSignature, right: &EditOpSignature) -> f64 {
    let mut score = 0.0;
    let mut max_score = 0.0;

    max_score += 3.0;
    if left.op_kind == right.op_kind {
        score += 3.0;
    }

    max_score += 1.0;
    if left.anchor_kind == right.anchor_kind {
        score += 1.0;
    }

    max_score += 2.0;
    if left.file_extension == right.file_extension {
        score += 2.0;
    }

    max_score += 2.0;
    if left.scope_tag == right.scope_tag {
        score += 2.0;
    }

    max_score += 2.0;
    score += shared_prefix_ratio(&left.context_hash, &right.context_hash) * 2.0;

    (score / max_score).clamp(0.0, 1.0)
}

fn shared_prefix_ratio(left: &str, right: &str) -> f64 {
    let max_len = left.len().min(right.len()).max(1) as f64;
    let shared = left
        .chars()
        .zip(right.chars())
        .take_while(|(left, right)| left == right)
        .count() as f64;
    shared / max_len
}
