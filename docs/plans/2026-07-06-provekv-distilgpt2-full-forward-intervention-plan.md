# proveKV DistilGPT2 full-forward intervention replay plan

## Goal

Move beyond single-head logit projection receipts by measuring the downstream effect of compressed attention candidate selection after reinjecting the altered attention output into a manual pretrained DistilGPT2 forward pass.

## Why this is the next gate

The prior DistilGPT2 captured replay closed the setup gap — real pretrained Q/K/V/logits existed — but it failed the strict logit/PPL-proxy gate because it only projected one captured head directly to vocab logits. That was useful but still not a true downstream intervention.

The next honest gate is full-forward intervention:

1. Run exact DistilGPT2 forward in NumPy from pretrained safetensors.
2. At a selected layer/head/query position, replace exact head attention output with compressed-candidate sparse attention output.
3. Continue the remaining pretrained forward path exactly.
4. Compare final model logits/PPL proxy against the exact forward.

## Implementation approach

Use the existing dependency-light safetensors/tokenizers/numpy stack rather than torch/transformers. The torch wheel install previously failed with ENOSPC; we should not block on it.

Add:

- `poly-kv/tools/distilgpt2_full_forward_intervention.py`
- `poly-kv/docs/codex-runs/P3/POLY_KV_DISTILGPT2_FULL_FORWARD_INTERVENTION_RECEIPT.json`
- `poly-kv/docs/codex-runs/P3/POLY_KV_DISTILGPT2_FULL_FORWARD_INTERVENTION_SUMMARY.md`
- a Rust fixture/receipt smoke test that verifies the stored full-forward receipt exists, has the expected schema, includes candidate sweep metrics, and records the claim boundary.

## Metrics

Receipt schema: `poly_kv_distilgpt2_full_forward_intervention_v1`

For each candidate_k:

- attention_output_cosine_mean
- attention_output_mse_mean
- final_logit_kl_mean
- final_top1_agreement
- final_ppl_proxy_exact
- final_ppl_proxy_compressed
- final_ppl_proxy_delta
- decoded_values_total
- full_decode_value_count
- decode_reduction
- passed/blockers

## Thresholds

Start honest and receipt-declared:

- min_attention_output_cosine: 0.50
- max_attention_output_mse: 0.10
- max_final_logit_kl: 0.50
- max_abs_ppl_delta: 25.0
- min_final_top1_agreement: 0.50

If only full-context candidate_k passes, report that as a diagnostic neutral/negative: quality can recover, but decode reduction is not proven.

## Claim boundary

Safe:

`poly-kv has a pretrained DistilGPT2 full-forward intervention replay receipt that reinjects compressed-candidate attention outputs into the downstream manual forward path and measures final logit/PPL-proxy drift.`

Not safe:

- real corpus PPL preservation;
- production KV-cache preservation;
- production speedup;
- full model cache replacement;
- comparison to KIVI/KVQuant/Quest.

## Verification

- RED focused test fails before receipt exists.
- `python3 -m py_compile tools/distilgpt2_full_forward_intervention.py`
- run script and store JSON receipt + markdown summary.
- focused model replay tests.
- full poly-kv tests.
- clippy.
- package.
- `bash scripts/provekv_stack_smoke.sh`.
