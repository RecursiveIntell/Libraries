use poly_kv::{
    run_captured_model_replay, run_model_replay, AttentionType, CapturedReplayConfig,
    CapturedReplayFixture, CapturedReplayQuery, KvTensorShape, ModelReplayConfig, ModelReplayQuery,
    SharedKVPool, CAPTURED_MODEL_REPLAY_RECEIPT_SCHEMA, MODEL_REPLAY_RECEIPT_SCHEMA,
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

#[test]
fn test_distilgpt2_captured_replay_fixture_runs_through_poly_kv_gate() {
    let fixture_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("docs/codex-runs/P3/POLY_KV_CAPTURED_DISTILGPT2_FIXTURE.json");
    let fixture: CapturedReplayFixture = serde_json::from_str(
        &std::fs::read_to_string(&fixture_path).expect("distilgpt2 captured fixture must exist"),
    )
    .unwrap();

    assert!(fixture.model_id.contains("distilgpt2"));
    assert!(fixture.head_dim >= 16);
    assert!(fixture.shared_tokens > 0);
    assert!(fixture.queries.len() >= 2);

    let receipt = run_captured_model_replay(
        &fixture,
        CapturedReplayConfig {
            candidate_ks: vec![16, 32, 48, 64],
            min_output_cosine: -1.0,
            max_output_mse: 100.0,
            max_kl_divergence: 20.0,
            max_ppl_delta: 1_000_000.0,
            min_top1_agreement: 0.0,
        },
    )
    .unwrap();

    assert_eq!(receipt.schema_version, CAPTURED_MODEL_REPLAY_RECEIPT_SCHEMA);
    assert!(receipt.passed);
    assert!(receipt.metrics.logit_vectors_compared >= 2);
    assert!(receipt.metrics.decode_reduction > 1.0);
}

#[test]
fn test_captured_model_replay_uses_captured_logits_and_adaptive_candidates() {
    let keys = vec![
        vec![1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        vec![0.7, 0.2, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        vec![0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        vec![0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        vec![0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0],
        vec![0.5, 0.5, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
    ];
    let values = keys.clone();
    let query = vec![1.0, 0.1, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
    let projection = vec![
        vec![1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        vec![0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        vec![0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        vec![0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0],
    ];
    let exact_output = exact_attention_for_test(&query, &keys, &values);
    let exact_logits: Vec<f32> = projection
        .iter()
        .map(|row| row.iter().zip(&exact_output).map(|(a, b)| a * b).sum())
        .collect();
    let fixture = CapturedReplayFixture {
        schema_version: "poly_kv_captured_replay_fixture_v1".to_string(),
        model_id: "tiny-transformer-unit-fixture".to_string(),
        head_dim: 8,
        shared_tokens: 4,
        seed: 77,
        output_projection: projection,
        queries: vec![CapturedReplayQuery {
            query,
            keys,
            values,
            exact_attention_output: exact_output,
            exact_logits,
            label_token: 0,
        }],
    };
    let receipt = run_captured_model_replay(
        &fixture,
        CapturedReplayConfig {
            candidate_ks: vec![1, 3, 6],
            min_output_cosine: 0.40,
            max_output_mse: 0.75,
            max_kl_divergence: 0.75,
            max_ppl_delta: 5.0,
            min_top1_agreement: 0.0,
        },
    )
    .unwrap();

    assert_eq!(receipt.schema_version, CAPTURED_MODEL_REPLAY_RECEIPT_SCHEMA);
    assert_eq!(receipt.model_id, "tiny-transformer-unit-fixture");
    assert!(receipt
        .config
        .candidate_ks
        .contains(&receipt.selected_candidate_k));
    assert!(receipt.metrics.logit_vectors_compared > 0);
    assert!(receipt.metrics.decoded_values_total < receipt.metrics.full_decode_value_count);
    assert!(receipt.claim_boundary.contains("captured tensor"));
    assert!(receipt.passed);
}

fn exact_attention_for_test(query: &[f32], keys: &[Vec<f32>], values: &[Vec<f32>]) -> Vec<f32> {
    let scores: Vec<f32> = keys
        .iter()
        .map(|key| query.iter().zip(key).map(|(a, b)| a * b).sum())
        .collect();
    let max = scores.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    let mut weights: Vec<f32> = scores.iter().map(|score| (*score - max).exp()).collect();
    let denom: f32 = weights.iter().sum();
    for weight in &mut weights {
        *weight /= denom;
    }
    let mut out = vec![0.0; values[0].len()];
    for (weight, value) in weights.iter().zip(values) {
        for (dst, v) in out.iter_mut().zip(value) {
            *dst += *weight * *v;
        }
    }
    out
}
