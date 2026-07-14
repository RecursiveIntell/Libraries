//! Approval and permit model.

use aidens_contracts::{
    ApprovalDecisionV1, ApprovalRequestV1, ArtifactId, CanonicalToolSideEffectClass, PermitGrantV1,
    PermitUseReportV1, StackContentDigest,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

const UNKNOWN_PERMIT_SCOPE_TOKEN: &str = "unknown";
const REVOCATION_LOCK_RETRY_LIMIT: u32 = 10_000;
const REVOCATION_LOCK_RETRY_DELAY: Duration = Duration::from_millis(1);
static REVOCATION_TEMPORARY_COUNTER: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PermitDecisionV1 {
    Allow,
    Deny(String),
    RequiresApproval,
}

pub type PermitV1 = PermitGrantV1;

/// Host-loaded permit authority. The authentication key is never serialized or exposed.
#[derive(Clone)]
pub struct HostPermitAuthorityV1 {
    issuer_id: String,
    key: [u8; 32],
    revocations: PermitRevocationStoreV1,
}

impl HostPermitAuthorityV1 {
    /// Load the process host's authority. Callers cannot supply or register a trust root.
    pub fn load() -> Result<Self, std::io::Error> {
        let root = std::env::var_os("AIDENS_HOST_STATE_DIR")
            .map(PathBuf::from)
            .ok_or_else(|| std::io::Error::other("AIDENS_HOST_STATE_DIR is not configured"))?;
        let issuer_id = std::env::var("AIDENS_HOST_PERMIT_ISSUER")
            .map_err(|_| std::io::Error::other("AIDENS_HOST_PERMIT_ISSUER is not configured"))?;
        let key_path = root.join("permit-authority-v1.key");
        let key_bytes = std::fs::read(&key_path)?;
        let key: [u8; 32] = key_bytes.try_into().map_err(|_| {
            std::io::Error::other(format!(
                "{} must contain exactly 32 bytes",
                key_path.display()
            ))
        })?;
        Ok(Self {
            issuer_id,
            key,
            revocations: PermitRevocationStoreV1::open(root)?,
        })
    }

    pub fn issue_for_context(
        &self,
        context: &PermitCheckContextV1,
        granted_by: impl Into<String>,
    ) -> PermitGrantV1 {
        let mut grant = PermitGrantV1::scoped(
            context.risk_class.clone(),
            context.tool_id.clone(),
            context.sandbox_root.clone(),
            granted_by,
        );
        grant.run_id = context.run_id.clone();
        grant.attempt_id = context.attempt_id.clone();
        grant.scope = format!(
            "tool={};sandbox={};run={};attempt={}",
            context.tool_id,
            context.sandbox_root,
            context.run_id.as_ref().map_or("", |id| id.0.as_str()),
            context.attempt_id.as_ref().map_or("", |id| id.0.as_str()),
        );
        grant.issuer_id.clone_from(&self.issuer_id);
        grant.refresh_authority_id();
        grant.integrity_tag = authentication_tag(&self.key, &grant.authority_material());
        grant
    }

    pub fn issue_expiring_for_context(
        &self,
        context: &PermitCheckContextV1,
        granted_by: impl Into<String>,
        lifetime: chrono::Duration,
    ) -> Result<PermitGrantV1, std::io::Error> {
        if lifetime <= chrono::Duration::zero() {
            return Err(std::io::Error::other("permit lifetime must be positive"));
        }
        let mut grant = self.issue_for_context(context, granted_by);
        grant.expires_at = Some(grant.granted_at + lifetime);
        grant.refresh_authority_id();
        grant.integrity_tag = authentication_tag(&self.key, &grant.authority_material());
        Ok(grant)
    }

    pub fn policy(&self) -> PermitPolicyV1 {
        PermitPolicyV1::default()
            .with_host_trusted_issuer(self.issuer_id.clone(), self.key)
            .with_revocation_store(self.revocations.clone())
    }

    pub fn revoke(
        &self,
        permit_id: &ArtifactId,
        revoked_by: impl Into<String>,
        reason: impl Into<String>,
    ) -> Result<(), std::io::Error> {
        self.revocations.revoke(permit_id, revoked_by, reason)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PermitCheckContextV1 {
    pub tool_id: String,
    pub risk_class: CanonicalToolSideEffectClass,
    pub sandbox_root: String,
    pub run_id: Option<ArtifactId>,
    pub attempt_id: Option<ArtifactId>,
}

impl PermitCheckContextV1 {
    pub fn new(
        tool_id: impl Into<String>,
        risk_class: CanonicalToolSideEffectClass,
        sandbox_root: impl Into<String>,
    ) -> Self {
        Self {
            tool_id: tool_id.into(),
            risk_class,
            sandbox_root: sandbox_root.into(),
            run_id: None,
            attempt_id: None,
        }
    }

    pub fn with_run_attempt(
        mut self,
        run_id: Option<ArtifactId>,
        attempt_id: Option<ArtifactId>,
    ) -> Self {
        self.run_id = run_id;
        self.attempt_id = attempt_id;
        self
    }
}

/// Durable, append-preserving permit revocation registry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PermitRevocationStoreV1 {
    path: PathBuf,
}

struct PermitRevocationLockV1 {
    path: PathBuf,
    file: Option<File>,
}

impl Drop for PermitRevocationLockV1 {
    fn drop(&mut self) {
        drop(self.file.take());
        if let Err(error) = std::fs::remove_file(&self.path) {
            if error.kind() != std::io::ErrorKind::NotFound {
                // Drop cannot report an error. The lockfile is deliberately retained rather
                // than silently allowing a concurrent writer after a failed unlock.
                eprintln!(
                    "failed to release revocation lock {}: {error}",
                    self.path.display()
                );
            }
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
struct PermitRevocationLedgerV1 {
    #[serde(default)]
    revocations: BTreeMap<String, PermitRevocationRecordV1>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct PermitRevocationRecordV1 {
    permit_id: String,
    revoked_by: String,
    reason: String,
    recorded_at_unix_nanos: String,
}

impl PermitRevocationStoreV1 {
    pub fn open(root: impl AsRef<Path>) -> Result<Self, std::io::Error> {
        std::fs::create_dir_all(root.as_ref())?;
        Ok(Self {
            path: root.as_ref().join("permit-revocations-v1.json"),
        })
    }

    pub fn revoke(
        &self,
        permit_id: &ArtifactId,
        revoked_by: impl Into<String>,
        reason: impl Into<String>,
    ) -> Result<(), std::io::Error> {
        let _lock = self.acquire_exclusive_lock()?;
        let mut ledger = self.load()?;
        let recorded_at_unix_nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(std::io::Error::other)?
            .as_nanos()
            .to_string();
        ledger
            .revocations
            .entry(permit_id.as_str().to_string())
            .or_insert_with(|| PermitRevocationRecordV1 {
                permit_id: permit_id.as_str().to_string(),
                revoked_by: revoked_by.into(),
                reason: reason.into(),
                recorded_at_unix_nanos,
            });
        self.persist(&ledger)
    }

    fn is_revoked(&self, permit_id: &ArtifactId) -> Result<bool, std::io::Error> {
        Ok(self.load()?.revocations.contains_key(permit_id.as_str()))
    }

    fn load(&self) -> Result<PermitRevocationLedgerV1, std::io::Error> {
        match std::fs::read(&self.path) {
            Ok(bytes) => serde_json::from_slice(&bytes).map_err(std::io::Error::other),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                Ok(PermitRevocationLedgerV1::default())
            }
            Err(error) => Err(error),
        }
    }

    fn acquire_exclusive_lock(&self) -> Result<PermitRevocationLockV1, std::io::Error> {
        let lock_path = self.path.with_extension("json.lock");
        for _ in 0..REVOCATION_LOCK_RETRY_LIMIT {
            match OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&lock_path)
            {
                Ok(file) => {
                    return Ok(PermitRevocationLockV1 {
                        path: lock_path,
                        file: Some(file),
                    })
                }
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                    std::thread::sleep(REVOCATION_LOCK_RETRY_DELAY);
                }
                Err(error) => return Err(error),
            }
        }
        Err(std::io::Error::new(
            std::io::ErrorKind::TimedOut,
            format!(
                "timed out acquiring exclusive revocation lock for {}",
                self.path.display()
            ),
        ))
    }

    fn persist(&self, ledger: &PermitRevocationLedgerV1) -> Result<(), std::io::Error> {
        let bytes = serde_json::to_vec_pretty(ledger).map_err(std::io::Error::other)?;
        let (temporary, mut file) = self.create_unique_temporary_file()?;
        let temporary_for_persist = temporary.clone();
        let result = (move || {
            file.write_all(&bytes)?;
            file.sync_all()?;
            drop(file);
            std::fs::rename(&temporary_for_persist, &self.path)?;
            File::open(self.parent_dir()?)?.sync_all()
        })();

        match result {
            Ok(()) => Ok(()),
            Err(error) => match std::fs::remove_file(&temporary) {
                Ok(()) => Err(error),
                Err(cleanup_error) if cleanup_error.kind() == std::io::ErrorKind::NotFound => {
                    Err(error)
                }
                Err(cleanup_error) => Err(std::io::Error::new(
                    error.kind(),
                    format!(
                        "{error}; additionally failed to remove temporary revocation ledger {}: {cleanup_error}",
                        temporary.display()
                    ),
                )),
            },
        }
    }

    fn create_unique_temporary_file(&self) -> Result<(PathBuf, File), std::io::Error> {
        let file_name = self
            .path
            .file_name()
            .ok_or_else(|| std::io::Error::other("revocation ledger path has no file name"))?
            .to_string_lossy();
        let parent = self.parent_dir()?;
        for _ in 0..16 {
            let sequence = REVOCATION_TEMPORARY_COUNTER.fetch_add(1, Ordering::Relaxed);
            let nanos = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(std::io::Error::other)?
                .as_nanos();
            let temporary = parent.join(format!(
                ".{file_name}.{}.{}.{}.tmp",
                std::process::id(),
                nanos,
                sequence,
            ));
            match OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&temporary)
            {
                Ok(file) => return Ok((temporary, file)),
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(error) => return Err(error),
            }
        }
        Err(std::io::Error::new(
            std::io::ErrorKind::AlreadyExists,
            "could not allocate a unique temporary revocation ledger path",
        ))
    }

    fn parent_dir(&self) -> Result<&Path, std::io::Error> {
        self.path
            .parent()
            .ok_or_else(|| std::io::Error::other("revocation ledger path has no parent directory"))
    }
}

#[derive(Clone, Default, PartialEq, Eq)]
pub struct PermitPolicyV1 {
    granted_risks: BTreeSet<CanonicalToolSideEffectClass>,
    grants: Vec<PermitGrantV1>,
    trusted_issuers: BTreeMap<String, [u8; 32]>,
    revoked_permits: BTreeSet<ArtifactId>,
    revocation_store: Option<PermitRevocationStoreV1>,
}

impl std::fmt::Debug for PermitPolicyV1 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PermitPolicyV1")
            .field("granted_risks", &self.granted_risks)
            .field("grants", &self.grants)
            .field(
                "trusted_issuer_ids",
                &self.trusted_issuers.keys().collect::<Vec<_>>(),
            )
            .field("revoked_permits", &self.revoked_permits)
            .field(
                "durable_revocation_store",
                &self
                    .revocation_store
                    .as_ref()
                    .map(|store| store.path.display().to_string()),
            )
            .finish()
    }
}

impl PermitPolicyV1 {
    fn with_host_trusted_issuer(mut self, issuer_id: impl Into<String>, key: [u8; 32]) -> Self {
        self.trusted_issuers.insert(issuer_id.into(), key);
        self
    }

    fn with_revocation_store(mut self, store: PermitRevocationStoreV1) -> Self {
        self.revocation_store = Some(store);
        self
    }

    pub fn revoke(mut self, permit_id: &ArtifactId) -> Self {
        self.revoked_permits.insert(permit_id.clone());
        self
    }

    pub fn verify_grant(&self, grant: &PermitGrantV1) -> bool {
        let has_exact_context = grant.run_id.as_ref().is_some_and(|id| !id.0.is_empty())
            && grant.attempt_id.as_ref().is_some_and(|id| !id.0.is_empty());
        let durably_revoked_or_unreadable = self
            .revocation_store
            .as_ref()
            .is_some_and(|store| store.is_revoked(&grant.permit_id).unwrap_or(true));
        if !has_exact_context
            || self.revoked_permits.contains(&grant.permit_id)
            || durably_revoked_or_unreadable
            || grant.integrity_tag.is_empty()
        {
            return false;
        }
        let Some(key) = self.trusted_issuers.get(&grant.issuer_id) else {
            return false;
        };
        let mut expected_id = grant.clone();
        expected_id.refresh_authority_id();
        if expected_id.permit_id != grant.permit_id {
            return false;
        }
        authentication_tag(key, &grant.authority_material()) == grant.integrity_tag
    }
    pub fn with_permit(mut self, permit: &PermitV1) -> Self {
        self.granted_risks.insert(permit.risk_class.clone());
        self.grants.push(permit.clone());
        self
    }

    pub fn with_grant(mut self, grant: PermitGrantV1) -> Self {
        self.granted_risks.insert(grant.risk_class.clone());
        self.grants.push(grant);
        self
    }

    pub fn decision_for_risk(&self, risk: &CanonicalToolSideEffectClass) -> PermitDecisionV1 {
        if matches!(risk, CanonicalToolSideEffectClass::ReadOnly)
            || self
                .grants
                .iter()
                .any(|grant| &grant.risk_class == risk && self.verify_grant(grant))
        {
            PermitDecisionV1::Allow
        } else {
            PermitDecisionV1::RequiresApproval
        }
    }

    pub fn decision_for_context(&self, context: &PermitCheckContextV1) -> PermitDecisionV1 {
        if matches!(context.risk_class, CanonicalToolSideEffectClass::ReadOnly) {
            return PermitDecisionV1::Allow;
        }
        if self.grant_for_context(context).is_some() {
            PermitDecisionV1::Allow
        } else {
            PermitDecisionV1::RequiresApproval
        }
    }

    pub fn grant_for_risk(&self, risk: &CanonicalToolSideEffectClass) -> Option<&PermitGrantV1> {
        self.grants
            .iter()
            .find(|grant| &grant.risk_class == risk && self.verify_grant(grant))
    }

    pub fn grant_for_context(&self, context: &PermitCheckContextV1) -> Option<&PermitGrantV1> {
        self.grants.iter().find(|grant| {
            self.verify_grant(grant)
                && grant.matches_scope(
                    &context.risk_class,
                    &context.tool_id,
                    &context.sandbox_root,
                    context.run_id.as_ref(),
                    context.attempt_id.as_ref(),
                )
        })
    }

    pub fn bound_context_for_tool(&self, tool_id: &str) -> Option<(ArtifactId, ArtifactId)> {
        self.grants.iter().find_map(|grant| {
            if grant.tool_id == tool_id && self.verify_grant(grant) {
                Some((grant.run_id.clone()?, grant.attempt_id.clone()?))
            } else {
                None
            }
        })
    }

    pub fn permit_use_receipt_for_context(
        &self,
        context: &PermitCheckContextV1,
    ) -> Option<PermitUseReportV1> {
        self.grant_for_context(context).map(|grant| {
            PermitUseReportV1::allowed(
                grant,
                context.tool_id.clone(),
                context.sandbox_root.clone(),
                context.run_id.clone(),
                context.attempt_id.clone(),
            )
        })
    }

    pub fn approval_request_for_tool(
        &self,
        tool_id: impl Into<String>,
        risk: CanonicalToolSideEffectClass,
        scope: impl Into<String>,
    ) -> Option<ApprovalRequestV1> {
        if self.decision_for_risk(&risk) == PermitDecisionV1::RequiresApproval {
            Some(ApprovalRequestV1::new(
                tool_id,
                risk,
                scope,
                "side-effect tool requires explicit permit",
            ))
        } else {
            None
        }
    }

    pub fn approval_request_for_context(
        &self,
        context: &PermitCheckContextV1,
    ) -> Option<ApprovalRequestV1> {
        if self.decision_for_context(context) == PermitDecisionV1::RequiresApproval {
            let mut request = ApprovalRequestV1::scoped(
                context.tool_id.clone(),
                context.risk_class.clone(),
                context.sandbox_root.clone(),
                "side-effect tool requires explicit scoped permit",
            );
            request.run_id = context.run_id.clone();
            request.attempt_id = context.attempt_id.clone();
            if context.run_id.is_some() || context.attempt_id.is_some() {
                request.scope = format!(
                    "{};run={};attempt={}",
                    request.scope,
                    context
                        .run_id
                        .as_ref()
                        .map(|id| id.0.as_str())
                        .unwrap_or(UNKNOWN_PERMIT_SCOPE_TOKEN),
                    context
                        .attempt_id
                        .as_ref()
                        .map(|id| id.0.as_str())
                        .unwrap_or(UNKNOWN_PERMIT_SCOPE_TOKEN)
                );
            }
            Some(request)
        } else {
            None
        }
    }

    pub fn deny_request(
        &self,
        request_id: ArtifactId,
        decided_by: impl Into<String>,
        reason: impl Into<String>,
    ) -> ApprovalDecisionV1 {
        ApprovalDecisionV1::denied(request_id, decided_by, reason)
    }
}

fn authentication_tag(key: &[u8; 32], material: &str) -> String {
    let mut authenticated = Vec::with_capacity(32 + material.len() + 32);
    authenticated.extend_from_slice(b"aidens.permit-auth.v1\0");
    authenticated.extend_from_slice(key);
    authenticated.extend_from_slice(&(material.len() as u64).to_be_bytes());
    authenticated.extend_from_slice(material.as_bytes());
    StackContentDigest::compute(&authenticated)
        .hex()
        .to_string()
}

pub fn default_decision(risk: &CanonicalToolSideEffectClass) -> PermitDecisionV1 {
    match risk {
        CanonicalToolSideEffectClass::ReadOnly => PermitDecisionV1::Allow,
        _ => PermitDecisionV1::RequiresApproval,
    }
}

pub fn requires_permit(risk: &CanonicalToolSideEffectClass) -> bool {
    !matches!(risk, CanonicalToolSideEffectClass::ReadOnly)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Barrier};

    const TEST_KEY: [u8; 32] = [7; 32];

    fn trusted_issuer() -> HostPermitAuthorityV1 {
        let root = std::env::temp_dir().join(format!(
            "aidens-permit-authority-test-{}",
            std::process::id()
        ));
        HostPermitAuthorityV1 {
            issuer_id: "test-issuer".into(),
            key: TEST_KEY,
            revocations: PermitRevocationStoreV1::open(root).unwrap(),
        }
    }

    fn host_policy_for_test(grant: PermitGrantV1) -> PermitPolicyV1 {
        PermitPolicyV1::default()
            .with_host_trusted_issuer("test-issuer", TEST_KEY)
            .with_grant(grant)
    }

    fn host_policy_with_revocation_store_for_test(
        grant: PermitGrantV1,
        root: &Path,
    ) -> PermitPolicyV1 {
        host_policy_for_test(grant)
            .with_revocation_store(PermitRevocationStoreV1::open(root).unwrap())
    }

    #[test]
    fn caller_constructed_grant_is_not_authority() {
        let grant = PermitGrantV1::scoped(
            CanonicalToolSideEffectClass::Write,
            "aidens:patch-apply:1",
            "/repo",
            "caller",
        );
        let context = PermitCheckContextV1::new(
            "aidens:patch-apply:1",
            CanonicalToolSideEffectClass::Write,
            "/repo",
        );

        assert_eq!(
            PermitPolicyV1::default()
                .with_host_trusted_issuer("test-issuer", TEST_KEY)
                .with_grant(grant)
                .decision_for_context(&context),
            PermitDecisionV1::RequiresApproval
        );
    }

    #[test]
    fn caller_cannot_self_register_an_issuer_as_host_authority() {
        let context = PermitCheckContextV1::new(
            "aidens:patch-apply:1",
            CanonicalToolSideEffectClass::Write,
            "/repo",
        )
        .with_run_attempt(
            Some(ArtifactId("run:one".into())),
            Some(ArtifactId("attempt:one".into())),
        );
        let caller_key = [99; 32];
        let mut grant = PermitGrantV1::scoped(
            context.risk_class.clone(),
            context.tool_id.clone(),
            context.sandbox_root.clone(),
            "caller",
        );
        grant.run_id = context.run_id.clone();
        grant.attempt_id = context.attempt_id.clone();
        grant.issuer_id = "caller".into();
        grant.refresh_authority_id();
        grant.integrity_tag = authentication_tag(&caller_key, &grant.authority_material());

        assert_eq!(
            PermitPolicyV1::default()
                .with_grant(grant)
                .decision_for_context(&context),
            PermitDecisionV1::RequiresApproval
        );
    }

    #[test]
    fn signed_grant_requires_exact_nonempty_run_and_attempt_ids() {
        let issuer = trusted_issuer();
        for context in [
            PermitCheckContextV1::new(
                "aidens:patch-apply:1",
                CanonicalToolSideEffectClass::Write,
                "/repo",
            ),
            PermitCheckContextV1::new(
                "aidens:patch-apply:1",
                CanonicalToolSideEffectClass::Write,
                "/repo",
            )
            .with_run_attempt(
                Some(ArtifactId(String::new())),
                Some(ArtifactId("attempt:one".into())),
            ),
            PermitCheckContextV1::new(
                "aidens:patch-apply:1",
                CanonicalToolSideEffectClass::Write,
                "/repo",
            )
            .with_run_attempt(Some(ArtifactId("run:one".into())), None),
        ] {
            let grant = issuer.issue_for_context(&context, "operator");
            let policy = host_policy_for_test(grant);
            assert_eq!(
                policy.decision_for_context(&context),
                PermitDecisionV1::RequiresApproval
            );
        }
    }

    #[test]
    fn signature_authenticates_every_mutable_grant_field() {
        let context = PermitCheckContextV1::new(
            "aidens:patch-apply:1",
            CanonicalToolSideEffectClass::Write,
            "/repo",
        )
        .with_run_attempt(
            Some(ArtifactId("run:one".into())),
            Some(ArtifactId("attempt:one".into())),
        );
        let grant = trusted_issuer().issue_for_context(&context, "operator");

        let mut mutations = Vec::new();
        let mut granted_at = grant.clone();
        granted_at.granted_at += std::time::Duration::from_secs(1);
        mutations.push(granted_at);
        let mut reasons = grant.clone();
        reasons.reason_codes.push("caller-added".into());
        mutations.push(reasons);
        let mut expiry = grant.clone();
        expiry.expires_at = Some(grant.granted_at + std::time::Duration::from_secs(3600));
        mutations.push(expiry);

        for mutated in mutations {
            assert!(!host_policy_for_test(mutated.clone()).verify_grant(&mutated));
        }
    }

    #[test]
    fn durable_revocation_is_reloaded_before_each_authorization() {
        let root =
            std::env::temp_dir().join(format!("aidens-permit-revocation-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let context = PermitCheckContextV1::new(
            "aidens:patch-apply:1",
            CanonicalToolSideEffectClass::Write,
            "/repo",
        )
        .with_run_attempt(
            Some(ArtifactId("run:one".into())),
            Some(ArtifactId("attempt:one".into())),
        );
        let grant = trusted_issuer().issue_for_context(&context, "operator");
        let policy = host_policy_with_revocation_store_for_test(grant.clone(), &root);
        assert_eq!(
            policy.decision_for_context(&context),
            PermitDecisionV1::Allow
        );

        PermitRevocationStoreV1::open(&root)
            .unwrap()
            .revoke(&grant.permit_id, "operator", "compromised")
            .unwrap();
        assert_eq!(
            policy.decision_for_context(&context),
            PermitDecisionV1::RequiresApproval
        );
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn concurrent_durable_revocations_preserve_every_permit() {
        let root = std::env::temp_dir().join(format!(
            "aidens-permit-revocation-concurrent-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos(),
        ));
        let _ = std::fs::remove_dir_all(&root);
        let store = PermitRevocationStoreV1::open(&root).unwrap();

        // A substantial existing ledger makes both concurrent calls spend enough time in
        // deserialize/serialize to expose a read-modify-write lost update in an unlocked store.
        let mut ledger = PermitRevocationLedgerV1::default();
        for index in 0..4_096 {
            let permit_id = format!("existing:{index}");
            ledger.revocations.insert(
                permit_id.clone(),
                PermitRevocationRecordV1 {
                    permit_id,
                    revoked_by: "seed".into(),
                    reason: "test setup".into(),
                    recorded_at_unix_nanos: "0".into(),
                },
            );
        }
        std::fs::write(
            root.join("permit-revocations-v1.json"),
            serde_json::to_vec(&ledger).unwrap(),
        )
        .unwrap();

        let barrier = Arc::new(Barrier::new(2));
        let left_store = store.clone();
        let left_barrier = barrier.clone();
        let left = std::thread::spawn(move || {
            left_barrier.wait();
            left_store.revoke(&ArtifactId("permit:left".into()), "operator", "left")
        });
        let right_store = store.clone();
        let right = std::thread::spawn(move || {
            barrier.wait();
            right_store.revoke(&ArtifactId("permit:right".into()), "operator", "right")
        });

        left.join().unwrap().unwrap();
        right.join().unwrap().unwrap();

        let reopened = PermitRevocationStoreV1::open(&root).unwrap();
        assert!(reopened
            .is_revoked(&ArtifactId("permit:left".into()))
            .unwrap());
        assert!(reopened
            .is_revoked(&ArtifactId("permit:right".into()))
            .unwrap());
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn trusted_grant_detects_tampering_and_revocation() {
        let issuer = trusted_issuer();
        let context = PermitCheckContextV1::new(
            "aidens:patch-apply:1",
            CanonicalToolSideEffectClass::Write,
            "/repo",
        )
        .with_run_attempt(
            Some(ArtifactId("run:one".into())),
            Some(ArtifactId("attempt:one".into())),
        );
        let grant = issuer.issue_for_context(&context, "operator");
        let policy = PermitPolicyV1::default()
            .with_host_trusted_issuer("test-issuer", TEST_KEY)
            .with_grant(grant.clone());
        assert_eq!(
            policy.decision_for_context(&context),
            PermitDecisionV1::Allow
        );

        let mut tampered = grant.clone();
        tampered.sandbox_root = "/other".into();
        let tampered_policy = PermitPolicyV1::default()
            .with_host_trusted_issuer("test-issuer", TEST_KEY)
            .with_grant(tampered);
        assert_eq!(
            tampered_policy.decision_for_context(&context),
            PermitDecisionV1::RequiresApproval
        );

        assert_eq!(
            policy
                .revoke(&grant.permit_id)
                .decision_for_context(&context),
            PermitDecisionV1::RequiresApproval
        );
    }

    #[test]
    fn permit_and_use_ids_are_replay_stable_and_context_bound() {
        let issuer = trusted_issuer();
        let context = PermitCheckContextV1::new(
            "aidens:patch-apply:1",
            CanonicalToolSideEffectClass::Write,
            "/repo",
        )
        .with_run_attempt(
            Some(ArtifactId("run:one".into())),
            Some(ArtifactId("attempt:one".into())),
        );
        let left = issuer.issue_for_context(&context, "operator");
        let right = left.clone();

        let left_use = PermitUseReportV1::allowed(
            &left,
            context.tool_id.clone(),
            context.sandbox_root.clone(),
            context.run_id.clone(),
            context.attempt_id.clone(),
        );
        let right_use = PermitUseReportV1::allowed(
            &right,
            context.tool_id.clone(),
            context.sandbox_root.clone(),
            context.run_id.clone(),
            context.attempt_id.clone(),
        );
        assert_ne!(left_use.receipt_id, right_use.receipt_id);

        let denied_left = PermitUseReportV1::denied(
            left.permit_id.clone(),
            context.tool_id.clone(),
            context.risk_class.clone(),
            context.sandbox_root.clone(),
            "expired",
        );
        let denied_right = PermitUseReportV1::denied(
            left.permit_id.clone(),
            context.tool_id.clone(),
            context.risk_class.clone(),
            context.sandbox_root.clone(),
            "revoked",
        );
        assert_ne!(denied_left.receipt_id, denied_right.receipt_id);

        let other_attempt =
            PermitCheckContextV1::new(context.tool_id, context.risk_class, context.sandbox_root)
                .with_run_attempt(
                    Some(ArtifactId("run:one".into())),
                    Some(ArtifactId("attempt:two".into())),
                );
        assert_ne!(
            left.permit_id,
            issuer
                .issue_for_context(&other_attempt, "operator")
                .permit_id
        );
    }

    #[test]
    fn write_requires_approval_by_default() {
        assert_eq!(
            default_decision(&CanonicalToolSideEffectClass::Write),
            PermitDecisionV1::RequiresApproval
        );
    }

    #[test]
    fn explicit_permit_allows_matching_risk() {
        let issuer = trusted_issuer();
        let context = PermitCheckContextV1::new(
            "aidens:admin:1",
            CanonicalToolSideEffectClass::Admin,
            "repo",
        )
        .with_run_attempt(
            Some(ArtifactId("run:admin".into())),
            Some(ArtifactId("attempt:admin".into())),
        );
        let permit = issuer.issue_for_context(&context, "test");
        let policy = PermitPolicyV1::default()
            .with_host_trusted_issuer("test-issuer", TEST_KEY)
            .with_permit(&permit);

        assert_eq!(
            policy.decision_for_risk(&CanonicalToolSideEffectClass::Admin),
            PermitDecisionV1::Allow
        );
        assert_eq!(
            policy.decision_for_risk(&CanonicalToolSideEffectClass::Write),
            PermitDecisionV1::RequiresApproval
        );
        assert_eq!(
            policy
                .grant_for_risk(&CanonicalToolSideEffectClass::Admin)
                .map(|grant| grant.permit_id.clone()),
            Some(permit.permit_id)
        );
    }

    #[test]
    fn scoped_permit_requires_matching_tool_and_sandbox() {
        let issuer = trusted_issuer();
        let matching = PermitCheckContextV1::new(
            "aidens:file-write:1",
            CanonicalToolSideEffectClass::Write,
            "/repo",
        )
        .with_run_attempt(
            Some(ArtifactId("run:write".into())),
            Some(ArtifactId("attempt:write".into())),
        );
        let permit = issuer.issue_for_context(&matching, "test");
        let policy = PermitPolicyV1::default()
            .with_host_trusted_issuer("test-issuer", TEST_KEY)
            .with_permit(&permit);
        let wrong_tool = PermitCheckContextV1::new(
            "aidens:shell:1",
            CanonicalToolSideEffectClass::Write,
            "/repo",
        )
        .with_run_attempt(matching.run_id.clone(), matching.attempt_id.clone());
        let wrong_root = PermitCheckContextV1::new(
            "aidens:file-write:1",
            CanonicalToolSideEffectClass::Write,
            "/other",
        )
        .with_run_attempt(matching.run_id.clone(), matching.attempt_id.clone());

        assert_eq!(
            policy.decision_for_context(&matching),
            PermitDecisionV1::Allow
        );
        assert_eq!(
            policy.decision_for_context(&wrong_tool),
            PermitDecisionV1::RequiresApproval
        );
        assert_eq!(
            policy.decision_for_context(&wrong_root),
            PermitDecisionV1::RequiresApproval
        );
        let receipt = policy
            .permit_use_receipt_for_context(&matching)
            .expect("matching grant emits use receipt");
        assert!(receipt.allowed);
        assert_eq!(receipt.permit_id, permit.permit_id);
    }

    #[test]
    fn approval_request_for_context_marks_missing_ids_explicitly() {
        let policy = PermitPolicyV1::default();
        let context = PermitCheckContextV1::new(
            "aidens:file-write:1",
            CanonicalToolSideEffectClass::Write,
            "repo",
        )
        .with_run_attempt(Some(ArtifactId("run-id:example".into())), None);

        let request = policy
            .approval_request_for_context(&context)
            .expect("write requests permit approval by default");

        assert!(request.scope.contains("run=run-id:example"));
        assert!(request.scope.contains("attempt=unknown"));
        assert!(request.scope.contains("tool=aidens:file-write:1"));
        assert!(!request.scope.contains("run=*"));
        assert!(!request.scope.contains("attempt=*"));
    }

    #[test]
    fn side_effect_risks_request_approval() {
        let policy = PermitPolicyV1::default();
        let request = policy
            .approval_request_for_tool(
                "aidens:file-write:1",
                CanonicalToolSideEffectClass::Write,
                "repo",
            )
            .expect("file-write requires approval");

        assert_eq!(request.tool_id, "aidens:file-write:1");
        assert_eq!(request.risk_class, CanonicalToolSideEffectClass::Write);
        assert!(requires_permit(&CanonicalToolSideEffectClass::Admin));
    }

    #[test]
    fn default_permit_policy_matches_reference_interpreter() {
        for risk_class in aidens_testkit::all_risk_classes() {
            let case = aidens_testkit::reference_permit_case(risk_class.clone());
            let decision = match default_decision(&risk_class) {
                PermitDecisionV1::Allow => "allow",
                PermitDecisionV1::RequiresApproval => "requires-approval",
                PermitDecisionV1::Deny(_) => "deny",
            };
            let reason_codes = if requires_permit(&risk_class) {
                vec!["approval-required"]
            } else {
                vec!["read-only-risk"]
            };
            let actual = serde_json::json!({
                "risk_class": aidens_testkit::json_string(&risk_class),
                "permit_required": requires_permit(&risk_class),
                "decision": decision,
                "reason_codes": reason_codes
            });
            let report = aidens_testkit::compare_case_to_actual(
                &case,
                "aidens-permit-kit::default_decision",
                actual,
            );

            assert!(
                report.passed,
                "{}",
                report
                    .findings
                    .iter()
                    .map(|finding| finding.human_diff.as_str())
                    .collect::<Vec<_>>()
                    .join("\n")
            );
        }
    }
}
