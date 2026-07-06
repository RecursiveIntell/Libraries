# poly-kv DistilGPT2 isolated attention speed bench

## Bottom line

Stored result: scalar speed_ratio_exact_over_compressed=0.1619; vectorized speed_ratio=0.2028; optimized_prequantized speed_ratio=0.4251; decode_reduction_mean=20.5938x.

## Aggregate metrics

- case_count: 36
- exact_attention_ns_mean: 187056.1990740741
- compressed_attention_ns_mean: 1203444.6074074074
- vectorized_compressed_attention_ns_mean: 979468.1629629628
- optimized_prequantized_compressed_attention_ns_mean: 452459.6694444445
- speed_ratio_exact_over_compressed: 0.16189815750925945
- speed_ratio_exact_over_vectorized_compressed: 0.20277358927404848
- speed_ratio_exact_over_optimized_prequantized: 0.4251478309409978
- speed_ratio_min: 0.07487905267735294
- speed_ratio_max: 0.6163262380511544
- vectorized_speed_ratio_min: 0.10554654948320215
- vectorized_speed_ratio_max: 0.7041043152156892
- optimized_prequantized_speed_ratio_min: 0.2198254457767166
- optimized_prequantized_speed_ratio_max: 1.3016184955225014
- decode_reduction_mean: 20.59375
- decode_reduction_min: 20.59375
- vectorized_decode_reduction_mean: 20.59375
- vectorized_decode_reduction_min: 20.59375
- optimized_prequantized_decode_reduction_mean: 20.59375
- optimized_prequantized_decode_reduction_min: 20.59375
- attention_output_cosine_mean: 0.9239712209248176
- attention_output_cosine_min: 0.531327073113727
- vectorized_attention_output_cosine_mean: 0.9239712209248176
- vectorized_attention_output_cosine_min: 0.531327073113727
- optimized_attention_output_cosine_mean: 0.9239712209248176
- optimized_attention_output_cosine_min: 0.531327073113727
- attention_output_mse_mean: 0.005437909768127689
- vectorized_attention_output_mse_mean: 0.005437909768127689
- optimized_attention_output_mse_mean: 0.005437909768127689
- topk_overlap_mean: 0.9861111111111112
- vectorized_topk_overlap_mean: 0.9861111111111112
- optimized_topk_overlap_mean: 0.9861111111111112

## Claim boundary

isolated NumPy attention-operator benchmark over precomputed DistilGPT2 Q/K/V tensors; setup/full-forward cost excluded and quality diagnostics excluded from timed hot paths for optimized/prequantized timing; not production runtime speedup, not GPU kernel evidence, not end-to-end generation latency evidence
