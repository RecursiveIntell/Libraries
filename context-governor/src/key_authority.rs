//! Descriptor-backed authority injected by Ares for certified operations.
//!
//! Neither request JSON nor certified CLI accepts a key path, raw key bytes,
//! keyring path, or a requested signing key ID.  Ares opens the governed files
//! while holding its lifecycle lock and passes only inherited descriptors.

use crate::{receipt_index, ContextGovernorError};
use serde::Deserialize;
use std::collections::{BTreeMap, HashSet};

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct SnapshotV2 {
    schema: String,
    sequence: u64,
    active_key_id: String,
    #[serde(default)]
    retired_key_ids: Vec<String>,
    #[serde(default)]
    compromised_key_ids: Vec<String>,
    #[serde(default)]
    keys: BTreeMap<String, String>,
}

fn descriptor_path(fd: i32) -> Result<String, ContextGovernorError> {
    if fd < 0 {
        return Err(ContextGovernorError::KeyUnreadable {
            path: "negative governed descriptor".to_string(),
        });
    }
    let path = format!("/proc/self/fd/{fd}");
    std::fs::metadata(&path)
        .map_err(|_| ContextGovernorError::KeyUnreadable { path: path.clone() })?;
    Ok(path)
}

/// The complete cryptographic authority for a single certified operation.
#[derive(Debug, Clone)]
pub struct GovernedKeyAuthority {
    ring: receipt_index::KeyRing,
    pub snapshot_sequence: u64,
}

impl GovernedKeyAuthority {
    /// `retired` is supplied as `(declared key id, inherited fd)` pairs.  It
    /// cannot select a signer: the snapshot's active id is authoritative.
    pub fn from_fds(
        active_fd: i32,
        snapshot_fd: i32,
        retired: &[(String, i32)],
    ) -> Result<Self, ContextGovernorError> {
        let active_path = descriptor_path(active_fd)?;
        let snapshot_path = descriptor_path(snapshot_fd)?;
        let raw = std::fs::read_to_string(&snapshot_path).map_err(|_| {
            ContextGovernorError::KeyUnreadable {
                path: snapshot_path.clone(),
            }
        })?;
        let snapshot: SnapshotV2 =
            serde_json::from_str(&raw).map_err(|_| ContextGovernorError::InvalidKeyEncoding {
                path: snapshot_path.clone(),
            })?;
        if snapshot.schema != "AresContextGovernorKeySnapshotV2" {
            return Err(ContextGovernorError::ConflictingActiveKeyState {
                reason: "unexpected governed snapshot schema".to_string(),
            });
        }
        if snapshot.active_key_id.is_empty()
            || snapshot.retired_key_ids.contains(&snapshot.active_key_id)
        {
            return Err(ContextGovernorError::ConflictingActiveKeyState {
                reason: "ambiguous active key state".to_string(),
            });
        }
        let active = receipt_index::load_hmac_key(std::path::Path::new(&active_path))?;
        let active_id = receipt_index::key_id(&active)?;
        if active_id != snapshot.active_key_id {
            return Err(ContextGovernorError::WrongConfiguredKeyId {
                expected: snapshot.active_key_id,
                actual: active_id,
            });
        }
        if snapshot.compromised_key_ids.contains(&active_id) {
            return Err(ContextGovernorError::CompromisedKey { key_id: active_id });
        }
        if !snapshot.keys.is_empty()
            && snapshot.keys.get(&active_id).map(String::as_str) != Some("active")
        {
            return Err(ContextGovernorError::ConflictingActiveKeyState {
                reason: "active key metadata mismatch".to_string(),
            });
        }
        let expected: HashSet<_> = snapshot.retired_key_ids.iter().cloned().collect();
        let supplied: HashSet<_> = retired.iter().map(|(id, _)| id.clone()).collect();
        if expected != supplied {
            return Err(ContextGovernorError::RequiredHistoricalKeyUnavailable {
                key_id: "governed snapshot retired key set".to_string(),
            });
        }
        let mut ring = receipt_index::KeyRing::new(active);
        ring.compromised = snapshot.compromised_key_ids.into_iter().collect();
        for (id, fd) in retired {
            if ring.compromised.contains(id) {
                continue;
            }
            if !snapshot.keys.is_empty()
                && snapshot.keys.get(id).map(String::as_str) != Some("retired")
            {
                return Err(ContextGovernorError::ConflictingActiveKeyState {
                    reason: "retired key metadata mismatch".to_string(),
                });
            }
            let path = descriptor_path(*fd)?;
            let key = receipt_index::load_hmac_key(std::path::Path::new(&path))?;
            let actual = receipt_index::key_id(&key)?;
            if actual != *id {
                return Err(ContextGovernorError::ComputedKeyIdMismatch {
                    expected: id.clone(),
                    actual,
                });
            }
            ring.retired.push((id.clone(), key));
        }
        Ok(Self {
            ring,
            snapshot_sequence: snapshot.sequence,
        })
    }

    pub fn key_ring(&self) -> &receipt_index::KeyRing {
        &self.ring
    }
}
