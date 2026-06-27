use hyperquant::{quantize_a2, quantize_z1, HyperQuantConfig, HyperQuantError, LatticeKind};

fn must<T, E: core::fmt::Debug>(result: core::result::Result<T, E>) -> T {
    match result {
        Ok(value) => value,
        Err(err) => panic!("expected Ok(..), got {err:?}"),
    }
}

#[test]
fn z1_quantization_is_deterministic_and_reconstructs_same_length() {
    let input = [0.1, -1.4, 2.6, 32768.0];
    let a = must(quantize_z1(&input, 10.0));
    let b = must(quantize_z1(&input, 10.0));

    assert_eq!(a, b);
    assert_eq!(a.kind, LatticeKind::Z1);
    assert_eq!(a.input_len, input.len());
    assert_eq!(a.codes.len(), input.len());
    assert_eq!(a.reconstructed.len(), input.len());
    assert!(a.mse.is_finite());
    assert!(a.codes.contains(&i16::MAX));
}

#[test]
fn invalid_scale_falls_back_to_one() {
    let input = [1.25, -2.75];
    let nonpositive = must(quantize_z1(&input, 0.0));
    let nonfinite = must(quantize_z1(&input, f32::NAN));
    let explicit_one = must(quantize_z1(&input, 1.0));

    assert_eq!(nonpositive.codes, explicit_one.codes);
    assert_eq!(nonfinite.codes, explicit_one.codes);
    assert_eq!(nonpositive.effective_scale, 1.0);
    assert_eq!(nonfinite.effective_scale, 1.0);
}

#[test]
fn non_finite_inputs_are_rejected() {
    assert_eq!(
        quantize_z1(&[1.0, f32::NAN], 1.0),
        Err(HyperQuantError::NonFiniteInput { index: 1 })
    );
    assert_eq!(
        quantize_a2(&[f32::INFINITY, 0.0], 1.0),
        Err(HyperQuantError::NonFiniteInput { index: 0 })
    );
}

#[test]
fn finite_inputs_that_overflow_receipt_metrics_are_rejected() {
    assert_eq!(
        quantize_z1(&[f32::MAX], 1.0),
        Err(HyperQuantError::NonFiniteArtifact { stage: "mse" })
    );
    assert_eq!(
        quantize_a2(&[f32::MAX, f32::MAX], 1.0),
        Err(HyperQuantError::NonFiniteArtifact { stage: "mse" })
    );
}

#[test]
fn a2_quantization_handles_even_and_odd_dimensions() {
    let even = must(quantize_a2(&[0.25, 0.75, -1.25, 2.0], 8.0));
    let odd = must(quantize_a2(&[0.25, 0.75, -1.25], 8.0));

    assert_eq!(even.kind, LatticeKind::A2);
    assert_eq!(even.reconstructed.len(), 4);
    assert_eq!(odd.reconstructed.len(), 3);
    assert_eq!(odd.codes.len(), 3);
    assert!(even.mse.is_finite());
    assert!(odd.mse.is_finite());
}

#[test]
fn a2_beats_or_matches_z1_on_triangular_lattice_point() {
    let input = [0.5, 0.866_025_4];
    let z1 = must(quantize_z1(&input, 1.0));
    let a2 = must(quantize_a2(&input, 1.0));

    assert!(a2.mse <= z1.mse, "a2 mse {} > z1 mse {}", a2.mse, z1.mse);
    assert!(
        a2.mse < 1.0e-6,
        "A2 basis point should reconstruct exactly enough"
    );
}

#[test]
fn d4_and_e8_are_explicitly_unsupported_not_fake_implemented() {
    let input = [1.0, 2.0, 3.0, 4.0];
    let d4 = HyperQuantConfig::new(LatticeKind::D4, 1.0).quantize(&input);
    let e8 = HyperQuantConfig::new(LatticeKind::E8, 1.0).quantize(&input);

    assert_eq!(
        d4.unwrap_err(),
        HyperQuantError::UnsupportedLattice(LatticeKind::D4)
    );
    assert_eq!(
        e8.unwrap_err(),
        HyperQuantError::UnsupportedLattice(LatticeKind::E8)
    );
}
