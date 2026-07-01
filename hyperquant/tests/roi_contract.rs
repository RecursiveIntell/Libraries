use hyperquant::{
    estimate_best_rice_profile, quantize_d4, quantize_z1, rht_tile, rice_decode_i16,
    rice_encode_i16, HyperQuantConfig, HyperQuantError, LatticeKind,
};

fn must<T, E: core::fmt::Debug>(result: core::result::Result<T, E>) -> T {
    match result {
        Ok(value) => value,
        Err(err) => panic!("expected Ok(..), got {err:?}"),
    }
}

#[test]
fn d4_quantizes_even_sum_lattice_points_exactly() {
    let input = [1.0, 1.0, 2.0, 0.0, -3.0, 1.0, 1.0, 1.0];
    let result = must(quantize_d4(&input, 1.0));

    assert_eq!(result.kind, LatticeKind::D4);
    assert_eq!(result.codes.len(), input.len());
    assert_eq!(result.reconstructed.len(), input.len());
    assert!(result.mse < 1.0e-6, "d4 mse was {}", result.mse);
}

#[test]
fn d4_config_is_implemented_and_e8_remains_explicitly_unsupported() {
    let input = [0.2, 0.8, -1.1, 2.4];
    let d4 = must(HyperQuantConfig::new(LatticeKind::D4, 4.0).quantize(&input));
    let e8 = HyperQuantConfig::new(LatticeKind::E8, 4.0).quantize(&input);

    assert_eq!(d4.kind, LatticeKind::D4);
    assert_eq!(
        e8,
        Err(HyperQuantError::UnsupportedLattice(LatticeKind::E8))
    );
}

#[test]
fn d4_beats_or_matches_z1_on_d4_lattice_point() {
    let input = [0.5, 0.5, 1.0, 0.0];
    let z1 = must(quantize_z1(&input, 1.0));
    let d4 = must(quantize_d4(&input, 2.0));

    assert!(d4.mse <= z1.mse, "d4 mse {} > z1 mse {}", d4.mse, z1.mse);
}

#[test]
fn rice_roundtrips_signed_codes_and_compresses_zeros() {
    let codes = [0, 0, 0, 0, 1, -1, 2, -2, 0, 0, 0, 0];
    let profile = must(estimate_best_rice_profile(&codes, 0..=6));
    let stream = must(rice_encode_i16(&codes, profile.k));
    let decoded = must(rice_decode_i16(&stream));

    assert_eq!(decoded, codes);
    assert!(stream.encoded_bits > 0);
    assert!(stream.encoded_bytes() < codes.len() * core::mem::size_of::<i16>());
    assert_eq!(profile.encoded_bits, stream.encoded_bits);
}

#[test]
fn rice_rejects_invalid_k() {
    assert!(rice_encode_i16(&[1, 2, 3], 32).is_err());
}

#[test]
fn rht_tile_is_deterministic_and_preserves_norm() {
    let input = [0.25, -0.5, 1.0, 2.0, -1.5, 0.75, -0.25, 0.5];
    let mut a = input;
    let mut b = input;

    let receipt_a = must(rht_tile(&mut a, 4, 123));
    let receipt_b = must(rht_tile(&mut b, 4, 123));

    assert_eq!(a, b);
    assert_eq!(receipt_a, receipt_b);
    assert_eq!(receipt_a.tile_dim, 4);
    assert_eq!(receipt_a.input_len, input.len());

    let before: f32 = input.iter().map(|x| x * x).sum();
    let after: f32 = a.iter().map(|x| x * x).sum();
    assert!(
        (before - after).abs() < 1.0e-5,
        "before {before}, after {after}"
    );
}

#[test]
fn rht_rejects_non_power_of_two_tile_dim() {
    let mut input = [1.0, 2.0, 3.0];
    assert_eq!(
        rht_tile(&mut input, 3, 99),
        Err(HyperQuantError::InvalidTileDimension { tile_dim: 3 })
    );
}

#[cfg(feature = "compat")]
mod compat_contract {
    use super::must;
    use hyperquant::{HyperQuantCodec, LatticeKind};
    use quant_codec_core::{CodecProfile, VectorCodec};

    #[test]
    fn compat_adapter_exposes_profile_and_roundtrips_shape() {
        let codec = HyperQuantCodec::new(LatticeKind::A2, 8.0);
        let input = [0.125, -0.25, 0.75, 1.5];
        let encoded = must(codec.encode_block(&input));
        let mut decoded = [0.0; 4];

        must(codec.decode_block(&encoded, &mut decoded));

        assert_eq!(codec.codec_id().as_str(), "hyperquant");
        assert_eq!(codec.codec_version(), env!("CARGO_PKG_VERSION"));
        assert_eq!(codec.block_dim(), Some(2));
        assert_eq!(codec.fixed_rate_bits(), Some(16));
        assert!(codec.is_lossy());
        assert_eq!(encoded.result.input_len, input.len());
        assert_eq!(decoded.len(), input.len());
    }

    #[test]
    fn compat_profile_digest_changes_when_config_changes() {
        let a = HyperQuantCodec::new(LatticeKind::Z1, 8.0);
        let b = HyperQuantCodec::new(LatticeKind::Z1, 4.0);
        let c = HyperQuantCodec::new(LatticeKind::A2, 8.0);

        assert_ne!(a.profile_digest(), b.profile_digest());
        assert_ne!(a.profile_digest(), c.profile_digest());
    }
}
