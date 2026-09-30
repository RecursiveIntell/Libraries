//! Native V2 admission events, envelope-gated admission, and the canonical
//! snapshot fold used by V2 (FINISH_PLAN Phase 3.2).
//!
//! Wire contract (pinned, cross-language with ClaimLedger Python PR #19):
//! - artifact digest: `ARTIFACT_DIGEST_DOMAIN` + u64BE length + bytes.
//! - signature preimage: `SIGNATURE_DOMAIN` + u64BE-len-prefixed fields
//!   (envelope digest, signer id, i64 nanos as exactly 8 BE bytes, policy id,
//!   admitted byte).
//!
//! Trust boundary: an admission event records a verified admission of the
//! release-artifact profile (3a). The event carries the trusted digest fields
//! and the stage the envelope reached; it is deliberately unprojectable by
//! snapshot v1. The dedicated V2 fold projects admitted support only from
//! `FullyVerified` envelopes and rejects duplicate native admission IDs.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::envelope::{
    ArtifactEnvelopeV1, EnvelopeVerificationContext, EnvelopeVerificationStatus,
};
use crate::error::ClaimLedgerError;
use crate::ids::sha256_bytes;
use crate::ledger::{LedgerEntry, LedgerEvent, LedgerSnapshot};
use crate::types::SupportState;

/// Version tag for the native admission record.
pub const NATIVE_ADMISSION_SCHEMA: &str = "claim-ledger.native-admission.v1";

/// A native V2 admission: the verifiable, trust-gated record of admitting one
/// release-artifact envelope (profile 3a) into a claim-ledger store.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NativeAdmission {
    /// Distinct native admission identifier (`nadm_...`), content-derived.
    pub admission_id: String,
    /// Domain-separated digest of the admitted artifact bytes.
    pub envelope_digest: String,
    /// Signer identity from the envelope.
    pub signer_id: String,
    /// Signature epoch in nanoseconds (the exact preimage field).
    pub trusted_timestamp_ns: i64,
    /// Admitted policy id verbatim from the envelope.
    pub policy_id: String,
    /// The admitted support state this admission projects.
    pub admitted_support_state: SupportState,
}

impl NativeAdmission {
    /// Builds a native admission from a verified envelope.
    ///
    /// # Panics
    /// Panics only when a SHA-256 output fails a 32-byte conversion (impossible).
    #[allow(clippy::unwrap_used)]
    pub fn from_envelope(envelope: &ArtifactEnvelopeV1) -> Self {
        let trusted_timestamp_ns = envelope
            .trusted_timestamp
            .timestamp_nanos_opt()
            .unwrap_or(0);
        Self {
            admission_id: native_admission_id(
                &envelope.signer_id,
                &envelope.artifact_digest,
                trusted_timestamp_ns,
                &envelope.policy_admission.policy_id,
            ),
            envelope_digest: envelope.artifact_digest.clone(),
            signer_id: envelope.signer_id.clone(),
            trusted_timestamp_ns,
            policy_id: envelope.policy_admission.policy_id.clone(),
            admitted_support_state: SupportState::Supported,
        }
    }
}

/// Builds the distinct native admission ID.
///
/// Domain: `claim-ledger.native-admission-id.v1` + u64BE-len-prefixed parts
/// (signer id, envelope digest, i64 BE nanos, policy id) -> `nadm_<hex>`.
pub fn native_admission_id(
    signer_id: &str,
    envelope_digest: &str,
    timestamp_ns: i64,
    policy_id: &str,
) -> String {
    let mut preimage = b"claim-ledger.native-admission-id.v1".to_vec();
    for part in [
        signer_id.as_bytes(),
        envelope_digest.as_bytes(),
        timestamp_ns.to_be_bytes().as_slice(),
        policy_id.as_bytes(),
    ] {
        preimage.extend_from_slice(&(part.len() as u64).to_be_bytes());
        preimage.extend_from_slice(part);
    }
    format!("nadm_{}", hex::encode(&sha256_bytes(&preimage)[..15]))
}

/// Pinned signature preimage over trust-relevant admission fields. Identical
/// layout to the envelope signature preimage (and to the P3.1 Python
/// implementation): `SIGNATURE_DOMAIN` + u64BE-len-prefixed fields; i64 nanos
/// are exactly 8 BE bytes.
pub fn admission_signature_preimage(
    envelope_digest: &str,
    signer_id: &str,
    timestamp_ns: i64,
    policy_id: &str,
    admitted: bool,
) -> Result<Vec<u8>, ClaimLedgerError> {
    let mut preimage = crate::envelope::signature_signing_domain().to_vec();
    for field in [
        envelope_digest.as_bytes(),
        signer_id.as_bytes(),
        timestamp_ns.to_be_bytes().as_slice(),
        policy_id.as_bytes(),
        &[u8::from(admitted)],
    ] {
        preimage.extend_from_slice(
            &(u64::try_from(field.len()).map_err(|_| {
                ClaimLedgerError::SerializationError("preimage field exceeds u64".into())
            })?)
            .to_be_bytes(),
        );
        preimage.extend_from_slice(field);
    }
    Ok(preimage)
}

/// Outcome of admission verification per stage.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AdmissionDecision {
    /// Fully verified: project the admitted support state.
    ProjectSupport(SupportState),
    /// A verification stage failed; the admission must be quarantined with a
    /// stable stage reason (digest | signature | unauthorized | time | policy).
    Quarantined(String),
}

/// Verify a native admission through the full staged envelope ladder.
///
/// Steps: (1) optionally recompute the artifact digest over the caller's
/// artifact bytes; (2) reconstruct the signed envelope identity fields from
/// the admission record; (3) run every stage in order and map the highest
/// reached stage to a decision. Only `FullyVerified` may project support.
///
/// # Arguments
/// * `envelope` - The signed artifact envelope carrying the admission.
/// * `artifact` - Optional exact artifact bytes for digest re-verification.
/// * `context`  - Verifier trust root, authorization, window, and policies.
pub fn verify_admission(
    envelope: &ArtifactEnvelopeV1,
    artifact: Option<&[u8]>,
    context: &EnvelopeVerificationContext,
) -> Result<AdmissionDecision, ClaimLedgerError> {
    let artifact_owned;
    let artifact_bytes: &[u8] = match artifact {
        Some(bytes) => bytes,
        None => {
            // No artifact supplied: verify against a length-0 buffer only if
            // the envelope's digest matches that of the empty byte string;
            // otherwise the digest stage fails without any artifact claim.
            artifact_owned = Vec::new();
            artifact_owned.as_slice()
        }
    };
    if crate::envelope::public_artifact_digest(artifact_bytes) != envelope.artifact_digest {
        return Ok(AdmissionDecision::Quarantined("digest".into()));
    }
    let report = envelope.verify(artifact_bytes, context);
    match report.status {
        EnvelopeVerificationStatus::FullyVerified => {
            Ok(AdmissionDecision::ProjectSupport(SupportState::Supported))
        }
        EnvelopeVerificationStatus::DigestValidOnly => {
            Ok(AdmissionDecision::Quarantined("signature".into()))
        }
        EnvelopeVerificationStatus::SignatureInvalid => {
            Ok(AdmissionDecision::Quarantined("signature".into()))
        }
        EnvelopeVerificationStatus::SignatureValidSignerUnauthorized => {
            Ok(AdmissionDecision::Quarantined("unauthorized".into()))
        }
        EnvelopeVerificationStatus::SignerAuthorizedTimeInvalid => {
            Ok(AdmissionDecision::Quarantined("time".into()))
        }
        EnvelopeVerificationStatus::TimeValidPolicyRejected => {
            Ok(AdmissionDecision::Quarantined("policy".into()))
        }
        EnvelopeVerificationStatus::DigestInvalid => {
            Ok(AdmissionDecision::Quarantined("digest".into()))
        }
    }
}

/// Wire fields of the ledger admission event (variant `admission_event`).
///
/// Distinct native IDs: the admission id is content-derived and fold-rejected
/// when replayed. The event is deliberately unprojectable by snapshot v1.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdmissionEventPayloadV1 {
    /// Distinct native admission identifier.
    pub admission_id: String,
    /// Envelope (artifact) digest admitted.
    pub envelope_digest: String,
    /// Signer identity from the verified envelope.
    pub signer_id: String,
    /// Signature epoch (nanoseconds) from the verified envelope.
    pub trusted_timestamp_ns: i64,
    /// Admitted policy id.
    pub policy_id: String,
    /// Highest verification stage reached before admission (`fully_verified`).
    pub envelope_verification: EnvelopeVerificationStatus,
    /// Support state projected by this admission.
    pub admitted_support_state: SupportState,
}

impl AdmissionEventPayloadV1 {
    /// Stable wire name of the recorded verification stage.
    pub fn envelope_verification_name(&self) -> &'static str {
        match self.envelope_verification {
            EnvelopeVerificationStatus::DigestInvalid => "digest_invalid",
            EnvelopeVerificationStatus::DigestValidOnly => "digest_valid_only",
            EnvelopeVerificationStatus::SignatureInvalid => "signature_invalid",
            EnvelopeVerificationStatus::SignatureValidSignerUnauthorized => {
                "signature_valid_signer_unauthorized"
            }
            EnvelopeVerificationStatus::SignerAuthorizedTimeInvalid => {
                "signer_authorized_time_invalid"
            }
            EnvelopeVerificationStatus::TimeValidPolicyRejected => "time_valid_policy_rejected",
            EnvelopeVerificationStatus::FullyVerified => "fully_verified",
        }
    }
}

/// Builds an admission-event payload from a fully verified envelope.
///
/// `artifact_bytes` must be the exact admitted artifact bytes; their
/// recomputed digest must match the envelope's binding. Returns `Err` when
/// the envelope is not `FullyVerified` under `context` — admission events are
/// only minted from verified envelopes.
pub fn admission_event_from_envelope(
    envelope: &ArtifactEnvelopeV1,
    artifact_bytes: &[u8],
    context: &EnvelopeVerificationContext,
) -> Result<AdmissionEventPayloadV1, ClaimLedgerError> {
    match verify_admission(envelope, Some(artifact_bytes), context)? {
        AdmissionDecision::ProjectSupport(state) => {
            let trusted_timestamp_ns = envelope
                .trusted_timestamp
                .timestamp_nanos_opt()
                .ok_or_else(|| {
                    ClaimLedgerError::SerializationError(
                        "trusted timestamp is outside the nanosecond range".into(),
                    )
                })?;
            Ok(AdmissionEventPayloadV1 {
                admission_id: native_admission_id(
                    &envelope.signer_id,
                    &envelope.artifact_digest,
                    trusted_timestamp_ns,
                    &envelope.policy_admission.policy_id,
                ),
                envelope_digest: envelope.artifact_digest.clone(),
                signer_id: envelope.signer_id.clone(),
                trusted_timestamp_ns,
                policy_id: envelope.policy_admission.policy_id.clone(),
                envelope_verification: EnvelopeVerificationStatus::FullyVerified,
                admitted_support_state: state,
            })
        }
        AdmissionDecision::Quarantined(reason) => {
            Err(ClaimLedgerError::AdmissionVerification(reason))
        }
    }
}

/// Builds the event's canonical preimage fields for the digest preimage writer.
pub fn admission_event_preimage_fields(event: &AdmissionEventPayloadV1) -> Vec<Vec<u8>> {
    let state_name = match event.admitted_support_state {
        SupportState::Supported => "supported",
        SupportState::PartiallySupported => "partially_supported",
        SupportState::Unsupported => "unsupported",
        SupportState::Contradicted => "contradicted",
        SupportState::HeuristicOnly => "heuristic_only",
        SupportState::Unknown => "unknown",
    };
    let digest_bytes = event.trusted_timestamp_ns.to_be_bytes().to_vec();
    let stage_name = event.envelope_verification_name();
    vec![
        event.admission_id.as_bytes().to_vec(),
        event.envelope_digest.as_bytes().to_vec(),
        event.signer_id.as_bytes().to_vec(),
        digest_bytes,
        event.policy_id.as_bytes().to_vec(),
        stage_name.as_bytes().to_vec(),
        state_name.as_bytes().to_vec(),
    ]
}

/// Canonical fold for V2 admission chains.
///
/// Projects ClaimAdded / SupportJudgment / Contradiction events exactly like
/// snapshot v1 (via the shared projection), records native admission IDs
/// (duplicate IDs are rejected), and leaves the projected support unchanged by
/// admission events' stream position.
#[derive(Debug, Default)]
pub struct SnapshotFoldV1 {
    seen_admission_ids: std::collections::BTreeSet<String>,
    projection: crate::ledger::SnapshotProjection,
    last_sequence: u64,
    last_entry_digest: Option<String>,
}

impl SnapshotFoldV1 {
    /// Applies one ledger entry. Admission events are recorded (duplicate IDs
    /// are rejected); every other event is projected through the snapshot v1
    /// projector.
    pub fn apply_entry(&mut self, entry: &LedgerEntry) -> Result<(), ClaimLedgerError> {
        match &entry.event {
            LedgerEvent::AdmissionEvent { payload } => {
                if !self.seen_admission_ids.insert(payload.admission_id.clone()) {
                    return Err(ClaimLedgerError::DuplicateNativeAdmissionId(
                        payload.admission_id.clone(),
                    ));
                }
            }
            other => self.projection.apply(other),
        }
        self.last_sequence = entry.sequence;
        self.last_entry_digest = Some(entry.entry_digest.clone());
        Ok(())
    }

    /// Applies one admission event (errors if the entry is not one).
    pub fn apply_admission_entry(&mut self, entry: &LedgerEntry) -> Result<(), ClaimLedgerError> {
        if !matches!(entry.event, LedgerEvent::AdmissionEvent { .. }) {
            return Err(ClaimLedgerError::LedgerCorrupt(
                "entry is not an admission event".into(),
            ));
        }
        self.apply_entry(entry)
    }

    /// Applies a full chain in stream order.
    pub fn apply_chain(&mut self, entries: &[LedgerEntry]) -> Result<(), ClaimLedgerError> {
        for entry in entries {
            self.apply_entry(entry)?;
        }
        Ok(())
    }

    /// Finishes the fold into a verified snapshot with a fixed `created_at`.
    ///
    /// Only projectable events shape the snapshot; admission events remain
    /// visible in the stream but do not alter the projected state. The anchor
    /// is the last entry applied, so the snapshot checkpoint binds the full
    /// folded prefix.
    pub fn into_snapshot(
        self,
        created_at: DateTime<Utc>,
    ) -> Result<LedgerSnapshot, ClaimLedgerError> {
        self.projection
            .into_snapshot(self.last_sequence, self.last_entry_digest, created_at)
    }

    /// Digest of the snapshot this fold represents, with a fixed `created_at`.
    pub fn fold_digest(&self, created_at: DateTime<Utc>) -> Result<String, ClaimLedgerError> {
        self.projection
            .clone()
            .into_snapshot(
                self.last_sequence,
                self.last_entry_digest.clone(),
                created_at,
            )
            .map(|snapshot| snapshot.snapshot_digest)
    }

    /// Admission IDs recorded by previous entries.
    pub fn seen_admission_ids(&self) -> &std::collections::BTreeSet<String> {
        &self.seen_admission_ids
    }
}

/// Folds a full chain into a verified snapshot in one call.
pub fn fold_snapshot(
    entries: &[LedgerEntry],
    created_at: DateTime<Utc>,
) -> Result<LedgerSnapshot, ClaimLedgerError> {
    let mut fold = SnapshotFoldV1::default();
    fold.apply_chain(entries)?;
    fold.into_snapshot(created_at)
}
