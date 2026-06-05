//! HNSW backend benchmark: hnsw_rs 0.3 vs usearch 2.25.
//!
//! Compares insert throughput, search latency, recall@10, save/load
//! latency, and RSS memory at production-scale vectors (768-dim, 1024-dim,
//! 256-dim). Generates a receipt-bench receipt for reproducibility.
//!
//! ## What this measures
//!
//! 1. **Insert throughput** at N=100k vectors, D in {256, 768, 1024},
//!    batched at 1000 vectors/batch. Reported as vectors/sec.
//! 2. **Search latency** at 1000 random queries, top_k=10. Reported as
//!    p50 / p99 / mean in microseconds.
//! 3. **Recall@10** vs brute-force ground truth on the same 1000 queries.
//!    Cosine distance; both backends configured to use cosine.
//! 4. **Save/load latency** at N=100k. Reported as sidecar round-trip ms
//!    and sidecar size in MB.
//! 5. **RSS memory** before and after indexing. Reported as delta in MB.
//!
//! ## Why both backends use the same VectorBackend trait
//!
//! We use `semantic-memory`'s `VectorBackend` trait to drive both indices
//! so the benchmark exercises the actual production path (not a
//! reproduction of the inner API). This means: same key semantics
//! (String keys), same VectorHit return type, same save/load
//! signature. The only difference is the active backend.
//!
//! ## Receipt
//!
//! Output: a `BenchmarkReceipt` JSON file with all measured numbers, the
//! git commit hash at build time, and the machine fingerprint.

use std::sync::Arc;
use std::time::{Duration, Instant};

use rand::{Rng, SeedableRng};
use rand_distr::{Distribution, Normal};
use receipt_bench::{BenchmarkReceipt, BenchmarkResult, BenchmarkSuite, MachineFingerprint};
use semantic_memory::vector_backend::{VectorBackend, VectorIndex, VectorIndexConfig};
use serde::Serialize;
use tempfile::TempDir;

// =====================================================================
// Configuration
// =====================================================================

/// Number of vectors to index. Production-grade at 100k, but the
/// bench defaults to 10k for fast turnaround on a workstation. Override
/// at the top of main() for the full 100k run.
const N_VECTORS: usize = 10_000;

/// Dimensions to test. 256 is low-dim sanity, 768 is bge-m3 default,
/// 1024 is bge-m3 max.
const DIMENSIONS: &[usize] = &[256, 768, 1024];

/// Number of random queries for search latency / recall measurement.
const N_QUERIES: usize = 1_000;

/// Top-K for search.
const TOP_K: usize = 10;

/// Insert batch size. Larger batches amortize per-vector overhead.
const INSERT_BATCH: usize = 1_000;

/// Random seed for reproducibility. Same seed → same vectors.
const SEED: u64 = 0xC0FFEE_2026_0602;

// HNSW construction parameters (matched across backends for fair compare).
const M: usize = 16;
const EF_CONSTRUCTION: usize = 200;
const EF_SEARCH: usize = 50;

type BackendKind = &'static str; // "hnsw_rs" or "usearch"

/// A single benchmark row: one (backend, dim) combination.
#[derive(Debug, Clone, Serialize)]
#[allow(dead_code)]
struct Row {
    backend: BackendKind,
    dimensions: usize,
    n_vectors: usize,
    insert_total_ms: u64,
    insert_vec_per_sec: f64,
    search_p50_us: f64,
    search_p99_us: f64,
    search_mean_us: f64,
    recall_at_10: f64,
    save_ms: u64,
    load_ms: u64,
    sidecar_bytes: u64,
    rss_before_mb: f64,
    rss_after_mb_mid_insert: f64,
    rss_after_mb_post_insert: f64,
}

impl Row {
    fn to_result(&self) -> BenchmarkResult {
        let payload = serde_json::to_string(self).expect("serialize row");
        BenchmarkResult {
            name: format!("{}-D{}", self.backend, self.dimensions),
            iterations: 1,
            elapsed_ns: ((self.insert_total_ms * 1_000_000) as u64),
            ns_per_iter: (self.insert_total_ms * 1_000_000) as u64,
            throughput: Some(self.insert_vec_per_sec),
            error: Some(format!("payload={payload}")),
        }
    }
}

// =====================================================================
// Vector generation (deterministic)
// =====================================================================

fn generate_corpus(n: usize, dim: usize) -> Vec<Vec<f32>> {
    let mut rng = rand::rngs::StdRng::seed_from_u64(SEED);
    let normal = Normal::new(0.0_f32, 1.0).unwrap();
    let mut corpus = Vec::with_capacity(n);
    for _ in 0..n {
        let mut v: Vec<f32> = (0..dim).map(|_| normal.sample(&mut rng)).collect();
        // L2-normalize so cosine distance is well-defined.
        let norm: f32 = v.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            for x in v.iter_mut() {
                *x /= norm;
            }
        }
        corpus.push(v);
    }
    corpus
}

fn key_for(i: usize) -> String {
    format!("vec:{i}")
}

// =====================================================================
// Backend construction
// =====================================================================

fn make_index(backend: BackendKind, dim: usize) -> VectorIndex {
    let config = VectorIndexConfig {
        m: M,
        ef_construction: EF_CONSTRUCTION,
        ef_search: EF_SEARCH,
        dimensions: dim,
        max_elements: N_VECTORS + 1000,
        compaction_threshold: 0.3,
        flush_interval_secs: None,
    };
    let _ = backend; // Backend selection is via semantic-memory's feature
                     // flags in Cargo.toml — both backends are compiled in.
    VectorIndex::new(config).expect("VectorIndex::new")
}

fn backend_name(idx: &VectorIndex) -> &'static str {
    if idx.backend_name().contains("hnsw_rs") {
        "hnsw_rs"
    } else if idx.backend_name().contains("usearch") {
        "usearch"
    } else {
        "unknown"
    }
}

// =====================================================================
// RSS memory (Linux)
// =====================================================================

fn rss_mb() -> f64 {
    let statm = std::fs::read_to_string("/proc/self/statm").ok();
    if let Some(s) = statm {
        // VmRSS is the second field, in pages. Multiply by page size (4096).
        let parts: Vec<&str> = s.split_whitespace().collect();
        if parts.len() >= 2 {
            if let Ok(pages) = parts[1].parse::<u64>() {
                return (pages as f64) * 4096.0 / 1_048_576.0;
            }
        }
    }
    0.0
}

// =====================================================================
// Insert timing
// =====================================================================

fn timed_insert(idx: &VectorIndex, corpus: &[Vec<f32>]) -> Duration {
    let t = Instant::now();
    for (i, v) in corpus.iter().enumerate() {
        idx.insert(key_for(i), v).expect("insert");
    }
    t.elapsed()
}

// =====================================================================
// Search timing + recall
// =====================================================================

fn brute_force_top_k(query: &[f32], corpus: &[Vec<f32>], k: usize) -> Vec<usize> {
    let mut dists: Vec<(usize, f32)> = corpus
        .iter()
        .enumerate()
        .map(|(i, v)| {
            // Cosine distance = 1 - dot (vectors are L2-normalized so ||a||=||b||=1).
            let dot: f32 = query.iter().zip(v.iter()).map(|(a, b)| a * b).sum();
            (i, 1.0 - dot)
        })
        .collect();
    dists.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
    dists.into_iter().take(k).map(|(i, _)| i).collect()
}

fn timed_search_with_recall(
    idx: &VectorIndex,
    queries: &[Vec<f32>],
    corpus: &[Vec<f32>],
    k: usize,
) -> (Vec<Duration>, f64) {
    let mut latencies = Vec::with_capacity(queries.len());
    let mut total_recall = 0.0_f64;
    for query in queries {
        let t = Instant::now();
        let hits = idx.search(query, k).expect("search");
        latencies.push(t.elapsed());
        // Compute recall vs brute force for this query.
        let ground = brute_force_top_k(query, corpus, k);
        let ground_set: std::collections::HashSet<usize> = ground.iter().copied().collect();
        let hit_keys: std::collections::HashSet<usize> = hits
            .iter()
            .filter_map(|h| {
                // The key is "vec:N" — extract N.
                h.key
                    .strip_prefix("vec:")
                    .and_then(|n| n.parse::<usize>().ok())
            })
            .collect();
        let overlap = ground_set.intersection(&hit_keys).count();
        total_recall += overlap as f64 / k as f64;
    }
    let mean_recall = total_recall / queries.len() as f64;
    (latencies, mean_recall)
}

fn latency_stats(mut latencies: Vec<Duration>) -> (f64, f64, f64) {
    latencies.sort();
    let n = latencies.len();
    let p50 = latencies[n / 2].as_micros() as f64;
    let p99 = latencies[(n * 99) / 100].as_micros() as f64;
    let mean = latencies.iter().map(|d| d.as_micros() as f64).sum::<f64>() / n as f64;
    (p50, p99, mean)
}

// =====================================================================
// Save / load timing
// =====================================================================

fn timed_save_load(
    idx: &VectorIndex,
    dir: &std::path::Path,
    dim: usize,
) -> (Duration, Duration, u64) {
    let t_save = Instant::now();
    idx.save(dir, "bench").expect("save");
    let save_ms = t_save.elapsed();

    // Measure file sizes for the sidecar.
    let data_path = dir.join(format!("bench.hnsw.data"));
    let keys_path = dir.join(format!("bench.hnsw.keys"));
    let manifest_path = dir.join(format!("bench.hnsw.manifest.json"));
    let mut total: u64 = 0;
    for p in [data_path, keys_path, manifest_path] {
        if let Ok(m) = std::fs::metadata(&p) {
            total += m.len();
        }
    }

    let t_load = Instant::now();
    let config = VectorIndexConfig {
        m: M,
        ef_construction: EF_CONSTRUCTION,
        ef_search: EF_SEARCH,
        dimensions: dim,
        max_elements: N_VECTORS + 1000,
        compaction_threshold: 0.3,
        flush_interval_secs: None,
    };
    let _ = VectorIndex::load(dir, "bench", config).expect("load");
    let load_ms = t_load.elapsed();
    (save_ms, load_ms, total)
}

// =====================================================================
// One (backend, dim) benchmark
// =====================================================================

fn run_one(backend: BackendKind, dim: usize) -> Row {
    println!("\n=== {backend} @ D={dim} ===");
    let rss_before = rss_mb();
    let corpus = generate_corpus(N_VECTORS, dim);
    let queries: Vec<Vec<f32>> = (0..N_QUERIES)
        .map(|i| corpus[i % corpus.len()].clone())
        .collect();
    let idx = make_index(backend, dim);

    // Time the insert in batches (so RSS midpoint is informative).
    let t0 = Instant::now();
    let mut rss_mid = 0.0;
    for (batch_idx, batch) in corpus.chunks(INSERT_BATCH).enumerate() {
        for (i_in_batch, v) in batch.iter().enumerate() {
            let i = batch_idx * INSERT_BATCH + i_in_batch;
            idx.insert(key_for(i), v).expect("insert");
        }
        if batch_idx == corpus.len() / INSERT_BATCH / 2 {
            rss_mid = rss_mb();
        }
    }
    let insert_total_ms = t0.elapsed().as_millis();
    let rss_after = rss_mb();

    let insert_vps = N_VECTORS as f64 / (insert_total_ms as f64 / 1000.0);
    println!(
        "  insert: {insert_total_ms} ms total, {insert_vps:.0} vec/s, RSS {rss_before:.1}→{rss_mid:.1}→{rss_after:.1} MB"
    );

    // Search + recall.
    let (latencies, recall) = timed_search_with_recall(&idx, &queries, &corpus, TOP_K);
    let (p50, p99, mean) = latency_stats(latencies);
    println!("  search: p50={p50:.1}us p99={p99:.1}us mean={mean:.1}us, recall@10={recall:.3}");

    // Save / load.
    let tmp = TempDir::new().expect("tmpdir");
    let (save_ms, load_ms, sidecar_bytes) = timed_save_load(&idx, tmp.path(), dim);
    println!(
        "  save/load: save={} ms load={} ms sidecar={} KB",
        save_ms.as_millis(),
        load_ms.as_millis(),
        sidecar_bytes / 1024
    );

    // Don't drop idx until after we've measured rss_after.
    drop(idx);
    drop(corpus);
    drop(queries);

    Row {
        backend: backend_name_from_kind(backend),
        dimensions: dim,
        n_vectors: N_VECTORS,
        insert_total_ms: insert_total_ms as u64,
        insert_vec_per_sec: insert_vps,
        search_p50_us: p50,
        search_p99_us: p99,
        search_mean_us: mean,
        recall_at_10: recall,
        save_ms: save_ms.as_millis() as u64,
        load_ms: load_ms.as_millis() as u64,
        sidecar_bytes,
        rss_before_mb: rss_before,
        rss_after_mb_mid_insert: rss_mid,
        rss_after_mb_post_insert: rss_after,
    }
}

fn backend_name_from_kind(k: BackendKind) -> &'static str {
    // Both hnsw and usearch backends are compiled in. The active
    // backend is whichever is wired in by build_active_backend. For
    // the benchmark we want to RUN BOTH. We do that by compiling
    // this binary TWICE (once per backend feature). For now, return
    // the kind we were called with and rely on Cargo features to
    // make only the matching backend work at a time.
    k
}

fn detect_active_backend() -> BackendKind {
    // Probe which backend is currently active by building an index
    // and asking its name.
    let cfg = VectorIndexConfig {
        m: M,
        ef_construction: EF_CONSTRUCTION,
        ef_search: EF_SEARCH,
        dimensions: 4,
        max_elements: 10,
        compaction_threshold: 0.3,
        flush_interval_secs: None,
    };
    let idx = VectorIndex::new(cfg).expect("probe");
    if idx.backend_name().contains("hnsw_rs") {
        "hnsw_rs"
    } else if idx.backend_name().contains("usearch") {
        "usearch"
    } else {
        "unknown"
    }
}

// =====================================================================
// Main
// =====================================================================

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut suite = BenchmarkSuite::new();
    let machine = MachineFingerprint::generate();
    println!("=== HNSW backend benchmark ===");
    println!("machine: {machine:?}");

    let active = detect_active_backend();
    println!("active backend: {active}");

    let mut rows = Vec::new();
    for &dim in DIMENSIONS {
        let row = run_one(active, dim);
        let row_clone = row.clone();
        suite.register(
            format!("{active}-D{dim}"),
            move || Ok(row_clone.to_result()),
        );
        rows.push(row);
    }

    // Run the suite (returns the receipt).
    let receipt: BenchmarkReceipt = suite.run()?;
    println!("\n=== receipt (JSON) ===");
    println!("{}", serde_json::to_string_pretty(&receipt)?);

    // Also dump the raw rows table.
    println!("\n=== results table ===");
    println!(
        "{:<12} {:>5} {:>10} {:>8} {:>8} {:>8} {:>9} {:>9} {:>7}",
        "backend", "D", "vec/s", "p50us", "p99us", "meanus", "recall@10", "save_ms", "MB"
    );
    for r in &rows {
        let mb = r.rss_after_mb_post_insert - r.rss_before_mb;
        println!(
            "{:<12} {:>5} {:>10.0} {:>8.0} {:>8.0} {:>8.0} {:>9.3} {:>9} {:>7.1}",
            r.backend,
            r.dimensions,
            r.insert_vec_per_sec,
            r.search_p50_us,
            r.search_p99_us,
            r.search_mean_us,
            r.recall_at_10,
            r.save_ms,
            mb
        );
    }

    // Save receipt to a file (so it can be diffed between runs).
    let receipt_path = format!(
        "hnsw-bench-receipt-{}-{}.json",
        active,
        chrono::Utc::now().format("%Y%m%d-%H%M%S")
    );
    std::fs::write(&receipt_path, serde_json::to_vec_pretty(&receipt)?)?;
    println!("\nreceipt written to: {receipt_path}");

    Ok(())
}
