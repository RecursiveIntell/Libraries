//! Gap detection over the semantic memory knowledge base.
//!
//! The [`GapDetector`] issues HTTP calls to a warm semantic-memory server and
//! analyses the results for structural gaps:
//!
//! - **MissingContext** — a fact with no second-order graph relations (isolated
//!   node in the knowledge graph).
//! - **MissingLink** — facts sharing the same namespace but having no graph
//!   edges between them.
//! - **StaleFact** — the server's integrity check reports a problem.
//! - **ContradictionGap** — reserved for future contradiction detection.

use anyhow::{anyhow, Context as _, Result};
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Classification of knowledge-base gaps the detector can surface.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum GapType {
    /// Fact has no graph connections (isolated node).
    MissingContext,
    /// Facts in the same namespace have no edges between them.
    MissingLink,
    /// A fact may be outdated or corrupted.
    StaleFact,
    /// Reserved for future contradiction detection.
    ContradictionGap,
}

impl std::fmt::Display for GapType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingContext => f.write_str("missing-context"),
            Self::MissingLink => f.write_str("missing-link"),
            Self::StaleFact => f.write_str("stale-fact"),
            Self::ContradictionGap => f.write_str("contradiction-gap"),
        }
    }
}

impl GapType {
    /// Parse a gap type from its kebab-case string form.
    pub fn from_kebab(s: &str) -> Option<Self> {
        match s {
            "missing-context" => Some(Self::MissingContext),
            "missing-link" => Some(Self::MissingLink),
            "stale-fact" => Some(Self::StaleFact),
            "contradiction-gap" => Some(Self::ContradictionGap),
            _ => None,
        }
    }
}

/// A single detected gap in the knowledge base.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedGap {
    /// What kind of gap was detected.
    pub gap_type: GapType,
    /// The fact ID (or a descriptive identifier for namespace-level gaps).
    pub fact_id: String,
    /// Human-readable explanation of the gap.
    pub description: String,
    /// Suggested remediation task description.
    pub suggested_task: String,
    /// Priority in `[0.0, 1.0]` — higher is more urgent.
    pub priority: f64,
}

/// Scans the semantic memory knowledge base for structural gaps via HTTP.
#[derive(Debug, Clone)]
pub struct GapDetector {
    http_base_url: String,
}

impl GapDetector {
    /// Create a detector targeting the given semantic-memory HTTP server base URL.
    pub fn new(http_base_url: impl Into<String>) -> Self {
        let mut url = http_base_url.into();
        if url.ends_with('/') {
            url.pop();
        }
        Self { http_base_url: url }
    }

    /// Create a detector targeting the default local server (`http://127.0.0.1:1738`).
    pub fn default_local() -> Self {
        Self::new("http://127.0.0.1:1738")
    }

    /// Run the full gap detection pass and return gaps sorted by priority (desc).
    ///
    /// At most `max_gaps` gaps are returned.
    pub async fn detect_gaps(&self, max_gaps: usize) -> Result<Vec<DetectedGap>> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| anyhow!("failed to build HTTP client: {e}"))?;

        // 1. Broad search to sample facts from the knowledge base.
        let search_results = self.search_facts(&client, 20).await?;

        // 2. Check each fact for graph isolation (MissingContext).
        let mut gaps: Vec<DetectedGap> = Vec::new();
        let mut facts_by_namespace: std::collections::HashMap<String, Vec<&SearchFact>> =
            std::collections::HashMap::new();

        for fact in &search_results {
            // Collect namespace grouping for MissingLink detection.
            if let Some(ns) = &fact.namespace {
                facts_by_namespace.entry(ns.clone()).or_default().push(fact);
            }

            // Check for second-order relations via /discord.
            let has_relations = self
                .check_discord_relations(&client, &fact.id)
                .await
                .unwrap_or(false);

            if !has_relations {
                gaps.push(DetectedGap {
                    gap_type: GapType::MissingContext,
                    fact_id: fact.id.clone(),
                    description: format!(
                        "Fact '{}' has no graph connections (isolated node). Content: {}",
                        fact.id,
                        truncate(&fact.content, 120)
                    ),
                    suggested_task: format!(
                        "Search for concepts related to fact '{}' and add graph edges to connect it.",
                        fact.id
                    ),
                    priority: 0.8,
                });
            }
        }

        // 3. MissingLink: facts in the same namespace with no edges between them.
        for (namespace, facts) in &facts_by_namespace {
            if facts.len() < 2 {
                continue;
            }
            // Check pairs within the same namespace (limit to avoid O(n^2) blowup).
            let max_pairs = 10.min(facts.len() * (facts.len() - 1) / 2);
            let mut checked = 0usize;
            'pair_loop: for i in 0..facts.len() {
                for j in (i + 1)..facts.len() {
                    if checked >= max_pairs {
                        break 'pair_loop;
                    }
                    checked += 1;
                    let connected = self
                        .check_edge_between(&client, &facts[i].id, &facts[j].id)
                        .await
                        .unwrap_or(false);
                    if !connected {
                        gaps.push(DetectedGap {
                            gap_type: GapType::MissingLink,
                            fact_id: format!("{}|{}", facts[i].id, facts[j].id),
                            description: format!(
                                "Facts '{}' and '{}' in namespace '{}' have no graph edge between them.",
                                facts[i].id, facts[j].id, namespace
                            ),
                            suggested_task: format!(
                                "Search for the relationship between facts '{}' and '{}' in namespace '{}'.",
                                facts[i].id, facts[j].id, namespace
                            ),
                            priority: 0.6,
                        });
                        // Only report one MissingLink per namespace to avoid flooding.
                        break 'pair_loop;
                    }
                }
            }
        }

        // 4. StaleFact: check DB integrity.
        let integrity_ok = self.check_integrity(&client).await.unwrap_or(true);
        if !integrity_ok {
            gaps.push(DetectedGap {
                gap_type: GapType::StaleFact,
                fact_id: "db-integrity".to_string(),
                description: "Semantic memory integrity check failed — one or more facts may be \
                    corrupted or stale."
                    .to_string(),
                suggested_task: "Run maintenance reconciliation and verify fact integrity across \
                    the knowledge base."
                    .to_string(),
                priority: 0.9,
            });
        }

        // 5. Sort by priority descending and truncate.
        gaps.sort_by(|a, b| {
            b.priority
                .partial_cmp(&a.priority)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        gaps.truncate(max_gaps);
        Ok(gaps)
    }

    // -- HTTP helpers --------------------------------------------------------

    /// POST /search with a broad query and return parsed fact hits.
    async fn search_facts(&self, client: &reqwest::Client, top_k: usize) -> Result<Vec<SearchFact>> {
        let body = serde_json::json!({
            "query": "knowledge base overview project research",
            "top_k": top_k,
        });
        let url = format!("{}/search", self.http_base_url);
        let resp = client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| anyhow!("POST /search failed: {e}"))?
            .error_for_status()
            .map_err(|e| anyhow!("POST /search returned error status: {e}"))?;
        let raw: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| anyhow!("failed to parse /search response: {e}"))?;

        // The semantic-memory server returns results in a "results" array.
        // Each result has: id, content, namespace, score.
        let results_arr = raw
            .get("results")
            .and_then(|v| v.as_array())
            .context("/search response missing 'results' array")?;

        let facts: Vec<SearchFact> = results_arr
            .iter()
            .filter_map(|item| {
                let id = item.get("id").and_then(|v| v.as_str())?.to_string();
                let content = item
                    .get("content")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let namespace = item
                    .get("namespace")
                    .and_then(|v| v.as_str())
                    .map(String::from);
                Some(SearchFact {
                    id,
                    content,
                    namespace,
                })
            })
            .collect();

        Ok(facts)
    }

    /// POST /discord with a single direct_id to check if the fact has
    /// second-order graph relations. Returns `true` if any related items exist.
    async fn check_discord_relations(
        &self,
        client: &reqwest::Client,
        fact_id: &str,
    ) -> Result<bool> {
        let body = serde_json::json!({
            "direct_result_ids": [fact_id],
        });
        let url = format!("{}/discord", self.http_base_url);
        let resp = client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| anyhow!("POST /discord failed: {e}"))?;

        if !resp.status().is_success() {
            // If discord endpoint fails, assume no relations.
            return Ok(false);
        }

        let raw: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| anyhow!("failed to parse /discord response: {e}"))?;

        // The discord endpoint returns related items. If the array is empty or
        // missing, the fact has no second-order relations.
        let has_related = raw
            .get("related")
            .and_then(|v| v.as_array())
            .map(|arr| !arr.is_empty())
            .unwrap_or(false);

        Ok(has_related)
    }

    /// Check if two facts have a graph edge between them by listing edges for
    /// one and seeing if the other appears.
    async fn check_edge_between(
        &self,
        client: &reqwest::Client,
        fact_a: &str,
        fact_b: &str,
    ) -> Result<bool> {
        // Use the /discord endpoint with both IDs — if they share a relation,
        // discord will surface it.
        let body = serde_json::json!({
            "direct_result_ids": [fact_a, fact_b],
        });
        let url = format!("{}/discord", self.http_base_url);
        let resp = client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| anyhow!("POST /discord (pair) failed: {e}"))?;

        if !resp.status().is_success() {
            return Ok(false);
        }

        let raw: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| anyhow!("failed to parse /discord (pair) response: {e}"))?;

        // If discord returns related items that include either fact, they're connected.
        let related = raw
            .get("related")
            .and_then(|v| v.as_array())
            .map(Vec::len)
            .unwrap_or(0);

        Ok(related > 0)
    }

    /// GET /verify-integrity — returns `true` if integrity check passes.
    async fn check_integrity(&self, client: &reqwest::Client) -> Result<bool> {
        let url = format!("{}/verify-integrity", self.http_base_url);
        let resp = client
            .get(&url)
            .send()
            .await
            .map_err(|e| anyhow!("GET /verify-integrity failed: {e}"))?;

        if !resp.status().is_success() {
            return Ok(false);
        }

        let raw: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| anyhow!("failed to parse /verify-integrity response: {e}"))?;

        // The integrity endpoint returns { "ok": true/false, ... } or
        // { "integrity_ok": true/false, ... }. Check both fields.
        let ok = raw
            .get("ok")
            .and_then(|v| v.as_bool())
            .or_else(|| raw.get("integrity_ok").and_then(|v| v.as_bool()))
            .unwrap_or(true);

        Ok(ok)
    }
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// A parsed search result from the semantic-memory /search endpoint.
#[derive(Debug, Clone)]
struct SearchFact {
    id: String,
    content: String,
    namespace: Option<String>,
}

/// Truncate a string to `max` chars, appending "…" if truncated.
fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        let mut end = max;
        while end > 0 && !s.is_char_boundary(end) {
            end -= 1;
        }
        format!("{}…", &s[..end])
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gap_type_display_roundtrip() {
        for variant in [
            GapType::MissingContext,
            GapType::MissingLink,
            GapType::StaleFact,
            GapType::ContradictionGap,
        ] {
            let s = variant.to_string();
            assert_eq!(GapType::from_kebab(&s), Some(variant));
        }
        assert_eq!(GapType::from_kebab("unknown"), None);
    }

    #[test]
    fn truncate_preserves_short_strings() {
        assert_eq!(truncate("hello", 120), "hello");
        assert_eq!(truncate("hello", 3), "hel…");
        assert_eq!(truncate("", 10), "");
    }

    #[test]
    fn gap_detector_strips_trailing_slash() {
        let d = GapDetector::new("http://localhost:1738/");
        assert_eq!(d.http_base_url, "http://localhost:1738");
    }

    #[test]
    fn gap_detector_default_local() {
        let d = GapDetector::default_local();
        assert_eq!(d.http_base_url, "http://127.0.0.1:1738");
    }

    /// Parse the canned /search response and verify SearchFact extraction logic.
    #[test]
    fn parse_search_response() {
        let canned = serde_json::json!({
            "results": [
                {
                    "id": "fact:aaa",
                    "content": "Rust is a systems programming language.",
                    "namespace": "general",
                    "score": 0.95
                },
                {
                    "id": "fact:bbb",
                    "content": "Tokio is an async runtime.",
                    "namespace": "coding",
                    "score": 0.82
                },
                {
                    "id": "fact:ccc",
                    "content": "Semantic memory stores facts.",
                    "score": 0.71
                }
            ]
        });

        let results_arr = canned
            .get("results")
            .and_then(|v| v.as_array())
            .expect("results array");

        let facts: Vec<SearchFact> = results_arr
            .iter()
            .filter_map(|item| {
                let id = item.get("id").and_then(|v| v.as_str())?.to_string();
                let content = item
                    .get("content")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let namespace = item
                    .get("namespace")
                    .and_then(|v| v.as_str())
                    .map(String::from);
                Some(SearchFact {
                    id,
                    content,
                    namespace,
                })
            })
            .collect();

        assert_eq!(facts.len(), 3);
        assert_eq!(facts[0].id, "fact:aaa");
        assert_eq!(facts[0].namespace.as_deref(), Some("general"));
        assert_eq!(facts[2].namespace, None);
    }

    /// Verify the MissingContext gap construction logic.
    #[test]
    fn missing_context_gap_construction() {
        let gap = DetectedGap {
            gap_type: GapType::MissingContext,
            fact_id: "fact:abc123".to_string(),
            description: "Fact has no graph connections.".to_string(),
            suggested_task: "Search for related concepts.".to_string(),
            priority: 0.8,
        };
        assert_eq!(gap.gap_type, GapType::MissingContext);
        assert!((gap.priority - 0.8).abs() < f64::EPSILON);
    }

    /// Verify the StaleFact gap construction logic.
    #[test]
    fn stale_fact_gap_construction() {
        let gap = DetectedGap {
            gap_type: GapType::StaleFact,
            fact_id: "db-integrity".to_string(),
            description: "Integrity check failed.".to_string(),
            suggested_task: "Run reconciliation.".to_string(),
            priority: 0.9,
        };
        assert_eq!(gap.gap_type, GapType::StaleFact);
        assert!((gap.priority - 0.9).abs() < f64::EPSILON);
    }

    /// Verify that gaps sort by priority descending.
    #[test]
    fn gaps_sort_by_priority_desc() {
        let mut gaps = vec![
            DetectedGap {
                gap_type: GapType::MissingLink,
                fact_id: "a".to_string(),
                description: "low".to_string(),
                suggested_task: "task".to_string(),
                priority: 0.6,
            },
            DetectedGap {
                gap_type: GapType::StaleFact,
                fact_id: "b".to_string(),
                description: "high".to_string(),
                suggested_task: "task".to_string(),
                priority: 0.9,
            },
            DetectedGap {
                gap_type: GapType::MissingContext,
                fact_id: "c".to_string(),
                description: "mid".to_string(),
                suggested_task: "task".to_string(),
                priority: 0.8,
            },
        ];
        gaps.sort_by(|a, b| {
            b.priority
                .partial_cmp(&a.priority)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        assert_eq!(gaps[0].gap_type, GapType::StaleFact);
        assert_eq!(gaps[1].gap_type, GapType::MissingContext);
        assert_eq!(gaps[2].gap_type, GapType::MissingLink);
    }

    /// Verify the /discord response parsing logic for relation detection.
    #[test]
    fn discord_response_parsing() {
        // Case 1: has related items → connected.
        let with_relations = serde_json::json!({
            "related": [
                {"id": "fact:xyz", "content": "related fact", "score": 0.5}
            ]
        });
        let has_related = with_relations
            .get("related")
            .and_then(|v| v.as_array())
            .map(|arr| !arr.is_empty())
            .unwrap_or(false);
        assert!(has_related);

        // Case 2: empty related → not connected.
        let without_relations = serde_json::json!({ "related": [] });
        let has_related = without_relations
            .get("related")
            .and_then(|v| v.as_array())
            .map(|arr| !arr.is_empty())
            .unwrap_or(false);
        assert!(!has_related);

        // Case 3: missing field → not connected (safe default).
        let missing_field = serde_json::json!({ "error": "something" });
        let has_related = missing_field
            .get("related")
            .and_then(|v| v.as_array())
            .map(|arr| !arr.is_empty())
            .unwrap_or(false);
        assert!(!has_related);
    }

    /// Verify the /verify-integrity response parsing logic.
    #[test]
    fn integrity_response_parsing() {
        // ok=true
        let ok_resp = serde_json::json!({ "ok": true, "details": "all good" });
        let ok = ok_resp
            .get("ok")
            .and_then(|v| v.as_bool())
            .or_else(|| ok_resp.get("integrity_ok").and_then(|v| v.as_bool()))
            .unwrap_or(true);
        assert!(ok);

        // integrity_ok=false
        let bad_resp = serde_json::json!({ "integrity_ok": false, "error": "corruption" });
        let ok = bad_resp
            .get("ok")
            .and_then(|v| v.as_bool())
            .or_else(|| bad_resp.get("integrity_ok").and_then(|v| v.as_bool()))
            .unwrap_or(true);
        assert!(!ok);

        // missing field → default true (optimistic)
        let empty_resp = serde_json::json!({ "status": "unknown" });
        let ok = empty_resp
            .get("ok")
            .and_then(|v| v.as_bool())
            .or_else(|| empty_resp.get("integrity_ok").and_then(|v| v.as_bool()))
            .unwrap_or(true);
        assert!(ok);
    }

    /// Verify the MissingLink namespace grouping logic.
    #[test]
    fn namespace_grouping_for_missing_link() {
        let facts = vec![
            SearchFact {
                id: "fact:1".to_string(),
                content: "first".to_string(),
                namespace: Some("general".to_string()),
            },
            SearchFact {
                id: "fact:2".to_string(),
                content: "second".to_string(),
                namespace: Some("general".to_string()),
            },
            SearchFact {
                id: "fact:3".to_string(),
                content: "third".to_string(),
                namespace: Some("coding".to_string()),
            },
        ];

        let mut groups: std::collections::HashMap<String, Vec<&SearchFact>> =
            std::collections::HashMap::new();
        for fact in &facts {
            if let Some(ns) = &fact.namespace {
                groups.entry(ns.clone()).or_default().push(fact);
            }
        }

        assert_eq!(groups.get("general").map(|v| v.len()), Some(2));
        assert_eq!(groups.get("coding").map(|v| v.len()), Some(1));
        // "general" has 2 facts → candidate for MissingLink.
        // "coding" has 1 fact → not a candidate.
    }
}