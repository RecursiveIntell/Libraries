# CPU/CUDA Fib Gram Page Scorer Receipt — 2026-06-29

## Scope

Implemented the hardware-facing page scorer slice for compressed-cache attention/retrieval scoring.

This pass followed the project hard rule: checked existing libraries first and reused:

- `gpu-backend` CUDA/CPU dispatch structure
- `gpu-backend::simd_nearest` / fallback patterns
- existing CUDA kernel source layout under `gpu-backend/kernels/`
- `fib_quant::FibScorer`, `FibPreparedQuery`, `GramTable`, and `bitpack::unpack_indices`
- existing `poly-kv::KVecCodec::score_batch_compact` path

## What shipped

### gpu-backend

Added page-level FibQuant Gram scorer primitives:

- `FibGramPageScoreInput`
- `score_fib_gram_pages_cpu`
- `score_fib_gram_pages`
- `topk_indices_desc`
- CUDA dispatch hook: `cuda::fib_gram_page_score_gpu`
- CUDA source kernel: `kernels/fib_gram_page_score.cu`
- CPU/dispatch tests: `gpu-backend/tests/page_scorer.rs`
- Benchmark/receipt example: `gpu-backend/examples/fib_page_scorer_bench.rs`

The page scorer operates on compressed scoring ingredients:

- query codeword indices
- stored codeword indices
- stored norms
- Gram table
- query norm

It does not reconstruct stored f32 vectors.

### fib-quant

Added:

- `FibScorer::score_batch_prepared_pages`

This flattens compressed codeword indices and stored norms, then dispatches to `gpu-backend::score_fib_gram_pages`.

Added test:

- `fib-quant/tests/page_scorer.rs`

The test verifies page-scorer results match existing scalar prepared scoring exactly within `1e-6`.

### poly-kv

Updated `FibQuantAdapter::score_batch_compact` to call:

- `FibScorer::score_batch_prepared_pages`

So `AgentShell::attention_topk_compressed` now reaches the new CPU/CUDA page scorer backend for compact fib pool pages.

## Runtime CUDA status on this host

CUDA runtime execution was not claimed.

Observed blockers:

```bash
nvidia-smi --query-gpu=name,driver_version,memory.total --format=csv,noheader || true
# NVIDIA-SMI has failed because it couldn't communicate with the NVIDIA driver.

which nvcc || true
# no nvcc in PATH

find gpu-backend/kernels -name '*.ptx'
# no combined.ptx present
```

The CUDA feature path compiles, and the Rust dispatch hook exists. Runtime GPU execution requires a working NVIDIA driver and a compiled PTX module containing `fib_gram_page_score`.

## Verification

```bash
cargo test -p gpu-backend --test page_scorer
# passed: 2 tests

cargo check -p gpu-backend --features gpu,precompiled-ptx
# passed

cargo test -p gpu-backend --features gpu,precompiled-ptx --test page_scorer
# passed: 2 tests; CPU fallback on this host due unavailable CUDA runtime/PTX

cargo test -p fib-quant --test page_scorer
# passed: 1 test

cargo test --manifest-path /home/sikmindz/Coding/Libraries/poly-kv/Cargo.toml --test compressed_attention_path
# passed: 2 tests; includes decoded_keys == 0 assertion and top-k overlap check

cargo check -p fib-quant --features gpu
# passed

cargo check --manifest-path /home/sikmindz/Coding/Libraries/poly-kv/Cargo.toml --features fib-quant/gpu
# passed

cargo check -p semantic-memory --no-default-features --features 'brute-force turbo-quant-codec poly-kv-codec'
# passed

cargo run -p gpu-backend --release --example fib_page_scorer_bench
# {"schema_version":"fib_gram_page_scorer_bench_v1","backend":"cpu","gpu_available":false,"n_candidates":4096,"block_count":16,"n_codewords":32,"elapsed_us":95,"candidate_block_scores":65536,"candidate_block_scores_per_sec":682958346.7939433,"top8":[448,69,3604,886,3350,3565,3948,1868]}
```

## Claim boundary

Safe claim now:

- CPU page scorer is implemented, tested, and wired into `fib-quant` and `poly-kv`.
- CUDA dispatch surface and kernel source are implemented and compile behind the GPU feature.
- The compressed poly-kv path still reports `decoded_keys == 0` while using the page scorer backend.

Not safe yet:

- CUDA runtime speedup on GTX 1070. This host could not run CUDA due driver/PTX blockers.
- End-to-end model PPL preservation for the new page scorer beyond archived ProveKV receipt and current fixture overlap tests.

## Next hard step

Compile `gpu-backend/kernels/*.cu` into `gpu-backend/kernels/combined.ptx` on a CUDA-capable machine, then run:

```bash
cargo test -p gpu-backend --features gpu,precompiled-ptx --test page_scorer
cargo run -p gpu-backend --release --features gpu,precompiled-ptx --example fib_page_scorer_bench
```

Then compare CPU vs CUDA page scorer latency with resident-page batching. Per-call GPU copies will still limit gains unless follow-up work introduces a resident `GpuPipeline` buffer.
