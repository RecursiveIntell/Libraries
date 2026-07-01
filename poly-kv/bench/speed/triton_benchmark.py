"""Benchmark Triton fused kernels vs Python vs dense matmul."""

import torch
import time
import json
from pathlib import Path
from triton_scorer import PerDimScorerFused, PerKeyScorerFused


def benchmark_fn(fn, n_warmup=10, n_runs=100):
    """Benchmark a function with warmup."""
    # Warmup
    for _ in range(n_warmup):
        fn()
    torch.cuda.synchronize()
    
    # Timed runs
    start = time.perf_counter()
    for _ in range(n_runs):
        fn()
    torch.cuda.synchronize()
    elapsed = time.perf_counter() - start
    
    return elapsed / n_runs


def main():
    device = 'cuda'
    dim = 128
    n_keys_list = [64, 128, 256, 512, 1024, 2048]
    
    results = []
    
    for n_keys in n_keys_list:
        print(f"\n{'='*60}")
        print(f"n_keys={n_keys}, dim={dim}")
        print(f"{'='*60}")
        
        # Generate random keys and query
        keys = torch.randn(n_keys, dim, device=device, dtype=torch.float32)
        query = torch.randn(dim, device=device, dtype=torch.float32)
        
        # === Dense baseline (cuBLAS) ===
        t_dense = benchmark_fn(lambda: torch.mv(keys, query))
        scores_dense = torch.mv(keys, query)
        print(f"Dense (cuBLAS):      {t_dense*1e6:7.2f} μs  ({1/t_dense:10.0f} keys/s)")
        
        # === Per-dim 8-bit fused (Triton) ===
        scorer_pd = PerDimScorerFused(dim=dim, bits=8)
        codes, k_norms, dim_mins, dim_ranges = scorer_pd.quantize_keys(keys)
        scaled_query, bias = scorer_pd.prepare_query(query, dim_mins, dim_ranges)
        
        t_pd_fused = benchmark_fn(lambda: scorer_pd.score(codes, k_norms, scaled_query, bias))
        scores_pd_fused = scorer_pd.score(codes, k_norms, scaled_query, bias)
        cos_pd = torch.nn.functional.cosine_similarity(
            scores_dense.unsqueeze(0), scores_pd_fused.unsqueeze(0)).item()
        print(f"Per-dim-8 fused:     {t_pd_fused*1e6:7.2f} μs  ({1/t_pd_fused:10.0f} keys/s)  cos={cos_pd:.4f}")
        
        # === Per-dim 8-bit Python ===
        def score_pd_python():
            key_recon_unit = dim_mins.unsqueeze(0) + (codes.float() / scorer_pd.levels) * dim_ranges.unsqueeze(0)
            return (key_recon_unit * query.unsqueeze(0)).sum(dim=-1) * k_norms
        
        t_pd_py = benchmark_fn(score_pd_python)
        print(f"Per-dim-8 Python:    {t_pd_py*1e6:7.2f} μs  ({1/t_pd_py:10.0f} keys/s)")
        
        # === Per-key 8-bit fused (Triton) ===
        scorer_pk = PerKeyScorerFused(dim=dim, bits=8)
        qk, scales = scorer_pk.quantize_keys(keys)
        
        t_pk_fused = benchmark_fn(lambda: scorer_pk.score(qk, scales, query))
        scores_pk_fused = scorer_pk.score(qk, scales, query)
        cos_pk = torch.nn.functional.cosine_similarity(
            scores_dense.unsqueeze(0), scores_pk_fused.unsqueeze(0)).item()
        print(f"Per-key-8 fused:     {t_pk_fused*1e6:7.2f} μs  ({1/t_pk_fused:10.0f} keys/s)  cos={cos_pk:.4f}")
        
        # === Per-key 8-bit Python ===
        def score_pk_python():
            levels = (1 << (scorer_pk.bits - 1)) - 1
            return ((qk.float() / levels) * scales.unsqueeze(-1) * query.unsqueeze(0)).sum(dim=-1)
        
        t_pk_py = benchmark_fn(score_pk_python)
        print(f"Per-key-8 Python:    {t_pk_py*1e6:7.2f} μs  ({1/t_pk_py:10.0f} keys/s)")
        
        # Record results
        results.append({
            'n_keys': n_keys,
            'dim': dim,
            'dense': {'time_us': t_dense * 1e6, 'keys_per_sec': 1/t_dense},
            'per_dim_8_fused': {
                'time_us': t_pd_fused * 1e6,
                'keys_per_sec': 1/t_pd_fused,
                'speedup_vs_dense': t_dense / t_pd_fused,
                'cosine': cos_pd,
            },
            'per_dim_8_python': {
                'time_us': t_pd_py * 1e6,
                'keys_per_sec': 1/t_pd_py,
                'speedup_vs_dense': t_dense / t_pd_py,
            },
            'per_key_8_fused': {
                'time_us': t_pk_fused * 1e6,
                'keys_per_sec': 1/t_pk_fused,
                'speedup_vs_dense': t_dense / t_pk_fused,
                'cosine': cos_pk,
            },
            'per_key_8_python': {
                'time_us': t_pk_py * 1e6,
                'keys_per_sec': 1/t_pk_py,
                'speedup_vs_dense': t_dense / t_pk_py,
            },
        })
    
    # Save results
    output_file = Path(__file__).parent / 'triton_benchmark_results.json'
    with open(output_file, 'w') as f:
        json.dump(results, f, indent=2)
    
    print(f"\n{'='*60}")
    print("SUMMARY")
    print(f"{'='*60}")
    print(f"{'n_keys':>6}  {'Dense':>10}  {'Fused PD':>10}  {'Py PD':>10}  {'Fused PK':>10}  {'Py PK':>10}")
    print(f"{'':>6}  {'(μs)':>10}  {'(μs)':>10}  {'(μs)':>10}  {'(μs)':>10}  {'(μs)':>10}")
    print(f"{'-'*6}  {'-'*10}  {'-'*10}  {'-'*10}  {'-'*10}  {'-'*10}")
    for r in results:
        print(f"{r['n_keys']:>6}  "
              f"{r['dense']['time_us']:>10.2f}  "
              f"{r['per_dim_8_fused']['time_us']:>10.2f}  "
              f"{r['per_dim_8_python']['time_us']:>10.2f}  "
              f"{r['per_key_8_fused']['time_us']:>10.2f}  "
              f"{r['per_key_8_python']['time_us']:>10.2f}")
    
    print(f"\nResults saved to {output_file}")


if __name__ == '__main__':
    main()
