mod common;

use common::*;
use poly_kv::*;

#[test]
fn synthetic_exact_fallback_roundtrip() {
    for shape in [shape_mha(), shape_mqa(), shape_gqa()] {
        let blocks = blocks_for(&shape);
        let pool = SharedKvPool::builder()
            .model_fingerprint(ModelFingerprint::new("synthetic:test-model").unwrap())
            .tokenizer_fingerprint(TokenizerFingerprint::new("synthetic:test-tokenizer").unwrap())
            .shape(shape.clone())
            .policy(CompressionPolicyV1::alpha_reference())
            .exact_fallback(ExactFallback::from_blocks(blocks.clone()))
            .key_codec(Q8KeyCodec::symmetric_per_block())
            .value_codec(RawExactValueCodec)
            .build_from_blocks(blocks.clone())
            .unwrap();
        let reader = pool.attach_reader(ReaderConfig::default()).unwrap();
        let req = KvSliceRequest::layer_span(LayerId(0), TokenSpan::new(0, shape.seq_len).unwrap())
            .for_role(KvRole::Value);
        let decoded = reader.decode_slice(req.clone()).unwrap();
        let exact = blocks
            .iter()
            .find(|block| block.layer == LayerId(0) && block.role == KvRole::Value)
            .unwrap();
        assert_eq!(decoded.data, exact.data);
        assert!(decoded.receipt.fallback.is_none());
        assert!(decoded.receipt.full_block_decoded);
        assert_eq!(
            decoded.receipt.decoded_full_values,
            shape.layer_element_count(KvRole::Value).unwrap() as u64
        );
        assert_eq!(decoded.receipt.returned_values, decoded.data.len() as u64);
        assert!(decoded.receipt.copy_performed);

        let fallback = reader.decode_slice_exact_fallback(req).unwrap();
        assert_eq!(fallback.data, exact.data);
        assert!(fallback.receipt.fallback.is_some());
    }
}

#[test]
fn synthetic_q8_key_drift_is_bounded_and_finite() {
    let shape = shape_mha();
    let blocks = blocks_for(&shape);
    let key = blocks
        .iter()
        .find(|block| block.role == KvRole::Key && block.layer == LayerId(0))
        .unwrap();
    let codec = Q8KeyCodec::symmetric_per_block();
    let encoded = codec.encode_block(&key.data).unwrap();
    let eval = codec.eval(&key.data, &encoded).unwrap();
    assert!(eval.passed, "{eval:?}");
    assert!(eval.mse.unwrap() <= 0.001);
    assert!(eval.cosine_similarity.unwrap() > 0.999);
    assert!(eval.max_abs_error.unwrap().is_finite());
}

#[test]
fn synthetic_decode_layer_returns_key_and_value() {
    let shape = shape_mha();
    let pool = build_pool(shape.clone());
    let reader = pool.attach_reader(ReaderConfig::default()).unwrap();
    let layer = reader.decode_layer(LayerId(1)).unwrap();
    assert_eq!(
        layer.key.data.len(),
        shape.layer_element_count(KvRole::Key).unwrap()
    );
    assert_eq!(
        layer.value.data.len(),
        shape.layer_element_count(KvRole::Value).unwrap()
    );
}

#[test]
fn builder_can_derive_exact_fallback_from_input_blocks() {
    let shape = shape_mha();
    let blocks = blocks_for(&shape);
    let pool = SharedKvPool::builder()
        .model_fingerprint(ModelFingerprint::new("synthetic:test-model").unwrap())
        .tokenizer_fingerprint(TokenizerFingerprint::new("synthetic:test-tokenizer").unwrap())
        .shape(shape)
        .build_from_exact_blocks(blocks)
        .unwrap();

    assert!(pool.exact_fallback_ref().is_some());
    assert_eq!(
        pool.build_receipt().exact_fallback_bytes,
        pool.manifest().exact_fallback_bytes
    );
}
