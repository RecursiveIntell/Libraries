#![cfg(feature = "fibquant-adapter")]

mod common;

use common::{blocks_for, shape_mha};
use poly_kv::adapters::fibquant::FibQuantValueCodec;
use poly_kv::*;

#[test]
fn fibquant_is_reachable_through_pool_builder_and_reader() {
    let shape = shape_mha();
    let blocks = blocks_for(&shape);
    let codec = FibQuantValueCodec::new(shape.head_dim as usize, 2, 4, 7)
        .expect("valid FibQuant profile")
        .with_max_mse(1.0)
        .expect("finite value quality budget");
    let mut policy = CompressionPolicyV1::alpha_reference();
    policy.quality_gate.max_value_mse = 1.0;

    let pool = SharedKvPool::builder()
        .model_fingerprint(ModelFingerprint::new("synthetic:fibquant-model").unwrap())
        .tokenizer_fingerprint(TokenizerFingerprint::new("synthetic:fibquant-tokenizer").unwrap())
        .shape(shape.clone())
        .policy(policy)
        .exact_fallback(ExactFallback::from_blocks(blocks.clone()))
        .key_codec(Q8KeyCodec::symmetric_per_block())
        .value_codec(codec)
        .build_from_blocks(blocks.clone())
        .expect("FibQuant pool build");

    assert_eq!(
        pool.manifest().policy.value_codec_id.as_str(),
        "poly-kv:value:fibquant"
    );
    assert!(pool.manifest().policy.lossy_values);
    assert!(pool.manifest().policy.quality_gate.passed);
    assert_eq!(
        pool.manifest().encoded_bytes,
        pool.build_receipt().encoded_bytes
    );
    assert!(pool
        .build_receipt()
        .compression_evals
        .iter()
        .any(|eval| eval.role == KvRole::Value && eval.eval.passed));

    let reader = pool.attach_reader(ReaderConfig::default()).unwrap();
    let request = KvSliceRequest::layer_span(LayerId(0), TokenSpan::new(0, shape.seq_len).unwrap())
        .for_role(KvRole::Value);
    let decoded = reader.decode_slice(request).expect("reader decode");

    assert_eq!(
        decoded.data.len(),
        shape.layer_element_count(KvRole::Value).unwrap()
    );
    assert!(decoded.data.iter().all(|value| value.is_finite()));
    assert!(decoded.receipt.fallback.is_none());
    assert!(decoded.receipt.full_block_decoded);
}
