#![allow(clippy::expect_used)]

use attestation_exchange::{
    AttestationEnvelopeV1, AttestationReplayabilityClassV1, AttestationValidationError,
    TransparencyAdmissibilityJudgmentV1, TransparencyReceiptV1, TrustRootExpirationPolicyV1,
    TrustRootRotationPolicyV1, TrustRootSetV1, VendorCertificationAdapterV1,
    VendorEvidenceTranslationV1, VendorTranslationModeV1,
};
use stack_ids::{
    AttestationEnvelopeId, ContentDigest, DisclosurePolicyId, TransparencyReceiptId, TrustRootSetId,
};

#[test]
fn vendor_adapter_requires_covered_artifact_families() {
    let err = VendorCertificationAdapterV1::new(
        "vca_001",
        "Acme Cloud Audit",
        "managed-kms",
        Vec::new(),
        VendorTranslationModeV1::AttestedAndLossinessDeclared,
        "2026-03/2027-03",
    )
    .expect_err("adapter without covered families must fail");
    assert_eq!(
        err,
        AttestationValidationError::MissingField("covered_artifact_families")
    );
}

#[test]
fn vendor_translation_requires_required_caveats() {
    let err = VendorEvidenceTranslationV1::new(
        "vet_001",
        "vca_001",
        vec!["pdf_audit_letter".to_string()],
        vec!["CertificationBundleV1".to_string()],
        vec!["vendor_internal_case_ids".to_string()],
        Vec::new(),
    )
    .expect_err("translation without caveats must fail");
    assert_eq!(
        err,
        AttestationValidationError::MissingField("required_caveats")
    );
}

#[test]
fn attestation_top_level_artifacts_validate_when_well_formed() {
    let envelope = AttestationEnvelopeV1::new(
        AttestationEnvelopeId::new("attn_001"),
        "CertificationBundleV1",
        "v1",
        ContentDigest::compute_str("digest_001"),
        "schema:certification-bundle-v1",
        "signer:vendor-auditor",
        "2026-03-15T12:00:00Z",
        TrustRootSetId::new("trs_001"),
        "vendor translated attestation",
        DisclosurePolicyId::new("dp_001"),
        None,
        AttestationReplayabilityClassV1::Replayable,
        Vec::new(),
        Vec::new(),
    )
    .expect("attestation envelope");
    envelope.validate().expect("attestation envelope valid");

    let trust_roots = TrustRootSetV1::new(
        TrustRootSetId::new("trs_001"),
        vec!["trust-root:acme-2026".to_string()],
        vec!["vendor_auditor".to_string()],
        TrustRootExpirationPolicyV1::Days365,
        TrustRootRotationPolicyV1::PublishedKeyset,
        vec!["AttestationEnvelopeV1".to_string()],
        vec!["signed_revocation_feed".to_string()],
        vec!["team:policy".to_string()],
    )
    .expect("trust root set");
    trust_roots.validate().expect("trust root set valid");

    let receipt = TransparencyReceiptV1::new(
        TransparencyReceiptId::new("tr_001"),
        AttestationEnvelopeId::new("attn_001"),
        "registry:transparency-log",
        "merkle-proof:001",
        "2026-03-15T12:05:00Z",
        TransparencyAdmissibilityJudgmentV1::Admitted,
    )
    .expect("transparency receipt");
    receipt.validate().expect("transparency receipt valid");
}
