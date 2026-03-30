use semantic_memory::SearchResult;
use serde::{Deserialize, Serialize};

/// A bundle of supporting evidence assembled from search results.
///
/// A `SearchEvidenceBundle` gathers the results that back an answer or
/// projection. It does NOT contain the answer itself — it is the
/// "show your work" component.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchEvidenceBundle {
    /// Unique bundle identifier.
    pub id: String,
    /// Supporting search results, ordered by relevance.
    pub items: Vec<EvidenceItem>,
    /// Overall confidence across all items (0.0 to 1.0).
    pub aggregate_confidence: f32,
}

/// A single evidence item derived from a search result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceItem {
    /// Content of the evidence.
    pub content: String,
    /// Which semantic-memory source produced this.
    pub source: semantic_memory::SearchSource,
    /// Relevance score from the search pipeline.
    pub score: f64,
    /// How this evidence relates to the query.
    pub relevance: EvidenceRelevance,
}

/// How a piece of evidence relates to the query.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceRelevance {
    /// Directly answers the query.
    Direct,
    /// Provides supporting context.
    Supporting,
    /// Provides contrasting/refuting information.
    Contrasting,
    /// Tangentially related.
    Tangential,
}

/// An assembled answer bundle with evidence.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnswerBundle {
    /// The answer or summary text (assembled by the caller, not by this crate).
    pub answer: String,
    /// Evidence backing the answer.
    pub evidence: SearchEvidenceBundle,
    /// Confidence in the answer given the evidence (0.0 to 1.0).
    pub confidence: f32,
}

/// Build a `SearchEvidenceBundle` from search results.
///
/// All items are initially tagged as `Direct` relevance. Callers should
/// reclassify relevance as appropriate for their use case.
pub fn bundle_from_results(results: &[SearchResult]) -> SearchEvidenceBundle {
    let items: Vec<EvidenceItem> = results
        .iter()
        .map(|r| EvidenceItem {
            content: r.content.clone(),
            source: r.source.clone(),
            score: r.score,
            relevance: EvidenceRelevance::Direct,
        })
        .collect();

    let aggregate_confidence = if items.is_empty() {
        0.0
    } else {
        let sum: f64 = items.iter().map(|i| i.score).sum();
        (sum / items.len() as f64).min(1.0) as f32
    };

    SearchEvidenceBundle {
        id: uuid::Uuid::new_v4().to_string(),
        items,
        aggregate_confidence,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use semantic_memory::{SearchResult, SearchSource};

    #[test]
    fn bundle_from_empty_results() {
        let bundle = bundle_from_results(&[]);
        assert!(bundle.items.is_empty());
        assert_eq!(bundle.aggregate_confidence, 0.0);
    }

    #[test]
    fn bundle_preserves_order() {
        let results = vec![
            SearchResult {
                content: "first".into(),
                source: SearchSource::Fact {
                    fact_id: "f1".into(),
                    namespace: "test".into(),
                },
                score: 0.9,
                bm25_rank: None,
                vector_rank: None,
                cosine_similarity: None,
            },
            SearchResult {
                content: "second".into(),
                source: SearchSource::Fact {
                    fact_id: "f2".into(),
                    namespace: "test".into(),
                },
                score: 0.7,
                bm25_rank: None,
                vector_rank: None,
                cosine_similarity: None,
            },
        ];
        let bundle = bundle_from_results(&results);
        assert_eq!(bundle.items.len(), 2);
        assert_eq!(bundle.items[0].content, "first");
        assert_eq!(bundle.items[1].content, "second");
    }
}
