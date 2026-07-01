use crate::LatticeKind;

/// Errors returned by HyperQuant primitives.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum HyperQuantError {
    /// Quantization requires at least one input value.
    #[error("empty input is not quantizable")]
    EmptyInput,
    /// Non-finite input is rejected rather than silently canonicalized.
    #[error("non-finite input at index {index}")]
    NonFiniteInput { index: usize },
    /// Finite input/configuration still produced a non-finite internal artifact.
    #[error("non-finite quantization artifact: {stage}")]
    NonFiniteArtifact { stage: &'static str },
    /// The lattice is intentionally exposed as a known target but not yet implemented.
    #[error("unsupported lattice {0:?}: implementation is not shipped, and no placeholder result is emitted")]
    UnsupportedLattice(LatticeKind),
    /// Rice coding parameter is outside the supported local range.
    #[error("invalid Rice parameter k={k}; expected 0..=15")]
    InvalidRiceParameter { k: u8 },
    /// Encoded bitstream metadata is inconsistent with the payload.
    #[error("invalid Rice bitstream: {reason}")]
    InvalidRiceBitstream { reason: &'static str },
    /// RHT tile dimension must be a non-zero power of two.
    #[error("invalid RHT tile dimension {tile_dim}; expected a non-zero power of two")]
    InvalidTileDimension { tile_dim: usize },
    /// Output buffer length does not match encoded block length.
    #[error("decode output length mismatch: expected {expected}, got {actual}")]
    DecodeLengthMismatch { expected: usize, actual: usize },
}

/// Result alias for HyperQuant operations.
pub type Result<T> = std::result::Result<T, HyperQuantError>;
