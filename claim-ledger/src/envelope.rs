//! Signed artifact envelope with staged trust verification.

use chrono::{DateTime, Utc};
use ring::signature::{Ed25519KeyPair, KeyPair, UnparsedPublicKey, ED25519};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};

const ARTIFACT_DIGEST_DOMAIN: &[u8] = b"recursiveintell:artifact-envelope:digest:v1\0";
const SIGNATURE_DOMAIN: &[u8] = b"recursiveintell:artifact-envelope:signature:v1\0";

/// Policy decision carried by an artifact envelope.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PolicyAdmission {
    /// Stable policy identifier.
    pub policy_id: String,
    /// Whether the producing policy admitted the artifact.
    pub admitted: bool,
}

impl PolicyAdmission {
    /// Creates an admitted policy decision.
    pub fn admitted(policy_id: impl Into<String>) -> Self {
        Self {
            policy_id: policy_id.into(),
            admitted: true,
        }
    }

    /// Creates a rejected policy decision.
    pub fn rejected(policy_id: impl Into<String>) -> Self {
        Self {
            policy_id: policy_id.into(),
            admitted: false,
        }
    }
}

/// Version-one envelope binding content, signer, trusted time, and policy.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactEnvelopeV1 {
    /// Domain-separated SHA-256 digest of the artifact bytes.
    pub artifact_digest: String,
    /// Ed25519 signature over the envelope identity fields, if present.
    pub signature: Option<Vec<u8>>,
    /// Stable signer identity.
    pub signer_id: String,
    /// Time asserted by the trusted-time source.
    pub trusted_timestamp: DateTime<Utc>,
    /// Policy admission asserted by the producer.
    pub policy_admission: PolicyAdmission,
    /// Ed25519 public key declared by the producer.
    pub signer_public_key: Option<[u8; 32]>,
}

impl ArtifactEnvelopeV1 {
    /// Creates an unsigned envelope whose artifact digest is already bound.
    pub fn unsigned(
        artifact: &[u8],
        signer_id: impl Into<String>,
        trusted_timestamp: DateTime<Utc>,
        policy_admission: PolicyAdmission,
    ) -> Self {
        Self {
            artifact_digest: artifact_digest(artifact),
            signature: None,
            signer_id: signer_id.into(),
            trusted_timestamp,
            policy_admission,
            signer_public_key: None,
        }
    }

    /// Signs all trust-relevant envelope identity fields with Ed25519.
    pub fn sign_ed25519(&mut self, secret_key: &[u8; 32]) -> Result<(), EnvelopeError> {
        let signing_key = Ed25519KeyPair::from_seed_unchecked(secret_key)
            .map_err(|_| EnvelopeError::InvalidSigningKey)?;
        let preimage = self.signature_preimage()?;
        self.signature = Some(signing_key.sign(&preimage).as_ref().to_vec());
        self.signer_public_key = Some(
            signing_key
                .public_key()
                .as_ref()
                .try_into()
                .map_err(|_| EnvelopeError::InvalidSigningKey)?,
        );
        Ok(())
    }

    /// Returns the producer-declared Ed25519 public key.
    pub fn signer_public_key(&self) -> Option<[u8; 32]> {
        self.signer_public_key
    }

    /// Verifies each trust stage without collapsing distinct failure states.
    pub fn verify(
        &self,
        artifact: &[u8],
        context: &EnvelopeVerificationContext,
    ) -> EnvelopeVerificationReport {
        let digest_valid = self.artifact_digest == artifact_digest(artifact);
        if !digest_valid {
            return EnvelopeVerificationReport::at(
                false,
                false,
                false,
                false,
                false,
                EnvelopeVerificationStatus::DigestInvalid,
            );
        }

        let signature_present = self.signature.is_some();
        let signature_valid = self.verify_signature(context);
        if !signature_present {
            return EnvelopeVerificationReport::at(
                true,
                false,
                false,
                false,
                false,
                EnvelopeVerificationStatus::DigestValidOnly,
            );
        }
        if !signature_valid {
            return EnvelopeVerificationReport::at(
                true,
                false,
                false,
                false,
                false,
                EnvelopeVerificationStatus::SignatureInvalid,
            );
        }

        let signer_authorized = context.authorized_signers.contains(&self.signer_id);
        if !signer_authorized {
            return EnvelopeVerificationReport::at(
                true,
                true,
                false,
                false,
                false,
                EnvelopeVerificationStatus::SignatureValidSignerUnauthorized,
            );
        }

        let time_valid = self.trusted_timestamp >= context.not_before
            && self.trusted_timestamp <= context.not_after;
        if !time_valid {
            return EnvelopeVerificationReport::at(
                true,
                true,
                true,
                false,
                false,
                EnvelopeVerificationStatus::SignerAuthorizedTimeInvalid,
            );
        }

        let policy_admitted = self.policy_admission.admitted
            && context
                .admitted_policies
                .contains(&self.policy_admission.policy_id);
        let status = if policy_admitted {
            EnvelopeVerificationStatus::FullyVerified
        } else {
            EnvelopeVerificationStatus::TimeValidPolicyRejected
        };
        EnvelopeVerificationReport::at(true, true, true, true, policy_admitted, status)
    }

    fn verify_signature(&self, context: &EnvelopeVerificationContext) -> bool {
        let Some(signature_bytes) = self.signature.as_deref() else {
            return false;
        };
        let Some(key_bytes) = context.signer_keys.get(&self.signer_id) else {
            return false;
        };
        let Ok(preimage) = self.signature_preimage() else {
            return false;
        };
        UnparsedPublicKey::new(&ED25519, key_bytes)
            .verify(&preimage, signature_bytes)
            .is_ok()
    }

    fn signature_preimage(&self) -> Result<Vec<u8>, EnvelopeError> {
        let timestamp = self
            .trusted_timestamp
            .timestamp_nanos_opt()
            .ok_or(EnvelopeError::TimestampOutOfRange)?;
        let mut preimage = SIGNATURE_DOMAIN.to_vec();
        for field in [
            self.artifact_digest.as_bytes(),
            self.signer_id.as_bytes(),
            &timestamp.to_be_bytes(),
            self.policy_admission.policy_id.as_bytes(),
            &[u8::from(self.policy_admission.admitted)],
        ] {
            preimage.extend_from_slice(&(field.len() as u64).to_be_bytes());
            preimage.extend_from_slice(field);
        }
        Ok(preimage)
    }
}

fn artifact_digest(artifact: &[u8]) -> String {
    let mut digest = Sha256::new();
    digest.update(ARTIFACT_DIGEST_DOMAIN);
    digest.update((artifact.len() as u64).to_be_bytes());
    digest.update(artifact);
    hex::encode(digest.finalize())
}

/// Verifier-owned trust roots, authorization, time window, and policy set.
#[derive(Debug, Clone)]
pub struct EnvelopeVerificationContext {
    signer_keys: BTreeMap<String, [u8; 32]>,
    authorized_signers: BTreeSet<String>,
    admitted_policies: BTreeSet<String>,
    not_before: DateTime<Utc>,
    not_after: DateTime<Utc>,
}

impl EnvelopeVerificationContext {
    /// Creates a context with an inclusive trusted-time window.
    pub fn new(not_before: DateTime<Utc>, not_after: DateTime<Utc>) -> Self {
        Self {
            signer_keys: BTreeMap::new(),
            authorized_signers: BTreeSet::new(),
            admitted_policies: BTreeSet::new(),
            not_before,
            not_after,
        }
    }

    /// Registers a verification key without authorizing the signer.
    pub fn with_signer_key(mut self, signer_id: impl Into<String>, key: [u8; 32]) -> Self {
        self.signer_keys.insert(signer_id.into(), key);
        self
    }

    /// Authorizes a registered signer identity.
    pub fn authorize_signer(mut self, signer_id: impl Into<String>) -> Self {
        self.authorized_signers.insert(signer_id.into());
        self
    }

    /// Admits an envelope policy identifier.
    pub fn admit_policy(mut self, policy_id: impl Into<String>) -> Self {
        self.admitted_policies.insert(policy_id.into());
        self
    }
}

/// Highest verification stage reached by an envelope.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EnvelopeVerificationStatus {
    /// Artifact bytes do not match the envelope digest.
    DigestInvalid,
    /// Digest matches, but no signature is present.
    DigestValidOnly,
    /// A signature is present but invalid or unverifiable.
    SignatureInvalid,
    /// Signature is valid, but signer identity lacks authorization.
    SignatureValidSignerUnauthorized,
    /// Signer is authorized, but trusted time is outside the accepted window.
    SignerAuthorizedTimeInvalid,
    /// Time is valid, but policy admission is rejected.
    TimeValidPolicyRejected,
    /// Every verification stage passed.
    FullyVerified,
}

/// Independent verification-stage results plus the highest reached status.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EnvelopeVerificationReport {
    /// Artifact digest matches.
    pub digest_valid: bool,
    /// Ed25519 signature verifies.
    pub signature_valid: bool,
    /// Signer is authorized by verifier policy.
    pub signer_authorized: bool,
    /// Trusted timestamp is inside the verifier window.
    pub time_valid: bool,
    /// Producer and verifier both admit the policy.
    pub policy_admitted: bool,
    /// Highest verification stage reached.
    pub status: EnvelopeVerificationStatus,
}

impl EnvelopeVerificationReport {
    fn at(
        digest_valid: bool,
        signature_valid: bool,
        signer_authorized: bool,
        time_valid: bool,
        policy_admitted: bool,
        status: EnvelopeVerificationStatus,
    ) -> Self {
        Self {
            digest_valid,
            signature_valid,
            signer_authorized,
            time_valid,
            policy_admitted,
            status,
        }
    }
}

/// Artifact-envelope construction errors.
#[derive(Debug, thiserror::Error)]
pub enum EnvelopeError {
    /// Timestamp cannot be represented at nanosecond precision.
    #[error("trusted timestamp is outside the nanosecond range")]
    TimestampOutOfRange,
    /// The provided Ed25519 seed could not create a signing key.
    #[error("invalid Ed25519 signing key")]
    InvalidSigningKey,
}
