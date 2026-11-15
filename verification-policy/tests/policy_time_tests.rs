use verification_policy::{policy_as_of, PolicySnapshot};

#[test]
fn temporal_policy_selection_compares_instants_not_wire_spelling() {
    let later = PolicySnapshot::permissive("later-instant", "2026-11-12T02:00:00-06:00");
    let earlier = PolicySnapshot::permissive("earlier-instant", "2026-11-12T07:30:00Z");
    let policies = vec![later, earlier];
    assert_eq!(
        policy_as_of(&policies, "2026-11-12T08:30:00Z")
            .unwrap()
            .policy_version,
        "later-instant"
    );
}

#[test]
fn missing_future_expired_malformed_and_reversed_policies_do_not_apply() {
    let now = "2026-11-12T08:30:00Z";
    assert!(policy_as_of(&[], now).is_none());
    let future = PolicySnapshot::permissive("future", "2027-01-01T00:00:00Z");
    let mut expired = PolicySnapshot::permissive("expired", "2020-01-01T00:00:00Z");
    expired.effective_to = Some("2026-11-12T08:30:00Z".into());
    let malformed = PolicySnapshot::permissive("malformed", "0000-not-an-instant");
    let mut bad_end = PolicySnapshot::permissive("bad-end", "2020-01-01T00:00:00Z");
    bad_end.effective_to = Some("zz-not-an-instant".into());
    let mut reversed = PolicySnapshot::permissive("reversed", "2020-01-01T00:00:00Z");
    reversed.effective_to = Some("2019-01-01T00:00:00Z".into());
    for policy in [future, expired, malformed, bad_end, reversed] {
        assert!(
            policy_as_of(std::slice::from_ref(&policy), now).is_none(),
            "accepted {}",
            policy.policy_version
        );
    }
}

#[test]
fn invalid_snapshot_metadata_and_invalid_query_time_cannot_select_authority() {
    let mut bad = PolicySnapshot::permissive("bad-schema", "2020-01-01T00:00:00Z");
    bad.schema_version = "not-the-policy-schema".into();
    assert!(policy_as_of(&[bad], "2026-11-12T08:30:00Z").is_none());
    let valid = PolicySnapshot::permissive("valid", "2020-01-01T00:00:00Z");
    assert!(policy_as_of(&[valid], "zz-invalid").is_none());
}
