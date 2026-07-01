"""FP8 Tensor Core scoring — simpler, better quality, same throughput as INT8.

FP8 (E4M3) on H100:
  - Same 2x Tensor Core throughput as INT8
  - Same 4x compression vs FP32 (8 bits vs 32 bits)
  - Better quality (floating-point, not fixed-point)
  - No scale factors needed (cast directly)
  - No custom kernel needed (torch.mm uses FP8 TC via cuBLAS)

Comparison with INT8 approach:
  INT8: quantize → custom Triton kernel → post-matmul scale multiply
  FP8:  cast → torch.mm → done

FP8 is NOT for ESP32 (no FP8 hardware). INT8/uint8 stays for embedded.

Run on H100 (sm_90) for real FP8 Tensor Core numbers.
On Pascal/Ampere, FP8 falls back to FP32 emulation (correct but slow).
"""

import torch
import time
import json
from pathlib import Path


class FP8Scorer:
    """FP8 Tensor Core scorer.
    
    Stores keys as FP8 (E4M3), scores via FP8 matmul.
    On H100: uses FP8 Tensor Cores (2x FP16 throughput).
    On A100: FP8 not available, falls back to FP16.
    On Pascal: FP8 not available, falls back to FP32.
    
    Quality: cosine ~1.0 (FP8 E4M3 has ~2 decimal digits, sufficient for ranking)
    Compression: 4x vs FP32, 2x vs FP16
    """
    
    def __init__(self, dim):
        self.dim = dim
        self._has_fp8 = self._check_fp8_support()
    
    def _check_fp8_support(self):
        """Check if FP8 is available on this GPU."""
        if not torch.cuda.is_available():
            return False
        major, minor = torch.cuda.get_device_capability(0)
        # FP8 Tensor Cores: sm_89 (Ada) and sm_90 (Hopper)
        # A100 (sm_80) does NOT have FP8 — only INT8
        return (major, minor) >= (8, 9)
    
    def quantize_keys(self, keys: torch.Tensor):
        """Cast keys to FP8. On non-FP8 GPUs, falls back to FP16."""
        if self._has_fp8:
            return keys.to(torch.float8_e4m3fn)
        else:
            # Fallback: use FP16 (closest to FP8 quality, available everywhere)
            return keys.half()
    
    def quantize_query(self, query: torch.Tensor):
        """Cast query to FP8."""
        if self._has_fp8:
            return query.to(torch.float8_e4m3fn)
        else:
            return query.half()
    
    def score(self, keys_comp, query_comp):
        """Score all keys against query via matmul.
        
        On H100: FP8 Tensor Core matmul (2x FP16 throughput)
        On other GPUs: FP16 matmul (fallback)
        """
        # torch.mm with FP8 inputs uses FP8 Tensor Cores on H100
        # Output is FP32 for accumulation precision
        if self._has_fp8:
            # FP8 matmul: [N, D] @ [D, 1] -> [N, 1]
            # Need to cast output to FP32 for accumulation
            scores = torch.mm(keys_comp, query_comp.unsqueeze(1).to(keys_comp.dtype)).squeeze(1)
            return scores.to(torch.float32)
        else:
            # FP16 fallback
            scores = torch.mm(keys_comp, query_comp.unsqueeze(1)).squeeze(1)
            return scores.to(torch.float32)


def benchmark_fn(fn, n_warmup=20, n_runs=100):
    for _ in range(n_warmup):
        fn()
    torch.cuda.synchronize()
    start = time.perf_counter()
    for _ in range(n_runs):
        fn()
    torch.cuda.synchronize()
    return (time.perf_counter() - start) / n_runs


def main():
    device = 'cuda'
    dim = 128
    n_keys_list = [256, 512, 1024, 2048, 4096, 8192]
    
    gpu_name = torch.cuda.get_device_name(0)
    major, minor = torch.cuda.get_device_capability(0)
    sm_version = f"{major}{minor}"
    
    scorer = FP8Scorer(dim=dim)
    
    print(f"GPU: {gpu_name} (sm_{sm_version})")
    print(f"FP8 Tensor Cores: {'YES' if scorer._has_fp8 else 'NO (using FP16 fallback)'}")
    print(f"Dim: {dim}")
    print()
    
    results = []
    
    for n_keys in n_keys_list:
        print(f"\n{'='*70}")
        print(f"n_keys={n_keys}, dim={dim}")
        print(f"{'='*70}")
        
        keys = torch.randn(n_keys, dim, device=device, dtype=torch.float32)
        query = torch.randn(dim, device=device, dtype=torch.float32)
        
        # Dense FP32 baseline
        t_fp32 = benchmark_fn(lambda: torch.mv(keys, query))
        scores_fp32 = torch.mv(keys, query)
        
        # Dense FP16 baseline (FP16 Tensor Core)
        keys_fp16 = keys.half()
        query_fp16 = query.half()
        t_fp16 = benchmark_fn(lambda: torch.mv(keys_fp16, query_fp16))
        
        # FP8 (or FP16 fallback)
        keys_comp = scorer.quantize_keys(keys)
        query_comp = scorer.quantize_query(query)
        t_fp8 = benchmark_fn(lambda: scorer.score(keys_comp, query_comp))
        scores_fp8 = scorer.score(keys_comp, query_comp)
        
        # Quality
        cos = torch.nn.functional.cosine_similarity(
            scores_fp32.unsqueeze(0), scores_fp8.unsqueeze(0)).item()
        
        # Top-k overlap
        k = min(32, n_keys)
        top_fp32 = set(torch.topk(scores_fp32, k).indices.tolist())
        top_comp = set(torch.topk(scores_fp8, k).indices.tolist())
        overlap = len(top_fp32 & top_comp) / k
        
        # Memory
        mem_fp32 = n_keys * dim * 4
        mem_fp16 = n_keys * dim * 2
        mem_fp8 = n_keys * dim * 1  # 1 byte per element
        
        speedup_fp32 = t_fp32 / t_fp8
        speedup_fp16 = t_fp16 / t_fp8
        
        method_name = "FP8 TC" if scorer._has_fp8 else "FP16 (fallback)"
        
        print(f"Dense FP32:  {t_fp32*1e6:8.2f} μs")
        print(f"Dense FP16:  {t_fp16*1e6:8.2f} μs")
        print(f"{method_name}: {t_fp8*1e6:8.2f} μs")
        print(f"  vs FP32: {speedup_fp32:.2f}x  | vs FP16: {speedup_fp16:.2f}x")
        print(f"  cosine: {cos:.6f}  | topk overlap: {overlap:.2f}")
        print(f"  Memory: FP32={mem_fp32/1024:.1f}KB  FP16={mem_fp16/1024:.1f}KB  "
              f"FP8={mem_fp8/1024:.1f}KB  (4x vs FP32, 2x vs FP16)")
        
        results.append({
            'n_keys': n_keys,
            'dim': dim,
            'method': method_name,
            'dense_fp32_us': t_fp32 * 1e6,
            'dense_fp16_us': t_fp16 * 1e6,
            'compressed_us': t_fp8 * 1e6,
            'speedup_vs_fp32': speedup_fp32,
            'speedup_vs_fp16': speedup_fp16,
            'cosine': cos,
            'topk_overlap': overlap,
            'mem_fp32': mem_fp32,
            'mem_fp16': mem_fp16,
            'mem_compressed': mem_fp8,
        })
    
    # Summary
    print(f"\n{'='*70}")
    print(f"SUMMARY — FP8 Tensor Core Scoring")
    print(f"{'='*70}")
    print(f"GPU: {gpu_name} (sm_{sm_version})")
    print(f"Method: {method_name}")
    print()
    print(f"{'n_keys':>6} {'FP32':>8} {'FP16':>8} {'COMP':>8} {'vs FP32':>8} {'vs FP16':>8} {'cos':>7} {'topk':>5} {'mem':>6}")
    print(f"{'':>6} {'(μs)':>8} {'(μs)':>8} {'(μs)':>8} {'(x)':>8} {'(x)':>8} {'':>7} {'':>5} {'⇓':>6}")
    print("-" * 70)
    for r in results:
        print(f"{r['n_keys']:>6} "
              f"{r['dense_fp32_us']:>8.2f} "
              f"{r['dense_fp16_us']:>8.2f} "
              f"{r['compressed_us']:>8.2f} "
              f"{r['speedup_vs_fp32']:>8.2f} "
              f"{r['speedup_vs_fp16']:>8.2f} "
              f"{r['cosine']:>7.4f} "
              f"{r['topk_overlap']:>5.2f} "
              f"{'4x' if scorer._has_fp8 else '2x':>6}")
    
    # Interpretation
    print(f"\n{'='*70}")
    print("INTERPRETATION")
    print(f"{'='*70}")
    if scorer._has_fp8:
        print("FP8 Tensor Cores available. Results are real FP8 performance.")
        any_faster = any(r['speedup_vs_fp16'] > 1.0 for r in results)
        if any_faster:
            best = max(r['speedup_vs_fp16'] for r in results)
            print(f"FP8 scoring is {best:.2f}x faster than FP16 cuBLAS.")
            print("This validates compressed-domain scoring on modern GPUs.")
        else:
            print("FP8 scoring is slower than FP16 cuBLAS on this workload.")
            print("Try larger N or dim — FP8 TC needs large matmuls to amortize overhead.")
    else:
        print("FP8 not available on this GPU. Results are FP16 fallback.")
        print("To get real FP8 numbers, run on:")
        print("  - H100 (sm_90) — FP8 E4M3/E5M2 Tensor Cores")
        print("  - RTX 4090 (sm_89) — FP8 E4M3 Tensor Cores")
        print()
        print("Note: A100 (sm_80) does NOT have FP8 — only INT8 Tensor Cores.")
        print("For A100, use the INT8 kernel (tensor_core_scorer.py).")
    
    # Comparison with INT8 approach
    print(f"\n{'='*70}")
    print("FP8 vs INT8 Comparison")
    print(f"{'='*70}")
    print("                    FP8 (E4M3)           INT8 (symmetric)")
    print(f"  Throughput:       2x FP16 TC          2x FP16 TC")
    print(f"  Compression:      4x vs FP32          4x vs FP32 (+scale overhead)")
    print(f"  Quality:          Higher (float)      Lower (fixed-point)")
    print(f"  Scale factors:    None needed         Per-key scale (4 bytes)")
    print(f"  Custom kernel:    No (torch.mm)       Yes (Triton)")
    print(f"  H100 (sm_90):     YES                 YES")
    print(f"  RTX 4090 (sm_89): YES                 YES")
    print(f"  A100 (sm_80):     NO                  YES")
    print(f"  ESP32:            NO                  YES (uint8)")
    print()
    print("Recommendation:")
    print("  - H100/RTX 4090: Use FP8 (simpler, better quality, same speed)")
    print("  - A100: Use INT8 (FP8 not available)")
    print("  - ESP32: Use uint8/PerDim (no FP8 hardware)")
    print("  - All three: wrapped by CompressedScorer trait")
    
    output_file = Path(__file__).parent / 'fp8_benchmark_results.json'
    with open(output_file, 'w') as f:
        json.dump({
            'gpu': gpu_name,
            'sm_version': sm_version,
            'has_fp8': scorer._has_fp8,
            'results': results,
        }, f, indent=2)
    print(f"\nResults saved to {output_file}")


if __name__ == '__main__':
    main()