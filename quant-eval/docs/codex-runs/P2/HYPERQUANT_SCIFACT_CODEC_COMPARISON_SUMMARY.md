# HyperQuant Scifact codec comparison — all-minilm, test split

Commands:

```bash
python3 -u quant-eval/tools/hyperquant_scifact/build_scifact_ollama.py \
  --out quant-eval/target/hyperquant-scifact/scifact-all-minilm-corpus.json \
  --work-dir quant-eval/target/hyperquant-scifact \
  --model all-minilm:latest

HQ_TOP_K=10 HQ_CANDIDATE_K=40 HQ_SCALE=8.0 \
HQ_MIN_TOP_K_OVERLAP=0.30 HQ_MIN_EXACT_RERANK_RECOVERY_AT_1=0.80 \
cargo run -p quant-eval --example hyperquant_scifact_compare -- \
  quant-eval/target/hyperquant-scifact/scifact-all-minilm-corpus.json \
  quant-eval/docs/codex-runs/P2/HYPERQUANT_SCIFACT_CODEC_COMPARISON_RECEIPT.json
```

Dataset:

- BEIR Scifact test split (`beir-scifact-test-v1`)
- Documents: 5,183
- Test queries with positive qrels: 300
- Embedding model: local Ollama `all-minilm:latest`
- Text truncation: 700 chars, L2-normalized vectors

Receipt:

- `quant-eval/docs/codex-runs/P2/HYPERQUANT_SCIFACT_CODEC_COMPARISON_RECEIPT.json`

Result:

| Profile | Family | Passed | Compression | R@10 | Top-K overlap | Exact-rerank recovery@1 | Rank drift p95 | Score-error p95 |
|---|---|---:|---:|---:|---:|---:|---:|---:|
| scalar_i8_per_vector_scale | baseline | true | 3.9588x | 0.7800 | 0.9933 | 0.8767 | 1 | 0.000636 |
| scalar_i8_global_symmetric | baseline | true | 4.0000x | 0.7733 | 0.9595 | 0.8767 | 5 | 0.003955 |
| hyperquant_A2_scale_8 | hyperquant | true | 2.0000x | 0.7582 | 0.5910 | 0.8733 | 75 | 0.123640 |
| hyperquant_Z1_scale_8 | hyperquant | true | 2.0000x | 0.7549 | 0.5514 | 0.8667 | 78 | 0.148934 |
| sign_binary_1bit | baseline | true | 32.0000x | 0.7238 | 0.5652 | 0.8567 | 105 | 0.153689 |

Interpretation:

- Simple int8 baselines beat current HyperQuant Z1/A2 on Scifact/all-minilm embedding retrieval quality and compression ratio.
- HyperQuant A2/Z1 still pass the candidate-gate thresholds and beat the 1-bit sign negative/high-compression control on exact-rerank recovery, R@10, rank drift, and score error.
- Current HyperQuant is worth pursuing as a research/evidence-bearing lattice primitive, but not as the first production embedding codec over int8.
- For immediate product ROI in semantic-memory/RAG embedding retrieval, int8/per-vector scaling is the stronger baseline today.

Claim boundary:

- This is BEIR/Scifact all-minilm candidate-gate comparison evidence for exact f32 baseline, two simple int8 baselines, sign-binary control, and current HyperQuant Z1/A2 reconstruct-and-rank path.
- This is not ANN/FAISS/PQ superiority evidence, not optimized packed-index throughput evidence, not model-quality preservation, not KV-cache validation, and not production admissibility.
