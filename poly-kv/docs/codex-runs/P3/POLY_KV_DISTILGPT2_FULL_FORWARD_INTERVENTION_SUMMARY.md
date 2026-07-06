# poly-kv DistilGPT2 full-forward intervention receipt

## Bottom line

Stored result: pass at candidate_k=8.
This is stronger than the single-head projection receipt because the compressed attention output is reinjected and the remaining DistilGPT2 forward path is executed before comparing final logits.

## Metrics

- attention_output_cosine_mean: 0.8915619867184161
- attention_output_mse_mean: 0.006326654986232046
- final_logit_kl_mean: 0.0008319035898564356
- final_top1_agreement: 1.0
- final_ppl_proxy_exact: 1.0337989642419394
- final_ppl_proxy_compressed: 1.0275989123476559
- final_ppl_proxy_delta: -0.00620005189428352
- topk_overlap_mean: 1.0
- decoded_values_total: 548
- full_decode_value_count: 2628
- decode_reduction: 4.795620437956204

## Blockers

- none

## Claim boundary

pretrained DistilGPT2 full-forward intervention replay; compressed candidate attention is reinjected into downstream manual forward path; not real corpus PPL preservation, not production KV-cache preservation, not production latency evidence

Safe: full-forward intervention replay receipt exists for pretrained DistilGPT2.
Not safe: real corpus PPL preservation, production KV-cache preservation, production speedup, or replacement for KIVI/KVQuant/Quest.
