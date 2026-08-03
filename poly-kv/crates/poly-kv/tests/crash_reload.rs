// CMP-002: PolyKV bounded vertical slice — crash/reload and cross-version
// fixtures.
//
// RED: a crash/reload round trip could lose blocks, reader state, or
// manifest information; an unsupported/corrupted manifest could be silently
// defaulted instead of failing closed.
//
// GREEN: restore_from_manifest rebuilds an identical pool (same decoded
// slice, same encoded byte accounting); wrong schema version -> typed
// UnsupportedSchemaVersion; corrupted manifest digest -> typed Manifest
// error.
mod common;

use common::*;
use poly_kv::*;

fn build_pool(shape: KvTensorShape) -> (SharedKvPool, Vec<ExactKvBlock>) {
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
    (pool, blocks)
}

#[test]
fn crash_reload_restores_identical_pool() {
    let shape = shape_mha();
    let (pool, blocks) = build_pool(shape.clone());

    // Persist the manifest (simulate a crash boundary: pool dropped, only
    // manifest bytes + exact blocks survive).
    let manifest_json = serde_json::to_string(pool.manifest()).unwrap();
    let encoded_bytes_before = pool.encoded_bytes();
    drop(pool);

    let manifest: poly_kv::manifest::KvPoolManifestV1 =
        serde_json::from_str(&manifest_json).unwrap();
    let restored = PoolBuilder::restore_from_manifest(&manifest, blocks.clone()).unwrap();

    // Manifest information is preserved exactly.
    assert_eq!(restored.manifest(), &manifest);
    assert_eq!(restored.encoded_bytes(), encoded_bytes_before);

    // Decoded content is identical to the original exact block.
    let reader = restored.attach_reader(ReaderConfig::default()).unwrap();
    let req = KvSliceRequest::layer_span(LayerId(0), TokenSpan::new(0, shape.seq_len).unwrap())
        .for_role(KvRole::Value);
    let decoded = reader.decode_slice(req.clone()).unwrap();
    let exact = blocks
        .iter()
        .find(|block| block.layer == LayerId(0) && block.role == KvRole::Value)
        .unwrap();
    assert_eq!(decoded.data, exact.data);
    assert!(decoded.receipt.fallback.is_none());

    // Reader state is fresh but reads identically (per-reader isolation).
    let reader2 = restored.attach_reader(ReaderConfig::default()).unwrap();
    let decoded2 = reader2.decode_slice(req).unwrap();
    assert_eq!(decoded2.data, decoded.data);
}

#[test]
fn cross_version_manifest_rejected_typed() {
    let shape = shape_mha();
    let (pool, blocks) = build_pool(shape);
    let mut manifest = pool.manifest().clone();
    manifest.schema_version = 999;

    let err = PoolBuilder::restore_from_manifest(&manifest, blocks).unwrap_err();
    match err {
        PolyKvError::UnsupportedSchemaVersion { got, expected } => {
            assert_eq!(got, 999);
            assert_eq!(expected, poly_kv::manifest::CURRENT_SCHEMA_VERSION);
        }
        other => panic!("expected UnsupportedSchemaVersion, got {other:?}"),
    }
}

#[test]
fn corrupted_manifest_digest_rejected_typed() {
    let shape = shape_mha();
    let (pool, blocks) = build_pool(shape);
    let mut manifest = pool.manifest().clone();
    // Tamper content without updating the digest: corruption must fail
    // closed, never silently restore with wrong metadata.
    manifest.encoded_bytes += 1;

    let err = PoolBuilder::restore_from_manifest(&manifest, blocks).unwrap_err();
    match err {
        PolyKvError::Manifest(msg) => {
            assert!(msg.contains("digest"), "unexpected message: {msg}");
        }
        other => panic!("expected Manifest corruption error, got {other:?}"),
    }
}
