use boundary_compiler::{
    canonicalize_json_v2, canonicalize_v2, Canonicalizer, ContentDigest, VersionedDigestV2,
};
use serde_json::json;

type Result = std::result::Result<(), Box<dyn std::error::Error>>;

#[test]
fn rfc_number_vectors() -> Result {
    let cases = [
        (0x0000000000000000, "0"),
        (0x8000000000000000, "0"),
        (0x0000000000000001, "5e-324"),
        (0x8000000000000001, "-5e-324"),
        (0x7fefffffffffffff, "1.7976931348623157e+308"),
        (0xffefffffffffffff, "-1.7976931348623157e+308"),
        (0x4340000000000000, "9007199254740992"),
        (0xc340000000000000, "-9007199254740992"),
        (0x4430000000000000, "295147905179352830000"),
        (0x44b52d02c7e14af5, "9.999999999999997e+22"),
        (0x44b52d02c7e14af6, "1e+23"),
        (0x44b52d02c7e14af7, "1.0000000000000001e+23"),
        (0x444b1ae4d6e2ef4e, "999999999999999700000"),
        (0x444b1ae4d6e2ef4f, "999999999999999900000"),
        (0x444b1ae4d6e2ef50, "1e+21"),
        (0x3eb0c6f7a0b5ed8c, "9.999999999999997e-7"),
        (0x3eb0c6f7a0b5ed8d, "0.000001"),
        (0x41b3de4355555553, "333333333.3333332"),
        (0x41b3de4355555554, "333333333.33333325"),
        (0x41b3de4355555555, "333333333.3333333"),
        (0x41b3de4355555556, "333333333.3333334"),
        (0x41b3de4355555557, "333333333.33333343"),
        (0xbecbf647612f3696, "-0.0000033333333333333333"),
        (0x43143ff3c1cb0959, "1424953923781206.2"),
    ];
    for (bits, expected) in cases {
        assert_eq!(
            canonicalize_v2(&json!(f64::from_bits(bits)))?,
            expected,
            "bits={bits:x}"
        );
        assert_eq!(
            canonicalize_json_v2(expected.as_bytes())?,
            expected,
            "canonical round trip bits={bits:x}"
        );
    }
    Ok(())
}

#[test]
fn utf16_order_and_string_preservation() -> Result {
    let input = json!({"\u{fb33}":"Hebrew","\u{1f600}":"Emoji","\u{80}":"Control","\r":"CR","1":"One","ö":"O","€":"Euro"});
    assert_eq!(canonicalize_v2(&input)?, "{\"\\r\":\"CR\",\"1\":\"One\",\"\u{80}\":\"Control\",\"ö\":\"O\",\"€\":\"Euro\",\"😀\":\"Emoji\",\"דּ\":\"Hebrew\"}");
    assert_eq!(
        canonicalize_v2(&json!("\u{7f}\u{80}\u{9f}\n\u{f}"))?,
        "\"\u{7f}\u{80}\u{9f}\\n\\u000f\""
    );
    assert_ne!(
        canonicalize_v2(&json!("é"))?,
        canonicalize_v2(&json!("e\u{301}"))?
    );
    Ok(())
}

#[test]
fn strict_raw_ingress_rejects_duplicates_and_invalid_unicode() -> Result {
    for raw in [
        r#"{"a":1,"a":2}"#,
        r#"{"a":1,"\u0061":2}"#,
        r#"{"x":{"a":1,"a":2}}"#,
        r#""\udead""#,
        "1e9999",
        "NaN",
        "{} {}",
    ] {
        assert!(
            canonicalize_json_v2(raw.as_bytes()).is_err(),
            "accepted {raw}"
        );
    }
    assert_eq!(
        canonicalize_json_v2(br#"{"x":{"a":1},"y":{"a":2}}"#)?,
        r#"{"x":{"a":1},"y":{"a":2}}"#
    );
    assert_eq!(canonicalize_json_v2(b"-0.0")?, "0");
    Ok(())
}

#[test]
fn integers_do_not_silently_lose_precision() -> Result {
    for value in [
        json!(u64::MAX),
        json!(i64::MAX),
        json!(9007199254740993_u64),
    ] {
        assert!(canonicalize_v2(&value).is_err());
    }
    assert_eq!(
        canonicalize_v2(&json!(9007199254740992_u64))?,
        "9007199254740992"
    );
    Ok(())
}

#[test]
fn scheme_tag_and_digest_are_verified_without_fallback() -> Result {
    let value = json!({"a":1});
    let digest = VersionedDigestV2::compute(&value)?;
    assert!(digest.verify(&value).is_ok());
    assert!(digest.verify(&json!({"a":2})).is_err());
    let encoded = serde_json::to_value(&digest)?;
    assert_eq!(encoded["scheme"], "boundary-compiler/rfc8785-v2");
    assert_ne!(encoded["digest"], ContentDigest::compute(&value)?.hex());
    let mut changed = encoded;
    changed["scheme"] = json!("legacy-v1");
    assert!(serde_json::from_value::<VersionedDigestV2>(changed).is_err());
    Ok(())
}

#[test]
fn raw_digest_preserves_binary64_source_and_numeric_policy() -> Result {
    let raw = b"9.999999999999997e+22";
    let original = json!(f64::from_bits(0x44b52d02c7e14af5));
    assert_eq!(
        VersionedDigestV2::compute_json(raw)?,
        VersionedDigestV2::compute(&original)?
    );
    assert_eq!(
        canonicalize_json_v2(b"295147905179352825856")?,
        "295147905179352830000"
    );
    for token in [
        "9007199254740993",
        "18446744073709551617",
        "295147905179352825857",
    ] {
        assert!(canonicalize_json_v2(token.as_bytes()).is_err());
    }
    let encoded = serde_json::to_string(&VersionedDigestV2::compute_json(raw)?)?;
    let decoded: VersionedDigestV2 = serde_json::from_str(&encoded)?;
    assert!(decoded.verify_json(raw).is_ok());
    assert!(decoded.verify_json(b"1").is_err());
    Ok(())
}

#[test]
fn bounds_and_tag_fail_closed() -> Result {
    assert!(canonicalize_json_v2(&vec![b' '; (1 << 20) + 1]).is_err());
    let mut deep = json!(0);
    for _ in 0..66 {
        deep = json!([deep]);
    }
    assert!(canonicalize_v2(&deep).is_err());
    assert!(canonicalize_json_v2(&serde_json::to_vec(&deep)?).is_err());
    let good = serde_json::to_value(VersionedDigestV2::compute(&json!(1))?)?;
    for field in ["scheme", "digest"] {
        let mut missing = good.clone();
        missing
            .as_object_mut()
            .ok_or("object expected")?
            .remove(field);
        assert!(serde_json::from_value::<VersionedDigestV2>(missing).is_err());
    }
    for bad in ["", "xyz", &"A".repeat(64)] {
        let mut value = good.clone();
        value["digest"] = json!(bad);
        assert!(serde_json::from_value::<VersionedDigestV2>(value).is_err());
    }
    Ok(())
}

#[test]
fn legacy_bytes_and_digest_remain_unchanged() -> Result {
    let old = Canonicalizer::new();
    assert_eq!(old.canonicalize(&json!(-0.0))?, "-0.0");
    assert_eq!(old.canonicalize(&json!("\u{80}"))?, "\"\\u0080\"");
    let value = json!({"\u{fb33}":1,"😀":2});
    assert_eq!(old.canonicalize(&value)?, "{\"דּ\":1,\"😀\":2}");
    assert_eq!(
        ContentDigest::compute(&value)?.hex(),
        blake3::hash(old.canonicalize(&value)?.as_bytes())
            .to_hex()
            .to_string()
    );
    Ok(())
}
