# poly-kv captured-tensor model replay receipt

## Bottom line

Captured-tensor replay over a deterministic NumPy tiny-transformer fixture.

Result: PASS under the declared captured-fixture gate.

## Environment boundary

- `torch`: not installed
- `transformers`: not installed
- `numpy`: available and used to generate the fixture
- Ollama models are present, but the Ollama API does not expose Q/K/V tensors/logits for this gate

This fixture is a real tiny transformer forward pass implemented in NumPy, not a pretrained LLM capture.

## Fixture

- fixture: `POLY_KV_CAPTURED_TINY_TRANSFORMER_FIXTURE.json`
- model_id: `numpy-tiny-transformer-deterministic-v1`
- tokens: 72
- shared tokens: 56
- hot shell tokens: 16
- queries: 4
- head_dim: 16
- vocab: 64

## Replay result

- schema: `poly_kv_captured_model_replay_v1`
- selected candidate_k: 16
- output cosine mean: 0.5847
- output MSE mean: 0.02212
- KL(exact || compressed) mean: 0.00622
- top-1 logit agreement: 0.25
- PPL proxy exact: 58.5943
- PPL proxy compressed: 53.9198
- PPL proxy delta: -4.6746
- decoded values: 64
- full-decode value count: 288
- decode reduction: 4.5x

## Candidate sweep

| candidate_k | cosine | MSE | KL | top1 | PPL delta | decode reduction | pass |
|---:|---:|---:|---:|---:|---:|---:|---|
| 8 | 0.5660 | 0.033996 | 0.01041 | 0.00 | -5.2998 | 9.0x | no |
| 16 | 0.5847 | 0.022120 | 0.00622 | 0.25 | -4.6746 | 4.5x | yes |
| 32 | 0.6651 | 0.012458 | 0.00385 | 0.25 | -4.7427 | 2.25x | yes |
| 48 | 0.7215 | 0.007827 | 0.00247 | 0.25 | -3.4503 | 1.5x | yes |
| 64 | 0.7402 | 0.006405 | 0.00203 | 0.25 | -2.8664 | 1.125x | yes |
| 72 | 0.7503 | 0.006026 | 0.00188 | 0.25 | -2.6921 | 1.0x | yes |

## Claim boundary

Safe: captured-tensor replay against a deterministic tiny-transformer fixture with Q/K/V/logits recorded in JSON.

Not safe: pretrained LLM PPL preservation, production KV-cache preservation, production latency evidence, provider/framework KV-cache byte-reduction evidence, or superiority over KIVI/KVQuant/Quest.

Next gate: install/use a framework that exposes internals (`torch`/`transformers`, patched llama.cpp, or safetensors+candle), capture Q/K/V/logits from an actual pretrained small model, and feed that artifact into this same API.
