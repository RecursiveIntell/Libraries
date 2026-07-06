# poly-kv DistilGPT2 held-out full-forward suite

## Bottom line

Stored result: pass across 18 prompt/head cases.

## Aggregate metrics

- pass_rate: 1.0
- attention_output_cosine_mean: 0.9707235371900992
- attention_output_cosine_min: 0.9263973826498619
- attention_output_mse_mean: 0.009537016191625624
- final_logit_kl_mean: 0.0006609734727441052
- final_logit_kl_max: 0.0039468171465648315
- final_top1_agreement_mean: 1.0
- final_top1_agreement_min: 1.0
- abs_ppl_delta_mean: 0.011403230926835237
- abs_ppl_delta_max: 0.047522017511103076
- decode_reduction_mean: 4.297520661157025
- decode_reduction_min: 4.297520661157025

## Claim boundary

layer-sweep DistilGPT2 full-forward intervention suite over fixed prompts, layers [0, 1, 2, 3, 4, 5], and heads [0]; not real-corpus PPL preservation, not production KV-cache preservation, not production latency evidence
