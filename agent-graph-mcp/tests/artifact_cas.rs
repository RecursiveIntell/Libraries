#![allow(clippy::expect_used)]

use agent_graph_mcp::artifact_store::ArtifactStoreError;
use agent_graph_mcp::store::PersistentStore;
use stack_ids::{ArtifactId, ContentDigest};
use std::fs;
use tempfile::tempdir;

#[test]
fn cas_put_get_and_corruption_quarantine_are_deterministic() {
    let temp = tempdir().expect("temporary data root");
    let persistent = PersistentStore::open(temp.path()).expect("persistent store");
    let store = persistent.artifacts(1024).expect("artifact store");
    let bytes = b"immutable artifact bytes";
    let digest = ContentDigest::compute(bytes);
    let reference = store
        .put(
            ArtifactId::new("artifact-1"),
            bytes,
            "application/octet-stream",
            &digest,
        )
        .expect("put artifact");
    assert_eq!(reference.size_bytes, bytes.len() as u64);
    assert_eq!(store.get(&digest).expect("get artifact"), bytes);
    let path = store.blob_path(&digest);
    fs::write(&path, b"tampered").expect("tamper blob");
    assert!(matches!(
        store.get(&digest),
        Err(ArtifactStoreError::Quarantined)
    ));
    assert!(!path.exists());
}

#[test]
fn cas_rejects_wrong_digest_and_oversized_input() {
    let temp = tempdir().expect("temporary data root");
    let persistent = PersistentStore::open(temp.path()).expect("persistent store");
    let store = persistent.artifacts(4).expect("artifact store");
    let wrong = ContentDigest::compute(b"wrong");
    assert!(matches!(
        store.put(
            ArtifactId::new("artifact-2"),
            b"bytes",
            "text/plain",
            &wrong
        ),
        Err(ArtifactStoreError::TooLarge)
    ));
    let right = ContentDigest::compute(b"abc");
    assert!(matches!(
        store.put(ArtifactId::new("artifact-3"), b"abc", "text/plain", &wrong),
        Err(ArtifactStoreError::DigestMismatch)
    ));
    assert!(store
        .put(ArtifactId::new("artifact-4"), b"abc", "text/plain", &right)
        .is_ok());
}
