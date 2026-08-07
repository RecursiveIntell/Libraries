/// HMAC key lifecycle: generation, storage, loading, rotation, and integrity verification.
use context_governor::receipt_index::{
    generate_hmac_key, load_hmac_key, rotate_hmac_key, save_hmac_key, sign_receipt_content,
    verify_receipt_integrity,
};
use tempfile::tempdir;

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
