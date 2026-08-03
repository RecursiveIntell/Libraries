#![cfg(feature = "fibquant-adapter")]

use poly_kv::adapters::fibquant::FibQuantValueCodec;
use poly_kv::{ValueCodec, VectorCodec};

#[test]
fn fibquant_value_codec_round_trips_with_receipt_backed_eval() {
    let codec = FibQuantValueCodec::new(8, 2, 4, 7)
        .expect("valid fibquant profile")
        .with_max_mse(1.0)
        .expect("valid quality budget");
    let input: Vec<f32> = (0..8).map(|value| (value as f32 + 1.0) / 8.0).collect();

    let encoded = codec.encode_values(&input).expect("encode succeeds");
    assert!(!encoded.is_empty());

    let mut decoded: Vec<f32> = vec![0.0; input.len()];
    codec
        .decode_values(&encoded, &mut decoded)
        .expect("decode succeeds");
    assert!(decoded.iter().all(|value| value.is_finite()));

    let report = codec
        .eval_values(&input, &encoded)
        .expect("evaluation succeeds");
    assert!(report.passed, "report: {report:?}");
    assert!(report.bytes_encoded > 0);
    assert!(report.bytes_exact > 0);
    assert!(report.cosine_similarity.unwrap_or_default() > 0.0);
}

#[test]
fn fibquant_value_codec_rejects_wrong_dimension_and_corrupt_payload() {
    let codec = FibQuantValueCodec::new(8, 2, 4, 7).expect("valid fibquant profile");
    let wrong_dimension = vec![1.0_f32; 7];
    let error = codec
        .encode_values(&wrong_dimension)
        .expect_err("dimension must fail");
    assert!(error.to_string().contains("dimension"));

    let corrupt = vec![0xde, 0xad, 0xbe, 0xef];
    let mut output = vec![0.0_f32; 8];
    let error = codec
        .decode_values(&corrupt, &mut output)
        .expect_err("corrupt payload must fail closed");
    assert!(error.to_string().contains("serialization") || error.to_string().contains("codec"));
}

#[test]
fn fibquant_value_codec_rejects_cross_profile_payload() {
    let expected = FibQuantValueCodec::new(8, 2, 4, 7).expect("valid expected profile");
    let substituted = FibQuantValueCodec::new(8, 2, 4, 8).expect("valid substituted profile");
    let input: Vec<f32> = (0..8).map(|value| (value as f32 + 1.0) / 8.0).collect();
    let encoded = substituted
        .encode_values(&input)
        .expect("substituted profile encodes");
    let mut output = vec![0.0_f32; 8];

    let error = expected
        .decode_values(&encoded, &mut output)
        .expect_err("cross-profile payload must fail closed");
    assert!(error.to_string().contains("profile"));
}

#[test]
fn fibquant_value_codec_requires_explicit_quality_budget_to_pass() {
    let uncalibrated = FibQuantValueCodec::new(8, 2, 4, 7).expect("valid profile");
    let input: Vec<f32> = (0..8).map(|value| (value as f32 + 1.0) / 8.0).collect();
    let encoded = uncalibrated.encode_values(&input).expect("encode succeeds");
    let report = uncalibrated
        .eval_values(&input, &encoded)
        .expect("evaluation succeeds");
    assert!(!report.passed);
    assert!(report
        .notes
        .iter()
        .any(|note| note.contains("quality budget")));
    assert!(encoded.len() as u64 <= uncalibrated.resource_limits().max_encoded_bytes);
}
