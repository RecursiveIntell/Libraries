"""Fused scoring kernels using Triton.

Triton JIT compiles to PTX, works on any GPU including Pascal (GTX 1070).
"""

import torch
import triton
import triton.language as tl


@triton.jit
def per_dim_score_kernel(
    codes_ptr,           # [n_keys, dim] uint8
    k_norms_ptr,         # [n_keys] float32
    scaled_query_ptr,    # [dim] float32
    bias_ptr,            # scalar float32 (single element)
    scores_ptr,          # [n_keys] float32
    n_keys,
    dim,
    BLOCK_SIZE: tl.constexpr,
):
    """Score one key per program.
    
    score = (bias + sum(codes[d] * scaled_query[d])) * k_norm
    """
    key_id = tl.program_id(0)
    
    if key_id >= n_keys:
        return
    
    # Load bias once
    bias = tl.load(bias_ptr)
    
    # Load k_norm for this key
    k_norm = tl.load(k_norms_ptr + key_id)
    
    # Accumulate dot product
    acc = 0.0
    
    for d in range(0, dim, BLOCK_SIZE):
        offsets = d + tl.arange(0, BLOCK_SIZE)
        mask = offsets < dim
        
        # Load codes (uint8) and scaled_query (float32)
        codes = tl.load(codes_ptr + key_id * dim + offsets, mask=mask, other=0).to(tl.int32).to(tl.float32)
        sq = tl.load(scaled_query_ptr + offsets, mask=mask, other=0.0)
        
        # Accumulate element-wise products
        acc += tl.sum(codes * sq)
    
    # Apply key normalization
    score = (acc + bias) * k_norm
    
    # Store result
    tl.store(scores_ptr + key_id, score)


@triton.jit
def per_key_score_kernel(
    qk_ptr,              # [n_keys, dim] int8
    scales_ptr,          # [n_keys] float32
    query_ptr,           # [dim] float32
    inv_levels_ptr,      # scalar float32 (single element)
    scores_ptr,          # [n_keys] float32
    n_keys,
    dim,
    BLOCK_SIZE: tl.constexpr,
):
    """Score one key per program.
    
    score = (scale / levels) * sum(qk[d] * query[d])
    """
    key_id = tl.program_id(0)
    
    if key_id >= n_keys:
        return
    
    # Load inv_levels and scale
    inv_levels = tl.load(inv_levels_ptr)
    scale = tl.load(scales_ptr + key_id)
    
    # Accumulate dot product
    acc = 0.0
    
    for d in range(0, dim, BLOCK_SIZE):
        offsets = d + tl.arange(0, BLOCK_SIZE)
        mask = offsets < dim
        
        # Load qk (int8) and query (float32)
        # Use int32 as intermediate to avoid sign-extension bugs
        qk_int = tl.load(qk_ptr + key_id * dim + offsets, mask=mask, other=0).to(tl.int32)
        qk_float = qk_int.to(tl.float32)
        q = tl.load(query_ptr + offsets, mask=mask, other=0.0)
        
        # Accumulate
        acc += tl.sum(qk_float * q)
    
    # Scale
    score = acc * inv_levels * scale
    
    # Store result
    tl.store(scores_ptr + key_id, score)


class PerDimScorerFused:
    """Fused per-dimension quantized scorer using Triton."""
    
    def __init__(self, dim: int, bits: int):
        self.dim = dim
        self.bits = bits
        self.levels = (1 << bits) - 1
        
        # Choose block size based on dim
        if dim <= 64:
            self.BLOCK_SIZE = 64
        elif dim <= 128:
            self.BLOCK_SIZE = 128
        else:
            self.BLOCK_SIZE = 256
    
    def quantize_keys(self, keys: torch.Tensor):
        """Quantize keys: uint8 codes + per-dim min/max statistics.
        
        Args:
            keys: [n_keys, dim] float32
            
        Returns:
            codes: [n_keys, dim] uint8
            k_norms: [n_keys] float32
            dim_mins: [dim] float32
            dim_ranges: [dim] float32
        """
        # Normalize keys to unit vectors
        k_norms = keys.norm(dim=1, keepdim=True)
        k_unit = keys / k_norms
        
        # Compute per-dimension min/max across all keys
        dim_mins = k_unit.min(dim=0).values
        dim_maxs = k_unit.max(dim=0).values
        dim_ranges = (dim_maxs - dim_mins).clamp(min=1e-6)
        
        # Quantize: codes = round((k_unit - min) / range * levels)
        k_unit_shifted = k_unit - dim_mins.unsqueeze(0)
        k_unit_normalized = k_unit_shifted / dim_ranges.unsqueeze(0)
        codes = torch.round(k_unit_normalized * self.levels).clamp(0, self.levels - 1).to(torch.uint8)
        
        return codes, k_norms.squeeze(1), dim_mins, dim_ranges
    
    def prepare_query(self, query: torch.Tensor, dim_mins: torch.Tensor, dim_ranges: torch.Tensor):
        """Precompute scaled_query for fused scoring.
        
        Args:
            query: [dim] float32
            dim_mins: [dim] float32
            dim_ranges: [dim] float32
            
        Returns:
            scaled_query: [dim] float32
            bias: float32 scalar
        """
        # scaled_query = query * dim_ranges / levels
        scaled_query = query * dim_ranges / self.levels
        
        # bias = sum(query * dim_mins)
        bias = (query * dim_mins).sum()
        
        return scaled_query, bias
    
    def score(self, codes: torch.Tensor, k_norms: torch.Tensor, 
              scaled_query: torch.Tensor, bias: torch.Tensor) -> torch.Tensor:
        """Fused scoring: dequantize + dot product in one kernel.
        
        Args:
            codes: [n_keys, dim] uint8
            k_norms: [n_keys] float32
            scaled_query: [dim] float32
            bias: float32 scalar
            
        Returns:
            scores: [n_keys] float32
        """
        n_keys = codes.shape[0]
        scores = torch.empty(n_keys, device=codes.device, dtype=torch.float32)
        
        # Convert bias to tensor for Triton
        bias_tensor = torch.tensor([bias], device=codes.device, dtype=torch.float32)
        
        # Launch kernel
        per_dim_score_kernel[(n_keys,)](
            codes, k_norms, scaled_query, bias_tensor, scores,
            n_keys, self.dim,
            BLOCK_SIZE=self.BLOCK_SIZE,
        )
        
        return scores


class PerKeyScorerFused:
    """Fused per-key quantized scorer using Triton."""
    
    def __init__(self, dim: int, bits: int):
        self.dim = dim
        self.bits = bits
        # For per-key quantization, we normalize to [-1, 1], so use signed levels
        self.levels = (1 << (bits - 1)) - 1
        
        if dim <= 64:
            self.BLOCK_SIZE = 64
        elif dim <= 128:
            self.BLOCK_SIZE = 128
        else:
            self.BLOCK_SIZE = 256
    
    def quantize_keys(self, keys: torch.Tensor):
        """Quantize keys: int8 codes + per-key scale.
        
        Args:
            keys: [n_keys, dim] float32
            
        Returns:
            qk: [n_keys, dim] int8
            scales: [n_keys] float32
        """
        # Compute per-key max absolute value
        scales = keys.abs().max(dim=1).values.clamp(min=1e-6)
        
        # Quantize: qk = round(key / scale * levels)
        qk_float = torch.round(keys / scales.unsqueeze(1) * self.levels)
        qk = qk_float.clamp(-self.levels, self.levels).to(torch.int8)
        
        return qk, scales
    
    def score(self, qk: torch.Tensor, scales: torch.Tensor, query: torch.Tensor) -> torch.Tensor:
        """Fused scoring: dequantize + dot product in one kernel.
        
        Args:
            qk: [n_keys, dim] int8
            scales: [n_keys] float32
            query: [dim] float32
            
        Returns:
            scores: [n_keys] float32
        """
        n_keys = qk.shape[0]
        scores = torch.empty(n_keys, device=qk.device, dtype=torch.float32)
        
        # Convert inv_levels to tensor for Triton
        inv_levels = torch.tensor([1.0 / self.levels], device=qk.device, dtype=torch.float32)
        
        # Launch kernel
        per_key_score_kernel[(n_keys,)](
            qk, scales, query, inv_levels, scores,
            n_keys, self.dim,
            BLOCK_SIZE=self.BLOCK_SIZE,
        )
        
        return scores
