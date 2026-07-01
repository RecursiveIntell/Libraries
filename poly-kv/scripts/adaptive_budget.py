#!/usr/bin/env python3
"""Layer-adaptive budget allocation for compressed attention.

Given per-layer fragility data (cosine p05 at a reference top-k),
compute per-layer budgets that minimize total selected keys while
keeping each layer's expected cosine above a target threshold.

Algorithm:
- If a layer's cosine_p05 is already above target at ref_k, reduce its budget
  proportionally to the slack (but never below min_k).
- If a layer's cosine_p05 is below target, increase its budget by extrapolating
  the deficit (assume cosine improves roughly as 1 - c/k^alpha with alpha ~ 0.3).
- Clamp all budgets to [min_k + recent_guard, max_k + recent_guard].
"""
from __future__ import annotations

import json
import math
from pathlib import Path


def allocate_budgets(
    layer_fragility: dict[int, float],
    ref_k: int,
    target_cosine: float = 0.995,
    min_k: int = 32,
    max_k: int = 256,
    recent_guard: int = 16,
) -> dict[int, int]:
    """Return per-layer top-k budgets (including recent_guard).

    Algorithm:
    - If cosine_p05 >= target: layer is safe at ref_k. Reduce budget proportional
      to how far ABOVE target the cosine is. The further above, the more we can cut.
      Use an aggressive exponential decay: scale = max(0.25, (1 - slack)^10)
      so a slack of 0.004 gives scale ~0.96 while 0.02 gives scale ~0.82 while
      0.04 gives scale ~0.66.
    - If cosine_p05 < target: layer is fragile. Increase budget by extrapolating
      the deficit: k = ref_k * (1 + deficit * 5), clamped to max_k.
    """
    budgets: dict[int, int] = {}
    for layer, cos_p05 in layer_fragility.items():
        if cos_p05 >= target_cosine:
            slack = cos_p05 - target_cosine
            # Aggressive exponential reduction for stable layers
            scale = max(0.25, (1.0 - slack) ** 10)
            k = max(min_k, int(ref_k * scale))
        else:
            deficit = target_cosine - cos_p05
            k = min(max_k, int(ref_k * (1.0 + deficit * 5.0)))
        budgets[layer] = k + recent_guard
    return budgets


def allocate_head_budgets(
    head_fragility: dict[tuple[int, int], float],
    ref_k: int,
    target_cosine: float = 0.995,
    min_k: int = 16,
    max_k: int = 256,
    recent_guard: int = 16,
) -> dict[tuple[int, int], int]:
    """Return per-(layer, head) top-k budgets (including recent_guard)."""
    budgets: dict[tuple[int, int], int] = {}
    for (layer, head), cos_p05 in head_fragility.items():
        if cos_p05 >= target_cosine:
            slack = cos_p05 - target_cosine
            scale = max(0.25, (1.0 - slack) ** 10)
            k = max(min_k, int(ref_k * scale))
        else:
            deficit = target_cosine - cos_p05
            k = min(max_k, int(ref_k * (1.0 + deficit * 5.0)))
        budgets[(layer, head)] = k + recent_guard
    return budgets


def fragility_from_receipt(receipt: dict) -> dict[int, float]:
    """Extract per-layer cosine_p05 from a receipt."""
    per_layer = receipt.get("attention", {}).get("per_layer", {})
    return {int(k): v["cosine_p05"] for k, v in per_layer.items() if v.get("cosine_p05") is not None}


def head_fragility_from_receipt(receipt: dict) -> dict[tuple[int, int], float]:
    """Extract per-(layer, head) cosine_p05 from a receipt with per_head data."""
    per_layer = receipt.get("attention", {}).get("per_layer", {})
    result: dict[tuple[int, int], float] = {}
    for layer_str, d in per_layer.items():
        layer = int(layer_str)
        per_head = d.get("per_head", {})
        for head_str, hd in per_head.items():
            head = int(head_str)
            cos = hd.get("cosine_p05")
            if cos is not None:
                result[(layer, head)] = cos
    return result


def compute_expected_mean_k(budgets: dict[int, int] | dict[tuple[int, int], int], seq_len: int) -> float:
    """Average selected keys given budgets and sequence length.

    For per-layer budgets, assumes uniform layer count = 1 (caller should divide).
    For per-(layer, head) budgets, averages across all (layer, head) pairs.
    """
    if not budgets:
        return 0.0
    total = sum(min(v, seq_len) for v in budgets.values())
    return total / len(budgets)


def validate_budgets(budgets: dict[int, int] | dict[tuple[int, int], int], min_k: int, max_k: int) -> bool:
    """Assert all budgets are within [min_k, max_k]."""
    return all(min_k <= v <= max_k for v in budgets.values())


# Built-in fragility defaults from the 256-token top_k=64 diagnostic run
# (per-layer cosine_p05 from sampled attention stats at 256 tokens)
DEFAULT_FRAGILITY_128TOK: dict[int, float] = {
    0: 0.7120, 1: 1.0000, 2: 0.9987, 3: 0.9645,
    4: 0.9683, 5: 0.9881, 6: 0.9957,
    7: 0.9995, 8: 0.9969, 9: 0.9903, 10: 0.9985,
    11: 0.9975, 12: 0.9962, 13: 0.9885, 14: 0.9906,
    15: 0.9965, 16: 0.9920, 17: 0.9974, 18: 0.9988,
    19: 0.9984, 20: 0.9965, 21: 0.9982, 22: 0.9976, 23: 0.9989,
}

# 512-token fragility map (per-layer cosine_p05 at top_k=64)
# Layer 0 collapses to 0.3905 — quantized scoring is nearly broken for it.
# Layers 1-23 are mostly stable (0.93-0.9999).
DEFAULT_FRAGILITY_512TOK: dict[int, float] = {
    0: 0.3905, 1: 0.9996, 2: 0.9962, 3: 0.9868,
    4: 0.9265, 5: 0.9881, 6: 0.9893,
    7: 0.9999, 8: 0.9943, 9: 0.9344, 10: 0.9975,
    11: 0.9950, 12: 0.9784, 13: 0.9701, 14: 0.9949,
    15: 0.9824, 16: 0.9838, 17: 0.9898, 18: 0.9839,
    19: 0.9667, 20: 0.9580, 21: 0.9858, 22: 0.9911, 23: 0.9952,
}


def load_fragility(path: Path | None, n_tokens: int = 256) -> dict[int, float]:
    """Load per-layer fragility from a JSON receipt file, or return defaults.

    Picks the closest fragility map by context length when no file is provided:
    - n_tokens <= 128: 128-token defaults
    - n_tokens <= 256: 256-token defaults
    - n_tokens > 256: 512-token defaults
    """
    if path is not None:
        data = json.loads(path.read_text())
        frag = fragility_from_receipt(data)
        if frag:
            return frag
    if n_tokens <= 128:
        return dict(DEFAULT_FRAGILITY_128TOK)
    elif n_tokens <= 256:
        return dict(DEFAULT_FRAGILITY_128TOK)
    else:
        return dict(DEFAULT_FRAGILITY_512TOK)


if __name__ == "__main__":
    # Quick CLI for testing
    import argparse
    ap = argparse.ArgumentParser()
    ap.add_argument("--fragility-file", type=Path, default=None)
    ap.add_argument("--ref-k", type=int, default=64)
    ap.add_argument("--target-cosine", type=float, default=0.995)
    ap.add_argument("--min-k", type=int, default=32)
    ap.add_argument("--max-k", type=int, default=256)
    ap.add_argument("--recent-guard", type=int, default=16)
    args = ap.parse_args()
    frag = load_fragility(args.fragility_file)
    budgets = allocate_budgets(frag, args.ref_k, args.target_cosine, args.min_k, args.max_k, args.recent_guard)
    for layer in sorted(budgets):
        print(f"  layer {layer:2d}: fragility={frag.get(layer, '?'):.4f}  budget={budgets[layer]}")
    mean_k = compute_expected_mean_k(budgets, 256)
    print(f"\n  expected mean k at 256 tokens: {mean_k:.1f}")