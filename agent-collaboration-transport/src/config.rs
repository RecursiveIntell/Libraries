use serde::{Deserialize, Serialize};
use stack_ids::{AgentId, ContentDigest};
use std::collections::BTreeMap;
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("configuration I/O error: {0}")]
    Io(String),
    #[error("configuration parse error: {0}")]
    Parse(String),
    #[error("configuration validation error: {0}")]
    Invalid(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PeerConfigV1 {
    pub protocol_version: String,
    pub agent_id: AgentId,
    pub bind_addr: String,
    pub key_file: String,
    pub capability_manifest_digest: ContentDigest,
    pub peers: BTreeMap<String, PeerEndpointV1>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PeerEndpointV1 {
    pub agent_id: AgentId,
    pub address: String,
    pub key_file: String,
}

impl PeerConfigV1 {
    pub fn from_path(path: &Path) -> Result<Self, ConfigError> {
        let bytes = std::fs::read(path).map_err(|error| ConfigError::Io(error.to_string()))?;
        let text =
            std::str::from_utf8(&bytes).map_err(|error| ConfigError::Parse(error.to_string()))?;
        let config: Self =
            toml::from_str(text).map_err(|error| ConfigError::Parse(error.to_string()))?;
        config.validate()?;
        Ok(config)
    }

    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.protocol_version != crate::PROTOCOL_VERSION {
            return Err(ConfigError::Invalid("unsupported protocol version".into()));
        }
        if self.bind_addr.trim().is_empty() || self.key_file.trim().is_empty() {
            return Err(ConfigError::Invalid(
                "bind_addr and key_file are required".into(),
            ));
        }
        if self.peers.is_empty() {
            return Err(ConfigError::Invalid(
                "at least one pinned peer is required".into(),
            ));
        }
        for (name, peer) in &self.peers {
            if name.trim().is_empty()
                || peer.address.trim().is_empty()
                || peer.key_file.trim().is_empty()
            {
                return Err(ConfigError::Invalid(
                    "peer names, addresses, and key files are required".into(),
                ));
            }
        }
        Ok(())
    }
}
