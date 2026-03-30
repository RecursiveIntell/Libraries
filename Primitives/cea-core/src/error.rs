#[derive(Debug, thiserror::Error)]
pub enum CeaCoreError {
    #[error("invalid input: {0}")]
    InvalidInput(String),

    #[error("invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("graph corruption: {0}")]
    GraphCorruption(String),

    #[error("unsupported graph schema version: expected {expected}, found {found}")]
    UnsupportedSchemaVersion { expected: u32, found: u32 },

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("persistence error: {0}")]
    Persistence(#[from] bincode::Error),
}
