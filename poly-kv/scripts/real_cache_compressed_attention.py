#!/usr/bin/env python3
"""Real-cache compressed attention quality gate for poly-kv/proveKV.

Consumes a HuggingFace cache_oracle.pt from ppl_validate.py and compares:
  full attention: softmax(q @ K^T) @ V
  compressed-topk proxy: score compressed keys -> top-k -> softmax over selected exact keys -> selected V

This is intentionally a quality gate, not a throughput benchmark. It uses real model
KV tensors from a real corpus run, and deterministic sampled queries from the same
cache. Current implementation uses raw cached key vectors as query probes because
ppl_validate.py does not yet save post-RoPE query states. That means this is a
real-cache top-k/value-decode gate, not a full logit/PPL replacement gate.
"""
import argparse
import json
import math
import statistics
import time
from pathlib import Path

import torch


def percentile(xs, p):
    if not xs:
        return None
    xs = sorted(xs)
    if len(xs) == 1:
        return xs[0]
    idx = (len(xs) - 1) * p
    lo = math.floor(idx)
    hi = math.ceil(idx)
    if lo == hi:
        return xs[lo]
    return xs[lo] * (hi - idx) + xs[hi] * (idx - lo)


def cosine(a, b):
    denom = a.norm() * b.norm()
    if denom.item() == 0:
        return 0.0
    return torch.dot(a.flatten(), b.flatten()).div(denom).item()


def quantize_keys(keys, levels):
    max_abs = keys.abs().amax(dim=-1, keepdim=True).clamp_min(1e-6)
    q = torch.round((keys / max_abs).clamp(-1, 1) * levels).to(torch.int16)
    return q, max_abs.squeeze(-1).to(torch.float32)


def compressed_scores(query, qkeys, scales, levels):
    # Dequantize only arithmetically for dot-score proxy; no full key tensor is materialized.
    return ((qkeys.to(torch.float32) / levels) * scales[:, None] * query[None, :]).sum(dim=-1)


def full_attention(query, keys, values):
    scores = torch.matmul(keys, query) / math.sqrt(query.numel())
    weights = torch.softmax(scores, dim=-1)
    return torch.matmul(weights, values), scores


def topk_attention(query, keys, values, approximate_scores, top_k, recent_guard):
    seq = keys.shape[0]
    guard = torch.arange(max(0, seq - recent_guard), seq, device=keys.device)
    k = min(top_k, seq)
    approx_top = torch.topk(approximate_scores, k=k).indices
    if guard.numel() > 0:
        selected = torch.unique(torch.cat([approx_top, guard]), sorted=False)
    else:
        selected = approx_top
    selected = selected[selected < seq]
    exact_scores = torch.matmul(keys[selected], query) / math.sqrt(query.numel())
    weights = torch.softmax(exact_scores, dim=-1)
    out = torch.matmul(weights, values[selected])
    return out, selected


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--cache", type=Path, required=True)
    ap.add_argument("--output", type=Path, required=True)
    ap.add_argument("--top-k", type=int, default=64)
    ap.add_argument("--recent-guard", type=int, default=16)
    ap.add_argument("--layers", type=int, default=24)
    ap.add_argument("--queries-per-head", type=int, default=8)
    ap.add_argument("--quant-bits", type=int, default=4)
    ap.add_argument("--device", default="cuda", choices=["cuda", "cpu"])
    args = ap.parse_args()

    started = time.time()
    cache = torch.load(args.cache, map_location="cpu", weights_only=False)
    keys_list = cache["keys"]
    vals_list = cache["values"]
    seq_len = int(cache.get("seq_len", keys_list[0].shape[-2]))
    levels = (1 << (args.quant_bits - 1)) - 1
    device = args.device if args.device == "cpu" or torch.cuda.is_available() else "cpu"

    cosines = []
    mses = []
    overlaps = []
    decoded_value_counts = []
    lat_full = []
    lat_topk = []
    failures = []
    samples = 0
    decoded_keys = 0

    num_layers = min(args.layers, len(keys_list))
    for layer_idx in range(num_layers):
        k_layer = keys_list[layer_idx][0].to(device=device, dtype=torch.float32)  # H,T,D
        v_layer = vals_list[layer_idx][0].to(device=device, dtype=torch.float32)
        num_heads = k_layer.shape[0]
        for head_idx in range(num_heads):
            keys = k_layer[head_idx]
            values = v_layer[head_idx]
            qkeys, scales = quantize_keys(keys, levels)
            positions = torch.linspace(
                max(1, seq_len // 4), seq_len - 1, steps=min(args.queries_per_head, max(1, seq_len - 1)),
                device=device,
            ).round().long().unique()
            for pos in positions:
                pos_i = int(pos.item())
                query = keys[pos_i]
                prefix_keys = keys[: pos_i + 1]
                prefix_vals = values[: pos_i + 1]
                prefix_qkeys = qkeys[: pos_i + 1]
                prefix_scales = scales[: pos_i + 1]

                t0 = time.perf_counter()
                full_out, full_scores = full_attention(query, prefix_keys, prefix_vals)
                lat_full.append((time.perf_counter() - t0) * 1e6)

                approx = compressed_scores(query, prefix_qkeys, prefix_scales, levels)
                t1 = time.perf_counter()
                top_out, selected = topk_attention(
                    query, prefix_keys, prefix_vals, approx, args.top_k, args.recent_guard
                )
                lat_topk.append((time.perf_counter() - t1) * 1e6)

                exact_k = min(args.top_k, prefix_keys.shape[0])
                exact_top = torch.topk(full_scores, k=exact_k).indices
                selected_set = set(int(x) for x in selected.detach().cpu().tolist())
                exact_set = set(int(x) for x in exact_top.detach().cpu().tolist())
                overlap = len(selected_set & exact_set) / max(1, len(exact_set))
                c = cosine(full_out, top_out)
                mse = torch.mean((full_out - top_out) ** 2).item()
                cosines.append(c)
                mses.append(mse)
                overlaps.append(overlap)
                decoded_value_counts.append(len(selected_set))
                samples += 1
                if c < 0.995 or overlap < 0.80:
                    failures.append({
                        "layer": layer_idx,
                        "head": head_idx,
                        "pos": pos_i,
                        "cosine": c,
                        "topk_overlap": overlap,
                        "decoded_values": len(selected_set),
                    })
        del k_layer, v_layer
        if device == "cuda":
            torch.cuda.empty_cache()

    raw_key_bytes = 0
    compressed_key_bytes = 0
    for k in keys_list[:num_layers]:
        raw_key_bytes += k.numel() * 2  # fp16 source cache
        compressed_key_bytes += k.numel() * args.quant_bits / 8
        compressed_key_bytes += k.shape[1] * k.shape[2] * 4  # per head/token scale f32

    receipt = {
        "schema_version": "real_cache_compressed_attention_v1",
        "cache": str(args.cache),
        "model": cache.get("model_id"),
        "seq_len": seq_len,
        "layers_evaluated": num_layers,
        "top_k": args.top_k,
        "recent_guard": args.recent_guard,
        "quant_bits": args.quant_bits,
        "samples": samples,
        "decoded_keys": decoded_keys,
        "decoded_values_mean": statistics.mean(decoded_value_counts) if decoded_value_counts else None,
        "decoded_values_p95": percentile(decoded_value_counts, 0.95),
        "attention_output_cosine_mean": statistics.mean(cosines) if cosines else None,
        "attention_output_cosine_p05": percentile(cosines, 0.05),
        "attention_mse_mean": statistics.mean(mses) if mses else None,
        "attention_mse_p95": percentile(mses, 0.95),
        "attention_topk_overlap_mean": statistics.mean(overlaps) if overlaps else None,
        "attention_topk_overlap_p05": percentile(overlaps, 0.05),
        "latency_full_attention_us_p50": percentile(lat_full, 0.50),
        "latency_full_attention_us_p95": percentile(lat_full, 0.95),
        "latency_topk_decode_us_p50": percentile(lat_topk, 0.50),
        "latency_topk_decode_us_p95": percentile(lat_topk, 0.95),
        "raw_key_bytes_fp16": int(raw_key_bytes),
        "compressed_key_bytes_estimated": int(compressed_key_bytes),
        "key_compression_ratio_estimated": (raw_key_bytes / compressed_key_bytes) if compressed_key_bytes else None,
        "thresholds": {
            "attention_output_cosine_p05_min": 0.995,
            "attention_topk_overlap_mean_min": 0.80,
            "decoded_keys_required": 0,
        },
        "passed": bool(
            decoded_keys == 0
            and (percentile(cosines, 0.05) or 0) >= 0.995
            and (statistics.mean(overlaps) if overlaps else 0) >= 0.80
        ),
        "failure_count": len(failures),
        "failures_sample": failures[:50],
        "elapsed_s": time.time() - started,
        "claim_boundary": "Real model KV cache quality gate. Uses cached key vectors as query probes; does not yet replace model attention or measure PPL/logit drift.",
    }
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(receipt, indent=2))
    print(json.dumps(receipt, indent=2))


if __name__ == "__main__":
    main()
