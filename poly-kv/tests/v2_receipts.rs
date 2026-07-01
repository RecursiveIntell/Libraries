use poly_kv::{
    AttentionSelectionReceiptV1, CacheIsolationMode, CacheSecurityPolicy, CompressionPolicy,
    Digest, HeadRole, RoleBudget,
};

#[test]
fn attention_selection_receipt_roundtrips() {
    let receipt = AttentionSelectionReceiptV1::new(
        Digest::from_hex_unchecked("pool"),
        Some(Digest::from_hex_unchecked("shell")),
        1,
        2,
        10,
        3,
        0,
        4,
        false,
        Some(0.01),
    );
    assert!(receipt.validate().is_ok());
    let json = serde_json::to_string(&receipt).unwrap();
    let decoded: AttentionSelectionReceiptV1 = serde_json::from_str(&json).unwrap();
    assert_eq!(decoded.decoded_keys, 0);
}

#[test]
fn shared_isolated_mode_requires_receipts() {
    let bad = CacheSecurityPolicy {
        tenant_or_scope: "tenant".into(),
        isolation_mode: CacheIsolationMode::SharedIsolated,
        allow_shared_pool: true,
        require_access_receipts: false,
    };
    assert!(bad.validate().is_err());
    let good = CacheSecurityPolicy {
        require_access_receipts: true,
        ..bad
    };
    assert!(good.validate().is_ok());
}

#[test]
fn compression_policy_role_budgets_validate() {
    let mut policy = CompressionPolicy::default_two_tier();
    policy.role_budgets.push(RoleBudget {
        role: HeadRole::Retrieval,
        min_bits: 0,
        max_bits: 0,
        force_exact_fallback: false,
    });
    assert!(policy.validate().is_err());
}
