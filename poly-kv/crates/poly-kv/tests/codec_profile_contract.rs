use poly_kv::{CodecProfile, Q8KeyCodec, RawExactValueCodec, VectorCodec};

fn assert_profile_contract<C>(codec: &C, expected_id: &str, expected_lossless: bool)
where
    C: VectorCodec + CodecProfile,
{
    let profile = codec.profile();
    assert_eq!(profile.codec_id().as_str(), expected_id);
    assert_eq!(profile.codec_version(), codec.codec_version());
    assert_eq!(profile.profile_digest(), codec.profile_digest());
    assert_eq!(codec.capabilities().is_lossless, expected_lossless);
    assert_eq!(!profile.is_lossy(), expected_lossless);
}

#[test]
fn built_in_codecs_expose_non_panicking_consistent_profiles() {
    assert_profile_contract(&RawExactValueCodec, "poly-kv:value:raw-exact", true);
    assert_profile_contract(
        &Q8KeyCodec::symmetric_per_block(),
        "poly-kv:q8-key:symmetric-per-block",
        false,
    );
}
