#!/usr/bin/env python3
"""Scoring-only benchmark: per-dim vs per-key vs dense.

Measures just the score computation — not the full forward pass.
The full forward pass is dominated by Python loop overhead.
This isolates the actual scoring method difference.

For each method:
- Generate synthetic keys (matching SmolLM2-1.7B shape: 128-dim)
- Quantize them
- Time: prepare query + score N keys + top-k selection
- Report: keys/sec, memory per key, accuracy vs dense
"""
import argparse
import json
import time
from pathlib import Path
import torch
import torch.nn.functional as F


def benchmark_dense(q, keys, n_runs=20):
    """Dense attention: full fp16 dot product."""
    torch.cuda.synchronize()
    t0 = time.time()
    for _ in range(n_runs):
        scores = (q.unsqueeze(0) * keys).sum(dim=-1)
    torch.cuda.synchronize()
    elapsed = (time.time() - t0) / n_runs
    return scores, elapsed


def benchmark_per_key_quantized(q, keys, bits, n_runs=20):
    """Per-key quantized scoring: one scale per key."""
    levels = (1 << (bits - 1)) - 1
    # Quantize keys (pre-computed, stored)
    max_abs = keys.abs().amax(dim=-1, keepdim=True).clamp_min(1e-6)
    qk = torch.round((keys / max_abs).clamp(-1, 1) * levels).to(torch.int8)
    scales = max_abs.squeeze(-1).float()
    
    torch.cuda.synchronize()
    t0 = time.time()
    for _ in range(n_runs):
        scores = ((qk.float() / levels) * scales[:, None] * q.unsqueeze(0)).sum(dim=-1)
    torch.cuda.synchronize()
    elapsed = (time.time() - t0) / n_runs
    return scores, elapsed, qk, scales


def benchmark_per_dim_quantized(q, keys, bits, n_runs=20):
    """Per-dim quantized scoring: one min/max per dimension."""
    per_dim_levels = (1 << bits) - 1
    # Quantize keys (pre-computed, stored)
    k_norms = keys.norm(dim=-1).clamp_min(1e-6)
    k_unit = keys / k_norms.unsqueeze(-1)
    dim_mins = k_unit.min(dim=0).values
    dim_maxs = k_unit.max(dim=0).values
    dim_ranges = (dim_maxs - dim_mins).clamp_min(1e-6)
    qk = torch.round(
        ((k_unit - dim_mins.unsqueeze(0)) / dim_ranges.unsqueeze(0)) * per_dim_levels
    ).clamp(0, per_dim_levels).to(torch.uint8)
    
    torch.cuda.synchronize()
    t0 = time.time()
    for _ in range(n_runs):
        key_recon_unit = dim_mins.unsqueeze(0) + (qk.float() / per_dim_levels) * dim_ranges.unsqueeze(0)
        scores = (key_recon_unit * q.unsqueeze(0)).sum(dim=-1) * k_norms
    torch.cuda.synchronize()
    elapsed = (time.time() - t0) / n_runs
    return scores, elapsed, qk, k_norms


def memory_per_key(n_keys, dim, method, bits):
    """Calculate memory per key in bytes."""
    if method == "dense":
        return dim * 2  # fp16
    elif method == "per-key":
        return (bits / 8) + 4  # int8 codes + fp32 scale
    elif method == "per-dim":
        # codes per key + shared dim stats amortized over keys
        codes = bits / 8  # uint8 codes per key
        norm = 4  # fp32 norm per key
        shared = (dim * 4 * 2) / n_keys  # dim_mins + dim_ranges amortized
        return codes + norm + shared


def rank_accuracy(dense_scores, approx_scores, top_k):
    """Measure how well approx scores match dense ranking."""
    dense_topk = torch.topk(dense_scores, k=min(top_k, len(dense_scores))).indices
    approx_topk = torch.topk(approx_scores, k=min(top_k, len(approx_scores))).indices
    overlap = len(set(dense_topk.tolist()) & set(approx_topk.tolist()))
    return overlap / top_k


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--dim", type=int, default=128, help="Head dimension (SmolLM2=128)")
    ap.add_argument("--n-keys", type=int, nargs="+", default=[64, 128, 256, 512],
                    help="Number of keys to benchmark")
    ap.add_argument("--n-runs", type=int, default=20)
    ap.add_argument("--device", default="cuda")
    ap.add_argument("--output", type=Path, required=True)
    args = ap.parse_args()

    device = args.device if args.device == "cpu" or torch.cuda.is_available() else "cpu"
    print(f"Device: {device}, dim: {args.dim}")

    results = []

    for n_keys in args.n_keys:
        print(f"\n{'='*60}")
        print(f"n_keys={n_keys}, dim={args.dim}")
        print(f"{'='*60}")

        # Generate synthetic keys matching typical transformer distribution
        torch.manual_seed(42)
        keys = torch.randn(n_keys, args.dim, device=device, dtype=torch.float16) * 0.1
        q = torch.randn(args.dim, device=device, dtype=torch.float16) * 0.1

        # Dense baseline
        dense_scores, dense_time = benchmark_dense(q.float(), keys.float(), args.n_runs)
        dense_kps = n_keys / dense_time
        dense_mem = memory_per_key(n_keys, args.dim, "dense", 16)

        print(f"  Dense:       {dense_time*1000:.3f}ms  {dense_kps/1e6:.2f}M keys/s  {dense_mem:.1f} B/key")

        configs = [
            ("per-key-4", "per-key", 4),
            ("per-key-8", "per-key", 8),
            ("per-dim-4", "per-dim", 4),
            ("per-dim-8", "per-dim", 8),
        ]

        for name, method, bits in configs:
            if method == "per-key":
                approx_scores, elapsed, qk, scales = benchmark_per_key_quantized(
                    q.float(), keys.float(), bits, args.n_runs)
                mem = memory_per_key(n_keys, args.dim, method, bits)
            else:
                approx_scores, elapsed, qk, k_norms = benchmark_per_dim_quantized(
                    q.float(), keys.float(), bits, args.n_runs)
                mem = memory_per_key(n_keys, args.dim, method, bits)

            kps = n_keys / elapsed
            speedup = dense_time / elapsed
            overlap = rank_accuracy(dense_scores, approx_scores, top_k=min(32, n_keys))
            cos = F.cosine_similarity(
                dense_scores.unsqueeze(0), approx_scores.unsqueeze(0), dim=-1).item()

            print(f"  {name:<14} {elapsed*1000:.3f}ms  {kps/1e6:.2f}M keys/s  "
                  f"speedup={speedup:.2f}x  mem={mem:.1f}B/key  "
                  f"cos={cos:.4f}  topk_overlap={overlap:.2f}")

            results.append({
                "n_keys": n_keys,
                "method": name,
                "latency_ms": elapsed * 1000,
                "keys_per_sec": kps,
                "speedup_vs_dense": speedup,
                "bytes_per_key": mem,
                "rank_cosine": cos,
                "topk_overlap": overlap,
            })

        results.append({
            "n_keys": n_keys,
            "method": "dense",
            "latency_ms": dense_time * 1000,
            "keys_per_sec": dense_kps,
            "speedup_vs_dense": 1.0,
            "bytes_per_key": dense_mem,
            "rank_cosine": 1.0,
            "topk_overlap": 1.0,
        })

    output = {
        "device": device,
        "dim": args.dim,
        "n_runs": args.n_runs,
        "results": results,
    }
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(output, indent=2))

    print(f"\n{'='*60}")
    print("SUMMARY")
    print(f"{'='*60}")
    print(f"{'Config':<16} {'n_keys':<8} {'Latency':<10} {'Keys/s':<12} {'Speedup':<10} {'Mem/key':<10} {'Cos':<8} {'TopK':<8}")
    print("-"*80)
    for r in sorted(results, key=lambda x: (x["n_keys"], x["method"])):
        print(f"{r['method']:<16} {r['n_keys']:<8} {r['latency_ms']:<10.3f} "
              f"{r['keys_per_sec']/1e6:<12.2f} {r['speedup_vs_dense']:<10.2f}x "
              f"{r['bytes_per_key']:<10.1f} {r['rank_cosine']:<8.4f} {r['topk_overlap']:<8.2f}")


if __name__ == "__main__":
    main()
