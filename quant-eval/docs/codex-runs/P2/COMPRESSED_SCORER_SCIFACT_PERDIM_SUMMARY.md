# Compressed-scorer BEIR/Scifact per-dim receipt summary

Dataset: beir-scifact-test-v1
Embedding model: all-minilm:latest
Profile: per_dim_8bit
Scoring path: compressed_domain_score_then_exact_f32_rerank
Docs: 5183
Queries: 300

Metrics:
- raw recall@10: 0.7800
- codec recall@10: 0.7767
- raw NDCG@K: 0.6259
- codec NDCG@K: 0.6247
- top-K overlap: 0.9891
- exact-rerank recovery@1: 0.8767
- rank drift p95: 5097.0
- mean score error@K: 0.000622
- score error p95@K: 0.001694
- compression ratio: 3.9588x
- decoded docs during candidate scoring: 0
- exact rerank count: 12000
- passed: True
- blockers: []

Claim boundary: candidate-gate evidence only; compressed candidates are not authoritative results and must be exact-f32 reranked before semantic-memory/product use
