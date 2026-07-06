use quant_eval::{run_compressed_attention_eval, CompressedAttentionConfig};

type VectorBatch = Vec<Vec<f32>>;

fn fixture_vectors() -> (VectorBatch, VectorBatch, VectorBatch) {
    let keys = vec![
        vec![1.0, 0.0, 0.0, 0.0],
        vec![0.9, 0.1, 0.0, 0.0],
        vec![0.0, 1.0, 0.0, 0.0],
        vec![0.0, 0.0, 1.0, 0.0],
    ];
    let values = vec![
        vec![1.0, 0.0, 0.0, 0.0],
        vec![0.8, 0.2, 0.0, 0.0],
        vec![0.0, 1.0, 0.0, 0.0],
        vec![0.0, 0.0, 1.0, 0.0],
    ];
    let queries = vec![vec![1.0, 0.05, 0.0, 0.0], vec![0.0, 0.95, 0.05, 0.0]];
    (keys, values, queries)
}

#[test]
fn compressed_attention_eval_emits_topk_decode_receipt() {
    let (keys, values, queries) = fixture_vectors();
    let receipt = run_compressed_attention_eval(
        &keys,
        &values,
        &queries,
        &CompressedAttentionConfig {
            bits: 8,
            top_k: 2,
            min_mean_output_cosine: 0.80,
            max_mean_output_mse: 0.10,
            min_top_k_overlap: 0.50,
        },
    )
    .expect("compressed attention eval succeeds");

    assert_eq!(receipt.schema, "compressed-attention-eval-v1");
    assert_eq!(
        receipt.scoring_path,
        "compressed_key_logits_topk_value_decode"
    );
    assert_eq!(receipt.query_count, 2);
    assert_eq!(receipt.cache_len, 4);
    assert_eq!(receipt.top_k, 2);
    assert_eq!(receipt.decompressed_value_count, 4);
    assert!(receipt.mean_output_cosine >= 0.80, "{receipt:?}");
    assert!(receipt.mean_top_k_overlap >= 0.50, "{receipt:?}");
    assert!(receipt.passed, "blockers: {:?}", receipt.blockers);
    assert!(receipt
        .claim_boundary
        .contains("attention fixture evidence only"));
}

#[test]
fn compressed_attention_eval_rejects_mismatched_cache_lengths() {
    let (keys, mut values, queries) = fixture_vectors();
    values.pop();
    let err = run_compressed_attention_eval(
        &keys,
        &values,
        &queries,
        &CompressedAttentionConfig::default(),
    )
    .expect_err("mismatched cache lengths must fail");

    assert!(err.to_string().contains("same length"));
}
