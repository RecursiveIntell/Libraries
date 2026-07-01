use hyperquant::LatticeKind;
use quant_eval::{run_hyperquant_retrieval_benchmark, HyperQuantRetrievalBenchmarkConfig};

#[test]
fn hyperquant_retrieval_benchmark_reports_latency_and_quality() {
    let config = HyperQuantRetrievalBenchmarkConfig {
        dim: 32,
        docs: 128,
        queries: 12,
        clusters: 8,
        top_k: 5,
        seed: 11,
        scale: 16.0,
        lattice: LatticeKind::D4,
    };

    let receipt = run_hyperquant_retrieval_benchmark(&config).expect("benchmark succeeds");

    assert_eq!(receipt.config, config);
    assert_eq!(receipt.queries, 12);
    assert_eq!(receipt.docs, 128);
    assert_eq!(receipt.top_k, 5);
    assert_eq!(receipt.lattice, LatticeKind::D4);
    assert_eq!(receipt.rejected_docs, 0);
    assert_eq!(receipt.rejected_queries, 0);
    assert!(receipt.raw_latency_ns.p50 > 0);
    assert!(receipt.hyperquant_latency_ns.p50 > 0);
    assert!(receipt.raw_latency_ns.p95 >= receipt.raw_latency_ns.p50);
    assert!(receipt.hyperquant_latency_ns.p95 >= receipt.hyperquant_latency_ns.p50);
    assert!(receipt.quality.recall_at_k >= 0.0 && receipt.quality.recall_at_k <= 1.0);
    assert!(receipt.quality.ndcg_at_k >= 0.0 && receipt.quality.ndcg_at_k <= 1.0);
    assert!(receipt.quality.exact_rerank_recovery_at_1 >= 0.0);
    assert!(receipt.quality.top_k_overlap >= 0.0 && receipt.quality.top_k_overlap <= 1.0);
    assert!(receipt.score_error.mean >= 0.0);
    assert!(receipt.rank_drift.p95 >= receipt.rank_drift.mean);
    assert!(receipt.raw_bytes > receipt.hyperquant_estimated_bytes);
    assert!(receipt.compression_ratio > 1.0);
    assert!(receipt.claim_boundary.contains("synthetic"));
}

#[test]
fn hyperquant_retrieval_benchmark_rejects_invalid_config() {
    let config = HyperQuantRetrievalBenchmarkConfig {
        dim: 0,
        ..HyperQuantRetrievalBenchmarkConfig::default()
    };

    assert!(run_hyperquant_retrieval_benchmark(&config).is_err());
}

#[test]
fn hyperquant_retrieval_benchmark_round_trips_json() {
    let config = HyperQuantRetrievalBenchmarkConfig {
        dim: 16,
        docs: 64,
        queries: 8,
        clusters: 4,
        top_k: 3,
        seed: 7,
        scale: 12.0,
        lattice: LatticeKind::A2,
    };

    let receipt = run_hyperquant_retrieval_benchmark(&config).expect("benchmark succeeds");
    let json = serde_json::to_string(&receipt).expect("receipt serializes");
    let decoded: quant_eval::HyperQuantRetrievalBenchmarkReceipt =
        serde_json::from_str(&json).expect("receipt deserializes");

    assert_eq!(decoded, receipt);
}
