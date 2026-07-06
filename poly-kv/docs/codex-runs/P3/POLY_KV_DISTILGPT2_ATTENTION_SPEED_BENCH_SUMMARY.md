# poly-kv DistilGPT2 isolated attention speed bench

## Bottom line

Stored result: scalar speed_ratio_exact_over_compressed=0.0134; vectorized speed_ratio=0.2322; decode_reduction_mean=20.5938x.

## Aggregate metrics

- case_count: 36
- exact_attention_ns_mean: 294800.6861111111
- compressed_attention_ns_mean: 25946750.887962967
- vectorized_compressed_attention_ns_mean: 1465669.808333333
- speed_ratio_exact_over_compressed: 0.013394096506046116
- speed_ratio_exact_over_vectorized_compressed: 0.23218512983095352
- speed_ratio_min: 0.0012756393134813486
- speed_ratio_max: 0.03696895255389062
- vectorized_speed_ratio_min: 0.027595925591126145
- vectorized_speed_ratio_max: 0.7734462261048245
- decode_reduction_mean: 20.59375
- decode_reduction_min: 20.59375
- vectorized_decode_reduction_mean: 20.59375
- vectorized_decode_reduction_min: 20.59375
- attention_output_cosine_mean: 0.9239712209248176
- attention_output_cosine_min: 0.531327073113727
- vectorized_attention_output_cosine_mean: 0.9239712209248176
- vectorized_attention_output_cosine_min: 0.531327073113727
- attention_output_mse_mean: 0.005437909768127689
- vectorized_attention_output_mse_mean: 0.005437909768127689
- topk_overlap_mean: 0.9861111111111112
- vectorized_topk_overlap_mean: 0.9861111111111112

## Claim boundary

isolated NumPy attention-operator benchmark over precomputed DistilGPT2 Q/K/V tensors; setup/full-forward cost excluded; not production runtime speedup, not GPU kernel evidence, not end-to-end generation latency evidence
