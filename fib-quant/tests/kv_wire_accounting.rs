#![cfg(feature = "kv")]

use fib_quant::kv::{
    encode_kv_tensor, encode_kv_wire, KvAttentionKind, KvAxisPolicyV1, KvCacheLayoutV1,
    KvCompressionProfileV1, KvDType, KvPageGeometryV1, KvRole, KvRopeState, KvTensorShapeV1,
};
use fib_quant::{FibQuantProfileV1, FibQuantizer};

fn fixture() -> (Vec<f32>, fib_quant::kv::KvEncodedTensorV1) {
    let shape = KvTensorShapeV1::new(
        KvRole::Value,
        KvAttentionKind::Mha,
        1,
        1,
        1,
        1,
        1,
        4,
        KvDType::F32,
        KvRopeState::NotApplicable,
    );
    let values = vec![0.25, -0.5, 0.75, 1.0];
    let fib_profile = FibQuantProfileV1::paper_default(4, 2, 4, 7).unwrap();
    let quantizer = FibQuantizer::new(fib_profile.clone()).unwrap();
    let profile = KvCompressionProfileV1::from_parts(
        "wire-accounting",
        &shape,
        fib_profile,
        quantizer.codebook().codebook_digest.clone(),
        KvAxisPolicyV1::PerToken,
        KvPageGeometryV1::new(1, 4, 1040),
    )
    .unwrap();
    let layout = KvCacheLayoutV1::canonical(&shape).unwrap();
    let tensor = encode_kv_tensor(shape, layout, profile, &values).unwrap();
    (values, tensor)
}

#[test]
fn wire_accounting_is_measured_against_raw_and_json_candidates() {
    let (values, tensor) = fixture();
    let json = serde_json::to_vec(&tensor).unwrap();
    let wire = encode_kv_wire(&tensor).unwrap();
    let raw_bytes = values.len() * std::mem::size_of::<f32>();

    println!(
        "wire-accounting raw_f32_bytes={} json_envelope_bytes={} framed_wire_bytes={} wire_header_bytes={}",
        raw_bytes,
        json.len(),
        wire.len(),
        fib_quant::kv::KV_WIRE_HEADER_LEN
    );

    assert!(wire.len() < json.len());
    assert!(wire.len() > fib_quant::kv::KV_WIRE_HEADER_LEN);
    assert_ne!(wire.len(), raw_bytes);
}
