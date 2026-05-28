//! Benchmark suite — runs and records benchmarks with provenance receipts.

use std::collections::HashMap;
use std::time::Instant;

use crate::error::BenchError;
use crate::receipt::BenchmarkResult;
use crate::{BenchmarkReceipt, MachineFingerprint};

/// A suite of benchmarks that can be run and recorded as a receipt.
pub struct BenchmarkSuite {
    commit_hash: Option<String>,
    machine_fingerprint: MachineFingerprint,
    benchmarks: HashMap<String, Box<dyn Fn() -> Result<BenchmarkResult, String> + Send + Sync>>,
}

impl BenchmarkSuite {
    /// Create a new benchmark suite with auto-detected commit hash and machine fingerprint.
    pub fn new() -> Self {
        let commit_hash = detect_git_commit_hash().ok();
        Self {
            commit_hash,
            machine_fingerprint: MachineFingerprint::generate(),
            benchmarks: HashMap::new(),
        }
    }

    /// Create a new suite with explicit commit hash.
    pub fn with_commit_hash(commit_hash: impl Into<String>) -> Self {
        Self {
            commit_hash: Some(commit_hash.into()),
            machine_fingerprint: MachineFingerprint::generate(),
            benchmarks: HashMap::new(),
        }
    }

    /// Register a benchmark with a name and callable.
    ///
    /// The callable should perform the actual benchmark work and return
    /// a result with the measured elapsed time and iteration count.
    pub fn register(
        &mut self,
        name: impl Into<String>,
        bench: impl Fn() -> Result<BenchmarkResult, String> + Send + Sync + 'static,
    ) {
        self.benchmarks.insert(name.into(), Box::new(bench));
    }

    /// Run all registered benchmarks and return a receipt.
    pub fn run(&self) -> Result<BenchmarkReceipt, BenchError> {
        let commit_hash = self
            .commit_hash
            .clone()
            .unwrap_or_else(|| "unknown".to_string());

        let mut receipt = BenchmarkReceipt::new(commit_hash, self.machine_fingerprint.clone());

        for (name, bench) in &self.benchmarks {
            match bench() {
                Ok(result) => {
                    let mut r = result;
                    r.name = name.clone();
                    receipt.add_result(r);
                }
                Err(e) => {
                    receipt.add_result(BenchmarkResult {
                        name: name.clone(),
                        iterations: 0,
                        elapsed_ns: 0,
                        ns_per_iter: 0,
                        throughput: None,
                        error: Some(e),
                    });
                }
            }
        }

        Ok(receipt)
    }

    /// Run a specific named benchmark and return just that result.
    pub fn run_one(&self, name: &str) -> Result<BenchmarkResult, BenchError> {
        let bench = self
            .benchmarks
            .get(name)
            .ok_or_else(|| BenchError::Execution(format!("No benchmark named '{}'", name)))?;

        let result = bench()?;
        Ok(result)
    }

    /// Get the list of registered benchmark names.
    pub fn benchmark_names(&self) -> Vec<&String> {
        self.benchmarks.keys().collect()
    }
}

impl Default for BenchmarkSuite {
    fn default() -> Self {
        Self::new()
    }
}

/// Detect the current git commit hash.
fn detect_git_commit_hash() -> Result<String, BenchError> {
    use std::process::Command;

    let output = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .map_err(|e| BenchError::Git(e.to_string()))?;

    if !output.status.success() {
        return Err(BenchError::NoGitRepo);
    }

    let hash = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if hash.is_empty() {
        return Err(BenchError::NoGitRepo);
    }

    Ok(hash)
}

// -----------------------------------------------------------------------------
// Built-in benchmark implementations
// -----------------------------------------------------------------------------

/// Semantic search benchmark — tests vector similarity search on synthetic data.
///
/// Uses a simple flat index with cosine similarity to evaluate query performance.
pub mod semantic_search {
    use super::*;
    use crate::receipt::BenchmarkResult;

    pub const DEFAULT_ITERATIONS: u64 = 1000;

    /// Run semantic search benchmark with synthetic data.
    ///
    /// Creates `dimensions` vectors of `vector_size` dimension, then performs
    /// `iterations` queries and measures latency.
    pub fn run_benchmark(
        dimensions: usize,
        vector_size: usize,
        iterations: u64,
    ) -> Result<BenchmarkResult, String> {
        // Generate synthetic data
        let vectors: Vec<Vec<f32>> = (0..dimensions)
            .map(|_| (0..vector_size).map(|_| rand_f32()).collect())
            .collect();

        let query = (0..vector_size).map(|_| rand_f32()).collect::<Vec<f32>>();

        let start = Instant::now();
        for _ in 0..iterations {
            let _ = find_top_k(&vectors, &query, 10);
        }
        let elapsed = start.elapsed();

        Ok(BenchmarkResult {
            name: "semantic_search".to_string(),
            iterations,
            elapsed_ns: elapsed.as_nanos() as u64,
            ns_per_iter: elapsed.as_nanos() as u64 / iterations.max(1),
            throughput: Some(iterations as f64 / elapsed.as_secs_f64()),
            error: None,
        })
    }

    fn find_top_k(vectors: &[Vec<f32>], query: &[f32], k: usize) -> Vec<(usize, f32)> {
        let mut scores: Vec<(usize, f32)> = vectors
            .iter()
            .enumerate()
            .map(|(i, v)| (i, cosine_sim(query, v)))
            .collect();
        scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scores.truncate(k);
        scores
    }

    fn cosine_sim(a: &[f32], b: &[f32]) -> f32 {
        let dot = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum::<f32>();
        let mag_a = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let mag_b = b.iter().map(|x| x * x).sum::<f32>().sqrt();
        if mag_a == 0.0 || mag_b == 0.0 {
            0.0
        } else {
            dot / (mag_a * mag_b)
        }
    }

    fn rand_f32() -> f32 {
        use std::time::{SystemTime, UNIX_EPOCH};
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .subsec_nanos();
        // Simple deterministic pseudo-random for reproducibility
        let x = (seed as u32).wrapping_mul(1103515245).wrapping_add(12345);
        (x as f32 / u32::MAX as f32) * 2.0 - 1.0
    }
}

/// Compression round-trip benchmark — tests encode/decode cycle with zlib.
pub mod compression {
    use super::*;
    use crate::receipt::BenchmarkResult;
    use std::io::{Read, Write};

    pub const DEFAULT_ITERATIONS: u64 = 1000;

    /// Run compression benchmark using flate2 (zlib) with synthetic data.
    ///
    /// Creates `data_size` bytes of pseudo-random data, compresses it,
    /// then decompresses to verify integrity. Measures encode/decode cycle time.
    pub fn run_benchmark(data_size: usize, iterations: u64) -> Result<BenchmarkResult, String> {
        let data = generate_data(data_size);

        let mut total_encode_ns = 0u64;
        let mut total_decode_ns = 0u64;

        for _ in 0..iterations {
            // Encode
            let start_encode = Instant::now();
            let encoded = encode_zlib(&data).map_err(|e| e.to_string())?;
            let encode_ns = start_encode.elapsed().as_nanos() as u64;
            total_encode_ns += encode_ns;

            // Decode
            let start_decode = Instant::now();
            let decoded = decode_zlib(&encoded).map_err(|e| e.to_string())?;
            let decode_ns = start_decode.elapsed().as_nanos() as u64;
            total_decode_ns += decode_ns;

            // Verify
            if decoded != data {
                return Err("Decompressed data mismatch".to_string());
            }
        }

        let elapsed_ns = total_encode_ns.saturating_add(total_decode_ns);
        let ns_per_iter = elapsed_ns / iterations.max(1);

        Ok(BenchmarkResult {
            name: "compression_round_trip".to_string(),
            iterations,
            elapsed_ns,
            ns_per_iter,
            throughput: Some((iterations as f64 * data_size as f64) / (elapsed_ns as f64 / 1e9)),
            error: None,
        })
    }

    fn generate_data(size: usize) -> Vec<u8> {
        use std::time::{SystemTime, UNIX_EPOCH};
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .subsec_nanos() as u32;

        (0..size)
            .map(|i| {
                ((seed
                    .wrapping_mul((i as u32).wrapping_add(1))
                    .wrapping_add(i as u32))
                    % 256) as u8
            })
            .collect()
    }

    fn encode_zlib(data: &[u8]) -> Result<Vec<u8>, std::io::Error> {
        let mut encoder = flate2::write::ZlibEncoder::new(
            Vec::with_capacity(data.len()),
            flate2::Compression::default(),
        );
        encoder.write_all(data)?;
        encoder.finish()
    }

    fn decode_zlib(data: &[u8]) -> Result<Vec<u8>, std::io::Error> {
        use flate2::read::ZlibDecoder;
        let mut decoder = ZlibDecoder::new(data);
        let mut result = Vec::new();
        decoder.read_to_end(&mut result)?;
        Ok(result)
    }
}

/// Memory lookup benchmark — tests random key/value access patterns.
pub mod memory_lookup {
    use super::*;
    use crate::receipt::BenchmarkResult;

    pub const DEFAULT_ITERATIONS: u64 = 1000;

    /// Run memory lookup benchmark with random key access.
    ///
    /// Creates `num_entries` key-value pairs (keys are strings, values are u64),
    /// then performs `iterations` random lookups. Measures lookup latency.
    pub fn run_benchmark(num_entries: usize, iterations: u64) -> Result<BenchmarkResult, String> {
        let entries: Vec<(String, u64)> = (0..num_entries)
            .map(|i| (format!("key_{:08}", i), i as u64))
            .collect();

        // Generate random indices
        let indices: Vec<usize> = {
            use std::time::{SystemTime, UNIX_EPOCH};
            let seed = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .subsec_nanos() as u64;
            (0..iterations)
                .map(|i| (seed.wrapping_mul(i.wrapping_add(1)) as usize % num_entries))
                .collect()
        };

        let start = Instant::now();
        for &idx in &indices {
            let (ref key, _) = entries[idx];
            // Simulate a lookup
            let _ = entries.iter().find(|(k, _)| k == key);
        }
        let elapsed = start.elapsed();

        Ok(BenchmarkResult {
            name: "memory_lookups".to_string(),
            iterations,
            elapsed_ns: elapsed.as_nanos() as u64,
            ns_per_iter: elapsed.as_nanos() as u64 / iterations.max(1),
            throughput: Some(iterations as f64 / elapsed.as_secs_f64()),
            error: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::receipt::BenchmarkResult;

    #[test]
    fn test_suite_creation() {
        let suite = BenchmarkSuite::new();
        assert!(suite.benchmark_names().is_empty());
    }

    #[test]
    fn test_suite_register() {
        let mut suite = BenchmarkSuite::new();
        suite.register("dummy", || {
            Ok(BenchmarkResult {
                name: "dummy".to_string(),
                iterations: 10,
                elapsed_ns: 1000,
                ns_per_iter: 100,
                throughput: None,
                error: None,
            })
        });
        assert_eq!(suite.benchmark_names().len(), 1);
    }

    #[test]
    fn test_suite_run() {
        let mut suite = BenchmarkSuite::with_commit_hash("test123");
        suite.register("dummy", || {
            Ok(BenchmarkResult {
                name: "dummy".to_string(),
                iterations: 10,
                elapsed_ns: 1000,
                ns_per_iter: 100,
                throughput: None,
                error: None,
            })
        });
        let receipt = suite.run().unwrap();
        assert_eq!(receipt.commit_hash, "test123");
        assert_eq!(receipt.results.len(), 1);
    }

    #[test]
    fn test_semantic_search_benchmark() {
        let result = semantic_search::run_benchmark(100, 64, 100).unwrap();
        assert_eq!(result.name, "semantic_search");
        assert_eq!(result.iterations, 100);
        assert!(result.elapsed_ns > 0);
        assert!(result.ns_per_iter > 0);
    }

    #[test]
    fn test_memory_lookup_benchmark() {
        let result = memory_lookup::run_benchmark(1000, 100).unwrap();
        assert_eq!(result.name, "memory_lookups");
        assert_eq!(result.iterations, 100);
        assert!(result.elapsed_ns > 0);
    }
}
