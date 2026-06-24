//! Tests for aidens-delegation-kit

use aidens_delegation_kit::{DelegationError, DelegationKitStatus};

#[test]
fn default_status_is_disabled() {
    let status = DelegationKitStatus::default();
    assert!(!status.enabled);
    assert!(status.note.contains("quarantined"));
}

#[test]
fn error_is_canonical_owner_required() {
    let err = DelegationError::CanonicalOwnerRequired;
    assert!(err.to_string().contains("canonical owner"));
}

#[test]
fn status_is_cloneable() {
    let status = DelegationKitStatus::default();
    let cloned = status.clone();
    assert_eq!(status, cloned);
}