//! Durable operator trust-root provisioning (decision 1a, 2026-09-30).
//!
//! Loads `~/.ares/trust/claim-ledger-trust-root.json` and builds an
//! [`EnvelopeVerificationContext`] from it. Fails closed on: missing root,
//! wrong file mode (the file must be 0600 on unix), malformed JSON, a
//! non-64-hex-char key, a key that is all zeros, or an inverted time window.
//!
//! The loader reads ONLY the root file (public data). It never reads any
//! `*.key` seed file; seed custody stays operator-side per decision 1a.

use std::fs;
use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use serde::Deserialize;

use crate::{EnvelopeVerificationContext, EnvelopeVerificationStatus};

/// Default location of the trust-root file.
pub fn default_trust_root_path() -> PathBuf {
    if let Some(home) = std::env::var_os("HOME") {
        return Path::new(&home)
            .join(".ares")
            .join("trust")
            .join("claim-ledger-trust-root.json");
    }
    PathBuf::from("claim-ledger-trust-root.json")
}

/// Typed failure modes of trust-root provisioning. The verifier never
/// guesses: any of these aborts with a typed error instead of a default.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TrustRootError {
    /// The configured root path does not exist.
    Missing(PathBuf),
    /// The file exists but is readable/writable beyond owner-only (unix).
    InsecurePermissions(PathBuf, u32),
    /// JSON malformed or a required field absent/empty.
    Malformed(String),
    /// The key is not 32 bytes of plausible Ed25519 material.
    InvalidKey(String),
    /// Time window inverted or zero-length.
    InvalidWindow,
    /// The signer id in `authorized_signers` has no key registered.
    UnkeyedSigner(String),
}

impl std::fmt::Display for TrustRootError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Missing(p) => write!(f, "trust root missing: {}", p.display()),
            Self::InsecurePermissions(p, mode) => {
                write!(
                    f,
                    "trust root {} must be mode 0600, found {:o}",
                    p.display(),
                    mode
                )
            }
            Self::Malformed(msg) => write!(f, "trust root malformed: {msg}"),
            Self::InvalidKey(msg) => write!(f, "trust root key invalid: {msg}"),
            Self::InvalidWindow => write!(f, "trust root time window invalid"),
            Self::UnkeyedSigner(id) => {
                write!(f, "authorized signer '{id}' has no registered key")
            }
        }
    }
}

impl std::error::Error for TrustRootError {}

#[derive(Debug, Deserialize)]
struct TrustRootFile {
    signer_id: String,
    ed25519_public_key_hex: String,
    authorized_signers: Vec<String>,
    admitted_policies: Vec<String>,
    not_before: DateTime<Utc>,
    not_after: DateTime<Utc>,
    /// Registry contract marker; must be the published schema id when present.
    #[serde(default)]
    schema: Option<String>,
}

/// Provisioned view of a trust root: the context plus the parsed file for
/// callers that want to record which root verified what.
#[derive(Debug, Clone)]
pub struct ProvisionedTrustRoot {
    pub context: EnvelopeVerificationContext,
    pub signer_id: String,
}

impl ProvisionedTrustRoot {
    /// Verify `artifact` bytes against `envelope` using this root.
    pub fn verify(
        &self,
        artifact: &[u8],
        envelope: &crate::ArtifactEnvelopeV1,
    ) -> EnvelopeVerificationStatus {
        envelope.verify(artifact, &self.context).status
    }
}

fn hex_to_key(s: &str) -> Result<[u8; 32], TrustRootError> {
    let trimmed = s.trim();
    if trimmed.len() != 64 {
        return Err(TrustRootError::InvalidKey(format!(
            "expected 64 hex chars, got {}",
            trimmed.len()
        )));
    }
    let bytes =
        hex::decode(trimmed).map_err(|e| TrustRootError::InvalidKey(format!("not hex: {e}")))?;
    let key = <[u8; 32]>::try_from(bytes.as_slice())
        .map_err(|_| TrustRootError::InvalidKey("wrong length after decode".into()))?;
    if key == [0u8; 32] {
        return Err(TrustRootError::InvalidKey("all-zero key".into()));
    }
    Ok(key)
}

fn require_mode_0600(path: &Path) -> Result<(), TrustRootError> {
    let meta = fs::metadata(path).map_err(|e| match e.kind() {
        std::io::ErrorKind::NotFound => TrustRootError::Missing(path.to_path_buf()),
        _ => TrustRootError::Missing(p2(path)),
    })?;
    let mode = meta_mode(&meta);
    if cfg!(unix) && mode & 0o077 != 0 {
        return Err(TrustRootError::InsecurePermissions(p2(path), mode & 0o777));
    }
    Ok(())
}

fn p2(p: &Path) -> PathBuf {
    p.to_path_buf()
}

#[cfg(unix)]
fn meta_mode(metadata: &std::fs::Metadata) -> u32 {
    use std::os::unix::fs::PermissionsExt;
    metadata.permissions().mode()
}

#[cfg(not(unix))]
fn meta_mode(_metadata: &std::fs::Metadata) -> u32 {
    0o600
}

/// Loads and validates the trust-root file, returning a ready
/// [`ProvisionedTrustRoot`] (fail-closed on any defect).
pub fn load_trust_root(path: &Path) -> Result<ProvisionedTrustRoot, TrustRootError> {
    require_mode_0600(path)?;
    let text = fs::read_to_string(path).map_err(|e| match e.kind() {
        std::io::ErrorKind::NotFound => TrustRootError::Missing(p2(path)),
        _ => TrustRootError::Malformed(format!("read error: {e}")),
    })?;
    load_trust_root_from_str(&text)
}

/// Parse + validate trust-root JSON text (shared by the file loader).
pub fn load_trust_root_from_str(text: &str) -> Result<ProvisionedTrustRoot, TrustRootError> {
    let file: TrustRootFile =
        serde_json::from_str(text).map_err(|e| TrustRootError::Malformed(e.to_string()))?;

    let key = hex_to_key(&file.ed25519_public_key_hex)?;
    if file.not_before >= file.not_after {
        return Err(TrustRootError::InvalidWindow);
    }
    if let Some(schema) = &file.schema {
        if schema != "ClaimLedgerTrustRootV1" {
            return Err(TrustRootError::Malformed(format!(
                "unsupported trust-root schema: {schema}"
            )));
        }
    }
    if file.authorized_signers.is_empty() {
        return Err(TrustRootError::Malformed(
            "authorized_signers is empty".into(),
        ));
    }
    if file.admitted_policies.is_empty() {
        return Err(TrustRootError::Malformed(
            "admitted_policies is empty".into(),
        ));
    }

    let mut context = EnvelopeVerificationContext::new(file.not_before, file.not_after)
        .with_signer_key(&file.signer_id, key)
        .authorize_signer(&file.signer_id);
    for policy in &file.admitted_policies {
        context = context.admit_policy(policy);
    }
    for signer in &file.authorized_signers {
        if signer != &file.signer_id {
            return Err(TrustRootError::UnkeyedSigner(signer.clone()));
        }
    }

    Ok(ProvisionedTrustRoot {
        context,
        signer_id: file.signer_id,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use ring::signature::{Ed25519KeyPair, KeyPair};

    fn root_text() -> String {
        // Deterministic disposable test key (NEVER an operator key).
        let schema = r#"{"schema":"ClaimLedgerTrustRootV1","#;
        let signer = r#" "signer_id":"fixture-operator","#;
        let key = r#" "ed25519_public_key_hex":"6a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a01","#;
        let auth = r#" "authorized_signers":["fixture-operator"],"#;
        let policies = r#" "admitted_policies":["policy-fixture"],"#;
        let nb = r#" "not_before":"2026-01-01T00:00:00Z","#;
        let na = r#" "not_after":"2027-01-01T00:00:00Z"}"#;
        format!("{schema}{signer}{key}{auth}{policies}{nb}{na}")
    }

    #[test]
    fn loads_root_and_builds_context() {
        let root = load_trust_root_from_str(&root_text()).expect("parses");
        assert_eq!(root.signer_id, "fixture-operator");
    }

    #[test]
    fn rejects_inverted_window() {
        let text = root_text().replace("2026-01-01T00:00:00Z", "2027-06-01T00:00:00Z");
        assert!(matches!(
            load_trust_root_from_str(&text),
            Err(TrustRootError::InvalidWindow)
        ));
    }

    #[test]
    fn rejects_unkeyed_extra_signer() {
        let text = root_text().replace(
            "\"authorized_signers\":[\"fixture-operator\"]",
            "\"authorized_signers\":[\"fixture-operator\",\"rogue\"]",
        );
        assert!(matches!(
            load_trust_root_from_str(&text),
            Err(TrustRootError::UnkeyedSigner(_))
        ));
    }

    #[test]
    fn rejects_short_key() {
        let text = root_text().replace(
            "6a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a01",
            "abcd",
        );
        assert!(matches!(
            load_trust_root_from_str(&text),
            Err(TrustRootError::InvalidKey(_))
        ));
    }

    #[test]
    fn rejects_all_zero_key() {
        let text = root_text().replace(
            "6a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a01",
            "0000000000000000000000000000000000000000000000000000000000000000",
        );
        assert!(matches!(
            load_trust_root_from_str(&text),
            Err(TrustRootError::InvalidKey(_))
        ));
    }

    #[test]
    fn round_trips_the_provisioned_root() {
        // Round-trip with a DISPOSABLE Ed25519 keypair whose public half is
        // registered in the root text (replacing the placeholder hex), so the
        // cryptographic lineage is exercised exactly as the operator flow is.
        let signing_key =
            Ed25519KeyPair::from_seed_unchecked(&[7u8; 32]).expect("deterministic fixture keypair");
        let fixture_pub_hex = hex::encode(signing_key.public_key().as_ref());
        let text = root_text().replace(
            "6a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a01",
            &fixture_pub_hex,
        );
        let root = load_trust_root_from_str(&text).expect("parses");
        let mut envelope = crate::ArtifactEnvelopeV1::unsigned(
            b"fixture-artifact-bytes",
            "fixture-operator",
            "2026-06-01T00:00:00Z".parse().unwrap(),
            crate::PolicyAdmission::admitted("policy-fixture"),
        );
        envelope.sign_ed25519(&[7u8; 32]).unwrap();
        let status = root.verify(b"fixture-artifact-bytes", &envelope);
        assert!(
            matches!(status, EnvelopeVerificationStatus::FullyVerified),
            "round-trip status: {status:?}"
        );
    }
}
