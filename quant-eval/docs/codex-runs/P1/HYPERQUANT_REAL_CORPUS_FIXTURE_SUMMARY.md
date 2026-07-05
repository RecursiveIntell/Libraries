# HyperQuant real-corpus fixture receipt

Command:

```bash
cargo run -p quant-eval --example hyperquant_real_corpus_receipt > quant-eval/docs/codex-runs/P1/HYPERQUANT_REAL_CORPUS_FIXTURE_RECEIPT.json
```

Receipt:

- `quant-eval/docs/codex-runs/P1/HYPERQUANT_REAL_CORPUS_FIXTURE_RECEIPT.json`

Result:

| Profile | Passed | Recall@K | NDCG@K | Top-K overlap | Exact-rerank recovery@1 | Rank drift max | Compression ratio |
|---|---:|---:|---:|---:|---:|---:|---:|
| Z1 | true | 1.0 | 1.0 | 1.0 | 1.0 | 0 | 2.0 |
| A2 | true | 1.0 | 1.0 | 1.0 | 1.0 | 0 | 2.0 |

Claim boundary:

This is a tiny hand-authored real-corpus/qrels fixture that proves the reusable HyperQuant retrieval gate, receipt schema, blockers, rank-drift metrics, score-error metrics, timing fields, and byte accounting. It is not BEIR/Scifact quality evidence, model-quality preservation evidence, or production admissibility.
