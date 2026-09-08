//! Lock-owning daemon primitives and durable lifecycle records.
use crate::{migrations, owner_lock::OwnerLock};
use rusqlite::{params, Connection};
use sha2::{Digest, Sha256};
use std::fmt;
use std::fs;
use std::io;
use std::os::unix::fs::{FileTypeExt, PermissionsExt};
use std::path::{Path, PathBuf};
use tokio::net::UnixListener;

pub const MAX_FRAME: usize = 1024 * 1024;

#[derive(Debug)]
pub enum DaemonError {
    AlreadyOwned,
    Io(io::Error),
    Sql(rusqlite::Error),
}

impl fmt::Display for DaemonError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AlreadyOwned => write!(
                formatter,
                "DATA_DIR_ALREADY_OWNED: another process owns this data directory"
            ),
            Self::Io(error) => write!(formatter, "DAEMON_IO: {error}"),
            Self::Sql(error) => write!(formatter, "DAEMON_SQL: {error}"),
        }
    }
}

impl std::error::Error for DaemonError {}

impl From<io::Error> for DaemonError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<rusqlite::Error> for DaemonError {
    fn from(error: rusqlite::Error) -> Self {
        Self::Sql(error)
    }
}

impl DaemonError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::AlreadyOwned => "DATA_DIR_ALREADY_OWNED",
            Self::Io(_) => "DAEMON_IO",
            Self::Sql(_) => "DAEMON_SQL",
        }
    }
}

/// A process-lifetime exclusive lock on the canonical data directory.
#[derive(Debug)]
pub struct DaemonLock {
    _inner: OwnerLock,
    pub path: PathBuf,
}

impl DaemonLock {
    pub fn acquire(data_dir: &Path) -> Result<Self, DaemonError> {
        let inner = OwnerLock::acquire(data_dir).map_err(|error| {
            if error.kind() == io::ErrorKind::AlreadyExists {
                DaemonError::AlreadyOwned
            } else {
                DaemonError::Io(error)
            }
        })?;
        let path = inner.path.clone();
        Ok(Self {
            _inner: inner,
            path,
        })
    }
}

/// Acquire the daemon lock and apply migrations before any store is exposed.
pub fn open_owned(
    data_dir: &Path,
    binary_digest: &str,
) -> Result<(DaemonLock, Connection), DaemonError> {
    crate::fs_security::validate_data_store(data_dir, None)?;
    let lock = DaemonLock::acquire(data_dir)?;
    let mut connection = Connection::open(data_dir.join("agent-graph.db"))?;
    migrations::apply(&mut connection, binary_digest)?;
    Ok((lock, connection))
}

pub fn record_instance_start(
    connection: &Connection,
    instance_id: &str,
    binary_digest: &str,
) -> rusqlite::Result<()> {
    connection.execute_batch(
        "CREATE TABLE IF NOT EXISTS server_instances (
            instance_id TEXT PRIMARY KEY,
            started_at TEXT NOT NULL,
            heartbeat_at TEXT NOT NULL,
            stopped_at TEXT,
            binary_digest TEXT NOT NULL
        )",
    )?;
    connection.execute(
        "INSERT INTO server_instances(instance_id, started_at, heartbeat_at, binary_digest)
         VALUES (?1, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP, ?2)",
        params![instance_id, binary_digest],
    )?;
    Ok(())
}

pub fn record_instance_stop(connection: &Connection, instance_id: &str) -> rusqlite::Result<()> {
    connection.execute(
        "UPDATE server_instances
         SET stopped_at = CURRENT_TIMESTAMP, heartbeat_at = CURRENT_TIMESTAMP
         WHERE instance_id = ?1 AND stopped_at IS NULL",
        [instance_id],
    )?;
    Ok(())
}

pub fn executable_digest() -> String {
    let bytes = std::env::current_exe()
        .ok()
        .and_then(|path| fs::read(path).ok())
        .unwrap_or_default();
    let mut digest = Sha256::new();
    digest.update(bytes);
    format!("sha256:{:x}", digest.finalize())
}

pub fn socket_path(runtime_dir: &Path, instance: &str) -> PathBuf {
    runtime_dir
        .join("agent-graph")
        .join(instance)
        .join("daemon.sock")
}

/// Prepare the private socket parent and remove only a stale Unix socket.
pub fn prepare_socket_path(runtime_dir: &Path, instance: &str) -> io::Result<PathBuf> {
    crate::fs_security::ensure_private_dir(runtime_dir)?;
    let namespace = runtime_dir.join("agent-graph");
    crate::fs_security::ensure_private_dir(&namespace)?;
    let parent = namespace.join(instance);
    crate::fs_security::ensure_private_dir(&parent)?;
    let path = socket_path(runtime_dir, instance);
    match fs::symlink_metadata(&path) {
        Ok(metadata) if metadata.file_type().is_symlink() => {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "daemon socket path is a symlink",
            ));
        }
        Ok(metadata) if !metadata.file_type().is_socket() => {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                "daemon socket path is not a stale Unix socket",
            ));
        }
        Ok(_) => fs::remove_file(&path)?,
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(error) => return Err(error),
    }
    Ok(path)
}

pub fn bind_private_socket(path: &Path) -> io::Result<UnixListener> {
    let listener = std::os::unix::net::UnixListener::bind(path)?;
    listener
        .set_nonblocking(true)
        .map_err(|error| io::Error::new(error.kind(), error.to_string()))?;
    fs::set_permissions(path, fs::Permissions::from_mode(0o600))?;
    let metadata = fs::metadata(path)?;
    if metadata.permissions().mode() & 0o777 != 0o600 {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "daemon socket permissions are not 0600",
        ));
    }
    UnixListener::from_std(listener)
}
