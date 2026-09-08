use crate::frame::FrameV1;
use crate::transport::{send_authenticated_frame, TransportError};
use tokio::net::TcpStream;

pub async fn connect_and_send(address: &str, frame: &FrameV1) -> Result<(), TransportError> {
    let mut stream = TcpStream::connect(address)
        .await
        .map_err(|error| TransportError::Frame(crate::FrameError::Io(error.to_string())))?;
    send_authenticated_frame(&mut stream, frame).await
}
