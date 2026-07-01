//! Experimental lattice quantization primitives.
//!
//! This crate starts deliberately small: tested CPU-side quantization building
//! blocks, explicit receipts, and conservative claim boundaries. It does not
//! claim model-quality preservation, CUDA support, or parity with any paper.

pub mod error;
pub mod lattice;
pub mod receipt;
pub mod rht;
pub mod rice;
pub mod scalar;

#[cfg(feature = "compat")]
pub mod codec;

#[cfg(feature = "compat")]
pub use codec::{HyperQuantCodec, HyperQuantEncodedBlock};
pub use error::{HyperQuantError, Result};
pub use lattice::{
    quantize_a2, quantize_d4, quantize_z1, HyperQuantConfig, HyperQuantResult, LatticeKind,
};
pub use receipt::{ClaimBoundary, HyperQuantReceiptV1};
pub use rht::{hadamard_in_place_power_of_two, rht_tile, seeded_sign_flip, RhtReceiptV1};
pub use rice::{
    estimate_best_rice_profile, rice_decode_i16, rice_decode_u32s, rice_encode_i16,
    rice_encode_u32s, unzigzag_u32, zigzag_i16, RiceBitstream, RiceProfile,
};
