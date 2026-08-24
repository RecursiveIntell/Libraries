//! Read-only Unix socket for cross-process live observation events.

#![cfg(unix)]

use crate::{LiveEvent, LiveHub};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::fs;
use std::io::{ErrorKind, Read, Write};
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{sync_channel, Receiver, SyncSender, TryRecvError};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;
use thiserror::Error;

const MAX_LIVE_FRAME_BYTES: usize = 68 * 1024;

/// Read-only live IPC errors.
#[derive(Debug, Error)]
pub enum LiveIpcError {
    #[error("live socket I/O failed: {0}")]
    Io(#[from] std::io::Error),
    #[error("live event serialization failed: {0}")]
    Json(#[from] serde_json::Error),
}

/// A bounded live client subscription from another process.
pub struct UnixLiveSubscription {
    receiver: Receiver<LiveEvent>,
}

impl UnixLiveSubscription {
    /// Try to receive without blocking.
    pub fn try_recv(&self) -> Result<LiveEvent, TryRecvError> {
        self.receiver.try_recv()
    }

    /// Receive with a bounded timeout.
    pub fn recv_timeout(
        &self,
        timeout: Duration,
    ) -> Result<LiveEvent, std::sync::mpsc::RecvTimeoutError> {
        self.receiver.recv_timeout(timeout)
    }
}

/// Shutdown handle for a live client.
pub struct UnixLiveClientHandle {
    stop: Arc<AtomicBool>,
    join: Option<JoinHandle<()>>,
}

impl UnixLiveClientHandle {
    /// Stop the live client reader.
    pub fn shutdown(mut self) {
        self.stop.store(true, Ordering::Release);
        if let Some(join) = self.join.take() {
            let _ = join.join();
        }
    }
}

impl Drop for UnixLiveClientHandle {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
    }
}

/// Start a read-only live client.
pub fn start_unix_live_client(
    path: impl AsRef<Path>,
    capacity: usize,
) -> Result<(UnixLiveSubscription, UnixLiveClientHandle), LiveIpcError> {
    let path = path.as_ref().to_path_buf();
    let (tx, rx) = sync_channel(capacity.max(1));
    let stop = Arc::new(AtomicBool::new(false));
    let worker_stop = Arc::clone(&stop);
    let join = thread::Builder::new()
        .name("stack-monitor-live-client".into())
        .spawn(move || live_client_loop(path, tx, worker_stop))?;
    Ok((
        UnixLiveSubscription { receiver: rx },
        UnixLiveClientHandle {
            stop,
            join: Some(join),
        },
    ))
}

fn live_client_loop(path: PathBuf, tx: SyncSender<LiveEvent>, stop: Arc<AtomicBool>) {
    let mut stream = loop {
        if stop.load(Ordering::Acquire) {
            return;
        }
        match UnixStream::connect(&path) {
            Ok(stream) => break stream,
            Err(_) => thread::sleep(Duration::from_millis(100)),
        }
    };
    let _ = stream.set_read_timeout(Some(Duration::from_millis(100)));
    while !stop.load(Ordering::Acquire) {
        match read_frame::<LiveEvent>(&mut stream) {
            Ok(event) => {
                if tx.try_send(event).is_err() {
                    // The bounded UI queue is full. The next cursor exposes the gap.
                    break;
                }
            }
            Err(LiveIpcError::Io(error))
                if matches!(error.kind(), ErrorKind::TimedOut | ErrorKind::WouldBlock) => {}
            Err(_) => break,
        }
    }
}

/// Shutdown handle for a live collector socket.
pub struct UnixLiveServerHandle {
    stop: Arc<AtomicBool>,
    join: Option<JoinHandle<()>>,
    path: PathBuf,
}

impl UnixLiveServerHandle {
    /// Stop the live socket and remove its path.
    pub fn shutdown(mut self) {
        self.stop.store(true, Ordering::Release);
        if let Some(join) = self.join.take() {
            let _ = join.join();
        }
        let _ = fs::remove_file(&self.path);
    }
}

impl Drop for UnixLiveServerHandle {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        let _ = fs::remove_file(&self.path);
    }
}

/// Start a read-only live socket backed by a collector hub.
pub fn start_unix_live_server(
    path: impl AsRef<Path>,
    hub: Arc<LiveHub>,
) -> Result<UnixLiveServerHandle, LiveIpcError> {
    let path = path.as_ref().to_path_buf();
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)?;
            fs::set_permissions(parent, fs::Permissions::from_mode(0o700))?;
        }
    }
    if path.exists() {
        fs::remove_file(&path)?;
    }
    let listener = UnixListener::bind(&path)?;
    fs::set_permissions(&path, fs::Permissions::from_mode(0o600))?;
    listener.set_nonblocking(true)?;
    let stop = Arc::new(AtomicBool::new(false));
    let worker_stop = Arc::clone(&stop);
    let join = thread::Builder::new()
        .name("stack-monitor-live-server".into())
        .spawn(move || live_server_loop(listener, hub, worker_stop))?;
    Ok(UnixLiveServerHandle {
        stop,
        join: Some(join),
        path,
    })
}

fn live_server_loop(listener: UnixListener, hub: Arc<LiveHub>, stop: Arc<AtomicBool>) {
    while !stop.load(Ordering::Acquire) {
        match listener.accept() {
            Ok((stream, _)) => {
                let worker_hub = Arc::clone(&hub);
                let worker_stop = Arc::clone(&stop);
                let _ = thread::Builder::new()
                    .name("stack-monitor-live-connection".into())
                    .spawn(move || live_connection_loop(stream, worker_hub, worker_stop));
            }
            Err(error) if error.kind() == ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(10));
            }
            Err(_) => break,
        }
    }
}

fn live_connection_loop(mut stream: UnixStream, hub: Arc<LiveHub>, stop: Arc<AtomicBool>) {
    let mut subscription = hub.subscribe();
    while !stop.load(Ordering::Acquire) {
        match subscription.try_recv() {
            Ok(event) => {
                if write_frame(&mut stream, &event).is_err() {
                    break;
                }
            }
            Err(crate::LiveReceive::Empty) => thread::sleep(Duration::from_millis(10)),
            Err(crate::LiveReceive::Lagged(_)) => {
                // Cursor discontinuity is preserved by subsequent LiveEvent cursors.
            }
            Err(crate::LiveReceive::Closed) => break,
        }
    }
}

fn write_frame<T: Serialize>(stream: &mut UnixStream, value: &T) -> Result<(), LiveIpcError> {
    let payload = serde_json::to_vec(value)?;
    if payload.len() > MAX_LIVE_FRAME_BYTES {
        return Err(LiveIpcError::Io(std::io::Error::new(
            ErrorKind::InvalidData,
            "live frame too large",
        )));
    }
    let length = u32::try_from(payload.len()).map_err(|_| {
        LiveIpcError::Io(std::io::Error::new(
            ErrorKind::InvalidData,
            "live frame too large",
        ))
    })?;
    stream.write_all(&length.to_be_bytes())?;
    stream.write_all(&payload)?;
    stream.flush()?;
    Ok(())
}

fn read_frame<T: DeserializeOwned>(stream: &mut UnixStream) -> Result<T, LiveIpcError> {
    let mut header = [0u8; 4];
    stream.read_exact(&mut header)?;
    let length = u32::from_be_bytes(header) as usize;
    if length == 0 || length > MAX_LIVE_FRAME_BYTES {
        return Err(LiveIpcError::Io(std::io::Error::new(
            ErrorKind::InvalidData,
            "invalid live frame length",
        )));
    }
    let mut payload = vec![0u8; length];
    stream.read_exact(&mut payload)?;
    Ok(serde_json::from_slice(&payload)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use stack_observation::{LifecycleStatus, ObservationEnvelope, ObservationKind};
    use std::os::unix::fs::PermissionsExt;

    #[test]
    fn live_socket_round_trip_delivers_cursor_event() {
        let dir = std::env::temp_dir().join(format!("stack-monitor-live-{}", uuid::Uuid::new_v4()));
        fs::create_dir(&dir).unwrap();
        fs::set_permissions(&dir, fs::Permissions::from_mode(0o700)).unwrap();
        let path = dir.join("live.sock");
        let hub = Arc::new(LiveHub::new(8));
        let server = start_unix_live_server(&path, Arc::clone(&hub)).unwrap();
        let (subscription, client) = start_unix_live_client(&path, 8).unwrap();
        thread::sleep(Duration::from_millis(100));
        let mut received = None;
        for sequence in 1..=20 {
            hub.publish(ObservationEnvelope::metadata(
                "live-ipc-test",
                "llm-pipeline",
                "live-adapter",
                sequence,
                ObservationKind::LlmCall,
                LifecycleStatus::Completed,
                "live socket",
            ));
            match subscription.recv_timeout(Duration::from_millis(100)) {
                Ok(event) => {
                    received = Some(event);
                    break;
                }
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {}
                Err(error) => panic!("live subscription disconnected: {error}"),
            }
        }
        let event = received.expect("live event delivered within retry window");
        assert!(event.cursor >= 1);
        assert_eq!(event.observation.producer_id, "live-ipc-test");
        client.shutdown();
        server.shutdown();
        assert!(!path.exists());
        fs::remove_dir(&dir).unwrap();
    }
}
