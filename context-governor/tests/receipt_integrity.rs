use context_governor::receipt_index::{sign_receipt_content, verify_receipt_integrity};

#[test]
fn hmac_verifies_same_content() {
    let key = b"test-integrity-key";
    let content = r#"{"receipt_id":"ctxr_test","compacted":42}"#;
    let hmac = sign_receipt_content(content, key);
    assert!(verify_receipt_integrity(content, key, &hmac));
}

#[test]
fn hmac_rejects_tampered_content() {
    let key = b"test-integrity-key";
    let original = r#"{"receipt_id":"ctxr_test","compacted":42}"#;
    let tampered = r#"{"receipt_id":"ctxr_test","compacted":99}"#;
    let hmac = sign_receipt_content(original, key);
    assert!(!verify_receipt_integrity(tampered, key, &hmac));
}

#[test]
fn hmac_rejects_wrong_key() {
    let content = r#"{"receipt_id":"ctxr_test"}"#;
    let hmac = sign_receipt_content(content, b"key-a");
    assert!(!verify_receipt_integrity(content, b"key-b", &hmac));
}

#[test]
fn hmac_is_deterministic_same_key_same_content() {
    let key = b"stable-key";
    let content = "deterministic input";
    let first = sign_receipt_content(content, key);
    let second = sign_receipt_content(content, key);
    assert_eq!(first, second);
    // 64 hex chars = 32 bytes = SHA-256 output
    assert_eq!(first.len(), 64);
}
