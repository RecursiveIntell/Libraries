# Phase 3 — Compression Survivability Lab

**Priority:** P0  
**Window:** 2-4 weeks  
**Owner:** turbo-quant, fib-quant, poly-kv, quant-governor, quant-eval  

---

## Objective

Build a benchmark harness that compares turbo-quant, fib-quant, and poly-kv against current public baselines for vector quantization, ANN retrieval, and KV-cache compression. The harness must emit receipt-bearing artifacts and separate vector codec correctness from retrieval quality from ANN performance from KV-cache compression.

---

## Required Benchmark Profiles

1. **codec_correctness** — encode/decode round-trip, reconstruction MSE, angular error, cosine distortion, inner-product distortion, deterministic reproducibility
2. **retrieval_quality** — NDCG@k, MAP@k, Recall@k, Precision@k, MRR using BEIR metrics
3. **ann_performance** — QPS, latency p50/p95/p99, index/build time, quantization time, peak RAM, resident RAM, disk footprint, warm/cold cache behavior
4. **local_recursiveintell** — code chunks from Coding/Libraries, docs/design notes/READMEs, claim/evidence packets, notebook/workbench artifacts
5. **kv_cache** — prefill vs decode separation, bytes/token, peak memory, tokens/sec, end-to-end latency, perplexity delta, long-context benchmark scores

---

## Baseline Matrix

### Vector and Retrieval Baseline

| Baseline family | Why it must be present | Primary track |
|---|---|---|
| Exact fp32 flat search | Ground-truth retrieval and distortion reference | Codec correctness, retrieval quality, ANN |
| Exact fp16 or bf16 flat search | Reduced-precision exact storage reference | Codec correctness, retrieval quality |
| Scalar quantization | Standard low-friction memory/latency baseline | Codec correctness, retrieval quality, ANN |
| Binary quantization | Extreme compression / Hamming baseline | Codec correctness, retrieval quality, ANN |
| Binary + float rescoring | Practical "fast first-stage + quality recovery" baseline | Retrieval quality, ANN |
| Product quantization | Canonical compressed-index baseline | Codec correctness, ANN |
| IVF + PQ / IVFPQ | Large-scale practical ANN baseline | ANN |
| RaBitQ / IVF_RABITQ | Modern randomized binary baseline with theoretical bound | Codec correctness, ANN |
| TurboQuant family | Modern high-compression, data-oblivious research baseline | Codec correctness, retrieval quality, ANN |
| turbo-quant | RecursiveIntell candidate under test | Codec correctness, retrieval quality, ANN |
| fib-quant | RecursiveIntell candidate under test | Codec correctness, retrieval quality, ANN |

### KV Baseline

| Baseline family | Why it must be present | Primary track |
|---|---|---|
| Full fp16 / bf16 KV cache | Quality and runtime reference | KV |
| Hugging Face QuantizedCache | Official library baseline for int2/int4/int8-style cache quantization | KV |
| vLLM FP8 KV cache | Official serving-system baseline with calibration options | KV |
| QJL | Zero-overhead 1-bit/JL residual-style baseline | KV |
| KIVI | Asymmetric 2-bit baseline with key/value-specific quantization | KV |
| KVQuant | Strong recent low-bit baseline with long-context/perplexity reporting | KV |
| PolarQuant | Key-cache-specific low-bit/search-table baseline | KV |
| TurboQuant | Paper baseline for near-optimal rate/distortion claims | KV |
| poly-kv | RecursiveIntell candidate if source proves KV-specific behavior | KV |

---

## Required Artifacts Per Run

| Artifact | Purpose |
|---|---|
| BenchmarkRunReceiptV1 | Run identity, command, git revision, seeds, timestamps, supersession pointer |
| DatasetManifestV1 | Dataset IDs, splits, hashes, embedding model IDs, preprocessing notes |
| CodecConfigV1 | Algorithm name, bit width, blocks, calibration mode, asymmetric/symmetric mode |
| BaselineConfigV1 | Exact baseline settings for comparison |
| HardwareProfileV1 | CPU, AVX flags, RAM, storage, GPU, backend, driver/runtime |
| MetricReportV1 | Raw and aggregated metrics |
| ResultComparisonV1 | Matched-budget and matched-quality deltas |
| FailureOrSkipRecordV1 | OOM, unsupported mode, calibration failure, missing backend |
| PublicClaimEligibilityRecordV1 | Whether any claim can be promoted beyond private/internal use |

---

## Metric Definitions

### Codec Correctness Metrics
- Encode time
- Decode time (if applicable)
- Memory footprint (bytes/vector and bits/dimension)
- Reconstruction MSE
- Angular error
- Cosine distortion
- Inner-product distortion
- Deterministic reproducibility under fixed seeds

### Retrieval Quality Metrics (BEIR)
- NDCG@k
- MAP@k
- Recall@k
- Precision@k
- MRR

### ANN Performance Metrics
- QPS (queries per second)
- Latency p50/p95/p99
- Index/training/build time
- Quantization time
- Peak RAM
- Resident RAM after load
- Disk footprint
- Warm-cache and cold-cache behavior
- Thread count

### KV Cache Metrics
- Prefill tokens/sec
- Decode tokens/sec
- Bytes/token or bytes per cached element
- Peak memory
- End-to-end latency
- Perplexity delta (Wikitext-2, C4)
- Long-context scores (LongBench, ZeroSCROLLS, RULER, needle-style)

---

## Claim-to-Metric Map

| Claim type | Required proof |
|---|---|
| "Uses less memory" | bytes/vector or bytes/token, peak RSS/VRAM, serialization size |
| "Faster retrieval" | QPS and latency at matched Recall@k or NDCG@k |
| "Better compression" | rate–distortion curve at matched task quality |
| "Better recall" | Recall@k or NDCG@k at matched storage budget and matched latency class |
| "Better ANN performance" | Frontier view across recall, QPS, build time, and index size |
| "Better KV quality" | Perplexity delta and/or long-context benchmark score at matched memory budget |
| "Better long-context efficiency" | Prefill/decode throughput plus LongBench / RULER / ZeroSCROLLS / needle-style scores |
| "Production-friendly" | Build/install reproducibility, hardware constraints, failure records, receipt completeness |

---

## Dataset Plan

### Deterministic CI Fixtures
- Seeded Gaussian vectors
- Clustered vectors
- Low-rank vectors
- Norm-skewed vectors
- Extreme outliers
- Duplicates
- Zero vectors
- Hand-constructed nearest-neighbor cases with known answers
- Short synthetic prompts (KV)
- Tiny deterministic long-context fixture (KV)

### Local RecursiveIntell Workloads
| Local corpus | Purpose |
|---|---|
| Code chunks from Coding/Libraries | Code retrieval and semantic search |
| Docs / design notes / READMEs | Long-document and project-history retrieval |
| Claim/evidence packets | Provenance-sensitive retrieval |
| Notebook / workbench artifacts | Gloss-like retrieval and mixed-structure search |

### Public Retrieval and ANN Comparability
- **BEIR** — heterogeneous zero-shot IR benchmark (primary)
- **MTEB/MMTEB retrieval subsets** — broader embedding context, multilingual, code retrieval
- **ANN-Benchmarks** — classic recall/QPS/index-size/build-time literature
- **VIBE** — modern embedding datasets, OOD settings, GPU awareness, quantized datasets, attention-style datasets (yi-128-ip, llama-128-ip)

### KV-Cache Comparability Datasets
- **Perplexity / LM stability:** Wikitext-2, C4
- **Long-context task benchmarks:** LongBench, ZeroSCROLLS, RULER, Needle-in-a-Haystack, L-Eval
- **Production-like RAG and summarization:** HotpotQA + Wikipedia RAG, QASPER-based summarization (SnapKV setup)

---

## Harness Architecture

### Rust Core
- Codec adapters
- Exact scoring kernels
- Microbench timing
- Deterministic fixture generation
- Receipt emission

### Python Orchestration
- Dataset acquisition and manifests
- BEIR / MTEB / MMTEB integration
- ANN-Benchmarks / VIBE-style experiment driving
- Hugging Face / vLLM KV runners
- Report assembly and supersession indexing

---

## Hardware and Environment Requirements

Benchmarks must record CPU/GPU environment tightly:
- Disable SMT/hyperthreading
- Disable E-cores on hybrid CPUs
- Set performance governor
- Check huge-page policy
- Record AVX flags (AVX-512 VPOPCNTDQ benefits RaBitQ)
- Record RAM, storage, GPU, backend, driver/runtime

For KV baselines, also log:
- Hugging Face cache backend
- vLLM kv_cache_dtype
- Calibration mode
- Attention backend
- Model dtype
- Whether scales were dataset-calibrated or estimated on-the-fly

---

## Implementation Order

1. Exact fp32/fp16 baselines
2. Scalar and binary baselines
3. Product quantization baseline
4. RecursiveIntell codecs under test (turbo-quant, fib-quant)
5. Retrieval quality on small deterministic and local corpora
6. ANN wrappers
7. KV-cache runners

---

## Acceptance Condition

**No public-safe performance claim is emitted unless the run has:**
- Complete receipt
- Matched baselines
- Recorded hardware
- Recorded datasets
- Validation passes

---

## Files to Create

```text
Libraries/quant-eval/
  Cargo.toml
  src/
    lib.rs
    codecs/
      exact.rs
      scalar.rs
      binary.rs
      product_quantization.rs
      turbo_quant_adapter.rs
      fib_quant_adapter.rs
    retrieval/
      beir_integration.rs
      metrics.rs
    ann/
      wrappers.rs
      frontier_plots.rs
    kv/
      huggingface_cache.rs
      vllm_cache.rs
      runners.rs
    receipts/
      benchmark_run_receipt.rs
      dataset_manifest.rs
      codec_config.rs
      baseline_config.rs
      hardware_profile.rs
      metric_report.rs
      result_comparison.rs
      failure_or_skip_record.rs
      public_claim_eligibility.rs
    fixtures/
      ci_deterministic.rs
      local_corpora.rs
  tests/
    codec_correctness.rs
    retrieval_quality.rs
    ann_performance.rs
    kv_cache.rs
  benches/
    codec_microbench.rs
    retrieval_bench.rs
    ann_bench.rs
    kv_bench.rs
  scripts/
    run_benchmark.py
    validate_receipts.py
    assemble_report.py
    compare_runs.py
  docs/
    BENCHMARK_SPEC.md
    DATASETS.md
    METRICS.md
    HARDWARE_HYGIENE.md
    PUBLIC_CLAIM_BOUNDARY.md
```

---

## Validation Commands

```bash
cargo metadata --format-version 1 > artifacts/cargo_metadata.json
cargo test --workspace --all-features
cargo bench --workspace

python -m pytest tests -q
python -m benchmark_harness.validate runs/latest/
python -m benchmark_harness.report runs/latest/
python -m benchmark_harness.compare --lhs runs/baseline --rhs runs/latest

python -m benchmark_harness.run --profile codec_correctness
python -m benchmark_harness.run --profile retrieval_quality
python -m benchmark_harness.run --profile ann_performance
python -m benchmark_harness.run --profile kv_cache
```

---

## Failure Metrics (First-Class Citizens)

The harness must record **failure** instead of silently dropping bad runs:
- Unsupported dimensions
- Calibration failures
- OOM
- Index-build explosions
- Non-determinism
- Accuracy cliffs on OOD data
- Short-context latency regressions
- Need for oversampling/rescoring/refinement to recover quality

---

## Public Claim Boundary

### Claims allowed after research only (now):
- RecursiveIntell is designing a benchmark harness that separates vector codec correctness, retrieval quality, ANN tradeoffs, and KV-cache compression
- Relevant public baselines in the current literature include scalar/binary/product quantization, RaBitQ, TurboQuant, QJL, KIVI, KVQuant, H2O, SnapKV, PyramidKV, RazorAttention, Hugging Face QuantizedCache, and vLLM FP8 KV cache
- Current public benchmark ecosystems worth mirroring include ANN-Benchmarks, big-ann-benchmarks, BEIR, MTEB/MMTEB, and VIBE

### Claims NOT safe until local harness exists and emits full receipts:
- Any memory-reduction percentage for turbo-quant, fib-quant, or poly-kv
- Any recall, NDCG, MRR, or perplexity delta for those crates
- Any "better than RaBitQ / PQ / KIVI / KVQuant / TurboQuant" statement
- Any throughput, latency, or build-time claim
- Any claim of negligible overhead, zero quality loss, or calibration-free operation for your own implementations
- Any claim about quant-codec-core or quant-governor as algorithms rather than support layers

Paper-reported values may be referenced only as **research-reported**, never as **reproduced locally**.

---

## Rollback Plan

- All benchmark changes are additive; prior runs remain accessible via supersession indexing
- If benchmark harness destabilizes workspace, move quant-eval to exclude list temporarily
- If codec adapters break existing turbo-quant/fib-quant/poly-kv tests, revert adapter changes and isolate behind feature flag
- No production path is enabled by benchmark work; benchmark is read-only observation layer
