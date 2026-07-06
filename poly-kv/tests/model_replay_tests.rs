use poly_kv::{
    run_model_replay, AttentionType, KvTensorShape, ModelReplayConfig, ModelReplayQuery,
    SharedKVPool, MODEL_REPLAY_RECEIPT_SCHEMA,
};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

fn shape() -> KvTensorShape {
    KvTensorShape {
        attention_type: AttentionType::MHA,
        num_layers: 1,
        num_heads: 2,
        num_kv_heads: 2,
        head_dim: 16,
        hidden_size: 32,
    }
}

fn corpus(n: usize, shape: &KvTensorShape, seed: u64) -> Vec<(String, Vec<f32>)> {
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let len = shape.num_layers as usize * shape.num_kv_heads as usize * shape.head_dim * 2;
    (0..n)
        .map(|i| {
            let values = (0..len)
                .map(|j| {
                    ((i as f32 * 0.021) + (j as f32 * 0.037)).sin() + rng.gen_range(-0.05..0.05)
                })
                .collect();
            (format!("token_{i}"), values)
        })
        .collect()
}

#[test]
fn test_model_replay_receipt_measures_attention_logits_and_ppl_proxy() {
    let shape = shape();
    let shared = corpus(32, &shape, 7);
    let hot = corpus(8, &shape, 8);
    let (pool, _pool_receipt) = SharedKVPool::build(&shared, &shape, 9).unwrap();
    let (shell, _shell_receipt) = pool
        .materialize_shell("agent_model_replay", &hot, 10)
        .unwrap();

    let queries = vec![
        ModelReplayQuery {
            query: (0..shape.head_dim)
                .map(|i| (i as f32 * 0.11).sin())
                .collect(),
            label_token: 3,
        },
        ModelReplayQuery {
            query: (0..shape.head_dim)
                .map(|i| (i as f32 * 0.07).cos())
                .collect(),
            label_token: 5,
        },
    ];
    let config = ModelReplayConfig {
        layer: 0,
        head: 0,
        candidate_ks: vec![4, 16, 40],
        vocab_size: 32,
        projection_seed: 123,
        min_output_cosine: 0.50,
        max_output_mse: 0.75,
        max_kl_divergence: 0.75,
        max_ppl_delta: 2.0,
        min_top1_agreement: 0.0,
    };

    let receipt = run_model_replay(&pool, &shell, &queries, config).unwrap();

    assert_eq!(receipt.schema_version, MODEL_REPLAY_RECEIPT_SCHEMA);
    assert!(receipt
        .config
        .candidate_ks
        .contains(&receipt.selected_candidate_k));
    assert!(receipt.metrics.exact_attention_outputs > 0);
    assert!(receipt.metrics.logit_vectors_compared > 0);
    assert!(receipt.metrics.ppl_proxy_exact > 0.0);
    assert!(receipt.metrics.decoded_values_total < receipt.metrics.full_decode_value_count);
    assert!(receipt.claim_boundary.contains("not real model PPL"));
    assert!(receipt
        .candidate_results
        .iter()
        .any(|candidate| candidate.passed));
    assert!(receipt.passed);
}

#[test]
fn test_model_replay_rejects_empty_candidate_list() {
    let shape = shape();
    let shared = corpus(4, &shape, 17);
    let hot = corpus(2, &shape, 18);
    let (pool, _) = SharedKVPool::build(&shared, &shape, 19).unwrap();
    let (shell, _) = pool
        .materialize_shell("agent_model_replay", &hot, 20)
        .unwrap();
    let queries = vec![ModelReplayQuery {
        query: vec![0.1; shape.head_dim],
        label_token: 0,
    }];
    let err = run_model_replay(&pool, &shell, &queries, ModelReplayConfig::default()).unwrap_err();
    assert!(err.to_string().contains("candidate_ks must not be empty"));
}
