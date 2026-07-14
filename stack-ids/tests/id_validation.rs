use stack_ids::{ClaimId, TrialId};

#[test]
fn empty_whitespace_and_cross_family_ids_fail_parsing() {
    for invalid in ["", " ", "\t", "\n", "   "] {
        assert!(invalid.parse::<ClaimId>().is_err());
    }
    assert!("trial:018f6f4e".parse::<ClaimId>().is_err());
    assert!("claim:018f6f4e".parse::<TrialId>().is_err());
}

#[test]
fn only_canonical_family_qualified_ids_parse() {
    assert!("claim:018f6f4e".parse::<ClaimId>().is_ok());
    for invalid in [
        "Claim:018f6f4e",
        "claim:",
        "claim:has space",
        "claim:leading/slash",
        "claim:é",
    ] {
        assert!(invalid.parse::<ClaimId>().is_err(), "{invalid:?}");
    }
}

#[test]
fn generated_ids_are_family_qualified_and_round_trip() {
    let generated = ClaimId::generate();
    assert!(generated.as_str().starts_with("claim:"));
    assert_eq!(generated.as_str().parse::<ClaimId>().unwrap(), generated);
}
