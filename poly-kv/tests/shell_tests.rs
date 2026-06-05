//! Shell-specific tests: deterministic materialize, digests, manifests.

use poly_kv::pool::SharedKVPool;
use poly_kv::shape::{AttentionType, KvTensorShape};
use poly_kv::AgentId;

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
fn test_shell_materialize_deterministic() {
    let shape = make_test_shape();
    let corpus = make_corpus(4);
    let (pool, _receipt) = SharedKVPool::build(&corpus, &shape, 42).unwrap();
    let agent_tokens = make_corpus(2);

    let (shell1, receipt1) = pool
        .materialize_shell("agent_x", &agent_tokens, 42)
        .unwrap();
    let (shell2, receipt2) = pool
        .materialize_shell("agent_x", &agent_tokens, 42)
        .unwrap();

    assert_eq!(receipt1.shell_digest, receipt2.shell_digest);
    assert_eq!(
        shell1.unique_layers[0].block_digest,
        shell2.unique_layers[0].block_digest
    );
}

#[test]
fn test_shell_manifest_validates_against_schema() {
    let shape = make_test_shape();
    let corpus = make_corpus(4);
    let (pool, _receipt) = SharedKVPool::build(&corpus, &shape, 42).unwrap();
    let agent_tokens = make_corpus(2);

    let (shell, _receipt) = pool
        .materialize_shell("agent_x", &agent_tokens, 42)
        .unwrap();
    assert!(shell.shell_manifest.validate().is_ok());
    assert_eq!(
        shell.shell_manifest.schema_version,
        poly_kv::SHELL_MANIFEST_SCHEMA
    );
}

#[test]
fn test_shell_empty_agent_produces_valid_shell() {
    let shape = make_test_shape();
    let corpus = make_corpus(4);
    let (pool, _receipt) = SharedKVPool::build(&corpus, &shape, 42).unwrap();

    let (shell, receipt) = pool.materialize_shell("empty_agent", &[], 42).unwrap();
    assert_eq!(shell.agent_id, AgentId::new("empty_agent"));
    assert_eq!(receipt.num_unique_tokens, 0);
    assert_eq!(receipt.shell_size_bytes, 0);
    assert_eq!(shell.pool_digest, pool.manifest.pool_id);
}

#[test]
fn test_shell_blocks_have_correct_codec() {
    let shape = make_test_shape();
    let corpus = make_corpus(4);
    let (pool, _receipt) = SharedKVPool::build(&corpus, &shape, 42).unwrap();
    let agent_tokens = make_corpus(2);

    let (shell, _receipt) = pool
        .materialize_shell("agent_x", &agent_tokens, 42)
        .unwrap();
    assert!(!shell.unique_layers.is_empty());
    for layer in &shell.unique_layers {
        for block in &layer.key_blocks {
            assert_eq!(block.codec, poly_kv::CODEC_TURBO_8BIT);
        }
    }
}

#[test]
fn test_shell_different_agents_produce_different_digests() {
    let shape = make_test_shape();
    let corpus = make_corpus(4);
    let (pool, _receipt) = SharedKVPool::build(&corpus, &shape, 42).unwrap();
    let agent_tokens = make_corpus(2);

    let (_shell1, receipt1) = pool
        .materialize_shell("agent_a", &agent_tokens, 42)
        .unwrap();
    let (_shell2, receipt2) = pool
        .materialize_shell("agent_b", &agent_tokens, 42)
        .unwrap();

    // Different agent IDs should produce different digests
    assert_ne!(receipt1.shell_digest, receipt2.shell_digest);
}

#[test]
fn test_shell_size_nonzero() {
    let shape = make_test_shape();
    let corpus = make_corpus(4);
    let (pool, _receipt) = SharedKVPool::build(&corpus, &shape, 42).unwrap();
    let agent_tokens = make_corpus(2);

    let (_shell, receipt) = pool
        .materialize_shell("agent_x", &agent_tokens, 42)
        .unwrap();
    assert!(receipt.shell_size_bytes > 0);
}

#[test]
fn test_shell_materialize_from_different_pools() {
    let shape = make_test_shape();
    let corpus1 = make_corpus(3);
    let corpus2 = make_corpus(5);
    let (pool1, _r1) = SharedKVPool::build(&corpus1, &shape, 42).unwrap();
    let (pool2, _r2) = SharedKVPool::build(&corpus2, &shape, 42).unwrap();
    let agent_tokens = make_corpus(1);

    let (_shell1, receipt1) = pool1
        .materialize_shell("agent_x", &agent_tokens, 42)
        .unwrap();
    let (_shell2, receipt2) = pool2
        .materialize_shell("agent_x", &agent_tokens, 42)
        .unwrap();

    assert_ne!(
        receipt1.shell_digest, receipt2.shell_digest,
        "Same agent tokens on different pools should produce different digests"
    );
}
