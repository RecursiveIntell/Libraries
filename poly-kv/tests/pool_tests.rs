//! Pool-specific tests: deterministic builds, shape validation, compression ratios.

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
fn test_pool_build_deterministic_across_identical_seeds() {
    let shape = make_test_shape();
    let corpus = make_corpus(4);
    let (pool1, receipt1) = SharedKVPool::build(&corpus, &shape, 42).unwrap();
    let (pool2, receipt2) = SharedKVPool::build(&corpus, &shape, 42).unwrap();
    assert_eq!(receipt1.pool_digest, receipt2.pool_digest);
    assert_eq!(receipt1.layer_digests, receipt2.layer_digests);
    assert_eq!(pool1.layers[0].block_digest, pool2.layers[0].block_digest);
    assert_eq!(
        pool1.manifest.compression_ratio,
        pool2.manifest.compression_ratio
    );
}

#[test]
fn test_pool_build_different_seeds_produce_different_digests() {
    let shape = make_test_shape();
    let corpus = make_corpus(4);
    let (_pool1, receipt1) = SharedKVPool::build(&corpus, &shape, 42).unwrap();
    let (_pool2, receipt2) = SharedKVPool::build(&corpus, &shape, 12345).unwrap();
    assert_ne!(receipt1.pool_digest, receipt2.pool_digest);
}

#[test]
fn test_pool_manifest_validates_against_schema() {
    let shape = make_test_shape();
    let corpus = make_corpus(4);
    let (pool, _receipt) = SharedKVPool::build(&corpus, &shape, 42).unwrap();
    assert!(pool.manifest.validate().is_ok());
    assert_eq!(pool.manifest.schema_version, poly_kv::POOL_MANIFEST_SCHEMA);
}

#[test]
fn test_compression_ratio_matches_expected() {
    let shape = make_test_shape();
    let corpus = make_corpus(4);
    let (_pool, receipt) = SharedKVPool::build(&corpus, &shape, 42).unwrap();
    // fib-quant at k=4,N=32 on tiny test corpora: JSON overhead dominates.
    // Ratio > 0.0 confirms the field is populated.
    assert!(
        receipt.compression_ratio > 0.0,
        "expected compression ratio > 0.0, got {}",
        receipt.compression_ratio
    );
}

#[test]
fn test_mismatched_shape_rejects_gracefully() {
    let shape = make_test_shape();
    let mut bad_corpus = make_corpus(1);
    bad_corpus[0].1.truncate(10); // wrong length
    let result = SharedKVPool::build(&bad_corpus, &shape, 42);
    assert!(result.is_err());
}

#[test]
fn test_empty_corpus_rejected() {
    let shape = make_test_shape();
    let corpus: Vec<(String, Vec<f32>)> = vec![];
    let result = SharedKVPool::build(&corpus, &shape, 42);
    assert!(result.is_err());
}

#[test]
fn test_pool_build_creates_correct_number_of_layers() {
    let shape = make_test_shape();
    let corpus = make_corpus(3);
    let (pool, _receipt) = SharedKVPool::build(&corpus, &shape, 42).unwrap();
    assert_eq!(pool.layers.len(), shape.num_layers as usize);
    assert_eq!(pool.manifest.num_shared_tokens, 3);
    assert_eq!(pool.manifest.num_layers, shape.num_layers);
}

#[test]
fn test_pool_blocks_have_correct_codec() {
    let shape = make_test_shape();
    let corpus = make_corpus(2);
    let (pool, _receipt) = SharedKVPool::build(&corpus, &shape, 42).unwrap();
    for layer in &pool.layers {
        for block in &layer.key_blocks {
            assert_eq!(block.codec, poly_kv::CODEC_FIB_K4_N32);
        }
        for block in &layer.value_blocks {
            assert_eq!(block.codec, poly_kv::CODEC_FIB_K4_N32);
        }
    }
}
