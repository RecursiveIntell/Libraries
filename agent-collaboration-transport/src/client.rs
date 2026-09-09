use crate::frame::FrameV1;
use crate::identity::StaticIdentity;
use crate::replay::ReplayCache;
use crate::transport::{receive_authenticated_frame, send_authenticated_frame, TransportError};
use stack_ids::ContentDigest;
use tokio::net::TcpStream;

pub async fn connect_and_send(address: &str, frame: &FrameV1) -> Result<(), TransportError> {
    let mut stream = TcpStream::connect(address)
        .await
        .map_err(|error| TransportError::Frame(crate::FrameError::Io(error.to_string())))?;
    send_authenticated_frame(&mut stream, frame).await
}

pub async fn connect_and_exchange(
    address: &str,
    local: &StaticIdentity,
    peer: &StaticIdentity,
    frame: &FrameV1,
    expected_capability_manifest_digest: &ContentDigest,
    replay: &ReplayCache,
) -> Result<FrameV1, TransportError> {
    let mut stream = TcpStream::connect(address)
        .await
        .map_err(|error| TransportError::Frame(crate::FrameError::Io(error.to_string())))?;
    send_authenticated_frame(&mut stream, frame).await?;
    receive_authenticated_frame(
        &mut stream,
        local,
        peer,
        expected_capability_manifest_digest,
        replay,
    )
    .await
}
