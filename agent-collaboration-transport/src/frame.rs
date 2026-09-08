use serde::{de::DeserializeOwned, Deserialize, Serialize};
use stack_ids::{AgentId, ContentDigest};
use thiserror::Error;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

pub const PROTOCOL_VERSION: &str = "agent_collaboration_transport_v1";
pub const MAX_FRAME_BYTES: usize = 1_048_576;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FrameV1 {
    pub protocol_version: String,
    pub sender_agent_id: AgentId,
    pub receiver_agent_id: AgentId,
    pub nonce: String,
    pub expires_at: String,
    pub capability_manifest_digest: ContentDigest,
    pub payload: Vec<u8>,
    pub auth_tag: Vec<u8>,
}

#[derive(Debug, Error)]
pub enum FrameError {
    #[error("frame exceeds maximum size")]
    TooLarge,
    #[error("frame length prefix is invalid")]
    InvalidLength,
    #[error("frame I/O error: {0}")]
    Io(String),
    #[error("frame serialization error: {0}")]
    Serialization(String),
}

pub async fn write_frame<W: AsyncWrite + Unpin>(
    writer: &mut W,
    frame: &FrameV1,
) -> Result<(), FrameError> {
    let bytes =
        serde_json::to_vec(frame).map_err(|error| FrameError::Serialization(error.to_string()))?;
    if bytes.len() > MAX_FRAME_BYTES {
        return Err(FrameError::TooLarge);
    }
    let length = u32::try_from(bytes.len()).map_err(|_| FrameError::TooLarge)?;
    writer
        .write_u32(length)
        .await
        .map_err(|error| FrameError::Io(error.to_string()))?;
    writer
        .write_all(&bytes)
        .await
        .map_err(|error| FrameError::Io(error.to_string()))?;
    writer
        .flush()
        .await
        .map_err(|error| FrameError::Io(error.to_string()))
}

pub async fn read_frame<R: AsyncRead + Unpin>(reader: &mut R) -> Result<FrameV1, FrameError> {
    let length = reader
        .read_u32()
        .await
        .map_err(|error| FrameError::Io(error.to_string()))?;
    let length = usize::try_from(length).map_err(|_| FrameError::InvalidLength)?;
    if length == 0 || length > MAX_FRAME_BYTES {
        return Err(FrameError::TooLarge);
    }
    let mut bytes = vec![0_u8; length];
    reader
        .read_exact(&mut bytes)
        .await
        .map_err(|error| FrameError::Io(error.to_string()))?;
    serde_json::from_slice(&bytes).map_err(|error| FrameError::Serialization(error.to_string()))
}

pub(crate) fn canonical_json<T: Serialize>(value: &T) -> Result<Vec<u8>, FrameError> {
    let value = serde_json::to_value(value)
        .map_err(|error| FrameError::Serialization(error.to_string()))?;
    boundary_compiler::Canonicalizer::new()
        .canonicalize_bytes(&value)
        .map_err(|error| FrameError::Serialization(error.to_string()))
}

#[allow(dead_code)]
fn _assert_deserializable<T: DeserializeOwned>() {}
