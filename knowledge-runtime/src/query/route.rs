//! Route planning: translates classified query modes into retrieval legs.

use crate::ids::Scope;
use crate::query::classify::QueryMode;
use serde::{Deserialize, Serialize};

/// A typed, inspectable query plan.
///
/// The planner translates a classified `QueryMode` into a sequence of
/// `RouteLeg`s, each of which targets a specific retrieval strategy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutePlan {
    /// Original query text.
    pub query: String,
    /// Scope for all legs.
    pub scope: Scope,
    /// Ordered retrieval legs.
    pub legs: Vec<RouteLeg>,
}

/// A single retrieval leg within a route plan.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteLeg {
    /// What this leg retrieves.
    pub strategy: RetrievalStrategy,
    /// Maximum results for this leg.
    pub limit: usize,
    /// Optional filter applied to this leg.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter: Option<LegFilter>,
}

/// Retrieval strategy for a route leg.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RetrievalStrategy {
    /// Hybrid BM25 + vector search via `semantic-memory`.
    HybridSearch,
    /// Entity registry lookup followed by targeted search.
    EntitySearch {
        /// The entity mention to resolve.
        mention: String,
    },
    /// Time-bounded search.
    TemporalSearch {
        /// Raw temporal expression (resolved at execution time).
        temporal_expr: String,
    },
}

impl RetrievalStrategy {
    /// Stable discriminant string for logging and provenance records.
    pub fn kind(&self) -> &'static str {
        match self {
            Self::HybridSearch => "hybrid",
            Self::EntitySearch { .. } => "entity",
            Self::TemporalSearch { .. } => "temporal",
        }
    }
}

/// Optional filter narrowing a route leg.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegFilter {
    /// Restrict to specific source types.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_types: Option<Vec<String>>,
    /// Restrict to a specific namespace.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub namespace: Option<String>,
}

/// Plan a route from a classified query mode.
pub fn plan(query: &str, mode: &QueryMode, scope: &Scope, default_limit: usize) -> RoutePlan {
    let legs = match mode {
        QueryMode::SemanticLookup => vec![RouteLeg {
            strategy: RetrievalStrategy::HybridSearch,
            limit: default_limit,
            filter: None,
        }],
        QueryMode::EntityLookup { mention } => vec![RouteLeg {
            strategy: RetrievalStrategy::EntitySearch {
                mention: mention.clone(),
            },
            limit: default_limit,
            filter: None,
        }],
        QueryMode::TemporalLookup { temporal_expr } => vec![RouteLeg {
            strategy: RetrievalStrategy::TemporalSearch {
                temporal_expr: temporal_expr.clone(),
            },
            limit: default_limit,
            filter: None,
        }],
        QueryMode::Mixed { components } => components
            .iter()
            .map(|c| match c {
                QueryMode::SemanticLookup => RouteLeg {
                    strategy: RetrievalStrategy::HybridSearch,
                    limit: default_limit,
                    filter: None,
                },
                QueryMode::EntityLookup { mention } => RouteLeg {
                    strategy: RetrievalStrategy::EntitySearch {
                        mention: mention.clone(),
                    },
                    limit: default_limit,
                    filter: None,
                },
                QueryMode::TemporalLookup { temporal_expr } => RouteLeg {
                    strategy: RetrievalStrategy::TemporalSearch {
                        temporal_expr: temporal_expr.clone(),
                    },
                    limit: default_limit,
                    filter: None,
                },
                QueryMode::Mixed { .. } => RouteLeg {
                    strategy: RetrievalStrategy::HybridSearch,
                    limit: default_limit,
                    filter: None,
                },
            })
            .collect(),
    };

    RoutePlan {
        query: query.to_string(),
        scope: scope.clone(),
        legs,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn semantic_produces_one_hybrid_leg() {
        let scope = Scope::new("test");
        let plan = plan("hello", &QueryMode::SemanticLookup, &scope, 10);
        assert_eq!(plan.legs.len(), 1);
        assert_eq!(plan.legs[0].strategy.kind(), "hybrid");
    }

    #[test]
    fn mixed_produces_multiple_legs() {
        let scope = Scope::new("test");
        let mode = QueryMode::Mixed {
            components: vec![
                QueryMode::EntityLookup {
                    mention: "foo".into(),
                },
                QueryMode::TemporalLookup {
                    temporal_expr: "last week".into(),
                },
            ],
        };
        let plan = plan("find foo from last week", &mode, &scope, 10);
        assert_eq!(plan.legs.len(), 2);
        assert_eq!(plan.legs[0].strategy.kind(), "entity");
        assert_eq!(plan.legs[1].strategy.kind(), "temporal");
    }
}
