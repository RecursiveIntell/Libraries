//! Lock-owning daemon primitives.
use crate::migrations;
use rusqlite::Connection;
use std::{
    fs::{self, File, OpenOptions},
    io,
    os::unix::fs::OpenOptionsExt,
    path::{Path, PathBuf},
};
pub const MAX_FRAME: usize = 1024 * 1024;
#[derive(Debug)]
pub enum DaemonError {
    AlreadyOwned,
    Io(io::Error),
    Sql(rusqlite::Error),
}
impl From<io::Error> for DaemonError {
    fn from(e: io::Error) -> Self {
        Self::Io(e)
    }
}
impl From<rusqlite::Error> for DaemonError {
    fn from(e: rusqlite::Error) -> Self {
        Self::Sql(e)
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
#[derive(Debug)]
pub struct DaemonLock {
    file: File,
    pub path: PathBuf,
}
impl DaemonLock {
    pub fn acquire(data_dir: &Path) -> Result<Self, DaemonError> {
        fs::create_dir_all(data_dir)?;
        let path = data_dir.join("daemon.lock");
        let file = OpenOptions::new()
            .create_new(true)
            .read(true)
            .write(true)
            .mode(0o600)
            .open(&path)
            .map_err(|e| {
                if e.kind() == io::ErrorKind::AlreadyExists {
                    DaemonError::AlreadyOwned
                } else {
                    DaemonError::Io(e)
                }
            })?;
        Ok(Self { file, path })
    }
}
impl Drop for DaemonLock {
    fn drop(&mut self) {
        let _ = self.file.sync_all();
        let _ = fs::remove_file(&self.path);
    }
}
pub fn open_owned(
    data_dir: &Path,
    binary_digest: &str,
) -> Result<(DaemonLock, Connection), DaemonError> {
    let lock = DaemonLock::acquire(data_dir)?;
    let mut c = Connection::open(data_dir.join("agent-graph.db"))?;
    migrations::apply(&mut c, binary_digest)?;
    Ok((lock, c))
}
pub fn socket_path(runtime_dir: &Path, instance: &str) -> PathBuf {
    runtime_dir
        .join("agent-graph")
        .join(instance)
        .join("daemon.sock")
}
