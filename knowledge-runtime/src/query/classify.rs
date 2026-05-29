//! Rule-based intent classification for incoming queries.
//!
//! Maps raw query text to a `QueryMode` that drives route planning.

use serde::{Deserialize, Serialize};

/// Parsing error for query mode operations (test-only).
#[cfg(test)]
use thiserror::Error;

/// Intent classification for an incoming query.
///
/// The classifier maps raw query text to a `QueryMode` that determines
/// which route legs to plan. This is a rule-based heuristic; LLM-based
/// classification is a future extension point.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QueryMode {
    /// Pure semantic / free-text similarity search.
    SemanticLookup,
    /// The query names a specific entity (person, function, file, etc.).
    EntityLookup {
        /// The raw entity mention extracted from the query.
        mention: String,
    },
    /// The query involves time constraints ("last week", "before v2.0").
    TemporalLookup {
        /// Raw temporal expression extracted from the query.
        temporal_expr: String,
    },
    /// Multiple intents detected — will fan out to multiple route legs.
    Mixed {
        /// Component intents.
        components: Vec<QueryMode>,
    },
}

impl QueryMode {
    /// Stable discriminant string.
    pub fn kind(&self) -> &'static str {
        match self {
            Self::SemanticLookup => "semantic",
            Self::EntityLookup { .. } => "entity",
            Self::TemporalLookup { .. } => "temporal",
            Self::Mixed { .. } => "mixed",
        }
    }

    /// Parse a `QueryMode` from a kind string.
    ///
    /// Returns `None` for unknown kinds — this is the correct fallback for
    /// unknown variant names (the `unreachable!` sites were never reachable in
    /// practice; this makes the exhaustive match non-trapping).
    pub fn from_kind(kind: &str) -> Option<Self> {
        match kind {
            "semantic" => Some(Self::SemanticLookup),
            "entity" => None, // EntityLookup needs a mention — not reconstructable from kind alone
            "temporal" => None, // TemporalLookup needs a temporal_expr
            "mixed" => None,  // Mixed needs components
            _ => None,
        }
    }
}

#[cfg(test)]
impl std::fmt::Display for QueryMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[cfg(test)]
impl std::error::Error for QueryMode {}

/// Classification result with confidence metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassifyResult {
    /// Determined query mode.
    pub mode: QueryMode,
    /// Confidence in the classification (0.0 to 1.0).
    /// A value of 1.0 means the classifier is certain (e.g. exact entity match).
    pub confidence: f32,
    /// Optional reasoning for observability.
    pub reason: Option<String>,
}

/// Classify a raw query string into a `QueryMode`.
///
/// Current implementation uses simple heuristics:
/// - Quoted strings or `@mentions` → `EntityLookup`
/// - Temporal keywords → `TemporalLookup`
/// - Multiple signals → `Mixed`
/// - Default → `SemanticLookup`
pub fn classify(query: &str) -> ClassifyResult {
    let mut components = Vec::new();

    // Check for entity mentions: quoted strings or @-prefixed tokens.
    for mention in extract_entity_mentions(query) {
        components.push(QueryMode::EntityLookup { mention });
    }

    // Check for temporal expressions.
    if let Some(temporal_expr) = extract_temporal_expr(query) {
        components.push(QueryMode::TemporalLookup { temporal_expr });
    }

    match components.len() {
        0 => ClassifyResult {
            mode: QueryMode::SemanticLookup,
            confidence: 0.8,
            reason: Some("no entity or temporal signals detected".into()),
        },
        1 => {
            if let Some(mode) = components.pop() {
                ClassifyResult {
                    confidence: 0.7,
                    reason: Some(format!("single signal: {}", mode.kind())),
                    mode,
                }
            } else {
                ClassifyResult {
                    mode: QueryMode::SemanticLookup,
                    confidence: 0.8,
                    reason: Some("no entity or temporal signals detected".into()),
                }
            }
        }
        _ => ClassifyResult {
            mode: QueryMode::Mixed { components },
            confidence: 0.6,
            reason: Some("multiple signal types detected".into()),
        },
    }
}

/// Extract an entity mention from a query.
///
/// Recognizes:
/// - `@name` tokens
/// - `"quoted strings"`
fn extract_entity_mentions(query: &str) -> Vec<String> {
    let mut mentions = Vec::new();

    // @-mention: first @-prefixed word
    for word in query.split_whitespace() {
        if let Some(name) = word.strip_prefix('@') {
            if !name.is_empty() {
                mentions.push(
                    name.trim_matches(|c: char| !c.is_alphanumeric() && c != '_' && c != '-')
                        .to_string(),
                );
            }
        }
    }

    // Quoted strings, allowing escaped quotes.
    let mut in_quote = false;
    let mut escaped = false;
    let mut current = String::new();
    for ch in query.chars() {
        if escaped {
            if in_quote {
                current.push(ch);
            }
            escaped = false;
            continue;
        }
        if ch == '\\' {
            escaped = true;
            continue;
        }
        if ch == '"' {
            if in_quote && !current.is_empty() {
                mentions.push(std::mem::take(&mut current));
            } else {
                current.clear();
            }
            in_quote = !in_quote;
            continue;
        }
        if in_quote {
            current.push(ch);
        }
    }

    mentions.retain(|mention| !mention.is_empty());
    mentions.sort();
    mentions.dedup();
    mentions
}

/// Extract a temporal expression from a query.
///
/// Recognizes common temporal keywords. Full NLP temporal parsing is deferred.
fn extract_temporal_expr(query: &str) -> Option<String> {
    let lower = query.to_lowercase();
    let temporal_markers = [
        "yesterday",
        "last week",
        "last month",
        "last year",
        "today",
        "this week",
        "this month",
        "before",
        "after",
        "since",
        "between",
        "recent",
        "recently",
        "latest",
        "oldest",
        "earlier",
    ];

    let mut markers = temporal_markers.to_vec();
    markers.sort_by_key(|marker| std::cmp::Reverse(marker.len()));
    for marker in markers {
        if lower.contains(marker) {
            return Some(marker.to_string());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pure_semantic_query() {
        let result = classify("how does the search pipeline work");
        assert_eq!(result.mode.kind(), "semantic");
    }

    #[test]
    fn at_mention_triggers_entity() {
        let result = classify("what does @reembed_all do");
        match &result.mode {
            QueryMode::EntityLookup { mention } => assert_eq!(mention, "reembed_all"),
            other => unreachable!("QueryMode::EntityLookup expected, got {other:?}"),
        }
    }

    #[test]
    fn quoted_string_triggers_entity() {
        let result = classify("find \"MemoryStore\" usage");
        match &result.mode {
            QueryMode::EntityLookup { mention } => assert_eq!(mention, "MemoryStore"),
            other => unreachable!("QueryMode::EntityLookup expected, got {other:?}"),
        }
    }

    #[test]
    fn temporal_keyword_triggers_temporal() {
        let result = classify("what changed last week");
        match &result.mode {
            QueryMode::TemporalLookup { temporal_expr } => {
                assert_eq!(temporal_expr, "last week");
            }
            other => unreachable!("QueryMode::TemporalLookup expected, got {other:?}"),
        }
    }

    #[test]
    fn mixed_entity_and_temporal() {
        let result = classify("what did @alice do last week");
        assert_eq!(result.mode.kind(), "mixed");
    }

    #[test]
    fn multiple_entity_mentions_are_preserved() {
        let result = classify(r#"compare @alice and @bob with "MemoryStore""#);
        match &result.mode {
            QueryMode::Mixed { components } => {
                let mentions: Vec<_> = components
                    .iter()
                    .filter_map(|component| match component {
                        QueryMode::EntityLookup { mention } => Some(mention.as_str()),
                        _ => None,
                    })
                    .collect();
                assert_eq!(mentions, vec!["MemoryStore", "alice", "bob"]);
            }
            other => unreachable!("QueryMode::Mixed expected, got {other:?}"),
        }
    }

    #[test]
    fn temporal_marker_prefers_more_specific_phrase() {
        let result = classify("summarize last week and recent changes");
        match &result.mode {
            QueryMode::TemporalLookup { temporal_expr } => {
                assert_eq!(temporal_expr, "last week");
            }
            other => unreachable!("QueryMode::TemporalLookup expected, got {other:?}"),
        }
    }
}
