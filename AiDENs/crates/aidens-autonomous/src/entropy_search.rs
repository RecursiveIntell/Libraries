//! Entropy-gradient search (FEUT-001) + saturation tracker.
//!
//! Replaces random gap detection with entropy-gradient-guided domain
//! selection. Computes where knowledge is most uncertain and most
//! changing, prioritizes those areas.
//!
//! priority = entropy / (1 + structuring_score)
//!
//! High entropy + low structuring = explore now.
//! High entropy + high structuring = implement (well understood but large).
//! Low entropy = saturated, move on.
//!
//! The saturation tracker monitors candidate yield per domain. When yield
//! drops below threshold for N consecutive explorations, the domain is
//! declared saturated and focus shifts elsewhere.

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Statistics for a single domain (namespace) in semantic-memory.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainStats {
    /// Namespace / domain identifier.
    pub domain: String,
    /// Total facts in this domain.
    pub fact_count: usize,
    /// Graph edge count in this domain.
    pub edge_count: usize,
    /// Contradiction count in this domain.
    pub contradiction_count: usize,
    /// Average structuring score (from subtraction engine).
    /// 0.0 if no structuring data available.
    pub avg_structuring_score: f64,
}

/// Computed entropy metrics for a domain.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainEntropy {
    /// Namespace / domain identifier.
    pub domain: String,
    /// Total facts.
    pub fact_count: usize,
    /// Computed entropy: higher = more unknown/uncertain.
    pub entropy: f64,
    /// Computed gradient: how fast knowledge is changing.
    pub gradient: f64,
    /// Exploration priority: entropy / (1 + structuring).
    pub priority: f64,
    /// Whether this domain is saturated.
    pub saturated: bool,
}

// ---------------------------------------------------------------------------
// Config
// ---------------------------------------------------------------------------

/// Configuration for the entropy-gradient searcher.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntropySearchConfig {
    /// Number of recent cycles to consider for growth rate.
    pub growth_window: usize,
    /// Number of consecutive low-yield explorations before saturation.
    pub saturation_window: usize,
    /// Yield threshold below which a domain is considered low-yield.
    pub saturation_threshold: usize,
    /// Domains to always skip (ingestion artifacts).
    pub skip_domains: Vec<String>,
}

impl Default for EntropySearchConfig {
    fn default() -> Self {
        Self {
            growth_window: 10,
            saturation_window: 3,
            saturation_threshold: 2,
            skip_domains: vec![
                "mixed".into(),
                "chatgpt".into(),
                "twitter".into(),
                "test".into(),
                "tool-receipts".into(),
            ],
        }
    }
}

// ---------------------------------------------------------------------------
// Searcher
// ---------------------------------------------------------------------------

/// Entropy-gradient-guided domain searcher with saturation tracking.
#[derive(Debug, Clone)]
pub struct EntropyGradientSearcher {
    /// HTTP base URL for semantic-memory server.
    pub http_base_url: String,
    /// Configuration.
    config: EntropySearchConfig,
    /// Domains that have been declared saturated.
    saturated: HashSet<String>,
    /// Per-domain yield history (candidates found per exploration).
    yield_history: HashMap<String, VecDeque<usize>>,
    /// Per-domain fact count history (for gradient computation).
    fact_count_history: HashMap<String, VecDeque<usize>>,
    /// Exploration count per domain.
    exploration_count: HashMap<String, usize>,
}

impl EntropyGradientSearcher {
    /// Create a new searcher targeting the given semantic-memory HTTP server.
    pub fn new(http_base_url: impl Into<String>) -> Self {
        let mut url = http_base_url.into();
        if url.ends_with('/') {
            url.pop();
        }
        Self {
            http_base_url: url,
            config: EntropySearchConfig::default(),
            saturated: HashSet::new(),
            yield_history: HashMap::new(),
            fact_count_history: HashMap::new(),
            exploration_count: HashMap::new(),
        }
    }

    /// Create with custom config.
    pub fn with_config(http_base_url: impl Into<String>, config: EntropySearchConfig) -> Self {
        let mut url = http_base_url.into();
        if url.ends_with('/') {
            url.pop();
        }
        Self {
            http_base_url: url,
            config,
            saturated: HashSet::new(),
            yield_history: HashMap::new(),
            fact_count_history: HashMap::new(),
            exploration_count: HashMap::new(),
        }
    }

    /// Query semantic-memory for domain statistics via the /stats endpoint.
    async fn query_domain_stats(&self) -> Result<Vec<DomainStats>> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(15))
            .build()
            .map_err(|e| anyhow!("failed to build HTTP client: {e}"))?;

        let url = format!("{}/stats", self.http_base_url);
        let resp = client
            .get(&url)
            .send()
            .await
            .map_err(|e| anyhow!("GET /stats failed: {e}"))?;

        if !resp.status().is_success() {
            return Err(anyhow!("GET /stats returned {}", resp.status()));
        }

        let data: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| anyhow!("failed to parse /stats response: {e}"))?;

        let mut stats = Vec::new();

        // Parse namespaces from stats response.
        if let Some(namespaces) = data.get("namespaces").and_then(|v| v.as_array()) {
            for ns in namespaces {
                let domain = ns
                    .get("namespace")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                if domain.is_empty() || self.config.skip_domains.contains(&domain) {
                    continue;
                }
                let fact_count =
                    ns.get("fact_count").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
                let edge_count =
                    ns.get("edge_count").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
                let contradiction_count = ns
                    .get("contradiction_count")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0) as usize;
                let avg_structuring_score = ns
                    .get("avg_structuring_score")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0);

                stats.push(DomainStats {
                    domain,
                    fact_count,
                    edge_count,
                    contradiction_count,
                    avg_structuring_score,
                });
            }
        }

        Ok(stats)
    }

    /// Compute entropy for a domain.
    /// entropy = log2(fact_count + 1) * (1 + contradiction_count) / (1 + edge_count)
    /// More facts with fewer edges and more contradictions = higher entropy.
    fn compute_entropy(stats: &DomainStats) -> f64 {
        let fact_component = (stats.fact_count as f64 + 1.0).log2();
        let contradiction_factor = 1.0 + stats.contradiction_count as f64;
        let edge_factor = 1.0 + stats.edge_count as f64;
        fact_component * contradiction_factor / edge_factor
    }

    /// Compute gradient for a domain based on fact count history.
    /// gradient = (current_facts - oldest_facts_in_window) / window_size
    /// Normalized by total facts to keep it in a reasonable range.
    fn compute_gradient(&self, domain: &str, current_facts: usize) -> f64 {
        if let Some(history) = self.fact_count_history.get(domain) {
            if history.is_empty() {
                return 0.0;
            }
            let oldest = history.front().copied().unwrap_or(current_facts);
            let delta = current_facts.saturating_sub(oldest) as f64;
            let window = self.config.growth_window.max(1) as f64;
            delta / window / (current_facts as f64 + 1.0)
        } else {
            0.0
        }
    }

    /// Compute exploration priority.
    /// priority = entropy / (1 + structuring_score)
    fn compute_priority(entropy: f64, structuring: f64) -> f64 {
        entropy / (1.0 + structuring)
    }

    /// Rank domains by exploration priority.
    pub async fn rank_domains(&self) -> Result<Vec<DomainEntropy>> {
        let stats = self.query_domain_stats().await?;

        let mut results: Vec<DomainEntropy> = stats
            .iter()
            .map(|s| {
                let entropy = Self::compute_entropy(s);
                let gradient = self.compute_gradient(&s.domain, s.fact_count);
                let priority = Self::compute_priority(entropy, s.avg_structuring_score);
                let saturated = self.saturated.contains(&s.domain);
                DomainEntropy {
                    domain: s.domain.clone(),
                    fact_count: s.fact_count,
                    entropy,
                    gradient,
                    priority,
                    saturated,
                }
            })
            .collect();

        // Sort by priority descending.
        results.sort_by(|a, b| {
            b.priority
                .partial_cmp(&a.priority)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        Ok(results)
    }

    /// Get the top N domains to explore next (excluding saturated).
    pub async fn next_targets(&self, n: usize) -> Result<Vec<DomainEntropy>> {
        let ranked = self.rank_domains().await?;
        Ok(ranked
            .into_iter()
            .filter(|d| !d.saturated)
            .take(n)
            .collect())
    }

    /// Record exploration yield for a domain (for saturation tracking).
    /// Also updates fact count history for gradient computation.
    pub fn record_exploration(
        &mut self,
        domain: &str,
        candidates_found: usize,
        current_facts: usize,
    ) {
        // Update yield history.
        let yield_hist = self.yield_history.entry(domain.to_string()).or_default();
        yield_hist.push_back(candidates_found);
        if yield_hist.len() > self.config.saturation_window {
            yield_hist.pop_front();
        }

        // Update fact count history.
        let fact_hist = self
            .fact_count_history
            .entry(domain.to_string())
            .or_default();
        fact_hist.push_back(current_facts);
        if fact_hist.len() > self.config.growth_window {
            fact_hist.pop_front();
        }

        // Update exploration count.
        *self
            .exploration_count
            .entry(domain.to_string())
            .or_default() += 1;

        // Check saturation.
        if self.check_saturation(domain) {
            self.saturated.insert(domain.to_string());
        }
    }

    /// Check if a domain is saturated based on yield history.
    /// Saturated if last N yields are all below threshold.
    fn check_saturation(&self, domain: &str) -> bool {
        let history = match self.yield_history.get(domain) {
            Some(h) => h,
            None => return false,
        };
        if history.len() < self.config.saturation_window {
            return false;
        }
        history
            .iter()
            .rev()
            .take(self.config.saturation_window)
            .all(|&yield_count| yield_count < self.config.saturation_threshold)
    }

    /// Whether a domain is currently saturated.
    pub fn is_saturated(&self, domain: &str) -> bool {
        self.saturated.contains(domain)
    }

    /// Get all saturated domains.
    pub fn saturated_domains(&self) -> Vec<String> {
        self.saturated.iter().cloned().collect()
    }

    /// Mark a domain as manually saturated.
    pub fn mark_saturated(&mut self, domain: &str) {
        self.saturated.insert(domain.to_string());
    }

    /// Clear saturation for a domain (give it another chance).
    pub fn clear_saturation(&mut self, domain: &str) {
        self.saturated.remove(domain);
        self.yield_history.remove(domain);
    }

    /// Whether all known domains are saturated.
    pub fn all_saturated(&self) -> bool {
        !self.saturated.is_empty()
            && self
                .yield_history
                .keys()
                .all(|d| self.saturated.contains(d))
    }

    /// Get exploration count for a domain.
    pub fn exploration_count(&self, domain: &str) -> usize {
        self.exploration_count.get(domain).copied().unwrap_or(0)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_entropy() {
        let stats = DomainStats {
            domain: "test".into(),
            fact_count: 100,
            edge_count: 10,
            contradiction_count: 5,
            avg_structuring_score: 0.0,
        };
        // entropy = log2(101) * 6 / 11 ≈ 6.66 * 6 / 11 ≈ 3.63
        let entropy = EntropyGradientSearcher::compute_entropy(&stats);
        assert!(entropy > 3.0 && entropy < 4.0);
    }

    #[test]
    fn test_compute_priority() {
        // High entropy + low structuring = high priority.
        let p1 = EntropyGradientSearcher::compute_priority(5.0, 0.0);
        // High entropy + high structuring = lower priority.
        let p2 = EntropyGradientSearcher::compute_priority(5.0, 5.0);
        assert!(p1 > p2);
    }

    #[test]
    fn test_saturation_detection() {
        let mut searcher = EntropyGradientSearcher::new("http://localhost:1738");
        // Record 3 low-yield explorations.
        searcher.record_exploration("test-domain", 1, 100);
        assert!(!searcher.is_saturated("test-domain"));
        searcher.record_exploration("test-domain", 0, 102);
        assert!(!searcher.is_saturated("test-domain"));
        searcher.record_exploration("test-domain", 1, 103);
        // 3 consecutive low yields → saturated.
        assert!(searcher.is_saturated("test-domain"));
    }

    #[test]
    fn test_saturation_not_triggered_with_good_yield() {
        let mut searcher = EntropyGradientSearcher::new("http://localhost:1738");
        searcher.record_exploration("good-domain", 1, 100);
        searcher.record_exploration("good-domain", 5, 105);
        searcher.record_exploration("good-domain", 1, 106);
        // Middle exploration had good yield → not saturated.
        assert!(!searcher.is_saturated("good-domain"));
    }

    #[test]
    fn test_clear_saturation() {
        let mut searcher = EntropyGradientSearcher::new("http://localhost:1738");
        searcher.mark_saturated("manual");
        assert!(searcher.is_saturated("manual"));
        searcher.clear_saturation("manual");
        assert!(!searcher.is_saturated("manual"));
    }

    #[test]
    fn test_all_saturated() {
        let mut searcher = EntropyGradientSearcher::new("http://localhost:1738");
        // No domains → not all saturated.
        assert!(!searcher.all_saturated());
        // Add domains and saturate them.
        searcher.record_exploration("d1", 0, 10);
        searcher.record_exploration("d1", 0, 10);
        searcher.record_exploration("d1", 0, 10);
        searcher.record_exploration("d2", 0, 5);
        searcher.record_exploration("d2", 0, 5);
        searcher.record_exploration("d2", 0, 5);
        assert!(searcher.all_saturated());
    }
}
