//! Integration tests: end-to-end pool + shell flows, round-trips, policy validation.

use poly_kv::policy::{CompressionPolicy, FibConfig, TurboConfig};
use poly_kv::pool::SharedKVPool;
use poly_kv::shape::{AttentionType, KvTensorShape};

fn make_test_shape() -> KvTensorShape {
    KvTensorShape {
        attention_type: AttentionType::MHA,
        num_layers: 2,
        num_heads: 4,
        num_kv_heads: 4,
        head_dim: 8,
        hidden_size: 32,
    }
}

fn make_corpus(n: usize) -> Vec<(String, Vec<f32>)> {
    use rand::Rng;
    use rand_chacha::{rand_core::SeedableRng, ChaCha8Rng};
    let mut rng = ChaCha8Rng::seed_from_u64(42);
    let shape = make_test_shape();
    let vec_len = shape.num_layers as usize * shape.num_kv_heads as usize * shape.head_dim * 2;
    (0..n)
        .map(|i| {
            let vec: Vec<f32> = (0..vec_len).map(|_| rng.gen_range(-1.0..1.0)).collect();
            (format!("token_{}", i), vec)
        })
        .collect()
}

#[test]
fn test_round_trip_pool_build_shell_materialize_digests_match() {
    let shape = make_test_shape();
    let corpus = make_corpus(4);
    let (pool, pool_receipt) = SharedKVPool::build(&corpus, &shape, 42).unwrap();
    let agent_tokens = make_corpus(2);

    let (shell, shell_receipt) = pool
        .materialize_shell("agent_x", &agent_tokens, 42)
        .unwrap();

    // Shell references its parent pool
    assert_eq!(shell.pool_digest, pool.manifest.pool_id);
    assert_eq!(shell_receipt.pool_digest, pool_receipt.pool_digest);

    // Pool and shell have proper sizes
    assert!(pool_receipt.pool_size_bytes > 0);
    assert!(shell_receipt.shell_size_bytes > 0);

    // Compression ratios are computed (may be < 1.0 for tiny test corpora
    // where JSON serialization overhead dominates)
    assert!(pool_receipt.compression_ratio > 0.0);
}

#[test]
fn test_fib_config_validation_rejects_invalid_k() {
    let mut config = FibConfig::default_k4_n32();
    config.k = 0;
    assert!(config.validate().is_err());
}

#[test]
fn test_fib_config_validation_rejects_invalid_n() {
    let mut config = FibConfig::default_k4_n32();
    config.n = 1;
    assert!(config.validate().is_err());
}

#[test]
fn test_turbo_config_validation_rejects_invalid_bits() {
    let mut config = TurboConfig::default_8bit();
    config.bits = 1; // min is 2
    assert!(config.validate().is_err());

    config.bits = 17; // max is 16
    assert!(config.validate().is_err());
}

#[test]
fn test_policy_validation() {
    let policy = CompressionPolicy::default_two_tier();
    assert!(policy.validate().is_ok());

    // Invalid shared codec
    let mut bad_policy = policy.clone();
    bad_policy.shared_codec = "unknown_codec".into();
    assert!(bad_policy.validate().is_err());

    // Invalid shell codec
    let mut bad_policy2 = policy.clone();
    bad_policy2.shell_codec = "unknown_codec".into();
    assert!(bad_policy2.validate().is_err());
}

#[test]
fn test_shape_validation_rejects_invalid_head_dim() {
    let mut shape = make_test_shape();
    shape.head_dim = 0;
    assert!(shape.validate().is_err());
}

#[test]
fn test_shape_validation_rejects_zero_layers() {
    let mut shape = make_test_shape();
    shape.num_layers = 0;
    assert!(shape.validate().is_err());
}

#[test]
fn test_gqa_shape_validation() {
    let shape = KvTensorShape {
        attention_type: AttentionType::GQA,
        num_layers: 40,
        num_heads: 32,
        num_kv_heads: 8,
        head_dim: 128,
        hidden_size: 4096,
    };
    assert!(shape.validate().is_ok());

    // GQA requires num_kv_heads < num_heads
    let mut bad = shape.clone();
    bad.num_kv_heads = 32; // same as num_heads
    assert!(bad.validate().is_err());
}

#[test]
fn test_mqa_shape_validation() {
    let shape = KvTensorShape {
        attention_type: AttentionType::MQA,
        num_layers: 32,
        num_heads: 32,
        num_kv_heads: 1,
        head_dim: 128,
        hidden_size: 4096,
    };
    assert!(shape.validate().is_ok());

    // MQA requires num_kv_heads == 1
    let mut bad = shape.clone();
    bad.num_kv_heads = 2;
    assert!(bad.validate().is_err());
}

#[test]
fn test_mha_shape_requires_kv_heads_match() {
    let mut shape = make_test_shape();
    shape.num_kv_heads = 2; // but num_heads is 4 and type is MHA
    assert!(shape.validate().is_err());
}

#[test]
fn test_non_finite_values_rejected() {
    let shape = make_test_shape();
    let vec_len = shape.num_layers as usize * shape.num_kv_heads as usize * shape.head_dim * 2;
    let mut vec: Vec<f32> = vec![0.1; vec_len];
    vec[0] = f32::NAN;
    let corpus = vec![("bad_token".to_string(), vec)];
    let result = SharedKVPool::build(&corpus, &shape, 42);
    assert!(result.is_err());
}
