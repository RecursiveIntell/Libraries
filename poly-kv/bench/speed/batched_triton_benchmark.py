"""Batched Triton kernel — processes BLOCK_M keys per program instead of 1.

Previous kernel: 512 programs × 128 ops each = poor occupancy, 69μs
cuBLAS:          1 program  × 65536 ops = full SM utilization, 16μs
This kernel:     8 programs × 8192 ops = good occupancy, should be ~10-20μs

The key insight: cuBLAS wins because it batches. Our algorithm is fine,
our kernel launch pattern was wrong.
"""

import torch
import triton
import triton.language as tl
import time
import json
from pathlib import Path


@triton.jit
def per_dim_score_batched_kernel(
    codes_ptr,           # [n_keys, dim] uint8
    k_norms_ptr,         # [n_keys] float32
    scaled_query_ptr,    # [dim] float32
    bias,                # float32 scalar
    scores_ptr,          # [n_keys] float32
    n_keys,
    dim,
    BLOCK_M: tl.constexpr,  # keys per block
    BLOCK_D: tl.constexpr,  # dim per block (>= dim)
):
    """Score BLOCK_M keys per program using tl.dot for hardware-accelerated matmul."""
    pid = tl.program_id(0)
    key_start = pid * BLOCK_M
    
    # Key indices for this block
    key_offsets = key_start + tl.arange(0, BLOCK_M)  # [BLOCK_M]
    key_mask = key_offsets < n_keys
    
    # Load k_norms for this block
    norms = tl.load(k_norms_ptr + key_offsets, mask=key_mask, other=0.0)  # [BLOCK_M]
    
    # Load codes: [BLOCK_M, dim] as float32
    # Each row is one key's uint8 codes
    d_offsets = tl.arange(0, BLOCK_D)  # [BLOCK_D]
    d_mask = d_offsets < dim
    
    # 2D index: [BLOCK_M, BLOCK_D]
    codes_2d = tl.load(
        codes_ptr + key_offsets[:, None] * dim + d_offsets[None, :],
        mask=key_mask[:, None] & d_mask[None, :],
        other=0,
    ).to(tl.float32)  # [BLOCK_M, BLOCK_D]
    
    # Load scaled_query: [BLOCK_D]
    sq = tl.load(scaled_query_ptr + d_offsets, mask=d_mask, other=0.0)  # [BLOCK_D]
    
    # Matmul: [BLOCK_M, BLOCK_D] @ [BLOCK_D] = [BLOCK_M]
    # Use tl.dot for potential hardware acceleration
    # tl.dot needs 2D operands, so reshape sq to [BLOCK_D, 1]
    # scores = codes_2d @ sq
    
    # Element-wise multiply + reduce (works when tl.dot dtype constraints fail)
    partial = codes_2d * sq[None, :]  # [BLOCK_M, BLOCK_D]
    dot = tl.sum(partial, axis=1)  # [BLOCK_M]
    
    # Apply bias and norm
    scores = (dot + bias) * norms
    
    # Store
    tl.store(scores_ptr + key_offsets, scores, mask=key_mask)


@triton.jit
def per_key_score_batched_kernel(
    qk_ptr,              # [n_keys, dim] int8
    scales_ptr,          # [n_keys] float32
    query_ptr,           # [dim] float32
    inv_levels,          # float32 scalar
    scores_ptr,          # [n_keys] float32
    n_keys,
    dim,
    BLOCK_M: tl.constexpr,
    BLOCK_D: tl.constexpr,
):
    """Per-key scoring, BLOCK_M keys per program."""
    pid = tl.program_id(0)
    key_start = pid * BLOCK_M
    
    key_offsets = key_start + tl.arange(0, BLOCK_M)
    key_mask = key_offsets < n_keys
    
    scales = tl.load(scales_ptr + key_offsets, mask=key_mask, other=0.0)
    
    d_offsets = tl.arange(0, BLOCK_D)
    d_mask = d_offsets < dim
    
    qk_2d = tl.load(
        qk_ptr + key_offsets[:, None] * dim + d_offsets[None, :],
        mask=key_mask[:, None] & d_mask[None, :],
        other=0,
    ).to(tl.int32).to(tl.float32)  # [BLOCK_M, BLOCK_D]
    
    q = tl.load(query_ptr + d_offsets, mask=d_mask, other=0.0)  # [BLOCK_D]
    
    partial = qk_2d * q[None, :]
    dot = tl.sum(partial, axis=1)
    
    scores = dot * inv_levels * scales
    
    tl.store(scores_ptr + key_offsets, scores, mask=key_mask)


class PerDimScorerBatched:
    def __init__(self, dim, bits=8, block_m=64):
        self.dim = dim
        self.bits = bits
        self.levels = (1 << bits) - 1
        self.BLOCK_M = block_m
        self.BLOCK_D = triton.next_power_of_2(dim)
    
    def quantize_keys(self, keys):
        k_norms = keys.norm(dim=1).clamp_min(1e-6)
        k_unit = keys / k_norms.unsqueeze(-1)
        dim_mins = k_unit.min(dim=0).values
        dim_maxs = k_unit.max(dim=0).values
        dim_ranges = (dim_maxs - dim_mins).clamp_min(1e-6)
        codes = torch.round(
            ((k_unit - dim_mins.unsqueeze(0)) / dim_ranges.unsqueeze(0)) * self.levels
        ).clamp(0, self.levels).to(torch.uint8)
        return codes, k_norms, dim_mins, dim_ranges
    
    def prepare_query(self, query, dim_mins, dim_ranges):
        scaled_query = (dim_ranges / self.levels) * query
        bias = float((dim_mins * query).sum().item())
        return scaled_query.contiguous(), bias
    
    def score(self, codes, k_norms, scaled_query, bias):
        n_keys = codes.shape[0]
        scores = torch.empty(n_keys, device=codes.device, dtype=torch.float32)
        grid = ((n_keys + self.BLOCK_M - 1) // self.BLOCK_M,)
        per_dim_score_batched_kernel[grid](
            codes, k_norms, scaled_query, bias, scores,
            n_keys, self.dim,
            BLOCK_M=self.BLOCK_M, BLOCK_D=self.BLOCK_D,
        )
        return scores


class PerKeyScorerBatched:
    def __init__(self, dim, bits=8, block_m=64):
        self.dim = dim
        self.bits = bits
        self.levels = (1 << (bits - 1)) - 1
        self.BLOCK_M = block_m
        self.BLOCK_D = triton.next_power_of_2(dim)
    
    def quantize_keys(self, keys):
        scales = keys.abs().max(dim=1).values.clamp(min=1e-6)
        qk_float = torch.round(keys / scales.unsqueeze(1) * self.levels)
        qk = qk_float.clamp(-self.levels, self.levels).to(torch.int8)
        return qk.contiguous(), scales.contiguous()
    
    def score(self, qk, scales, query):
        n_keys = qk.shape[0]
        scores = torch.empty(n_keys, device=qk.device, dtype=torch.float32)
        inv_levels = 1.0 / self.levels
        grid = ((n_keys + self.BLOCK_M - 1) // self.BLOCK_M,)
        per_key_score_batched_kernel[grid](
            qk, scales, query, inv_levels, scores,
            n_keys, self.dim,
            BLOCK_M=self.BLOCK_M, BLOCK_D=self.BLOCK_D,
        )
        return scores


def benchmark_fn(fn, n_warmup=20, n_runs=100):
    for _ in range(n_warmup):
        fn()
    torch.cuda.synchronize()
    start = time.perf_counter()
    for _ in range(n_runs):
        fn()
    torch.cuda.synchronize()
    return (time.perf_counter() - start) / n_runs


def main():
    device = 'cuda'
    dim = 128
    n_keys_list = [64, 128, 256, 512, 1024, 2048]
    block_m_values = [16, 32, 64, 128]
    
    results = []
    
    for n_keys in n_keys_list:
        print(f"\n{'='*60}")
        print(f"n_keys={n_keys}, dim={dim}")
        print(f"{'='*60}")
        
        keys = torch.randn(n_keys, dim, device=device, dtype=torch.float32)
        query = torch.randn(dim, device=device, dtype=torch.float32)
        
        # Dense baseline
        t_dense = benchmark_fn(lambda: torch.mv(keys, query))
        scores_dense = torch.mv(keys, query)
        print(f"Dense (cuBLAS):      {t_dense*1e6:7.2f} μs")
        
        # Per-dim batched — try different BLOCK_M values
        best_pd_time = float('inf')
        best_pd_bm = 0
        for bm in block_m_values:
            scorer = PerDimScorerBatched(dim=dim, bits=8, block_m=bm)
            codes, k_norms, dim_mins, dim_ranges = scorer.quantize_keys(keys)
            scaled_query, bias = scorer.prepare_query(query, dim_mins, dim_ranges)
            
            t = benchmark_fn(lambda: scorer.score(codes, k_norms, scaled_query, bias))
            if t < best_pd_time:
                best_pd_time = t
                best_pd_bm = bm
        
        scorer = PerDimScorerBatched(dim=dim, bits=8, block_m=best_pd_bm)
        codes, k_norms, dim_mins, dim_ranges = scorer.quantize_keys(keys)
        scaled_query, bias = scorer.prepare_query(query, dim_mins, dim_ranges)
        scores_pd = scorer.score(codes, k_norms, scaled_query, bias)
        cos_pd = torch.nn.functional.cosine_similarity(
            scores_dense.unsqueeze(0), scores_pd.unsqueeze(0)).item()
        print(f"Per-dim-8 batched:   {best_pd_time*1e6:7.2f} μs  (BLOCK_M={best_pd_bm})  cos={cos_pd:.4f}  speedup={t_dense/best_pd_time:.2f}x")
        
        # Per-key batched — try different BLOCK_M values
        best_pk_time = float('inf')
        best_pk_bm = 0
        for bm in block_m_values:
            scorer = PerKeyScorerBatched(dim=dim, bits=8, block_m=bm)
            qk, scales = scorer.quantize_keys(keys)
            
            t = benchmark_fn(lambda: scorer.score(qk, scales, query))
            if t < best_pk_time:
                best_pk_time = t
                best_pk_bm = bm
        
        scorer = PerKeyScorerBatched(dim=dim, bits=8, block_m=best_pk_bm)
        qk, scales = scorer.quantize_keys(keys)
        scores_pk = scorer.score(qk, scales, query)
        cos_pk = torch.nn.functional.cosine_similarity(
            scores_dense.unsqueeze(0), scores_pk.unsqueeze(0)).item()
        print(f"Per-key-8 batched:   {best_pk_time*1e6:7.2f} μs  (BLOCK_M={best_pk_bm})  cos={cos_pk:.4f}  speedup={t_dense/best_pk_time:.2f}x")
        
        results.append({
            'n_keys': n_keys,
            'dim': dim,
            'dense_us': t_dense * 1e6,
            'per_dim_batched_us': best_pd_time * 1e6,
            'per_dim_block_m': best_pd_bm,
            'per_dim_speedup': t_dense / best_pd_time,
            'per_dim_cosine': cos_pd,
            'per_key_batched_us': best_pk_time * 1e6,
            'per_key_block_m': best_pk_bm,
            'per_key_speedup': t_dense / best_pk_time,
            'per_key_cosine': cos_pk,
        })
    
    print(f"\n{'='*60}")
    print("SUMMARY — Batched Triton vs cuBLAS")
    print(f"{'='*60}")
    print(f"{'n_keys':>6}  {'Dense':>8}  {'PD batch':>10}  {'PD speed':>10}  {'PK batch':>10}  {'PK speed':>10}")
    print(f"{'':>6}  {'(μs)':>8}  {'(μs)':>10}  {'(x)':>10}  {'(μs)':>10}  {'(x)':>10}")
    print("-" * 60)
    for r in results:
        print(f"{r['n_keys']:>6}  "
              f"{r['dense_us']:>8.2f}  "
              f"{r['per_dim_batched_us']:>10.2f}  "
              f"{r['per_dim_speedup']:>10.2f}  "
              f"{r['per_key_batched_us']:>10.2f}  "
              f"{r['per_key_speedup']:>10.2f}")
    
    output_file = Path(__file__).parent / 'batched_triton_results.json'
    with open(output_file, 'w') as f:
        json.dump(results, f, indent=2)
    print(f"\nResults saved to {output_file}")


if __name__ == '__main__':
    main()