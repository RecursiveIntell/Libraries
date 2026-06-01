# Poly-KV GPU Benchmark Results — 2026-06-01

## TL;DR

The poly-kv pool build path was refactored from per-vector encode to
batched per-(layer,head) encode, exposing the fib-quant `encode_batch()`
GPU dispatch. **The dispatch is real and works** — receipts now report
`backend: "gpu"` only when the per-call probe says the batch cleared
the threshold (n≥16, d≥64).

**But the headline GPU win is small**: the encode path is dominated by
the Lloyd-Max codebook lookup (`nearest_index` ×32 per vector) which
runs on CPU even with `--features gpu`. Only the Hadamard rotation
itself is GPU-accelerated.

## What changed

1. `KVecCodec` trait gained `encode_batch / decode_batch / is_gpu_accelerated / is_gpu_accelerated_for(n, d)`.
2. `FibQuantAdapter` overrides all four. `encode_batch` builds one
   `FibQuantizer` (deterministic from profile) and threads through
   `fib_quant.encode_batch`.
3. `pool.rs` collects per-(layer, head) K and V vectors across all
   tokens, dispatches two batched `encode_batch` calls per layer.
4. Receipt `backend` is now driven by `codec.is_gpu_accelerated_for(batch_n, head_dim)`,
   not `cfg!(feature = "gpu")`.
5. fib-quant: fixed `FibQuantError::Internal` → `NumericalFailure`
   (variant didn't exist) so `--features gpu` actually compiles.
6. gpu-backend: removed unused `total_scalars` variable.
7. New test `test_pool_build_digest_invariant_across_corpora_size`
   guards against the "receipt says gpu, code did cpu" failure mode.
8. New example `poly_kv_gpu_bench` covers nomic 768-dim and qwen3
   2560-dim, three corpus sizes, with split timing.

## Receipt honesty invariants

- `test_pool_build_digest_invariant_across_corpora_size`: a 4-doc
  corpus must report `backend: "cpu"` even with `--features gpu`
  because the per-(layer, head) batch is below the threshold.
- The bench's `gpu_dispatch` column is the static probe; the
  `backend` column is the receipt. The bench asserts they match.

## Numbers

All times in milliseconds, single-threaded release build.

| Machine | Shape | n | wall | encode_only | codebook | batch | gpu_dispatch | backend |
|---|---|---|---|---|---|---|---|---|
| **msi i7-6700HQ + GTX 1070, --features gpu** | nomic 768 | 4 | 454 | 442 | 12 | 48 | yes | gpu |
| msi i7-6700HQ + GTX 1070, --features gpu | nomic 768 | 20 | 1282 | 1319 | -37 | 240 | yes | gpu |
| msi i7-6700HQ + GTX 1070, --features gpu | nomic 768 | 80 | 4398 | 4425 | -27 | 960 | yes | gpu |
| msi i7-6700HQ + GTX 1070, --features gpu | qwen3 2560 | 4 | 1444 | 1430 | 14 | 16 | yes | gpu |
| msi i7-6700HQ + GTX 1070, --features gpu | qwen3 2560 | 20 | 4005 | 3964 | 41 | 80 | yes | gpu |
| msi i7-6700HQ + GTX 1070, --features gpu | qwen3 2560 | 80 | 13213 | 13377 | -164 | 320 | yes | gpu |
| **msi i7-6700HQ + GTX 1070, CPU only** | nomic 768 | 4 | 449 | 433 | 16 | 48 | no | cpu |
| msi i7-6700HQ + GTX 1070, CPU only | nomic 768 | 20 | 1292 | 1278 | 14 | 240 | no | cpu |
| msi i7-6700HQ + GTX 1070, CPU only | nomic 768 | 80 | 4386 | 4520 | -134 | 960 | no | cpu |
| msi i7-6700HQ + GTX 1070, CPU only | qwen3 2560 | 4 | 1476 | 1461 | 15 | 16 | no | cpu |
| msi i7-6700HQ + GTX 1070, CPU only | qwen3 2560 | 20 | 4007 | 3986 | 21 | 80 | no | cpu |
| msi i7-6700HQ + GTX 1070, CPU only | qwen3 2560 | 80 | 13491 | 13609 | -118 | 320 | no | cpu |
| **laptop Ryzen 7 7730U (APU), CPU** | nomic 768 | 4 | 325 | 313 | 12 | 48 | no | cpu |
| laptop Ryzen 7 7730U (APU), CPU | nomic 768 | 20 | 943 | 969 | -26 | 240 | no | cpu |
| laptop Ryzen 7 7730U (APU), CPU | nomic 768 | 80 | 3205 | 3153 | 52 | 960 | no | cpu |
| laptop Ryzen 7 7730U (APU), CPU | qwen3 2560 | 4 | 992 | 958 | 34 | 16 | no | cpu |
| laptop Ryzen 7 7730U (APU), CPU | qwen3 2560 | 20 | 2718 | 2749 | -31 | 80 | no | cpu |
| laptop Ryzen 7 7730U (APU), CPU | qwen3 2560 | 80 | 9481 | 9316 | 165 | 320 | no | cpu |

The "codebook" column being sometimes negative reflects the noise of
comparing two timed loops back-to-back; it is not a real negative cost.
Treat |codebook| < 200ms as "codebook is amortized into the encode loop."

## Findings

### 1. GPU dispatch is real and works

Receipts correctly report `backend: "gpu"` for the qwen3 n=4 case
(batch=16, exactly at threshold) and larger. They correctly report
`"cpu"` for any case where the per-call probe says no.

### 2. GPU win is small (~1-3% on msi)

The end-to-end pool build is dominated by **codebook lookup**, not
Hadamard rotation. The fib-quant `encode_batch` GPU path only
accelerates the Hadamard step. The `nearest_index` loop in
`finish_batch_encode` runs `d/k = 32` times per vector, all on CPU.

For nomic 768 (n=80): 4398ms GPU vs 4386ms CPU → 0.3% faster.
For qwen3 2560 (n=80): 13213ms GPU vs 13491ms CPU → 2.1% faster.

The earlier gpu-backend isolated kernel numbers (99K Hadamard vec/s)
are real, but the **Hadamard step is not the bottleneck** in
end-to-end pool build. The win will only show up when batch sizes
are large enough that the per-vector overhead of the Hadamard launch
is hidden — and even then, the codebook lookup dominates.

### 3. The laptop APU beats msi's CPU

Laptop is a Ryzen 7 7730U (Zen 3, 2022, ~4.5GHz boost). msi is an
i7-6700HQ (Skylake, 2015, 2.6GHz base). The APU is **30-40% faster**
than msi's CPU on these benchmarks. The GPU on msi only barely
catches up to the i7-6700HQ.

### 4. The Hadamard is not where the time is

To actually win with GPU on poly-kv, we'd need to also accelerate
the codebook lookup. The Lloyd-Max codebook is a fixed small table
(32 entries × k floats). A specialized CUDA kernel for "scan
codebook and pick min-distance index" would be the next step —
analogous to the lloyd_max_encode kernel that already exists in
gpu-backend for the *encoding* side. The *decoding* side is what's
needed in `finish_batch_encode`.

## What this changes for the public narrative

This is exactly the kind of finding the doctrine says to publish
honestly: the GPU pipeline works, the receipts are honest, the
numbers say the win is small for poly-kv pool build as currently
scoped. Don't claim "poly-kv is X× faster on GPU" — claim "poly-kv
GPU pipeline is wired with honest receipt accounting; current end-to-end
workload is dominated by codebook lookup and the GPU win is small.
Next step: GPU-accelerate codebook lookup."

## Reproduce

```bash
# on msi
cd ~/Coding/Libraries/poly-kv
cargo run --release --example poly_kv_gpu_bench --features gpu

# on laptop
cd ~/Coding/Libraries/poly-kv
cargo run --release --example poly_kv_gpu_bench
```
