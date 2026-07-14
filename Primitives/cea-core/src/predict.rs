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
    /// Structural matching is advisory-only and must be explicitly enabled.
    #[serde(default)]
    pub enable_fuzzy_matching: bool,
    /// Minimum interpretable structural agreement required for a fuzzy match.
    #[serde(default = "default_min_structural_similarity")]
    pub min_structural_similarity: f64,
    /// Maximum coverage a fuzzy match may contribute.
    #[serde(default = "default_fuzzy_coverage_cap")]
    pub fuzzy_coverage_cap: f64,
}

fn default_min_structural_similarity() -> f64 {
    0.75
}
fn default_fuzzy_coverage_cap() -> f64 {
    0.25
}

impl Default for PredictionConfig {
    fn default() -> Self {
        Self {
            risk_confidence_threshold: 0.65,
            zero_shot_coverage_threshold: 0.6,
            fuzzy_top_k: 3,
            min_samples_per_signature: calibration::MIN_SAMPLES_PER_SIGNATURE,
            enable_fuzzy_matching: false,
            min_structural_similarity: default_min_structural_similarity(),
            fuzzy_coverage_cap: default_fuzzy_coverage_cap(),
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
        let matches = resolve_signature_matches(signature, graph, config);
        let signature_coverage = matches
            .iter()
            .map(|candidate| candidate.coverage_weight)
            .fold(0.0, f64::max);
        coverage_total += signature_coverage;
        if matches.is_empty() {
            continue;
        }

        let mut signature_confidence = 0.0_f64;
        let mut signature_edge_count = 0_usize;

        for matched in matches {
            for (target_index, edge) in graph.outgoing_edges(matched.node_index) {
                let Some(CausalNode::Effect(effect_signature)) =
                    graph.graph.node_weight(target_index)
                else {
                    continue;
                };

                let edge_observations = edge.stats.observations as f64;
                let match_coverage = matched.coverage_weight;
                let raw_reliability =
                    calibration::conservative_reliability(edge.stats.alpha, edge.stats.beta)
                        * match_coverage;
                signature_confidence += raw_reliability;
                signature_edge_count += 1;
                let edge_conservative_confidence = calibration::advisory_confidence(
                    raw_reliability,
                    1.0,
                    edge_observations,
                    1,
                    config.min_samples_per_signature,
                );

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

        if signature_edge_count > 0 {
            signature_confidences.push(signature_confidence / signature_edge_count as f64);
        }
    }

    let coverage_fraction = (coverage_total / signatures.len() as f64).clamp(0.0, 1.0);
    let total_signal = positive + negative;
    let modeled_correctness = if total_signal.abs() < f64::EPSILON {
        0.5
    } else {
        (positive / total_signal).clamp(0.0, 1.0)
    };
    let blended_correctness =
        modeled_correctness * coverage_fraction + 0.5 * (1.0 - coverage_fraction);
    let signature_confidence = if signature_confidences.is_empty() {
        0.0
    } else {
        signature_confidences.iter().sum::<f64>() / signature_confidences.len() as f64
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

    // A graph edge records association. Independent interventional support is
    // required before any caller may skip checks, and that evidence is not
    // represented in this legacy graph snapshot.
    let zero_shot_eligible = false;

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
    config: &PredictionConfig,
) -> Vec<MatchCandidate> {
    let node_id = edit_op_node_id(signature);
    if let Some(node_index) = graph.node_index_map.get(&node_id) {
        return vec![MatchCandidate {
            node_index: *node_index,
            coverage_weight: 1.0,
        }];
    }

    if !config.enable_fuzzy_matching {
        return Vec::new();
    }

    let mut fuzzy = graph
        .cause_nodes()
        .into_iter()
        .filter_map(|(node_index, candidate)| {
            let similarity = heuristic_similarity(signature, candidate);
            (similarity >= config.min_structural_similarity).then_some((node_index, similarity))
        })
        .collect::<Vec<_>>();

    fuzzy.sort_by(|left, right| {
        right
            .1
            .partial_cmp(&left.1)
            .unwrap_or(Ordering::Equal)
            .then_with(|| left.0.index().cmp(&right.0.index()))
    });
    fuzzy.truncate(config.fuzzy_top_k.max(1));

    let total_similarity = fuzzy.iter().map(|(_, similarity)| *similarity).sum::<f64>();
    if total_similarity <= f64::EPSILON {
        return Vec::new();
    }

    fuzzy
        .into_iter()
        .map(|(node_index, similarity)| MatchCandidate {
            node_index,
            coverage_weight: similarity.min(config.fuzzy_coverage_cap.clamp(0.0, 1.0)),
        })
        .collect()
}

fn heuristic_similarity(left: &EditOpSignature, right: &EditOpSignature) -> f64 {
    let mut score: f64 = 0.0;
    let mut max_score: f64 = 0.0;

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
    score += line_change_similarity(left, right) * 2.0;

    // Cryptographic digests are opaque: either an informative source context
    // matches exactly or it supplies no structural evidence. The digest of an
    // empty context is shared by every range edit and must not count as a match.
    max_score += 3.0;
    if is_informative_context_hash(&left.context_hash)
        && is_informative_context_hash(&right.context_hash)
        && left.context_hash == right.context_hash
    {
        score += 3.0;
    }

    (score / max_score).clamp(0.0, 1.0)
}

fn line_change_similarity(left: &EditOpSignature, right: &EditOpSignature) -> f64 {
    fn ratio(left: u32, right: u32) -> f64 {
        match (left, right) {
            (0, 0) => 1.0,
            _ => f64::from(left.min(right)) / f64::from(left.max(right).max(1)),
        }
    }

    (ratio(left.lines_added, right.lines_added) + ratio(left.lines_removed, right.lines_removed))
        / 2.0
}

fn is_informative_context_hash(hash: &str) -> bool {
    !hash.is_empty() && hash != blake3::hash(b"").to_hex().as_str()
}
