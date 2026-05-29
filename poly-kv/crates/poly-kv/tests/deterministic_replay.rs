mod common;

use common::*;
use poly_kv::*;

#[test]
fn build_receipt_is_deterministic_for_same_fixture() {
    let shape = shape_gqa();
    let pool_a = build_pool(shape.clone());
    let pool_b = build_pool(shape);

    assert_eq!(
        pool_a.build_receipt().input_digest,
        pool_b.build_receipt().input_digest
    );
    assert_eq!(
        pool_a.build_receipt().manifest_digest,
        pool_b.build_receipt().manifest_digest
    );
    assert_eq!(pool_a.manifest().blocks, pool_b.manifest().blocks);
}

#[test]
fn build_receipt_is_deterministic_for_reordered_input() {
    let shape = shape_mqa();
    let mut blocks = blocks_for(&shape);
    blocks.reverse();
    let fallback = ExactFallback::from_blocks(blocks.clone());
    let pool = SharedKvPool::builder()
        .model_fingerprint(ModelFingerprint::new("synthetic:test-model").unwrap())
        .tokenizer_fingerprint(TokenizerFingerprint::new("synthetic:test-tokenizer").unwrap())
        .shape(shape.clone())
        .exact_fallback(fallback)
        .build_from_blocks(blocks)
        .unwrap();

    let canonical = build_pool(shape);
    assert_eq!(
        pool.build_receipt().manifest_digest,
        canonical.build_receipt().manifest_digest
    );
}
