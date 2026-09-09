use context_governor::{
    migrate_v2_store, read_v3_manifest, verify_v3_projection, CompactRequest, CompactionPolicy,
    ContextGovernorError, FileContextStore, Message, V3MigrationOptions, V3ProjectionError,
};

fn populate(store: &FileContextStore, session: &str) -> String {
    let response = store
        .compact_next_v2(
            CompactRequest {
                session_id: session.into(),
                messages: vec![
                    Message {
                        role: "tool".into(),
                        content: "shared exact evidence marker ".repeat(500),
                        ..Default::default()
                    },
                    Message {
                        role: "user".into(),
                        content: "latest task".into(),
                        ..Default::default()
                    },
                ],
                policy: CompactionPolicy {
                    target_tokens: 260,
                    protect_first_n: 0,
                    protect_last_n: 1,
                    ..Default::default()
                },
                focus: None,
                hmac_key_path: None,
            },
            None,
        )
        .unwrap();
    let id = response.receipt.receipt_id.clone();
    store.save_v2(&response).unwrap();
    store.rebuild_lineage_index().unwrap();
    id
}

#[test]
fn existing_projection_is_verified_read_only_and_wrong_key_fails_closed() {
    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("source");
    let store = FileContextStore::with_hmac_key(&source, &[3; 32]);
    populate(&store, "verify-session");
    let output = temp.path().join("projection");
    let key = vec![7; 32];
    let options = V3MigrationOptions {
        encryption_key: Some(key.clone()),
        require_encryption: true,
        ..Default::default()
    };
    let migrated = migrate_v2_store(&store, &output, &options).unwrap();
    let verified = verify_v3_projection(&store, &output, Some(&key)).unwrap();
    assert!(verified.complete);
    assert_eq!(verified.input_receipts, migrated.input_receipts);
    assert_eq!(verified.exact_evidence_items, migrated.exact_evidence_items);
    assert_eq!(
        verified.unique_evidence_blobs,
        migrated.unique_evidence_blobs
    );
    assert_eq!(verified.plaintext_bytes, migrated.plaintext_bytes);
    assert_eq!(verified.blob_bytes, migrated.blob_bytes);

    let wrong = verify_v3_projection(&store, &output, Some(&[9; 32]));
    assert!(matches!(
        wrong,
        Err(ContextGovernorError::V3Projection(
            V3ProjectionError::SourceMismatch
        ))
    ));
    let second_migration = migrate_v2_store(&store, &output, &options);
    assert!(matches!(
        second_migration,
        Err(ContextGovernorError::V3Projection(
            V3ProjectionError::ExistingProjection
        ))
    ));
}

#[test]
fn projection_verification_detects_manifest_tampering() {
    let temp = tempfile::tempdir().unwrap();
    let store = FileContextStore::with_hmac_key(temp.path().join("source"), &[3; 32]);
    let id = populate(&store, "tamper-session");
    let output = temp.path().join("projection");
    migrate_v2_store(&store, &output, &Default::default()).unwrap();
    let path = output.join(".v3/manifests").join(format!("{id}.json"));
    let mut value: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    value["session_id"] = serde_json::Value::String("attacker-session".into());
    std::fs::write(&path, serde_json::to_vec_pretty(&value).unwrap()).unwrap();
    let result = verify_v3_projection(&store, &output, None);
    assert!(matches!(
        result,
        Err(ContextGovernorError::V3Projection(
            V3ProjectionError::SourceMismatch
        ))
    ));
}

#[test]
fn projection_verification_detects_source_snapshot_drift() {
    let temp = tempfile::tempdir().unwrap();
    let store = FileContextStore::with_hmac_key(temp.path().join("source"), &[3; 32]);
    populate(&store, "before-snapshot");
    let output = temp.path().join("projection");
    migrate_v2_store(&store, &output, &Default::default()).unwrap();
    populate(&store, "after-snapshot");
    let result = verify_v3_projection(&store, &output, None);
    assert!(matches!(
        result,
        Err(ContextGovernorError::V3Projection(
            V3ProjectionError::SourceChanged
        ))
    ));
}

#[test]
fn concurrent_fresh_migrations_have_one_projection_owner() {
    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("source");
    let seed = FileContextStore::with_hmac_key(&source, &[3; 32]);
    populate(&seed, "race-session");
    let output = temp.path().join("projection");
    let barrier = std::sync::Arc::new(std::sync::Barrier::new(2));
    let mut handles = Vec::new();
    for _ in 0..2 {
        let source = source.clone();
        let output = output.clone();
        let barrier = barrier.clone();
        handles.push(std::thread::spawn(move || {
            let store = FileContextStore::with_hmac_key(source, &[3; 32]);
            barrier.wait();
            migrate_v2_store(&store, output, &Default::default())
        }));
    }
    let results = handles
        .into_iter()
        .map(|handle| handle.join().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(results.iter().filter(|result| result.is_ok()).count(), 1);
    assert_eq!(
        results
            .iter()
            .filter(|result| matches!(
                result,
                Err(ContextGovernorError::V3Projection(
                    V3ProjectionError::ExistingProjection
                ))
            ))
            .count(),
        1
    );
    let store = FileContextStore::with_hmac_key(source, &[3; 32]);
    assert!(verify_v3_projection(&store, output, None).unwrap().complete);
}

#[cfg(unix)]
#[test]
fn projection_verification_rejects_symlinked_blob() {
    use std::os::unix::fs::symlink;

    let temp = tempfile::tempdir().unwrap();
    let store = FileContextStore::with_hmac_key(temp.path().join("source"), &[3; 32]);
    let id = populate(&store, "symlink-session");
    let output = temp.path().join("projection");
    migrate_v2_store(&store, &output, &Default::default()).unwrap();
    let manifest = read_v3_manifest(&output, &id).unwrap();
    let blob = output.join(".v3").join(&manifest.evidence[0].blob_relpath);
    let saved = temp.path().join("saved-blob");
    std::fs::rename(&blob, &saved).unwrap();
    symlink(&saved, &blob).unwrap();
    let result = verify_v3_projection(&store, &output, None);
    assert!(matches!(
        result,
        Err(ContextGovernorError::V3Projection(
            V3ProjectionError::UnsafePath
        ))
    ));
}
