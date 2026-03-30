use serde::{Deserialize, Serialize};

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
}

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
    if let Some(mention) = extract_entity_mention(query) {
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
fn extract_entity_mention(query: &str) -> Option<String> {
    // @-mention: first @-prefixed word
    for word in query.split_whitespace() {
        if let Some(name) = word.strip_prefix('@') {
            if !name.is_empty() {
                return Some(name.to_string());
            }
        }
    }

    // Quoted string: first double-quoted substring
    if let Some(start) = query.find('"') {
        if let Some(end) = query[start + 1..].find('"') {
            let mention = &query[start + 1..start + 1 + end];
            if !mention.is_empty() {
                return Some(mention.to_string());
            }
        }
    }

    None
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

    for marker in &temporal_markers {
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
            other => panic!("expected EntityLookup, got {:?}", other),
        }
    }

    #[test]
    fn quoted_string_triggers_entity() {
        let result = classify("find \"MemoryStore\" usage");
        match &result.mode {
            QueryMode::EntityLookup { mention } => assert_eq!(mention, "MemoryStore"),
            other => panic!("expected EntityLookup, got {:?}", other),
        }
    }

    #[test]
    fn temporal_keyword_triggers_temporal() {
        let result = classify("what changed last week");
        match &result.mode {
            QueryMode::TemporalLookup { temporal_expr } => {
                assert_eq!(temporal_expr, "last week");
            }
            other => panic!("expected TemporalLookup, got {:?}", other),
        }
    }

    #[test]
    fn mixed_entity_and_temporal() {
        let result = classify("what did @alice do last week");
        assert_eq!(result.mode.kind(), "mixed");
    }
}
