//! GPU backend crate for vector quantization acceleration.
//!
//! Provides CUDA kernels for:
//! - Fast Walsh-Hadamard Transform (WHT) — shared by fib-quant and turbo-quant
//! - Lloyd-Max scalar quantization — per-coordinate codebook quantization
//! - Bit-packing — compact index storage
//!
//! Feature-gated: `gpu` feature enables CUDA via cudarc.
//! Without it, this crate is a stub — all operations return `GpuUnavailable`.

use std::sync::OnceLock;

#[cfg(feature = "gpu")]
pub mod cuda;
pub mod error;
pub mod fallback;

pub use error::GpuError;

/// Result type for GPU operations.
pub type Result<T> = std::result::Result<T, GpuError>;

/// Global GPU context — initialized once, shared across crates.
static GPU_CTX: OnceLock<Option<GpuContext>> = OnceLock::new();

/// GPU context holding device, stream, and compiled kernels.
#[derive(Debug)]
pub struct GpuContext {
    /// CUDA device index
    pub device_index: u32,
    /// Available device memory in bytes
    pub memory_bytes: usize,
    /// Device name
    pub device_name: String,
}

impl GpuContext {
    /// Initialize GPU context. Returns None if no CUDA device is available
    /// or the `gpu` feature is disabled.
    pub fn init() -> Option<&'static GpuContext> {
        GPU_CTX.get_or_init(|| {
            #[cfg(feature = "gpu")]
            {
                cuda::init_context().ok()
            }
            #[cfg(not(feature = "gpu"))]
            {
                None
            }
        });
        GPU_CTX.get().and_then(|c| c.as_ref())
    }

    /// Check if GPU acceleration is available.
    pub fn is_available() -> bool {
        Self::init().is_some()
    }

    /// Minimum batch size for GPU to be worth the launch overhead.
    pub const GPU_MIN_BATCH_SIZE: usize = 16;
    /// Minimum dimension for GPU acceleration (small dims are faster on CPU).
    pub const GPU_MIN_DIM: usize = 64;
}

/// Batched Hadamard Walsh-Hadamard Transform.
///
/// Applies in-place WHT to `n` vectors of length `dim`.
/// `dim` must be a power of 2. Pad input before calling.
/// Uses GPU if available and batch size warrants it.
pub fn hadamard_batch(data: &mut [f32], n: usize, dim: usize, seed: u64) -> Result<()> {
    if data.len() != n * dim {
        return Err(GpuError::DimensionMismatch {
            expected: n * dim,
            got: data.len(),
        });
    }

    #[cfg(feature = "gpu")]
    {
        if let Some(ctx) = GpuContext::init() {
            if n >= GpuContext::GPU_MIN_BATCH_SIZE && dim >= GpuContext::GPU_MIN_DIM {
                return cuda::hadamard_batch_gpu(ctx, data, n, dim, seed);
            }
        }
    }

    // CPU fallback
    fallback::hadamard_batch_cpu(data, n, dim, seed)
}

/// Batched Lloyd-Max quantization.
///
/// Quantizes a block of `n` vectors, each of `dim` scalars, into
/// `n_levels` codebook entries per block of size `k`.
///
/// Returns (indices, norms) where:
/// - indices: flat u8 array of length n * (dim / k) — codebook indices
/// - norms: flat f32 array of length n * (dim / k) — per-block L2 norms
pub fn lloyd_max_batch(
    vectors: &[f32],
    n: usize,
    dim: usize,
    k: usize,
    n_levels: usize,
    seed: u64,
) -> Result<(Vec<u8>, Vec<f32>)> {
    if vectors.len() != n * dim {
        return Err(GpuError::DimensionMismatch {
            expected: n * dim,
            got: vectors.len(),
        });
    }
    if dim % k != 0 {
        return Err(GpuError::InvalidConfig(format!(
            "dim ({}) must be divisible by k ({})",
            dim, k
        )));
    }

    #[cfg(feature = "gpu")]
    {
        if let Some(ctx) = GpuContext::init() {
            if n >= GpuContext::GPU_MIN_BATCH_SIZE {
                return cuda::lloyd_max_batch_gpu(ctx, vectors, n, dim, k, n_levels, seed);
            }
        }
    }

    fallback::lloyd_max_batch_cpu(vectors, n, dim, k, n_levels, seed)
}

/// Batched Lloyd-Max decode.
///
/// Reconstructs approximate f32 vectors from quantized indices and norms.
pub fn lloyd_max_decode_batch(
    indices: &[u8],
    norms: &[f32],
    n: usize,
    dim: usize,
    k: usize,
    n_levels: usize,
    seed: u64,
) -> Result<Vec<f32>> {
    let blocks_per_vector = dim / k;
    if indices.len() != n * blocks_per_vector * k {
        return Err(GpuError::DimensionMismatch {
            expected: n * blocks_per_vector * k,
            got: indices.len(),
        });
    }

    #[cfg(feature = "gpu")]
    {
        if let Some(ctx) = GpuContext::init() {
            if n >= GpuContext::GPU_MIN_BATCH_SIZE {
                return cuda::lloyd_max_decode_batch_gpu(
                    ctx, indices, norms, n, dim, k, n_levels, seed,
                );
            }
        }
    }

    fallback::lloyd_max_decode_batch_cpu(indices, norms, n, dim, k, n_levels, seed)
}

/// Bit-pack quantized indices into compact byte array.
///
/// Input: flat u8 array where each byte is a codebook index (0..n_levels-1).
/// Output: packed bytes using `bits_per_index` bits per index.
pub fn bitpack(indices: &[u8], bits_per_index: usize) -> Result<Vec<u8>> {
    if bits_per_index == 0 || bits_per_index > 8 {
        return Err(GpuError::InvalidConfig(format!(
            "bits_per_index must be 1-8, got {}",
            bits_per_index
        )));
    }

    #[cfg(feature = "gpu")]
    {
        if let Some(ctx) = GpuContext::init() {
            if indices.len() >= 1024 {
                return cuda::bitpack_gpu(ctx, indices, bits_per_index);
            }
        }
    }

    fallback::bitpack_cpu(indices, bits_per_index)
}
