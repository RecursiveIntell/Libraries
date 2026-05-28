//! Integration tests for quant-eval.

use quant_eval::{
    AdmissibilityTest, CodecProfile, CompressionBenchmark, CompressionBenchmarkConfig,
    SemanticMemoryBenchmark, SemanticMemoryConfig,
};

#[test]
fn test_compression_benchmark_integration() {
    let config = CompressionBenchmarkConfig {
        dim: 64,
        db_size: 100,
        queries: 10,
        seed: 42,
        top_k: 5,
        iterations: 10,
    };

    let benchmark = CompressionBenchmark::with_config(config);
    let result = benchmark.run().expect("benchmark should succeed");

    assert_eq!(result.queries, 10);
    assert_eq!(result.db_size, 100);
    assert!(result.recall_at_k >= 0.0 && result.recall_at_k <= 1.0);
    assert!(result.mrr >= 0.0 && result.mrr <= 1.0);
}

#[test]
fn test_semantic_memory_benchmark_integration() {
    let config = SemanticMemoryConfig {
        dim: 32,
        index_size: 100,
        num_queries: 5,
        top_k: 5,
        seed: 42,
    };

    let benchmark = SemanticMemoryBenchmark::with_config(config);
    let result = benchmark.run().expect("benchmark should succeed");

    assert_eq!(result.queries, 5);
    assert!(result.raw_quality.ndcg_at_k >= 0.0 && result.raw_quality.ndcg_at_k <= 1.0);
    assert!(result.degradation_ratio >= 0.0 && result.degradation_ratio <= 1.0);
}

#[test]
fn test_admissibility_test_integration() {
    let test = AdmissibilityTest::new();
    let test_vectors = AdmissibilityTest::standard_test_vectors(32);

    let summary = test
        .run(&test_vectors)
        .expect("admissibility test should succeed");

    assert_eq!(summary.total, test_vectors.len() * 3); // 3 profiles
    assert!(summary.passed > 0);
}

#[test]
fn test_codec_profiles() {
    let profiles = CodecProfile::standard_profiles();
    assert_eq!(profiles.len(), 3);

    let fast = CodecProfile::fast();
    assert_eq!(fast.name, "fast");
    assert!(fast.compression_ratio > 0.0);

    let balanced = CodecProfile::balanced();
    assert_eq!(balanced.name, "balanced");

    let high = CodecProfile::high_compression();
    assert_eq!(high.name, "high_compression");
    assert!(high.compression_ratio > balanced.compression_ratio);
}

#[test]
fn test_receipt_round_trip() {
    use quant_eval::{BenchmarkReceipt, BenchmarkResult, MachineFingerprint};

    let fp = MachineFingerprint::from_hex("00".repeat(64).as_str());
    let mut receipt = BenchmarkReceipt::with_fingerprint("abc123".to_string(), &fp);

    receipt.add_result(BenchmarkResult {
        name: "test".to_string(),
        iterations: 100,
        elapsed_ns: 1_000_000,
        ns_per_iter: 10_000,
        throughput: Some(100_000.0),
        error: None,
    });

    let json = receipt.to_json().expect("should serialize");
    let parsed = BenchmarkReceipt::from_json(&json).expect("should deserialize");

    assert_eq!(parsed.commit_hash, "abc123");
    assert_eq!(parsed.results.len(), 1);
    assert_eq!(parsed.results[0].name, "test");
}
