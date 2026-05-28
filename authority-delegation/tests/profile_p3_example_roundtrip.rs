#![allow(clippy::expect_used)]

use authority_delegation::{
    ApprovalMatrixV1, ConflictClassCatalogV1, DelegationMatrixV1, RoleCatalogV1,
};

#[test]
fn role_catalog_v1_roundtrips() {
    let body =
        std::fs::read_to_string("../examples/role-catalog-v1.example.json").expect("read example");
    let value: RoleCatalogV1 = serde_json::from_str(&body).expect("parse example");
    let encoded = serde_json::to_string(&value).expect("serialize");
    let _: RoleCatalogV1 = serde_json::from_str(&encoded).expect("deserialize");
}

#[test]
fn delegation_matrix_v1_roundtrips() {
    let body = std::fs::read_to_string("../examples/delegation-matrix-v1.example.json")
        .expect("read example");
    let value: DelegationMatrixV1 = serde_json::from_str(&body).expect("parse example");
    let encoded = serde_json::to_string(&value).expect("serialize");
    let _: DelegationMatrixV1 = serde_json::from_str(&encoded).expect("deserialize");
}

#[test]
fn approval_matrix_v1_roundtrips() {
    let body = std::fs::read_to_string("../examples/approval-matrix-v1.example.json")
        .expect("read example");
    let value: ApprovalMatrixV1 = serde_json::from_str(&body).expect("parse example");
    let encoded = serde_json::to_string(&value).expect("serialize");
    let _: ApprovalMatrixV1 = serde_json::from_str(&encoded).expect("deserialize");
}

#[test]
fn conflict_class_catalog_v1_roundtrips() {
    let body = std::fs::read_to_string("../examples/conflict-class-catalog-v1.example.json")
        .expect("read example");
    let value: ConflictClassCatalogV1 = serde_json::from_str(&body).expect("parse example");
    let encoded = serde_json::to_string(&value).expect("serialize");
    let _: ConflictClassCatalogV1 = serde_json::from_str(&encoded).expect("deserialize");
}
