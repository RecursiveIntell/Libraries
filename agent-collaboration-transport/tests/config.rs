#![allow(clippy::expect_used)]

use agent_collaboration_transport::{ConfigError, PeerConfigV1, PROTOCOL_VERSION};
use tempfile::tempdir;

const CONFIG: &str = r#"
protocol_version = "agent_collaboration_transport_v1"
agent_id = "controller"
bind_addr = "127.0.0.1:43100"
key_file = "/private/controller.key"
capability_manifest_digest = "0000000000000000000000000000000000000000000000000000000000000000"

[peers.worker]
agent_id = "worker"
address = "127.0.0.1:43101"
key_file = "/private/worker.key"
"#;

#[test]
fn strict_peer_config_roundtrips_and_rejects_unknown_fields() {
    let temp = tempdir().expect("temporary config directory");
    let path = temp.path().join("peer.toml");
    std::fs::write(&path, CONFIG).expect("config");
    let config = PeerConfigV1::from_path(&path).expect("valid config");
    assert_eq!(config.protocol_version, PROTOCOL_VERSION);
    assert_eq!(config.peers.len(), 1);
    assert!(config.validate().is_ok());

    std::fs::write(&path, format!("{CONFIG}\nunknown = true\n")).expect("invalid config");
    assert!(matches!(
        PeerConfigV1::from_path(&path),
        Err(ConfigError::Parse(_))
    ));
}
