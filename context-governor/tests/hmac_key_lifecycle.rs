/// HMAC key lifecycle: generation, storage, loading, rotation, and integrity verification.
use context_governor::receipt_index::{
    generate_hmac_key, key_fingerprint, load_hmac_key, load_hmac_key_ring, rotate_hmac_key,
    save_hmac_key, sign_receipt_content, verify_all_receipts, verify_receipt_integrity, KeyRing,
};
use context_governor::{
    compact_context, CompactRequest, CompactionPolicy, FileContextStore, Message,
};
use tempfile::tempdir;

fn msg(role: &str, content: &str) -> Message {
    Message {
        id: None,
        role: role.into(),
        content: content.into(),
        name: None,
        metadata: Default::default(),
    }
}

#[test]
fn generate_sign_verify_cycle() {
    let key = generate_hmac_key();
    assert_eq!(key.len(), 32);
    let content = "test receipt payload";
    let sig = sign_receipt_content(content, &key);
    assert_eq!(sig.len(), 64); // hex-encoded SHA256
    assert!(verify_receipt_integrity(content, &key, &sig));
}

#[test]
fn tampered_content_rejected() {
    let key = generate_hmac_key();
    let sig = sign_receipt_content("original", &key);
    assert!(!verify_receipt_integrity("tampered", &key, &sig));
}

#[test]
fn wrong_key_rejected() {
    let key_a = generate_hmac_key();
    let key_b = generate_hmac_key();
    assert_ne!(key_a, key_b);
    let sig = sign_receipt_content("data", &key_a);
    assert!(!verify_receipt_integrity("data", &key_b, &sig));
}

#[test]
fn determinism() {
    let key = generate_hmac_key();
    let sig1 = sign_receipt_content("hello", &key);
    let sig2 = sign_receipt_content("hello", &key);
    assert_eq!(sig1, sig2);
}

#[test]
fn save_load_cycle() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("test.key");
    let key = generate_hmac_key();
    save_hmac_key(&path, &key).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let metadata = std::fs::metadata(&path).unwrap();
        assert_eq!(metadata.permissions().mode() & 0o777, 0o600);
    }
    let loaded = load_hmac_key(&path).unwrap();
    assert_eq!(key, loaded);
}

#[test]
fn rotation_preserves_old_key() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("rotate.key");
    let old_key = generate_hmac_key();
    save_hmac_key(&path, &old_key).unwrap();

    let (old_back, new_key) = rotate_hmac_key(&path).unwrap();
    assert_eq!(old_key, old_back);
    assert_ne!(old_key, new_key);
    // Old key can still verify existing signatures
    let sig = sign_receipt_content("persisted data", &old_key);
    assert!(verify_receipt_integrity("persisted data", &old_key, &sig));
    // New key is on disk
    let loaded = load_hmac_key(&path).unwrap();
    assert_eq!(new_key, loaded);
}

#[test]
fn multi_key_ring_verifies_old_receipts() {
    let old_key = generate_hmac_key();
    let new_key = generate_hmac_key();
    let old_fpr = key_fingerprint(&old_key);

    // Sign with old key
    let content = "old receipt";
    let sig = sign_receipt_content(content, &old_key);
    let full = format!("{old_fpr}:{sig}");

    // Key ring with new_key active, old_key retired
    let mut ring = KeyRing::new(new_key);
    ring.retired.push((old_fpr.clone(), old_key.clone()));

    // Should verify with retired key
    assert!(ring.sign_and_verify(content, &full));

    // Should NOT verify with wrong fingerprint
    let wrong = format!("deadbeef:{sig}");
    assert!(!ring.sign_and_verify(content, &wrong));
}

#[test]
fn literal_v1_eight_hex_signature_with_legacy_key_length_verifies() {
    // Literal fixture from the historical V1 wire format: short key material,
    // 8-hex SHA-256 fingerprint prefix, and HMAC over canonical JSON without
    // the detached `hmac` field.
    let fixture = serde_json::json!({
        "receipt_id": "ctxr_legacy_literal",
        "schema": "ContextCompactionReceiptV1",
        "value": 7,
        "hmac": "94eeb7bb:19758fd3c9d0b11fd6a665eb696425a801779fe4be1c29d96f4d78f9efd05b94"
    });
    let ring = KeyRing::new(b"legacy-key".to_vec());
    assert!(ring.verify_json(&fixture, "hmac"));
    let dir = tempdir().unwrap();
    std::fs::write(
        dir.path().join("ctxr_legacy_literal.json"),
        serde_json::to_vec_pretty(&fixture).unwrap(),
    )
    .unwrap();
    assert_eq!(
        verify_all_receipts(dir.path(), &ring, None),
        (1, 1, Vec::new())
    );

    let mut full_id_fixture = fixture;
    full_id_fixture["hmac"] = serde_json::Value::String(
        "94eeb7bbe979dd0d2f0bb085172b76a1bc9e61789839480834a9bcb5aaf9c6ef:19758fd3c9d0b11fd6a665eb696425a801779fe4be1c29d96f4d78f9efd05b94"
            .to_string(),
    );
    assert!(ring.verify_json(&full_id_fixture, "hmac"));
}

#[test]
fn legacy_keyring_loader_accepts_short_v1_verification_key_only() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("legacy-v1.key");
    std::fs::write(&path, b"legacy-key").unwrap();
    assert!(load_hmac_key(&path).is_err());
    let ring = load_hmac_key_ring(&path).unwrap();
    assert_eq!(ring.active, b"legacy-key");
    // V2/current signing remains strict and cannot mint a full key identity
    // from this historical verification-only material.
    assert!(ring.active_key_id().is_err());
}

#[test]
fn fingerprint_is_deterministic() {
    let key = generate_hmac_key();
    assert_eq!(key_fingerprint(&key), key_fingerprint(&key));
}

#[test]
fn fingerprint_differs_for_different_keys() {
    let k1 = generate_hmac_key();
    let k2 = generate_hmac_key();
    assert_ne!(key_fingerprint(&k1), key_fingerprint(&k2));
}

#[test]
fn rotated_key_ring_loads_retired_key_and_verifies_old_signature() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("rotate.key");
    let old_key = generate_hmac_key();
    save_hmac_key(&path, &old_key).unwrap();

    let receipt = serde_json::json!({"receipt_id": "ctxr_rotation", "value": 1});
    let old_ring = KeyRing::new(old_key);
    let signature = old_ring.sign_json(&receipt, "hmac").unwrap();
    let mut signed = receipt;
    signed["hmac"] = serde_json::Value::String(signature);

    rotate_hmac_key(&path).unwrap();
    let ring = load_hmac_key_ring(&path).unwrap();
    assert_eq!(ring.retired.len(), 1);
    assert!(ring.verify_json(&signed, "hmac"));
}

#[test]
fn batch_verification_is_read_only_and_detects_tampering() {
    let dir = tempdir().unwrap();
    let key = generate_hmac_key();
    let ring = KeyRing::new(key);
    let mut receipt = serde_json::json!({"receipt_id": "ctxr_smoke", "value": 1});
    receipt["hmac"] = serde_json::Value::String(ring.sign_json(&receipt, "hmac").unwrap());
    let path = dir.path().join("ctxr_smoke.json");
    std::fs::write(&path, serde_json::to_string_pretty(&receipt).unwrap()).unwrap();
    let before = std::fs::read(&path).unwrap();

    let (total, passed, failures) = verify_all_receipts(dir.path(), &ring, None);
    assert_eq!((total, passed), (1, 1));
    assert!(failures.is_empty());
    assert_eq!(before, std::fs::read(&path).unwrap());

    receipt["value"] = serde_json::json!(2);
    std::fs::write(&path, serde_json::to_string_pretty(&receipt).unwrap()).unwrap();
    let (total, passed, failures) = verify_all_receipts(dir.path(), &ring, None);
    assert_eq!((total, passed), (1, 0));
    assert_eq!(failures, vec!["ctxr_smoke: HMAC verification failed"]);
}

#[test]
fn durable_store_re_signs_final_receipt_and_rejects_keyless_signed_write() {
    let dir = tempdir().unwrap();
    let key_path = dir.path().join("hmac.key");
    let key = generate_hmac_key();
    save_hmac_key(&key_path, &key).unwrap();
    let response = compact_context(CompactRequest {
        session_id: "signed-store".into(),
        messages: vec![msg("tool", &"bulk ".repeat(600)), msg("user", "latest")],
        policy: CompactionPolicy {
            target_tokens: 100,
            protect_last_n: 1,
            ..Default::default()
        },
        focus: None,
        hmac_key_path: Some(key_path.display().to_string()),
    })
    .unwrap();
    assert!(response.hmac.is_some());

    let store = FileContextStore::new(dir.path().join("receipts"));
    assert!(store.save_with_status(&response).is_err());
    let saved = store
        .save_with_status_with_hmac_key(&response, &key)
        .unwrap();
    let ring = KeyRing::new(key);
    let persisted: serde_json::Value =
        serde_json::from_slice(&std::fs::read(saved.path).unwrap()).unwrap();
    assert!(ring.verify_json(&persisted, "hmac"));
}
