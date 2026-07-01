use quant_eval::{run_synthetic_compressed_attention_bench, CompressedAttentionBenchConfig};

#[test]
fn synthetic_compressed_attention_receipt_reports_quality_and_bytes() {
    let cfg = CompressedAttentionBenchConfig {
        model_or_fixture: "synthetic".into(),
        codec: "toy".into(),
        context_len: 4,
        top_k: 2,
        raw_fp16_cache_bytes: 1024,
        compressed_cache_bytes: 128,
        quality_threshold_cosine: 0.99,
    };
    let receipt = run_synthetic_compressed_attention_bench(
        cfg,
        &[1.0, 0.5, -0.2, 0.1],
        &[0.99, 0.49, -0.1, 0.0],
    );
    assert_eq!(
        receipt.schema_version,
        "compressed_cache_attention_bench_v1"
    );
    assert!(receipt.compression_ratio > 7.9);
    assert!(receipt.topk_overlap >= 1.0);
    assert!(receipt.passed);
}
