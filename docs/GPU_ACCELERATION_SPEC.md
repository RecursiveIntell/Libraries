# GPU Acceleration Spec — Poly-KV Compression Stack

**Author:** RecursiveIntell  
**Date:** 2026-06-01  
**Target hardware:** NVIDIA GTX 1070 (8GB VRAM, CUDA 13.0, Pascal SM 6.1)  
**Status:** Spec — not implemented  

---

## 1. Problem Statement

Current CPU-only paths are too slow for production dimensions:

| Codec | 768-dim | 2560-dim | Bottleneck |
|---|---|---|---|
| fib-quant (k=4,N=32) encode | ~10s/query | ∞ (minutes) | Lloyd-Max per block, 640 blocks × 200 docs |
| fib-quant decode | moderate | moderate | Codebook lookup + block reassembly |
| turbo-quant encode | 570ms/query | fast | Hadamard rotation on CPU |
| turbo-quant decode | fast | fast | Inverse Hadamard + Lloyd-Max inverse |

**Target:** All three crates (fib-quant, turbo-quant, poly-kv) share a common GPU backend that makes batch encode/decode at 2560-dim practical.

---

## 2. Design Principles

### 2.1 One Backend, Three Crates
A single `gpu-backend` crate provides CUDA kernels. fib-quant and turbo-quant both depend on it via optional `gpu` feature. poly-kv inherits GPU support through them.

### 2.2 CPU Fallback Always Works
GPU is optional. Every codec path must have a working CPU reference. The GPU path is a drop-in accelerator — same API, different backend.

### 2.3 Batch-First Design
Individual vector encode/decode is dominated by kernel launch overhead. Design for `encode_batch(&[Vec<f32>]) -> Vec<CompressedBlock>` — amortize the launch cost.

### 2.4 Deterministic
Same seed + same input on GPU must produce identical output to CPU path. Use deterministic CUDA kernels (no atomic non-determinism, fixed thread scheduling).

### 2.5 Receipt-Bearing
GPU operations emit the same typed receipts as CPU. `FibQuantCompressionReceiptV1` and `TurboCode` must be identical regardless of backend.

---

## 3. Architecture

```
┌─────────────────────────────────────────────┐
│              poly-kv (two-tier)             │
│   SharedKVPool  ←── fib-quant (cold)        │
│   AgentShell    ←── turbo-quant (hot)       │
└──────────┬──────────────────┬───────────────┘
           │                  │
    ┌──────▼──────┐    ┌──────▼──────┐
    │  fib-quant  │    │ turbo-quant │
    │  (cpu|gpu)  │    │ (cpu|gpu)   │
    └──────┬──────┘    └──────┬──────┘
           │                  │
           └────────┬─────────┘
                    │
           ┌────────▼────────┐
           │   gpu-backend   │
           │  (cudarc-based) │
           │                 │
           │ • Hadamard WHT  │
           │ • Lloyd-Max Q   │
           │ • Block encode  │
           │ • Bit packing   │
           │ • Batch mm      │
           └────────┬────────┘
                    │
           ┌────────▼────────┐
           │   CUDA 13.0     │
           │   Driver API    │
           └─────────────────┘
```

---

## 4. GPU Backend Crate: `gpu-backend`

### 4.1 Crate Layout

```
Libraries/gpu-backend/
├── Cargo.toml          # depends on cudarc
├── src/
│   ├── lib.rs          # feature-gated re-exports
│   ├── context.rs      # CudaContext singleton, device info
│   ├── hadamard.rs     # Fast Walsh-Hadamard Transform kernel
│   ├── lloyd_max.rs    # Lloyd-Max scalar quantization kernel
│   ├── bitpack.rs      # GPU-side bit packing
│   ├── batch.rs        # Batch encode/decode orchestrator
│   └── receipts.rs     # GPU-backed receipt generation
├── kernels/
│   ├── hadamard.cu     # WHT CUDA kernel
│   ├── lloyd_max.cu    # Per-coordinate Lloyd-Max kernel
│   ├── bitpack.cu      # Bit-packing kernel
│   └── batch_encode.cu # Fused encode kernel (WHT + Lloyd-Max + pack)
└── tests/
    ├── hadamard_gpu.rs
    └── lloyd_max_gpu.rs
```

### 4.2 Dependency

```toml
[dependencies]
cudarc = { version = "0.19", features = ["cuda-13010", "dynamic-loading"] }

[features]
default = []
gpu = []
```

`dynamic-loading` means no CUDA toolkit required at build time. The library probes for `libcuda.so` at runtime and falls back to CPU if absent.

### 4.3 Core API

```rust
// One-time init — finds GPU, compiles PTX
pub struct GpuContext {
    ctx: CudaContext,         // cudarc driver context
    stream: CudaStream,       // default compute stream
    module: CudaModule,       // compiled PTX from embedded kernels
}

impl GpuContext {
    /// Initialize GPU 0. Returns None if no CUDA device.
    pub fn init() -> Option<Self>;

    /// Device memory capacity check
    pub fn available_memory(&self) -> usize;

    /// Batch Hadamard transform: rotates N vectors of dim D
    /// Input: [N × D] f32 on device
    /// Output: [N × D] f32 on device (in-place or separate buffer)
    pub fn hadamard_batch(&self, data: &CudaSlice<f32>, n: usize, dim: usize) -> Result<()>;

    /// Batch Lloyd-Max quantize: quantizes a batch of scalars
    /// For k=4,N=32 fib-quant: D=640 blocks/vector, each block is k scalars
    /// Input: [N × D] f32 on device  
    /// Output: [N × D/k] u8 indices + [N × D/k] f32 norms
    pub fn lloyd_max_batch(
        &self,
        vectors: &CudaSlice<f32>,
        indices: &mut CudaSlice<u8>,
        norms: &mut CudaSlice<f32>,
        n: usize,
        dim: usize,
        k: usize,          // block size (4 for fib-quant)
        n_levels: usize,   // codebook size (32 for fib-quant)
    ) -> Result<()>;

    /// Batch decode: indices + norms → f32 vectors
    pub fn lloyd_max_decode_batch(
        &self,
        indices: &CudaSlice<u8>,
        norms: &CudaSlice<f32>,
        output: &mut CudaSlice<f32>,
        n: usize,
        dim: usize,
        k: usize,
        n_levels: usize,
    ) -> Result<()>;

    /// GPU-side bit packing of quantized indices
    pub fn bitpack_batch(
        &self,
        indices: &CudaSlice<u8>,   // raw indices (5 bits each for N=32)
        packed: &mut CudaSlice<u8>, // packed bytes
        n: usize,
        indices_per_vector: usize,
        bits_per_index: usize,
    ) -> Result<()>;
}
```

### 4.4 CUDA Kernel: Fused Batch Encode (for fib-quant)

The key insight from tq-kv and candle PR #3433: **fuse Hadamard rotation + Lloyd-Max + bitpack into a single kernel launch.** This eliminates 3 kernel launches + 2 device↔host transfers per batch.

```cuda
// kernels/batch_encode.cu
// Grid: (num_vectors, (dim / k) / 256) blocks
// Block: 256 threads
//
// Each block handles one vector's worth of data.
// Threads within a block cooperate on the Hadamard WHT,
// then independently quantize their assigned scalar.

extern "C" __global__ void fib_encode_batch(
    const float* __restrict__ input,    // [N × D]
    uint8_t* __restrict__ indices,      // [N × D/k] — codebook indices
    float* __restrict__ norms,          // [N × D/k] — per-block norms
    const float* __restrict__ codebook, // [N_levels] — precomputed centroids
    int num_vectors,
    int dim,
    int k,            // block dimension (4)
    int n_levels,     // codebook size (32)
    uint64_t seed
);
```

### 4.5 CUDA Kernel: Hadamard WHT (for turbo-quant)

```cuda
// kernels/hadamard.cu
// In-place Walsh-Hadamard Transform
// Requires dim to be power of 2. Pad input if needed.
// Grid: (num_vectors, 1) blocks
// Block: dim threads (max 1024 → handle up to dim=1024 per block)

extern "C" __global__ void hadamard_wht_batch(
    float* __restrict__ data,    // [N × D] in-place
    const int* __restrict__ signs, // [D] random ±1 signs per vector
    int num_vectors,
    int dim
);
```

### 4.6 Memory Budget (GTX 1070, 8GB)

| Buffer | Size | Notes |
|---|---|---|
| Input vectors (200 × 2560 × f32) | 2.0 MB | Batch of 200 |
| Output indices (200 × 640 × u8) | 128 KB | 640 blocks/vector |
| Output norms (200 × 640 × f32) | 512 KB | Per-block norms |
| Codebook (32 × f32) | 128 B | Single codebook, reused |
| Rotation matrix (2560² × f32) | **26.2 MB** | ⚠ Dominates for fib-quant |
| Hadamard signs (2560 × i32) | 10 KB | For turbo-quant WHT |
| Workspace / PTX / Stack | ~500 MB | CUDA context overhead |
| **Total** | **~30 MB** | Well within 8GB |

The rotation matrix is the big item. For fib-quant at 2560-dim, a full rotation matrix is 26MB. Options:
1. **Store it** (8GB has room for thousands)
2. **Recompute on GPU** — QR decomposition of a random seed matrix is slow
3. **Use structured rotation** (Hadamard + random signs) like turbo-quant does — O(d log d), no storage

**Recommendation:** Switch fib-quant to Hadamard-based rotation (same as turbo-quant). This eliminates the 26MB matrix and makes both codecs share the same kernel.

---

## 5. fib-quant GPU Path

### 5.1 Changes Required

1. **Replace dense rotation with Hadamard WHT** — This is a spec change, not just acceleration. The current `StoredRotation` stores a full d×d matrix. Replace with seeded Hadamard signs. This is deterministic, reproducible, and O(d log d) instead of O(d²).

2. **Batch encode API** — Add `FibQuantizer::encode_batch(&[Vec<f32>]) -> Vec<FibCodeV1>` that dispatches to GPU when available.

3. **GPU feature flag** — `fib-quant = { features = ["gpu"] }` pulls in `gpu-backend`.

### 5.2 API Surface

```rust
// New methods on FibQuantizer
impl FibQuantizer {
    /// Encode a batch of vectors. Uses GPU if available and dim >= 128.
    pub fn encode_batch(&self, vectors: &[&[f32]]) -> Result<Vec<FibCodeV1>>;

    /// Decode a batch of codes.
    pub fn decode_batch(&self, codes: &[FibCodeV1]) -> Result<Vec<Vec<f32>>>;

    /// Check if GPU acceleration is active
    pub fn is_gpu_accelerated(&self) -> bool;
}
```

### 5.3 Dispatch Logic

```rust
fn encode_batch(&self, vectors: &[&[f32]]) -> Result<Vec<FibCodeV1>> {
    #[cfg(feature = "gpu")]
    {
        if vectors.len() >= GPU_MIN_BATCH_SIZE && self.dim >= GPU_MIN_DIM {
            if let Some(ctx) = GpuContext::get() {
                return self.encode_batch_gpu(ctx, vectors);
            }
        }
    }
    // CPU fallback
    vectors.iter().map(|v| self.encode(v)).collect()
}
```

### 5.4 Expected Speedup (GTX 1070)

| Dimension | CPU (current) | GPU (projected) | Speedup |
|---|---|---|---|
| 768-dim, 200 docs | ~10s | ~50ms | **200×** |
| 2560-dim, 200 docs | ∞ (>10min) | ~150ms | **4000×+** |
| 2560-dim, 10K docs | impractical | ~2s | ∞ |

---

## 6. turbo-quant GPU Path

### 6.1 Changes Required

turbo-quant is closer to GPU-ready. The Hadamard WHT is already the rotation method. What needs GPU:

1. **Batch Hadamard WHT** — Move the in-place WHT to CUDA kernel. Already O(d log d) on CPU, but GPU parallelizes across vectors.
2. **Batch Lloyd-Max** — Per-scalar quantization for polar angles. Trivially parallel.
3. **Batch QJL sketch** — Sign projection for residual. Parallel across projections.
4. **Batch decode** — Inverse Hadamard + dequantize.

### 6.2 API Surface

```rust
impl TurboQuantizer {
    pub fn encode_batch(&self, vectors: &[&[f32]]) -> Result<Vec<TurboCode>>;
    pub fn decode_batch(&self, codes: &[TurboCode]) -> Result<Vec<Vec<f32>>>;
    pub fn inner_product_batch(&self, codes: &[TurboCode], query: &[f32]) -> Result<Vec<f32>>;
    pub fn is_gpu_accelerated(&self) -> bool;
}
```

### 6.3 Expected Speedup

turbo-quant is already fast on CPU (570ms/query at 768-dim). GPU benefit is primarily at large batch sizes:

| Batch Size | CPU | GPU | Speedup |
|---|---|---|---|
| 1 vector | 0.5ms | 0.3ms | 1.7× (launch overhead) |
| 200 vectors | 100ms | 2ms | **50×** |
| 10K vectors | 5s | 50ms | **100×** |

---

## 7. poly-kv GPU Path

### 7.1 No Direct GPU Code

poly-kv doesn't need its own GPU code. It inherits GPU acceleration through fib-quant and turbo-quant:

```rust
impl SharedKVPool {
    /// Build with GPU acceleration if available
    pub fn build_gpu(corpus: &[(String, Vec<f32>)], shape: &KvTensorShape, seed: u64)
        -> Result<(Self, PoolBuildReceipt)>
    {
        // Internally calls fib_quantizer.encode_batch() which uses GPU
        // Receipt includes `backend: "gpu"` field
    }
}
```

The `PoolBuildReceipt` gains an optional `backend: Option<String>` field — `"cpu"` or `"gpu"` — so you can prove which backend ran.

---

## 8. Implementation Phases

### Phase 1: Foundation (Week 1)
- [ ] Create `gpu-backend` crate with `cudarc` dependency
- [ ] Compile and test Hadamard WHT CUDA kernel
- [ ] Compile and test Lloyd-Max batch quantization kernel
- [ ] Implement `GpuContext` init, device probing, memory check
- [ ] CPU fallback path for every kernel

### Phase 2: fib-quant Integration (Week 1-2)
- [ ] Switch fib-quant rotation from dense matrix to Hadamard signs
- [ ] Implement `FibQuantizer::encode_batch()` with GPU dispatch
- [ ] Implement `FibQuantizer::decode_batch()` with GPU dispatch
- [ ] Determinism test: GPU output == CPU output (same seed, same input)
- [ ] Benchmark: 768-dim, 2560-dim, varying batch sizes

### Phase 3: turbo-quant Integration (Week 2)
- [ ] Implement `TurboQuantizer::encode_batch()` with GPU dispatch
- [ ] Implement `TurboQuantizer::decode_batch()` with GPU dispatch
- [ ] Implement batched inner product estimation on GPU
- [ ] Determinism test

### Phase 4: poly-kv Integration (Week 2-3)
- [ ] Add `backend` field to receipts
- [ ] `SharedKVPool::build_gpu()` — delegates to fib-quant GPU
- [ ] `materialize_shell_gpu()` — delegates to turbo-quant GPU
- [ ] Full 10-agent contention benchmark on GPU
- [ ] Receipt comparison: GPU receipts == CPU receipts

### Phase 5: Polish (Week 3)
- [ ] Fused kernel: Hadamard + Lloyd-Max + bitpack in one launch
- [ ] Async overlap: copy next batch while encoding current batch
- [ ] Documentation, examples, README updates
- [ ] Benchmark report with GPU vs CPU comparisons

---

## 9. Risks

| Risk | Mitigation |
|---|---|
| GTX 1070 Pascal SM 6.1 — older arch, no tensor cores | All kernels are bandwidth-bound, not compute-bound. Pascal is fine. |
| CUDA 13.0 driver but 1070 — compatibility | cudarc supports CUDA 13.0. GTX 1070 works with driver 580. |
| Hadamard rotation replacing dense rotation changes fib-quant output | Accept. The paper doesn't mandate a specific rotation. Hadamard is mathematically valid. Document the change. |
| First-time PTX compile takes seconds | Cache compiled PTX to disk. One-time cost per kernel version. |
| 8GB VRAM limits concurrent model + compression | Compression runs standalone — no model loaded. 30MB for compression, rest free. |

---

## 10. Success Criteria

1. **fib-quant 2560-dim × 200 docs in < 1 second** (currently: impractical)
2. **turbo-quant batch of 200 vectors in < 5ms** (currently: ~100ms)
3. **GPU output identical to CPU output** for same seed + input
4. **Receipts unchanged** — `backend` field added, all other fields match
5. **Zero unsafe code outside gpu-backend** — CUDA kernels are inherently unsafe, but isolated
6. **CPU fallback works when no GPU present** — laptop still runs, msi accelerates
