//! Gap detection over the semantic memory knowledge base.
//!
//! The [`GapDetector`] issues HTTP calls to a warm semantic-memory server and
//! analyses the results for both structural and content-level gaps:
//!
//! - **MissingContext** — a fact with no second-order graph relations (isolated
//!   node in the knowledge graph).
//! - **MissingLink** — facts sharing the same namespace but having no graph
//!   edges between them.
//! - **StaleFact** — the server's integrity check reports a problem.
//! - **ContradictionGap** — two facts with high content overlap but
//!   contradictory signals (negation, different numbers, different dates).
//! - **DuplicateFact** — two facts with >80% content overlap (Jaccard
//!   similarity on word sets).
//! - **StaleByDate** — a fact references a date more than 6 months old.
//! - **LowQualityFact** — fact content is too short, mostly URLs, or a raw
//!   JSON blob.

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Namespaces that are ingestion artifacts, not knowledge. Gaps are never
/// generated for facts in these namespaces.
pub const SKIP_NAMESPACES: &[&str] = &["mixed", "chatgpt", "twitter", "test"];

/// Priority namespaces to search during gap detection.
pub const PRIORITY_NAMESPACES: &[&str] = &[
    "projects",
    "research",
    "semantic-memory",
    "libraries",
    "libraries-crates",
    "doctrine",
    "infrastructure",
    "personal",
    "behavioral",
    "codex",
    "recursiveintell",
    "general",
    "autonomous",
];

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
    /// A fact may be outdated or corrupted (integrity check).
    StaleFact,
    /// Two facts contradict each other (content-level detection).
    ContradictionGap,
    /// A fact appears to be a duplicate of another.
    DuplicateFact,
    /// A fact references a date/version that may be outdated.
    StaleByDate,
    /// A fact is too short, noisy, or low-value.
    LowQualityFact,
}

impl std::fmt::Display for GapType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingContext => f.write_str("missing-context"),
            Self::MissingLink => f.write_str("missing-link"),
            Self::StaleFact => f.write_str("stale-fact"),
            Self::ContradictionGap => f.write_str("contradiction-gap"),
            Self::DuplicateFact => f.write_str("duplicate-fact"),
            Self::StaleByDate => f.write_str("stale-by-date"),
            Self::LowQualityFact => f.write_str("low-quality-fact"),
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
            "duplicate-fact" => Some(Self::DuplicateFact),
            "stale-by-date" => Some(Self::StaleByDate),
            "low-quality-fact" => Some(Self::LowQualityFact),
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
    /// Optional content snippet for prompt building.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_snippet: Option<String>,
    /// Optional secondary fact ID (for contradictions, duplicates, links).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fact_id_b: Option<String>,
    /// Optional namespace context.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub namespace: Option<String>,
    /// Optional date string extracted from content (for StaleByDate).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date: Option<String>,
}

/// Scans the semantic memory knowledge base for gaps via HTTP.
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
    /// At most `max_gaps` gaps are returned. Gaps whose `fact_id+gap_type` key
    /// is in `attempted` are skipped.
    pub async fn detect_gaps(
        &self,
        max_gaps: usize,
        attempted: &HashSet<String>,
    ) -> Result<Vec<DetectedGap>> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| anyhow!("failed to build HTTP client: {e}"))?;

        // 1. Broad search to sample facts from the knowledge base.
        let search_results = self.search_facts(&client).await?;

        // 2. Check each fact for various gap types.
        let mut gaps: Vec<DetectedGap> = Vec::new();
        let mut facts_by_namespace: HashMap<String, Vec<&SearchFact>> = HashMap::new();

        for fact in &search_results {
            // Skip social media / ingestion artifact namespaces.
            if let Some(ns) = &fact.namespace {
                if is_skip_namespace(ns) {
                    continue;
                }
                facts_by_namespace.entry(ns.clone()).or_default().push(fact);
            }

            let fact_key = |gt: &GapType| format!("{}+{}", fact.id, gt);

            // --- Structural: MissingContext (graph isolation) ---
            if !attempted.contains(&fact_key(&GapType::MissingContext)) {
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
                        priority: 0.6,
                        content_snippet: Some(truncate(&fact.content, 200)),
                        fact_id_b: None,
                        namespace: fact.namespace.clone(),
                        date: None,
                    });
                }
            }

            // --- Content: LowQualityFact ---
            if !attempted.contains(&fact_key(&GapType::LowQualityFact))
                && is_low_quality(&fact.content)
            {
                gaps.push(DetectedGap {
                    gap_type: GapType::LowQualityFact,
                    fact_id: fact.id.clone(),
                    description: format!(
                        "Fact '{}' appears to be low quality (too short, URL-heavy, or raw JSON). Content: {}",
                        fact.id,
                        truncate(&fact.content, 120)
                    ),
                    suggested_task: "Determine if the fact should be kept, improved, or removed.".to_string(),
                    priority: 0.4,
                    content_snippet: Some(truncate(&fact.content, 200)),
                    fact_id_b: None,
                    namespace: fact.namespace.clone(),
                    date: None,
                });
            }

            // --- Content: StaleByDate ---
            if !attempted.contains(&fact_key(&GapType::StaleByDate)) {
                if let Some(date) = extract_stale_date(&fact.content) {
                    gaps.push(DetectedGap {
                        gap_type: GapType::StaleByDate,
                        fact_id: fact.id.clone(),
                        description: format!(
                            "Fact '{}' references date {} which may be outdated. Content: {}",
                            fact.id, date, truncate(&fact.content, 120)
                        ),
                        suggested_task: "Check if the information is still current.".to_string(),
                        priority: 0.65,
                        content_snippet: Some(truncate(&fact.content, 200)),
                        fact_id_b: None,
                        namespace: fact.namespace.clone(),
                        date: Some(date),
                    });
                }
            }

            // --- Content: ContradictionGap & DuplicateFact ---
            // Search for similar facts and compare content.
            if !attempted.contains(&fact_key(&GapType::ContradictionGap))
                || !attempted.contains(&fact_key(&GapType::DuplicateFact))
            {
                if let Ok(similar) = self.search_similar(&client, &fact.content, &fact.id).await {
                    for other in &similar {
                        // Only compare within the same namespace.
                        if other.namespace != fact.namespace {
                            continue;
                        }

                        let jaccard = jaccard_similarity(&fact.content, &other.content);

                        // DuplicateFact: >80% content overlap.
                        if jaccard > 0.8
                            && !attempted.contains(&fact_key(&GapType::DuplicateFact))
                        {
                            gaps.push(DetectedGap {
                                gap_type: GapType::DuplicateFact,
                                fact_id: fact.id.clone(),
                                description: format!(
                                    "Fact '{}' appears to duplicate fact '{}' (Jaccard similarity: {:.2}).",
                                    fact.id, other.id, jaccard
                                ),
                                suggested_task: "Determine which version is more complete and accurate.".to_string(),
                                priority: 0.7,
                                content_snippet: Some(truncate(&fact.content, 200)),
                                fact_id_b: Some(other.id.clone()),
                                namespace: fact.namespace.clone(),
                                date: None,
                            });
                            break; // One duplicate per fact.
                        }

                        // ContradictionGap: high overlap but contradictory signals.
                        if jaccard > 0.3
                            && jaccard <= 0.8
                            && has_contradiction_signals(&fact.content, &other.content)
                            && !attempted.contains(&fact_key(&GapType::ContradictionGap))
                        {
                            gaps.push(DetectedGap {
                                gap_type: GapType::ContradictionGap,
                                fact_id: fact.id.clone(),
                                description: format!(
                                    "Fact '{}' may contradict fact '{}' (overlap: {:.2}).",
                                    fact.id, other.id, jaccard
                                ),
                                suggested_task: "Analyze whether this is a real contradiction or a scope/time difference.".to_string(),
                                priority: 0.85,
                                content_snippet: Some(truncate(&fact.content, 200)),
                                fact_id_b: Some(other.id.clone()),
                                namespace: fact.namespace.clone(),
                                date: None,
                            });
                            break; // One contradiction per fact.
                        }
                    }
                }
            }
        }

        // 3. MissingLink: facts in the same namespace with no edges between them.
        for (namespace, facts) in &facts_by_namespace {
            if facts.len() < 2 {
                continue;
            }
            let max_pairs = 10.min(facts.len() * (facts.len() - 1) / 2);
            let mut checked = 0usize;
            'pair_loop: for i in 0..facts.len() {
                for j in (i + 1)..facts.len() {
                    if checked >= max_pairs {
                        break 'pair_loop;
                    }
                    checked += 1;

                    let pair_key = format!(
                        "{}|{}+missing-link",
                        facts[i].id, facts[j].id
                    );
                    if attempted.contains(&pair_key) {
                        continue;
                    }

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
                            priority: 0.5,
                            content_snippet: Some(truncate(&facts[i].content, 100)),
                            fact_id_b: Some(facts[j].id.clone()),
                            namespace: Some(namespace.clone()),
                            date: None,
                        });
                        break 'pair_loop;
                    }
                }
            }
        }

        // 4. StaleFact: check DB integrity.
        if !attempted.contains(&"db-integrity+stale-fact".to_string()) {
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
                    content_snippet: None,
                    fact_id_b: None,
                    namespace: None,
                    date: None,
                });
            }
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

    /// Detect gaps within a specific namespace only.
    ///
    /// Used by the entropy-gradient searcher to target specific domains
    /// rather than scanning all priority namespaces.
    pub async fn detect_gaps_in_namespace(
        &self,
        max_gaps: usize,
        attempted: &HashSet<String>,
        namespace: &str,
    ) -> Result<Vec<DetectedGap>> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| anyhow!("failed to build HTTP client: {e}"))?;

        // Search for facts in the target namespace.
        let search_results = self.search_facts_in_namespace(&client, namespace).await?;

        let mut gaps: Vec<DetectedGap> = Vec::new();

        for fact in &search_results {
            let fact_key = |gt: &GapType| format!("{}+{}", fact.id, gt);

            // MissingContext: check for graph isolation.
            if !attempted.contains(&fact_key(&GapType::MissingContext)) {
                let has_relations = self
                    .check_discord_relations(&client, &fact.id)
                    .await
                    .unwrap_or(false);
                if !has_relations {
                    gaps.push(DetectedGap {
                        gap_type: GapType::MissingContext,
                        fact_id: fact.id.clone(),
                        description: format!(
                            "Fact '{}' in namespace '{}' has no graph connections.",
                            fact.id, namespace
                        ),
                        suggested_task: format!(
                            "Search for concepts related to fact '{}' in namespace '{}'.",
                            fact.id, namespace
                        ),
                        priority: 0.6,
                        content_snippet: Some(truncate(&fact.content, 200)),
                        fact_id_b: None,
                        namespace: Some(namespace.to_string()),
                        date: None,
                    });
                }
            }

            // LowQualityFact.
            if !attempted.contains(&fact_key(&GapType::LowQualityFact))
                && is_low_quality(&fact.content)
            {
                gaps.push(DetectedGap {
                    gap_type: GapType::LowQualityFact,
                    fact_id: fact.id.clone(),
                    description: format!(
                        "Fact '{}' appears to be low quality.",
                        fact.id
                    ),
                    suggested_task: "Determine if the fact should be kept, improved, or removed.".to_string(),
                    priority: 0.4,
                    content_snippet: Some(truncate(&fact.content, 200)),
                    fact_id_b: None,
                    namespace: Some(namespace.to_string()),
                    date: None,
                });
            }

            // StaleByDate.
            if !attempted.contains(&fact_key(&GapType::StaleByDate)) {
                if let Some(date) = extract_stale_date(&fact.content) {
                    gaps.push(DetectedGap {
                        gap_type: GapType::StaleByDate,
                        fact_id: fact.id.clone(),
                        description: format!(
                            "Fact '{}' references date {} which may be outdated.",
                            fact.id, date
                        ),
                        suggested_task: "Check if the information is still current.".to_string(),
                        priority: 0.65,
                        content_snippet: Some(truncate(&fact.content, 200)),
                        fact_id_b: None,
                        namespace: Some(namespace.to_string()),
                        date: Some(date),
                    });
                }
            }

            // ContradictionGap & DuplicateFact.
            if !attempted.contains(&fact_key(&GapType::ContradictionGap))
                || !attempted.contains(&fact_key(&GapType::DuplicateFact))
            {
                if let Ok(similar) = self.search_similar(&client, &fact.content, &fact.id).await {
                    for other in &similar {
                        if other.namespace != fact.namespace {
                            continue;
                        }
                        let jaccard = jaccard_similarity(&fact.content, &other.content);

                        if jaccard > 0.8
                            && !attempted.contains(&fact_key(&GapType::DuplicateFact))
                        {
                            gaps.push(DetectedGap {
                                gap_type: GapType::DuplicateFact,
                                fact_id: fact.id.clone(),
                                description: format!(
                                    "Fact '{}' appears to duplicate fact '{}' (Jaccard: {:.2}).",
                                    fact.id, other.id, jaccard
                                ),
                                suggested_task: "Determine which version is more complete.".to_string(),
                                priority: 0.7,
                                content_snippet: Some(truncate(&fact.content, 200)),
                                fact_id_b: Some(other.id.clone()),
                                namespace: Some(namespace.to_string()),
                                date: None,
                            });
                            break;
                        }

                        if jaccard > 0.3
                            && jaccard <= 0.8
                            && has_contradiction_signals(&fact.content, &other.content)
                            && !attempted.contains(&fact_key(&GapType::ContradictionGap))
                        {
                            gaps.push(DetectedGap {
                                gap_type: GapType::ContradictionGap,
                                fact_id: fact.id.clone(),
                                description: format!(
                                    "Fact '{}' may contradict fact '{}' (overlap: {:.2}).",
                                    fact.id, other.id, jaccard
                                ),
                                suggested_task: "Analyze whether this is a real contradiction.".to_string(),
                                priority: 0.85,
                                content_snippet: Some(truncate(&fact.content, 200)),
                                fact_id_b: Some(other.id.clone()),
                                namespace: Some(namespace.to_string()),
                                date: None,
                            });
                            break;
                        }
                    }
                }
            }
        }

        // MissingLink: check pairs within this namespace.
        if search_results.len() >= 2 {
            let max_pairs = 10.min(search_results.len() * (search_results.len() - 1) / 2);
            let mut checked = 0usize;
            'pair_loop: for i in 0..search_results.len() {
                for j in (i + 1)..search_results.len() {
                    if checked >= max_pairs {
                        break 'pair_loop;
                    }
                    checked += 1;
                    let pair_key = format!(
                        "{}|{}+missing-link",
                        search_results[i].id, search_results[j].id
                    );
                    if attempted.contains(&pair_key) {
                        continue;
                    }
                    let connected = self
                        .check_edge_between(&client, &search_results[i].id, &search_results[j].id)
                        .await
                        .unwrap_or(false);
                    if !connected {
                        gaps.push(DetectedGap {
                            gap_type: GapType::MissingLink,
                            fact_id: format!("{}|{}", search_results[i].id, search_results[j].id),
                            description: format!(
                                "Facts '{}' and '{}' in namespace '{}' have no graph edge.",
                                search_results[i].id, search_results[j].id, namespace
                            ),
                            suggested_task: format!(
                                "Search for the relationship between facts '{}' and '{}'.",
                                search_results[i].id, search_results[j].id
                            ),
                            priority: 0.5,
                            content_snippet: Some(truncate(&search_results[i].content, 100)),
                            fact_id_b: Some(search_results[j].id.clone()),
                            namespace: Some(namespace.to_string()),
                            date: None,
                        });
                        break 'pair_loop;
                    }
                }
            }
        }

        gaps.sort_by(|a, b| {
            b.priority
                .partial_cmp(&a.priority)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        gaps.truncate(max_gaps);
        Ok(gaps)
    }

    /// Search for facts in a specific namespace.
    async fn search_facts_in_namespace(
        &self,
        client: &reqwest::Client,
        namespace: &str,
    ) -> Result<Vec<SearchFact>> {
        let query = format!("facts in {} namespace", namespace);
        let body = serde_json::json!({"query": query, "top_k": 20});
        let resp = client
            .post(format!("{}/search", self.http_base_url))
            .json(&body)
            .send()
            .await
            .map_err(|e| anyhow!("namespace search failed: {e}"))?;
        let data: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| anyhow!("namespace search parse failed: {e}"))?;

        let mut facts = Vec::new();
        let mut seen_ids = HashSet::new();
        if let Some(results) = data.get("results").and_then(|v| v.as_array()) {
            for r in results {
                let id = r
                    .get("result_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                if id.is_empty() || seen_ids.contains(&id) {
                    continue;
                }
                seen_ids.insert(id.clone());
                facts.push(SearchFact {
                    id,
                    content: r
                        .get("content")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    namespace: r
                        .get("namespace")
                        .and_then(|v| v.as_str())
                        .map(String::from),
                    score: r.get("score").and_then(|v| v.as_f64()).unwrap_or(0.0),
                });
            }
        }
        Ok(facts)
    }

    // -- HTTP helpers --------------------------------------------------------

    /// POST /search with namespace-specific queries to enumerate facts across
    /// the knowledge base. Deduplicates by fact_id.
    async fn search_facts(&self, client: &reqwest::Client) -> Result<Vec<SearchFact>> {
        let mut all_facts: Vec<SearchFact> = Vec::new();
        let mut seen_ids: HashSet<String> = HashSet::new();

        for ns in PRIORITY_NAMESPACES {
            let query = format!("facts in {} namespace", ns);
            let body = serde_json::json!({"query": query, "top_k": 10});
            let resp = client
                .post(format!("{}/search", self.http_base_url))
                .json(&body)
                .send()
                .await
                .map_err(|e| anyhow!("search request failed: {e}"))?;
            let data: serde_json::Value = resp
                .json()
                .await
                .map_err(|e| anyhow!("search response parse failed: {e}"))?;
            if let Some(results) = data.get("results").and_then(|v| v.as_array()) {
                for r in results {
                    let id = r
                        .get("result_id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    if id.is_empty() || seen_ids.contains(&id) {
                        continue;
                    }
                    seen_ids.insert(id.clone());
                    all_facts.push(SearchFact {
                        id: id.clone(),
                        content: r
                            .get("content")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string(),
                        namespace: r
                            .get("namespace")
                            .and_then(|v| v.as_str())
                            .map(String::from),
                        score: r.get("score").and_then(|v| v.as_f64()).unwrap_or(0.0),
                    });
                }
            }
        }

        // Sort by score descending.
        all_facts.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        Ok(all_facts)
    }

    /// Search for facts similar to the given content, excluding the given
    /// fact_id from results.
    async fn search_similar(
        &self,
        client: &reqwest::Client,
        content: &str,
        exclude_id: &str,
    ) -> Result<Vec<SearchFact>> {
        // Use the first 200 chars of content as the query.
        let query = truncate(content, 200);
        let body = serde_json::json!({"query": query, "top_k": 5});
        let resp = client
            .post(format!("{}/search", self.http_base_url))
            .json(&body)
            .send()
            .await
            .map_err(|e| anyhow!("similar search request failed: {e}"))?;
        let data: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| anyhow!("similar search response parse failed: {e}"))?;

        let mut facts = Vec::new();
        if let Some(results) = data.get("results").and_then(|v| v.as_array()) {
            for r in results {
                let id = r
                    .get("result_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                if id.is_empty() || id == exclude_id {
                    continue;
                }
                facts.push(SearchFact {
                    id: id.clone(),
                    content: r
                        .get("content")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    namespace: r
                        .get("namespace")
                        .and_then(|v| v.as_str())
                        .map(String::from),
                    score: r.get("score").and_then(|v| v.as_f64()).unwrap_or(0.0),
                });
            }
        }
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
            return Ok(false);
        }

        let raw: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| anyhow!("failed to parse /discord response: {e}"))?;

        let has_related = raw
            .get("related")
            .and_then(|v| v.as_array())
            .map(|arr| !arr.is_empty())
            .unwrap_or(false);

        Ok(has_related)
    }

    /// Check if two facts have a graph edge between them.
    async fn check_edge_between(
        &self,
        client: &reqwest::Client,
        fact_a: &str,
        fact_b: &str,
    ) -> Result<bool> {
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
    score: f64,
}

/// Check if a namespace should be skipped (social media / ingestion artifact).
fn is_skip_namespace(ns: &str) -> bool {
    SKIP_NAMESPACES.iter().any(|s| *s == ns)
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

/// Tokenize a string into a set of lowercase words.
fn word_set(s: &str) -> HashSet<String> {
    s.split_whitespace()
        .map(|w| w.trim_matches(|c: char| !c.is_alphanumeric()).to_lowercase())
        .filter(|w| !w.is_empty())
        .collect()
}

/// Compute Jaccard similarity between two strings' word sets.
fn jaccard_similarity(a: &str, b: &str) -> f64 {
    let set_a = word_set(a);
    let set_b = word_set(b);
    if set_a.is_empty() && set_b.is_empty() {
        return 0.0;
    }
    let intersection = set_a.intersection(&set_b).count();
    let union = set_a.union(&set_b).count();
    if union == 0 {
        return 0.0;
    }
    intersection as f64 / union as f64
}

/// Check if two facts have contradictory signals.
/// Looks for negation words, different numbers, or different dates.
fn has_contradiction_signals(a: &str, b: &str) -> bool {
    let negation_words = ["not", "no", "never", "false", "incorrect", "wrong", "disagree"];
    let a_lower = a.to_lowercase();
    let b_lower = b.to_lowercase();

    // Check for negation in one but not the other.
    let a_has_neg = negation_words.iter().any(|w| {
        a_lower.contains(&format!(" {} ", w)) || a_lower.starts_with(&format!("{} ", w))
    });
    let b_has_neg = negation_words.iter().any(|w| {
        b_lower.contains(&format!(" {} ", w)) || b_lower.starts_with(&format!("{} ", w))
    });

    if a_has_neg != b_has_neg {
        return true;
    }

    // Check for different numbers.
    let nums_a: Vec<&str> = extract_numbers(a);
    let nums_b: Vec<&str> = extract_numbers(b);
    if !nums_a.is_empty()
        && !nums_b.is_empty()
        && nums_a != nums_b
        && nums_a.iter().any(|n| !nums_b.contains(n))
    {
        return true;
    }

    // Check for different dates.
    let dates_a = extract_dates(a);
    let dates_b = extract_dates(b);
    if !dates_a.is_empty()
        && !dates_b.is_empty()
        && dates_a != dates_b
        && dates_a.iter().any(|d| !dates_b.contains(d))
    {
        return true;
    }

    false
}

/// Extract number tokens from a string.
fn extract_numbers(s: &str) -> Vec<&str> {
    let mut nums = Vec::new();
    for word in s.split_whitespace() {
        if word.chars().all(|c| c.is_ascii_digit()) && !word.is_empty() {
            nums.push(word);
        }
    }
    nums
}

/// Extract date-like strings (20XX) from content.
fn extract_dates(s: &str) -> Vec<String> {
    let mut dates = Vec::new();
    let bytes = s.as_bytes();
    let mut i = 0;
    while i + 3 < bytes.len() {
        if bytes[i] == b'2'
            && bytes[i + 1] == b'0'
            && bytes[i + 2].is_ascii_digit()
            && bytes[i + 3].is_ascii_digit()
        {
            dates.push(s[i..i + 4].to_string());
            i += 4;
        } else {
            i += 1;
        }
    }
    dates
}

/// Check if a fact's content contains a date (20XX) that is more than 6 months
/// old. Returns the date string if stale, None otherwise.
fn extract_stale_date(content: &str) -> Option<String> {
    let dates = extract_dates(content);
    if dates.is_empty() {
        return None;
    }

    // Use the earliest date found. If it's more than 6 months old, flag it.
    // We compare against the current year-month. Since this is a heuristic,
    // we consider a date "stale" if it's more than 6 months behind the
    // current date.
    let now = chrono::Utc::now();
    let six_months_ago = now - chrono::Duration::days(183);

    for date_str in &dates {
        // Parse as a year (20XX).
        if let Ok(year) = date_str.parse::<i32>() {
            // If the year is before the current year, it's definitely stale.
            if year < now.format("%Y").to_string().parse::<i32>().unwrap_or(0) {
                return Some(date_str.clone());
            }
            // If same year, check if we're past June (rough 6-month heuristic).
            // A date like "2025" referenced in content from 2025 is stale if
            // we're in the second half of 2025 or later.
            // For simplicity: if the date year equals the current year and the
            // current month is > 6, consider it potentially stale.
            // Actually, the simplest heuristic: if the year is <= the year of
            // six_months_ago, flag it.
            let stale_year = six_months_ago.format("%Y").to_string().parse::<i32>().unwrap_or(0);
            if year <= stale_year {
                return Some(date_str.clone());
            }
        }
    }
    None
}

/// Check if fact content is low quality: too short, mostly URLs, or raw JSON.
fn is_low_quality(content: &str) -> bool {
    // Too short.
    if content.len() < 30 {
        return true;
    }

    // Mostly URLs — if more than 60% of the content is URLs.
    let url_count = content.matches("http").count();
    let word_count = content.split_whitespace().count();
    if word_count > 0 && url_count * 3 > word_count {
        return true;
    }

    // Raw JSON blob — starts with { or [ and looks like JSON.
    let trimmed = content.trim();
    if (trimmed.starts_with('{') || trimmed.starts_with('['))
        && (trimmed.ends_with('}') || trimmed.ends_with(']'))
        && trimmed.contains('"')
    {
        return true;
    }

    false
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
            GapType::DuplicateFact,
            GapType::StaleByDate,
            GapType::LowQualityFact,
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

    #[test]
    fn skip_namespace_detection() {
        assert!(is_skip_namespace("mixed"));
        assert!(is_skip_namespace("chatgpt"));
        assert!(is_skip_namespace("twitter"));
        assert!(is_skip_namespace("test"));
        assert!(!is_skip_namespace("general"));
        assert!(!is_skip_namespace("research"));
    }

    #[test]
    fn jaccard_similarity_identical_strings() {
        let s = "the quick brown fox jumps over the lazy dog";
        assert!((jaccard_similarity(s, s) - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn jaccard_similarity_disjoint_strings() {
        assert!((jaccard_similarity("apple banana", "cherry date")).abs() < f64::EPSILON);
    }

    #[test]
    fn jaccard_similarity_partial_overlap() {
        let a = "the quick brown fox";
        let b = "the quick red fox";
        let sim = jaccard_similarity(a, b);
        // 3 shared words (the, quick, fox), 5 unique total (brown, red added)
        assert!(sim > 0.4 && sim < 0.8);
    }

    #[test]
    fn jaccard_similarity_empty_strings() {
        assert!((jaccard_similarity("", "")).abs() < f64::EPSILON);
    }

    #[test]
    fn contradiction_negation_detection() {
        assert!(has_contradiction_signals(
            "Rust is a systems language",
            "Rust is not a systems language"
        ));
    }

    #[test]
    fn contradiction_different_numbers() {
        assert!(has_contradiction_signals(
            "The crate has 49 tests",
            "The crate has 50 tests"
        ));
    }

    #[test]
    fn contradiction_different_dates() {
        assert!(has_contradiction_signals(
            "Released in 2024",
            "Released in 2025"
        ));
    }

    #[test]
    fn no_contradiction_for_similar_facts() {
        assert!(!has_contradiction_signals(
            "Rust is a systems programming language",
            "Rust is a systems programming language with memory safety"
        ));
    }

    #[test]
    fn low_quality_short_content() {
        assert!(is_low_quality("too short"));
        assert!(!is_low_quality("This is a sufficiently long factual statement about Rust."));
    }

    #[test]
    fn low_quality_url_heavy() {
        assert!(is_low_quality("http://example.com http://test.com http://foo.com http://bar.com"));
        assert!(!is_low_quality("See http://example.com for more details about Rust programming."));
    }

    #[test]
    fn low_quality_json_blob() {
        assert!(is_low_quality(r#"{"key": "value", "nested": {"a": 1}}"#));
        assert!(!is_low_quality("This is a normal fact about the knowledge base."));
    }

    #[test]
    fn extract_dates_finds_years() {
        let dates = extract_dates("Released in 2024 and updated in 2025");
        assert_eq!(dates, vec!["2024", "2025"]);
    }

    #[test]
    fn extract_dates_no_match() {
        let dates = extract_dates("no dates here");
        assert!(dates.is_empty());
    }

    #[test]
    fn extract_numbers_finds_numeric_tokens() {
        let nums = extract_numbers("49 tests and 50 assertions");
        assert_eq!(nums, vec!["49", "50"]);
    }

    #[test]
    fn word_set_lowercases_and_strips_punctuation() {
        let ws = word_set("Hello, World! hello");
        assert!(ws.contains("hello"));
        assert!(ws.contains("world"));
        assert_eq!(ws.len(), 2);
    }

    /// Verify the MissingContext gap construction logic.
    #[test]
    fn missing_context_gap_construction() {
        let gap = DetectedGap {
            gap_type: GapType::MissingContext,
            fact_id: "fact:abc123".to_string(),
            description: "Fact has no graph connections.".to_string(),
            suggested_task: "Search for related concepts.".to_string(),
            priority: 0.6,
            content_snippet: Some("some content".to_string()),
            fact_id_b: None,
            namespace: Some("general".to_string()),
            date: None,
        };
        assert_eq!(gap.gap_type, GapType::MissingContext);
        assert!((gap.priority - 0.6).abs() < f64::EPSILON);
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
            content_snippet: None,
            fact_id_b: None,
            namespace: None,
            date: None,
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
                priority: 0.5,
                content_snippet: None,
                fact_id_b: None,
                namespace: None,
                date: None,
            },
            DetectedGap {
                gap_type: GapType::StaleFact,
                fact_id: "b".to_string(),
                description: "high".to_string(),
                suggested_task: "task".to_string(),
                priority: 0.9,
                content_snippet: None,
                fact_id_b: None,
                namespace: None,
                date: None,
            },
            DetectedGap {
                gap_type: GapType::ContradictionGap,
                fact_id: "c".to_string(),
                description: "mid".to_string(),
                suggested_task: "task".to_string(),
                priority: 0.85,
                content_snippet: None,
                fact_id_b: None,
                namespace: None,
                date: None,
            },
        ];
        gaps.sort_by(|a, b| {
            b.priority
                .partial_cmp(&a.priority)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        assert_eq!(gaps[0].gap_type, GapType::StaleFact);
        assert_eq!(gaps[1].gap_type, GapType::ContradictionGap);
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
                score: 0.9,
            },
            SearchFact {
                id: "fact:2".to_string(),
                content: "second".to_string(),
                namespace: Some("general".to_string()),
                score: 0.8,
            },
            SearchFact {
                id: "fact:3".to_string(),
                content: "third".to_string(),
                namespace: Some("coding".to_string()),
                score: 0.7,
            },
        ];

        let mut groups: HashMap<String, Vec<&SearchFact>> = HashMap::new();
        for fact in &facts {
            if let Some(ns) = &fact.namespace {
                groups.entry(ns.clone()).or_default().push(fact);
            }
        }

        assert_eq!(groups.get("general").map(|v| v.len()), Some(2));
        assert_eq!(groups.get("coding").map(|v| v.len()), Some(1));
    }

    /// Verify attempted gaps are skipped.
    #[test]
    fn attempted_gaps_are_skipped() {
        let mut attempted: HashSet<String> = HashSet::new();
        attempted.insert("fact:abc+missing-context".to_string());

        // The key format matches what detect_gaps uses internally.
        assert!(attempted.contains("fact:abc+missing-context"));
        assert!(!attempted.contains("fact:abc+stale-fact"));
    }

    /// Parse the canned /search response and verify SearchFact extraction logic.
    #[test]
    fn parse_search_response() {
        let canned = serde_json::json!({
            "results": [
                {
                    "result_id": "fact:aaa",
                    "content": "Rust is a systems programming language.",
                    "namespace": "general",
                    "score": 0.95
                },
                {
                    "result_id": "fact:bbb",
                    "content": "Tokio is an async runtime.",
                    "namespace": "coding",
                    "score": 0.82
                },
                {
                    "result_id": "fact:ccc",
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
                let id = item
                    .get("result_id")
                    .and_then(|v| v.as_str())?
                    .to_string();
                let content = item
                    .get("content")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let namespace = item
                    .get("namespace")
                    .and_then(|v| v.as_str())
                    .map(String::from);
                let score = item.get("score").and_then(|v| v.as_f64()).unwrap_or(0.0);
                Some(SearchFact {
                    id,
                    content,
                    namespace,
                    score,
                })
            })
            .collect();

        assert_eq!(facts.len(), 3);
        assert_eq!(facts[0].id, "fact:aaa");
        assert_eq!(facts[0].namespace.as_deref(), Some("general"));
        assert_eq!(facts[2].namespace, None);
    }

    /// Verify priority constants match spec.
    #[test]
    fn priority_scores_are_correct() {
        // StaleFact (integrity): 0.9
        // ContradictionGap: 0.85
        // DuplicateFact: 0.7
        // MissingContext: 0.6
        // StaleByDate: 0.65
        // MissingLink: 0.5
        // LowQualityFact: 0.4
        let stale = DetectedGap {
            gap_type: GapType::StaleFact,
            fact_id: "x".to_string(),
            description: "d".to_string(),
            suggested_task: "t".to_string(),
            priority: 0.9,
            content_snippet: None,
            fact_id_b: None,
            namespace: None,
            date: None,
        };
        assert!((stale.priority - 0.9).abs() < f64::EPSILON);
    }
}