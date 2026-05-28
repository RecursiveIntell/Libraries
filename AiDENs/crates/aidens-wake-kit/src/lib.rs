//! Wake-signal construction for P11 daemon queues.

use aidens_contracts::{ArtifactId, CanonicalToolSideEffectClass, WakeSignalV1};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum WakeError {
    #[error("wake source must not be empty")]
    EmptySource,
    #[error("wake signal key must not be empty")]
    EmptySignalKey,
}

pub fn wake_signal(
    namespace_id: ArtifactId,
    source: impl Into<String>,
    signal_key: impl Into<String>,
    payload: serde_json::Value,
    risk: CanonicalToolSideEffectClass,
) -> Result<WakeSignalV1, WakeError> {
    let source = source.into();
    let signal_key = signal_key.into();
    if source.trim().is_empty() {
        return Err(WakeError::EmptySource);
    }
    if signal_key.trim().is_empty() {
        return Err(WakeError::EmptySignalKey);
    }
    Ok(WakeSignalV1::new(
        namespace_id,
        source,
        signal_key,
        payload,
        risk,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use aidens_contracts::DaemonNamespaceV1;

    #[test]
    fn wake_signal_identity_uses_source_key_and_payload() {
        let ns = DaemonNamespaceV1::new("wake-test", "target/p11", "daemon");
        let signal = wake_signal(
            ns.namespace_id,
            "filesystem",
            "README.md:modified",
            serde_json::json!({"path":"README.md"}),
            CanonicalToolSideEffectClass::ReadOnly,
        )
        .unwrap();
        assert!(signal.idempotency_key.contains("filesystem"));
        assert!(signal.idempotency_key.contains("README.md:modified"));
        assert!(signal.idempotency_key.contains(&signal.payload_digest));
    }

    #[test]
    fn empty_signal_key_is_rejected() {
        let ns = DaemonNamespaceV1::new("wake-test-empty", "target/p11", "daemon");
        let err = wake_signal(
            ns.namespace_id,
            "filesystem",
            "",
            serde_json::json!({}),
            CanonicalToolSideEffectClass::ReadOnly,
        )
        .unwrap_err();
        assert!(matches!(err, WakeError::EmptySignalKey));
    }
}
