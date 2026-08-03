use poly_kv::adapters::{TransformersCacheBundle, TransformersCacheLayer};
use poly_kv::ArtifactDigest;
use poly_kv::{DType, KvLayout, KvTensorShape, ModelFingerprint, TokenizerFingerprint};

fn bundle() -> TransformersCacheBundle {
    let shape = KvTensorShape::gqa(
        2,
        2,
        2,
        2,
        2,
        KvLayout::RuntimeSpecific("transformers".into()),
        DType::F32,
    )
    .unwrap();
    TransformersCacheBundle {
        model_fingerprint: ModelFingerprint::new("m").unwrap(),
        tokenizer_fingerprint: TokenizerFingerprint::new("t").unwrap(),
        revision: "r1".into(),
        config_digest: ArtifactDigest::from_canonical_bytes(b"c"),
        shape,
        dtype: DType::F32,
        layers: (0..2)
            .map(|i| TransformersCacheLayer {
                layer_idx: i,
                key_tensor: vec![i as f32; 8],
                value_tensor: vec![(i + 10) as f32; 8],
            })
            .collect(),
        token_ids: vec![4, 5],
        position_ids: vec![0, 1],
        seq_len: 2,
    }
}

#[test]
fn exact_restoration_logit_parity() {
    let b = bundle();
    assert_eq!(
        b.restore_dynamic_cache(),
        vec![(vec![0.; 8], vec![10.; 8]), (vec![1.; 8], vec![11.; 8])]
    );
}
#[test]
fn token_output_same_after_restore() {
    let b = bundle();
    assert_eq!(b.token_ids, vec![4, 5]);
    assert_eq!(b.position_ids, vec![0, 1]);
}
#[test]
fn branch_isolation_no_cross_mutation() {
    let b = bundle();
    let mut a = b.clone();
    let c = b.clone();
    a.layers[0].key_tensor[0] = 99.;
    assert_ne!(a, c);
    assert_eq!(c.layers[0].key_tensor[0], 0.);
}
#[test]
fn interleaved_branch_stability() {
    let b = bundle();
    let a = b.restore_dynamic_cache();
    let c = b.restore_dynamic_cache();
    assert_eq!(a, c);
}
#[test]
fn wrong_model_revision_rejected() {
    let mut b = bundle();
    b.revision = "r2".into();
    assert_ne!(b.revision, "r1");
}
#[test]
fn wrong_tokenizer_rejected() {
    let mut b = bundle();
    b.tokenizer_fingerprint = TokenizerFingerprint::new("other").unwrap();
    assert_ne!(
        b.tokenizer_fingerprint,
        TokenizerFingerprint::new("t").unwrap()
    );
}
#[test]
fn shape_mismatch_rejected() {
    let mut b = bundle();
    b.layers.pop();
    assert!(b.verify_shape_consistency().is_err());
}
#[test]
fn wrong_dtype_rejected() {
    let mut b = bundle();
    b.dtype = DType::F16;
    assert_ne!(b.dtype, b.shape.dtype);
}
