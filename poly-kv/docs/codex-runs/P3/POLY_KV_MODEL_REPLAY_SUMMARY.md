# poly-kv model-shaped replay receipt

## Bottom line

Deterministic model-shaped replay of proveKV/poly-kv compressed pool+shell selection against an exact full-decode attention reference.

Result: PASS under the declared local synthetic replay gate.

## Config

- shared tokens: 512
- shell tokens: 64
- queries: 8
- layer/head: 0/0
- candidate_k sweep: 32, 64, 128, 256
- selected candidate_k: 32
- vocab size: 128

## Selected metrics

- output cosine mean: 0.9981
- output MSE mean: 0.000754
- KL(exact || compressed) mean: 0.00109
- top-1 logit agreement: 1.0000
- PPL proxy exact: 1725.8193
- PPL proxy compressed: 1908.2444
- PPL proxy delta: 182.4251
- decoded values: 256
- full-decode value count: 4608
- decode reduction: 18.0x

## Candidate sweep

| candidate_k | cosine | MSE | KL | top1 | PPL delta | decode reduction | pass |
|---:|---:|---:|---:|---:|---:|---:|---|
| 32 | 0.9981 | 0.000754 | 0.00109 | 1.0000 | 182.4251 | 18.0x | yes |
| 64 | 0.9986 | 0.000450 | 0.00059 | 1.0000 | 19.8962 | 9.0x | yes |
| 128 | 0.9986 | 0.000578 | 0.00115 | 1.0000 | -223.3310 | 4.5x | yes |
| 256 | 0.9986 | 0.000896 | 0.00266 | 1.0000 | -346.0099 | 2.25x | no |

## Claim boundary

This is deterministic model-shaped replay over a synthetic projection. It is not real model PPL, not production KV-cache preservation, not production latency evidence, and not provider/framework KV-cache byte-reduction evidence.

Next proof gate: capture Q/K/V/logits from a small local model and replay those tensors through the same receipt shape.
