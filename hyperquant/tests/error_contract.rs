use hyperquant::{quantize_a2, HyperQuantConfig, HyperQuantError, LatticeKind};

#[test]
fn empty_input_is_rejected() {
    assert_eq!(
        quantize_a2(&[], 1.0).unwrap_err(),
        HyperQuantError::EmptyInput
    );
}

#[test]
fn non_finite_input_error_message_is_explicit() {
    let err = quantize_a2(&[1.0, f32::NAN], 1.0).unwrap_err();
    assert!(err.to_string().contains("non-finite input"));
    assert!(err.to_string().contains("1"));
}

#[test]
fn unsupported_lattice_error_message_is_explicit() {
    let err = HyperQuantConfig::new(LatticeKind::E8, 1.0)
        .quantize(&[0.0; 8])
        .unwrap_err();
    assert!(err.to_string().contains("unsupported lattice"));
    assert!(err.to_string().contains("E8"));
}
