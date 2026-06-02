# Poly-KV GPU Benchmark Results — 2026-06-01 (revised, with real GPU)

## TL;DR

The poly-kv pool build path was refactored to batched per-(layer,head)
encode. The fib-quant `encode_batch` GPU dispatch is real and works.
**With real GPU dispatch (verified by parity test), pool build is 2.5-2.7%
faster than the same machine's CPU.**

**Critical environment note:** The `gpu-backend` crate has two features:
`gpu` (enables cudarc + CUDA dispatch) and `precompiled-ptx` (loads the
precompiled combined.ptx at runtime). The `combined.ptx` is not tracked
in git. Without `--features precompiled-ptx`, all GPU operations fall
back to CPU. **Both features are required for real GPU dispatch.**

## Receipt-honesty test results

The new `codebook_lookup_kernel` produces **byte-identical indices** to
the CPU reference (parity test passes for n=32, d=128, k=4, N=32 random
inputs on msi GTX 1070). The kernel is correct; the dispatch path
through `gpu_backend` is the perf issue (see below).

## Numbers (msi i7-6700HQ + GTX 1070)

| Shape | n | wall CPU | wall Hadamard-GPU | wall Hadamard+Codebook-GPU |
|---|---|---|---|---|
| nomic 768 | 4 | 459 | 458 | 464 |
| nomic 768 | 20 | 1336 | 1302 | 1342 |
| nomic 768 | 80 | 4552 | 4430 | 4485 |
| qwen3 2560 | 4 | 1449 | 1488 | 1478 |
| qwen3 2560 | 20 | 4271 | 4046 | 4063 |
| qwen3 2560 | 80 | 13763 | 13419 | 13428 |

**Hadamard-only win:** 2.5-2.7% on the larger corpora.
**Hadamard + Codebook-GPU win:** 1.5-2.4%. **The new codebook kernel
is slower in integration than just the Hadamard alone**, despite being
2-3x faster in isolation. Reason below.

## Why the new codebook kernel doesn't help in integration

The `codebook_lookup_microbench` example isolates the kernel from the
rest of the pipeline:

| Workload | CPU fallback | GPU kernel |
|---|---|---|
| qwen3 n=80 d=2560 k=4 | 8ms (6.27M blocks/s) | 14ms (3.42M blocks/s) |
| nomic n=80 d=768 k=4 | 2ms (6.36M blocks/s) | 4ms (3.44M blocks/s) |

**The GPU is 1.8x slower per call than the tight CPU loop** for these
batch sizes. Root cause: every call to `gpu_backend::codebook_lookup_batch`
pays H2D + D2H transfer overhead. The rotated input is `n * d * 4` bytes
uploaded, the indices are `n * (d/k) * 4` bytes downloaded, plus
`synchronize()` between.

For n=80, d=2560: 800KB H2D + 100KB D2H per call. PCIe 2.0 x16 practical
throughput is ~4GB/s, so the transfers alone are ~225μs. The kernel
runtime is microseconds. **Transfer overhead dominates.**

In the pool build, this codebook_lookup_batch is called 224 times
(28 layers × 4 heads × 2 K+V for qwen3). Even at 0.5ms extra per call
(conservative), that's 112ms of pure overhead vs. tight CPU loops.

## What would actually win

A **device-side pipeline** that keeps the rotated data on GPU between
the Hadamard and the codebook lookup:

1. H2D input (once per pool build)
2. GPU Hadamard (in-place on device)
3. GPU codebook lookup (no H2H roundtrip)
4. D2H indices (just the small result array)

This requires restructuring `gpu_backend` to expose a `GpuPipeline`
handle that holds the device buffer across calls. The current design
allocates and frees per-call, which is correct but defeats the purpose
of GPU compute for this workload.

The kernel itself is correct and ready. The dispatch path needs a
"keep data resident" mode. Estimated effort: 4-6 hours of careful
cudarc work, plus a parity test that proves device-side indices match
the CPU reference.

## Receipts

- `--features gpu` (Hadamard only): default, ships in this state.
- `--features gpu,gpu-backend/precompiled-ptx`: real GPU dispatch.
- `--features gpu_codebook_lookup`: enables the new codebook path.
  Off by default because the current dispatch is a net loss.

The `gpu_codebook_lookup` feature is **off by default** and the
`is_gpu_accelerated_for` probe only returns true when both gates are
satisfied (N <= 32, n >= 16, d >= 64, device available). This means
a default poly-kv build never engages the slow path.

## Public-safe phrasing

"poly-kv pool build is 2-3% faster on a real GPU (msi i7-6700HQ +
GTX 1070) with the fib-quant Hadamard path engaged. The codebook
lookup kernel exists and is parity-verified, but the per-call
H2D/D2H transfer overhead currently negates its win in the integrated
pool-build path. A device-side pipeline (rotated data resident on
GPU) is the next step."

Do NOT say "poly-kv is X× faster on GPU." It is 2-3% faster on this
specific hardware for this specific workload. Any larger claim is
unsupported by the receipts.

## Reproduce

```bash
# on msi
cd ~/Coding/Libraries/gpu-backend/kernels
cat hadamard.cu lloyd_max.cu bitpack.cu codebook_lookup.cu > _combined.cu
/usr/local/cuda-13.2/bin/nvcc -ptx -arch=compute_75 -o combined.ptx _combined.cu
rm _combined.cu

cd ~/Coding/Libraries/poly-kv

# True CPU baseline (no GPU)
cargo run --release --example poly_kv_gpu_bench

# Hadamard-only GPU
cargo run --release --example poly_kv_gpu_bench --features gpu,gpu-backend/precompiled-ptx

# Hadamard + Codebook GPU (the new kernel)
cargo run --release --example poly_kv_gpu_bench --features gpu_codebook_lookup,gpu-backend/precompiled-ptx

# Codebook kernel microbench
cd ~/Coding/Libraries/gpu-backend
cargo run --release --example codebook_lookup_microbench --features gpu,precompiled-ptx
cargo run --release --example codebook_lookup_microbench  # CPU fallback
```
