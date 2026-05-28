//! `scr-runtime-compression` — Runtime integration adapter for semantic-memory.
//!
//! Provides:
//! - [`CompressedSearchPath`] — search path wrapper that carries compression metadata
//! - [`ExactFallbackAdapter`] — decode adapter that decompresses via turbo-quant / fib-quant
//!
//! ## Design principles
//!
//! - **Never owns codec truth** — all compression/decompression is delegated to
//!   `turbo-quant` and `fib-quant`. This crate holds only the integration layer.
//! - **No `unwrap` in production paths** — all fallible operations return `Result` or `Option`.
//! - **Rust 2021, MSRV 1.75** — compatible with the workspace minimum.

mod error;
mod compressed_search_path;
mod exact_fallback_adapter;

pub use error::{CompressionError, DecompressError};
pub use compressed_search_path::CompressedSearchPath;
pub use exact_fallback_adapter::ExactFallbackAdapter;

/// Codec identity for runtime dispatch.
///
/// Each compressed representation carries a codec discriminant so the adapter
/// can delegate to the correct encoder/decoder without static knowledge of the
/// underlying codec crate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CodecId {
    /// TurboQuant polar-code + residual-sketch codec.
    TurboQuant,
    /// FibQuant radial-angular codec.
    FibQuant,
    /// Uncompressed representation (identity pass-through).
    Uncompressed,
}

impl CodecId {
    /// Returns `true` if this codec requires exact fallback on decode.
    pub fn requires_exact_fallback(self) -> bool {
        matches!(self, Self::TurboQuant | Self::FibQuant)
    }
}

impl std::fmt::Display for CodecId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TurboQuant => write!(f, "turbo_quant"),
            Self::FibQuant => write!(f, "fib_quant"),
            Self::Uncompressed => write!(f, "uncompressed"),
        }
    }
}