use chrono::{Duration, TimeZone, Utc};
use claim_ledger::{
    ArtifactEnvelopeV1, EnvelopeVerificationContext, EnvelopeVerificationStatus, PolicyAdmission,
};

#[test]
fn valid_digest_without_signature_is_digest_valid_only() {
    let now = Utc.timestamp_opt(1_000, 123).unwrap();
    let envelope = ArtifactEnvelopeV1::unsigned(
        b"artifact",
        "signer-a",
        now,
        PolicyAdmission::admitted("policy-a"),
    );
    let context =
        EnvelopeVerificationContext::new(now - Duration::seconds(1), now + Duration::seconds(1))
            .admit_policy("policy-a");

    let report = envelope.verify(b"artifact", &context);
    assert!(report.digest_valid);
    assert!(!report.signature_valid);
    assert_eq!(report.status, EnvelopeVerificationStatus::DigestValidOnly);
}

#[test]
fn valid_signature_from_unauthorized_signer_is_not_authorized() {
    let now = Utc.timestamp_opt(1_000, 456).unwrap();
    let signing_key = [7_u8; 32];
    let mut envelope = ArtifactEnvelopeV1::unsigned(
        b"artifact",
        "signer-a",
        now,
        PolicyAdmission::admitted("policy-a"),
    );
    envelope.sign_ed25519(&signing_key).unwrap();

    let context =
        EnvelopeVerificationContext::new(now - Duration::seconds(1), now + Duration::seconds(1))
            .with_signer_key("signer-a", envelope.signer_public_key().unwrap())
            .admit_policy("policy-a");
    let report = envelope.verify(b"artifact", &context);

    assert!(report.digest_valid);
    assert!(report.signature_valid);
    assert!(!report.signer_authorized);
    assert_eq!(
        report.status,
        EnvelopeVerificationStatus::SignatureValidSignerUnauthorized
    );
}
