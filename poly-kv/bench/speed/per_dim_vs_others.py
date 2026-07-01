#!/usr/bin/env python3
"""Speed and memory benchmark for per-dim scorer vs other scorers.

Measures:
- Latency (seconds per forward pass)
- Tokens/sec
- Peak GPU memory (MB)
- Quality metrics (logit cosine p05, KL p95, PPL delta)

Compares:
- Dense attention (baseline)
- Simple quantized (per-key max)
- Per-dim (4-bit, 8-bit)
- Fib-quant (if available)
- Turbo-quant (if available)

Usage:
  python per_dim_vs_others.py --model HuggingFaceTB/SmolLM2-1.7B-Instruct --n-tokens 256 --device cuda
"""
import argparse
import json
import sys
import time
from pathlib import Path
import torch
import torch.nn.functional as F

# Add parent scripts dir to path
sys.path.insert(0, str(Path(__file__).parent.parent.parent / "scripts"))
from compressed_attention_forward_ppl import (
    causal_nll_from_logits,
    logit_metrics,
    patch_model,
    percentile,
    windowed_ppl,
)
from transformers import AutoModelForCausalLM, AutoTokenizer
from datasets import load_dataset


def measure_memory_mb():
    """Return peak GPU memory in MB (or 0 if CPU)."""
    if torch.cuda.is_available():
        return torch.cuda.max_memory_allocated() / (1024 * 1024)
    return 0.0


def reset_memory():
    """Reset GPU memory tracking."""
    if torch.cuda.is_available():
        torch.cuda.reset_peak_memory_stats()
        torch.cuda.empty_cache()


def run_scorer(model, ids, scorer_name, quant_bits, top_k, recent_guard, adaptive_budget,
               budget_target_cosine, budget_min_k, budget_max_k, budget_ref_k,
               n_runs=3, collect_stats=True):
    """Run a scorer configuration and return metrics + logits."""
    # Reset memory tracking
    reset_memory()
    peak_mem = 0.0
    
    # Configure scorer
    ab = None
    if adaptive_budget:
        sys.path.insert(0, str(Path(__file__).parent.parent.parent / "scripts"))
        import adaptive_budget
        ab = adaptive_budget
        frag = ab.load_fragility(None, ids.shape[1])
        top_k = ab.allocate_budgets(
            frag, budget_ref_k, budget_target_cosine,
            budget_min_k, budget_max_k, recent_guard
        )
    
    # Patch model
    from compressed_attention_forward_ppl import AttentionStats
    stats = AttentionStats()
    patch_model(
        model, top_k, recent_guard, quant_bits, stats, collect_stats,
        scorer=scorer_name,
        quant_bits_policy=None,
        full_precision_layers=None,
    )
    
    # Warmup run
    with torch.no_grad():
        _ = model(input_ids=ids, use_cache=False, return_dict=True).logits
    
    # Timed runs
    latencies = []
    logits_list = []
    for _ in range(n_runs):
        reset_memory()
        t0 = time.time()
        with torch.no_grad():
            out = model(input_ids=ids, use_cache=False, return_dict=True).logits
        latencies.append(time.time() - t0)
        logits_list.append(out.detach())
        peak_mem = max(peak_mem, measure_memory_mb())
    
    avg_latency = sum(latencies) / len(latencies)
    tokens_per_sec = ids.shape[1] / avg_latency
    
    # Use last run logits for quality comparison
    comp_logits = logits_list[-1]
    
    return {
        "scorer": scorer_name,
        "quant_bits": quant_bits,
        "top_k": top_k if isinstance(top_k, int) else "adaptive",
        "avg_latency_s": avg_latency,
        "tokens_per_sec": tokens_per_sec,
        "peak_memory_mb": peak_mem,
        "stats": stats.summary(),
        "comp_logits": comp_logits,
    }


def main():
    ap = argparse.ArgumentParser(description="Benchmark per-dim scorer vs others")
    ap.add_argument("--model", default="HuggingFaceTB/SmolLM2-1.7B-Instruct")
    ap.add_argument("--corpus", default="wikitext-2")
    ap.add_argument("--n-tokens", type=int, default=256)
    ap.add_argument("--ppl-frac", type=float, default=0.3)
    ap.add_argument("--device", default="cuda", choices=["cuda", "cpu"])
    ap.add_argument("--n-runs", type=int, default=3)
    ap.add_argument("--output", type=Path, required=True)
    
    # Budget config
    ap.add_argument("--adaptive-budget", action="store_true")
    ap.add_argument("--budget-target-cosine", type=float, default=0.98)
    ap.add_argument("--budget-min-k", type=int, default=64)
    ap.add_argument("--budget-max-k", type=int, default=256)
    ap.add_argument("--budget-ref-k", type=int, default=256)
    ap.add_argument("--recent-guard", type=int, default=16)
    ap.add_argument("--top-k", type=int, default=64)
    
    args = ap.parse_args()
    
    device = args.device if args.device == "cpu" or torch.cuda.is_available() else "cpu"
    print(f"Using device: {device}")
    
    # Load model and tokenizer
    print(f"Loading model: {args.model}")
    tokenizer = AutoTokenizer.from_pretrained(args.model)
    model = AutoModelForCausalLM.from_pretrained(args.model, torch_dtype=torch.float16).to(device)
    model.eval()
    model.config._attn_implementation = "eager"
    
    # Load corpus
    if args.corpus == "wikitext-2":
        ds = load_dataset("Salesforce/wikitext", "wikitext-2-raw-v1", split="test")
        text = "\n\n".join(ds["text"])
    else:
        raise ValueError(args.corpus)
    ids = tokenizer(text, return_tensors="pt").input_ids[:, :args.n_tokens].to(device)
    print(f"Input shape: {ids.shape}")
    
    # Baseline: dense attention
    print("\n=== Baseline: Dense Attention ===")
    reset_memory()
    t0 = time.time()
    with torch.no_grad():
        base_logits = model(input_ids=ids, use_cache=False, return_dict=True).logits.detach()
    baseline_latency = time.time() - t0
    baseline_mem = measure_memory_mb()
    baseline_tokens_sec = ids.shape[1] / baseline_latency
    print(f"Latency: {baseline_latency:.3f}s, Tokens/sec: {baseline_tokens_sec:.1f}, Memory: {baseline_mem:.1f}MB")
    
    base_nll = causal_nll_from_logits(base_logits, ids)
    base_ppl, eval_start, eval_end = windowed_ppl(base_nll, args.ppl_frac)
    
    # Configurations to benchmark
    configs = [
        ("quantized", 4, 64, "Simple 4-bit per-key quantization"),
        ("quantized", 8, 64, "Simple 8-bit per-key quantization"),
        ("per-dim", 4, 64, "Per-dim 4-bit"),
        ("per-dim", 8, 64, "Per-dim 8-bit"),
        ("per-dim", 4, "adaptive", "Per-dim 4-bit + adaptive budget"),
        ("per-dim", 8, "adaptive", "Per-dim 8-bit + adaptive budget"),
    ]
    
    results = []
    
    for scorer_name, quant_bits, top_k_cfg, desc in configs:
        print(f"\n=== {desc} ===")
        print(f"Scorer: {scorer_name}, Bits: {quant_bits}, Top-k: {top_k_cfg}")
        
        # Reload model fresh to avoid nested patching
        print(f"Reloading model...")
        del model
        torch.cuda.empty_cache()
        model = AutoModelForCausalLM.from_pretrained(args.model, torch_dtype=torch.float16).to(device)
        model.eval()
        model.config._attn_implementation = "eager"
        
        adaptive = (top_k_cfg == "adaptive")
        top_k_val = args.top_k if not adaptive else None
        
        try:
            res = run_scorer(
                model, ids, scorer_name, quant_bits, top_k_val, args.recent_guard,
                adaptive, args.budget_target_cosine, args.budget_min_k,
                args.budget_max_k, args.budget_ref_k,
                n_runs=args.n_runs, collect_stats=True,
            )
            
            # Compute quality metrics using logits from run_scorer
            comp_logits = res.pop("comp_logits")
            metrics = logit_metrics(base_logits, comp_logits, ids, eval_start)
            comp_nll = causal_nll_from_logits(comp_logits, ids)
            comp_ppl, _, _ = windowed_ppl(comp_nll, args.ppl_frac)
            delta_pct = (comp_ppl - base_ppl) / base_ppl * 100.0
            
            res.update(metrics)
            res["baseline_ppl"] = base_ppl
            res["compressed_ppl"] = comp_ppl
            res["delta_ppl_pct"] = delta_pct
            
            print(f"Latency: {res['avg_latency_s']:.3f}s, Tokens/sec: {res['tokens_per_sec']:.1f}, Memory: {res['peak_memory_mb']:.1f}MB")
            print(f"PPL: {base_ppl:.2f} → {comp_ppl:.2f} (Δ {delta_pct:+.2f}%)")
            print(f"Logit cosine p05: {metrics['logit_cosine_p05']:.4f}, KL p95: {metrics['kl_p95']:.4f}")
            
            results.append(res)
            
        except Exception as e:
            print(f"FAILED: {e}")
            import traceback
            traceback.print_exc()
            results.append({
                "scorer": scorer_name,
                "quant_bits": quant_bits,
                "top_k": top_k_val,
                "error": str(e),
            })
    
    # Save results
    output = {
        "model": args.model,
        "corpus": args.corpus,
        "n_tokens": args.n_tokens,
        "device": device,
        "baseline": {
            "latency_s": baseline_latency,
            "tokens_per_sec": baseline_tokens_sec,
            "peak_memory_mb": baseline_mem,
            "ppl": base_ppl,
        },
        "results": results,
    }
    
    args.output.parent.mkdir(parents=True, exist_ok=True)
    with open(args.output, "w") as f:
        json.dump(output, f, indent=2)
    print(f"\nResults saved to {args.output}")
    
    # Print summary table
    print("\n" + "="*80)
    print("SUMMARY")
    print("="*80)
    print(f"{'Scorer':<20} {'Bits':<6} {'Top-k':<10} {'Latency':<10} {'Tok/s':<8} {'Mem(MB)':<10} {'ΔPPL%':<8} {'Cos05':<8} {'KL95':<8}")
    print("-"*80)
    print(f"{'Dense (baseline)':<20} {'fp16':<6} {'all':<10} {baseline_latency:<10.3f} {baseline_tokens_sec:<8.1f} {baseline_mem:<10.1f} {'0.00':<8} {'1.0000':<8} {'0.0000':<8}")
    for r in results:
        if "error" in r:
            print(f"{r['scorer']:<20} {r['quant_bits']:<6} {str(r.get('top_k','?')):<10} {'ERROR':<10} {'':<8} {'':<10} {'':<8} {'':<8} {'':<8}")
        else:
            cos05 = r.get("logit_cosine_p05", 0)
            kl95 = r.get("kl_p95", 0)
            print(f"{r['scorer']:<20} {r['quant_bits']:<6} {str(r['top_k']):<10} {r['avg_latency_s']:<10.3f} {r['tokens_per_sec']:<8.1f} {r['peak_memory_mb']:<10.1f} {r['delta_ppl_pct']:<+8.2f} {cos05:<8.4f} {kl95:<8.4f}")


if __name__ == "__main__":
    main()
