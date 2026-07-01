# Per-Dim Scorer: Final Assessment & Next ROI

## What Exists (Verified)

1. **Rust `PerDimScorer`** — asymmetric per-dim quantization, no_std/alloc, 21 tests pass
2. **Rust `AttentionCache<S: CompressedScorer>`** — one-head compressed attention, no_std, ESP32-S3 target passes
3. **Rust `CompressedScorer` trait** — codec-agnostic, 3 implementations (per-dim, fib, turbo)
4. **Rust `CompressedScorerAdapter`** in scr-runtime-compression — wraps any scorer for runtime dispatch
5. **Python quality gates** — 256/512 tokens pass with 8-bit per-dim + generous budget
6. **Triton fused kernels** — correct (cos=1.0), but 4-6x slower than cuBLAS on GTX 1070

## What the Evidence Says

| Claim | Verdict |
|-------|---------|
| Memory savings | ✅ Real: 12-57x compression vs fp16 |
| Quality | ✅ Real: cosine > 0.99 (4-bit), 1.0 (8-bit) |
| Speed on GPU | ❌ 4-6x slower than cuBLAS (even with fused Triton kernels) |
| Speed on embedded | ⚠️ Likely wins: ESP32 is memory-bound, no cuBLAS, less PSRAM traffic |
| Production-ready | ❌ Needs integration testing, real workload validation |

## Highest ROI Move: ESP32 Compressed Attention Demo

**Why this is the highest ROI:**

The per-dim scorer loses on GPU because cuBLAS is unbeatable for small matmuls. But on ESP32-S3:
- There is no cuBLAS. All math is software float.
- PSRAM reads dominate (100+ ns per access).
- Compressed keys read 1 byte/dim vs 4 bytes/dim (fp32) = 4x less memory traffic.
- The `AttentionCache` was designed exactly for this: score compressed, decode only top-k.
- All Rust code is already no_std compatible and compiles for xtensa-esp32s3.

**What the demo would prove:**
- Compressed attention works on a $5 microcontroller
- Memory savings enable longer context windows on limited hardware
- Speed is competitive or better because memory bandwidth is the bottleneck
- The entire compressed-scorer stack has a real use case

**What it would look like:**
- ESP32-S3 with a small attention head (dim=32 or 64)
- Pre-quantized keys stored in flash/PSRAM
- PerDimScorer scores compressed keys
- AttentionCache selects top-k and decodes only selected values
- Output drives an OLED display or LED

**Estimated effort:** 1-2 days (toolchain is already set up, code compiles for ESP32)

## Second ROI: Publish compressed-scorer to crates.io

**Why:**
- 21 tests pass, no_std compatible, clear API
- Fills a real gap: no other crate provides codec-agnostic compressed-domain scoring
- Low effort (1-2 hours)
- Creates a public verifiable artifact

**Positioning:** "Codec-agnostic compressed-domain scoring for retrieval and attention. no_std compatible for embedded."

## Third ROI: Wire PerDim into scr-runtime-compression CodecId

**Why:**
- `CodecId` enum currently has TurboQuant, FibQuant, Polar, Qjl, Uncompressed
- Adding `PerDim` completes the dispatch story
- Low effort (2-3 hours)
- Makes the scorer available to any runtime consumer

## What NOT to Pursue

- ❌ More GPU kernel optimization (GTX 1070 lacks Tensor Cores, cuBLAS wins)
- ❌ INT8 Tensor Core path (needs Ampere+ GPU Josh doesn't have)
- ❌ Sparse attention (different project, doesn't leverage what exists)
- ❌ Larger Python benchmark harness (already have enough evidence)

## Recommended Sequence

1. Publish compressed-scorer to crates.io (1-2 hr)
2. Add PerDim to CodecId in scr-runtime-compression (2-3 hr)
3. ESP32 compressed attention demo (1-2 days)
4. Write up results as a blog post / portfolio piece (1 hr)

Total: 2-3 days for maximum value extraction from what already exists.