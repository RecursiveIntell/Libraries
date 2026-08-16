//! Controlled-fixture authority tests: no production key lifecycle is touched.

use context_governor::{key_authority::GovernedKeyAuthority, receipt_index};
use std::fs;
use std::os::fd::AsRawFd;
use tempfile::tempdir;

fn fixture(
    snapshot_active: Option<String>,
) -> (tempfile::NamedTempFile, tempfile::NamedTempFile, String) {
    let directory = tempdir().unwrap();
    // Keep directory alive by leaking this tiny controlled test fixture; files
    // themselves are NamedTempFile-owned and are removed on drop.
    let directory = Box::leak(Box::new(directory));
    let mut key = tempfile::NamedTempFile::new_in(directory.path()).unwrap();
    use std::io::Write;
    key.write_all(&[0x5a; 32]).unwrap();
    key.flush().unwrap();
    let id = receipt_index::key_id(&[0x5a; 32]).unwrap();
    let snapshot = serde_json::json!({
        "schema": "AresContextGovernorKeySnapshotV2",
        "sequence": 1,
        "active_key_id": snapshot_active.unwrap_or_else(|| id.clone()),
        "retired_key_ids": [],
        "compromised_key_ids": [],
        "keys": {id.clone(): "active"},
    });
    let mut metadata = tempfile::NamedTempFile::new_in(directory.path()).unwrap();
    metadata
        .write_all(serde_json::to_string(&snapshot).unwrap().as_bytes())
        .unwrap();
    metadata.flush().unwrap();
    (key, metadata, id)
}

#[test]
fn held_descriptors_establish_the_only_signing_authority() {
    let (key, snapshot, id) = fixture(None);
    let authority =
        GovernedKeyAuthority::from_fds(key.as_raw_fd(), snapshot.as_raw_fd(), &[]).unwrap();
    assert_eq!(authority.key_ring().active_key_id().unwrap(), id);
}

#[test]
fn forged_snapshot_active_metadata_is_rejected() {
    let (key, snapshot, _) = fixture(Some("0".repeat(64)));
    assert!(GovernedKeyAuthority::from_fds(key.as_raw_fd(), snapshot.as_raw_fd(), &[]).is_err());
}

#[test]
fn missing_historical_descriptor_is_rejected() {
    let (key, snapshot, id) = fixture(None);
    let retired = "1".repeat(64);
    fs::write(
        snapshot.path(),
        serde_json::to_string(&serde_json::json!({
            "schema": "AresContextGovernorKeySnapshotV2", "sequence": 2,
            "active_key_id": id, "retired_key_ids": [retired],
            "compromised_key_ids": [], "keys": {}
        }))
        .unwrap(),
    )
    .unwrap();
    assert!(GovernedKeyAuthority::from_fds(key.as_raw_fd(), snapshot.as_raw_fd(), &[]).is_err());
}

#[test]
fn governed_v2_authority_rejects_legacy_short_key_material() {
    let mut key = tempfile::NamedTempFile::new().unwrap();
    use std::io::Write;
    key.write_all(b"legacy-key").unwrap();
    key.flush().unwrap();
    let mut snapshot = tempfile::NamedTempFile::new().unwrap();
    snapshot
        .write_all(
            serde_json::to_string(&serde_json::json!({
                "schema": "AresContextGovernorKeySnapshotV2",
                "sequence": 1,
                "active_key_id": "94eeb7bbe979dd0d2f0bb085172b76a1bc9e61789839480834a9bcb5aaf9c6ef",
                "retired_key_ids": [],
                "compromised_key_ids": [],
                "keys": {},
            }))
            .unwrap()
            .as_bytes(),
        )
        .unwrap();
    snapshot.flush().unwrap();

    assert!(matches!(
        GovernedKeyAuthority::from_fds(key.as_raw_fd(), snapshot.as_raw_fd(), &[]),
        Err(context_governor::ContextGovernorError::InvalidKeyLength { .. })
    ));
}
