# Changelog

All notable changes to `gpu-backend` are documented here.

## [Unreleased]

## [0.1.0-alpha.1] — 2026-06-02

First crates.io release.

### Added

- `simd_nearest_codeword` — AVX2+FMA SIMD implementation
  of the k=4, N=32 codebook lookup loop. **6.7× speedup**
  over scalar f32 (8ms → 1.2ms on 16 random seeds).
  Runtime feature detection via `is_x86_feature_detected!`.
  Parity-verified byte-identical to scalar f32 on all 16 seeds.
- Hadamard rotation kernel (CPU fallback + CUDA dispatch).
- Codebook lookup kernel (CPU fallback + CUDA dispatch).
  Parity-verified byte-identical to CPU reference on
  random inputs.
- `cudarc` CUDA driver bindings (optional, behind `gpu` feature).
- `blake3` digest for parity-test results.

### Benchmarks

- SIMD nearest-codeword: 6.7× over scalar f32.
- GPU Hadamard-only: 2.5-2.7% win on the larger corpora.
- GPU codebook lookup: parity-verified, but slower than CPU
  in integration due to per-call H2D/D2H overhead.

### Test coverage

- 16 parity tests in `tests/`:
  - SIMD vs scalar f32, 16 random seeds, byte-identical.
  - Hadamard vs reference, 8 random seeds, byte-identical.
  - Codebook lookup vs reference, 8 random seeds, byte-identical.

[Unreleased]: https://github.com/RecursiveIntell/Libraries/compare/gpu-backend-v0.1.0-alpha.1...HEAD
[0.1.0-alpha.1]: https://github.com/RecursiveIntell/Libraries/releases/tag/gpu-backend-v0.1.0-alpha.1
