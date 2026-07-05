//! Mission system — structured high-ROI objectives for the autonomous loop.
//!
//! A [`Mission`] is a typed objective that knows what to search for, what
//! constitutes an issue, what prompt to send to the model, and how to
//! evaluate the response. The [`MissionScheduler`] picks the highest-priority
//! mission that is due to run and records results to dynamically adjust
//! priority over time.
//!
//! The mission system complements the existing [`GapDetector`] by providing
//! goal-directed scans rather than purely structural ones. Each mission
//! implements the [`MissionImpl`] trait and is dispatched by the scheduler
//! from within the [`AutonomousLoop`].
//!
//! [`GapDetector`]: crate::gap_detector::GapDetector
//! [`AutonomousLoop`]: crate::loop_driver::AutonomousLoop

use crate::gap_detector::{DetectedGap, GapType, PRIORITY_NAMESPACES, SKIP_NAMESPACES};
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

// ---------------------------------------------------------------------------
// MissionQuery
// ---------------------------------------------------------------------------

/// A single search query specification used by a mission's `detect_issues`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissionQuery {
    /// The search query string sent to POST /search.
    pub query: String,
    /// Optional namespace filter. If `None`, searches all namespaces.
    pub namespaces: Option<Vec<String>>,
    /// Maximum number of results to request.
    pub top_k: usize,
}

// ---------------------------------------------------------------------------
// Mission enum
// ---------------------------------------------------------------------------

/// The eight mission types the autonomous loop can schedule.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Mission {
    /// Verify that published crate version facts are current.
    VerifyPublishedCrates,
    /// Verify that file paths referenced in facts still exist.
    VerifyFileReferences,
    /// Detect pairs of facts with contradictory claims.
    DetectContradictions,
    /// Audit each namespace for graph connectivity and coverage.
    AuditNamespaceCompleteness,
    /// Trace provenance/source attributions to verify they still exist.
    TraceProvenanceChains,
    /// Find duplicate facts with high content overlap across namespaces.
    FindDuplicates,
    /// Check if codebase metric facts match the current state.
    VerifyCodebaseSync,
    /// Find facts referencing old dates or versions.
    StaleDateDetection,
}

impl Mission {
    /// Kebab-case name for serialization / logging.
    pub fn as_kebab(&self) -> &'static str {
        match self {
            Self::VerifyPublishedCrates => "verify-published-crates",
            Self::VerifyFileReferences => "verify-file-references",
            Self::DetectContradictions => "detect-contradictions",
            Self::AuditNamespaceCompleteness => "audit-namespace-completeness",
            Self::TraceProvenanceChains => "trace-provenance-chains",
            Self::FindDuplicates => "find-duplicates",
            Self::VerifyCodebaseSync => "verify-codebase-sync",
            Self::StaleDateDetection => "stale-date-detection",
        }
    }

    /// Delegate to the mission-specific implementation's search queries.
    pub fn search_queries(&self) -> Vec<MissionQuery> {
        match self {
            Self::VerifyPublishedCrates => VerifyPublishedCratesMission.search_queries(),
            Self::VerifyFileReferences => VerifyFileReferencesMission.search_queries(),
            Self::DetectContradictions => DetectContradictionsMission.search_queries(),
            Self::AuditNamespaceCompleteness => AuditNamespaceCompletenessMission.search_queries(),
            Self::TraceProvenanceChains => TraceProvenanceChainsMission.search_queries(),
            Self::FindDuplicates => FindDuplicatesMission.search_queries(),
            Self::VerifyCodebaseSync => VerifyCodebaseSyncMission.search_queries(),
            Self::StaleDateDetection => StaleDateDetectionMission.search_queries(),
        }
    }

    /// Delegate to the mission-specific implementation's issue detection.
    pub async fn detect_issues(
        &self,
        http_base_url: &str,
        attempted: &HashSet<String>,
    ) -> Result<Vec<DetectedGap>> {
        match self {
            Self::VerifyPublishedCrates => {
                VerifyPublishedCratesMission
                    .detect_issues(http_base_url, attempted)
                    .await
            }
            Self::VerifyFileReferences => {
                VerifyFileReferencesMission
                    .detect_issues(http_base_url, attempted)
                    .await
            }
            Self::DetectContradictions => {
                DetectContradictionsMission
                    .detect_issues(http_base_url, attempted)
                    .await
            }
            Self::AuditNamespaceCompleteness => {
                AuditNamespaceCompletenessMission
                    .detect_issues(http_base_url, attempted)
                    .await
            }
            Self::TraceProvenanceChains => {
                TraceProvenanceChainsMission
                    .detect_issues(http_base_url, attempted)
                    .await
            }
            Self::FindDuplicates => {
                FindDuplicatesMission
                    .detect_issues(http_base_url, attempted)
                    .await
            }
            Self::VerifyCodebaseSync => {
                VerifyCodebaseSyncMission
                    .detect_issues(http_base_url, attempted)
                    .await
            }
            Self::StaleDateDetection => {
                StaleDateDetectionMission
                    .detect_issues(http_base_url, attempted)
                    .await
            }
        }
    }

    /// Delegate to the mission-specific implementation's prompt builder.
    pub fn prompt_for_gap(&self, gap: &DetectedGap) -> String {
        match self {
            Self::VerifyPublishedCrates => VerifyPublishedCratesMission.prompt_for_gap(gap),
            Self::VerifyFileReferences => VerifyFileReferencesMission.prompt_for_gap(gap),
            Self::DetectContradictions => DetectContradictionsMission.prompt_for_gap(gap),
            Self::AuditNamespaceCompleteness => {
                AuditNamespaceCompletenessMission.prompt_for_gap(gap)
            }
            Self::TraceProvenanceChains => TraceProvenanceChainsMission.prompt_for_gap(gap),
            Self::FindDuplicates => FindDuplicatesMission.prompt_for_gap(gap),
            Self::VerifyCodebaseSync => VerifyCodebaseSyncMission.prompt_for_gap(gap),
            Self::StaleDateDetection => StaleDateDetectionMission.prompt_for_gap(gap),
        }
    }

    /// Base priority in `[0.0, 1.0]`.
    pub fn base_priority(&self) -> f64 {
        match self {
            Self::VerifyPublishedCrates => VerifyPublishedCratesMission.priority(),
            Self::VerifyFileReferences => VerifyFileReferencesMission.priority(),
            Self::DetectContradictions => DetectContradictionsMission.priority(),
            Self::AuditNamespaceCompleteness => AuditNamespaceCompletenessMission.priority(),
            Self::TraceProvenanceChains => TraceProvenanceChainsMission.priority(),
            Self::FindDuplicates => FindDuplicatesMission.priority(),
            Self::VerifyCodebaseSync => VerifyCodebaseSyncMission.priority(),
            Self::StaleDateDetection => StaleDateDetectionMission.priority(),
        }
    }

    /// Run frequency (every N iterations).
    pub fn frequency(&self) -> usize {
        match self {
            Self::VerifyPublishedCrates => VerifyPublishedCratesMission.frequency(),
            Self::VerifyFileReferences => VerifyFileReferencesMission.frequency(),
            Self::DetectContradictions => DetectContradictionsMission.frequency(),
            Self::AuditNamespaceCompleteness => AuditNamespaceCompletenessMission.frequency(),
            Self::TraceProvenanceChains => TraceProvenanceChainsMission.frequency(),
            Self::FindDuplicates => FindDuplicatesMission.frequency(),
            Self::VerifyCodebaseSync => VerifyCodebaseSyncMission.frequency(),
            Self::StaleDateDetection => StaleDateDetectionMission.frequency(),
        }
    }
}

impl std::fmt::Display for Mission {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_kebab())
    }
}

// ---------------------------------------------------------------------------
// MissionImpl trait
// ---------------------------------------------------------------------------

/// Trait implemented by each mission variant to define its search, detection,
/// prompt, priority, and frequency logic.
#[allow(async_fn_in_trait)]
pub trait MissionImpl: Send + Sync {
    /// Human-readable name (kebab-case).
    fn name(&self) -> &str;

    /// The search queries this mission issues against POST /search.
    fn search_queries(&self) -> Vec<MissionQuery>;

    /// Run mission-specific detection logic and return detected gaps.
    ///
    /// `http_base_url` is the semantic-memory server base URL.
    /// `attempted` is the set of already-attempted gap keys to avoid duplicates.
    async fn detect_issues(
        &self,
        http_base_url: &str,
        attempted: &HashSet<String>,
    ) -> Result<Vec<DetectedGap>>;

    /// Build the remediation prompt for a specific detected gap.
    fn prompt_for_gap(&self, gap: &DetectedGap) -> String;

    /// Base priority in `[0.0, 1.0]` — higher is more urgent.
    fn priority(&self) -> f64;

    /// Run every N loop iterations.
    fn frequency(&self) -> usize;
}

// ---------------------------------------------------------------------------
// ScheduledMission & MissionScheduler
// ---------------------------------------------------------------------------

/// A mission scheduled for periodic execution with adaptive priority.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledMission {
    /// The mission variant.
    pub mission: Mission,
    /// Last iteration this mission was run (0 = never).
    pub last_run: usize,
    /// Issues found during the last run.
    pub issues_found: usize,
    /// Dynamic priority — adjusted based on issue yield.
    pub dynamic_priority: f64,
}

impl ScheduledMission {
    /// Create a scheduled mission with initial state.
    fn new(mission: Mission) -> Self {
        let base = mission.base_priority();
        Self {
            mission,
            last_run: 0,
            issues_found: 0,
            dynamic_priority: base,
        }
    }

    /// Base priority from the mission's implementation.
    fn base_priority(&self) -> f64 {
        self.mission.base_priority()
    }

    /// Frequency from the mission's implementation.
    fn frequency(&self) -> usize {
        self.mission.frequency()
    }

    /// Whether this mission is due to run at the given iteration.
    fn is_due(&self, current_iteration: usize) -> bool {
        if self.last_run == 0 {
            return true;
        }
        current_iteration >= self.last_run + self.frequency()
    }
}

/// Schedules missions for the autonomous loop, picking the highest-priority
/// due mission and adjusting priorities based on issue yield.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissionScheduler {
    /// All scheduled missions.
    missions: Vec<ScheduledMission>,
}

impl MissionScheduler {
    /// Create a scheduler with all 8 missions at their default priorities.
    pub fn new() -> Self {
        let missions = vec![
            ScheduledMission::new(Mission::VerifyPublishedCrates),
            ScheduledMission::new(Mission::VerifyFileReferences),
            ScheduledMission::new(Mission::DetectContradictions),
            ScheduledMission::new(Mission::AuditNamespaceCompleteness),
            ScheduledMission::new(Mission::TraceProvenanceChains),
            ScheduledMission::new(Mission::FindDuplicates),
            ScheduledMission::new(Mission::VerifyCodebaseSync),
            ScheduledMission::new(Mission::StaleDateDetection),
        ];
        Self { missions }
    }

    /// Create a scheduler with a custom set of missions.
    pub fn with_missions(missions: Vec<ScheduledMission>) -> Self {
        Self { missions }
    }

    /// Pick the highest-priority mission that is due to run.
    ///
    /// Returns `None` if no mission is due.
    pub fn next_mission(&self, current_iteration: usize) -> Option<&Mission> {
        self.missions
            .iter()
            .filter(|sm| sm.is_due(current_iteration))
            .max_by(|a, b| {
                a.dynamic_priority
                    .partial_cmp(&b.dynamic_priority)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|sm| &sm.mission)
    }

    /// Record the result of a mission run and adjust dynamic priority.
    ///
    /// If `issues_found == 0`, reduce dynamic priority by 0.1 (min 0.1).
    /// If `issues_found > 0`, reset dynamic priority to the mission's base priority.
    pub fn record_result(
        &mut self,
        mission_name: &str,
        issues_found: usize,
        current_iteration: usize,
    ) {
        for sm in &mut self.missions {
            if sm.mission.as_kebab() == mission_name {
                sm.last_run = current_iteration;
                sm.issues_found = issues_found;
                if issues_found == 0 {
                    sm.dynamic_priority = (sm.dynamic_priority - 0.1).max(0.1);
                } else {
                    sm.dynamic_priority = sm.base_priority();
                }
                break;
            }
        }
    }

    /// Get a snapshot of all scheduled missions (for TUI / debugging).
    pub fn scheduled_missions(&self) -> &[ScheduledMission] {
        &self.missions
    }

    /// Get the dynamic priority of a specific mission by name.
    pub fn priority_of(&self, mission_name: &str) -> Option<f64> {
        self.missions
            .iter()
            .find(|sm| sm.mission.as_kebab() == mission_name)
            .map(|sm| sm.dynamic_priority)
    }
}

impl Default for MissionScheduler {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// HTTP helper
// ---------------------------------------------------------------------------

/// A parsed search result from POST /search.
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct SearchResult {
    id: String,
    content: String,
    namespace: Option<String>,
    score: f64,
}

/// POST /search with the given query parameters and parse results.
async fn http_search(http_base_url: &str, query: &MissionQuery) -> Result<Vec<SearchResult>> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| anyhow!("failed to build HTTP client: {e}"))?;

    let body = if let Some(ref namespaces) = query.namespaces {
        serde_json::json!({
            "query": query.query,
            "top_k": query.top_k,
            "namespaces": namespaces,
        })
    } else {
        serde_json::json!({
            "query": query.query,
            "top_k": query.top_k,
        })
    };

    let url = format!("{}/search", http_base_url);
    let resp = client
        .post(&url)
        .json(&body)
        .send()
        .await
        .map_err(|e| anyhow!("search request failed: {e}"))?;

    let data: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| anyhow!("search response parse failed: {e}"))?;

    parse_search_results(&data)
}

/// Parse the JSON response from /search into SearchResult vec.
fn parse_search_results(data: &serde_json::Value) -> Result<Vec<SearchResult>> {
    let mut results = Vec::new();
    let mut seen = HashSet::new();

    if let Some(arr) = data.get("results").and_then(|v| v.as_array()) {
        for r in arr {
            let id = r
                .get("result_id")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            if id.is_empty() || seen.contains(&id) {
                continue;
            }
            seen.insert(id.clone());
            results.push(SearchResult {
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

    Ok(results)
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
        .map(|w| {
            w.trim_matches(|c: char| !c.is_alphanumeric())
                .to_lowercase()
        })
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

/// Check if two facts have contradictory signals (negation, different numbers, different dates).
fn has_contradiction_signals(a: &str, b: &str) -> bool {
    let negation_words = [
        "not",
        "no",
        "never",
        "false",
        "incorrect",
        "wrong",
        "disagree",
    ];
    let a_lower = a.to_lowercase();
    let b_lower = b.to_lowercase();

    let a_has_neg = negation_words
        .iter()
        .any(|w| a_lower.contains(&format!(" {} ", w)) || a_lower.starts_with(&format!("{} ", w)));
    let b_has_neg = negation_words
        .iter()
        .any(|w| b_lower.contains(&format!(" {} ", w)) || b_lower.starts_with(&format!("{} ", w)));

    if a_has_neg != b_has_neg {
        return true;
    }

    // Different numbers.
    let nums_a: Vec<&str> = a
        .split_whitespace()
        .filter(|w| w.chars().all(|c| c.is_ascii_digit()) && !w.is_empty())
        .collect();
    let nums_b: Vec<&str> = b
        .split_whitespace()
        .filter(|w| w.chars().all(|c| c.is_ascii_digit()) && !w.is_empty())
        .collect();
    if !nums_a.is_empty() && !nums_b.is_empty() && nums_a.iter().any(|n| !nums_b.contains(n)) {
        return true;
    }

    // Different dates (20XX).
    let dates_a: Vec<String> = extract_year_strings(a);
    let dates_b: Vec<String> = extract_year_strings(b);
    if !dates_a.is_empty() && !dates_b.is_empty() && dates_a.iter().any(|d| !dates_b.contains(d)) {
        return true;
    }

    false
}

/// Extract year-like strings (20XX) from content.
fn extract_year_strings(s: &str) -> Vec<String> {
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

/// Extract full date strings (20XX-MM or 20XX-MM-DD or 20XX.MM.DD) from content.
fn extract_full_dates(s: &str) -> Vec<String> {
    let mut dates = Vec::new();
    let bytes = s.as_bytes();
    let mut i = 0;
    while i + 6 < bytes.len() {
        // Look for 20XX[-/.]NN
        if bytes[i] == b'2'
            && bytes[i + 1] == b'0'
            && bytes[i + 2].is_ascii_digit()
            && bytes[i + 3].is_ascii_digit()
            && (bytes[i + 4] == b'-' || bytes[i + 4] == b'.' || bytes[i + 4] == b'/')
            && bytes[i + 5].is_ascii_digit()
            && bytes[i + 6].is_ascii_digit()
        {
            let end = (i + 7).min(bytes.len());
            dates.push(s[i..end].to_string());
            i += 7;
        } else {
            i += 1;
        }
    }
    dates
}

/// Extract version-like strings (vN.N.N) from content.
fn extract_versions(s: &str) -> Vec<String> {
    let mut versions = Vec::new();
    let bytes = s.as_bytes();
    let mut i = 0;
    while i + 2 < bytes.len() {
        if bytes[i] == b'v' && bytes[i + 1].is_ascii_digit() {
            // Scan forward to capture vN.N.N...
            let mut end = i + 1;
            while end < bytes.len() && (bytes[end].is_ascii_digit() || bytes[end] == b'.') {
                end += 1;
            }
            versions.push(s[i..end].to_string());
            i = end;
        } else {
            i += 1;
        }
    }
    versions
}

/// Extract file paths from content (e.g., /path/to/file.rs, src/lib.rs).
fn extract_file_paths(s: &str) -> Vec<String> {
    let mut paths = Vec::new();
    // Match paths like /a/b/c.rs or src/lib.rs or crates/foo/Cargo.toml
    let mut chars = s.char_indices().peekable();
    while let Some((idx, c)) = chars.next() {
        if c == '/'
            || (c.is_ascii_alphabetic() && idx + 3 < s.len() && s[idx..].starts_with("src/")
                || s[idx..].starts_with("crates/")
                || s[idx..].starts_with("docs/"))
        {
            // Try to capture a path-like token ending in a file extension
            let mut end = idx;
            if c == '/' {
                end = idx + 1;
            }
            while end < s.len() {
                let ch = s.as_bytes()[end];
                if ch.is_ascii_alphanumeric()
                    || ch == b'_'
                    || ch == b'/'
                    || ch == b'-'
                    || ch == b'.'
                {
                    end += 1;
                } else {
                    break;
                }
            }
            let candidate = &s[idx..end];
            if candidate.ends_with(".rs")
                || candidate.ends_with(".toml")
                || candidate.ends_with(".py")
                || candidate.ends_with(".md")
                || candidate.ends_with(".json")
            {
                if candidate.len() > 4 && !paths.contains(&candidate.to_string()) {
                    paths.push(candidate.to_string());
                }
            }
        }
    }
    paths
}

/// Check if a date string is more than 6 months old.
fn is_stale_date(date_str: &str) -> bool {
    let now = chrono::Utc::now();
    let six_months_ago = now - chrono::Duration::days(183);

    // Try full date parse (20XX-MM-DD or 20XX-MM).
    if date_str.len() >= 7 {
        let parts: Vec<&str> = date_str
            .split(|c: char| c == '-' || c == '.' || c == '/')
            .collect();
        if parts.len() >= 2 {
            if let (Ok(year), Ok(month)) = (parts[0].parse::<i32>(), parts[1].parse::<u32>()) {
                let day = parts
                    .get(2)
                    .and_then(|d| d.parse::<u32>().ok())
                    .unwrap_or(1);
                if let Some(date) = chrono::NaiveDate::from_ymd_opt(year, month, day) {
                    return date < six_months_ago.date_naive();
                }
            }
        }
    }

    // Fallback: year-only comparison.
    if let Ok(year) = date_str.parse::<i32>() {
        let stale_year = six_months_ago
            .format("%Y")
            .to_string()
            .parse::<i32>()
            .unwrap_or(0);
        return year <= stale_year;
    }

    false
}

/// Check if a namespace should be skipped (social media / ingestion artifact).
fn is_skip_namespace(ns: &str) -> bool {
    SKIP_NAMESPACES.iter().any(|s| *s == ns)
}

/// Count facts per namespace from search results.
fn count_by_namespace(results: &[SearchResult]) -> std::collections::HashMap<String, usize> {
    let mut counts = std::collections::HashMap::new();
    for r in results {
        if let Some(ref ns) = r.namespace {
            if !is_skip_namespace(ns) {
                *counts.entry(ns.clone()).or_insert(0) += 1;
            }
        }
    }
    counts
}

// ===========================================================================
// Mission implementations
// ===========================================================================

// ---------------------------------------------------------------------------
// 1. VerifyPublishedCrates
// ---------------------------------------------------------------------------

/// Mission: verify published crate version facts are current.
#[derive(Debug, Clone)]
pub struct VerifyPublishedCratesMission;

impl MissionImpl for VerifyPublishedCratesMission {
    fn name(&self) -> &str {
        "verify-published-crates"
    }

    fn search_queries(&self) -> Vec<MissionQuery> {
        vec![MissionQuery {
            query: "crate version published crates.io semantic-memory claim-ledger turbo-quant"
                .to_string(),
            namespaces: Some(vec![
                "projects".to_string(),
                "libraries".to_string(),
                "libraries-crates".to_string(),
                "semantic-memory".to_string(),
            ]),
            top_k: 20,
        }]
    }

    async fn detect_issues(
        &self,
        http_base_url: &str,
        attempted: &HashSet<String>,
    ) -> Result<Vec<DetectedGap>> {
        let mut all_results = Vec::new();
        for q in self.search_queries() {
            let results = http_search(http_base_url, &q).await.unwrap_or_default();
            all_results.extend(results);
        }

        let mut gaps = Vec::new();
        for fact in &all_results {
            let versions = extract_versions(&fact.content);
            if versions.is_empty() {
                continue;
            }

            let key = format!("{}+stale-fact", fact.id);
            if attempted.contains(&key) {
                continue;
            }

            // Check if the fact mentions version info that might be outdated.
            // We flag any fact with version numbers as a candidate — the model
            // will verify whether it's current.
            let snippet = truncate(&fact.content, 200);
            gaps.push(DetectedGap {
                gap_type: GapType::StaleFact,
                fact_id: fact.id.clone(),
                description: format!(
                    "Fact '{}' mentions crate version(s) {} which may be outdated. Content: {}",
                    fact.id,
                    versions.join(", "),
                    truncate(&fact.content, 120)
                ),
                suggested_task: "Verify whether this version information is current.".to_string(),
                priority: self.priority(),
                content_snippet: Some(snippet),
                fact_id_b: None,
                content_b: None,
                namespace: fact.namespace.clone(),
                date: None,
            });
        }

        Ok(gaps)
    }

    fn prompt_for_gap(&self, gap: &DetectedGap) -> String {
        let snippet = gap.content_snippet.as_deref().unwrap_or(&gap.description);
        format!(
            "The knowledge base says {}. Verify whether this version information is current. \
             Check if newer versions exist.",
            snippet
        )
    }

    fn priority(&self) -> f64 {
        0.9
    }

    fn frequency(&self) -> usize {
        20
    }
}

// ---------------------------------------------------------------------------
// 2. VerifyFileReferences
// ---------------------------------------------------------------------------

/// Mission: verify file paths referenced in facts still exist.
#[derive(Debug, Clone)]
pub struct VerifyFileReferencesMission;

impl MissionImpl for VerifyFileReferencesMission {
    fn name(&self) -> &str {
        "verify-file-references"
    }

    fn search_queries(&self) -> Vec<MissionQuery> {
        vec![MissionQuery {
            query: "file path src lib.rs Cargo.toml crates".to_string(),
            namespaces: Some(vec!["projects".to_string(), "libraries".to_string()]),
            top_k: 20,
        }]
    }

    async fn detect_issues(
        &self,
        http_base_url: &str,
        attempted: &HashSet<String>,
    ) -> Result<Vec<DetectedGap>> {
        let mut all_results = Vec::new();
        for q in self.search_queries() {
            let results = http_search(http_base_url, &q).await.unwrap_or_default();
            all_results.extend(results);
        }

        let mut gaps = Vec::new();
        let known_prefixes = ["crates/", "src/", "docs/"];

        for fact in &all_results {
            let paths = extract_file_paths(&fact.content);
            if paths.is_empty() {
                continue;
            }

            for path in &paths {
                let key = format!("{}+stale-fact", fact.id);
                if attempted.contains(&key) {
                    continue;
                }

                // Flag if the path starts with a known prefix — these are
                // codebase paths that may have moved or been deleted.
                let starts_with_known = known_prefixes.iter().any(|p| path.starts_with(p));
                if !starts_with_known && !path.starts_with('/') {
                    continue;
                }

                gaps.push(DetectedGap {
                    gap_type: GapType::StaleFact,
                    fact_id: fact.id.clone(),
                    description: format!(
                        "Fact '{}' references file path '{}' which may no longer exist. Content: {}",
                        fact.id,
                        path,
                        truncate(&fact.content, 120)
                    ),
                    suggested_task: format!(
                        "Verify whether file '{}' still exists at the expected location.",
                        path
                    ),
                    priority: self.priority(),
                    content_snippet: Some(truncate(&fact.content, 200)),
                    fact_id_b: None,
                    content_b: None,
                    namespace: fact.namespace.clone(),
                    date: None,
                });
                break; // One gap per fact.
            }
        }

        Ok(gaps)
    }

    fn prompt_for_gap(&self, gap: &DetectedGap) -> String {
        let snippet = gap.content_snippet.as_deref().unwrap_or(&gap.description);
        format!(
            "The knowledge base references a file path: {}. Verify whether this file still exists \
             at the expected location and whether the reference is accurate.",
            snippet
        )
    }

    fn priority(&self) -> f64 {
        0.8
    }

    fn frequency(&self) -> usize {
        15
    }
}

// ---------------------------------------------------------------------------
// 3. DetectContradictions
// ---------------------------------------------------------------------------

/// Mission: detect pairs of facts with contradictory claims.
#[derive(Debug, Clone)]
pub struct DetectContradictionsMission;

impl MissionImpl for DetectContradictionsMission {
    fn name(&self) -> &str {
        "detect-contradictions"
    }

    fn search_queries(&self) -> Vec<MissionQuery> {
        // One query per priority namespace, top_k=15 each.
        PRIORITY_NAMESPACES
            .iter()
            .map(|ns| MissionQuery {
                query: format!("facts in {} namespace", ns),
                namespaces: Some(vec![ns.to_string()]),
                top_k: 15,
            })
            .collect()
    }

    async fn detect_issues(
        &self,
        http_base_url: &str,
        attempted: &HashSet<String>,
    ) -> Result<Vec<DetectedGap>> {
        let mut all_results = Vec::new();
        for q in self.search_queries() {
            let results = http_search(http_base_url, &q).await.unwrap_or_default();
            all_results.extend(results);
        }

        // Group by namespace for within-namespace comparison.
        let mut by_namespace: std::collections::HashMap<String, Vec<&SearchResult>> =
            std::collections::HashMap::new();
        for fact in &all_results {
            if let Some(ref ns) = fact.namespace {
                if is_skip_namespace(ns) {
                    continue;
                }
                by_namespace.entry(ns.clone()).or_default().push(fact);
            }
        }

        let mut gaps = Vec::new();
        for (namespace, facts) in &by_namespace {
            if facts.len() < 2 {
                continue;
            }

            // Limit pairs to avoid O(n²) blowup.
            let max_pairs = 20.min(facts.len() * (facts.len() - 1) / 2);
            let mut checked = 0usize;
            'pair_loop: for i in 0..facts.len() {
                for j in (i + 1)..facts.len() {
                    if checked >= max_pairs {
                        break 'pair_loop;
                    }
                    checked += 1;

                    let pair_key = format!("{}|{}+contradiction-gap", facts[i].id, facts[j].id);
                    if attempted.contains(&pair_key) {
                        continue;
                    }

                    let jaccard = jaccard_similarity(&facts[i].content, &facts[j].content);
                    if jaccard > 0.3
                        && jaccard <= 0.8
                        && has_contradiction_signals(&facts[i].content, &facts[j].content)
                    {
                        gaps.push(DetectedGap {
                            gap_type: GapType::ContradictionGap,
                            fact_id: facts[i].id.clone(),
                            description: format!(
                                "Fact '{}' may contradict fact '{}' in namespace '{}' (Jaccard: {:.2}).",
                                facts[i].id, facts[j].id, namespace, jaccard
                            ),
                            suggested_task: "Analyze whether this is a real contradiction or a scope/time difference.".to_string(),
                            priority: self.priority(),
                            content_snippet: Some(truncate(&facts[i].content, 200)),
                            fact_id_b: Some(facts[j].id.clone()),
                            content_b: Some(truncate(&facts[j].content, 200)),
                            namespace: Some(namespace.clone()),
                            date: None,
                        });
                        break 'pair_loop;
                    }
                }
            }
        }

        Ok(gaps)
    }

    fn prompt_for_gap(&self, gap: &DetectedGap) -> String {
        let fact_a = gap.content_snippet.as_deref().unwrap_or(&gap.description);
        let fact_b = gap
            .content_b
            .as_deref()
            .or(gap.fact_id_b.as_deref())
            .unwrap_or("another fact");
        format!(
            "Two facts may contradict each other: '{}' vs '{}'. \
             Analyze whether this is a real contradiction or a scope/time difference.",
            fact_a, fact_b
        )
    }

    fn priority(&self) -> f64 {
        0.85
    }

    fn frequency(&self) -> usize {
        10
    }
}

// ---------------------------------------------------------------------------
// 4. AuditNamespaceCompleteness
// ---------------------------------------------------------------------------

/// Mission: audit each namespace for graph connectivity and coverage.
#[derive(Debug, Clone)]
pub struct AuditNamespaceCompletenessMission;

impl MissionImpl for AuditNamespaceCompletenessMission {
    fn name(&self) -> &str {
        "audit-namespace-completeness"
    }

    fn search_queries(&self) -> Vec<MissionQuery> {
        PRIORITY_NAMESPACES
            .iter()
            .map(|ns| MissionQuery {
                query: format!("facts in {} namespace", ns),
                namespaces: Some(vec![ns.to_string()]),
                top_k: 5,
            })
            .collect()
    }

    async fn detect_issues(
        &self,
        http_base_url: &str,
        attempted: &HashSet<String>,
    ) -> Result<Vec<DetectedGap>> {
        let mut all_results = Vec::new();
        for q in self.search_queries() {
            let results = http_search(http_base_url, &q).await.unwrap_or_default();
            all_results.extend(results);
        }

        let counts = count_by_namespace(&all_results);
        let mut gaps = Vec::new();

        for (namespace, count) in &counts {
            let key = format!("ns:{}+missing-context", namespace);
            if attempted.contains(&key) {
                continue;
            }

            // Flag namespaces with fewer than 3 facts as underdeveloped.
            if *count < 3 {
                gaps.push(DetectedGap {
                    gap_type: GapType::MissingContext,
                    fact_id: format!("ns:{}", namespace),
                    description: format!(
                        "Namespace '{}' has only {} facts and may be underdeveloped.",
                        namespace, count
                    ),
                    suggested_task: format!(
                        "Search for additional information that should be stored in namespace '{}'.",
                        namespace
                    ),
                    priority: self.priority(),
                    content_snippet: None,
                    fact_id_b: None,
                    content_b: None,
                    namespace: Some(namespace.clone()),
                    date: None,
                });
            }
        }

        // Also check for namespaces that are known but returned zero results.
        for ns in PRIORITY_NAMESPACES {
            if is_skip_namespace(ns) {
                continue;
            }
            if !counts.contains_key(*ns) {
                let key = format!("ns:{}+missing-context", ns);
                if attempted.contains(&key) {
                    continue;
                }
                gaps.push(DetectedGap {
                    gap_type: GapType::MissingContext,
                    fact_id: format!("ns:{}", ns),
                    description: format!(
                        "Namespace '{}' returned no facts — it may be empty or missing.",
                        ns
                    ),
                    suggested_task: format!(
                        "Search for information that should be stored in namespace '{}'.",
                        ns
                    ),
                    priority: self.priority(),
                    content_snippet: None,
                    fact_id_b: None,
                    content_b: None,
                    namespace: Some(ns.to_string()),
                    date: None,
                });
            }
        }

        Ok(gaps)
    }

    fn prompt_for_gap(&self, gap: &DetectedGap) -> String {
        let ns = gap.namespace.as_deref().unwrap_or("this");
        let count = if gap.description.contains("only") {
            gap.description.clone()
        } else {
            "this namespace".to_string()
        };
        format!(
            "The namespace '{}' has {} and may be underdeveloped. \
             Search for additional information that should be stored in this namespace.",
            ns, count
        )
    }

    fn priority(&self) -> f64 {
        0.5
    }

    fn frequency(&self) -> usize {
        30
    }
}

// ---------------------------------------------------------------------------
// 5. TraceProvenanceChains
// ---------------------------------------------------------------------------

/// Mission: trace provenance/source attributions to verify they still exist.
#[derive(Debug, Clone)]
pub struct TraceProvenanceChainsMission;

impl MissionImpl for TraceProvenanceChainsMission {
    fn name(&self) -> &str {
        "trace-provenance-chains"
    }

    fn search_queries(&self) -> Vec<MissionQuery> {
        vec![MissionQuery {
            query: "source provenance evidence receipt verification claim".to_string(),
            namespaces: Some(vec![
                "projects".to_string(),
                "research".to_string(),
                "doctrine".to_string(),
            ]),
            top_k: 20,
        }]
    }

    async fn detect_issues(
        &self,
        http_base_url: &str,
        attempted: &HashSet<String>,
    ) -> Result<Vec<DetectedGap>> {
        let mut all_results = Vec::new();
        for q in self.search_queries() {
            let results = http_search(http_base_url, &q).await.unwrap_or_default();
            all_results.extend(results);
        }

        let mut gaps = Vec::new();
        for fact in &all_results {
            let key = format!("{}+stale-fact", fact.id);
            if attempted.contains(&key) {
                continue;
            }

            // Check if the fact has provenance-related content.
            let content_lower = fact.content.to_lowercase();
            let has_provenance = content_lower.contains("source:")
                || content_lower.contains("provenance")
                || content_lower.contains("evidence")
                || content_lower.contains("receipt")
                || content_lower.contains("verified");

            if has_provenance {
                let snippet = truncate(&fact.content, 200);
                gaps.push(DetectedGap {
                    gap_type: GapType::StaleFact,
                    fact_id: fact.id.clone(),
                    description: format!(
                        "Fact '{}' has a provenance claim. Verify the source exists and supports the claim. Content: {}",
                        fact.id,
                        truncate(&fact.content, 120)
                    ),
                    suggested_task: "Verify the source exists and supports the claim.".to_string(),
                    priority: self.priority(),
                    content_snippet: Some(snippet),
                    fact_id_b: None,
                    content_b: None,
                    namespace: fact.namespace.clone(),
                    date: None,
                });
            }
        }

        Ok(gaps)
    }

    fn prompt_for_gap(&self, gap: &DetectedGap) -> String {
        let snippet = gap.content_snippet.as_deref().unwrap_or(&gap.description);
        format!(
            "The fact '{}' has a provenance claim. Verify the source exists and supports the claim.",
            snippet
        )
    }

    fn priority(&self) -> f64 {
        0.7
    }

    fn frequency(&self) -> usize {
        25
    }
}

// ---------------------------------------------------------------------------
// 6. FindDuplicates
// ---------------------------------------------------------------------------

/// Mission: find duplicate facts with high content overlap across namespaces.
#[derive(Debug, Clone)]
pub struct FindDuplicatesMission;

impl MissionImpl for FindDuplicatesMission {
    fn name(&self) -> &str {
        "find-duplicates"
    }

    fn search_queries(&self) -> Vec<MissionQuery> {
        vec![
            MissionQuery {
                query: "knowledge base facts overview summary".to_string(),
                namespaces: None,
                top_k: 20,
            },
            MissionQuery {
                query: "project crate library information".to_string(),
                namespaces: None,
                top_k: 20,
            },
        ]
    }

    async fn detect_issues(
        &self,
        http_base_url: &str,
        attempted: &HashSet<String>,
    ) -> Result<Vec<DetectedGap>> {
        let mut all_results = Vec::new();
        for q in self.search_queries() {
            let results = http_search(http_base_url, &q).await.unwrap_or_default();
            all_results.extend(results);
        }

        let mut gaps = Vec::new();
        let mut seen_pairs: HashSet<String> = HashSet::new();

        for i in 0..all_results.len() {
            for j in (i + 1)..all_results.len() {
                let pair_key =
                    format!("{}|{}+duplicate-fact", all_results[i].id, all_results[j].id);
                let rev_pair =
                    format!("{}|{}+duplicate-fact", all_results[j].id, all_results[i].id);
                if attempted.contains(&pair_key)
                    || attempted.contains(&rev_pair)
                    || seen_pairs.contains(&pair_key)
                    || seen_pairs.contains(&rev_pair)
                {
                    continue;
                }

                let jaccard = jaccard_similarity(&all_results[i].content, &all_results[j].content);
                if jaccard > 0.7 {
                    seen_pairs.insert(pair_key.clone());
                    gaps.push(DetectedGap {
                        gap_type: GapType::DuplicateFact,
                        fact_id: all_results[i].id.clone(),
                        description: format!(
                            "Fact '{}' appears to duplicate fact '{}' (Jaccard similarity: {:.2}).",
                            all_results[i].id, all_results[j].id, jaccard
                        ),
                        suggested_task: "Determine which version is more complete and accurate."
                            .to_string(),
                        priority: self.priority(),
                        content_snippet: Some(truncate(&all_results[i].content, 200)),
                        fact_id_b: Some(all_results[j].id.clone()),
                        content_b: Some(truncate(&all_results[j].content, 200)),
                        namespace: all_results[i].namespace.clone(),
                        date: None,
                    });
                }
            }
            // Limit total gaps.
            if gaps.len() >= 15 {
                break;
            }
        }

        Ok(gaps)
    }

    fn prompt_for_gap(&self, gap: &DetectedGap) -> String {
        let snippet = gap.content_snippet.as_deref().unwrap_or(&gap.description);
        format!(
            "A fact appears to duplicate another: '{}'. \
             Determine which version is more complete and accurate.",
            snippet
        )
    }

    fn priority(&self) -> f64 {
        0.6
    }

    fn frequency(&self) -> usize {
        20
    }
}

// ---------------------------------------------------------------------------
// 7. VerifyCodebaseSync
// ---------------------------------------------------------------------------

/// Mission: check if codebase metric facts match the current state.
#[derive(Debug, Clone)]
pub struct VerifyCodebaseSyncMission;

impl MissionImpl for VerifyCodebaseSyncMission {
    fn name(&self) -> &str {
        "verify-codebase-sync"
    }

    fn search_queries(&self) -> Vec<MissionQuery> {
        vec![MissionQuery {
            query: "crates count LOC tests workspace members".to_string(),
            namespaces: Some(vec!["projects".to_string(), "libraries".to_string()]),
            top_k: 20,
        }]
    }

    async fn detect_issues(
        &self,
        http_base_url: &str,
        attempted: &HashSet<String>,
    ) -> Result<Vec<DetectedGap>> {
        let mut all_results = Vec::new();
        for q in self.search_queries() {
            let results = http_search(http_base_url, &q).await.unwrap_or_default();
            all_results.extend(results);
        }

        let mut gaps = Vec::new();
        // Look for facts mentioning specific counts (N crates, N tests, N LOC).
        let metric_patterns = [
            "crates",
            "tests",
            "LOC",
            "lines of code",
            "workspace members",
            "modules",
            "files",
        ];

        for fact in &all_results {
            let key = format!("{}+stale-fact", fact.id);
            if attempted.contains(&key) {
                continue;
            }

            let content_lower = fact.content.to_lowercase();
            let has_metric = metric_patterns
                .iter()
                .any(|p| content_lower.contains(&p.to_lowercase()));

            // Also check for numeric tokens near metric keywords.
            let has_numbers = fact
                .content
                .split_whitespace()
                .any(|w| w.chars().all(|c| c.is_ascii_digit()) && !w.is_empty());

            if has_metric && has_numbers {
                let snippet = truncate(&fact.content, 200);
                gaps.push(DetectedGap {
                    gap_type: GapType::StaleFact,
                    fact_id: fact.id.clone(),
                    description: format!(
                        "Fact '{}' references specific codebase metrics that may be stale. Content: {}",
                        fact.id,
                        truncate(&fact.content, 120)
                    ),
                    suggested_task: "Verify whether these codebase metrics are still accurate.".to_string(),
                    priority: self.priority(),
                    content_snippet: Some(snippet),
                    fact_id_b: None,
                    content_b: None,
                    namespace: fact.namespace.clone(),
                    date: None,
                });
            }
        }

        Ok(gaps)
    }

    fn prompt_for_gap(&self, gap: &DetectedGap) -> String {
        let snippet = gap.content_snippet.as_deref().unwrap_or(&gap.description);
        format!(
            "The knowledge base says: '{}'. This references specific codebase metrics. \
             Verify whether these numbers are still accurate.",
            snippet
        )
    }

    fn priority(&self) -> f64 {
        0.75
    }

    fn frequency(&self) -> usize {
        15
    }
}

// ---------------------------------------------------------------------------
// 8. StaleDateDetection
// ---------------------------------------------------------------------------

/// Mission: find facts referencing old dates or versions.
#[derive(Debug, Clone)]
pub struct StaleDateDetectionMission;

impl MissionImpl for StaleDateDetectionMission {
    fn name(&self) -> &str {
        "stale-date-detection"
    }

    fn search_queries(&self) -> Vec<MissionQuery> {
        vec![MissionQuery {
            query: "date version release updated published".to_string(),
            namespaces: None,
            top_k: 30,
        }]
    }

    async fn detect_issues(
        &self,
        http_base_url: &str,
        attempted: &HashSet<String>,
    ) -> Result<Vec<DetectedGap>> {
        let mut all_results = Vec::new();
        for q in self.search_queries() {
            let results = http_search(http_base_url, &q).await.unwrap_or_default();
            all_results.extend(results);
        }

        let mut gaps = Vec::new();
        for fact in &all_results {
            let key = format!("{}+stale-by-date", fact.id);
            if attempted.contains(&key) {
                continue;
            }

            // Look for full dates (20XX-MM-DD or 20XX-MM) first.
            let full_dates = extract_full_dates(&fact.content);
            let mut stale_date: Option<String> = None;

            for d in &full_dates {
                if is_stale_date(d) {
                    stale_date = Some(d.clone());
                    break;
                }
            }

            // Fallback to year-only dates.
            if stale_date.is_none() {
                let years = extract_year_strings(&fact.content);
                for y in &years {
                    if is_stale_date(y) {
                        stale_date = Some(y.clone());
                        break;
                    }
                }
            }

            if let Some(date) = stale_date {
                let snippet = truncate(&fact.content, 200);
                gaps.push(DetectedGap {
                    gap_type: GapType::StaleByDate,
                    fact_id: fact.id.clone(),
                    description: format!(
                        "Fact '{}' references date {} which may be outdated. Content: {}",
                        fact.id,
                        date,
                        truncate(&fact.content, 120)
                    ),
                    suggested_task: "Check if the information is still current.".to_string(),
                    priority: self.priority(),
                    content_snippet: Some(snippet),
                    fact_id_b: None,
                    content_b: None,
                    namespace: fact.namespace.clone(),
                    date: Some(date),
                });
            }
        }

        Ok(gaps)
    }

    fn prompt_for_gap(&self, gap: &DetectedGap) -> String {
        let date = gap.date.as_deref().unwrap_or("an old date");
        let snippet = gap.content_snippet.as_deref().unwrap_or(&gap.description);
        format!(
            "A fact references date {} which may be outdated: '{}'. \
             Check if the information is still current.",
            date, snippet
        )
    }

    fn priority(&self) -> f64 {
        0.65
    }

    fn frequency(&self) -> usize {
        10
    }
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // --- Mission enum tests ---

    #[test]
    fn mission_display_and_kebab() {
        assert_eq!(
            Mission::VerifyPublishedCrates.to_string(),
            "verify-published-crates"
        );
        assert_eq!(
            Mission::VerifyFileReferences.as_kebab(),
            "verify-file-references"
        );
        assert_eq!(
            Mission::DetectContradictions.as_kebab(),
            "detect-contradictions"
        );
        assert_eq!(
            Mission::AuditNamespaceCompleteness.as_kebab(),
            "audit-namespace-completeness"
        );
        assert_eq!(
            Mission::TraceProvenanceChains.as_kebab(),
            "trace-provenance-chains"
        );
        assert_eq!(Mission::FindDuplicates.as_kebab(), "find-duplicates");
        assert_eq!(
            Mission::VerifyCodebaseSync.as_kebab(),
            "verify-codebase-sync"
        );
        assert_eq!(
            Mission::StaleDateDetection.as_kebab(),
            "stale-date-detection"
        );
    }

    #[test]
    fn mission_impl_returns_correct_priority() {
        assert!((VerifyPublishedCratesMission.priority() - 0.9).abs() < f64::EPSILON);
        assert!((VerifyFileReferencesMission.priority() - 0.8).abs() < f64::EPSILON);
        assert!((DetectContradictionsMission.priority() - 0.85).abs() < f64::EPSILON);
        assert!((AuditNamespaceCompletenessMission.priority() - 0.5).abs() < f64::EPSILON);
        assert!((TraceProvenanceChainsMission.priority() - 0.7).abs() < f64::EPSILON);
        assert!((FindDuplicatesMission.priority() - 0.6).abs() < f64::EPSILON);
        assert!((VerifyCodebaseSyncMission.priority() - 0.75).abs() < f64::EPSILON);
        assert!((StaleDateDetectionMission.priority() - 0.65).abs() < f64::EPSILON);
    }

    #[test]
    fn mission_impl_returns_correct_frequency() {
        assert_eq!(VerifyPublishedCratesMission.frequency(), 20);
        assert_eq!(VerifyFileReferencesMission.frequency(), 15);
        assert_eq!(DetectContradictionsMission.frequency(), 10);
        assert_eq!(AuditNamespaceCompletenessMission.frequency(), 30);
        assert_eq!(TraceProvenanceChainsMission.frequency(), 25);
        assert_eq!(FindDuplicatesMission.frequency(), 20);
        assert_eq!(VerifyCodebaseSyncMission.frequency(), 15);
        assert_eq!(StaleDateDetectionMission.frequency(), 10);
    }

    #[test]
    fn mission_impl_names_match() {
        assert_eq!(
            VerifyPublishedCratesMission.name(),
            "verify-published-crates"
        );
        assert_eq!(VerifyFileReferencesMission.name(), "verify-file-references");
        assert_eq!(DetectContradictionsMission.name(), "detect-contradictions");
        assert_eq!(
            AuditNamespaceCompletenessMission.name(),
            "audit-namespace-completeness"
        );
        assert_eq!(
            TraceProvenanceChainsMission.name(),
            "trace-provenance-chains"
        );
        assert_eq!(FindDuplicatesMission.name(), "find-duplicates");
        assert_eq!(VerifyCodebaseSyncMission.name(), "verify-codebase-sync");
        assert_eq!(StaleDateDetectionMission.name(), "stale-date-detection");
    }

    // --- Search query tests ---

    #[test]
    fn verify_published_crates_search_queries() {
        let queries = VerifyPublishedCratesMission.search_queries();
        assert_eq!(queries.len(), 1);
        assert!(queries[0].query.contains("crate version"));
        assert!(queries[0].namespaces.is_some());
        assert_eq!(queries[0].top_k, 20);
    }

    #[test]
    fn verify_file_references_search_queries() {
        let queries = VerifyFileReferencesMission.search_queries();
        assert_eq!(queries.len(), 1);
        assert!(queries[0].query.contains("file path"));
        assert_eq!(queries[0].top_k, 20);
    }

    #[test]
    fn detect_contradictions_has_one_query_per_namespace() {
        let queries = DetectContradictionsMission.search_queries();
        assert_eq!(queries.len(), PRIORITY_NAMESPACES.len());
        for q in &queries {
            assert_eq!(q.top_k, 15);
        }
    }

    #[test]
    fn audit_namespace_completeness_queries() {
        let queries = AuditNamespaceCompletenessMission.search_queries();
        assert_eq!(queries.len(), PRIORITY_NAMESPACES.len());
        for q in &queries {
            assert_eq!(q.top_k, 5);
        }
    }

    #[test]
    fn find_duplicates_has_broad_queries() {
        let queries = FindDuplicatesMission.search_queries();
        assert!(queries.len() >= 1);
        assert!(queries.iter().all(|q| q.top_k >= 20));
    }

    #[test]
    fn stale_date_detection_top_k_30() {
        let queries = StaleDateDetectionMission.search_queries();
        assert_eq!(queries[0].top_k, 30);
    }

    // --- Prompt tests ---

    #[test]
    fn verify_published_crates_prompt() {
        let gap = DetectedGap {
            gap_type: GapType::StaleFact,
            fact_id: "fact:1".to_string(),
            description: "d".to_string(),
            suggested_task: "t".to_string(),
            priority: 0.9,
            content_snippet: Some("semantic-memory v0.5.0".to_string()),
            fact_id_b: None,
            content_b: None,
            namespace: None,
            date: None,
        };
        let prompt = VerifyPublishedCratesMission.prompt_for_gap(&gap);
        assert!(prompt.contains("version information"));
        assert!(prompt.contains("semantic-memory v0.5.0"));
    }

    #[test]
    fn verify_file_references_prompt() {
        let gap = DetectedGap {
            gap_type: GapType::StaleFact,
            fact_id: "fact:2".to_string(),
            description: "d".to_string(),
            suggested_task: "t".to_string(),
            priority: 0.8,
            content_snippet: Some("src/lib.rs".to_string()),
            fact_id_b: None,
            content_b: None,
            namespace: None,
            date: None,
        };
        let prompt = VerifyFileReferencesMission.prompt_for_gap(&gap);
        assert!(prompt.contains("file path"));
        assert!(prompt.contains("src/lib.rs"));
    }

    #[test]
    fn detect_contradictions_prompt() {
        let gap = DetectedGap {
            gap_type: GapType::ContradictionGap,
            fact_id: "fact:a".to_string(),
            description: "d".to_string(),
            suggested_task: "t".to_string(),
            priority: 0.85,
            content_snippet: Some("Rust has 49 tests".to_string()),
            fact_id_b: Some("fact:b".to_string()),
            content_b: None,
            namespace: Some("general".to_string()),
            date: None,
        };
        let prompt = DetectContradictionsMission.prompt_for_gap(&gap);
        assert!(prompt.contains("contradict"));
        assert!(prompt.contains("Rust has 49 tests"));
        assert!(prompt.contains("fact:b"));
    }

    #[test]
    fn audit_namespace_completeness_prompt() {
        let gap = DetectedGap {
            gap_type: GapType::MissingContext,
            fact_id: "ns:general".to_string(),
            description: "only 2 facts".to_string(),
            suggested_task: "t".to_string(),
            priority: 0.5,
            content_snippet: None,
            fact_id_b: None,
            content_b: None,
            namespace: Some("general".to_string()),
            date: None,
        };
        let prompt = AuditNamespaceCompletenessMission.prompt_for_gap(&gap);
        assert!(prompt.contains("general"));
        assert!(prompt.contains("underdeveloped"));
    }

    #[test]
    fn trace_provenance_chains_prompt() {
        let gap = DetectedGap {
            gap_type: GapType::StaleFact,
            fact_id: "fact:3".to_string(),
            description: "d".to_string(),
            suggested_task: "t".to_string(),
            priority: 0.7,
            content_snippet: Some("source: https://example.com".to_string()),
            fact_id_b: None,
            content_b: None,
            namespace: None,
            date: None,
        };
        let prompt = TraceProvenanceChainsMission.prompt_for_gap(&gap);
        assert!(prompt.contains("provenance"));
        assert!(prompt.contains("source"));
    }

    #[test]
    fn find_duplicates_prompt() {
        let gap = DetectedGap {
            gap_type: GapType::DuplicateFact,
            fact_id: "fact:4".to_string(),
            description: "d".to_string(),
            suggested_task: "t".to_string(),
            priority: 0.6,
            content_snippet: Some("Rust is a systems language".to_string()),
            fact_id_b: Some("fact:5".to_string()),
            content_b: None,
            namespace: None,
            date: None,
        };
        let prompt = FindDuplicatesMission.prompt_for_gap(&gap);
        assert!(prompt.contains("duplicate"));
        assert!(prompt.contains("Rust is a systems language"));
    }

    #[test]
    fn verify_codebase_sync_prompt() {
        let gap = DetectedGap {
            gap_type: GapType::StaleFact,
            fact_id: "fact:6".to_string(),
            description: "d".to_string(),
            suggested_task: "t".to_string(),
            priority: 0.75,
            content_snippet: Some("The workspace has 12 crates and 91 tests".to_string()),
            fact_id_b: None,
            content_b: None,
            namespace: None,
            date: None,
        };
        let prompt = VerifyCodebaseSyncMission.prompt_for_gap(&gap);
        assert!(prompt.contains("codebase metrics"));
        assert!(prompt.contains("12 crates"));
    }

    #[test]
    fn stale_date_detection_prompt() {
        let gap = DetectedGap {
            gap_type: GapType::StaleByDate,
            fact_id: "fact:7".to_string(),
            description: "d".to_string(),
            suggested_task: "t".to_string(),
            priority: 0.65,
            content_snippet: Some("Released 2024-01".to_string()),
            fact_id_b: None,
            content_b: None,
            namespace: None,
            date: Some("2024-01".to_string()),
        };
        let prompt = StaleDateDetectionMission.prompt_for_gap(&gap);
        assert!(prompt.contains("2024-01"));
        assert!(prompt.contains("outdated"));
    }

    // --- MissionScheduler tests ---

    #[test]
    fn scheduler_new_creates_8_missions() {
        let scheduler = MissionScheduler::new();
        assert_eq!(scheduler.scheduled_missions().len(), 8);
    }

    #[test]
    fn scheduler_next_mission_returns_highest_priority_due() {
        let scheduler = MissionScheduler::new();
        // At iteration 0, all missions are due. VerifyPublishedCrates has
        // the highest base priority (0.9).
        let next = scheduler.next_mission(0);
        assert!(next.is_some());
        assert_eq!(next.unwrap(), &Mission::VerifyPublishedCrates);
    }

    #[test]
    fn scheduler_next_mission_none_when_all_recently_run() {
        let mut scheduler = MissionScheduler::new();
        // Record a run for all missions at iteration 5.
        for sm in scheduler.scheduled_missions().to_vec() {
            scheduler.record_result(sm.mission.as_kebab(), 0, 5);
        }
        // At iteration 5 + 1, no mission should be due (minimum frequency is 10).
        assert!(scheduler.next_mission(6).is_none());
    }

    #[test]
    fn scheduler_record_result_lowers_priority_on_zero_issues() {
        let mut scheduler = MissionScheduler::new();
        let initial = scheduler.priority_of("verify-published-crates").unwrap();
        assert!((initial - 0.9).abs() < f64::EPSILON);

        scheduler.record_result("verify-published-crates", 0, 10);
        let after = scheduler.priority_of("verify-published-crates").unwrap();
        assert!((after - 0.8).abs() < f64::EPSILON);
    }

    #[test]
    fn scheduler_record_result_resets_priority_on_issues() {
        let mut scheduler = MissionScheduler::new();
        // Lower it first.
        scheduler.record_result("verify-published-crates", 0, 10);
        assert!(
            (scheduler.priority_of("verify-published-crates").unwrap() - 0.8).abs() < f64::EPSILON
        );

        // Now find issues — priority should reset to base (0.9).
        scheduler.record_result("verify-published-crates", 5, 30);
        assert!(
            (scheduler.priority_of("verify-published-crates").unwrap() - 0.9).abs() < f64::EPSILON
        );
    }

    #[test]
    fn scheduler_priority_floor_at_0_1() {
        let mut scheduler = MissionScheduler::new();
        // Repeatedly record zero issues to floor the priority.
        for i in 0..20 {
            scheduler.record_result("audit-namespace-completeness", 0, i * 30);
        }
        let priority = scheduler
            .priority_of("audit-namespace-completeness")
            .unwrap();
        assert!((priority - 0.1).abs() < f64::EPSILON);
    }

    #[test]
    fn scheduler_mission_becomes_due_after_frequency() {
        let mut scheduler = MissionScheduler::new();
        // Run DetectContradictions at iteration 5 (frequency = 10).
        scheduler.record_result("detect-contradictions", 2, 5);

        // At iteration 14, not yet due.
        assert!(scheduler
            .next_mission(14)
            .map(|m| m != &Mission::DetectContradictions)
            .unwrap_or(true));

        // At iteration 15, it should be due again.
        // But VerifyPublishedCrates (0.9) might still be higher priority
        // and also due. Let's test with a scheduler where only
        // DetectContradictions is present.
        let sm = ScheduledMission {
            mission: Mission::DetectContradictions,
            last_run: 5,
            issues_found: 2,
            dynamic_priority: 0.85,
        };
        let sched2 = MissionScheduler::with_missions(vec![sm]);
        assert!(sched2.next_mission(15).is_some());
        assert_eq!(
            sched2.next_mission(15).unwrap(),
            &Mission::DetectContradictions
        );
    }

    #[test]
    fn scheduler_default_is_new() {
        let scheduler = MissionScheduler::default();
        assert_eq!(scheduler.scheduled_missions().len(), 8);
    }

    // --- Helper function tests ---

    #[test]
    fn extract_versions_finds_v_versions() {
        let versions = extract_versions("semantic-memory v0.5.0 and claim-ledger v1.2.3");
        assert_eq!(versions.len(), 2);
        assert!(versions.contains(&"v0.5.0".to_string()));
        assert!(versions.contains(&"v1.2.3".to_string()));
    }

    #[test]
    fn extract_versions_no_match() {
        assert!(extract_versions("no versions here").is_empty());
    }

    #[test]
    fn extract_file_paths_finds_rs_and_toml() {
        let paths = extract_file_paths("see src/lib.rs and crates/foo/Cargo.toml for details");
        assert!(!paths.is_empty());
        assert!(paths.iter().any(|p| p.contains("src/lib.rs")));
        assert!(paths.iter().any(|p| p.contains("Cargo.toml")));
    }

    #[test]
    fn extract_full_dates_finds_date_strings() {
        let dates = extract_full_dates("Updated 2024-01-15 and 2023.06.01");
        assert_eq!(dates.len(), 2);
        assert!(dates[0].starts_with("2024"));
    }

    #[test]
    fn is_stale_date_old_date_is_stale() {
        assert!(is_stale_date("2023-01-15"));
        assert!(is_stale_date("2024-06"));
    }

    #[test]
    fn is_stale_date_recent_date_not_stale() {
        // Use today's date so the test remains deterministic throughout the year.
        let now = chrono::Utc::now();
        let recent = now.format("%Y-%m-%d").to_string();
        assert!(!is_stale_date(&recent));
    }

    #[test]
    fn jaccard_similarity_identical() {
        let s = "the quick brown fox";
        assert!((jaccard_similarity(s, s) - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn jaccard_similarity_disjoint() {
        assert!((jaccard_similarity("apple banana", "cherry date")).abs() < f64::EPSILON);
    }

    #[test]
    fn contradiction_signals_negation() {
        assert!(has_contradiction_signals(
            "Rust is a systems language",
            "Rust is not a systems language"
        ));
    }

    #[test]
    fn contradiction_signals_different_numbers() {
        assert!(has_contradiction_signals(
            "The crate has 49 tests",
            "The crate has 50 tests"
        ));
    }

    #[test]
    fn no_contradiction_for_similar() {
        assert!(!has_contradiction_signals(
            "Rust is a systems programming language",
            "Rust is a systems programming language with memory safety"
        ));
    }

    #[test]
    fn truncate_short_string_unchanged() {
        assert_eq!(truncate("hello", 120), "hello");
    }

    #[test]
    fn truncate_long_string_appends_ellipsis() {
        let long = "a".repeat(200);
        let t = truncate(&long, 100);
        assert!(t.ends_with('…'));
        // The ellipsis char is 3 bytes, so total is 100 + 3 = 103 bytes.
        assert!(t.len() <= 103);
    }

    #[test]
    fn parse_search_results_extracts_facts() {
        let data = serde_json::json!({
            "results": [
                {
                    "result_id": "fact:aaa",
                    "content": "Test content",
                    "namespace": "general",
                    "score": 0.95
                },
                {
                    "result_id": "fact:bbb",
                    "content": "Another fact",
                    "namespace": "coding",
                    "score": 0.82
                }
            ]
        });
        let results = parse_search_results(&data).unwrap();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].id, "fact:aaa");
        assert_eq!(results[0].namespace.as_deref(), Some("general"));
    }

    #[test]
    fn parse_search_results_deduplicates() {
        let data = serde_json::json!({
            "results": [
                {"result_id": "fact:aaa", "content": "a", "namespace": "g", "score": 0.9},
                {"result_id": "fact:aaa", "content": "a", "namespace": "g", "score": 0.9},
            ]
        });
        let results = parse_search_results(&data).unwrap();
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn parse_search_results_empty() {
        let data = serde_json::json!({});
        let results = parse_search_results(&data).unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn count_by_namespace_groups_correctly() {
        let results = vec![
            SearchResult {
                id: "a".into(),
                content: "x".into(),
                namespace: Some("g".into()),
                score: 0.9,
            },
            SearchResult {
                id: "b".into(),
                content: "y".into(),
                namespace: Some("g".into()),
                score: 0.8,
            },
            SearchResult {
                id: "c".into(),
                content: "z".into(),
                namespace: Some("c".into()),
                score: 0.7,
            },
        ];
        let counts = count_by_namespace(&results);
        assert_eq!(counts.get("g"), Some(&2));
        assert_eq!(counts.get("c"), Some(&1));
    }

    #[test]
    fn count_by_namespace_skips_skip_namespaces() {
        let results = vec![
            SearchResult {
                id: "a".into(),
                content: "x".into(),
                namespace: Some("test".into()),
                score: 0.9,
            },
            SearchResult {
                id: "b".into(),
                content: "y".into(),
                namespace: Some("general".into()),
                score: 0.8,
            },
        ];
        let counts = count_by_namespace(&results);
        assert!(!counts.contains_key("test"));
        assert!(counts.contains_key("general"));
    }

    #[test]
    fn mission_enum_all_variants_are_distinct() {
        let all = [
            Mission::VerifyPublishedCrates,
            Mission::VerifyFileReferences,
            Mission::DetectContradictions,
            Mission::AuditNamespaceCompleteness,
            Mission::TraceProvenanceChains,
            Mission::FindDuplicates,
            Mission::VerifyCodebaseSync,
            Mission::StaleDateDetection,
        ];
        // Verify all kebab names are unique.
        let names: Vec<&str> = all.iter().map(|m| m.as_kebab()).collect();
        let unique: std::collections::HashSet<&str> = names.iter().copied().collect();
        assert_eq!(names.len(), unique.len());
    }

    #[test]
    fn mission_impl_trait_object_works() {
        let mission = Mission::VerifyPublishedCrates;
        assert_eq!(mission.as_kebab(), "verify-published-crates");
        assert!((mission.base_priority() - 0.9).abs() < f64::EPSILON);
        assert_eq!(mission.frequency(), 20);
    }

    #[test]
    fn scheduled_mission_is_due_initially() {
        let sm = ScheduledMission::new(Mission::VerifyPublishedCrates);
        assert!(sm.is_due(0));
        assert!(sm.is_due(100));
    }

    #[test]
    fn scheduled_mission_not_due_after_run() {
        let mut sm = ScheduledMission::new(Mission::DetectContradictions);
        sm.last_run = 10;
        // Frequency is 10.
        assert!(!sm.is_due(19));
        assert!(sm.is_due(20));
    }
}
