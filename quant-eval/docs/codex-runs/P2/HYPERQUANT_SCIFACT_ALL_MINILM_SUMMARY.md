# HyperQuant BEIR/Scifact receipt — all-minilm, test split

Commands:

```bash
python3 -u quant-eval/tools/hyperquant_scifact/build_scifact_ollama.py \
  --out quant-eval/target/hyperquant-scifact/scifact-all-minilm-corpus.json \
  --work-dir quant-eval/target/hyperquant-scifact \
  --model all-minilm:latest

HQ_TOP_K=10 HQ_CANDIDATE_K=40 HQ_SCALE=8.0 \
HQ_MIN_TOP_K_OVERLAP=0.30 HQ_MIN_EXACT_RERANK_RECOVERY_AT_1=0.80 \
cargo run -p quant-eval --example hyperquant_scifact_eval -- \
  quant-eval/target/hyperquant-scifact/scifact-all-minilm-corpus.json \
  quant-eval/docs/codex-runs/P2/HYPERQUANT_SCIFACT_ALL_MINILM_RECEIPT.json
```

Dataset:

- BEIR Scifact test split (`beir-scifact-test-v1`)
- Source: `https://public.ukp.informatik.tu-darmstadt.de/thakur/BEIR/datasets/scifact.zip`
- Qrels digest: `sha256:0864bb985e0ca...` (full digest in JSON receipt)
- Documents: 5,183
- Test queries with positive qrels: 300
- Embedding model: local Ollama `all-minilm:latest`
- Text truncation: 700 chars, L2-normalized vectors

Receipt:

- `quant-eval/docs/codex-runs/P2/HYPERQUANT_SCIFACT_ALL_MINILM_RECEIPT.json`

Result:

| Profile | Passed | Raw R@1 | Raw R@5 | Raw R@10 | Codec R@1 | Codec R@5 | Codec R@10 | Top-K overlap | Exact-rerank recovery@1 | Rank drift p95 | Compression ratio |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Z1 | true | 0.4579 | 0.6922 | 0.7800 | 0.4412 | 0.6657 | 0.7549 | 0.5514 | 0.8667 | 78 | 2.0 |
| A2 | true | 0.4579 | 0.6922 | 0.7800 | 0.4440 | 0.6778 | 0.7582 | 0.5910 | 0.8733 | 75 | 2.0 |

Interpretation:

- Both implemented HyperQuant profiles pass the declared candidate-gate thresholds: top-K overlap >= 0.30 and exact-rerank recovery@1 >= 0.80.
- A2 is slightly better than Z1 on this receipt for codec R@5/R@10, top-K overlap, exact-rerank recovery, rank drift p95, and score-error p95.
- This is evidence that the current HyperQuant Z1/A2 primitive can preserve enough Scifact/all-minilm candidate set quality for exact rerank to recover the top relevant document in most queries.

Claim boundary:

- This is BEIR/Scifact retrieval-gate evidence for `quant-eval` + current `hyperquant` Z1/A2 using local Ollama `all-minilm:latest` embeddings.
- It is not evidence of model perplexity preservation, transformer KV-cache quality, production admissibility, or superiority over other codecs.
- It does not validate D4/E8; current public `hyperquant` still exposes D4/E8 as unsupported roadmap targets.
