"""Fused CUDA kernels for compressed-domain scoring.

Two kernels:
1. per_dim_score: uint8 codes + precomputed scaled_query → float scores
2. per_key_score: int8 codes + per-key scale → float scores

Each warp (32 threads) scores one key. Query vector cached in shared memory.
"""

import torch
from torch.utils.cpp_extension import load_inline

CUDA_SOURCE = r"""
#include <torch/extension.h>
#include <cuda_runtime.h>
#include <cstdint>

// ============================================================================
// Per-dimension quantized scoring kernel
// ============================================================================
// Each warp scores one key.
// score[i] = (bias + sum_d(codes[i,d] * scaled_query[d])) * k_norms[i]
//
// scaled_query and bias are precomputed on host (once per query):
//   scaled_query[d] = dim_ranges[d] / levels * query[d]
//   bias = dot(dim_mins, query)

__global__ __launch_bounds__(256, 4)
void per_dim_score_kernel(
    const uint8_t* __restrict__ codes,        // [n_keys, dim]
    const float* __restrict__ k_norms,        // [n_keys]
    const float* __restrict__ scaled_query,   // [dim]
    const float bias,
    float* __restrict__ scores,               // [n_keys]
    const int n_keys,
    const int dim
) {
    extern __shared__ float sh_sq[];  // shared copy of scaled_query

    const int warp_in_block = threadIdx.x >> 5;          // threadIdx.x / 32
    const int lane           = threadIdx.x & 31;          // threadIdx.x % 32
    const int warps_per_block = blockDim.x >> 5;
    const int key_id = blockIdx.x * warps_per_block + warp_in_block;

    // Collaborative load of scaled_query into shared memory
    for (int d = threadIdx.x; d < dim; d += blockDim.x) {
        sh_sq[d] = scaled_query[d];
    }
    __syncthreads();

    if (key_id >= n_keys) return;

    const uint8_t* key_codes = codes + key_id * dim;
    float partial = 0.0f;

    // Each lane handles dim/32 elements (strided by 32 for coalesced access)
    for (int d = lane; d < dim; d += 32) {
        partial += static_cast<float>(key_codes[d]) * sh_sq[d];
    }

    // Warp-level reduction via shuffle (no shared memory needed)
    partial += __shfl_down_sync(0xffffffff, partial, 16);
    partial += __shfl_down_sync(0xffffffff, partial, 8);
    partial += __shfl_down_sync(0xffffffff, partial, 4);
    partial += __shfl_down_sync(0xffffffff, partial, 2);
    partial += __shfl_down_sync(0xffffffff, partial, 1);

    if (lane == 0) {
        scores[key_id] = (partial + bias) * k_norms[key_id];
    }
}

// ============================================================================
// Per-key quantized scoring kernel
// ============================================================================
// score[i] = (scales[i] / levels) * sum_d(qk[i,d] * query[d])

__global__ __launch_bounds__(256, 4)
void per_key_score_kernel(
    const int8_t* __restrict__ qk,           // [n_keys, dim]
    const float* __restrict__ scales,        // [n_keys]
    const float* __restrict__ query,         // [dim]
    const float inv_levels,
    float* __restrict__ scores,              // [n_keys]
    const int n_keys,
    const int dim
) {
    extern __shared__ float sh_q[];

    const int warp_in_block = threadIdx.x >> 5;
    const int lane           = threadIdx.x & 31;
    const int warps_per_block = blockDim.x >> 5;
    const int key_id = blockIdx.x * warps_per_block + warp_in_block;

    for (int d = threadIdx.x; d < dim; d += blockDim.x) {
        sh_q[d] = query[d];
    }
    __syncthreads();

    if (key_id >= n_keys) return;

    const int8_t* key_qk = qk + key_id * dim;
    float partial = 0.0f;

    for (int d = lane; d < dim; d += 32) {
        partial += static_cast<float>(key_qk[d]) * sh_q[d];
    }

    partial += __shfl_down_sync(0xffffffff, partial, 16);
    partial += __shfl_down_sync(0xffffffff, partial, 8);
    partial += __shfl_down_sync(0xffffffff, partial, 4);
    partial += __shfl_down_sync(0xffffffff, partial, 2);
    partial += __shfl_down_sync(0xffffffff, partial, 1);

    if (lane == 0) {
        scores[key_id] = partial * inv_levels * scales[key_id];
    }
}

// ============================================================================
// Torch C++ wrappers
// ============================================================================

torch::Tensor per_dim_score_cuda(
    torch::Tensor codes,        // [n_keys, dim] uint8
    torch::Tensor k_norms,      // [n_keys] float32
    torch::Tensor scaled_query, // [dim] float32
    float bias
) {
    TORCH_CHECK(codes.is_cuda(), "codes must be a CUDA tensor");
    TORCH_CHECK(codes.is_contiguous(), "codes must be contiguous");
    TORCH_CHECK(k_norms.is_contiguous(), "k_norms must be contiguous");
    TORCH_CHECK(scaled_query.is_contiguous(), "scaled_query must be contiguous");

    const int n_keys = codes.size(0);
    const int dim    = codes.size(1);

    auto scores = torch::empty({n_keys},
        torch::TensorOptions().dtype(torch::kFloat32).device(codes.device()));

    if (n_keys == 0) return scores;

    const int threads = 256;
    const int warps_per_block = threads / 32;
    const int blocks = (n_keys + warps_per_block - 1) / warps_per_block;
    const int shmem  = dim * sizeof(float);

    per_dim_score_kernel<<<blocks, threads, shmem>>>(
        codes.data_ptr<uint8_t>(),
        k_norms.data_ptr<float>(),
        scaled_query.data_ptr<float>(),
        bias,
        scores.data_ptr<float>(),
        n_keys, dim
    );

    return scores;
}

torch::Tensor per_key_score_cuda(
    torch::Tensor qk,          // [n_keys, dim] int8
    torch::Tensor scales,      // [n_keys] float32
    torch::Tensor query,       // [dim] float32
    float inv_levels
) {
    TORCH_CHECK(qk.is_cuda(), "qk must be a CUDA tensor");
    TORCH_CHECK(qk.is_contiguous(), "qk must be contiguous");
    TORCH_CHECK(scales.is_contiguous(), "scales must be contiguous");
    TORCH_CHECK(query.is_contiguous(), "query must be contiguous");

    const int n_keys = qk.size(0);
    const int dim    = qk.size(1);

    auto scores = torch::empty({n_keys},
        torch::TensorOptions().dtype(torch::kFloat32).device(qk.device()));

    if (n_keys == 0) return scores;

    const int threads = 256;
    const int warps_per_block = threads / 32;
    const int blocks = (n_keys + warps_per_block - 1) / warps_per_block;
    const int shmem  = dim * sizeof(float);

    per_key_score_kernel<<<blocks, threads, shmem>>>(
        qk.data_ptr<int8_t>(),
        scales.data_ptr<float>(),
        query.data_ptr<float>(),
        inv_levels,
        scores.data_ptr<float>(),
        n_keys, dim
    );

    return scores;
}
"""

print("Compiling fused scorer CUDA kernels...", flush=True)
_module = load_inline(
    name='fused_scorer',
    cpp_sources='',
    cuda_sources=CUDA_SOURCE,
    functions=['per_dim_score_cuda', 'per_key_score_cuda'],
    extra_cuda_cflags=['-O3', '--use_fast_math', '-lineinfo'],
    verbose=False,
)
print("Compilation complete.", flush=True)


class PerDimScorerFused:
    """Fused per-dimension quantized scorer."""

    def __init__(self, dim: int, bits: int):
        self.dim = dim
        self.bits = bits
        self.levels = (1 << bits) - 1

    def quantize_keys(self, keys: torch.Tensor):
        """Quantize keys → (codes, k_norms, dim_mins, dim_ranges)."""
        k_norms = keys.norm(dim=-1).clamp_min(1e-6)
        k_unit = keys / k_norms.unsqueeze(-1)
        dim_mins = k_unit.min(dim=0).values
        dim_maxs = k_unit.max(dim=0).values
        dim_ranges = (dim_maxs - dim_mins).clamp_min(1e-6)
        codes = torch.round(
            ((k_unit - dim_mins.unsqueeze(0)) / dim_ranges.unsqueeze(0)) * self.levels
        ).clamp(0, self.levels).to(torch.uint8)
        return codes, k_norms, dim_mins, dim_ranges

    def prepare_query(self, query: torch.Tensor,
                      dim_mins: torch.Tensor, dim_ranges: torch.Tensor):
        """Precompute scaled_query and bias (once per query)."""
        scaled_query = (dim_ranges / self.levels) * query
        bias = float((dim_mins * query).sum().item())
        return scaled_query.contiguous(), bias

    def score(self, codes: torch.Tensor, k_norms: torch.Tensor,
              scaled_query: torch.Tensor, bias: float) -> torch.Tensor:
        """Fused scoring: one kernel launch for all keys."""
        return _module.per_dim_score_cuda(
            codes.contiguous(), k_norms.contiguous(),
            scaled_query.contiguous(), bias)


class PerKeyScorerFused:
    """Fused per-key quantized scorer."""

    def __init__(self, dim: int, bits: int):
        self.dim = dim
        self.bits = bits
        self.levels = (1 << (bits - 1)) - 1

    def quantize_keys(self, keys: torch.Tensor):
        """Quantize keys → (qk, scales)."""
        max_abs = keys.abs().amax(dim=-1, keepdim=True).clamp_min(1e-6)
        qk = torch.round((keys / max_abs).clamp(-1, 1) * self.levels).to(torch.int8)
        scales = max_abs.squeeze(-1).float()
        return qk.contiguous(), scales.contiguous()

    def score(self, qk: torch.Tensor, scales: torch.Tensor,
              query: torch.Tensor) -> torch.Tensor:
        """Fused scoring: one kernel launch for all keys."""
        inv_levels = 1.0 / self.levels
        return _module.per_key_score_cuda(
            qk.contiguous(), scales.contiguous(),
            query.contiguous(), inv_levels)
