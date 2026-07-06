# poly-kv compressed attention benchmark receipt

## Bottom line

Local synthetic benchmark of the new proveKV/poly-kv compressed pool+shell selection path.

Result: PASS under the declared local candidate gate.

## Config

- shared tokens: 512
- shell tokens: 64
- queries: 8
- top_k: 128
- head_dim: 64
- heads: 4
- layers: 2

## Metrics

- pool compression ratio: 21.27x
- pool build: 45.41 ms
- shell materialize: 14.55 ms
- legacy full-decode key-score mean: 27.34 ms/query
- compressed candidate mean: 18.33 ms/query
- speed ratio legacy/compressed: 1.492x
- top-k overlap mean vs legacy: 0.338
- value decode reduction vs full value decode: 4.50x
- decoded values total: 1024

## Claim boundary

local synthetic benchmark of compressed candidate selection over reconstructed KV artifacts; not model-quality, logit, PPL, or production latency evidence

This is not model-quality, logit, PPL, or production latency evidence.
