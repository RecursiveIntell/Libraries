# Per-Dim Scorer Benchmark Analysis

## Executive Summary

**The per-dim scorer implementation is complete and correct, but the benchmark results reveal a critical finding: compressed-domain scoring is SLOWER than dense fp16 matmul at realistic context lengths.**

This contradicts the initial hypothesis that compressed scoring would be faster. The methods work correctly and save memory, but they do not provide a speed advantage without custom fused kernels.

## Benchmark Results

### Scoring-Only Benchmark (MSI, RTX 4090, dim=128)

| Method      | 64 keys     | 128 keys    | 256 keys    | 512 keys    |
|-------------|-------------|-------------|-------------|-------------|
| **dense**   | 1.23ms (0.05M k/s) | 0.051ms (2.5M k/s) | 0.049ms (5.2M k/s) | 0.047ms (11M k/s) |
| per-key-4   | 0.13ms (0.5M k/s) **10x faster** | 0.13ms (1.0M k/s) **0.4x** | 0.12ms (2.1M k/s) **0.4x** | 0.12ms (4.1M k/s) **0.4x** |
| per-key-8   | 0.12ms (0.6M k/s) **10x faster** | 0.12ms (1.0M k/s) **0.4x** | 0.12ms (2.1M k/s) **0.4x** | 0.12ms (4.4M k/s) **0.4x** |
| per-dim-4   | 0.18ms (0.4M k/s) **7x faster** | 0.16ms (0.8M k/s) **0.3x** | 0.16ms (1.6M k/s) **0.3x** | 0.16ms (3.2M k/s) **0.3x** |
| per-dim-8   | 0.17ms (0.4M k/s) **7x faster** | 0.16ms (0.8M k/s) **0.3x** | 0.16ms (1.6M k/s) **0.3x** | 0.16ms (3.1M k/s) **0.3x** |

**Key observations:**
- Quantized methods win ONLY at very small batches (64 keys): 7-10x speedup
- At realistic context lengths (128-512 keys), dense matmul is 2.5-3x FASTER
- Dense achieves 11M keys/sec at 512 keys vs 3-4M for quantized
- Per-key quantization is faster than per-dim (1.3-1.5x)
- Both methods achieve excellent quality (cos > 0.99, topk overlap > 0.88)

### Memory Efficiency

| Method    | Bytes/Key | Compression Ratio |
|-----------|-----------|-------------------|
| dense     | 256 B     | 1.0x (baseline)   |
| per-key-4 | 4.5 B     | 57x               |
| per-key-8 | 5.0 B     | 51x               |
| per-dim-4 | 6.5-21 B  | 12-39x            |
| per-dim-8 | 7.0-21 B  | 12-37x            |

**Memory savings are real:** 51-57x compression for per-key, 12-39x for per-dim.

## Why Compressed Scoring is Slower

The Python benchmark reveals the fundamental problem: **dequantization overhead dominates the computation.**

### Breakdown of Operations

**Dense matmul:**
```python
scores = q @ keys.T  # Single optimized cuBLAS call
```
- One fused operation
- Highly optimized (cuBLAS, Tensor Cores)
- Memory bandwidth: 256 bytes/key

**Quantized scoring:**
```python
# Per-key example
qk_float = qk.float()                    # int8 → fp32 conversion
recon = (qk_float / levels) * scales     # Elementwise multiply
scores = (recon * q).sum(dim=-1)         # Dot product
```
- Multiple kernel launches (type conversion, elementwise ops, reduction)
- Python interpreter overhead
- No fusion
- Memory bandwidth: 4.5-5 bytes/key (but overhead eats the savings)

**The bottleneck:** Each operation launches a separate CUDA kernel. The kernel launch overhead (~10-50μs each) dominates at small batch sizes and prevents GPU utilization at scale.

## What Would Be Needed for Speed

To make compressed scoring faster than dense, we need **fused CUDA kernels**:

### 1. Fused Dequantize-Dot Kernel
```cuda
__global__ void fused_perdim_score(
    const uint8_t* keys_codes,     // [n_keys, dim]
    const float* dim_mins,         // [dim]
    const float* dim_ranges,       // [dim]
    const float* k_norms,          // [n_keys]
    const float* query,            // [dim]
    float* scores,                 // [n_keys]
    int n_keys, int dim
) {
    // Single kernel: dequantize + dot product + scale
    // No intermediate buffers
    // Shared memory for query broadcast
}
```

### 2. Top-K Selection Fusion
Fuse scoring with top-k selection to avoid writing all scores to global memory:
```cuda
// Score and accumulate top-k in registers
// Only write top-k indices to global memory
```

### 3. Quantized Matmul (INT8 Tensor Cores)
Modern GPUs (Ampere+) have INT8 Tensor Cores that can do:
```
D = A_int8 @ B_int8  // Direct integer matmul
```
This would require:
- Storing keys as INT8 (not UINT8)
- Adjusting per-dim quantization to signed
- Using `torch.matmul(q_int8, keys_int8)` with proper scaling

### Estimated Speedup with Fused Kernels
Based on similar work (QServe, AWQ, GPTQ):
- Fused dequantize-dot: **3-5x speedup** over current Python
- INT8 Tensor Cores: **8-12x speedup** over fp16 dense
- Combined with top-k fusion: **10-15x speedup**

This would make compressed scoring competitive or faster than dense at all batch sizes.

## Quality Analysis

Both methods achieve excellent ranking quality:

| Method    | Rank Cosine | Top-K Overlap (k=32) |
|-----------|-------------|----------------------|
| per-key-4 | 0.9933-0.9936 | 0.88-1.00          |
| per-key-8 | 1.0000      | 1.00                 |
| per-dim-4 | 0.9930-0.9949 | 0.91-1.00          |
| per-dim-8 | 1.0000      | 1.00                 |

**8-bit methods achieve perfect ranking** (cosine = 1.0, overlap = 1.0) because quantization error is small relative to score differences.

**4-bit methods have slight degradation** but still very good (cosine > 0.99, overlap > 0.88).

### When Quality Matters
- For **candidate generation** (top-k selection): Both 4-bit and 8-bit work well
- For **exact attention** (softmax weighting): Use 8-bit or exact fallback

## Recommendations

### Use Cases Where Per-Dim Makes Sense

1. **Memory-constrained environments** (edge devices, long contexts)
   - 51x memory savings is real
   - Speed doesn't matter if you can't fit the KV cache

2. **Very small batch sizes** (< 64 keys)
   - Quantized methods are 7-10x faster here
   - Useful for incremental decoding

3. **Candidate generation with exact fallback**
   - Use per-dim to select top-k candidates
   - Decode and re-rank with exact attention
   - Amortizes dequantization cost over fewer keys

### Use Cases Where Per-Dim Does NOT Make Sense

1. **General-purpose attention replacement**
   - Dense matmul is faster at realistic batch sizes
   - No speed benefit, only memory benefit

2. **Latency-critical inference**
   - Python overhead makes it 2-3x slower
   - Would need custom CUDA kernels

3. **When you have GPU memory to spare**
   - Memory savings don't justify speed loss

## Conclusion

**The per-dim scorer is a correct, high-quality implementation that saves memory but does not provide speed benefits in its current Python form.**

### What We Achieved
✅ Correct asymmetric per-dim quantization (Rust + Python)  
✅ Excellent ranking quality (cosine > 0.99, 8-bit = perfect)  
✅ Significant memory savings (12-57x compression)  
✅ Quality gates pass (256/512 tokens, PPL delta < 1%, cosine p05 > 0.995)  
✅ No-std/embedded compatible (Rust implementation)  

### What We Did NOT Achieve
❌ Speed advantage over dense attention  
❌ Competitive latency at realistic batch sizes  
❌ Production-ready performance without custom kernels  

### Next Steps (If Pursuing Speed)

1. **Write fused CUDA kernels** (estimated 2-3 weeks)
   - Fused dequantize-dot kernel
   - Top-k selection fusion
   - Expected speedup: 3-5x over current Python

2. **Explore INT8 Tensor Cores** (estimated 1-2 weeks)
   - Adjust quantization to signed INT8
   - Use `torch.matmul` with INT8 inputs
   - Expected speedup: 8-12x over fp16 dense

3. **Benchmark against production systems** (1 week)
   - Compare with vLLM, TensorRT-LLM quantized attention
   - Validate against Flash Attention 2
   - Measure end-to-end inference speedup

### Final Verdict

**The per-dim scorer is a research success but a production dead end without custom kernels.**

- For **memory-constrained** applications: Use it (51x compression is real)
- For **speed-critical** applications: Don't use it (2-3x slower)
- For **embedded/edge** applications: Use Rust implementation (no-std compatible)
- For **general inference**: Stick with dense attention or wait for fused kernels

The implementation is correct and the quality is excellent. The only missing piece is the performance engineering to make it fast.

## Files Modified

### Rust (`compressed-scorer`)
- `src/per_dim_impl.rs`: Asymmetric per-dim scorer implementation
- `src/lib.rs`: Export per_dim module
- `src/adaptive_budget.rs`: Fix no_std vec! macro
- `src/attention_cache.rs`: Fix no_std vec! macro
- `src/integration_tests.rs`: Fix unused imports
- `tests/working_set.rs`: Update test
- `README.md`: Update verification receipts

### Python (`poly-kv/scripts`)
- `compressed_attention_forward_ppl.py`: Add `--scorer per-dim` option
- `bench/speed/scoring_only.py`: Scoring-only benchmark
- `bench/speed/per_dim_vs_others.py`: Full forward benchmark (too slow)

### Documentation
- `docs/plans/per-dim-scorer-roadmap.md`: Implementation roadmap
- `docs/plans/per-dim-scorer-benchmark-analysis.md`: This analysis
- `docs/gates/per-dim-scorer-receipt.md`: Quality gate results

### Benchmark Results
- `bench/speed/scoring-only-results.json`: Raw benchmark data
- `bench/ppl/smollm2-1.7b/wikitext-2/compressed-forward-perdim-adaptive-256-unit-q8-t98-rk256/regression/receipt.json`: 256-token gate
- `bench/ppl/smollm2-1.7b/wikitext-2/compressed-forward-perdim-adaptive-512-unit-q8-t98-rk256/regression/receipt.json`: 512-token gate
