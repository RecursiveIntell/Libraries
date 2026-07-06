# poly-kv DistilGPT2 isolated attention speed bench

## Bottom line

Stored result: scalar=0.1502; vectorized=0.1977; optimized=0.4585; batch=0.5822; quest=0.1752; decode_reduction=20.5938x.

## Aggregate metrics

- case_count: 36
- exact_attention_ns_mean: 716210.8787037036
- compressed_attention_ns_mean: 4775119.641666668
- vectorized_compressed_attention_ns_mean: 3699345.6527777775
- optimized_prequantized_compressed_attention_ns_mean: 1725743.888888889
- batch_compressed_attention_ns_mean: 1336235.225925926
- quest_page_filtered_attention_ns_mean: 4753844.0287037045
- speed_ratio_exact_over_compressed: 0.1501829887754323
- speed_ratio_exact_over_vectorized_compressed: 0.1976652712871872
- speed_ratio_exact_over_optimized_prequantized: 0.45848479380741936
- speed_ratio_exact_over_batch: 0.5821640862823483
- speed_ratio_exact_over_quest: 0.17520193243142612
- speed_ratio_min: 0.07885137885907553
- speed_ratio_max: 0.3675971643134666
- vectorized_speed_ratio_min: 0.08569134760916308
- vectorized_speed_ratio_max: 0.48542448950293543
- optimized_prequantized_speed_ratio_min: 0.21732675620737252
- optimized_prequantized_speed_ratio_max: 1.3077971485569773
- batch_speed_ratio_min: 0.2306145941298066
- batch_speed_ratio_max: 1.3655808292100433
- quest_speed_ratio_min: 0.07465436870018219
- quest_speed_ratio_max: 0.584714313242622
- decode_reduction_mean: 20.59375
- decode_reduction_min: 20.59375
- vectorized_decode_reduction_mean: 20.59375
- vectorized_decode_reduction_min: 20.59375
- optimized_prequantized_decode_reduction_mean: 20.59375
- optimized_prequantized_decode_reduction_min: 20.59375
- batch_decode_reduction_mean: 20.59375
- batch_decode_reduction_min: 20.59375
- quest_decode_reduction_mean: 20.59375
- quest_decode_reduction_min: 20.59375
- attention_output_cosine_mean: 0.9239712209248176
- attention_output_cosine_min: 0.531327073113727
- vectorized_attention_output_cosine_mean: 0.9239712209248176
- vectorized_attention_output_cosine_min: 0.531327073113727
- optimized_attention_output_cosine_mean: 0.9239712209248176
- optimized_attention_output_cosine_min: 0.531327073113727
- batch_attention_output_cosine_mean: 0.9239712209248176
- batch_attention_output_cosine_min: 0.6694070475501903
- quest_attention_output_cosine_mean: 0.7687472116689563
- quest_attention_output_cosine_min: 0.26598106678525635
- attention_output_mse_mean: 0.005437909768127689
- vectorized_attention_output_mse_mean: 0.005437909768127689
- optimized_attention_output_mse_mean: 0.005437909768127689
- topk_overlap_mean: 0.9861111111111112
- vectorized_topk_overlap_mean: 0.9861111111111112
- optimized_topk_overlap_mean: 0.9861111111111112
- batch_topk_overlap_mean: 0.9861111111111112
- quest_topk_overlap_mean: 0.23543647710314375

## Claim boundary

isolated NumPy attention-operator benchmark over precomputed DistilGPT2 Q/K/V tensors; setup/full-forward cost excluded and quality diagnostics excluded from timed hot paths for optimized/batch/quest timing; includes Quest-style page min/max pre-filter (arXiv:2406.10774); not production runtime speedup, not GPU kernel evidence, not end-to-end generation latency evidence
