use crate::frame::{read_frame, write_frame, FrameError, FrameV1};
use crate::identity::{IdentityError, StaticIdentity};
use crate::replay::ReplayCache;
use chrono::Utc;
use stack_ids::ContentDigest;
use thiserror::Error;
use tokio::io::{AsyncRead, AsyncWrite};

#[derive(Debug, Error)]
pub enum TransportError {
    #[error(transparent)]
    Frame(#[from] FrameError),
    #[error(transparent)]
    Identity(#[from] IdentityError),
    #[error("replayed frame nonce")]
    Replay,
}

pub async fn send_authenticated_frame<W: AsyncWrite + Unpin>(
    writer: &mut W,
    frame: &FrameV1,
) -> Result<(), TransportError> {
    write_frame(writer, frame)
        .await
        .map_err(TransportError::from)
}

pub async fn receive_authenticated_frame<R: AsyncRead + Unpin>(
    reader: &mut R,
    local: &StaticIdentity,
    peer: &StaticIdentity,
    expected_capability_manifest_digest: &ContentDigest,
    replay: &ReplayCache,
) -> Result<FrameV1, TransportError> {
    let frame = read_frame(reader).await?;
    local.verify_frame(
        &frame,
        peer,
        expected_capability_manifest_digest,
        Utc::now(),
    )?;
    if !replay.admit(&frame.nonce) {
        return Err(TransportError::Replay);
    }
    Ok(frame)
}
