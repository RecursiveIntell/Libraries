use crate::frame::{canonical_json, FrameError, FrameV1, PROTOCOL_VERSION};
use chrono::{DateTime, Utc};
use hmac::{Hmac, Mac};
use stack_ids::{AgentId, ContentDigest};
use thiserror::Error;
use uuid::Uuid;

type HmacSha256 = Hmac<sha2::Sha256>;

#[derive(Debug, Error)]
pub enum IdentityError {
    #[error("identity key must contain at least 32 bytes")]
    WeakKey,
    #[error("frame identity validation failed: {0}")]
    Invalid(String),
    #[error("frame authentication failed")]
    AuthenticationFailed,
    #[error("frame expired or has invalid expiry")]
    Expired,
    #[error("frame serialization failed: {0}")]
    Serialization(String),
}

#[derive(Clone)]
pub struct StaticIdentity {
    pub agent_id: AgentId,
    key: Vec<u8>,
}

impl StaticIdentity {
    pub fn new(agent_id: AgentId, key: impl Into<Vec<u8>>) -> Result<Self, IdentityError> {
        let key = key.into();
        if key.len() < 32 {
            return Err(IdentityError::WeakKey);
        }
        Ok(Self { agent_id, key })
    }

    pub fn issue_frame(
        &self,
        receiver_agent_id: AgentId,
        capability_manifest_digest: ContentDigest,
        expires_at: String,
        payload: Vec<u8>,
    ) -> Result<FrameV1, IdentityError> {
        let mut frame = FrameV1 {
            protocol_version: PROTOCOL_VERSION.into(),
            sender_agent_id: self.agent_id.clone(),
            receiver_agent_id,
            nonce: Uuid::new_v4().to_string(),
            expires_at,
            capability_manifest_digest,
            payload,
            auth_tag: Vec::new(),
        };
        frame.auth_tag = self.sign(&frame)?;
        Ok(frame)
    }

    pub fn verify_frame(
        &self,
        frame: &FrameV1,
        expected_peer: &StaticIdentity,
        expected_capability_manifest_digest: &ContentDigest,
        now: DateTime<Utc>,
    ) -> Result<(), IdentityError> {
        if frame.protocol_version != PROTOCOL_VERSION {
            return Err(IdentityError::Invalid(
                "unsupported protocol version".into(),
            ));
        }
        if frame.sender_agent_id != expected_peer.agent_id {
            return Err(IdentityError::Invalid("unexpected sender identity".into()));
        }
        if frame.receiver_agent_id != self.agent_id {
            return Err(IdentityError::Invalid("wrong receiver audience".into()));
        }
        if &frame.capability_manifest_digest != expected_capability_manifest_digest {
            return Err(IdentityError::Invalid(
                "capability manifest digest is not admitted".into(),
            ));
        }
        let expiry = DateTime::parse_from_rfc3339(&frame.expires_at)
            .map(|value| value.with_timezone(&Utc))
            .map_err(|_| IdentityError::Expired)?;
        if expiry <= now {
            return Err(IdentityError::Expired);
        }
        let mut unsigned = frame.clone();
        unsigned.auth_tag.clear();
        let bytes = canonical_json(&unsigned)
            .map_err(|error| IdentityError::Serialization(error.to_string()))?;
        let mut mac = HmacSha256::new_from_slice(&expected_peer.key)
            .map_err(|error| IdentityError::Serialization(error.to_string()))?;
        mac.update(&bytes);
        mac.verify_slice(&frame.auth_tag)
            .map_err(|_| IdentityError::AuthenticationFailed)?;
        Ok(())
    }

    fn sign(&self, frame: &FrameV1) -> Result<Vec<u8>, IdentityError> {
        let mut unsigned = frame.clone();
        unsigned.auth_tag.clear();
        let bytes = canonical_json(&unsigned)
            .map_err(|error| IdentityError::Serialization(error.to_string()))?;
        let mut mac = HmacSha256::new_from_slice(&self.key)
            .map_err(|error| IdentityError::Serialization(error.to_string()))?;
        mac.update(&bytes);
        Ok(mac.finalize().into_bytes().to_vec())
    }

    pub fn frame_bytes_without_auth(frame: &FrameV1) -> Result<Vec<u8>, FrameError> {
        let mut unsigned = frame.clone();
        unsigned.auth_tag.clear();
        canonical_json(&unsigned)
    }
}
