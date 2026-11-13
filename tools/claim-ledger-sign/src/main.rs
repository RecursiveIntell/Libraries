//! Operator-side signer for exact artifact bytes.
//!
//! Binds an exact artifact file's SHA-256 into a `claim_ledger::ArtifactEnvelopeV1`
//! and signs the envelope's identity fields with an Ed25519 key read from a
//! 32-byte seed file. The tool never accepts an already-computed digest in place
//! of the file's own bytes, never writes the signing seed, and never reads seed
//! material from a digest string.
//!
//! Trust boundary: possession of a valid seed is the signer's proof of identity;
//! acceptance of signed artifacts is a verifier-side decision keyed by the
//! signer's *public* key, which this tool also prints for configuration.

use std::{fs, path::PathBuf};

use chrono::{DateTime, Utc};
use claim_ledger::PolicyAdmission;
use clap::{Parser, ValueEnum};
use ring::signature::{Ed25519KeyPair, KeyPair};
use sha2::{Digest, Sha256};

const SEED_BYTES: usize = 32;

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
enum SignPolicy {
    /// Policy id asserted for operator-admitted support artifacts.
    OperatorAdmitted,
}

impl SignPolicy {
    fn policy_id(self) -> &'static str {
        match self {
            Self::OperatorAdmitted => "recursiveintell:operator-admitted-support:v1",
        }
    }
}

#[derive(Parser, Debug)]
#[command(
    name = "claim-ledger-sign",
    about = "Sign exact artifact bytes for verifier-side acceptance; prints the signed envelope JSON and public key.",
    after_help = "The signing seed file must be exactly 32 bytes (an Ed25519 seed). Never commit it."
)]
struct Cli {
    /// Path to the exact artifact bytes to bind and sign.
    #[arg(long)]
    artifact: PathBuf,
    /// Path to the 32-byte Ed25519 signing seed. Never copied or printed.
    #[arg(long)]
    seed: PathBuf,
    /// Trusted-time assertion stamped into the envelope identity.
    #[arg(long)]
    trusted_timestamp: String,
    /// Stable signer identity recorded in the envelope. Must be authorized
    /// (with the matching public key) on the verifier side.
    #[arg(long, default_value = "local-operator")]
    signer_id: String,
    /// Policy asserted by this signature.
    #[arg(long, value_enum)]
    policy: SignPolicy,
    /// Optional destination for the signed envelope JSON (pretty printed).
    #[arg(long)]
    out: Option<PathBuf>,
}

fn main() -> Result<(), String> {
    let cli = Cli::parse();
    let artifact = fs::read(&cli.artifact)
        .map_err(|error| format!("artifact read failed {}: {error}", cli.artifact.display()))?;
    let seed = fs::read(&cli.seed)
        .map_err(|error| format!("seed read failed {}: {error}", cli.seed.display()))?;
    let mut seed = seed;
    // Tolerate one trailing newline from text editors; anything else is rejected.
    if seed.len() == SEED_BYTES + 1 && seed[SEED_BYTES] == b'\n' {
        seed.pop();
    }
    if seed.len() != SEED_BYTES {
        return Err(format!(
            "signing seed must be exactly {SEED_BYTES} bytes, got {}",
            seed.len()
        ));
    }
    let trusted_timestamp = DateTime::parse_from_rfc3339(&cli.trusted_timestamp)
        .map_err(|error| format!("trusted timestamp is not RFC 3339: {error}"))?
        .with_timezone(&Utc);

    let seed_array: [u8; SEED_BYTES] = seed.as_slice().try_into().map_err(|_| {
        "internal error: 32-byte slice could not seed an Ed25519 key pair".to_string()
    })?;
    let signing_key = Ed25519KeyPair::from_seed_unchecked(&seed_array)
        .map_err(|error| format!("invalid Ed25519 seed: {error}"))?;

    let envelope = claim_ledger::envelope::ArtifactEnvelopeV1::unsigned(
        &artifact,
        cli.signer_id.clone(),
        trusted_timestamp,
        PolicyAdmission::admitted(cli.policy.policy_id()),
    );
    let mut envelope = envelope;
    claim_ledger::envelope::ArtifactEnvelopeV1::sign_ed25519(&mut envelope, &seed_array)
        .map_err(|error| format!("signing failed: {error}"))?;
    drop(seed);

    let artifact_digest = {
        let mut hasher = Sha256::new();
        hasher.update(b"recursiveintell:artifact-envelope:digest:v1\0");
        hasher.update((artifact.len() as u64).to_be_bytes());
        hasher.update(&artifact);
        hex::encode(hasher.finalize())
    };
    if artifact_digest != envelope.artifact_digest {
        // Reachable only if the library's digest rule changes without this
        // tool's constant being updated; refuse rather than sign on a stale
        // binding formula.
        return Err("artifact digest binding mismatch".to_string());
    }

    let public_key_hex = hex::encode(signing_key.public_key().as_ref());

    let signed_json = serde_json::to_string_pretty(&envelope)
        .map_err(|error| format!("envelope serialization failed: {error}"))?;
    if let Some(out_path) = &cli.out {
        fs::write(out_path, format!("{signed_json}\n"))
            .map_err(|error| format!("envelope write failed {}: {error}", out_path.display()))?;
    }
    let report = serde_json::json!({
        "schema": "ClaimLedgerOperatorSignatureV1",
        "artifact_path": cli.artifact.display().to_string(),
        "artifact_bytes": artifact.len(),
        "artifact_digest": envelope.artifact_digest,
        "signer_id": cli.signer_id,
        "policy_id": cli.policy.policy_id(),
        "trusted_timestamp": cli.trusted_timestamp,
        "signer_public_key": public_key_hex,
        "envelope": envelope,
    });
    println!(
        "{}",
        serde_json::to_string_pretty(&report)
            .map_err(|error| format!("report serialization failed: {error}"))?
    );
    if let Some(out_path) = &cli.out {
        eprintln!(
            "signed envelope for {} bytes written to {}",
            artifact.len(),
            out_path.display()
        );
    }
    Ok(())
}

/// Re-exported so the fixture test module can exercise the shared digest rule
/// against the library's own implementation.
#[doc(hidden)]
pub fn artifact_digest_matches_library(artifact: &[u8], digest: &str) -> bool {
    digest
        == claim_ledger::envelope::ArtifactEnvelopeV1::unsigned(
            artifact,
            "digest-check",
            chrono::DateTime::<Utc>::default(),
            PolicyAdmission::rejected("digest-check"),
        )
        .artifact_digest
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seed_length_constant_is_the_ed25519_seed_size() {
        // The 32-byte guard is enforced in main(); this pins the constant that
        // drives both the length check and the optional trailing-newline case.
        assert_eq!(SEED_BYTES, 32);
    }
}

#[cfg(test)]
mod binding_tests {
    use chrono::DateTime;
    use claim_ledger::{
        envelope::ArtifactEnvelopeV1, EnvelopeVerificationContext, EnvelopeVerificationStatus,
        PolicyAdmission,
    };

    fn test_envelope() -> ArtifactEnvelopeV1 {
        let seed = [7u8; 32];
        let mut envelope = ArtifactEnvelopeV1::unsigned(
            b"deterministic artifact bytes",
            "test-signer",
            DateTime::from_timestamp(1_800_000_000, 0).unwrap(),
            PolicyAdmission::admitted("recursiveintell:operator-admitted-support:v1"),
        );
        envelope.sign_ed25519(&seed).unwrap();
        envelope
    }

    #[test]
    fn signed_envelope_fully_verifies_against_trust_root() {
        let envelope = test_envelope();
        let context = EnvelopeVerificationContext::new(
            DateTime::from_timestamp(1_799_999_999, 0).unwrap(),
            DateTime::from_timestamp(1_800_000_001, 0).unwrap(),
        )
        .with_signer_key("test-signer", envelope.signer_public_key().unwrap())
        .authorize_signer("test-signer")
        .admit_policy("recursiveintell:operator-admitted-support:v1");
        let report = envelope.verify(b"deterministic artifact bytes", &context);
        assert_eq!(report.status, EnvelopeVerificationStatus::FullyVerified);
        assert!(report.policy_admitted);
    }
}
