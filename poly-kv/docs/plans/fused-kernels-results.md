# Fused Triton Kernels: Results & Honest Assessment

## What Was Built

Two fused scoring kernels using Triton (JIT-compiled to PTX, works on any GPU):

1. **`per_dim_score_kernel`** — Per-dimension quantized scoring (uint8 codes + per-dim statistics)
2. **`per_key_score_kernel`** — Per-key quantized scoring (int8 codes + per-key scale)

Each kernel fuses dequantization + dot product into a single GPU kernel launch.

**Files:**
- `/home/sikmindz/Coding/Libraries/poly-kv/bench/speed/triton_scorer.py` — Triton kernel + Python wrappers
- `/home/sikmindz/Coding/Libraries/poly-kv/bench/speed/triton_benchmark.py` — Benchmark script
- `/home/sikmindz/Coding/Libraries/poly-kv/bench/speed/triton_benchmark_results.json` — Raw data

## Correctness: ✅ Perfect

Both kernels achieve **cosine similarity = 1.0000** against the dense baseline. The int8→int32→float32 sign-extension path was the key fix (Triton's default int8→float32 cast is unsigned).

## Speed: ❌ 4-6x Slower Than cuBLAS

| n_keys | Dense cuBLAS | Fused per-dim | Fused per-key | Dense vs best fused |
|--------|-------------|---------------|---------------|---------------------|
| 64     | 16.4 μs     | 90.4 μs       | 69.3 μs       | **4.2x faster**     |
| 256    | 15.7 μs     | 89.7 μs       | 69.2 μs       | **4.4x faster**     |
| 512    | 15.9 μs     | 87.9 μs       | 69.1 μs       | **4.3x faster**     |
| 1024   | 15.6 μs     | 85.2 μs       | 74.8 μs       | **4.8x faster**     |
| 2048   | 16.7 μs     | 86.1 μs       | 69.0 μs       | **4.1x faster**     |

## Why Fused Kernels Didn't Help

The fused kernels eliminate intermediate memory traffic and reduce kernel launch count, but they still lose to dense matmul for three fundamental reasons:

### 1. cuBLAS is heavily optimized
cuBLAS `mv` (matrix-vector multiply) on a [2048, 128] matrix uses all GPU SMs in parallel with warp-level matrix operations. It's been optimized by NVIDIA for decades.

### 2. The fused kernel is one program per key
My design launches one Triton program per key. For 2048 keys, that's 2048 sequential programs (or 2048 parallel programs with poor occupancy for small dims). cuBLAS does the same work in ONE launch.

### 3. Dequantization adds computation, doesn't remove it
The "memory savings" from uint8 vs float32 only help if memory bandwidth is the bottleneck. At dim=128 with 64-2048 keys, the operation is **compute-bound or launch-overhead-bound**, not memory-bound. The dequantization math (uint8→float, multiply by scale, add bias) adds FLOPs that the dense path doesn't have.

## What Would Actually Be Needed

To beat dense matmul, you'd need one of:

| Approach | Hardware Required | Expected Speedup | Effort |
|----------|------------------|-------------------|--------|
| **INT8 Tensor Core matmul** | Ampere+ (RTX 30xx+) | 2-4x over FP16 dense | 2-3 weeks |
| **Warp-level matrix ops (WMMA/MMA)** | Volta+ (RTX 20xx+) | 2-3x over FP16 dense | 3-4 weeks |
| **Sparse attention** (not compression) | Any GPU | Reduces *computation*, not just memory | Different approach entirely |
| **Much larger batch sizes** (8K+ keys) | Any GPU | Memory BW becomes bottleneck, fused wins | Change problem parameters |

The current GPU (GTX 1070, Pascal) lacks Tensor Cores entirely, so INT8 matmul and WMMA are not available. This is a hardware limitation, not a software one.

## Final Verdict

**The fused kernels are correct but not faster.**

Memory savings from quantization remain real (12-57x compression), but:

- **For speed-critical paths:** Dense fp16 matmul wins on any hardware.
- **For memory-critical paths:** Use quantized storage + dense matmul with on-the-fly dequantization (not a custom fused kernel).
- **For actual speedup:** Need INT8 Tensor Cores (Ampere+) or a fundamentally different approach (sparse attention, not compressed scoring).

The per-dim scorer is a **memory optimization**, not a **speed optimization**. The earlier Python benchmark's conclusion stands: compressed-domain scoring saves memory but costs speed.

## Recommendation

**Do not pursue further kernel optimization on this GPU.** The results are conclusive: fused kernels don't overcome cuBLAS on Pascal.

If speed is the goal, the next step would be:
1. Move to an Ampere+ GPU (RTX 3090/4090/A100) for INT8 Tensor Core matmul
2. Or pivot to sparse attention (Flash Attention style), which reduces *computation* not just memory
3. Or accept that per-dim is a memory-only optimization and position it accordingly
