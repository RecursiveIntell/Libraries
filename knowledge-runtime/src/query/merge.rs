use semantic_memory::SearchResult;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Merge policy for combining results from multiple route legs.
///
/// The merge pipeline:
/// 1. **Collect** — gather raw results from each leg
/// 2. **Fuse duplicates** — merge duplicates by source identity, preserving
///    provenance from all contributing legs (union of source legs, best score)
/// 3. **Normalize** — bring scores to a common scale
/// 4. **Boost** — apply multi-leg support boost to fused results
/// 5. **Rank** — order by final score descending
/// 6. **Truncate** — apply the final limit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergePolicy {
    /// Final result limit after merge.
    pub limit: usize,
    /// How to handle score normalization across legs.
    pub normalization: ScoreNormalization,
    /// Boost factor applied to results that appeared in multiple legs.
    /// Each additional leg multiplies the score by `(1.0 + multi_leg_boost)`.
    /// Set to 0.0 to disable boosting.
    pub multi_leg_boost: f64,
}

impl Default for MergePolicy {
    fn default() -> Self {
        Self {
            limit: 10,
            normalization: ScoreNormalization::MinMax,
            multi_leg_boost: 0.1,
        }
    }
}

/// Score normalization strategy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScoreNormalization {
    /// COR-004: Global min-max normalization to [0, 1] range across all fused results.
    /// Applied after duplicate fusion, before multi-leg boosting.
    MinMax,
    /// No normalization — raw scores from each leg.
    None,
}

/// A result from a single route leg, tagged with its leg index.
#[derive(Debug, Clone)]
pub struct LegResult {
    /// Which leg produced this result.
    pub leg_index: usize,
    /// The search result.
    pub result: SearchResult,
}

/// Merged output from the merge pipeline.
#[derive(Debug, Clone)]
pub struct MergedResults {
    /// Final ordered results.
    pub results: Vec<MergedItem>,
    /// How many duplicate occurrences were fused (not discarded).
    pub duplicates_fused: usize,
    /// Total raw results before merge.
    pub total_raw: usize,
}

/// A single item in the merged result set.
#[derive(Debug, Clone)]
pub struct MergedItem {
    /// The search result (uses the highest-scoring occurrence).
    pub result: SearchResult,
    /// Final score after normalization and boosting.
    pub final_score: f64,
    /// Which leg(s) contributed this result (fused provenance).
    pub source_legs: Vec<usize>,
    /// Per-leg raw scores for transparency.
    pub per_leg_scores: Vec<(usize, f64)>,
}

/// Intermediate accumulator for fusing duplicates.
struct FusedEntry {
    /// Best result (highest raw score).
    best_result: SearchResult,
    /// All legs that produced this result.
    source_legs: Vec<usize>,
    /// Per-leg scores.
    per_leg_scores: Vec<(usize, f64)>,
    /// Best raw score.
    best_score: f64,
}

/// Execute the merge pipeline on results from multiple legs.
pub fn merge(leg_results: Vec<Vec<LegResult>>, policy: &MergePolicy) -> MergedResults {
    // Phase 1: Collect
    let all_results: Vec<LegResult> = leg_results.into_iter().flatten().collect();
    let total_raw = all_results.len();

    // COR-029: Use BTreeMap for deterministic iteration order in hot paths
    let mut fused_map: BTreeMap<String, FusedEntry> = BTreeMap::new();
    let mut insertion_order: Vec<String> = Vec::new();

    for lr in all_results {
        let key = result_identity_key(&lr.result);
        if let Some(entry) = fused_map.get_mut(&key) {
            // Fuse: add this leg's provenance
            if !entry.source_legs.contains(&lr.leg_index) {
                entry.source_legs.push(lr.leg_index);
            }
            entry.per_leg_scores.push((lr.leg_index, lr.result.score));
            if lr.result.score > entry.best_score {
                entry.best_score = lr.result.score;
                entry.best_result = lr.result;
            }
        } else {
            insertion_order.push(key.clone());
            fused_map.insert(
                key,
                FusedEntry {
                    best_score: lr.result.score,
                    per_leg_scores: vec![(lr.leg_index, lr.result.score)],
                    source_legs: vec![lr.leg_index],
                    best_result: lr.result,
                },
            );
        }
    }

    let duplicates_fused = total_raw - fused_map.len();

    // Collect in insertion order for determinism
    let fused: Vec<FusedEntry> = insertion_order
        .into_iter()
        .filter_map(|k| fused_map.remove(&k))
        .collect();

    // Phase 3: Normalize scores
    let items: Vec<MergedItem> = match policy.normalization {
        ScoreNormalization::MinMax => {
            let (min_score, max_score) = fused.iter().fold((f64::MAX, f64::MIN), |(mn, mx), e| {
                (mn.min(e.best_score), mx.max(e.best_score))
            });
            let range = max_score - min_score;

            fused
                .into_iter()
                .map(|e| {
                    let normalized = if range > f64::EPSILON {
                        (e.best_score - min_score) / range
                    } else {
                        1.0
                    };
                    // Phase 4: Boost for multi-leg support
                    let extra_legs = (e.source_legs.len() as f64 - 1.0).max(0.0);
                    let boosted = normalized * (1.0 + extra_legs * policy.multi_leg_boost);

                    MergedItem {
                        result: e.best_result,
                        final_score: boosted,
                        source_legs: e.source_legs,
                        per_leg_scores: e.per_leg_scores,
                    }
                })
                .collect()
        }
        ScoreNormalization::None => fused
            .into_iter()
            .map(|e| {
                let extra_legs = (e.source_legs.len() as f64 - 1.0).max(0.0);
                let boosted = e.best_score * (1.0 + extra_legs * policy.multi_leg_boost);

                MergedItem {
                    result: e.best_result,
                    final_score: boosted,
                    source_legs: e.source_legs,
                    per_leg_scores: e.per_leg_scores,
                }
            })
            .collect(),
    };

    // Phase 5: Rank by final score (descending), with deterministic tie-breaking.
    // Tie-break: more source legs first, then by content identity key (lexicographic).
    let mut ranked = items;
    ranked.sort_by(|a, b| {
        b.final_score
            .partial_cmp(&a.final_score)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| b.source_legs.len().cmp(&a.source_legs.len()))
            .then_with(|| result_identity_key(&a.result).cmp(&result_identity_key(&b.result)))
    });

    // Phase 6: Truncate
    ranked.truncate(policy.limit);

    MergedResults {
        results: ranked,
        duplicates_fused,
        total_raw,
    }
}

/// Generate a dedup key from a search result's source identity.
fn result_identity_key(result: &SearchResult) -> String {
    match &result.source {
        semantic_memory::SearchSource::Fact { fact_id, .. } => format!("fact:{fact_id}"),
        semantic_memory::SearchSource::Chunk { chunk_id, .. } => format!("chunk:{chunk_id}"),
        semantic_memory::SearchSource::Message {
            message_id,
            session_id,
            ..
        } => {
            format!("msg:{session_id}:{message_id}")
        }
        semantic_memory::SearchSource::Episode { episode_id, .. } => {
            format!("episode:{episode_id}")
        }
        semantic_memory::SearchSource::Projection {
            projection_kind,
            projection_id,
            ..
        } => {
            format!("projection:{projection_kind}:{projection_id}")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use semantic_memory::{SearchResult, SearchSource};

    fn make_fact_result(id: &str, score: f64) -> SearchResult {
        SearchResult {
            content: format!("fact {id}"),
            source: SearchSource::Fact {
                fact_id: id.to_string(),
                namespace: "test".to_string(),
            },
            score,
            bm25_rank: None,
            vector_rank: None,
            cosine_similarity: None,
        }
    }

    #[test]
    fn fuses_duplicates_across_legs_with_provenance() {
        let leg0 = vec![LegResult {
            leg_index: 0,
            result: make_fact_result("f1", 0.9),
        }];
        let leg1 = vec![LegResult {
            leg_index: 1,
            result: make_fact_result("f1", 0.8),
        }];

        let merged = merge(vec![leg0, leg1], &MergePolicy::default());
        assert_eq!(merged.results.len(), 1);
        assert_eq!(merged.duplicates_fused, 1);

        // Fused result should have provenance from both legs
        let item = &merged.results[0];
        assert!(item.source_legs.contains(&0));
        assert!(item.source_legs.contains(&1));
        assert_eq!(item.per_leg_scores.len(), 2);

        // Should use the higher score
        assert_eq!(item.result.score, 0.9);
    }

    #[test]
    fn multi_leg_boost_increases_score() {
        let leg0 = vec![LegResult {
            leg_index: 0,
            result: make_fact_result("f1", 0.8),
        }];
        let leg1 = vec![LegResult {
            leg_index: 1,
            result: make_fact_result("f1", 0.7),
        }];
        let single = vec![LegResult {
            leg_index: 0,
            result: make_fact_result("f2", 0.8),
        }];

        let policy = MergePolicy {
            limit: 10,
            normalization: ScoreNormalization::None,
            multi_leg_boost: 0.1,
        };

        let merged = merge(vec![leg0, leg1, single], &policy);
        // f1 (2 legs, score 0.8 * 1.1 = 0.88) should rank above f2 (1 leg, score 0.8)
        assert_eq!(merged.results[0].result.content, "fact f1");
        assert!(merged.results[0].final_score > merged.results[1].final_score);
    }

    #[test]
    fn respects_limit() {
        let leg: Vec<LegResult> = (0..20)
            .map(|i| LegResult {
                leg_index: 0,
                result: make_fact_result(&format!("f{i}"), 1.0 - (i as f64 * 0.01)),
            })
            .collect();

        let policy = MergePolicy {
            limit: 5,
            ..Default::default()
        };
        let merged = merge(vec![leg], &policy);
        assert_eq!(merged.results.len(), 5);
    }

    #[test]
    fn ranks_by_score_descending() {
        let leg = vec![
            LegResult {
                leg_index: 0,
                result: make_fact_result("low", 0.1),
            },
            LegResult {
                leg_index: 0,
                result: make_fact_result("high", 0.9),
            },
        ];

        let merged = merge(vec![leg], &MergePolicy::default());
        assert!(merged.results[0].final_score >= merged.results[1].final_score);
    }

    #[test]
    fn merge_is_deterministic() {
        let make_legs = || {
            vec![
                vec![
                    LegResult {
                        leg_index: 0,
                        result: make_fact_result("a", 0.5),
                    },
                    LegResult {
                        leg_index: 0,
                        result: make_fact_result("b", 0.5),
                    },
                ],
                vec![LegResult {
                    leg_index: 1,
                    result: make_fact_result("a", 0.5),
                }],
            ]
        };

        let r1 = merge(make_legs(), &MergePolicy::default());
        let r2 = merge(make_legs(), &MergePolicy::default());

        assert_eq!(r1.results.len(), r2.results.len());
        for (a, b) in r1.results.iter().zip(r2.results.iter()) {
            assert_eq!(a.result.content, b.result.content);
            assert_eq!(a.source_legs, b.source_legs);
        }
    }
}
