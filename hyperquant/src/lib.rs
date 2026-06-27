//! Experimental lattice quantization primitives.
//!
//! This crate starts deliberately small: tested CPU-side quantization building
//! blocks, explicit receipts, and conservative claim boundaries. It does not
//! claim model-quality preservation, CUDA support, or parity with any paper.

pub mod error;
pub mod lattice;
pub mod receipt;
pub mod scalar;

pub use error::{HyperQuantError, Result};
pub use lattice::{quantize_a2, quantize_z1, HyperQuantConfig, HyperQuantResult, LatticeKind};
pub use receipt::{ClaimBoundary, HyperQuantReceiptV1};
