# poly-kv DistilGPT2 held-out full-forward suite

## Bottom line

Stored result: pass across 6 prompt/head cases.

## Aggregate metrics

- pass_rate: 1.0
- attention_output_cosine_mean: 0.9649223448353369
- attention_output_cosine_min: 0.9263973826498619
- attention_output_mse_mean: 0.0015536965586267074
- final_logit_kl_mean: 0.0009796520716441554
- final_logit_kl_max: 0.0039468171465648315
- final_top1_agreement_mean: 1.0
- final_top1_agreement_min: 1.0
- abs_ppl_delta_mean: 0.01395379533418828
- abs_ppl_delta_max: 0.047522017511103076
- decode_reduction_mean: 4.297520661157024
- decode_reduction_min: 4.297520661157025

## Claim boundary

held-out DistilGPT2 full-forward intervention suite over fixed prompts and selected heads; not real-corpus PPL preservation, not production KV-cache preservation, not production latency evidence
