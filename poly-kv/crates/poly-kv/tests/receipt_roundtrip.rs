mod common;

use common::*;
use poly_kv::*;

#[test]
fn manifests_and_receipts_roundtrip_through_serde() {
    let pool = build_pool(shape_mha());
    let manifest_json = serde_json::to_string(pool.manifest()).unwrap();
    let manifest: KvPoolManifestV1 = serde_json::from_str(&manifest_json).unwrap();
    assert_eq!(manifest.manifest_digest, pool.manifest().manifest_digest);

    let build_json = serde_json::to_string(pool.build_receipt()).unwrap();
    let build: PoolBuildReceiptV1 = serde_json::from_str(&build_json).unwrap();
    assert_eq!(build.manifest_digest, pool.build_receipt().manifest_digest);
    assert_eq!(
        build.compression_evals.len(),
        pool.build_receipt().block_count as usize
    );
    assert!(build
        .compression_evals
        .iter()
        .any(|receipt| receipt.role == KvRole::Key
            && receipt.eval.mse.is_some()
            && receipt.ideal_codec_bits_per_scalar == Some(8.0)
            && receipt.realized_encoded_bytes > 0
            && receipt.metadata_bytes > 0));

    let reader = pool.attach_reader(ReaderConfig::default()).unwrap();
    let reader_json = serde_json::to_string(reader.injection_receipt()).unwrap();
    let injection: ReaderInjectionReceiptV1 = serde_json::from_str(&reader_json).unwrap();
    assert_eq!(
        injection.manifest_digest,
        reader.injection_receipt().manifest_digest
    );

    let decoded = reader
        .decode_slice(
            KvSliceRequest::layer_span(LayerId(0), TokenSpan::new(0, 2).unwrap())
                .for_role(KvRole::Key),
        )
        .unwrap();
    let decode_json = serde_json::to_string(&decoded.receipt).unwrap();
    let decode: DecodeReceiptV1 = serde_json::from_str(&decode_json).unwrap();
    assert_eq!(decode.decoded_values, decoded.receipt.decoded_values);
}

#[test]
fn manifest_memory_uses_canonical_serialized_length() {
    let pool = build_pool(shape_mha());
    assert_eq!(
        pool.build_receipt().memory.manifest_bytes,
        pool.manifest().canonical_serialized_len()
    );
    assert!(pool.manifest().canonical_serialized_len() > 0);
}
