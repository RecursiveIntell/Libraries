use context_governor::{
    migrate_v2_store, read_v3_evidence, read_v3_manifest, CompactRequest, CompactionPolicy,
    FileContextStore, Message, V3MigrationOptions,
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
fn shared_encrypted_evidence_remains_readable_for_every_receipt() {
    let temp = tempfile::tempdir().unwrap();
    let store = FileContextStore::with_hmac_key(temp.path().join("source"), &[3; 32]);
    let ids = [populate(&store, "session-a"), populate(&store, "session-b")];
    let output = temp.path().join("projection");
    let key = vec![7; 32];
    let options = V3MigrationOptions {
        encryption_key: Some(key.clone()),
        require_encryption: true,
        ..Default::default()
    };
    let report = migrate_v2_store(&store, &output, &options).unwrap();
    assert!(report.complete, "{report:?}");
    for id in &ids {
        let manifest = read_v3_manifest(&output, id).unwrap();
        for reference in &manifest.evidence {
            let result = read_v3_evidence(&output, &manifest, &reference.source_id, Some(&key));
            assert!(
                result.is_ok(),
                "shared encrypted blob cannot be read for {id}/{}: {result:?}",
                reference.source_id
            );
        }
    }
}

#[test]
fn corrupt_existing_blob_cannot_be_reported_as_completed_resume() {
    let temp = tempfile::tempdir().unwrap();
    let store = FileContextStore::with_hmac_key(temp.path().join("source"), &[3; 32]);
    let id = populate(&store, "resume-session");
    let output = temp.path().join("projection");
    let options = V3MigrationOptions::default();
    assert!(migrate_v2_store(&store, &output, &options).unwrap().complete);
    let manifest = read_v3_manifest(&output, &id).unwrap();
    let path = output.join(".v3").join(&manifest.evidence[0].blob_relpath);
    std::fs::write(path, b"corrupt existing blob").unwrap();
    let result = migrate_v2_store(&store, &output, &options);
    assert!(
        result.as_ref().map(|report| !report.complete).unwrap_or(true),
        "corrupt existing output was silently accepted: {result:?}"
    );
}

#[test]
fn missing_source_directory_is_not_successful_empty_migration() {
    let temp = tempfile::tempdir().unwrap();
    let store = FileContextStore::with_hmac_key(temp.path().join("missing"), &[3; 32]);
    let result = migrate_v2_store(&store, temp.path().join("projection"), &Default::default());
    assert!(
        result.as_ref().map(|report| !report.complete).unwrap_or(true),
        "missing source was silently treated as a complete empty corpus: {result:?}"
    );
}
