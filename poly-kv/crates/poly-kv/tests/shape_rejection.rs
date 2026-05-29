mod common;

use common::*;
use poly_kv::*;

#[test]
fn mismatched_shape_is_rejected() {
    let shape = shape_mha();
    let mut blocks = blocks_for(&shape);
    blocks[0].shape =
        KvTensorShape::gqa(2, 2, 2, 8, 8, KvLayout::LayersHeadsTokensDim, DType::F32).unwrap();
    let err = SharedKvPool::builder()
        .model_fingerprint(ModelFingerprint::new("synthetic:test-model").unwrap())
        .tokenizer_fingerprint(TokenizerFingerprint::new("synthetic:test-tokenizer").unwrap())
        .shape(shape)
        .exact_fallback(ExactFallback::from_blocks(blocks.clone()))
        .build_from_blocks(blocks)
        .unwrap_err();
    assert!(matches!(err, PolyKvError::ShapeMismatch { .. }));
}

#[test]
fn malformed_token_span_is_rejected() {
    let pool = build_pool(shape_mha());
    let reader = pool.attach_reader(ReaderConfig::default()).unwrap();
    assert!(TokenSpan::new(4, 4).is_err());

    let req = KvSliceRequest::layer_span(LayerId(0), TokenSpan::new(7, 9).unwrap());
    let err = reader.decode_slice(req).unwrap_err();
    assert!(matches!(err, PolyKvError::InvalidSpan { .. }));
}
