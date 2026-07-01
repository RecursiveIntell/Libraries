//! Safe Cypher query extraction from LLM responses.
//!
//! Provides [`parse_cypher_block`] for extracting read-only Cypher queries,
//! rejecting any response that contains write or unsafe clause keywords.

use crate::error::{ParseError, ParseOptions};
use crate::extract::preprocess_opts;

const UNSAFE_KEYWORDS: &[&str] = &[
    "CREATE", "MERGE", "DELETE", "SET", "DROP", "REMOVE", "CALL", "LOAD CSV",
];

/// Extract a read-only Cypher query from an LLM response.
///
/// Prefers a fenced ` ```cypher ... ``` ` block when present; falls back to
/// the trimmed response text.
///
/// # Errors
///
/// - [`ParseError::EmptyResponse`] — nothing remains after extraction and trim.
/// - [`ParseError::Unparseable`] — the query contains a write or unsafe keyword
///   (`CREATE`, `MERGE`, `DELETE`, `SET`, `DROP`, `REMOVE`, `CALL`, `LOAD CSV`).
///
/// # Examples
///
/// ```
/// use llm_output_parser::parse_cypher_block;
///
/// let ok = parse_cypher_block("```cypher\nMATCH (n) RETURN n\n```").unwrap();
/// assert_eq!(ok, "MATCH (n) RETURN n");
///
/// let err = parse_cypher_block("CREATE (n:Person)");
/// assert!(err.is_err());
/// ```
pub fn parse_cypher_block(response: &str) -> Result<String, ParseError> {
    let opts = ParseOptions::default();
    let cleaned = preprocess_opts(response, opts.strip_think_tags);

    let query = extract_fenced(&cleaned).unwrap_or_else(|| cleaned.trim().to_string());

    if query.is_empty() {
        return Err(ParseError::EmptyResponse);
    }

    let upper = query.to_uppercase();
    for kw in UNSAFE_KEYWORDS {
        if upper.contains(kw) {
            return Err(ParseError::Unparseable {
                expected_format: "read-only cypher",
                text: truncate_200(&query),
            });
        }
    }

    Ok(query)
}

fn extract_fenced(s: &str) -> Option<String> {
    let lower = s.to_ascii_lowercase();
    let fence_start = lower.find("```cypher")?;
    let after_open = &s[fence_start + "```cypher".len()..];
    let body_start = after_open.find('\n').map(|p| p + 1).unwrap_or(0);
    let body = &after_open[body_start..];
    let close = body.find("```")?;
    Some(body[..close].trim().to_string())
}

fn truncate_200(s: &str) -> String {
    if s.chars().count() <= 200 {
        s.to_string()
    } else {
        format!("{}...", s.chars().take(200).collect::<String>())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cypher_extracts_fenced_block() {
        let input = "```cypher\nMATCH (n) RETURN n\n```";
        let result = parse_cypher_block(input).unwrap();
        assert_eq!(result, "MATCH (n) RETURN n");
    }

    #[test]
    fn cypher_falls_back_to_plain_text() {
        let input = "MATCH (n) WHERE n.age > 18 RETURN n LIMIT 10";
        let result = parse_cypher_block(input).unwrap();
        assert_eq!(result, input);
    }

    #[test]
    fn cypher_rejects_create() {
        let err = parse_cypher_block("CREATE (n:Person {name: 'Alice'})").unwrap_err();
        assert_eq!(err.kind(), "unparseable");
    }

    #[test]
    fn cypher_rejects_merge() {
        let err = parse_cypher_block("MERGE (n:Person {name: 'Bob'})").unwrap_err();
        assert_eq!(err.kind(), "unparseable");
    }

    #[test]
    fn cypher_rejects_delete() {
        let err = parse_cypher_block("MATCH (n) DELETE n").unwrap_err();
        assert_eq!(err.kind(), "unparseable");
    }

    #[test]
    fn cypher_rejects_set() {
        let err = parse_cypher_block("MATCH (n) SET n.x = 1").unwrap_err();
        assert_eq!(err.kind(), "unparseable");
    }

    #[test]
    fn cypher_rejects_drop() {
        let err = parse_cypher_block("DROP INDEX ON :Person(name)").unwrap_err();
        assert_eq!(err.kind(), "unparseable");
    }

    #[test]
    fn cypher_rejects_remove() {
        let err = parse_cypher_block("MATCH (n) REMOVE n.age").unwrap_err();
        assert_eq!(err.kind(), "unparseable");
    }

    #[test]
    fn cypher_rejects_call() {
        let err = parse_cypher_block("CALL db.indexes()").unwrap_err();
        assert_eq!(err.kind(), "unparseable");
    }

    #[test]
    fn cypher_rejects_load_csv() {
        let err = parse_cypher_block("LOAD CSV FROM 'file.csv' AS row").unwrap_err();
        assert_eq!(err.kind(), "unparseable");
    }

    #[test]
    fn cypher_rejects_empty() {
        let err = parse_cypher_block("   ").unwrap_err();
        assert_eq!(err.kind(), "empty_response");
    }

    #[test]
    fn cypher_rejects_empty_fenced_block() {
        let err = parse_cypher_block("```cypher\n   \n```").unwrap_err();
        assert_eq!(err.kind(), "empty_response");
    }

    #[test]
    fn cypher_accepts_lowercase_match_return() {
        let input = "match (n) where n.age > 5 return n limit 5";
        let result = parse_cypher_block(input).unwrap();
        assert_eq!(result, input);
    }

    #[test]
    fn cypher_rejects_lowercase_create() {
        let err = parse_cypher_block("create (n:Foo)").unwrap_err();
        assert_eq!(err.kind(), "unparseable");
    }
}
