# poly-kv pretrained DistilGPT2 captured replay receipt

## Bottom line

This is the first real pretrained-model captured-tensor replay fixture for the proveKV/poly-kv compressed candidate path. It is a diagnostic/negative receipt, not a pass: compressed candidate selection over one captured DistilGPT2 layer/head does not preserve the captured logit proxy under the existing strict gate.

## Fixture

- model: distilgpt2-safetensors-manual-forward:2290a62682d06624634c1f46a6ad5be0f47f38aa:layer0:head0
- schema: poly_kv_captured_replay_fixture_v1
- head_dim: 64
- shared_tokens: 48
- queries: 4
- source_model: distilgpt2
- model_snapshot: 2290a62682d06624634c1f46a6ad5be0f47f38aa
- model_safetensors_sha256: sha256:e1ff18884359fe8beb795a5f414feb85a6ce3d929ad019c0d958c039d2b94a1b
- selected_vocab_size: 256
- runtime: numpy+safetensors+tokenizers manual DistilGPT2 forward; torch/transformers not used because torch wheel install hit ENOSPC
- projection_boundary: single captured attention-head contribution projected through that layer c_proj slice and tied token embeddings; not full downstream model replay

## Receipt

- schema: poly_kv_captured_model_replay_v1
- passed: false
- selected_candidate_k: 72
- output_cosine_mean: 0.6511622521629038
- output_mse_mean: 0.024125953283932373
- kl_divergence_mean: 6.243054382079233
- top1_agreement: 0.0
- ppl_proxy_exact: 1.01550793250192
- ppl_proxy_compressed: 574.3590544658935
- ppl_proxy_delta: 573.3435465333915
- decoded_values_total: 243
- full_decode_value_count: 243
- decode_reduction: 1.0

## Blockers

- kl_divergence_mean 6.2431 > 4.0000
- abs(ppl_proxy_delta) 573.3435 > 10.0000
- top1_agreement 0.0000 < 0.2500

## Interpretation

- This closes the environment/setup gap: we now have a reproducible pretrained DistilGPT2 safetensors capture path without torch/transformers.
- The result is not a KV-cache preservation win. It shows the current single-head projection proxy is too weak for logit/PPL claims.
- The next valid step is a full-forward replay gate that reinjects compressed attention outputs into the downstream model path, or a framework capture path with actual per-layer intervention.

## Claim boundary

Safe: pretrained DistilGPT2 Q/K/V/logit capture path and negative diagnostic replay receipt exist.
Not safe: real PPL preservation, production KV-cache preservation, production speedup, or replacement for KIVI/KVQuant/Quest.
