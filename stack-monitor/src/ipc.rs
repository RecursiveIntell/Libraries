//! Unix-domain-socket transport for cross-process observations.
//!
//! This is the first cross-process slice. Frames are bounded, length-delimited
//! JSON envelopes. The collector still owns normalized SQLite writes through
//! the existing bounded `MonitorClient` queue.

#![cfg(unix)]

use crate::{
    start_collector_with_live, CollectorHandle, EmitStatus, LiveHub, MonitorClient, TransportStats,
};
use serde_json::from_slice;
use stack_observation::{ObservationEnvelope, ObservationError, MAX_PAYLOAD_BYTES};
use std::fs;
use std::io::{ErrorKind, Read, Write};
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::mpsc::{sync_channel, Receiver, SyncSender, TrySendError};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;
use thiserror::Error;

const FRAME_HEADER_BYTES: usize = 4;
const MAX_FRAME_BYTES: usize = MAX_PAYLOAD_BYTES + 4096;

/// IPC setup or frame error.
#[derive(Debug, Error)]
pub enum IpcError {
    #[error("Unix socket I/O failed: {0}")]
    Io(#[from] std::io::Error),
    #[error("observation serialization failed: {0}")]
    Json(#[from] serde_json::Error),
    #[error("observation contract failed: {0}")]
    Observation(#[from] ObservationError),
}

/// Cross-process producer counters.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct IpcStats {
    pub attempted: u64,
    pub accepted: u64,
    pub sent: u64,
    pub dropped: u64,
    pub connection_failures: u64,
}

#[derive(Default)]
struct IpcCounters {
    attempted: AtomicU64,
    accepted: AtomicU64,
    sent: AtomicU64,
    dropped: AtomicU64,
    connection_failures: AtomicU64,
}

impl IpcCounters {
    fn snapshot(&self) -> IpcStats {
        IpcStats {
            attempted: self.attempted.load(Ordering::Relaxed),
            accepted: self.accepted.load(Ordering::Relaxed),
            sent: self.sent.load(Ordering::Relaxed),
            dropped: self.dropped.load(Ordering::Relaxed),
            connection_failures: self.connection_failures.load(Ordering::Relaxed),
        }
    }
}

/// Bounded cross-process producer. `try_emit` never connects or writes inline.
#[derive(Clone)]
pub struct UnixMonitorClient {
    tx: SyncSender<ObservationEnvelope>,
    counters: Arc<IpcCounters>,
}

impl UnixMonitorClient {
    /// Enqueue an observation for the IPC sender without blocking.
    pub fn try_emit(&self, event: ObservationEnvelope) -> Result<EmitStatus, ObservationError> {
        self.counters.attempted.fetch_add(1, Ordering::Relaxed);
        event.validate()?;
        match self.tx.try_send(event) {
            Ok(()) => {
                self.counters.accepted.fetch_add(1, Ordering::Relaxed);
                Ok(EmitStatus::Accepted)
            }
            Err(TrySendError::Full(_)) | Err(TrySendError::Disconnected(_)) => {
                self.counters.dropped.fetch_add(1, Ordering::Relaxed);
                Ok(EmitStatus::Dropped)
            }
        }
    }

    /// Snapshot IPC producer counters.
    pub fn stats(&self) -> IpcStats {
        self.counters.snapshot()
    }
}

impl crate::ObservationEmitter for UnixMonitorClient {
    fn emit_observation(&self, event: ObservationEnvelope) -> Result<EmitStatus, ObservationError> {
        self.try_emit(event)
    }
}

/// Shutdown handle for a Unix IPC producer.
pub struct UnixMonitorClientHandle {
    stop: Arc<AtomicBool>,
    join: Option<JoinHandle<()>>,
    counters: Arc<IpcCounters>,
}

impl UnixMonitorClientHandle {
    /// Stop the sender after draining events already accepted by its queue.
    pub fn shutdown(mut self) -> IpcStats {
        self.stop.store(true, Ordering::Release);
        if let Some(join) = self.join.take() {
            let _ = join.join();
        }
        self.counters.snapshot()
    }

    /// Snapshot counters without stopping the sender.
    pub fn stats(&self) -> IpcStats {
        self.counters.snapshot()
    }
}

impl Drop for UnixMonitorClientHandle {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
    }
}

/// Start a bounded Unix-socket producer.
pub fn start_unix_client(
    path: impl AsRef<Path>,
    capacity: usize,
) -> Result<(UnixMonitorClient, UnixMonitorClientHandle), IpcError> {
    let path = path.as_ref().to_path_buf();
    let (tx, rx) = sync_channel(capacity.max(1));
    let counters = Arc::new(IpcCounters::default());
    let worker_counters = Arc::clone(&counters);
    let stop = Arc::new(AtomicBool::new(false));
    let worker_stop = Arc::clone(&stop);
    let join = thread::Builder::new()
        .name("stack-monitor-unix-producer".into())
        .spawn(move || producer_loop(path, rx, worker_stop, worker_counters))?;
    Ok((
        UnixMonitorClient {
            tx,
            counters: Arc::clone(&counters),
        },
        UnixMonitorClientHandle {
            stop,
            join: Some(join),
            counters,
        },
    ))
}

fn producer_loop(
    path: PathBuf,
    rx: Receiver<ObservationEnvelope>,
    stop: Arc<AtomicBool>,
    counters: Arc<IpcCounters>,
) {
    let mut stream: Option<UnixStream> = None;
    while !stop.load(Ordering::Acquire) {
        match rx.recv_timeout(Duration::from_millis(25)) {
            Ok(event) => {
                if stream.is_none() {
                    stream = connect(&path, &counters);
                }
                if let Some(current) = stream.as_mut() {
                    match write_frame(current, &event) {
                        Ok(()) => {
                            counters.sent.fetch_add(1, Ordering::Relaxed);
                        }
                        Err(_) => {
                            stream = None;
                            counters.connection_failures.fetch_add(1, Ordering::Relaxed);
                            counters.dropped.fetch_add(1, Ordering::Relaxed);
                        }
                    };
                } else {
                    counters.dropped.fetch_add(1, Ordering::Relaxed);
                }
            }
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {}
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => break,
        }
    }

    while let Ok(event) = rx.try_recv() {
        if stream.is_none() {
            stream = connect(&path, &counters);
        }
        if let Some(current) = stream.as_mut() {
            if write_frame(current, &event).is_ok() {
                counters.sent.fetch_add(1, Ordering::Relaxed);
            } else {
                counters.connection_failures.fetch_add(1, Ordering::Relaxed);
                counters.dropped.fetch_add(1, Ordering::Relaxed);
                stream = None;
            }
        } else {
            counters.dropped.fetch_add(1, Ordering::Relaxed);
        }
    }
}

fn connect(path: &Path, counters: &IpcCounters) -> Option<UnixStream> {
    match UnixStream::connect(path) {
        Ok(stream) => Some(stream),
        Err(_) => {
            counters.connection_failures.fetch_add(1, Ordering::Relaxed);
            None
        }
    }
}

fn write_frame(stream: &mut UnixStream, event: &ObservationEnvelope) -> Result<(), IpcError> {
    let payload = serde_json::to_vec(event)?;
    if payload.len() > MAX_FRAME_BYTES {
        return Err(IpcError::Observation(ObservationError::PayloadTooLarge {
            actual: payload.len(),
            maximum: MAX_FRAME_BYTES,
        }));
    }
    let length = u32::try_from(payload.len()).map_err(|_| {
        IpcError::Io(std::io::Error::new(
            ErrorKind::InvalidData,
            "frame too large",
        ))
    })?;
    stream.write_all(&length.to_be_bytes())?;
    stream.write_all(&payload)?;
    stream.flush()?;
    Ok(())
}

/// Collector-side Unix socket server.
pub struct UnixCollectorHandle {
    stop: Arc<AtomicBool>,
    join: Option<JoinHandle<()>>,
    storage: Option<CollectorHandle>,
    path: PathBuf,
}

impl UnixCollectorHandle {
    /// Stop accepting connections, drain the storage collector, and remove the socket.
    pub fn shutdown(mut self) -> TransportStats {
        self.stop.store(true, Ordering::Release);
        if let Some(join) = self.join.take() {
            let _ = join.join();
        }
        let stats = self
            .storage
            .take()
            .map(CollectorHandle::shutdown)
            .unwrap_or_default();
        let _ = fs::remove_file(&self.path);
        stats
    }
}

impl Drop for UnixCollectorHandle {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        let _ = fs::remove_file(&self.path);
    }
}

/// Ensure the socket parent is private enough for local observation traffic.
fn prepare_socket_parent(path: &Path) -> Result<(), IpcError> {
    let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    else {
        return Ok(());
    };
    if !parent.exists() {
        fs::create_dir_all(parent)?;
        fs::set_permissions(parent, fs::Permissions::from_mode(0o700))?;
        return Ok(());
    }
    let mode = fs::metadata(parent)?.permissions().mode() & 0o777;
    if mode & 0o077 != 0 {
        return Err(IpcError::Io(std::io::Error::new(
            ErrorKind::PermissionDenied,
            format!(
                "socket parent {} is not private (mode {mode:o})",
                parent.display()
            ),
        )));
    }
    Ok(())
}

/// Bind a Unix collector socket and route frames through the bounded collector.
pub fn start_unix_collector(
    path: impl AsRef<Path>,
    store: crate::ActivityStore,
    capacity: usize,
) -> Result<UnixCollectorHandle, IpcError> {
    start_unix_collector_with_live(path, store, capacity, None)
}

/// Bind a Unix collector socket and optionally publish durable events to a live hub.
pub fn start_unix_collector_with_live(
    path: impl AsRef<Path>,
    store: crate::ActivityStore,
    capacity: usize,
    live: Option<Arc<LiveHub>>,
) -> Result<UnixCollectorHandle, IpcError> {
    let path = path.as_ref().to_path_buf();
    prepare_socket_parent(&path)?;
    if path.exists() {
        fs::remove_file(&path)?;
    }
    let listener = UnixListener::bind(&path)?;
    fs::set_permissions(&path, fs::Permissions::from_mode(0o600))?;
    listener.set_nonblocking(true)?;
    let (sink, storage) = start_collector_with_live(store, capacity, live);
    let stop = Arc::new(AtomicBool::new(false));
    let worker_stop = Arc::clone(&stop);
    let join = thread::Builder::new()
        .name("stack-monitor-unix-collector".into())
        .spawn(move || accept_loop(listener, sink, worker_stop))?;
    Ok(UnixCollectorHandle {
        stop,
        join: Some(join),
        storage: Some(storage),
        path,
    })
}

fn accept_loop(listener: UnixListener, sink: MonitorClient, stop: Arc<AtomicBool>) {
    while !stop.load(Ordering::Acquire) {
        match listener.accept() {
            Ok((stream, _)) => {
                let _ = stream.set_read_timeout(Some(Duration::from_millis(100)));
                let worker_sink = sink.clone();
                let worker_stop = Arc::clone(&stop);
                let _ = thread::Builder::new()
                    .name("stack-monitor-unix-connection".into())
                    .spawn(move || read_connection(stream, worker_sink, worker_stop));
            }
            Err(error) if error.kind() == ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(10));
            }
            Err(_) => break,
        }
    }
}

fn read_connection(mut stream: UnixStream, sink: MonitorClient, stop: Arc<AtomicBool>) {
    while !stop.load(Ordering::Acquire) {
        let mut header = [0u8; FRAME_HEADER_BYTES];
        if stream.read_exact(&mut header).is_err() {
            break;
        }
        let length = u32::from_be_bytes(header) as usize;
        if length == 0 || length > MAX_FRAME_BYTES {
            break;
        }
        let mut payload = vec![0u8; length];
        if stream.read_exact(&mut payload).is_err() {
            break;
        }
        let event: ObservationEnvelope = match from_slice(&payload) {
            Ok(event) => event,
            Err(_) => break,
        };
        if sink.try_emit(event).is_err() {
            break;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use stack_observation::{LifecycleStatus, ObservationKind};
    use std::thread;

    fn event() -> ObservationEnvelope {
        ObservationEnvelope::metadata(
            "ipc-test",
            "llm-pipeline",
            "ipc-adapter",
            1,
            ObservationKind::LlmCall,
            LifecycleStatus::Started,
            "ipc round trip",
        )
    }

    #[test]
    fn unix_socket_round_trip_reaches_collector_storage() {
        let _guard = crate::test_support::global_sink_guard();
        let dir = std::env::temp_dir().join(format!("stack-monitor-{}", uuid::Uuid::new_v4()));
        fs::create_dir(&dir).unwrap();
        fs::set_permissions(&dir, fs::Permissions::from_mode(0o700)).unwrap();
        let path = dir.join("collector.sock");
        let store = crate::ActivityStore::open(":memory:").unwrap();
        let collector = start_unix_collector(&path, store.clone(), 8).unwrap();
        let (client, client_handle) = start_unix_client(&path, 8).unwrap();
        assert_eq!(client.try_emit(event()).unwrap(), EmitStatus::Accepted);
        let client_stats = client_handle.shutdown();
        assert_eq!(client_stats.sent, 1);

        for _ in 0..50 {
            if store.observation_count_for_producer("ipc-test").unwrap() == 1 {
                break;
            }
            thread::sleep(Duration::from_millis(10));
        }
        let collector_stats = collector.shutdown();
        assert_eq!(collector_stats.persisted, 1);
        assert_eq!(store.observation_count_for_producer("ipc-test").unwrap(), 1);
        assert!(!path.exists());
        fs::remove_dir(&dir).unwrap();
    }

    #[test]
    fn malformed_frame_does_not_kill_collector() {
        let _guard = crate::test_support::global_sink_guard();
        let dir =
            std::env::temp_dir().join(format!("stack-monitor-malformed-{}", uuid::Uuid::new_v4()));
        fs::create_dir(&dir).unwrap();
        fs::set_permissions(&dir, fs::Permissions::from_mode(0o700)).unwrap();
        let path = dir.join("collector.sock");
        let store = crate::ActivityStore::open(":memory:").unwrap();
        let collector = start_unix_collector(&path, store.clone(), 8).unwrap();
        let mut malformed = UnixStream::connect(&path).unwrap();
        malformed
            .write_all(&((MAX_FRAME_BYTES as u32) + 1).to_be_bytes())
            .unwrap();
        drop(malformed);
        let (client, client_handle) = start_unix_client(&path, 8).unwrap();
        assert_eq!(client.try_emit(event()).unwrap(), EmitStatus::Accepted);
        assert_eq!(client_handle.shutdown().sent, 1);
        for _ in 0..50 {
            if store.observation_count_for_producer("ipc-test").unwrap() == 1 {
                break;
            }
            thread::sleep(Duration::from_millis(10));
        }
        assert_eq!(collector.shutdown().persisted, 1);
        assert_eq!(store.observation_count_for_producer("ipc-test").unwrap(), 1);
        assert!(!path.exists());
        fs::remove_dir(&dir).unwrap();
    }
}
