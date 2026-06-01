//! CUDA-accelerated GPU operations via cudarc.
//!
//! Only compiled when the `gpu` feature is enabled.

use crate::error::GpuError;
use crate::Result;
use crate::GpuContext;

/// Initialize CUDA context and probe device capabilities.
pub fn init_context() -> Result<GpuContext> {
    use cudarc::driver::{CudaContext, CudaDevice, CudaStream, DriverError};

    let ctx = CudaContext::new(0).map_err(|e| match e {
        DriverError::NoDevice => GpuError::GpuUnavailable,
        other => GpuError::CudaError(format!("CUDA init failed: {}", other)),
    })?;

    let device = ctx.device(0).map_err(|e| {
        GpuError::CudaError(format!("failed to get device 0: {}", e))
    })?;

    let name = device.name().unwrap_or_else(|_| "unknown".into());
    let memory_bytes = device.total_mem().unwrap_or(0) as usize;

    Ok(GpuContext {
        device_index: 0,
        memory_bytes,
        device_name: name,
    })
}

/// GPU-accelerated Hadamard batch transform.
pub fn hadamard_batch_gpu(
    _ctx: &GpuContext,
    data: &mut [f32],
    n: usize,
    dim: usize,
    seed: u64,
) -> Result<()> {
    // For now: delegate to CPU fallback.
    // Full CUDA kernel implementation requires PTX compilation at build time
    // or runtime NVRTC. The CPU path is correct and deterministic.
    //
    // When cudarc kernel support is wired:
    // 1. Allocate device memory for data
    // 2. Copy data to device
    // 3. Launch hadamard_wht_batch kernel
    // 4. Copy result back to host
    crate::fallback::hadamard_batch_cpu(data, n, dim, seed)
}

/// GPU-accelerated Lloyd-Max batch quantization.
pub fn lloyd_max_batch_gpu(
    _ctx: &GpuContext,
    vectors: &[f32],
    n: usize,
    dim: usize,
    k: usize,
    n_levels: usize,
    seed: u64,
) -> Result<(Vec<u8>, Vec<f32>)> {
    // Delegate to CPU fallback for now.
    // The CPU path produces correct output that matches what a GPU kernel would.
    // GPU kernel implementation follows the same mathematical steps.
    crate::fallback::lloyd_max_batch_cpu(vectors, n, dim, k, n_levels, seed)
}

/// GPU-accelerated Lloyd-Max batch decode.
pub fn lloyd_max_decode_batch_gpu(
    _ctx: &GpuContext,
    indices: &[u8],
    norms: &[f32],
    n: usize,
    dim: usize,
    k: usize,
    n_levels: usize,
    seed: u64,
) -> Result<Vec<f32>> {
    crate::fallback::lloyd_max_decode_batch_cpu(indices, norms, n, dim, k, n_levels, seed)
}

/// GPU-accelerated bit packing.
pub fn bitpack_gpu(
    _ctx: &GpuContext,
    indices: &[u8],
    bits_per_index: usize,
) -> Result<Vec<u8>> {
    crate::fallback::bitpack_cpu(indices, bits_per_index)
}
