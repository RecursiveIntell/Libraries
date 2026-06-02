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

    /// Persistence-layer error. Wraps `postcard::Error` (which is a
    /// single enum type covering both serialize and deserialize
    /// failures — much cleaner than bincode 2's split EncodeError /
    /// DecodeError design). The bincode 1.3.3 source this replaced
    /// had a similar `#[from] bincode::Error` design; the API is
    /// effectively identical at the call site.
    #[error("persistence error: {0}")]
    Persistence(#[from] postcard::Error),
}
