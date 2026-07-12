//! Semantic-memory retrieval quality benchmark adapter.
//!
//! Runs `SMQueryFixture` files against the semantic-memory HTTP server at
//! `http://127.0.0.1:1738/search` and computes standard IR metrics:
//! Recall@k, nDCG@k, MRR, p95/p99 latency.
//!
//! Enable with `--features sm-adapter` or in Cargo.toml:
//! ```toml
//! receipt-bench = { features = ["sm-adapter"] }
//! ```

use std::time::Instant;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::MachineFingerprint;

// ─── Query class ─────────────────────────────────────────────────────────────

/// Query class describes the retrieval difficulty and intent of a fixture.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum QueryClass {
    /// Direct factual lookup — single entity, high specificity.
    A,
    /// Semantic / conceptual — multi-term, moderate specificity.
    B,
    /// Multi-hop / complex — cross-domain, may require graph expansion.
    C,
    /// Stale / supersession check — verifies superseded facts are filtered.
    D,
    /// Contradiction — tests that conflicting facts surface correctly.
    E,
    /// Namespace-scoped — verifies namespace filtering is honoured.
    F,
}

// ─── Fixture ─────────────────────────────────────────────────────────────────

/// A single query fixture with ground-truth relevant IDs.
///
/// Serialized as one JSON object per line in a `.jsonl` fixture file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SMQueryFixture {
    /// The query string sent to `/search`.
    pub query: String,
    /// Ground-truth item IDs that SHOULD appear in the top-k results.
    /// Format: `"fact:<uuid>"` or `"chunk:<uuid>"`.
    pub relevant_ids: Vec<String>,
    /// Optional namespace filter applied to the search request.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub namespace: Option<String>,
    /// Query class (A–F) for reporting and filtering.
    pub query_class: QueryClass,
    /// Optional top_k override (default: 10 from `SMBenchmarkConfig`).
    #[serde(default)]
    pub top_k: Option<usize>,
    /// Optional human-readable note about this fixture.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

/// Load fixtures from a JSONL file (one `SMQueryFixture` per line).
/// Lines starting with `//` and blank lines are skipped.
pub fn load_fixtures_from_jsonl(path: &std::path::Path) -> Result<Vec<SMQueryFixture>, String> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("failed to read {}: {}", path.display(), e))?;
    content
        .lines()
        .filter(|l| !l.trim().is_empty() && !l.trim_start().starts_with("//"))
        .map(|line| {
            serde_json::from_str::<SMQueryFixture>(line).map_err(|e| {
                format!(
                    "fixture parse error ({}): {}",
                    e,
                    &line[..line.len().min(80)]
                )
            })
        })
        .collect()
}

/// Load all `.jsonl` fixture files from a directory, sorted by filename.
pub fn load_fixtures_from_dir(dir: &std::path::Path) -> Result<Vec<SMQueryFixture>, String> {
    let mut paths: Vec<_> = std::fs::read_dir(dir)
        .map_err(|e| format!("failed to read dir {}: {}", dir.display(), e))?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|ext| ext == "jsonl"))
        .collect();
    paths.sort();

    let mut all = Vec::new();
    for path in &paths {
        all.extend(load_fixtures_from_jsonl(path)?);
    }
    Ok(all)
}

// ─── Per-query run result ─────────────────────────────────────────────────────

/// Result of running a single query fixture against the live server.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SMRunResult {
    /// The fixture that was evaluated.
    pub fixture: SMQueryFixture,
    /// Ordered list of returned item IDs (as returned by the server).
    pub returned_ids: Vec<String>,
    /// Recall at k=5.
    pub recall_at_5: f64,
    /// Recall at k=10.
    pub recall_at_10: f64,
    /// nDCG at k=5.
    pub ndcg_at_5: f64,
    /// nDCG at k=10.
    pub ndcg_at_10: f64,
    /// Mean Reciprocal Rank (1/rank of first relevant result; 0 if no hit).
    pub mrr: f64,
    /// Query latency in milliseconds (wall-clock, including HTTP round-trip).
    pub latency_ms: f64,
    /// True if the HTTP request or response parsing failed.
    pub errored: bool,
    /// Error message when `errored` is true.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

// ─── Summary stats ────────────────────────────────────────────────────────────

/// Aggregate summary statistics over all run results in a report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SMSummaryStats {
    pub num_queries: usize,
    pub num_errors: usize,
    pub mean_recall_at_5: f64,
    pub mean_recall_at_10: f64,
    pub mean_ndcg_at_5: f64,
    pub mean_ndcg_at_10: f64,
    pub mean_mrr: f64,
    pub p95_latency_ms: f64,
    pub p99_latency_ms: f64,
    pub mean_latency_ms: f64,
    pub min_latency_ms: f64,
    pub max_latency_ms: f64,
}

// ─── Full benchmark report ────────────────────────────────────────────────────

/// Full benchmark report for one SM retrieval quality run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SMBenchmarkReport {
    /// Human label for this run (e.g. "before", "after", "main").
    pub suite_name: String,
    /// Per-query results.
    pub results: Vec<SMRunResult>,
    /// Aggregate summary statistics.
    pub summary: SMSummaryStats,
    /// Git commit hash of the code being benchmarked.
    pub commit_hash: String,
    /// Timestamp of the run (UTC).
    pub timestamp: DateTime<Utc>,
    /// Machine fingerprint for environment pinning.
    pub machine_fingerprint: MachineFingerprint,
    /// Total elapsed wall-clock time for the run in milliseconds.
    pub elapsed_ms: u64,
}

impl SMBenchmarkReport {
    /// Emit JSONL receipts: one line per query result, then one summary line.
    ///
    /// The output is deterministic given the same results (timestamp is
    /// excluded from the per-result lines), making it suitable for
    /// content-addressable storage and before/after diffing.
    pub fn to_jsonl(&self) -> String {
        let mut lines: Vec<String> = self
            .results
            .iter()
            .map(|r| serde_json::to_string(r).unwrap_or_default())
            .collect();
        let summary_rec = serde_json::json!({
            "record_type": "summary",
            "suite_name":  self.suite_name,
            "commit_hash": self.commit_hash,
            "timestamp":   self.timestamp,
            "summary":     self.summary,
            "elapsed_ms":  self.elapsed_ms,
        });
        lines.push(serde_json::to_string(&summary_rec).unwrap_or_default());
        lines.join("\n")
    }

    /// Compute a deterministic 32-hex-char content hash of this report.
    ///
    /// The hash covers suite_name, commit_hash, returned IDs, and rounded
    /// metric values. Timestamp and machine fingerprint are excluded so that
    /// logically identical runs on different machines produce the same hash.
    pub fn report_hash(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(self.suite_name.as_bytes());
        hasher.update(self.commit_hash.as_bytes());
        for r in &self.results {
            hasher.update(r.fixture.query.as_bytes());
            for id in &r.returned_ids {
                hasher.update(id.as_bytes());
            }
            hasher.update(format!("{:.4}{:.4}{:.4}", r.recall_at_5, r.ndcg_at_5, r.mrr).as_bytes());
        }
        let bytes = hasher.finalize();
        encode_hex_16(&bytes[..16])
    }

    /// Print a human-readable summary table to stdout.
    pub fn print_summary(&self) {
        println!("=== SM Retrieval Benchmark: {} ===", self.suite_name);
        println!("Commit:    {}", self.commit_hash);
        println!("Timestamp: {}", self.timestamp.to_rfc3339());
        println!(
            "Queries:   {}  Errors: {}  Elapsed: {} ms",
            self.summary.num_queries, self.summary.num_errors, self.elapsed_ms
        );
        println!();
        println!("--- Quality Metrics ---");
        println!("MRR:         {:.4}", self.summary.mean_mrr);
        println!("Recall@5:    {:.4}", self.summary.mean_recall_at_5);
        println!("Recall@10:   {:.4}", self.summary.mean_recall_at_10);
        println!("nDCG@5:      {:.4}", self.summary.mean_ndcg_at_5);
        println!("nDCG@10:     {:.4}", self.summary.mean_ndcg_at_10);
        println!();
        println!("--- Latency ---");
        println!("Mean:  {:.2} ms", self.summary.mean_latency_ms);
        println!("p95:   {:.2} ms", self.summary.p95_latency_ms);
        println!("p99:   {:.2} ms", self.summary.p99_latency_ms);
        println!("Min:   {:.2} ms", self.summary.min_latency_ms);
        println!("Max:   {:.2} ms", self.summary.max_latency_ms);
        println!();
        println!(
            "{:<60} {:>6} {:>6} {:>6} {:>8}",
            "Query", "R@5", "R@10", "MRR", "Lat(ms)"
        );
        println!("{}", "-".repeat(86));
        for r in &self.results {
            let q = truncate_str(&r.fixture.query, 60);
            let err = if r.errored { " ERR" } else { "" };
            println!(
                "{:<60} {:>6.3} {:>6.3} {:>6.3} {:>8.1}{}",
                q, r.recall_at_5, r.recall_at_10, r.mrr, r.latency_ms, err
            );
        }
    }
}

// ─── Comparison report ────────────────────────────────────────────────────────

/// Before/after delta between two SM benchmark reports.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparisonReport {
    pub before_suite: String,
    pub after_suite: String,
    pub before_hash: String,
    pub after_hash: String,
    /// Delta in mean Recall@5 (after − before; positive = improved).
    pub delta_recall_at_5: f64,
    /// Delta in mean Recall@10.
    pub delta_recall_at_10: f64,
    /// Delta in mean nDCG@5.
    pub delta_ndcg_at_5: f64,
    /// Delta in mean nDCG@10.
    pub delta_ndcg_at_10: f64,
    /// Delta in mean MRR.
    pub delta_mrr: f64,
    /// Delta in p95 latency (negative = faster = improved).
    pub delta_p95_latency_ms: f64,
    /// Delta in p99 latency.
    pub delta_p99_latency_ms: f64,
    /// Delta in mean latency.
    pub delta_mean_latency_ms: f64,
    /// True iff all quality deltas ≥ 0 and p95 latency delta ≤ 0.
    pub improved: bool,
}

impl ComparisonReport {
    /// Print a human-readable comparison summary to stdout.
    pub fn print_summary(&self) {
        println!(
            "=== SM Benchmark Comparison: {} → {} ===",
            self.before_suite, self.after_suite
        );
        println!();
        println!("--- Quality Deltas (positive = improved) ---");
        println!("MRR:       {:+.4}", self.delta_mrr);
        println!("Recall@5:  {:+.4}", self.delta_recall_at_5);
        println!("Recall@10: {:+.4}", self.delta_recall_at_10);
        println!("nDCG@5:    {:+.4}", self.delta_ndcg_at_5);
        println!("nDCG@10:   {:+.4}", self.delta_ndcg_at_10);
        println!();
        println!("--- Latency Deltas (negative = faster = improved) ---");
        println!("Mean:  {:+.2} ms", self.delta_mean_latency_ms);
        println!("p95:   {:+.2} ms", self.delta_p95_latency_ms);
        println!("p99:   {:+.2} ms", self.delta_p99_latency_ms);
        println!();
        let verdict = if self.improved {
            "IMPROVED"
        } else {
            "MIXED / REGRESSED"
        };
        println!("Verdict: {verdict}");
    }
}

/// Compare two benchmark reports and compute metric deltas.
pub fn compare_reports(before: &SMBenchmarkReport, after: &SMBenchmarkReport) -> ComparisonReport {
    let b = &before.summary;
    let a = &after.summary;

    let delta_recall_at_5 = a.mean_recall_at_5 - b.mean_recall_at_5;
    let delta_recall_at_10 = a.mean_recall_at_10 - b.mean_recall_at_10;
    let delta_ndcg_at_5 = a.mean_ndcg_at_5 - b.mean_ndcg_at_5;
    let delta_ndcg_at_10 = a.mean_ndcg_at_10 - b.mean_ndcg_at_10;
    let delta_mrr = a.mean_mrr - b.mean_mrr;
    let delta_p95_latency_ms = a.p95_latency_ms - b.p95_latency_ms;
    let delta_p99_latency_ms = a.p99_latency_ms - b.p99_latency_ms;
    let delta_mean_latency_ms = a.mean_latency_ms - b.mean_latency_ms;

    let improved = delta_recall_at_5 >= 0.0
        && delta_recall_at_10 >= 0.0
        && delta_ndcg_at_5 >= 0.0
        && delta_ndcg_at_10 >= 0.0
        && delta_mrr >= 0.0
        && delta_p95_latency_ms <= 0.0;

    ComparisonReport {
        before_suite: before.suite_name.clone(),
        after_suite: after.suite_name.clone(),
        before_hash: before.report_hash(),
        after_hash: after.report_hash(),
        delta_recall_at_5,
        delta_recall_at_10,
        delta_ndcg_at_5,
        delta_ndcg_at_10,
        delta_mrr,
        delta_p95_latency_ms,
        delta_p99_latency_ms,
        delta_mean_latency_ms,
        improved,
    }
}

// ─── IR metric helpers ────────────────────────────────────────────────────────

pub fn recall_at_k(returned: &[String], relevant: &[String], k: usize) -> f64 {
    if relevant.is_empty() {
        return 0.0;
    }
    let relevant_set: std::collections::HashSet<&str> =
        relevant.iter().map(|s| s.as_str()).collect();
    let hits = returned
        .iter()
        .take(k)
        .filter(|id| relevant_set.contains(id.as_str()))
        .count();
    hits as f64 / relevant.len() as f64
}

pub fn ndcg_at_k(returned: &[String], relevant: &[String], k: usize) -> f64 {
    let relevant_set: std::collections::HashSet<&str> =
        relevant.iter().map(|s| s.as_str()).collect();

    let dcg: f64 = returned
        .iter()
        .take(k)
        .enumerate()
        .map(|(i, id)| {
            if relevant_set.contains(id.as_str()) {
                1.0 / (i as f64 + 2.0).log2()
            } else {
                0.0
            }
        })
        .sum();

    let idcg: f64 = (0..relevant.len().min(k))
        .map(|i| 1.0 / (i as f64 + 2.0).log2())
        .sum();

    if idcg == 0.0 {
        0.0
    } else {
        dcg / idcg
    }
}

pub fn reciprocal_rank(returned: &[String], relevant: &[String]) -> f64 {
    let relevant_set: std::collections::HashSet<&str> =
        relevant.iter().map(|s| s.as_str()).collect();
    for (i, id) in returned.iter().enumerate() {
        if relevant_set.contains(id.as_str()) {
            return 1.0 / (i as f64 + 1.0);
        }
    }
    0.0
}

fn percentile(sorted: &[f64], p: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    let idx = ((p / 100.0) * (sorted.len() as f64 - 1.0)).floor() as usize;
    sorted[idx.min(sorted.len() - 1)]
}

// ─── HTTP adapter types ───────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct HttpSearchResponse {
    #[serde(default)]
    results: Vec<HttpSearchResult>,
}

#[derive(Debug, Deserialize)]
struct HttpSearchResult {
    result_id: String,
}

// ─── Benchmark runner config ─────────────────────────────────────────────────

/// Configuration for a single SM benchmark run.
pub struct SMBenchmarkConfig {
    /// HTTP server base URL.
    pub server_url: String,
    /// Suite label written into the report (e.g. "before", "after", "main").
    pub suite_name: String,
    /// Git commit hash of the code being benchmarked.
    pub commit_hash: String,
    /// Default top_k if not set per-fixture.
    pub default_top_k: usize,
    /// Warmup queries to run before measurement (results discarded).
    pub warmup_queries: usize,
    /// HTTP request timeout in seconds.
    pub timeout_secs: u64,
}

impl Default for SMBenchmarkConfig {
    fn default() -> Self {
        Self {
            server_url: "http://127.0.0.1:1738".to_string(),
            suite_name: "default".to_string(),
            commit_hash: detect_git_hash(),
            default_top_k: 10,
            warmup_queries: 1,
            timeout_secs: 30,
        }
    }
}

// ─── Main runner ─────────────────────────────────────────────────────────────

/// Run a retrieval quality benchmark against the live semantic-memory HTTP server.
///
/// The server must be reachable at `config.server_url` before calling this.
/// Warmup queries are fired first and their results discarded.
/// Returns an `SMBenchmarkReport` with per-query and aggregate metrics.
pub fn run_sm_benchmark(
    fixtures: Vec<SMQueryFixture>,
    config: SMBenchmarkConfig,
) -> Result<SMBenchmarkReport, String> {
    run_sm_benchmark_with_auth(fixtures, config, None)
}

/// Run a retrieval benchmark with an optional bearer token.
///
/// This separate entry point preserves source compatibility for callers that
/// construct [`SMBenchmarkConfig`] with struct literals.
pub fn run_sm_benchmark_with_auth(
    fixtures: Vec<SMQueryFixture>,
    config: SMBenchmarkConfig,
    http_auth_token: Option<&str>,
) -> Result<SMBenchmarkReport, String> {
    let http_auth_token = match http_auth_token {
        Some(raw) => {
            let token = raw.trim();
            if token.is_empty() || token.chars().any(char::is_whitespace) {
                return Err(
                    "HTTP auth token must be non-empty and contain no whitespace".to_string(),
                );
            }
            Some(token)
        }
        None => None,
    };
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(config.timeout_secs))
        .build()
        .map_err(|e| format!("failed to build HTTP client: {}", e))?;

    let start = Instant::now();
    let mut results: Vec<SMRunResult> = Vec::with_capacity(fixtures.len());
    let mut latencies: Vec<f64> = Vec::new();

    for fixture in fixtures.iter().take(config.warmup_queries) {
        let _ = query_server(
            &client,
            &config.server_url,
            fixture,
            &config,
            http_auth_token,
        );
    }

    for fixture in &fixtures {
        let result = query_server(
            &client,
            &config.server_url,
            fixture,
            &config,
            http_auth_token,
        );
        if !result.errored {
            latencies.push(result.latency_ms);
        }
        results.push(result);
    }

    let summary = compute_summary(&results, &latencies);
    let elapsed_ms = start.elapsed().as_millis() as u64;

    Ok(SMBenchmarkReport {
        suite_name: config.suite_name,
        results,
        summary,
        commit_hash: config.commit_hash,
        timestamp: Utc::now(),
        machine_fingerprint: MachineFingerprint::generate(),
        elapsed_ms,
    })
}

fn query_server(
    client: &reqwest::blocking::Client,
    server_url: &str,
    fixture: &SMQueryFixture,
    config: &SMBenchmarkConfig,
    http_auth_token: Option<&str>,
) -> SMRunResult {
    let top_k = fixture.top_k.unwrap_or(config.default_top_k);
    let mut payload = serde_json::json!({
        "query": fixture.query,
        "top_k":  top_k,
    });
    if let Some(ref ns) = fixture.namespace {
        payload["namespaces"] = serde_json::json!([ns]);
    }

    let start = Instant::now();
    let mut request = client.post(format!("{}/search", server_url)).json(&payload);
    if let Some(token) = http_auth_token {
        request = request.bearer_auth(token);
    }
    let response = request.send();
    let latency_ms = start.elapsed().as_secs_f64() * 1000.0;

    let err_result = |msg: String| SMRunResult {
        fixture: fixture.clone(),
        returned_ids: Vec::new(),
        recall_at_5: 0.0,
        recall_at_10: 0.0,
        ndcg_at_5: 0.0,
        ndcg_at_10: 0.0,
        mrr: 0.0,
        latency_ms,
        errored: true,
        error: Some(msg),
    };

    let resp = match response {
        Err(e) => return err_result(format!("request failed: {}", e)),
        Ok(r) => r,
    };
    let status = resp.status();
    if !status.is_success() {
        return err_result(format!("server returned HTTP {status}"));
    }

    let search_resp = match resp.json::<HttpSearchResponse>() {
        Err(e) => return err_result(format!("failed to parse response: {}", e)),
        Ok(r) => r,
    };

    let returned_ids: Vec<String> = search_resp
        .results
        .into_iter()
        .map(|r| r.result_id)
        .collect();
    let rel = &fixture.relevant_ids;

    SMRunResult {
        recall_at_5: recall_at_k(&returned_ids, rel, 5),
        recall_at_10: recall_at_k(&returned_ids, rel, 10),
        ndcg_at_5: ndcg_at_k(&returned_ids, rel, 5),
        ndcg_at_10: ndcg_at_k(&returned_ids, rel, 10),
        mrr: reciprocal_rank(&returned_ids, rel),
        fixture: fixture.clone(),
        returned_ids,
        latency_ms,
        errored: false,
        error: None,
    }
}

fn compute_summary(results: &[SMRunResult], latencies: &[f64]) -> SMSummaryStats {
    let ok: Vec<&SMRunResult> = results.iter().filter(|r| !r.errored).collect();
    let n = ok.len() as f64;

    let mean_of = |vals: Vec<f64>| -> f64 {
        if vals.is_empty() {
            0.0
        } else {
            vals.iter().sum::<f64>() / vals.len() as f64
        }
    };

    let mut sorted_lat = latencies.to_vec();
    sorted_lat.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let mean_latency = if latencies.is_empty() {
        0.0
    } else {
        latencies.iter().sum::<f64>() / latencies.len() as f64
    };

    let (min_lat, max_lat) = if latencies.is_empty() {
        (0.0, 0.0)
    } else {
        let min = latencies.iter().cloned().fold(f64::INFINITY, f64::min);
        let max = latencies.iter().cloned().fold(0.0_f64, f64::max);
        (min, max)
    };

    SMSummaryStats {
        num_queries: results.len(),
        num_errors: results.iter().filter(|r| r.errored).count(),
        mean_recall_at_5: mean_of(ok.iter().map(|r| r.recall_at_5).collect()),
        mean_recall_at_10: mean_of(ok.iter().map(|r| r.recall_at_10).collect()),
        mean_ndcg_at_5: mean_of(ok.iter().map(|r| r.ndcg_at_5).collect()),
        mean_ndcg_at_10: mean_of(ok.iter().map(|r| r.ndcg_at_10).collect()),
        mean_mrr: if n == 0.0 {
            0.0
        } else {
            ok.iter().map(|r| r.mrr).sum::<f64>() / n
        },
        p95_latency_ms: percentile(&sorted_lat, 95.0),
        p99_latency_ms: percentile(&sorted_lat, 99.0),
        mean_latency_ms: mean_latency,
        min_latency_ms: min_lat,
        max_latency_ms: max_lat,
    }
}

// ─── Internal helpers ─────────────────────────────────────────────────────────

fn detect_git_hash() -> String {
    std::process::Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

fn encode_hex_16(data: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut s = String::with_capacity(data.len() * 2);
    for &b in data {
        s.push(HEX[(b >> 4) as usize] as char);
        s.push(HEX[(b & 0xf) as usize] as char);
    }
    s
}

fn truncate_str(s: &str, max_chars: usize) -> String {
    if s.chars().count() <= max_chars {
        s.to_string()
    } else {
        let truncated: String = s.chars().take(max_chars - 1).collect();
        format!("{}…", truncated)
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture(query: &str, relevant: &[&str]) -> SMQueryFixture {
        SMQueryFixture {
            query: query.to_string(),
            relevant_ids: relevant.iter().map(|s| s.to_string()).collect(),
            namespace: None,
            query_class: QueryClass::A,
            top_k: Some(10),
            note: None,
        }
    }

    fn run_result_from(fixture: SMQueryFixture, returned: &[&str]) -> SMRunResult {
        let returned_ids: Vec<String> = returned.iter().map(|s| s.to_string()).collect();
        let rel = &fixture.relevant_ids;
        SMRunResult {
            recall_at_5: recall_at_k(&returned_ids, rel, 5),
            recall_at_10: recall_at_k(&returned_ids, rel, 10),
            ndcg_at_5: ndcg_at_k(&returned_ids, rel, 5),
            ndcg_at_10: ndcg_at_k(&returned_ids, rel, 10),
            mrr: reciprocal_rank(&returned_ids, rel),
            fixture,
            returned_ids,
            latency_ms: 10.0,
            errored: false,
            error: None,
        }
    }

    fn make_report(suite: &str, commit: &str, results: Vec<SMRunResult>) -> SMBenchmarkReport {
        let latencies: Vec<f64> = results
            .iter()
            .filter(|r| !r.errored)
            .map(|r| r.latency_ms)
            .collect();
        let summary = compute_summary(&results, &latencies);
        SMBenchmarkReport {
            suite_name: suite.to_string(),
            results,
            summary,
            commit_hash: commit.to_string(),
            timestamp: Utc::now(),
            machine_fingerprint: MachineFingerprint::from_hex(&"0".repeat(64)),
            elapsed_ms: 100,
        }
    }

    // ─── HTTP adapter tests ────────────────────────────────────────────────

    #[test]
    fn authenticated_runner_rejects_invalid_token_before_network_io() {
        let error = run_sm_benchmark_with_auth(
            vec![fixture("auth query", &["fact:a"])],
            SMBenchmarkConfig::default(),
            Some("first second"),
        )
        .expect_err("invalid token must fail");
        assert!(error.contains("contain no whitespace"));
    }

    #[test]
    fn authenticated_http_errors_are_reported_not_parsed_as_empty_success() {
        use std::io::{Read, Write};
        use std::net::TcpListener;

        let listener = TcpListener::bind("127.0.0.1:0").expect("bind mock server");
        let address = listener.local_addr().expect("mock address");
        let server = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("accept request");
            let mut request = [0_u8; 4096];
            let bytes = stream.read(&mut request).expect("read request");
            let request = String::from_utf8_lossy(&request[..bytes]);
            assert!(request.contains("authorization: Bearer test-token\r\n"));
            let body = r#"{"error":"unauthorized"}"#;
            write!(
                stream,
                "HTTP/1.1 401 Unauthorized\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(),
                body
            )
            .expect("write response");
        });

        let config = SMBenchmarkConfig {
            server_url: format!("http://{address}"),
            warmup_queries: 0,
            ..SMBenchmarkConfig::default()
        };
        let report = run_sm_benchmark_with_auth(
            vec![fixture("auth query", &["fact:a"])],
            config,
            Some("test-token"),
        )
        .expect("benchmark report");
        server.join().expect("mock server thread");

        assert_eq!(report.summary.num_errors, 1);
        assert!(report.results[0].errored);
        assert!(report.results[0]
            .error
            .as_deref()
            .is_some_and(|error| error.contains("HTTP 401 Unauthorized")));
    }

    // ─── IR metric unit tests ─────────────────────────────────────────────

    #[test]
    fn recall_at_k_perfect() {
        let returned = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        let relevant = vec!["a".to_string(), "b".to_string()];
        assert!((recall_at_k(&returned, &relevant, 5) - 1.0).abs() < 1e-9);
    }

    #[test]
    fn recall_at_k_partial() {
        // Only "a" in top-2, 1/3 relevant found = 0.333…
        let returned = vec!["a".to_string(), "x".to_string(), "b".to_string()];
        let relevant = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        assert!((recall_at_k(&returned, &relevant, 2) - 1.0 / 3.0).abs() < 1e-9);
    }

    #[test]
    fn recall_at_k_empty_relevant() {
        let returned = vec!["a".to_string()];
        assert!((recall_at_k(&returned, &[], 5) - 0.0).abs() < 1e-9);
    }

    #[test]
    fn ndcg_at_k_perfect() {
        let returned = vec!["a".to_string(), "b".to_string()];
        let relevant = vec!["a".to_string(), "b".to_string()];
        assert!((ndcg_at_k(&returned, &relevant, 2) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn ndcg_at_k_no_hits() {
        let returned = vec!["x".to_string(), "y".to_string()];
        let relevant = vec!["a".to_string(), "b".to_string()];
        assert!((ndcg_at_k(&returned, &relevant, 5) - 0.0).abs() < 1e-9);
    }

    #[test]
    fn ndcg_at_k_partial_hit_lower_than_perfect() {
        // With two relevant items, returning only one in top-2 gives lower nDCG.
        // Note: with binary relevance, ["a","b"] and ["b","a"] are equal (same
        // grade on both items), so this test uses a noise item to distinguish.
        let relevant = vec!["a".to_string(), "b".to_string()];
        let returned_perfect = vec!["a".to_string(), "b".to_string()];
        let returned_partial = vec!["x".to_string(), "a".to_string()];
        let ndcg_perfect = ndcg_at_k(&returned_perfect, &relevant, 2);
        let ndcg_partial = ndcg_at_k(&returned_partial, &relevant, 2);
        assert!(
            (ndcg_perfect - 1.0).abs() < 1e-6,
            "perfect nDCG should be 1.0"
        );
        assert!(
            ndcg_partial < 1.0,
            "partial hit nDCG should be < 1.0, got {}",
            ndcg_partial
        );
        assert!(
            ndcg_partial > 0.0,
            "partial hit nDCG should be > 0.0, got {}",
            ndcg_partial
        );
    }

    #[test]
    fn mrr_at_rank_1() {
        let returned = vec!["a".to_string(), "b".to_string()];
        let relevant = vec!["a".to_string()];
        assert!((reciprocal_rank(&returned, &relevant) - 1.0).abs() < 1e-9);
    }

    #[test]
    fn mrr_at_rank_2() {
        let returned = vec!["x".to_string(), "a".to_string()];
        let relevant = vec!["a".to_string()];
        assert!((reciprocal_rank(&returned, &relevant) - 0.5).abs() < 1e-9);
    }

    #[test]
    fn mrr_no_hit() {
        let returned = vec!["x".to_string()];
        let relevant = vec!["a".to_string()];
        assert!((reciprocal_rank(&returned, &relevant) - 0.0).abs() < 1e-9);
    }

    // ─── Aggregation tests ────────────────────────────────────────────────

    #[test]
    fn summary_computes_correctly() {
        let f = fixture("q", &["a", "b"]);
        let r = run_result_from(f, &["a", "b", "c"]);
        let latencies = vec![r.latency_ms];
        let summary = compute_summary(&[r], &latencies);
        assert_eq!(summary.num_queries, 1);
        assert_eq!(summary.num_errors, 0);
        assert!((summary.mean_recall_at_5 - 1.0).abs() < 1e-9);
        assert!((summary.mean_mrr - 1.0).abs() < 1e-9);
        assert!(summary.mean_latency_ms > 0.0);
    }

    #[test]
    fn summary_empty_latencies_does_not_panic() {
        let f = fixture("q", &["a"]);
        let mut r = run_result_from(f, &[]);
        r.errored = true;
        let summary = compute_summary(&[r], &[]);
        assert_eq!(summary.num_errors, 1);
        assert_eq!(summary.min_latency_ms, 0.0);
        assert_eq!(summary.max_latency_ms, 0.0);
    }

    // ─── Report tests ─────────────────────────────────────────────────────

    #[test]
    fn report_to_jsonl_has_correct_line_count() {
        let f = fixture("q", &["a", "b"]);
        let r = run_result_from(f, &["a", "b", "c"]);
        let report = make_report("test", "abc", vec![r]);
        let jsonl = report.to_jsonl();
        let lines: Vec<&str> = jsonl.lines().filter(|l| !l.is_empty()).collect();
        assert_eq!(lines.len(), 2, "1 result line + 1 summary line");
    }

    #[test]
    fn report_to_jsonl_lines_are_valid_json() {
        let f = fixture("q", &["a"]);
        let r = run_result_from(f, &["a"]);
        let report = make_report("test", "abc", vec![r]);
        for line in report.to_jsonl().lines() {
            if line.is_empty() {
                continue;
            }
            let v: serde_json::Value = serde_json::from_str(line)
                .unwrap_or_else(|e| panic!("invalid JSON line: {} — {}", e, line));
            assert!(v.is_object());
        }
    }

    #[test]
    fn report_hash_is_deterministic() {
        let f = fixture("q", &["a"]);
        let r = run_result_from(f, &["a"]);
        let report = make_report("s", "abc", vec![r]);
        assert_eq!(report.report_hash(), report.report_hash());
    }

    #[test]
    fn report_hash_changes_with_different_results() {
        let f1 = fixture("q", &["a", "b"]);
        let r1 = run_result_from(f1, &["a", "x"]);

        let f2 = fixture("q", &["a", "b"]);
        let r2 = run_result_from(f2, &["a", "b"]);

        let report1 = make_report("s", "abc", vec![r1]);
        let report2 = make_report("s", "abc", vec![r2]);

        assert_ne!(report1.report_hash(), report2.report_hash());
    }

    // ─── Comparison tests ─────────────────────────────────────────────────

    #[test]
    fn compare_reports_detects_improvement() {
        let f_before = fixture("q", &["a", "b"]);
        let r_before = run_result_from(f_before, &["x", "y"]);

        let f_after = fixture("q", &["a", "b"]);
        let r_after = run_result_from(f_after, &["a", "b"]);

        let before = make_report("before", "aaa", vec![r_before]);
        let after = make_report("after", "bbb", vec![r_after]);
        let comp = compare_reports(&before, &after);

        assert!(comp.delta_recall_at_5 > 0.0);
        assert!(comp.delta_mrr > 0.0);
        assert!(comp.improved);
    }

    #[test]
    fn compare_reports_identical_is_not_regressed() {
        let f = fixture("q", &["a"]);
        let r1 = run_result_from(f.clone(), &["a"]);
        let r2 = run_result_from(f, &["a"]);
        let before = make_report("b", "aaa", vec![r1]);
        let after = make_report("a", "aaa", vec![r2]);
        let comp = compare_reports(&before, &after);
        assert!((comp.delta_mrr).abs() < 1e-9);
        assert!(
            comp.improved,
            "identical runs should be considered 'improved'"
        );
    }

    // ─── JSONL loader test ────────────────────────────────────────────────

    #[test]
    fn load_fixtures_from_jsonl_roundtrip() {
        let fixtures = [
            SMQueryFixture {
                query: "test query one".to_string(),
                relevant_ids: vec!["fact:aaa".to_string(), "chunk:bbb".to_string()],
                namespace: Some("research".to_string()),
                query_class: QueryClass::B,
                top_k: Some(5),
                note: Some("example".to_string()),
            },
            SMQueryFixture {
                query: "test query two".to_string(),
                relevant_ids: vec!["fact:ccc".to_string()],
                namespace: None,
                query_class: QueryClass::A,
                top_k: None,
                note: None,
            },
        ];

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.jsonl");
        let jsonl: String = fixtures
            .iter()
            .map(|f| serde_json::to_string(f).unwrap())
            .collect::<Vec<_>>()
            .join("\n");
        std::fs::write(&path, jsonl).unwrap();

        let loaded = load_fixtures_from_jsonl(&path).unwrap();
        assert_eq!(loaded.len(), 2);
        assert_eq!(loaded[0].query, "test query one");
        assert_eq!(loaded[0].namespace, Some("research".to_string()));
        assert_eq!(loaded[1].query, "test query two");
        assert!(loaded[1].namespace.is_none());
    }
}
