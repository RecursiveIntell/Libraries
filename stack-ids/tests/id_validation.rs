use proptest::prelude::*;
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

proptest! {
    #[test]
    fn whitespace_only_ids_are_always_rejected(
        characters in prop::collection::vec(
            prop_oneof![Just(' '), Just('\t'), Just('\n'), Just('\r')],
            0..32,
        )
    ) {
        let candidate: String = characters.into_iter().collect();
        prop_assert!(candidate.parse::<ClaimId>().is_err());
    }

    #[test]
    fn cross_family_substitution_is_always_rejected(
        payload in "[A-Za-z0-9][A-Za-z0-9._~-]{0,32}"
    ) {
        let trial = format!("trial:{}", payload);
        let claim = format!("claim:{}", payload);
        prop_assert!(trial.parse::<ClaimId>().is_err());
        prop_assert!(claim.parse::<TrialId>().is_err());
    }
}
