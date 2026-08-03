#![cfg(feature = "kv")]

use fib_quant::kv::{
    decode_kv_pages, encode_kv_tensor, KvAttentionKind, KvAxisPolicyV1, KvCacheLayoutV1,
    KvCompressionProfileV1, KvDType, KvEncodedTensorV1, KvFallbackModeV1, KvPageGeometryV1, KvRole,
    KvRopeState, KvTensorShapeV1,
};
use fib_quant::{FibQuantProfileV1, FibQuantizer};

fn fixture(tokens_per_page: u32) -> KvEncodedTensorV1 {
    let shape = KvTensorShapeV1::new(
        KvRole::Value,
        KvAttentionKind::Mha,
        1,
        1,
        1,
        1,
        4,
        8,
        KvDType::F32,
        KvRopeState::NotApplicable,
    );
    let layout = KvCacheLayoutV1::canonical(&shape).unwrap();
    let mut fib_profile = FibQuantProfileV1::paper_default(8, 2, 8, 17).unwrap();
    fib_profile.training_samples = 64;
    fib_profile.lloyd_restarts = 1;
    fib_profile.lloyd_iterations = 1;
    let quantizer = FibQuantizer::new(fib_profile.clone()).unwrap();
    let profile = KvCompressionProfileV1::from_parts(
        "corruption",
        &shape,
        fib_profile,
        quantizer.codebook().codebook_digest.clone(),
        KvAxisPolicyV1::PerToken,
        KvPageGeometryV1::new(tokens_per_page, 8, 64),
    )
    .unwrap();
    let values: Vec<f32> = (0..shape.element_count().unwrap())
        .map(|idx| idx as f32 * 0.05 + 0.5)
        .collect();
    encode_kv_tensor(shape, layout, profile, &values).unwrap()
}

fn refresh_page_digest(encoded: &mut KvEncodedTensorV1, page_index: usize) {
    let digest = encoded.pages[page_index]
        .compute_digest(&encoded.shape)
        .unwrap();
    encoded.pages[page_index].page_digest = digest.clone();
    encoded.receipt.page_digests[page_index] = digest;
}

#[test]
fn page_digest_tamper_rejects_decode() {
    let mut encoded = fixture(2);
    encoded.pages[0].page_digest.push_str("-tampered");
    assert!(decode_kv_pages(&encoded).is_err());
}

#[test]
fn structurally_consistent_missing_page_rejects_decode() {
    let mut encoded = fixture(2);
    encoded.pages.pop();
    encoded.receipt.encoded_pages = encoded.pages.len() as u32;
    encoded.receipt.page_digests = encoded
        .pages
        .iter()
        .map(|page| page.page_digest.clone())
        .collect();
    encoded.receipt.compressed_blocks = encoded
        .pages
        .iter()
        .map(|page| page.encoded_blocks.len() as u32)
        .sum();

    assert!(decode_kv_pages(&encoded).is_err());
}

#[test]
fn duplicate_block_coordinate_rejects_decode() {
    let mut encoded = fixture(2);
    encoded.pages[0].encoded_blocks[1].token = encoded.pages[0].encoded_blocks[0].token;
    refresh_page_digest(&mut encoded, 0);

    assert!(decode_kv_pages(&encoded).is_err());
}

#[test]
fn page_source_digest_must_match_receipt() {
    let mut encoded = fixture(2);
    encoded.pages[0].source_tensor_digest = "blake3:wrong-source".to_string();
    refresh_page_digest(&mut encoded, 0);

    assert!(decode_kv_pages(&encoded).is_err());
}

#[test]
fn block_reservation_must_match_page_geometry() {
    let mut encoded = fixture(2);
    encoded.pages[0].encoded_blocks[0].fixed_size_bytes += 1;
    refresh_page_digest(&mut encoded, 0);

    assert!(decode_kv_pages(&encoded).is_err());
}

#[test]
fn receipt_counts_must_match_realized_pages_and_blocks() {
    let mut encoded = fixture(2);
    encoded.receipt.compressed_blocks += 1;

    assert!(decode_kv_pages(&encoded).is_err());
}

#[test]
fn keep_raw_policy_requires_declared_and_sized_fallback() {
    let encoded = fixture(2);
    let mut no_fallback = encoded.profile.clone();
    no_fallback.fallback_policy.raw_fallback_available = false;
    assert!(no_fallback.validate_for_shape(&encoded.shape).is_err());

    let mut undersized = encoded.profile.clone();
    undersized.page_geometry.encoded_block_bytes = 1;
    assert!(undersized.validate_for_shape(&encoded.shape).is_err());

    let mut fail_closed = encoded.profile.clone();
    fail_closed.fallback_policy.mode = KvFallbackModeV1::FailClosed;
    fail_closed.fallback_policy.raw_fallback_available = false;
    fail_closed.validate_for_shape(&encoded.shape).unwrap();
}
