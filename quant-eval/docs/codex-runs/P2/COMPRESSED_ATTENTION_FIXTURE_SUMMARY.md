# Compressed attention fixture receipt summary

Schema: compressed-attention-eval-v1
Scoring path: compressed_key_logits_topk_value_decode
Cache length: 4
Queries: 2
Dim: 4
Top-K decoded values per query: 2

Metrics:
- mean output cosine: 1.0000
- mean output MSE: 0.000005
- mean top-K overlap: 1.0000
- decompressed value count: 4
- passed: True
- blockers: []

Claim boundary: attention fixture evidence only; not model-quality, perplexity, latency, or production KV-cache preservation evidence
