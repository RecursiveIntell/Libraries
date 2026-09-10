//! Bounded framing and stdio transport helpers for the thin proxy.
use std::fmt;
use std::io::{self, Read, Write};
use std::os::unix::net::UnixStream;
use std::path::Path;

use tokio::io::{
    AsyncBufRead, AsyncBufReadExt, AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, BufReader,
};
use tokio::net::UnixStream as TokioUnixStream;

pub const MAX_FRAME: usize = 1024 * 1024;

#[derive(Debug)]
pub enum ProxyError {
    DaemonUnavailable,
    FrameTooLarge,
    Io(io::Error),
    Protocol(String),
}

impl fmt::Display for ProxyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DaemonUnavailable => {
                write!(f, "DAEMON_UNAVAILABLE: durable daemon is not reachable")
            }
            Self::FrameTooLarge => write!(f, "FRAME_TOO_LARGE: protocol frame exceeds 1 MiB"),
            Self::Io(error) => write!(f, "proxy I/O failure: {error}"),
            Self::Protocol(error) => write!(f, "proxy protocol failure: {error}"),
        }
    }
}

impl std::error::Error for ProxyError {}

impl From<io::Error> for ProxyError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

pub fn connect(path: &Path) -> Result<UnixStream, ProxyError> {
    UnixStream::connect(path).map_err(|_| ProxyError::DaemonUnavailable)
}

pub async fn connect_async(path: &Path) -> Result<TokioUnixStream, ProxyError> {
    TokioUnixStream::connect(path)
        .await
        .map_err(|_| ProxyError::DaemonUnavailable)
}

pub fn read_frame<R: Read>(reader: &mut R) -> Result<Vec<u8>, ProxyError> {
    let mut length = [0u8; 4];
    reader.read_exact(&mut length)?;
    let length = u32::from_be_bytes(length) as usize;
    if length > MAX_FRAME {
        return Err(ProxyError::FrameTooLarge);
    }
    let mut payload = vec![0; length];
    reader.read_exact(&mut payload)?;
    Ok(payload)
}

pub fn write_frame<W: Write>(writer: &mut W, payload: &[u8]) -> Result<(), ProxyError> {
    if payload.len() > MAX_FRAME {
        return Err(ProxyError::FrameTooLarge);
    }
    writer.write_all(&(payload.len() as u32).to_be_bytes())?;
    writer.write_all(payload)?;
    writer.flush()?;
    Ok(())
}

pub async fn read_frame_async<R: AsyncRead + Unpin>(reader: &mut R) -> Result<Vec<u8>, ProxyError> {
    let length = reader.read_u32().await? as usize;
    if length > MAX_FRAME {
        return Err(ProxyError::FrameTooLarge);
    }
    let mut payload = vec![0; length];
    reader.read_exact(&mut payload).await?;
    Ok(payload)
}

pub async fn write_frame_async<W: AsyncWrite + Unpin>(
    writer: &mut W,
    payload: &[u8],
) -> Result<(), ProxyError> {
    if payload.len() > MAX_FRAME {
        return Err(ProxyError::FrameTooLarge);
    }
    writer.write_u32(payload.len() as u32).await?;
    writer.write_all(payload).await?;
    writer.flush().await?;
    Ok(())
}

/// Read one newline-delimited MCP message without allowing an unbounded line.
pub async fn read_bounded_line<R: AsyncBufRead + Unpin>(
    reader: &mut R,
) -> Result<Option<Vec<u8>>, ProxyError> {
    let mut line = Vec::new();
    loop {
        let chunk = reader.fill_buf().await?;
        if chunk.is_empty() {
            if line.is_empty() {
                return Ok(None);
            }
            break;
        }
        if let Some(newline) = chunk.iter().position(|byte| *byte == b'\n') {
            if line.len() + newline > MAX_FRAME {
                return Err(ProxyError::FrameTooLarge);
            }
            line.extend_from_slice(&chunk[..newline]);
            reader.consume(newline + 1);
            break;
        }
        if line.len() + chunk.len() > MAX_FRAME {
            return Err(ProxyError::FrameTooLarge);
        }
        line.extend_from_slice(chunk);
        let consumed = chunk.len();
        reader.consume(consumed);
    }
    if line.last() == Some(&b'\r') {
        line.pop();
    }
    Ok(Some(line))
}

/// Forward newline-delimited MCP messages from this process to the daemon.
pub async fn run_stdio_proxy(socket_path: &Path) -> Result<(), ProxyError> {
    let socket = connect_async(socket_path).await?;
    let (mut socket_reader, mut socket_writer) = socket.into_split();
    let stdin = tokio::io::stdin();
    let mut input = BufReader::new(stdin);
    let mut stdout = tokio::io::stdout();

    while let Some(line) = read_bounded_line(&mut input).await? {
        if line.is_empty() {
            continue;
        }
        let expects_response = serde_json::from_slice::<serde_json::Value>(&line)
            .map(|value| value.get("id").is_some())
            .unwrap_or(true);
        write_frame_async(&mut socket_writer, &line).await?;
        if expects_response {
            let response = read_frame_async(&mut socket_reader).await?;
            stdout.write_all(&response).await?;
            stdout.write_all(b"\n").await?;
            stdout.flush().await?;
        }
    }
    Ok(())
}
