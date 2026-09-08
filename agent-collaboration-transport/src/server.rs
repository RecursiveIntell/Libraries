use crate::identity::StaticIdentity;
use crate::replay::ReplayCache;
use crate::transport::{receive_authenticated_frame, TransportError};
use stack_ids::ContentDigest;
use tokio::net::TcpListener;

pub async fn accept_one(
    listener: &TcpListener,
    local: &StaticIdentity,
    expected_peer: &StaticIdentity,
    expected_capability_manifest_digest: &ContentDigest,
    replay: &ReplayCache,
) -> Result<crate::FrameV1, TransportError> {
    let (mut stream, _) = listener
        .accept()
        .await
        .map_err(|error| TransportError::Frame(crate::FrameError::Io(error.to_string())))?;
    receive_authenticated_frame(
        &mut stream,
        local,
        expected_peer,
        expected_capability_manifest_digest,
        replay,
    )
    .await
}
