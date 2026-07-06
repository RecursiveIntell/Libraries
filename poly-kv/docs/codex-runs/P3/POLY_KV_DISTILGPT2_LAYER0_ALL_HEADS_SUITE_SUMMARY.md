# poly-kv DistilGPT2 held-out full-forward suite

## Bottom line

Stored result: pass across 36 prompt/head cases.

## Aggregate metrics

- pass_rate: 1.0
- attention_output_cosine_mean: 0.9167993986625428
- attention_output_cosine_min: 0.5972163668204995
- attention_output_mse_mean: 0.004706318137882546
- final_logit_kl_mean: 0.021511326225276864
- final_logit_kl_max: 0.3367433592765583
- final_top1_agreement_mean: 0.9791666666666666
- final_top1_agreement_min: 0.5
- abs_ppl_delta_mean: 0.09589616152110778
- abs_ppl_delta_max: 1.699312903389837
- decode_reduction_mean: 4.297520661157025
- decode_reduction_min: 4.297520661157025

## Claim boundary

all-head DistilGPT2 full-forward intervention suite over fixed prompts, layers [0], and heads [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]; not real-corpus PPL preservation, not production KV-cache preservation, not production latency evidence
