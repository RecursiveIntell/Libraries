use hyperquant::{quantize_a2, quantize_z1, ClaimBoundary, HyperQuantConfig, LatticeKind};

fn must<T, E: core::fmt::Debug>(result: core::result::Result<T, E>) -> T {
    match result {
        Ok(value) => value,
        Err(err) => panic!("expected Ok(..), got {err:?}"),
    }
}

#[test]
fn receipt_records_digests_and_claim_boundary() {
    let input = [0.125, -0.5, 1.5];
    let result = must(quantize_z1(&input, 16.0));
    let receipt = result.receipt();

    assert_eq!(receipt.kind, LatticeKind::Z1);
    assert_eq!(receipt.input_len, input.len());
    assert_eq!(receipt.code_len, result.codes.len());
    assert_eq!(receipt.mse, result.mse);
    assert_eq!(
        receipt.claim_boundary,
        ClaimBoundary::ExperimentalPrimitiveOnly
    );
    assert_eq!(receipt.input_digest, result.input_digest);
    assert_eq!(receipt.config_digest, result.config_digest);
    assert_ne!(receipt.input_digest, receipt.config_digest);
    assert!(receipt.input_digest.starts_with("blake3:"));
    assert!(receipt.config_digest.starts_with("blake3:"));
}

#[test]
fn receipt_is_json_serializable() {
    let input = [0.25, 0.75];
    let result = must(quantize_a2(&input, 4.0));
    let receipt = result.receipt();

    let json = must(serde_json::to_string(&receipt));
    let decoded: hyperquant::HyperQuantReceiptV1 = must(serde_json::from_str(&json));
    assert_eq!(decoded, receipt);
}

#[test]
fn receipt_is_bound_to_quantized_input_digest() {
    let first = must(quantize_z1(&[1.0, 2.0], 4.0));
    let second = must(quantize_z1(&[2.0, 1.0], 4.0));

    assert_ne!(first.receipt().input_digest, second.receipt().input_digest);
}

#[test]
fn config_digest_changes_when_kind_or_scale_changes() {
    let a = HyperQuantConfig::new(LatticeKind::Z1, 4.0).config_digest();
    let b = HyperQuantConfig::new(LatticeKind::A2, 4.0).config_digest();
    let c = HyperQuantConfig::new(LatticeKind::Z1, 8.0).config_digest();

    assert_ne!(a, b);
    assert_ne!(a, c);
}
